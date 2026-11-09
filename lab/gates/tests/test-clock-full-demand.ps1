# Exercise the frozen input, evidence and verdict helpers without taking the rig lock.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\..\scripts\common.ps1"
$gateDir = Join-Path $Root 'lab\gates'
foreach ($source in @('clock-gate.ps1', 'clock-full-demand.ps1')) {
  $tokens = $null
  $errors = $null
  $ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $gateDir $source), [ref]$tokens, [ref]$errors)
  if ($errors.Count) { throw $errors }
  foreach ($definition in $ast.EndBlock.Statements) {
    if ($definition -is [Management.Automation.Language.FunctionDefinitionAst] -and
        $definition.Name -in @('New-ClockInput', 'Measure-Clocks', 'New-SustainedInput', 'New-Run', 'Read-Audit', 'Check-FullReplay', 'Rate-Status')) {
      . ([scriptblock]::Create($definition.Extent.Text.Replace('$PSScriptRoot', "'" + $gateDir + "'")))
    }
  }
}
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Save-Artifact([string]$Path) {}
function Invoke-Script($Path, $Arguments, $Timeout) {
  $stdout = if ($Arguments.ContainsKey('File')) {
    & "$PSHOME\powershell.exe" -NoProfile -File $Path -File $Arguments.File
  } else {
    & "$PSHOME\powershell.exe" -NoProfile -File $Path -A $Arguments.A -B $Arguments.B
  }
  [pscustomobject]@{ Code = $LASTEXITCODE; Text = $stdout -join "`n"; File = $Path }
}
function Diagnostic($Check) { $Check.Text }
$result = Join-Path $Root 'bot\build\full-demand-gate-test'
$null = New-Item -ItemType Directory -Force $result
$botDir = $result
$ticks = 800
$prefix = 'test'
$course = [pscustomobject]@{ Name = 'full-blocks'; Start = '0.5,-60,0.5,0,0' }
$sustained = New-Run $course.Name 2000 12000
$name = New-ClockInput $course
$extended = New-SustainedInput $name
$baseInput = [IO.File]::ReadAllLines("$result\pathtracing\inputs\$name.jsonl")
$longInput = [IO.File]::ReadAllLines("$result\pathtracing\inputs\$extended.jsonl")
Assert ($longInput.Length -eq 12001) 'Sustained input length'
for ($i = 0; $i -lt $baseInput.Length; $i++) {
  Assert ($baseInput[$i] -ceq $longInput[$i]) "Input prefix mismatch at line $i"
}
Assert ((($longInput[-1] | ConvertFrom-Json).t) -eq 11999) 'Last idle tick index'

function Write-Sample($Run, [int]$ClientStep, [int]$ServerStep, [bool]$Mutant) {
  $Run.Recording = "rows-$($Run.ExpectedRows)-$($Run.Rate).jsonl"
  $Run.ReplaySeconds = $Run.ExpectedRows / $Run.Rate
  $path = Join-Path $result $Run.Recording
  $writer = [IO.StreamWriter]::new($path, $false)
  try {
    $writer.WriteLine('{"type":"recording","toggleSprint":false,"toggleCrouch":false,"autoJump":false,"sprintWindow":0}')
    for ($i = 0; $i -lt $Run.ExpectedRows; $i++) {
      $clock = $Run.Rate + $i
      $bits = if ($Mutant -and $i -eq 799) { '0000000000000001' } else { '0000000000000000' }
      $writer.WriteLine('{"t":' + $i + ',"tickCount":' + $clock + ',"gameTime":' + $clock +
        ',"position":{"x":0.0,"xBits":"' + $bits + '"}}')
    }
    $writer.WriteLine('{"type":"end","ticks":' + $Run.ExpectedRows + ',"complete":true}')
  } finally { $writer.Dispose() }
  $samples = @(for ($i = 0; $i -lt 3; $i++) {
    [pscustomobject]@{ Server = 10 + $i * $ServerStep; ClientBefore = 20 + $i * $ClientStep
      ClientAfter = 20 + $i * $ClientStep; BeforeSeconds = $i; AfterSeconds = $i + 0.01 }
  })
  ConvertTo-Json -InputObject $samples | Set-Content ([IO.Path]::ChangeExtension($path, '.clocks.json')) -Encoding ASCII
}
$reference = New-Run $course.Name 20 800
Write-Sample $reference 20 20 $false
Assert (@(Check-FullReplay $reference $null $false).Count -eq 0) '20 TPS reference'
$reference.Status = 'qualified'
$stress = New-Run $course.Name 4000 800
Write-Sample $stress 2000 4000 $false
Assert (@(Check-FullReplay $stress $reference $false).Count -eq 0) 'Separate clocks must preserve movement qualification'
Assert (-not $stress.Clock.Lockstep) 'Old offset test must report drift'
Assert ((Rate-Status $stress.Clock.ClientTpsMin $stress.Clock.ClientTpsMax 4000) -ceq 'below target') 'Rate shortfall must be visible'
Write-Sample $stress 2000 4000 $true
Assert (@(Check-FullReplay $stress $reference $false).Count -gt 0 -and -not $stress.Identical) 'One bit at tick 799 must fail'

