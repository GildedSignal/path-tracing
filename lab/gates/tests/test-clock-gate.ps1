# Run the actual gate driver with every rig/process operation replaced by an offline double.
$ErrorActionPreference = 'Stop'
$gate = Join-Path $PSScriptRoot '..\clock-gate.ps1'
$tokens = $null
$parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($gate, [ref]$tokens, [ref]$parseErrors)
$driver = @($ast.EndBlock.Statements | Where-Object { $_ -is [Management.Automation.Language.TryStatementAst] })
if ($driver.Count -ne 1) { throw 'Expected one top-level gate driver.' }
$driver = [scriptblock]::Create($driver[0].Extent.Text)
$distribution = $ast.FindAll({ param($node)
  $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Distribution'
}, $false)[0]
. ([scriptblock]::Create($distribution.Extent.Text))
$build = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\..\bot\build\clock-gate-tests'))
$null = New-Item -ItemType Directory -Force $build

function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Enter-Rig($Holder) {
  if ($scenario -eq 'locked') { throw 'Rig held by another job.' }
  $events.Add('lock')
}
function Exit-Rig { $events.Add('unlock') }
function Assert-Absent { }
function Players($Reply) { @() }
function Bot-Processes { @() }
function Require($Reply, $Action) { if ($Reply.Code -ne 0) { throw "$Action failed" }; $Reply }
function Diagnostic($Reply) { $Reply.Text }
function Note($Text) { $notes.Add($Text) }
function Rcon([string[]]$Text) {
  foreach ($command in $Text) {
    $events.Add($command)
    if ($scenario -eq 'restore-failure' -and $command -eq 'tick rate 20' -and $runs.Count -gt 0) {
      return [pscustomobject]@{ Code = 1; Text = 'Injected restore failure' }
    }
  }
  [pscustomobject]@{ Code = 0; Text = '' }
}
function Get-FixtureCourses {
  foreach ($name in @('full-blocks', 'ice', 'edges')) { [pscustomobject]@{ Name = $name } }
}
function Start-ClockBot($Follow) {
  if ($scenario -eq 'launch-failure') { $events.Add('tick rate 100'); throw 'Injected launch failure' }
}
function Stop-ClockBot { $events.Add('stop-owned-bot') }
function Build-ClockCourse($Course) { }
function New-ClockInput($Course) { 'fake-input' }
function Compare-ClockRecording($Baseline, $Recording) {
  $different = $scenario -eq 'mismatch' -or ($scenario -eq 'limit' -and $Recording -match '200$')
  [pscustomobject]@{ Code = [int]$different; Text = 'comparison'; File = 'fake-comparison.log' }
}
function Replay-Clock($Course, $InputName, $Rate, $Follow) {
  $limit = $scenario -eq 'limit' -and $Rate -eq 200
  $off = $Follow -eq 'false' -and $scenario -ne 'off-failure'
  $clientRate = if ($off) { 20 } else { $Rate }
  $clock = [pscustomobject]@{ Lockstep = -not ($limit -or $off); Samples = 8; ServerTicks = 800
    ClientTicksMin = 798; ClientTicksMax = 802; RatioMin = 0.99; RatioMax = 1.01
    ClientTpsMin = $clientRate - 1; ClientTpsMax = $clientRate + 1
    ServerTpsMin = $Rate - 1; ServerTpsMax = $Rate + 1 }
  $corrections = [int]($scenario -eq 'correction')
  if ($corrections) { $failures.Add('Injected server correction.') }
  $run = [pscustomobject]@{ Course = $Course.Name; Rate = $Rate; Follow = $Follow
    Path = "$($Course.Name)-$Rate"; Clock = $clock; Corrections = $corrections; Identical = $null; Comparison = '' }
  $runs.Add($run)
  $run
}

foreach ($scenario in @('pass', 'limit', 'mismatch', 'correction', 'off-failure', 'launch-failure', 'restore-failure', 'locked')) {
  $result = Join-Path $build ($scenario + '-' + [Guid]::NewGuid().ToString('N'))
  $Rig = $result
  $botDir = Join-Path $Rig 'clients\bot'
  $null = New-Item -ItemType Directory -Force (Join-Path $botDir 'mods')
  @('toggleSprint:false', 'toggleCrouch:false', 'autoJump:false', 'sprintWindow:0') |
    Set-Content -LiteralPath (Join-Path $botDir 'options.txt') -Encoding ASCII
  $Jar = Join-Path $result 'fake.jar'
  Set-Content -LiteralPath $Jar -Value 'offline-double' -Encoding ASCII
  $events = [Collections.Generic.List[string]]::new()
  $runs = [Collections.Generic.List[object]]::new()
  $notes = [Collections.Generic.List[string]]::new()
  $failures = [Collections.Generic.List[string]]::new()
  $started = @{}
  $locked = $false
  $exitCode = 1
  $BotName = 'PathBot'
  $prefix = 'offline'
  $watch = [Diagnostics.Stopwatch]::StartNew()
  . $driver
  $expected = if ($scenario -in @('pass', 'limit')) { 0 } else { 1 }
  Assert ($exitCode -eq $expected) "Unexpected verdict for $scenario"
  if ($scenario -eq 'locked') {
    Assert ($events.Count -eq 0) 'A refused lock must leave the rig untouched'
  } else {
    Assert ($events[-3] -eq 'tick rate 20') "Missing final rate restoration for $scenario"
    Assert ($events[-2] -eq 'stop-owned-bot' -and $events[-1] -eq 'unlock') "Cleanup order for $scenario"
  }
  if ($scenario -eq 'limit') { Assert ($highest -eq 100) 'Highest lockstep rate must exclude the limit' }
  if ($scenario -eq 'pass') { Assert ($highest -eq 200) 'Highest lockstep rate must include every course' }
}
Write-Host 'Clock gate driver checks passed for success, limits, mismatches, corrections, off control, launch failure, restore failure and lock refusal.'
