# Headless clock test. No connection to the rig and no Minecraft initialization.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$testBuild = Join-Path $Root 'bot\build\clock-tests'
$null = New-Item -ItemType Directory -Force $testBuild
$classpath = (Get-GameClasspath) -join ';'
& "$Jdk\javac.exe" --release 25 -proc:none -cp "$Root\bot\build\pathtracing.jar;$classpath" -d $testBuild "$PSScriptRoot\ClockTests.java"
if ($LASTEXITCODE) { throw 'Clock test compilation failed' }
& "$Jdk\java.exe" -cp "$testBuild;$Root\bot\build\pathtracing.jar;$classpath" pathtracing.ClockTests `
  "$Root\bot\build\pathtracing.jar" "$Libraries\com\mojang\minecraft\$McVersion\minecraft-$McVersion-client.jar"
if ($LASTEXITCODE) { throw 'Clock tests failed' }

# Load only pure gate functions. Executing the gate itself would start a client.
$tokens = $null
$parseErrors = $null
$gate = Join-Path $Root 'lab\gates\clock-gate.ps1'
$ast = [Management.Automation.Language.Parser]::ParseFile($gate, [ref]$tokens, [ref]$parseErrors)
foreach ($definition in $ast.FindAll({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] }, $false)) {
  if ($definition.Name -in @('Measure-Clocks', 'Read-Progress', 'Distribution', 'New-ClockInput', 'Compare-ClockRecording')) {
    $gateDirectory = "'" + (Split-Path -Parent $gate).Replace("'", "''") + "'"
    . ([scriptblock]::Create($definition.Extent.Text.Replace('$PSScriptRoot', $gateDirectory)))
  }
}
function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Samples([int[]]$ClientTicks) {
  for ($i = 0; $i -lt $ClientTicks.Count; $i++) {
    [pscustomobject]@{ Server = 100 * $i; ClientBefore = $ClientTicks[$i]; ClientAfter = $ClientTicks[$i] + 2
      BeforeSeconds = [double]$i; AfterSeconds = $i + 0.02 }
  }
}
$fast = Measure-Clocks @(Samples @(1000, 1100, 1200))
Assert $fast.Lockstep 'Equal-rate samples must be lockstep'
Assert ($fast.RatioMin -lt 1 -and $fast.RatioMax -gt 1) 'Measured ratio must bracket one'
$slow = Measure-Clocks @(Samples @(1000, 1020, 1040))
Assert (-not $slow.Lockstep) 'The property-off rate must not be called lockstep'
Assert ($slow.ClientTpsMin -lt 20 -and $slow.ClientTpsMax -gt 20) 'Off control must bracket 20 TPS'
$drift = Measure-Clocks @(Samples @(1000, 1090, 1200))
Assert (-not $drift.Lockstep) 'Intermediate drift must be detected even when endpoints agree'
$rejected = $false
try { Measure-Clocks @(Samples @(1000, 1100)) | Out-Null } catch { $rejected = $true }
Assert $rejected 'Insufficient sampling must refuse a result'
$rejected = $false
$wrapped = @(Samples @(1000, 1100, 1200))
$wrapped[2].Server = 1
try { Measure-Clocks $wrapped | Out-Null } catch { $rejected = $true }
Assert $rejected 'Wrapped game time must refuse a result'
Assert ((Distribution @(1, 3, 2)) -eq 'n=3 min=1 median=2 p90=3 p99=3 max=3') 'Distribution values'
$progress = Join-Path $testBuild 'progress.jsonl'
[IO.File]::WriteAllText("$progress.tmp", "{`"type`":`"recording`"}`n{`"t`":0,`"tickCount`":50}`n{`"t`":1")
Assert ((Read-Progress $progress).tickCount -eq 50) 'A partial last write must not become a sample'
[IO.File]::WriteAllText($progress, "{`"t`":0,`"tickCount`":50}`n{`"type`":`"end`"}`n")
Assert ((Read-Progress $progress).tickCount -eq 50) 'Published recording progress'

