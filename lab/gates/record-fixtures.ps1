param(
  [string[]]$Courses,
  [ValidateRange(1, 2147483647)][int]$Runs = 8,
  [ValidateRange(1, 2147483627)][int]$Ticks = 400,
  [int]$Seed = 1,
  [string]$Jar,
  [ValidateSet('peaceful', 'normal')][string]$ResourceDifficulty,
  [string]$ResourceProbeJar,
  [ValidateSet(20, 200)][int]$LandingRate
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
. "$PSScriptRoot\courses.ps1"
. "$PSScriptRoot\fixture-rig.ps1"

$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$result = Join-Path $PSScriptRoot "results\$stamp-fixtures"
$null = New-Item -ItemType Directory -Path $result
$report = Join-Path $result 'report.md'
$powershell = Join-Path $PSHOME 'powershell.exe'
$scripts = Join-Path $PSScriptRoot '..\..\scripts'
$botDir = Join-Path $Rig 'clients\bot'
$fixtures = Join-Path $PSScriptRoot '..\fixtures'
$landing = $PSBoundParameters.ContainsKey('LandingRate')
if ($landing) {
  if (-not $env:PATHTRACING_ROOT -or $ServerPort -in @(25565, 25566) -or
      -not (Test-Path -LiteralPath (Join-Path $Rig 'landing-motion.disposable') -PathType Leaf) -or
      -not (Test-Path -LiteralPath (Join-Path $Rig 'resource-pair.disposable') -PathType Leaf)) {
    throw 'Landing capture requires a marked disposable copy and its own port pair.'
  }
  if ($ResourceDifficulty -or -not $ResourceProbeJar -or -not $Jar -or $Runs -ne 1 -or
      $Ticks -ne 400 -or $Seed -ne 1 -or ($Courses -join ',') -cne 'edges,full-blocks,ice') {
    throw 'Frozen landing capture requires -Courses edges,full-blocks,ice -Runs 1 -Ticks 400 -Seed 1, -Jar and -ResourceProbeJar.'
  }
  $fixtures = Join-Path $result 'capture'
} elseif ($ResourceDifficulty) {
  if (-not $env:PATHTRACING_ROOT -or $ServerPort -in @(25565, 25566) -or
      -not (Test-Path -LiteralPath (Join-Path $Rig 'resource-pair.disposable') -PathType Leaf)) {
    throw 'Resource capture requires a marked disposable copy and its own port pair.'
  }
  if (-not $ResourceProbeJar -or $Runs -ne 1 -or $Ticks -ne 80 -or $Seed -ne 1 -or
      $Courses.Count -ne 1 -or $Courses[0] -cne 'full-blocks') {
    throw 'Frozen resource pair requires -Courses full-blocks -Runs 1 -Ticks 80 -Seed 1 and -ResourceProbeJar.'
  }
  $fixtures = Join-Path $result 'pair'
} elseif ($ResourceProbeJar) { throw '-ResourceProbeJar requires -ResourceDifficulty.' }
$prefix = 'l0_' + $stamp
$timeout = if ($landing) { 60 } else { [int][Math]::Min(2147400, [Math]::Ceiling($Ticks / 10.0) + 60) }
$locked = $false
$exitCode = 1
$started = @{}
$commands = [Collections.Generic.List[string]]::new()
$notes = [Collections.Generic.List[string]]::new()
$artifacts = [Collections.Generic.List[string]]::new()
$outcomes = [Collections.Generic.List[object]]::new()
$selected = @()
$activeRecording = $null
$completed = $false
$originalGameMode = $null
$recordTicks = $Ticks + 20
$comparableEnds = 0
$resourceProbe = $null
$previousProbeOptIn = $env:PATHTRACING_RESOURCE_PROBE
$previousLandingOptIn = $env:PATHTRACING_LANDING_PROBE

function Cleanup([string]$Name, [scriptblock]$Action) {
  try { & $Action } catch { Note "$Name cleanup failed: $($_.Exception.Message)" }
}
function Axis-TieCollision($Step) {
  $xb = [string]$Step.requested.xBits; $zb = [string]$Step.requested.zBits
  if ($xb -cnotmatch '^[0-9a-f]{16}$' -or $zb -cnotmatch '^[0-9a-f]{16}$') { return $false }
  # Clear exactly the sign bit, retaining all exponent and significand bits.
  $xm = ([Convert]::ToInt32($xb.Substring(0,1),16) -band 7).ToString('x') + $xb.Substring(1)
  $zm = ([Convert]::ToInt32($zb.Substring(0,1),16) -band 7).ToString('x') + $zb.Substring(1)
  $xm -cne '0000000000000000' -and $xm -ceq $zm -and
    (($xb -cne $Step.result.xBits) -xor ($zb -cne $Step.result.zBits))
}

function Signed-TickOffset([long]$QueryTime, [long]$ClientTime) {
  $period = 2147483647L
  $offset = $QueryTime - ($ClientTime % $period)
  if ($offset -gt 1073741823) { $offset -= $period }
  if ($offset -lt -1073741823) { $offset += $period }
  $offset
}
function At-Rest($Row) {
  $Row.onGround -eq $true -and $null -ne $Row.velocity -and
    $null -ne $Row.velocity.x -and $null -ne $Row.velocity.z -and
    [double]$Row.velocity.x -eq 0 -and [double]$Row.velocity.z -eq 0
}
function Require-StableResources($First, $Second) {
  foreach ($field in @('Health', 'foodSaturationLevel', 'foodExhaustionLevel')) {
    $a = [BitConverter]::ToUInt32([BitConverter]::GetBytes([single]$First.$field), 0)
    $b = [BitConverter]::ToUInt32([BitConverter]::GetBytes([single]$Second.$field), 0)
    if ($a -ne $b) { throw "Start resources changed during settle: $field." }
  }
  foreach ($field in @('foodLevel', 'foodTickTimer')) {
    if ($First.$field -ne $Second.$field) { throw "Start resources changed during settle: $field." }
  }
}
function Restore-Start([string]$Name, [string]$Start) {
  Assert-Absent
  $null = Require (Rcon ('tp ' + $BotName + ' ' + $Start.Replace(',', ' '))) 'Thaw teleport'
  $reply = Require (Rcon "attribute $BotName minecraft:max_health get") 'Maximum health query'
  if ($reply.Text -notmatch '(-?\d+(?:\.\d+)?)\s*$') { throw 'Cannot read maximum health.' }
  $maximum = [single]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
  $clock = [Diagnostics.Stopwatch]::StartNew()
  do {
    $state = Server-Snapshot $Name
    if ([single]$state.Health -eq $maximum -and $state.foodTickTimer -eq 0 -and $state.TicksFrozen -eq 0) { return $state }
    if ([single]$state.Health -ne $maximum -or $state.foodTickTimer -ne 0) {
      $null = Require (Rcon "effect give $BotName instant_health") 'Heal bot'
    }
    if ($clock.Elapsed.TotalSeconds -ge 60) { throw 'Bot did not reach maximum health, foodTickTimer=0 and ticksFrozen=0.' }
    Start-Sleep -Milliseconds 100
    Assert-Absent
  } while ($true)
}

function Export-World([string]$Name, [int[]]$Min, [int[]]$Max, [switch]$Readiness) {
  $path = Join-Path $botDir "pathtracing\fixtures\$Name.world.jsonl"
  if (-not $Readiness) { $artifacts.Add($path) }
  $text = 'export ' + $Name + ' ' + (($Min + $Max) -join ' ')
  $null = Require (Bot $text ("Exported ${Name}: \d+ blocks\.") 120) 'Client world export'
  if (-not (Test-Path -LiteralPath $path)) { throw "Export completion has no file: $path" }
  if (-not $Readiness) { Save-Artifact $path }
  $path
}
function Server-Snapshot([string]$Name) {
  $reply = Require (Invoke-Script (Join-Path $scripts 'server-state.ps1') @{ PlayerName = $BotName }) 'Server snapshot'
  $path = Join-Path $result "$Name.server.json"
  Set-Content -LiteralPath $path -Value $reply.Text -Encoding ASCII
  $state = $reply.Text | ConvertFrom-Json
  foreach ($field in @('Health', 'foodLevel', 'foodSaturationLevel', 'foodExhaustionLevel', 'foodTickTimer', 'TicksFrozen', 'playerGameType', 'gameTimeBefore', 'gameTimeAfter')) {
    if ($null -eq $state.PSObject.Properties[$field]) { throw "Server snapshot missing $field; see $($reply.File)." }
  }
  $state
}
function Set-Float($Object, [string]$Name, $Value) {
  $number = [single]$Value
  $bits = [BitConverter]::ToUInt32([BitConverter]::GetBytes($number), 0).ToString('x8')
  $Object | Add-Member -NotePropertyName $Name -NotePropertyValue $number -Force
  $Object | Add-Member -NotePropertyName ($Name + 'Bits') -NotePropertyValue $bits -Force
}
function Server-Resources($Object, $State) {
  Set-Float $Object health $State.Health
  Set-Float $Object saturationLevel $State.foodSaturationLevel
  Set-Float $Object exhaustion $State.foodExhaustionLevel
  $Object | Add-Member -NotePropertyName foodLevel -NotePropertyValue ([int]$State.foodLevel) -Force
  $Object | Add-Member -NotePropertyName foodTickTimer -NotePropertyValue ([int]$State.foodTickTimer) -Force
}
function New-ResourceInputs([string]$Path, [string]$Start) {
  $rows = @('{"start":[' + $Start + ']}')
  $rows += for ($t = 0; $t -lt 80; $t++) {
    $jump = if ($t -in @(0, 20, 40)) { 'true' } else { 'false' }
    '{"t":' + $t + ',"forward":false,"back":false,"left":false,"right":false,"jump":' + $jump +
      ',"sneak":false,"sprint":false,"yaw":0,"pitch":0}'
  }
  $null = New-Item -ItemType Directory -Path (Split-Path -Parent $Path) -Force
  $rows | Set-Content -LiteralPath $Path -Encoding ASCII
}
function Resource-Command([string]$Command, [string]$Expected) {
  $reply = Require (Rcon $Command) $Command
  if ($reply.Text.Trim() -cne $Expected) { throw "Resource command refused. See $($reply.File)." }
}
function Prepare-ResourceStart {
  $display = $ResourceDifficulty.Substring(0,1).ToUpperInvariant() + $ResourceDifficulty.Substring(1)
  $reply = Require (Rcon "difficulty $ResourceDifficulty") 'Set resource difficulty'
  if ($reply.Text.Trim() -cnotin @("The difficulty has been set to $display", "The difficulty did not change; it is already set to $display")) {
    throw "Difficulty refused. See $($reply.File)."
  }
  Resource-Command 'gamerule natural_health_regeneration true' 'Gamerule natural_health_regeneration is now set to: true'
  Resource-Command 'gamerule spawn_mobs false' 'Gamerule spawn_mobs is now set to: false'
  Resource-Command "effect give $BotName minecraft:saturation 1 20 true" "Applied effect Saturation to $BotName"
  Resource-Command "effect give $BotName minecraft:hunger 1 38 true" "Applied effect Hunger to $BotName"
  # Both effects last twenty server ticks. The start snapshot and probe enforce their completion.
  Start-Sleep -Seconds 2
}
function Seal-Landing([string]$Run) {
  $client = Join-Path $botDir "pathtracing/recordings/$Run.client-motion.csv"
  $server = Join-Path $Rig "server/pathtracing-phase/$Run.server-motion.csv"
  $resources = Join-Path $Rig "server/pathtracing-phase/$Run.csv"
  try {
    try { Resource-Command "ptphase resources stop $Run" "Resource capture saved $Run" }
    finally { Resource-Command "ptlanding stop $Run" "Landing capture sealed $Run" }
    $deadline = [Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath $client -PathType Leaf) -or
           -not (Test-Path -LiteralPath $server -PathType Leaf)) {
      if ($deadline.Elapsed.TotalSeconds -ge 10) { throw "Missing ordered end fence for $Run." }
      Start-Sleep -Milliseconds 50
    }
  } finally {
    Save-Artifact $client
    Save-Artifact "$client.tmp"
    if (-not (Test-Path -LiteralPath $client -PathType Leaf)) {
      Save-Artifact (Join-Path $botDir "pathtracing/recordings/$Run.client-motion.partial.csv")
    }
    Save-Artifact $server
    Save-Artifact "$server.tmp"
    if (Test-Path -LiteralPath $resources -PathType Leaf) {
      Copy-Item -LiteralPath $resources -Destination (Join-Path $result "$Run.resources.csv")
    }
  }
}
function Keep-Fixture([string]$Course, [int]$Run, [string]$Recording, [string]$World, $Player, $ServerStart, $ServerEnd, $LastTick) {
  $directory = Join-Path $fixtures $Course
  $null = New-Item -ItemType Directory -Force -Path $directory
  $base = if ($ResourceDifficulty) { [IO.Path]::GetFileNameWithoutExtension($Recording) } else { "$Course-$Run" }
  $recordTarget = Join-Path $directory "$base.jsonl"
  $worldTarget = Join-Path $directory "$base.world.jsonl"
  foreach ($path in @($recordTarget, $worldTarget, "$recordTarget.tmp", "$worldTarget.tmp")) {
    if (Test-Path -LiteralPath $path) { throw "Fixture output already exists: $path" }
  }
  # Health and food influence the client movement law. The server alone owns saturation:
  # ServerPlayer.doTick sends it when health/food change or saturation crosses zero.
  # The raw client observation stays in the recording header.
  if ([single]$Player.health -ne [single]$ServerStart.Health -or
      $Player.foodLevel -ne $ServerStart.foodLevel) {
    throw 'Initial server resources disagree with the settled client snapshot; pre-replay resources cannot initialize this fixture.'
  }
  Server-Resources $Player $ServerStart
  $end = [pscustomobject]@{ kind = 'serverEnd'; atRest = (At-Rest $LastTick) }
  Server-Resources $end $ServerEnd
  if ($null -eq $LastTick.PSObject.Properties['gameTime']) { throw 'Recording lacks final gameTime.' }
  # Vanilla time query gametime returns the game time modulo Integer.MAX_VALUE.
  $offsetMin = Signed-TickOffset $ServerEnd.gameTimeBefore $LastTick.gameTime
  $offsetMax = Signed-TickOffset $ServerEnd.gameTimeAfter $LastTick.gameTime
  $end | Add-Member -NotePropertyName serverTickOffsetMin -NotePropertyValue $offsetMin
  $end | Add-Member -NotePropertyName serverTickOffsetMax -NotePropertyValue $offsetMax
  $offset = if ($offsetMin -eq $offsetMax) { $offsetMin } else { $null }
  $end | Add-Member -NotePropertyName serverTickOffset -NotePropertyValue $offset
  Note "$Course run $Run serverEnd offset: $offset; bracket [$offsetMin,$offsetMax]; atRest: $($end.atRest)."
  $stream = [IO.File]::Open("$worldTarget.tmp", [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
  $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false))
  try {
    $writer.NewLine = "`n"
    $writer.WriteLine(($Player | ConvertTo-Json -Depth 20 -Compress))
    $writer.Write([IO.File]::ReadAllText($World))
    $writer.WriteLine(($end | ConvertTo-Json -Depth 5 -Compress))
  } finally { $writer.Dispose() }
  [IO.File]::Copy($Recording, "$recordTarget.tmp", $false)
  $worldPublished = $false
  try {
    [IO.File]::Move("$worldTarget.tmp", $worldTarget)
    $worldPublished = $true
    [IO.File]::Move("$recordTarget.tmp", $recordTarget)
  } catch {
    if ($worldPublished) { Cleanup 'Publication rollback' { Remove-Item -LiteralPath $worldTarget } }
    throw
  }
  if ($end.atRest -and $null -ne $offset) { $script:comparableEnds++ }
}

