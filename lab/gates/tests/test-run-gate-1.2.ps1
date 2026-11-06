# Offline integration tests. A copied driver sees only fake common/RCON/launch scripts and a fake rig.
# Real generators/comparators/checkers remain in use. No game or live rig command is invoked.
param([string[]]$Case)
$ErrorActionPreference = 'Stop'
$Case = @($Case | ForEach-Object { $_.Split(',') })
$caseCount = 0
$gateDir = Split-Path -Parent $PSScriptRoot
$sandbox = Join-Path $gateDir ('results\stub-tests-' + [Guid]::NewGuid().ToString('N'))
$fakeRepo = Join-Path $sandbox 'repo'
$fakeGates = Join-Path $fakeRepo 'lab\gates'
$fakeScripts = Join-Path $fakeRepo 'scripts'
$fakeRig = Join-Path $sandbox 'rig with spaces'
$shell = Join-Path $PSHOME 'powershell.exe'
$null = New-Item -ItemType Directory -Force $fakeGates, $fakeScripts, $fakeRig
# Shorten only readiness deadlines in the test copy; retain real child-command timeouts.
$driver = Get-Content -LiteralPath (Join-Path $gateDir 'run-gate-1.2.ps1') -Raw
$driver = $driver.Replace('Elapsed.TotalSeconds -lt 120', 'Elapsed.TotalSeconds -lt 1')
$driver = $driver.Replace('Elapsed.TotalSeconds -lt 180', 'Elapsed.TotalSeconds -lt 1')
Set-Content -LiteralPath (Join-Path $fakeGates 'run-gate-1.2.ps1') -Value $driver -Encoding ASCII
foreach ($file in @('new-inputs.ps1', 'compare-recordings.ps1', 'check-recording.ps1', 'check-dump.ps1')) {
  Copy-Item -LiteralPath (Join-Path $gateDir $file) -Destination $fakeGates
}

