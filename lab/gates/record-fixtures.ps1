param(
  [string[]]$Courses,
  [ValidateRange(1, 2147483647)][int]$Runs = 8,
  [ValidateRange(1, 2147483647)][int]$Ticks = 400,
  [int]$Seed = 1,
  [string]$Jar
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
$prefix = 'l0_' + $stamp
$timeout = [int][Math]::Min(2147400, [Math]::Ceiling($Ticks / 10.0) + 60)
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
    ($xb -cne $Step.result.xBits -or $zb -cne $Step.result.zBits)
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
function Keep-Fixture([string]$Course, [int]$Run, [string]$Recording, [string]$World, $Player, $ServerStart, $ServerEnd, $LastTick) {
  $directory = Join-Path $fixtures $Course
  $null = New-Item -ItemType Directory -Force -Path $directory
  $recordTarget = Join-Path $directory "$Course-$Run.jsonl"
  $worldTarget = Join-Path $directory "$Course-$Run.world.jsonl"
  foreach ($path in @($recordTarget, $worldTarget, "$recordTarget.tmp", "$worldTarget.tmp")) {
    if (Test-Path -LiteralPath $path) { throw "Fixture output already exists: $path" }
  }
  # Preserve the full client snapshot in the recording header. Resources used by Rust come
  # from one pre-replay server entity snapshot, with its timing limitation stated in the report.
  if ([single]$Player.health -ne [single]$ServerStart.Health -or
      $Player.foodLevel -ne $ServerStart.foodLevel -or
      [single]$Player.saturationLevel -ne [single]$ServerStart.foodSaturationLevel) {
    throw 'Initial server resources disagree with the settled client snapshot; pre-replay resources cannot initialize this fixture.'
  }
  Server-Resources $Player $ServerStart
  $end = [pscustomobject]@{ kind = 'serverEnd' }
  Server-Resources $end $ServerEnd
  if ($null -eq $LastTick.PSObject.Properties['gameTime']) { throw 'Recording lacks final gameTime.' }
  # Vanilla time query gametime returns the game time modulo Integer.MAX_VALUE.
  $last = [long]$LastTick.gameTime % 2147483647
  $offsetMin = ([long]$ServerEnd.gameTimeBefore - $last + 2147483647) % 2147483647
  $offsetMax = ([long]$ServerEnd.gameTimeAfter - $last + 2147483647) % 2147483647
  $end | Add-Member -NotePropertyName serverTickOffsetMin -NotePropertyValue $offsetMin
  $end | Add-Member -NotePropertyName serverTickOffsetMax -NotePropertyValue $offsetMax
  $offset = if ($offsetMin -eq $offsetMax) { $offsetMin } else { $null }
  $end | Add-Member -NotePropertyName serverTickOffset -NotePropertyValue $offset
  Note "$Course run $Run serverEnd offset: $offset; bracket [$offsetMin,$offsetMax]. Compare resources only for an exact zero offset."
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
    $null = Require (Invoke-Script (Join-Path $scripts 'launch-client.ps1') @{ Role = 'bot' } 30 'bot') 'Bot launch'
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
      if ($exitCode -eq 3) { throw }
      for ($i = 1; $i -le $Runs; $i++) { $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $false; Reason = $_.Exception.Message }) }
      continue
    }
    for ($i = 1; $i -le $Runs; $i++) {
      $runName = "${prefix}_${name}_$i"
      $path = Join-Path $botDir "pathtracing\recordings\$runName.jsonl"
      $artifacts.Add($path)
      try {
        $runSeed = [int]([long]$Seed + [long]$courseIndex * $Runs + $i - 1)
        if ($name -eq 'corner') {
          $null = Require (Invoke-Script (Join-Path $PSScriptRoot 'corner-inputs.ps1') @{
            Name = $runName; Ticks = $Ticks; Start = $course.Start
          }) 'Generate corner inputs'
        } else {
          $null = Require (Invoke-Script (Join-Path $PSScriptRoot 'new-inputs.ps1') @{
            Name = $runName; Ticks = $Ticks; Seed = $runSeed; Start = $course.Start; SneakPercent = $course.SneakPercent
          }) 'Generate inputs'
        }
        Save-Artifact (Join-Path $botDir "pathtracing\inputs\$runName.jsonl")
        Assert-Absent
        # Move off any powder snow, then wait for the vanilla thaw (also on other courses).
        $null = Require (Rcon ('tp ' + $BotName + ' ' + $course.Start.Replace(',', ' '))) 'Thaw teleport'
        $thaw = [Diagnostics.Stopwatch]::StartNew()
        do {
          $serverStart = Server-Snapshot "$runName-start"
          if ($serverStart.TicksFrozen -eq 0) { break }
          if ($thaw.Elapsed.TotalSeconds -ge 60) { throw 'Bot did not thaw to ticksFrozen=0.' }
          Start-Sleep -Milliseconds 100
          Assert-Absent
        } while ($true)
        $activeRecording = $path
        $replayReply = Bot "replay $runName $runName" ("Saved ${runName}: $Ticks ticks\.") $timeout
        if ($replayReply.Text -match '(?i)Aborted') { $activeRecording = $null }
        $null = Require $replayReply 'Replay'
        $activeRecording = $null
        # First operation after the completion reply; the entity data itself is one tick.
        $serverEnd = Server-Snapshot "$runName-end"
        Save-Artifact $path
        $rows = @(Get-Content -LiteralPath $path | ForEach-Object { $_ | ConvertFrom-Json })
        $player = $rows[0].initialPlayer
        if ($null -eq $player -or $player.kind -ne 'player') { throw 'Recording lacks the pre-input initialPlayer snapshot; install the job 1.6 bot jar.' }
        $ticksRead = @($rows | Where-Object { $null -ne $_.PSObject.Properties['t'] })
        if ($ticksRead.Count -ne $Ticks) { throw "Expected $Ticks ticks, got $($ticksRead.Count)." }
        if ($player.ticksFrozen -ne 0) { throw 'Initial player ticksFrozen is not zero.' }
        if ($name -eq 'corner') {
          $ties = @($ticksRead | Where-Object {
            @($_.collisionSteps | Where-Object { Axis-TieCollision $_ }).Count -gt 0
          })
          Note "corner run ${i}: nonzero bit-exact |dx|=|dz| collision exists: $($ties.Count -gt 0); ticks: $($ties.t -join ',')."
        }
        $positions = @($player.position) + @($ticksRead | ForEach-Object { $_.position })
        $min = @(); $max = @()
        foreach ($axis in @('x', 'y', 'z')) {
          $bounds = $positions | ForEach-Object { $_.$axis } | Measure-Object -Minimum -Maximum
          $min += [int][Math]::Floor($bounds.Minimum) - 3
          $max += [int][Math]::Floor($bounds.Maximum) + 3
        }
        $world = Export-World $runName $min $max
        $check = Require (Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $path } $timeout) 'Recording check'
        Keep-Fixture $name $i $path $world $player $serverStart $serverEnd $ticksRead[-1]
        $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $true; Reason = "Seed $runSeed; $($check.File); export bounds $($min -join ',') to $($max -join ',')." })
      } catch {
        $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $false; Reason = $_.Exception.Message })
        if ($exitCode -eq 3) { throw }
      } finally {
        Cleanup 'Cancel replay' {
          if ($activeRecording -and (Test-Path -LiteralPath "$activeRecording.tmp")) {
            $null = Require (Bot 'rec stop' ("Saved ${runName}:")) 'Cancel own replay'
          }
        }
        $activeRecording = $null
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
    'serverEnd includes serverTickOffset from the final recorded gameTime; compare only when exactly zero.',
    'Game-time queries bracket the entity read; a bracket spanning ticks has a null offset and must be skipped.',
    'The client gameTime is the server-synchronized client clock; this is not proof of packet-level tick alignment.',
    'The initialPlayer header retains every client field. The world player line substitutes the pre-replay server resources.',
    'Start resources, including exhaustion and foodTickTimer, are assumed constant until input zero; no tick alignment is claimed there.',
    'A run is rejected if initial health, food or saturation disagree with that settled client snapshot.',
    'No health, hunger, inventory or effect reset is performed. Courses are static snapshots after the replay.',
    'The water course includes still and flowing water. Bubble columns are omitted because the frozen world schema has no bubble hook.',
    'Recordings retain their original header and end trailer. Apply jobs/1.6-l0-fixtures/law-reader.patch to the separate law reader.',
    'Seeds are Seed+courseIndex*Runs+run-1 in the courses.ps1 declaration order, stable under course selection.', '')
  $body += 'Step-up mutant at 0.6f + 1 ulp: no vanilla geometry produces that candidate; equivalent on the game domain. Integrator replaces it at gate freeze.'
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