try {
  $available = @(Get-FixtureCourses)
  $requested = @($Courses | Where-Object { $_ } | ForEach-Object { $_.Split(',') })
  if ($requested.Count -eq 0) { $selected = $available }
  else {
    foreach ($name in $requested) {
      $course = @($available | Where-Object { $_.Name -ceq $name })
      if ($course.Count -ne 1) { throw "Unknown course '$name'. Choose $($available.Name -join ', ')." }
      if ($name -in $selected.Name) { throw "Duplicate course '$name'." }
      $selected += $course[0]
    }
  }
  # A course's seed range is stable when -Courses selects a subset or changes order.
  if ([long]$Seed + [long]$available.Count * $Runs - 1 -gt [int]::MaxValue) { throw 'Course seed range must fit a signed 32-bit seed.' }
  foreach ($course in $selected) {
    for ($i = 1; $i -le $Runs; $i++) {
      foreach ($suffix in @('.jsonl', '.world.jsonl', '.jsonl.tmp', '.world.jsonl.tmp')) {
        $target = Join-Path $fixtures "$($course.Name)\$($course.Name)-$i$suffix"
        if (Test-Path -LiteralPath $target) { throw "Fixture output already exists: $target" }
      }
    }
  }
  Enter-Rig 'record-fixtures'
  $locked = $true
  if ($ResourceDifficulty -or $landing) {
    if ((Test-Port $ServerHost $ServerPort) -or (Test-Port $ServerHost $RconPort) -or @(Bot-Processes).Count -gt 0) {
      throw 'Resource capture requires an idle disposable rig.'
    }
    $properties = @(Get-Content -LiteralPath (Join-Path $Rig 'server/server.properties'))
    foreach ($setting in @("server-port=$ServerPort", "rcon.port=$RconPort", 'server-ip=127.0.0.1',
        'enable-rcon=true', 'level-name=world', 'online-mode=false')) {
      $key = $setting.Split('=')[0] + '='
      $values = @($properties | Where-Object { $_.StartsWith($key) })
      if ($values.Count -ne 1 -or $values[0] -cne $setting) { throw "Disposable rig requires $setting." }
    }
    $save = Join-Path $Rig ("server/world/players/data/" + (Get-OfflineUuid $BotName) + '.dat')
    if ((Test-Path -LiteralPath $save) -or (Test-Path -LiteralPath "${save}_old")) {
      throw 'Resource capture requires a fresh PathBot save in each disposable copy.'
    }
    $serverMods = Join-Path $Rig 'server/mods'
    if (@(Get-ChildItem -LiteralPath $serverMods -Filter 'fabric-api-*.jar').Count -ne 1) {
      throw 'Install the reference Fabric API in the disposable server mods before recording.'
    }
    $probeTarget = Join-Path $serverMods 'phase-probe.jar'
    [IO.File]::Copy((Resolve-Path -LiteralPath $ResourceProbeJar).Path, $probeTarget)
    $resourceProbe = $probeTarget
    $env:PATHTRACING_RESOURCE_PROBE = '1'
    if ($landing) { $env:PATHTRACING_LANDING_PROBE = '1' }
  }
  Note "Rig: $Rig; courses: $($selected.Name -join ', '); Runs=$Runs; Ticks=$Ticks; Seed=$Seed."
  $serverOpen = Test-Port $ServerHost $ServerPort
  $online = @()
  if ($serverOpen -or (Test-Port $ServerHost $RconPort)) {
    $online = @(Wait-Rcon)
    if ($PlayerName -in $online) { $exitCode = 3; throw "$PlayerName is online. Refusing to disturb Julien." }
  }
  $botRunning = @(Bot-Processes).Count -gt 0 -or $BotName -in $online
  $target = Join-Path $botDir 'mods\pathtracing.jar'
  if ($Jar) {
    $source = (Resolve-Path -LiteralPath $Jar).Path
    $same = (Test-Path -LiteralPath $target) -and
      ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -eq (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash)
    if ($botRunning -and -not $same) { $exitCode = 3; throw 'The jar differs and the bot client is running. Close it before supplying -Jar.' }
    if (-not $botRunning -and -not $same) { Copy-Item -LiteralPath $source -Destination $target -Force }
  }
  if (-not (Test-Path -LiteralPath $target)) { $exitCode = 3; throw "Missing $target; supply -Jar." }
  Note "Bot jar SHA256: $((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash)."
  $options = Join-Path $botDir 'options.txt'
  if (-not (Test-Path -LiteralPath $options)) { $exitCode = 3; throw 'Missing bot options; run scripts\setup.ps1.' }
  Save-Artifact $options
  $lines = @(Get-Content -LiteralPath $options)
  foreach ($setting in @('autoJump:false', 'toggleSprint:false', 'toggleCrouch:false', 'sprintWindow:0')) {
    $values = @($lines | Where-Object { $_ -cmatch ('^' + $setting.Split(':')[0] + ':') })
    if ($values.Count -ne 1 -or $values[0] -cne $setting) { $exitCode = 3; throw "Bot options must contain exactly $setting." }
  }
  if (-not $serverOpen) { $null = Require (Invoke-Script (Join-Path $scripts 'start-server.ps1') @{} 30 'server') 'Server launch' }
  $online = @(Wait-Rcon)
  Assert-Absent
  if ($BotName -notin $online -and @(Bot-Processes).Count -eq 0) {
    $launcher = Join-Path $scripts 'launch-client.ps1'
    $arguments = @{ Role = 'bot' }
    if ($landing) {
      $parameters = (Get-Command $launcher).Parameters
      if ($LandingRate -eq 200 -and (-not $parameters.ContainsKey('FollowTickRate') -or
          -not $parameters.ContainsKey('FullTickDemand'))) { throw 'Integrate the lab client clock before the 200 TPS capture.' }
      if ($parameters.ContainsKey('FollowTickRate')) {
        $arguments.FollowTickRate = if ($LandingRate -eq 200) { 'true' } else { 'false' }
      }
      if ($LandingRate -eq 200) { $arguments.FullTickDemand = 'true' }
    }
    $null = Require (Invoke-Script $launcher $arguments 30 'bot') 'Bot launch'
  }
  $clock = [Diagnostics.Stopwatch]::StartNew()
  $ready = $false
  do {
    Assert-Absent
    if ($BotName -in (Players (Rcon 'list'))) {
      $reply = Bot 'rec fixture-ping' 'Usage: replay' 2 'Recorder/dumper:|Replay is running\.'
      if ($reply.Code -eq 0) { $ready = $true; break }
    }
    Start-Sleep -Milliseconds 500
  } while ($clock.Elapsed.TotalSeconds -lt 180)
  if (-not $ready) { $exitCode = 4; throw 'Timed out waiting for PathBot recorder.' }
  $null = Require (Bot 'stop' 'Stopped\.') 'Bot stop'
  Assert-Absent
  $originalGameMode = [int](Server-Snapshot 'original-mode').playerGameType
  if ($originalGameMode -notin 0..3) { throw 'Unrecognized original game mode.' }
  $null = Require (Rcon "gamemode survival $BotName") 'Survival mode'
  if ($landing) {
    $reply = Require (Rcon 'difficulty normal') 'Set normal difficulty'
    if ($reply.Text.Trim() -cnotin @('The difficulty has been set to Normal', 'The difficulty did not change; it is already set to Normal')) {
      throw "Difficulty refused. See $($reply.File)."
    }
    Resource-Command 'gamerule natural_health_regeneration true' 'Gamerule natural_health_regeneration is now set to: true'
    Resource-Command 'gamerule spawn_mobs false' 'Gamerule spawn_mobs is now set to: false'
    $null = Require (Rcon "tick rate $LandingRate") 'Set landing tick rate'
  }

  foreach ($course in $selected) {
    $name = $course.Name
    $courseIndex = [array]::IndexOf($available.Name, $name)
    try {
      Assert-Absent
      # Teleport first to load the course on the server, at the superflat ground height.
      $load = 'tp {0} {1} -60 {2}' -f $BotName, ($course.Min[0]+16.5).ToString([Globalization.CultureInfo]::InvariantCulture),
        ($course.Min[2]+4.5).ToString([Globalization.CultureInfo]::InvariantCulture)
      $null = Require (Rcon $load) 'Course chunk load teleport'
      Start-Sleep -Seconds 2
      Assert-Absent
      $built = Require (Rcon $course.Commands) 'Build course'
      if ($built.Text -match '(?im)unknown|incorrect|not loaded|too many|outside|invalid|error') { throw "Course build refused; see $($built.File)." }
      $null = Require (Rcon ('tp ' + $BotName + ' ' + $course.Start.Replace(',', ' '))) 'Course start teleport'
      Start-Sleep -Seconds 2
      $probe = Join-Path $botDir "pathtracing\fixtures\${prefix}_${name}_ready.world.jsonl"
      try {
        $null = Export-World "${prefix}_${name}_ready" $course.Min $course.Max -Readiness
        $marker = $course.Marker -join ','
        $seen = @(Get-Content -LiteralPath $probe | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
          $_.kind -eq 'block' -and ($_.at -join ',') -eq $marker -and $_.block -eq 'minecraft:gold_block'
        })
        if ($seen.Count -ne 1) { throw 'Client readiness export did not see the final course marker.' }
      } finally {
        foreach ($readyPath in @($probe, "$probe.tmp")) {
          Cleanup 'Readiness export' { if (Test-Path -LiteralPath $readyPath) { Remove-Item -LiteralPath $readyPath } }
        }
      }
    } catch {
      if ($exitCode -eq 3 -or $landing) { throw }
      for ($i = 1; $i -le $Runs; $i++) { $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $false; Reason = $_.Exception.Message }) }
      continue
    }
    for ($i = 1; $i -le $Runs; $i++) {
      $runName = "${prefix}_${name}_$i"
      $path = Join-Path $botDir "pathtracing\recordings\$runName.jsonl"
      $artifacts.Add($path)
      try {
        $runSeed = [int]([long]$Seed + [long]$courseIndex * $Runs + $i - 1)
        $inputPath = Join-Path $botDir "pathtracing\inputs\$runName.jsonl"
        if ($ResourceDifficulty) {
          New-ResourceInputs $inputPath $course.Start
          Prepare-ResourceStart
        } elseif ($name -eq 'corner') {
          $null = Require (Invoke-Script (Join-Path $PSScriptRoot 'corner-inputs.ps1') @{
            Name = $runName; Ticks = $Ticks; Start = $course.Start
          }) 'Generate corner inputs'
        } else {
          $null = Require (Invoke-Script (Join-Path $PSScriptRoot 'new-inputs.ps1') @{
            Name = $runName; Ticks = $Ticks; Seed = $runSeed; Start = $course.Start; SneakPercent = $course.SneakPercent
          }) 'Generate inputs'
        }
        # Preserve the last angles exactly and release every key for twenty recorded ticks.
        $zero = (Get-Content -LiteralPath $inputPath | Select-Object -Last 1).Replace(':true', ':false')
        $tail = for ($t = $Ticks; $t -lt $recordTicks; $t++) { $zero -replace '"t":\d+', ('"t":' + $t) }
        Add-Content -LiteralPath $inputPath -Value $tail -Encoding ASCII
        if ($landing) {
          Copy-Item -LiteralPath $inputPath -Destination (Join-Path $result "$runName.inputs.jsonl")
        } else { Save-Artifact $inputPath }
        Assert-Absent
        $serverStart = Restore-Start "$runName-start" $course.Start
        $activeRecording = $path
        $settleReply = Bot "replay $runName $runName wait" ("Settled ${runName};") $timeout
        if ($settleReply.Text -match '(?i)Aborted') { $activeRecording = $null }
        $null = Require $settleReply 'Replay settle'
        $serverSettled = Server-Snapshot "$runName-settled"
        Require-StableResources $serverStart $serverSettled
        if ($landing) {
          Resource-Command "ptphase resources start $BotName $runName" "Resource capture started $runName"
          Resource-Command "ptlanding start $BotName $runName" "Landing capture started $runName"
        }
        if ($ResourceDifficulty) {
          $resourcePath = Join-Path $Rig "server/pathtracing-phase/$runName.csv"
          Resource-Command "ptphase resources start $BotName $runName" "Resource capture started $runName"
        }
        try {
          $replayReply = Bot 'rec continue' ("Saved ${runName}: $recordTicks ticks\.") $timeout
          if ($landing) { $null = Require $replayReply 'Replay' }
        } finally {
          if ($landing) {
            Save-Artifact $path
            Seal-Landing $runName
          }
        }
        if ($replayReply.Text -match '(?i)Aborted') { $activeRecording = $null }
        $null = Require $replayReply 'Replay'
        $activeRecording = $null
        if ($ResourceDifficulty) {
          Resource-Command "ptphase resources stop $runName" "Resource capture saved $runName"
          if (-not (Test-Path -LiteralPath $resourcePath -PathType Leaf)) { throw 'Missing resource trace.' }
          Save-Artifact $resourcePath
        }
        # First operation after the completion reply; the entity data itself is one tick.
        $serverEnd = Server-Snapshot "$runName-end"
        Save-Artifact $path
        $rows = @(Get-Content -LiteralPath $path | ForEach-Object { $_ | ConvertFrom-Json })
        $player = $rows[0].initialPlayer
        if ($null -eq $player -or $player.kind -ne 'player') { throw 'Recording lacks the pre-input initialPlayer snapshot; install the job 1.6 bot jar.' }
        $ticksRead = @($rows | Where-Object { $null -ne $_.PSObject.Properties['t'] })
        if ($ticksRead.Count -ne $recordTicks) { throw "Expected $recordTicks ticks, got $($ticksRead.Count)." }
        if ($player.ticksFrozen -ne 0) { throw 'Initial player ticksFrozen is not zero.' }
        if ($name -eq 'corner') {
          $ties = @($ticksRead | Where-Object {
            @($_.collisionSteps | Where-Object { Axis-TieCollision $_ }).Count -gt 0
          })
          Note "corner run ${i}: nonzero bit-exact |dx|=|dz| collision exists: $($ties.Count -gt 0); ticks: $($ties.t -join ',')."
          if ($ties.Count -eq 0) { throw 'corner: no witness with exactly one clipped horizontal axis.' }
        }
        $positions = @($player.position) + @($ticksRead | ForEach-Object { $_.position })
        $min = @(); $max = @()
        foreach ($axis in @('x', 'y', 'z')) {
          $bounds = $positions | ForEach-Object { $_.$axis } | Measure-Object -Minimum -Maximum
          $min += [int][Math]::Floor($bounds.Minimum) - 3
          $max += [int][Math]::Floor($bounds.Maximum) + 3
        }
        $world = Export-World $runName $min $max
        if ($landing) {
          $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $true; Reason = "Seed $runSeed; raw capture awaiting 1.6f verification." })
          continue
        }
        $check = Require (Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $path } $timeout) 'Recording check'
        if ($ResourceDifficulty -and -not (At-Rest $ticksRead[-1])) { throw 'Resource pair must end at rest.' }
        Keep-Fixture $name $i $path $world $player $serverStart $serverEnd $ticksRead[-1]
        if ($ResourceDifficulty) {
          Copy-Item -LiteralPath $resourcePath -Destination (Join-Path $fixtures "$name/$runName.resources.csv")
        }
        $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $true; Reason = "Seed $runSeed; $($check.File); export bounds $($min -join ',') to $($max -join ',')." })
      } catch {
        $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $false; Reason = $_.Exception.Message })
        if ($exitCode -eq 3 -or $landing) { throw }
      } finally {
        if ($ResourceDifficulty -or $landing) { Cleanup 'Cancel resource capture' { $null = Require (Rcon 'ptphase resources cancel') 'Cancel probe' } }
        if ($landing) {
          Cleanup 'Cancel landing capture' { $null = Require (Rcon 'ptlanding cancel') 'Cancel probe' }
          Save-Artifact (Join-Path $Rig "server/pathtracing-phase/$runName.server-motion.partial.csv")
        }
        Cleanup 'Cancel replay' {
          if ($activeRecording -and (Test-Path -LiteralPath "$activeRecording.tmp")) {
            $null = Require (Bot 'rec stop' ("Saved ${runName}:")) 'Cancel own replay'
          }
        }
        $activeRecording = $null
        Cleanup 'Restore start resources' { $null = Restore-Start "$runName-cleanup" $course.Start }
        Cleanup 'Recording artifact' { Save-Artifact $path }
        Cleanup 'Partial recording artifact' { Save-Artifact "$path.tmp" }
      }
    }
  }
  $completed = $true
} catch { Note $_.Exception.Message }
finally {
  if ($locked) {
    if ($null -ne $originalGameMode) {
      Cleanup 'Game mode' { $null = Require (Rcon ("gamemode " + @('survival', 'creative', 'adventure', 'spectator')[$originalGameMode] + " $BotName")) 'Restore game mode' }
    }
    foreach ($path in $artifacts) {
      Cleanup 'Artifact' { Save-Artifact $path }
      Cleanup 'Partial artifact' { Save-Artifact "$path.tmp" }
    }
    foreach ($role in @('bot', 'server')) {
      try { Stop-Owned $role } catch { Note "Cleanup failed: $($_.Exception.Message)" }
    }
    if ($landing) {
      foreach ($role in @('bot', 'server')) {
        $directory = if ($role -eq 'bot') { $botDir } else { Join-Path $Rig 'server' }
        $latest = Join-Path $directory 'logs/latest.log'
        if (Test-Path -LiteralPath $latest -PathType Leaf) {
          Cleanup "Save $role game log" { Copy-Item -LiteralPath $latest -Destination (Join-Path $result "$role-latest.log") }
        }
      }
    }
    if ($resourceProbe) {
      Cleanup 'Remove resource probe' { Remove-Item -LiteralPath $resourceProbe }
      $env:PATHTRACING_RESOURCE_PROBE = $previousProbeOptIn
      if ($landing) { $env:PATHTRACING_LANDING_PROBE = $previousLandingOptIn }
    }
    try { Exit-Rig } catch { Note "Lock release failed: $($_.Exception.Message)" }
  }
  $kept = @($outcomes | Where-Object { $_.Kept }).Count
  $rejected = @($outcomes | Where-Object { -not $_.Kept }).Count
  $success = $completed -and $selected.Count -gt 0 -and $kept -eq ([long]$selected.Count * $Runs) -and
    @($notes | Where-Object { $_ -match 'failed:|Timed out|Refusing|online' }).Count -eq 0
  if ($success) { $exitCode = 0 }
  $overall = if ($exitCode -eq 0) { 'Overall: PASS' } else { 'Overall: FAIL' }
  $body = @('# L0 fixture recording', '', "UTC $stamp", "Rig $Rig", "Runs $Runs; ticks $Ticks; seed base $Seed.",
    "Recordings kept $kept; rejected $rejected.", '',
    'Each course is a 32x32 enclosed patch at x=4096+128*courseIndex,z=4096 on the lab superflat.',
    'Client readiness requires a full loaded-chunk export and receipt of the final gold-block marker after construction.',
    'Server resources use one atomic data get entity per snapshot, including foodTickTimer.',
    "serverEnd snapshots with atRest true and a non-null offset: $comparableEnds of $kept.",
    'Every input sequence ends with 20 zero-input ticks. serverEnd atRest requires onGround and zero horizontal velocity in the last row.',
    'serverEnd includes signed serverTickOffset from the final recorded gameTime; the reader advances nonnegative offsets as zero-input ticks only with atRest true and a non-null offset.',
    'Game-time queries bracket the entity read; a bracket spanning ticks has a null offset and must be skipped.',
    'The client gameTime is the server-synchronized client clock; this is not proof of packet-level tick alignment.',
    'The initialPlayer header retains every client field. The world player line substitutes the pre-replay server resources.',
    'Before each run and during cleanup, heal as needed and wait for maximum health, foodTickTimer=0 and TicksFrozen=0.',
    'Replay pauses after settle for a second server snapshot. Every resource field must be bit-identical to the first snapshot before rec continue.',
    'A run is rejected if initial health or food disagree with the settled client snapshot.',
    'Saturation is taken from the verified server snapshot. The raw header retains the client observation.',
    'Courses are static snapshots after the replay.',
    'The water course includes still and flowing water. Bubble columns are omitted because the frozen world schema has no bubble hook.',
    'Recordings retain their original header and end trailer. Reader integration belongs to job 1.5e.',
    'Seeds are Seed+courseIndex*Runs+run-1 in the courses.ps1 declaration order, stable under course selection.', '')
  $body += 'Step-up mutant at 0.6f + 1 ulp: no vanilla geometry produces that candidate; equivalent on the game domain. Integrator replaces it at gate freeze.'
  if ($ResourceDifficulty) {
    $body += "1.5h $ResourceDifficulty case. Raw server resource trace and its recording and world are in pair/full-blocks for the independent Rust gate."
  }
  if ($landing) {
    $body = @('# 1.6f raw landing capture', '', "UTC $stamp", "Rig $Rig",
      "Normal difficulty; configured rate $LandingRate; seed base $Seed; $recordTicks replay ticks.",
      'PASS here means capture completed. The dedicated verifier adjudicates the evidence.')
  }
  foreach ($course in $selected) {
    $entries = @($outcomes | Where-Object { $_.Course -eq $course.Name })
    $body += "Course $($course.Name) - kept $(@($entries | Where-Object { $_.Kept }).Count); rejected $(@($entries | Where-Object { -not $_.Kept }).Count); not run $($Runs-$entries.Count)."
    foreach ($entry in $entries) {
      $status = if ($entry.Kept) { 'KEPT' } else { 'REJECTED' }
      $body += "- Run $($entry.Run) $status - $($entry.Reason)"
    }
    $body += ''
  }
  $body += @('Driver and cleanup', '') + @($notes | ForEach-Object { "- $_" })
  $body += @('', 'Command logs', '') + @($commands | ForEach-Object { "- [$_]($_)" })
  $body += @('', $overall)
  Set-Content -LiteralPath $report -Value $body -Encoding ASCII
}
Write-Output $report
Write-Output $overall
exit $exitCode