$common = @'
$Rig = $env:GATE_TEST_RIG
$ServerHost = '127.0.0.1'
$ServerPort = 25565
$RconPort = 25575
$PlayerName = 'Brain_Forge'
$BotName = 'PathBot'
function Read-State { Get-Content -LiteralPath (Join-Path $Rig 'state.json') -Raw | ConvertFrom-Json }
function Write-State($State) { $State | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Rig 'state.json') -Encoding ASCII }
function Enter-Rig($Holder) {
  if ($env:GATE_TEST_CASE -eq 'locked') { throw 'Rig is held by stub owner.' }
  Set-Content -LiteralPath (Join-Path $Rig 'rig.lock') -Value $Holder
  Add-Content -LiteralPath (Join-Path $Rig 'events.txt') 'lock'
}
function Exit-Rig {
  Add-Content -LiteralPath (Join-Path $Rig 'events.txt') 'unlock'
  Remove-Item -LiteralPath (Join-Path $Rig 'rig.lock')
}
function Test-Port($HostName, $Port) {
  $state = Read-State
  if ($env:GATE_TEST_CASE -eq 'server-timeout') { return $false }
  [bool]$state.server
}
# The driver uses CIM only to discover existing bot processes. This fake never queries the OS.
function Get-CimInstance {
  param($ClassName, $Filter)
  if ($env:GATE_TEST_CASE -in @('running-different', 'running-same', 'running-offline')) {
    [pscustomobject]@{ CommandLine = 'javaw.exe @"' + (Join-Path $Rig 'clients\bot\launch.args') + '"' }
  }
}
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'common.ps1') -Value $common -Encoding ASCII
$rcon = @'
param([string[]]$Commands)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$state = Read-State
Add-Content -LiteralPath (Join-Path $Rig 'events.txt') $Commands
if ($env:GATE_TEST_CASE -eq 'rcon-hang') { Start-Sleep -Seconds 60 }
if ($Commands -eq 'list') {
  if ($env:GATE_TEST_CASE -eq 'bad-list') { 'not a player list'; exit 0 }
  $names = @()
  if ($state.bot) { $names += 'PathBot' }
  if ($env:GATE_TEST_CASE -eq 'owner') { $names += 'Brain_Forge' }
  if ($env:GATE_TEST_CASE -eq 'similar-name') { $names += 'Brain_Forge2' }
  if ($env:GATE_TEST_CASE -eq 'owner-later' -and $state.pinged) { $names += 'Brain_Forge' }
  "There are $($names.Count) of a max of 4 players online: $($names -join ', ')"
} elseif ($Commands -like 'tp *8.5*') {
  $path = Join-Path $Rig "clients\bot\pathtracing\recordings\$($state.recording).jsonl.tmp"
  if (Test-Path -LiteralPath $path) {
    for ($t = 5; $t -lt 15; $t++) {
      $row = [ordered]@{ t = $t; tickCount = 100 + $t; position = @{ x = 8.5; xBits = '4021000000000000' } }
      if ($t -eq 6 -and $env:GATE_TEST_CASE -ne 'no-negative-anomaly') {
        $row.anomaly = @(@{ type = 'ClientboundPlayerPositionPacket'; position = @{ x = 8.5; y = -60; z = 0.5 } })
      }
      if ($env:GATE_TEST_CASE -eq 'unrelated-anomaly' -and $t -eq 6) { $row.anomaly[0].type = 'OtherPacket' }
      $row | ConvertTo-Json -Compress -Depth 6 | Add-Content -LiteralPath $path -Encoding ASCII
    }
    $child = Start-Process (Join-Path $PSHOME 'powershell.exe') -ArgumentList @(
      '-NoProfile', '-File', ('"' + (Join-Path $PSScriptRoot 'tick-writer.ps1') + '"')
    ) -PassThru -WindowStyle Hidden
    $state.writerPid = $child.Id
    Write-State $state
    Add-Content -LiteralPath (Join-Path $Rig 'owned-pids.txt') $child.Id
  }
  'Teleported PathBot to 8.5, -60.0, 0.5'
} elseif ($Commands -eq 'stop') {
  'Stopping the server'
  if ($state.serverPid) { Stop-Process -Id $state.serverPid -Force -ErrorAction SilentlyContinue }
} else { 'Command succeeded' }
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'rcon.ps1') -Value $rcon -Encoding ASCII
$tickWriter = @'
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$state = Read-State
$path = Join-Path $Rig "clients\bot\pathtracing\recordings\$($state.recording).jsonl.tmp"
for ($t = 15; $t -lt 500; $t++) {
  if (Test-Path -LiteralPath (Join-Path $Rig 'stop-writer')) { break }
  $row = @{ t = $t; tickCount = 100 + $t; position = @{ x = 8.5; xBits = '4021000000000000' } }
  [IO.File]::AppendAllText($path, (($row | ConvertTo-Json -Compress -Depth 6) + "`n"), [Text.Encoding]::ASCII)
  Start-Sleep -Milliseconds 100
}
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'tick-writer.ps1') -Value $tickWriter -Encoding ASCII
$launch = @'
param([string]$Role)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$state = Read-State
if (-not $Role) { $Role = 'server' }
Add-Content -LiteralPath (Join-Path $Rig 'events.txt') "launch-$Role"
$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes('Start-Sleep -Seconds 600'))
$child = Start-Process (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoProfile', '-EncodedCommand', $encoded -WindowStyle Hidden
Add-Content -LiteralPath (Join-Path $Rig 'owned-pids.txt') $child.Id
if ($Role -eq 'bot') { $state.bot = $true; $state.botPid = $child.Id }
else { $state.server = $true; $state.serverPid = $child.Id }
Write-State $state
$child
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'start-server.ps1') -Value $launch -Encoding ASCII
Set-Content -LiteralPath (Join-Path $fakeScripts 'launch-client.ps1') -Value $launch -Encoding ASCII
$botCommand = @'
param([string]$Text, [string]$WaitFor, [int]$TimeoutSeconds, [string]$FailOn)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
$state = Read-State
Add-Content -LiteralPath (Join-Path $Rig 'events.txt') "bot:$Text"
$directory = Join-Path $Rig 'clients\bot\pathtracing\recordings'
$null = New-Item -ItemType Directory -Force $directory
$header = '{"type":"recording","autoJump":false,"toggleSprint":false,"toggleCrouch":false,"sprintWindow":0}'
if ($Text -eq 'rec gate-ping') {
  if ($FailOn -match 'Usage') { 'Ping must allow Usage'; exit 2 }
  if ($WaitFor -ne 'Usage: replay') { 'Wrong ping regex'; exit 2 }
  $state.pinged = $true
  Write-State $state
  '[pathtracing] Usage: replay <name> [out], rec start <name>, rec stop, dump blocks'
} elseif ($Text -eq 'stop') { '[pathtracing] Stopped.' }
elseif ($Text -like 'replay *') {
  $words = $Text.Split(' ')
  $name = $words[2]
  $inputPath = Join-Path $Rig "clients\bot\pathtracing\inputs\$($words[1]).jsonl"
  $ticks = @(Get-Content -LiteralPath $inputPath).Count - 1
  if ($WaitFor -ne "Saved ${name}: $ticks ticks\.") { 'Wrong replay regex'; exit 2 }
  if ($TimeoutSeconds -ne (60 + [Math]::Ceiling($ticks / 10.0))) { 'Wrong scaled timeout'; exit 2 }
  $path = Join-Path $directory "$name.jsonl"
  Set-Content -LiteralPath $path -Value $header -Encoding ASCII
  for ($t = 0; $t -lt $ticks; $t++) {
    $x = 0.5
    if ($env:GATE_TEST_CASE -eq 'different' -and $name.EndsWith('_b')) { $x = 7 }
    $row = [ordered]@{ t = $t; tickCount = $(if ($name.EndsWith('_b')) { 200 + $t } else { 100 + $t }); position = @{ x = $x; xBits = '3fe0000000000000' } }
    $row.gameTime = if ($name.EndsWith('_b')) { 2000+$t } else { 1000+$t }
    if ($env:GATE_TEST_CASE -eq 'positive-anomaly') { $row.anomaly = @(@{ type = 'Push' }) }
    $row | ConvertTo-Json -Depth 6 -Compress | Add-Content -LiteralPath $path -Encoding ASCII
  }
  @{ type = 'end'; ticks = $ticks; complete = $true } | ConvertTo-Json -Compress | Add-Content -LiteralPath $path -Encoding ASCII
  if ($env:GATE_TEST_CASE -eq 'replay-failure') { '[pathtracing] Aborted stub replay'; exit 2 }
  "[pathtracing] Saved ${name}: $ticks ticks."
} elseif ($Text -eq 'dump blocks') {
  $dumpDir = Join-Path $Rig 'clients\bot\pathtracing\dumps'
  $null = New-Item -ItemType Directory -Force $dumpDir
  $path = Join-Path $dumpDir 'blocks.jsonl'
  $rows = @(
    '{"type":"dump","mainHand":{},"offHand":{}}'
    '{"block":"minecraft:ice","friction":0.98}'
    '{"block":"minecraft:blue_ice","friction":0.989}'
    '{"block":"minecraft:slime_block","friction":0.8}'
    '{"block":"minecraft:soul_sand","speedFactor":0.4}'
    '{"block":"minecraft:honey_block","speedFactor":0.4,"jumpFactor":0.5}'
    '{"type":"summary","writtenCount":5,"registryCount":5}'
  )
  if ($env:GATE_TEST_CASE -in @('bad-dump', 'owned-bad-dump')) { $rows[1] = '{"block":"minecraft:ice","friction":0.6}' }
  $rows | Set-Content -LiteralPath $path -Encoding ASCII
  '[pathtracing] Dumped 5 block states; independent registry count 5.'
} elseif ($Text -like 'rec start *') {
  $state.recording = $Text.Split(' ')[2]
  Write-State $state
  $path = Join-Path $directory "$($state.recording).jsonl.tmp"
  Set-Content -LiteralPath $path -Value $header -Encoding ASCII
  for ($t = 0; $t -lt 5; $t++) {
    $row = @{ t = $t; tickCount = 100 + $t; position = @{ x = 0.5; xBits = '3fe0000000000000' } }
    if ($env:GATE_TEST_CASE -eq 'baseline-anomaly') { $row.anomaly = @(@{ type = 'Push' }) }
    $row | ConvertTo-Json -Depth 5 -Compress | Add-Content -LiteralPath $path -Encoding ASCII
  }
  "[pathtracing] Recording $($state.recording)."
} elseif ($Text -eq 'rec stop') {
  if ($state.writerPid) {
    Set-Content -LiteralPath (Join-Path $Rig 'stop-writer') -Value 'stop'
    $writer = Get-Process -Id $state.writerPid -ErrorAction SilentlyContinue
    if ($writer -and -not $writer.WaitForExit(3000)) { throw 'Stub writer failed to stop.' }
  }
  $path = Join-Path $directory "$($state.recording).jsonl.tmp"
  $ticks = @(Get-Content -LiteralPath $path).Count - 1
  if ($env:GATE_TEST_CASE -ne 'negative-malformed') {
    @{ type = 'end'; ticks = $ticks; complete = $true } | ConvertTo-Json -Compress | Add-Content -LiteralPath $path -Encoding ASCII
  }
  Move-Item -LiteralPath $path -Destination $path.Replace('.tmp', '')
  "[pathtracing] Saved $($state.recording): $ticks ticks."
} else { throw "Unexpected bot command $Text" }
'@
Set-Content -LiteralPath (Join-Path $fakeGates 'bot-command.ps1') -Value $botCommand -Encoding ASCII

