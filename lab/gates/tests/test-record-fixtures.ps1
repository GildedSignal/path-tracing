# Offline copied-driver tests. Only fake common, RCON, bot and process launchers can reach the rig.
# The actual input generator, course data, transport wrapper and recording checker remain in use.
param([string[]]$Case)
$ErrorActionPreference = 'Stop'
$Case = @($Case | Where-Object { $_ } | ForEach-Object { $_.Split(',') })
$gateDir = Split-Path -Parent $PSScriptRoot
$sandbox = Join-Path $gateDir ('results\fixture-stubs-' + [Guid]::NewGuid().ToString('N'))
$repo = Join-Path $sandbox 'repo'
$fakeGates = Join-Path $repo 'lab\gates'
$fakeScripts = Join-Path $repo 'scripts'
$shell = Join-Path $PSHOME 'powershell.exe'
$null = New-Item -ItemType Directory -Force $fakeGates, $fakeScripts
foreach ($file in @('record-fixtures.ps1', 'fixture-rig.ps1', 'courses.ps1', 'new-inputs.ps1', 'corner-inputs.ps1', 'check-recording.ps1')) {
  $text = Get-Content -LiteralPath (Join-Path $gateDir $file) -Raw
  $text = $text.Replace('Elapsed.TotalSeconds -lt 120', 'Elapsed.TotalSeconds -lt 1')
  $text = $text.Replace('Elapsed.TotalSeconds -lt 180', 'Elapsed.TotalSeconds -lt 1')
  $text = $text.Replace('Start-Sleep -Seconds 2', 'Start-Sleep -Milliseconds 10')
  Set-Content -LiteralPath (Join-Path $fakeGates $file) -Value $text -Encoding ASCII
}
$common = @'
$Rig = $env:FIXTURE_TEST_RIG
$ServerHost = '127.0.0.1'
$ServerPort = 25565
$RconPort = 25575
$PlayerName = 'Brain_Forge'
$BotName = 'PathBot'
function Read-State { Get-Content -LiteralPath (Join-Path $Rig 'state.json') -Raw | ConvertFrom-Json }
function Write-State($State) { $State | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Rig 'state.json') -Encoding ASCII }
function Enter-Rig($Holder) {
  if ($env:FIXTURE_TEST_CASE -eq 'locked') { throw 'Rig is held by stub owner.' }
  Set-Content -LiteralPath (Join-Path $Rig 'rig.lock') -Value $Holder
  Add-Content -LiteralPath (Join-Path $Rig 'events.txt') 'lock'
}
function Exit-Rig {
  Add-Content -LiteralPath (Join-Path $Rig 'events.txt') 'unlock'
  Remove-Item -LiteralPath (Join-Path $Rig 'rig.lock')
}
function Test-Port($HostName, $Port) {
  if ($env:FIXTURE_TEST_CASE -eq 'server-timeout') { return $false }
  [bool](Read-State).server
}
function Get-CimInstance {
  param($ClassName, $Filter)
  if ($env:FIXTURE_TEST_CASE -eq 'running-different') {
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
foreach ($command in $Commands) {
  Add-Content -LiteralPath (Join-Path $Rig 'events.txt') $command
  if ($command -eq 'list') {
    $names = @()
    if ($state.bot) { $names += 'PathBot' }
    if ($env:FIXTURE_TEST_CASE -eq 'owner' -or ($env:FIXTURE_TEST_CASE -eq 'owner-later' -and $state.replays -gt 0)) { $names += 'Brain_Forge' }
    if ($env:FIXTURE_TEST_CASE -eq 'similar-name') { $names += 'Brain_Forge2' }
    "There are $($names.Count) of a max of 4 players online: $($names -join ', ')"
  } elseif ($command -eq 'stop') {
    if ($state.serverPid) { Stop-Process -Id $state.serverPid -Force -ErrorAction SilentlyContinue }
    'Stopping the server'
  } elseif ($command -like 'fill *' -and $env:FIXTURE_TEST_CASE -eq 'build-failure') {
    'That position is not loaded'
  } elseif ($command -like 'attribute *') {
    'Maximum health is 20'
  } elseif ($command -like 'effect give *') {
    $state.healed = $true; Write-State $state
    'Command succeeded'
  } else { 'Command succeeded' }
}
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'rcon.ps1') -Value $rcon -Encoding ASCII
$snapshot = @'
param([string]$PlayerName)
. "$PSScriptRoot\common.ps1"
Add-Content -LiteralPath (Join-Path $Rig 'events.txt') 'snapshot'
$state = Read-State
if ($env:FIXTURE_TEST_CASE -eq 'snapshot-failure' -and (Read-State).replays -gt 0) { 'bad snapshot'; exit 1 }
$frozen = 0
if ($env:FIXTURE_TEST_CASE -eq 'thaw' -and $state.replays -eq 1 -and -not $state.thawed -and
    (Get-Content -LiteralPath (Join-Path $Rig 'events.txt'))[-2] -like 'attribute *') {
  $frozen = 12
  $state.thawed = $true
  Write-State $state
  Add-Content -LiteralPath (Join-Path $Rig 'events.txt') 'thaw-wait'
}
$time = if ($env:FIXTURE_TEST_CASE -eq 'end-delayed' -and $state.replays -gt 0) { 125 }
  elseif ($env:FIXTURE_TEST_CASE -eq 'end-negative' -and $state.replays -gt 0) { 118 } else { 121 }
$after = if ($env:FIXTURE_TEST_CASE -eq 'end-uncertain') { $time+1 } else { $time }
$hurt = -not $state.healed -and ($env:FIXTURE_TEST_CASE -eq 'heal' -or
  ($env:FIXTURE_TEST_CASE -eq 'abort-continue' -and $state.replays -eq 2 -and -not $state.recording))
$health = if ($hurt) { 18 } else { 20 }
$timer = if ($hurt) { 7 } else { 0 }
$exhaustion = if ($env:FIXTURE_TEST_CASE -eq 'settle-exhaustion' -and $state.recording) { 0.8 } else { 0.75 }
if ($env:FIXTURE_TEST_CASE -eq 'settle-timer' -and $state.recording) { $timer = 1 }
@{ Health=$health; foodLevel=20; foodSaturationLevel=5; foodExhaustionLevel=$exhaustion; foodTickTimer=$timer;
  TicksFrozen=$frozen; playerGameType=1; gameTimeBefore=$time; gameTimeAfter=$after } | ConvertTo-Json -Compress
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'server-state.ps1') -Value $snapshot -Encoding ASCII
$launch = @'
param([string]$Role)
. "$PSScriptRoot\common.ps1"
if (-not $Role) { $Role = 'server' }
Add-Content -LiteralPath (Join-Path $Rig 'events.txt') "launch-$Role"
$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes('Start-Sleep -Seconds 600'))
$child = Start-Process (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoProfile', '-EncodedCommand', $encoded -WindowStyle Hidden
Add-Content -LiteralPath (Join-Path $Rig 'owned-pids.txt') $child.Id
$state = Read-State
if ($Role -eq 'bot') { $state.bot = $true; $state.botPid = $child.Id }
else { $state.server = $true; $state.serverPid = $child.Id }
Write-State $state
$child
'@
Set-Content -LiteralPath (Join-Path $fakeScripts 'start-server.ps1') -Value $launch -Encoding ASCII
Set-Content -LiteralPath (Join-Path $fakeScripts 'launch-client.ps1') -Value $launch -Encoding ASCII
$bot = @'
param([string]$Text, [string]$WaitFor, [int]$TimeoutSeconds, [string]$FailOn)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
. "$PSScriptRoot\courses.ps1"
$state = Read-State
Add-Content -LiteralPath (Join-Path $Rig 'events.txt') "bot:$Text"
$directory = Join-Path $Rig 'clients\bot\pathtracing\recordings'
$null = New-Item -ItemType Directory -Force $directory
$w = $Text.Split(' ')
if ($w[0] -eq 'replay') {
  $state.replays++
  $state.recording = $w[2]
  Write-State $state
  Set-Content -LiteralPath (Join-Path $directory "$($w[2]).jsonl.tmp") -Value '{"type":"recording"}' -Encoding ASCII
  "Settled $($w[2]); waiting for rec continue."
  exit 0
}
if ($Text -eq 'rec continue') { $w = @('replay', $state.recording, $state.recording) }
if ($w[0] -eq 'rec' -and $w[1] -eq 'fixture-ping') { $reply = 'Usage: replay' }
elseif ($w[0] -eq 'stop') { $reply = 'Stopped.' }
elseif ($w[0] -eq 'replay') {
  $inputRows = @(Get-Content -LiteralPath (Join-Path $Rig "clients\bot\pathtracing\inputs\$($w[1]).jsonl") | ForEach-Object { $_ | ConvertFrom-Json })
  $start = $inputRows[0].start
  $initial = [ordered]@{ kind = 'player'; position = @{ x = $start[0]; y = $start[1]; z = $start[2] };
    velocity = @{ x = 0; y = -0.0784000015258789; z = 0 }; onGround = $true; pose = 'STANDING';
    speed = 0.1; speedBits = '3dcccccd'; foodLevel = 20; health = 20; saturationLevel = 5; exhaustion = 0; ticksFrozen = 0 }
  if ($env:FIXTURE_TEST_CASE -eq 'initial-mismatch') { $initial.health = 19 }
  $header = @{ type = 'recording'; autoJump = $false; toggleSprint = $false; toggleCrouch = $false; sprintWindow = 0; initialPlayer = $initial }
  if ($env:FIXTURE_TEST_CASE -eq 'missing-initial') { $header.Remove('initialPlayer') }
  $path = Join-Path $directory "$($w[2]).jsonl"
  $rows = @($header | ConvertTo-Json -Depth 10 -Compress)
  $count = $inputRows.Count - 1
  for ($t = 0; $t -lt $count; $t++) {
    $row = @{ t = $t; tickCount = 100 + $t; gameTime = 100 + $t; position = @{ x = $start[0]+2; y = $start[1]; z = $start[2]-1 };
      onGround=($env:FIXTURE_TEST_CASE -ne 'end-airborne'); velocity=@{ x=0; y=-0.0784; z=0 }; speedModifiers=@(); ticksFrozen=0 }
    if ($w[1] -like '*_corner_*' -and $env:FIXTURE_TEST_CASE -ne 'corner-no-tie') {
      $row.collisionSteps = @(@{ requested = @{ xBits='3fb999999999999a'; zBits='3fb999999999999a' };
        result = @{ xBits='3fb999999999999a'; zBits='0000000000000000' } })
    }
    if ($env:FIXTURE_TEST_CASE -in @('anomaly', 'owned-anomaly') -and $t -eq 0) { $row.anomaly = @(@{ type = 'ClientboundPlayerPositionPacket' }) }
    if ($env:FIXTURE_TEST_CASE -eq 'tick-gap') { $row.tickCount = 100 + 2 * $t }
    $rows += $row | ConvertTo-Json -Depth 5 -Compress
  }
  if ($env:FIXTURE_TEST_CASE -eq 'replay-failure' -or ($env:FIXTURE_TEST_CASE -eq 'abort-continue' -and $state.replays -eq 2)) {
    # Real abort closes the writer, leaves .tmp, and ignores rec stop thereafter.
    $state.recording = ''; Write-State $state
    Set-Content -LiteralPath "$path.tmp" -Value @($rows[0..1] + '{"type":"end","ticks":1,"complete":false}') -Encoding ASCII
    'Replay failed. Aborted; partial output remains.'; exit 2
  }
  if ($env:FIXTURE_TEST_CASE -eq 'cleanup-failure' -and $state.replays -eq 2) {
    Set-Content -LiteralPath "$path.tmp" -Value $rows -Encoding ASCII
    'Transport failed before completion'; exit 1
  }
  $rows += @{ type = 'end'; ticks = $count; complete = ($env:FIXTURE_TEST_CASE -ne 'incomplete') } | ConvertTo-Json -Compress
  Set-Content -LiteralPath $path -Value $rows -Encoding ASCII
  Remove-Item -LiteralPath "$path.tmp"
  $state.recording = ''; Write-State $state
  $reply = "Saved $($w[2]): $count ticks."
} elseif ($Text -eq 'rec stop') {
  if (-not $state.recording) { 'Timed out: inactive recorder sends no reply.'; exit 1 }
  if ($env:FIXTURE_TEST_CASE -eq 'cleanup-failure') { 'Cancel transport failed'; exit 1 }
  $path = Join-Path $directory "$($state.recording).jsonl"
  Add-Content -LiteralPath "$path.tmp" -Value '{"type":"end","ticks":2,"complete":false}' -Encoding ASCII
  Move-Item -LiteralPath "$path.tmp" -Destination $path
  $reply = "Saved $($state.recording): 2 ticks."
  $state.recording = ''; Write-State $state
} elseif ($w[0] -eq 'export') {
  $isReady = $w[1].EndsWith('_ready')
  if (-not $isReady -and $env:FIXTURE_TEST_CASE -eq 'export-failure') { 'Export failed'; exit 2 }
  $exports = Join-Path $Rig 'clients\bot\pathtracing\fixtures'
  $null = New-Item -ItemType Directory -Force $exports
  $path = Join-Path $exports "$($w[1]).world.jsonl"
  $rows = @('{"kind":"world","minY":-64,"difficulty":2,"naturalRegeneration":true}')
  if ($isReady) {
    $course = @(Get-FixtureCourses | Where-Object { $w[1].Contains('_' + $_.Name + '_ready') })[0]
    $id = if ($env:FIXTURE_TEST_CASE -eq 'stale-client') { 'minecraft:stone' } else { 'minecraft:gold_block' }
    $rows += @{ kind = 'block'; at = $course.Marker; block = $id; boxes = @(@(0,0,0,1,1,1)) } | ConvertTo-Json -Compress -Depth 5
  } else { $rows += '{"kind":"block","at":[4112,-61,4100],"boxes":[[0,0,0,1,1,1]],"friction":0.6,"frictionBits":"3f19999a"}' }
  Set-Content -LiteralPath $path -Value $rows -Encoding ASCII
  $reply = "Exported $($w[1]): 1 blocks."
} else { throw "Unexpected bot command $Text" }
if ($reply -notmatch $WaitFor) { throw "Stub reply '$reply' does not match '$WaitFor'." }
"[pathtracing] $reply"
'@
Set-Content -LiteralPath (Join-Path $fakeGates 'bot-command.ps1') -Value $bot -Encoding ASCII

function Assert([bool]$Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
$caseCount = 0
function Run-Case([string]$Name, [int]$Expected, [string]$Reason = '', [bool]$Launch = $false) {
  if ($Case -and $Name -notin $Case) { return }
  $script:caseCount++
  $env:FIXTURE_TEST_CASE = $Name
  $env:FIXTURE_TEST_RIG = Join-Path $sandbox "rig with spaces\$Name"
  $caseRig = $env:FIXTURE_TEST_RIG
  # Each case gets its own publication directory by rewriting only this path in its driver copy.
  $driver = (Get-Content -LiteralPath (Join-Path $fakeGates 'record-fixtures.ps1') -Raw).Replace(
    "`$fixtures = Join-Path `$PSScriptRoot '..\fixtures'", "`$fixtures = Join-Path `$Rig 'kept'")
  Set-Content -LiteralPath (Join-Path $fakeGates 'record-fixtures.ps1') -Value $driver -Encoding ASCII
  $null = New-Item -ItemType Directory -Force (Join-Path $caseRig 'clients\bot\mods')
  $state = @{ server = -not $Launch; bot = -not $Launch; serverPid = 0; botPid = 0; replays = 0; recording = ''; thawed = $false; healed=$false }
  $state | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $caseRig 'state.json') -Encoding ASCII
  $jar = Join-Path $caseRig 'clients\bot\mods\pathtracing.jar'
  Set-Content -LiteralPath $jar -Value 'installed' -Encoding ASCII
  $options = @('autoJump:false', 'toggleSprint:false', 'toggleCrouch:false', 'sprintWindow:0')
  if ($Name -eq 'bad-options') { $options[0] = 'autoJump:true' }
  Set-Content -LiteralPath (Join-Path $caseRig 'clients\bot\options.txt') -Value $options -Encoding ASCII
  if ($Name -eq 'locked') { Set-Content -LiteralPath (Join-Path $caseRig 'rig.lock') -Value 'existing-owner' }
  $argsForDriver = @('-NoProfile', '-File', (Join-Path $fakeGates 'record-fixtures.ps1'), '-Courses', 'full-blocks', '-Runs', '1', '-Ticks', '2')
  if ($Name -eq 'multi-course') { $argsForDriver[4] = 'edges,water' }
  if ($Name -in @('abort-continue', 'cleanup-failure')) { $argsForDriver[4] = 'full-blocks,ice'; $argsForDriver[6] = '3' }
  if ($Name -in @('corner', 'corner-no-tie')) { $argsForDriver[4] = 'corner' }
  if ($Name -eq 'thaw') { $argsForDriver[6] = '2' }
  if ($Name -eq 'unknown-course') { $argsForDriver[4] = 'missing' }
  if ($Name -eq 'duplicate-course') { $argsForDriver[4] = 'ice,ice' }
  if ($Name -eq 'defaults') { $argsForDriver = $argsForDriver[0..4] }
  if ($Name -eq 'all-courses') { $argsForDriver = @($argsForDriver[0..2]) + @('-Runs', '1', '-Ticks', '2') }
  if ($Name -eq 'seed-overflow') { $argsForDriver += @('-Seed', '2147483647') }
  if ($Name -eq 'running-different') {
    $candidate = Join-Path $caseRig "candidate's jar.jar"
    Set-Content -LiteralPath $candidate -Value 'candidate' -Encoding ASCII
    $argsForDriver += @('-Jar', $candidate)
  }
  if ($Name -eq 'existing') {
    $null = New-Item -ItemType Directory -Force (Join-Path $caseRig 'kept\full-blocks')
    Set-Content -LiteralPath (Join-Path $caseRig 'kept\full-blocks\full-blocks-1.world.jsonl') -Value 'original' -Encoding ASCII
  }
  $output = @(& $shell @argsForDriver 2>&1)
  $actual = $LASTEXITCODE
  $owned = Join-Path $caseRig 'owned-pids.txt'
  try {
    Assert ($actual -eq $Expected) "$Name expected exit $Expected, got $actual; $($output -join ' | ')"
    $report = [string]$output[-2]
    Assert (Test-Path -LiteralPath $report) "$Name missing report; $($output -join ' | ')"
    $text = Get-Content -LiteralPath $report -Raw
    if ($Reason) { Assert ($text.Contains($Reason)) "$Name missing '$Reason' in $report" }
    Assert ($text.Contains('atRest true and a non-null offset:')) "$Name missing snapshot comparability count."
    $events = @()
    if (Test-Path -LiteralPath (Join-Path $caseRig 'events.txt')) { $events = @(Get-Content -LiteralPath (Join-Path $caseRig 'events.txt')) }
    if ($Name -eq 'owner') { Assert (($events -join ',') -eq 'lock,list,unlock') 'Owner refusal changed rig.' }
    if ($Name -eq 'locked') { Assert ((Get-Content -LiteralPath (Join-Path $caseRig 'rig.lock') -Raw).Trim() -eq 'existing-owner') 'Removed another lock.' }
    else { Assert (-not (Test-Path -LiteralPath (Join-Path $caseRig 'rig.lock'))) "$Name leaked lock." }
    if (-not $Launch) { Assert ('stop' -notin $events -and @($events | Where-Object { $_ -like 'launch-*' }).Count -eq 0) "$Name changed existing process lifetime." }
    if (Test-Path -LiteralPath $owned) {
      foreach ($ownedId in Get-Content -LiteralPath $owned) { Assert ($null -eq (Get-Process -Id ([int]$ownedId) -ErrorAction SilentlyContinue)) "$Name leaked owned process $ownedId." }
    }
    $kept = @(Get-ChildItem -LiteralPath (Join-Path $caseRig 'kept') -Filter '*.jsonl' -Recurse -ErrorAction SilentlyContinue)
    if ($Expected -eq 0 -or $Name -in @('abort-continue', 'cleanup-failure')) {
      $expectedPairs = if ($Name -eq 'defaults') { 8 } elseif ($Name -in @('multi-course', 'thaw')) { 2 } elseif ($Name -eq 'all-courses') { 14 } elseif ($Name -in @('abort-continue', 'cleanup-failure')) { 5 } else { 1 }
      Assert ($kept.Count -eq 2*$expectedPairs) "$Name wrong kept count $($kept.Count)."
      foreach ($recording in @($kept | Where-Object { -not $_.Name.EndsWith('.world.jsonl') })) {
        $header = Get-Content -LiteralPath $recording.FullName -First 1 | ConvertFrom-Json
        $worldLines = @(Get-Content -LiteralPath ($recording.FullName.Replace('.jsonl', '.world.jsonl')))
        $player = $worldLines[0] | ConvertFrom-Json
        Assert ($player.position.x -eq $header.initialPlayer.position.x) "$Name player was post-tick."
        Assert ($player.speedBits -ceq '3dcccccd') "$Name lost initial raw bits."
        Assert ($player.exhaustionBits -ceq '3f400000') "$Name missing initial server exhaustion."
        Assert ($player.foodTickTimer -eq 0) "$Name missing zero server foodTickTimer."
        Assert (($worldLines[-1] | ConvertFrom-Json).kind -eq 'serverEnd') "$Name missing serverEnd."
        $end = $worldLines[-1] | ConvertFrom-Json
        $expectedOffset = if ($Name -eq 'end-delayed') { 4 } elseif ($Name -eq 'end-negative') { -3 } elseif ($Name -eq 'end-uncertain') { $null } else { 0 }
        if ($Name -in @('pass','end-delayed','end-uncertain','end-negative','end-airborne')) {
          if ($Name -eq 'pass') { $expectedOffset = 0 }
          Assert ($end.serverTickOffset -eq $expectedOffset) "$Name wrong server tick offset."
          Assert ($end.atRest -eq ($Name -ne 'end-airborne')) "$Name wrong atRest."
          $comparable = if ($end.atRest -and $null -ne $expectedOffset) { 1 } else { 0 }
          Assert ($text.Contains("atRest true and a non-null offset: $comparable of 1.")) "$Name wrong comparability count."
        }
        $raw = Get-ChildItem -LiteralPath (Join-Path $caseRig 'clients\bot\pathtracing\recordings') -Filter '*.jsonl' |
          Where-Object { (Get-FileHash -LiteralPath $_.FullName).Hash -eq (Get-FileHash -LiteralPath $recording.FullName).Hash }
        Assert (@($raw).Count -gt 0) "$Name changed the recording bytes."
      }
      $eventsText = $events -join "`n"
      Assert ($eventsText -match 'Saved|snapshot') "$Name no server snapshots."
      $replayAt = [array]::FindIndex($events, [Predicate[string]]{ param($line) $line -like 'bot:replay *' })
      Assert ($events[$replayAt+1] -eq 'snapshot') "$Name did not snapshot immediately after replay."
      Assert ($events[$replayAt+3] -eq 'bot:rec continue' -and $events[$replayAt+4] -eq 'snapshot') "$Name missing settle/read/continue/end sequence."
      foreach ($inputFile in Get-ChildItem -LiteralPath (Join-Path $caseRig 'clients\bot\pathtracing\inputs') -Filter '*.jsonl') {
        $tail = @(Get-Content -LiteralPath $inputFile.FullName | Select-Object -Last 20 | ForEach-Object { $_ | ConvertFrom-Json })
        Assert ($tail.Count -eq 20) "$Name missing twenty zero-input ticks."
        foreach ($row in $tail) {
          foreach ($key in @('forward','back','left','right','jump','sneak','sprint')) { Assert ($row.$key -eq $false) "$Name tail held $key." }
        }
      }
      if ($Name -eq 'heal') { Assert ('effect give PathBot instant_health' -in $events) 'Did not heal before replay.' }
      # The first visited x is two blocks beyond the initial x. The export must include both.
      if ($Name -notin @('multi-course','corner','corner-no-tie')) { Assert ($eventsText -match 'bot:export .*_full-blocks_1 4109 -63 4096 4117 -57 4103') "$Name wrong visited bounds/margin." }
      Assert ('gamemode creative PathBot' -in $events) "$Name did not restore original game mode."
      if ($Name -eq 'thaw') {
        $thawAt = [array]::IndexOf($events, 'thaw-wait')
        Assert ($thawAt -ge 0 -and $events[$thawAt+2] -eq 'snapshot') 'Did not wait for a zero-frozen snapshot before replay.'
      }
      Assert (@(Get-ChildItem -LiteralPath (Split-Path -Parent $report) -Filter '*_ready*').Count -eq 0) "$Name kept the full readiness export."
      if ($Name -eq 'abort-continue') {
        Assert ('bot:rec stop' -notin $events) 'Sent rec stop after abort.'
        Assert ('effect give PathBot instant_health' -in $events) 'Abort cleanup did not heal the bot.'
        Assert ($text.Contains('Course ice - kept 3; rejected 0')) 'Abort prevented following course.'
        Assert ($text.Contains('Course full-blocks - kept 2; rejected 1')) 'Abort prevented following run.'
      }
    } elseif ($Name -ne 'existing') { Assert ($kept.Count -eq 0) "$Name published rejected fixture." }
    if ($Name -eq 'existing') { Assert ((Get-Content -LiteralPath $kept[0].FullName -Raw).Trim() -eq 'original') 'Overwrote fixture.' }
    if ($Name -in @('settle-exhaustion','settle-timer')) {
      Assert ('bot:rec continue' -notin $events) "$Name continued after a changing resource."
      Assert ('bot:rec stop' -in $events) "$Name left a paused recorder active."
      Assert (-not $text.Contains('Restore start resources cleanup failed:')) "$Name failed to restore start resources."
    }
    if ($Name -eq 'running-different') { Assert ((Get-Content -LiteralPath $jar -Raw).Trim() -eq 'installed') 'Replaced running jar.' }
    Write-Output "PASS $Name (exit $actual) - $report"
  } finally {
    if (Test-Path -LiteralPath $owned) { foreach ($ownedId in Get-Content -LiteralPath $owned) { Stop-Process -Id ([int]$ownedId) -Force -ErrorAction SilentlyContinue } }
  }
}
$previousRig = $env:FIXTURE_TEST_RIG
$previousCase = $env:FIXTURE_TEST_CASE
try {
  Run-Case pass 0 'Course full-blocks - kept 1; rejected 0'
  Run-Case multi-course 0 'Course edges - kept 1; rejected 0'
  Run-Case launch-owned 0 '' $true
  Run-Case owned-anomaly 1 'Recording check failed' $true
  Run-Case owner 3 'online'
  Run-Case owner-later 3 'online'
  Run-Case similar-name 0
  Run-Case locked 1 'Rig is held'
  Run-Case bad-options 3 'Bot options must'
  Run-Case running-different 3 'jar differs'
  Run-Case server-timeout 4 'Timed out' $true
  Run-Case seed-overflow 1 'seed range'
  Run-Case existing 1 'already exists'
  Run-Case unknown-course 1 'Unknown course'
  Run-Case duplicate-course 1 'Duplicate course'
  Run-Case build-failure 1 'Course build refused'
  Run-Case stale-client 1 'final course marker'
  Run-Case anomaly 1 'Recording check failed'
  Run-Case tick-gap 1 'Recording check failed'
  Run-Case incomplete 1 'Recording check failed'
  Run-Case missing-initial 1 'pre-input initialPlayer'
  Run-Case snapshot-failure 1 'Server snapshot failed'
  Run-Case initial-mismatch 1 'Initial server resources disagree'
  Run-Case export-failure 1 'Client world export failed'
  Run-Case replay-failure 1 'Replay failed'
  Run-Case abort-continue 1 'Course ice - kept 3; rejected 0'
  Run-Case cleanup-failure 1 'Cancel replay cleanup failed:'
  Run-Case thaw 0
  Run-Case end-delayed 0 'bracket [4,4]'
  Run-Case end-uncertain 0 'bracket [0,1]'
  Run-Case corner 0 'collision exists: True'
  Run-Case corner-no-tie 1 'no witness'
  Run-Case end-negative 0 'bracket [-3,-3]'
  Run-Case end-airborne 0
  Run-Case heal 0
  Run-Case settle-exhaustion 1 'Start resources changed during settle: foodExhaustionLevel'
  Run-Case settle-timer 1 'Start resources changed during settle: foodTickTimer'
  Run-Case defaults 0 'Runs 8; ticks 400; seed base 1.'
  Run-Case all-courses 0 'Course edges - kept 1; rejected 0'
  Assert ($caseCount -gt 0) 'No cases selected.'
  Write-Output "Offline fixture tests passed ($caseCount cases). Evidence $sandbox"
} finally {
  $env:FIXTURE_TEST_RIG = $previousRig
  $env:FIXTURE_TEST_CASE = $previousCase
}