Write-Sample $sustained 2000 2000 $false
$log = Join-Path $result 'client.log'
$auditName = [IO.Path]::GetFileNameWithoutExtension($sustained.Recording)
Set-Content -LiteralPath $log "Clock audit $auditName rows=12000 complete=true activeNanos=6000000000 frames=500 scheduled=12015 executed=12015 dropped=0 startMillis=1000"
$sustained.Audit = Read-Audit $log $sustained.Recording
Assert (@(Check-FullReplay $sustained $reference $true).Count -eq 0) 'Sustained validation and 800-row comparison'
Assert ($sustained.Audit.ActiveTps -eq 2000 -and $sustained.EndToEndTps -eq 2000) 'Measured TPS'
Assert ($sustained.ClockWindows.Count -eq 1 -and $sustained.ClockWindows[0].ClientTpsMin -lt 2000) 'Sustained window bounds'
$sustained.Audit.Scheduled++
$sustained.Audit.Dropped++
Assert ((@(Check-FullReplay $sustained $reference $true) -join ' ') -match 'Dropped 1 ticks') 'A single lost tick must be reported'

$savedRig = $env:PATHTRACING_ROOT
try {
  $env:PATHTRACING_ROOT = Join-Path $result 'launcher-rig'
  foreach ($role in @('bot', 'player')) { $null = New-Item -ItemType Directory -Force "$env:PATHTRACING_ROOT\clients\$role\mods" }
  function Start-Process { param($FilePath, $ArgumentList, $WorkingDirectory, [switch]$PassThru) }
  & "$Root\scripts\launch-client.ps1" -Role bot
  Assert (-not (Select-String -Path "$env:PATHTRACING_ROOT\clients\bot\launch.args" -Pattern 'fullTickDemand|clockAudit')) 'Default launcher keeps lab flags off'
  & "$Root\scripts\launch-client.ps1" -Role bot -FullTickDemand true -ClockAudit true
  Assert (@(Select-String -Path "$env:PATHTRACING_ROOT\clients\bot\launch.args" -Pattern 'fullTickDemand=true|clockAudit=true').Count -eq 2) 'Opt-in bot flags'
  & "$Root\scripts\launch-client.ps1" -Role player -FullTickDemand true -ClockAudit true
  Assert (-not (Select-String -Path "$env:PATHTRACING_ROOT\clients\player\launch.args" -Pattern 'fullTickDemand|clockAudit')) 'Player launcher omits lab flags'
} finally {
  Remove-Item Function:\Start-Process
  $env:PATHTRACING_ROOT = $savedRig
}
Write-Output '1.8c offline gate checks passed.'