function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Run-Case([string]$Name, [int]$Expected, [string[]]$GateLines = @(), [bool]$Launch = $false) {
  if ($Case -and $Name -notin $Case) { return }
  $script:caseCount++
  $env:GATE_TEST_CASE = $Name
  $env:GATE_TEST_RIG = Join-Path $fakeRig $Name
  $caseRig = $env:GATE_TEST_RIG
  $null = New-Item -ItemType Directory -Force (Join-Path $caseRig 'clients\bot\mods')
  $state = @{ server = -not $Launch; bot = -not $Launch; serverPid = 0; botPid = 0; writerPid = 0; recording = ''; pinged = $false }
  if ($Name -in @('running-offline', 'running-different')) { $state.bot = $false }
  if ($Name -eq 'bot-only') { $state.bot = $false }
  if ($Name -eq 'server-only') { $state.server = $false }
  if ($Name -eq 'locked') { Set-Content -LiteralPath (Join-Path $caseRig 'rig.lock') -Value 'existing-owner' }
  $state | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $caseRig 'state.json') -Encoding ASCII
  $options = @('autoJump:false', 'toggleSprint:false', 'toggleCrouch:false', 'sprintWindow:0')
  if ($Name -eq 'bad-options') { $options[0] = 'autoJump:true' }
  if ($Name -eq 'duplicate-options') { $options += 'autoJump:false' }
  if ($Name -ne 'missing-options') { $options | Set-Content -LiteralPath (Join-Path $caseRig 'clients\bot\options.txt') -Encoding ASCII }
  $jar = Join-Path $caseRig 'clients\bot\mods\pathtracing.jar'
  Set-Content -LiteralPath $jar -Value 'installed-jar' -Encoding ASCII
  $newJar = Join-Path $caseRig "candidate's jar.jar"
  Set-Content -LiteralPath $newJar -Value $(if ($Name -eq 'running-same') { 'installed-jar' } else { 'new-jar' }) -Encoding ASCII
  $arguments = @('-NoProfile', '-File', (Join-Path $fakeGates 'run-gate-1.2.ps1'))
  $runCount = 1
  $tickCount = 4
  if ($Name -eq 'defaults') { $runCount = 8; $tickCount = 400 }
  else { $arguments += @('-Runs', '1', '-Ticks', '4') }
  if ($Name -in @('running-same', 'running-different', 'online-different', 'install-jar')) { $arguments += @('-Jar', $newJar) }
  if ($Name -eq 'seed-overflow') { $arguments += @('-Seed', '2147483647') }
  $output = @(& $shell @arguments 2>&1)
  $actual = $LASTEXITCODE
  $owned = Join-Path $caseRig 'owned-pids.txt'
  try {
    Assert ($actual -eq $Expected) "$Name exit: expected $Expected, got $actual. Output: $($output -join ' | ')"
    Assert ($output.Count -ge 2) "$Name did not print report and overall last."
    $report = [string]$output[-2]
    Assert (Test-Path -LiteralPath $report) "$Name missing report: $report"
    $text = Get-Content -LiteralPath $report -Raw
    Assert ($output[-1] -eq $(if ($Expected -eq 0) { 'Overall: PASS' } else { 'Overall: FAIL' })) "$Name overall mismatch."
    foreach ($gateLine in $GateLines) { Assert ($text.Contains($gateLine)) "$Name missing $gateLine in $report" }
    if ($Name -eq 'locked') {
      Assert ((Get-Content -LiteralPath (Join-Path $caseRig 'rig.lock') -Raw).Trim() -eq 'existing-owner') 'Removed the existing lock.'
    } else { Assert (-not (Test-Path -LiteralPath (Join-Path $caseRig 'rig.lock'))) "$Name leaked the rig lock." }
    $eventsPath = Join-Path $caseRig 'events.txt'
    $events = if (Test-Path -LiteralPath $eventsPath) { @(Get-Content -LiteralPath $eventsPath) } else { @() }
    if ($Name -ne 'locked') { Assert ($events[0] -eq 'lock' -and $events[-1] -eq 'unlock') "$Name lock order." }
    Assert (@($events | Where-Object { $_ -like '*player*' }).Count -eq 0) "$Name touched the player client."
    if ($Name -eq 'owner') { Assert (($events -join ',') -eq 'lock,list,unlock') 'Owner refusal changed the rig.' }
    if (-not $Launch -and $Name -notin @('bot-only', 'server-only')) {
      Assert ('stop' -notin $events -and @($events | Where-Object { $_ -like 'launch-*' }).Count -eq 0) "$Name changed pre-existing process lifetime."
    }
    if ($Name -eq 'bot-only') { Assert ('launch-bot' -in $events -and 'launch-server' -notin $events -and 'stop' -notin $events) 'Bot-only disturbed existing server.' }
    if ($Name -eq 'server-only') { Assert ('launch-server' -in $events -and 'launch-bot' -notin $events -and 'stop' -in $events) 'Server-only disturbed existing bot.' }
    if ($Name -eq 'running-different') {
      Assert ((Get-Content -LiteralPath $jar -Raw).Trim() -eq 'installed-jar') 'Replaced a running jar.'
    }
    if ($Name -eq 'install-jar') { Assert ((Get-Content -LiteralPath $jar -Raw).Trim() -eq 'new-jar') 'Jar was not installed.' }
    if ($Launch) {
      Assert ('launch-server' -in $events -and 'stop' -in $events) "$Name did not launch/stop owned server."
      if ($Name -ne 'server-timeout') { Assert ('launch-bot' -in $events) "$Name did not launch bot." }
    }
    if (Test-Path -LiteralPath $owned) {
      foreach ($ownedId in (Get-Content -LiteralPath $owned)) { Assert ($null -eq (Get-Process -Id ([int]$ownedId) -ErrorAction SilentlyContinue)) "$Name left owned process $ownedId running." }
    }
    if ($Expected -eq 0) {
      $dir = Split-Path -Parent $report
      Assert (@(Get-ChildItem -LiteralPath $dir -Filter '*_a.jsonl').Count -eq $runCount) "$Name missing replay copies."
      Assert (Test-Path -LiteralPath (Join-Path $dir 'dump-summary.jsonl')) "$Name missing dump summary."
      $inputFile = Get-ChildItem -LiteralPath $dir -Filter '*.jsonl' | Where-Object { $_.Name -match '_1\.jsonl$' }
      $expectedInput = Join-Path $caseRig 'clients\bot\pathtracing\inputs\expected.jsonl'
      $null = & $shell -NoProfile -File (Join-Path $fakeGates 'new-inputs.ps1') -Name expected -Ticks $tickCount -Seed 2 -Start '0.5,-60,0.5,0,0'
      Assert ((Get-FileHash -LiteralPath $inputFile.FullName).Hash -eq (Get-FileHash -LiteralPath $expectedInput).Hash) "$Name wrong seeded inputs."
      foreach ($log in (Get-ChildItem -LiteralPath $dir -Filter '*.log' | Where-Object Name -ne 'driver.log')) {
        Assert ((Get-Content -LiteralPath $log.FullName -Raw).Contains('Exit:')) "$Name incomplete command log."
      }
    }
    Write-Output "PASS $Name (exit $actual): $report"
  } finally {
    # Test-only fail-safe: clean stub sleeper processes even when the driver under test fails.
    if (Test-Path -LiteralPath $owned) {
      foreach ($ownedId in (Get-Content -LiteralPath $owned)) { Stop-Process -Id ([int]$ownedId) -Force -ErrorAction SilentlyContinue }
    }
  }
}
$previousRig = $env:GATE_TEST_RIG
$previousCase = $env:GATE_TEST_CASE
try {
  Run-Case 'pass' 0 @('Gate 1: PASS', 'Gate 2: PASS', 'Gate 3: PASS', 'Gate 4: PASS')
  Run-Case 'launch-owned' 0 @('Gate 4: PASS') $true
  Run-Case 'bot-only' 0
  Run-Case 'server-only' 0
  Run-Case 'owned-bad-dump' 1 @('Gate 3: FAIL', 'Gate 4: PASS') $true
  Run-Case 'owner' 3
  Run-Case 'owner-later' 3
  Run-Case 'similar-name' 0
  Run-Case 'locked' 1
  Run-Case 'bad-options' 3
  Run-Case 'duplicate-options' 3
  Run-Case 'missing-options' 3
  Run-Case 'running-different' 3
  Run-Case 'online-different' 3
  Run-Case 'running-same' 0
  Run-Case 'running-offline' 4
  Run-Case 'install-jar' 0 @() $true
  Run-Case 'server-timeout' 4 @() $true
  Run-Case 'bad-list' 1
  Run-Case 'seed-overflow' 1
  Run-Case 'different' 1 @('Gate 1: FAIL', 'Gate 2: PASS', 'Gate 3: PASS', 'Gate 4: PASS')
  Run-Case 'positive-anomaly' 1 @('Gate 1: PASS', 'Gate 2: FAIL', 'Gate 4: PASS')
  Run-Case 'replay-failure' 1 @('Gate 1: FAIL', 'Gate 2: FAIL', 'Gate 4: PASS')
  Run-Case 'bad-dump' 1 @('Gate 3: FAIL', 'Gate 4: PASS')
  Run-Case 'no-negative-anomaly' 1 @('Gate 4: FAIL')
  Run-Case 'unrelated-anomaly' 1 @('Gate 4: FAIL')
  Run-Case 'negative-malformed' 1 @('Gate 4: FAIL')
  Run-Case 'baseline-anomaly' 1 @('Gate 4: FAIL')
  Run-Case 'rcon-hang' 4
  Run-Case 'defaults' 0 @('Runs: 8; ticks per replay: 400;', 'Gate 1: PASS', 'Gate 4: PASS')
  Assert ($caseCount -gt 0) 'No test cases selected.'
  Write-Output "Offline stub integration tests passed ($caseCount cases). Evidence: $sandbox"
} finally {
  $env:GATE_TEST_RIG = $previousRig
  $env:GATE_TEST_CASE = $previousCase
}