$botDir = Join-Path $testBuild 'input-rig'
$prefix = 'offline'
$ticks = 800
function Save-Artifact($Path) { }
foreach ($name in @('full-blocks', 'ice', 'edges')) {
  $inputName = New-ClockInput ([pscustomobject]@{ Name = $name; Start = '0.5,-60,0.5,0,0' })
  $inputRows = @(Get-Content (Join-Path $botDir "pathtracing\inputs\$inputName.jsonl") | ForEach-Object { $_ | ConvertFrom-Json })
  Assert ($inputRows.Count -eq 801) 'Input length must be frozen'
  Assert ($inputRows[1].forward -and -not $inputRows[601].forward) 'Movement and coasting input phases'
  if ($name -eq 'edges') {
    Assert ($inputRows[1].sneak -and $inputRows[1].left -and -not $inputRows[1].jump) 'Sneak edge controls'
  } else {
    Assert ($inputRows[1].sprint -and $inputRows[1].jump -and $inputRows[51].yaw -eq 180) 'Sprint-jump controls'
  }
}
$result = Join-Path $testBuild 'comparison'
$null = New-Item -ItemType Directory -Force $result
$baseline = Join-Path $testBuild 'base.jsonl'
$other = Join-Path $testBuild 'other.jsonl'
$header = '{"type":"recording","toggleSprint":false,"toggleCrouch":false,"autoJump":false,"sprintWindow":0}'
$row = '{"t":0,"tickCount":1,"gameTime":100,"position":{"x":0.0,"xBits":"0000000000000000"}}'
$end = '{"type":"end","ticks":1,"complete":true}'
[IO.File]::WriteAllLines($baseline, @($header, $row, $end))
[IO.File]::WriteAllLines($other, @($header, $row.Replace('"tickCount":1', '"tickCount":50').Replace('"gameTime":100', '"gameTime":200'), $end))
function Invoke-Script($Path, $Parameters) {
  $output = & "$PSHOME\powershell.exe" -NoProfile -File "$Root\lab\gates\compare-recordings.ps1" -A $Parameters.A -B $Parameters.B
  [pscustomobject]@{ Code = $LASTEXITCODE; Text = $output -join "`n" }
}
Assert ((Compare-ClockRecording $baseline $other).Code -eq 0) 'Only absolute counters may differ'
[IO.File]::WriteAllLines($other, @($header, $row.Replace('0000000000000000', '0000000000000001'), $end))
Assert ((Compare-ClockRecording $baseline $other).Code -ne 0) 'A one-bit movement difference must fail'
Remove-Item Function:\Invoke-Script, Function:\Save-Artifact

# Exercise the real launcher with Start-Process replaced, including Julien's client.
$savedRig = $env:PATHTRACING_ROOT
try {
  $env:PATHTRACING_ROOT = Join-Path $testBuild 'launcher-rig'
  foreach ($role in @('player', 'bot')) { $null = New-Item -ItemType Directory -Force (Join-Path $env:PATHTRACING_ROOT "clients\$role\mods") }
  function Start-Process { param($FilePath, $ArgumentList, $WorkingDirectory, [switch]$PassThru) }
  & "$Root\scripts\launch-client.ps1" -Role bot
  $argsOn = Get-Content (Join-Path $env:PATHTRACING_ROOT 'clients\bot\launch.args')
  Assert ('"-Dpathtracing.followTickRate=true"' -cin $argsOn) 'Default bot clock property'
  & "$Root\scripts\launch-client.ps1" -Role bot -FollowTickRate false
  $argsOff = Get-Content (Join-Path $env:PATHTRACING_ROOT 'clients\bot\launch.args')
  Assert ('"-Dpathtracing.followTickRate=false"' -cin $argsOff) 'Property-off launcher control'
  & "$Root\scripts\launch-client.ps1" -Role player
  $argsPlayer = Get-Content (Join-Path $env:PATHTRACING_ROOT 'clients\player\launch.args')
  Assert (@($argsPlayer | Where-Object { $_ -match 'followTickRate' -or $_ -eq '""' }).Count -eq 0) 'Player launch arguments'
} finally {
  Remove-Item Function:\Start-Process
  $env:PATHTRACING_ROOT = $savedRig
}
Write-Host 'Gate measurement, partial-write, input, comparison and launcher checks passed.'
& "$Root\lab\gates\tests\test-clock-gate.ps1"
foreach ($script in @('scripts\launch-client.ps1', 'lab\gates\clock-gate.ps1', 'bot\tests\test-clock.ps1', 'lab\gates\tests\test-clock-gate.ps1')) {
  $tokens = $null
  $parseErrors = $null
  $path = Join-Path $Root $script
  [void][Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$parseErrors)
  if ($parseErrors.Count) { throw "PowerShell parse errors in $script -- $parseErrors" }
  if (@([IO.File]::ReadAllBytes($path) | Where-Object { $_ -gt 127 }).Count) { throw "Non-ASCII script $script" }
}
Write-Host 'PowerShell 5.1 parsing and ASCII checks passed.'
