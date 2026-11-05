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

function Export-World([string]$Name, [int[]]$Min, [int[]]$Max) {
  $path = Join-Path $botDir "pathtracing\fixtures\$Name.world.jsonl"
  $artifacts.Add($path)
  $text = 'export ' + $Name + ' ' + (($Min + $Max) -join ' ')
  $null = Require (Bot $text ("Exported ${Name}: \d+ blocks\.") 120) 'Client world export'
  if (-not (Test-Path -LiteralPath $path)) { throw "Export completion has no file: $path" }
  Save-Artifact $path
  $path
}
function Server-Snapshot([string]$Name) {
  $reply = Require (Invoke-Script (Join-Path $scripts 'server-state.ps1') @{ PlayerName = $BotName }) 'Server snapshot'
  $path = Join-Path $result "$Name.server.json"
  Set-Content -LiteralPath $path -Value $reply.Text -Encoding ASCII
  $state = $reply.Text | ConvertFrom-Json
  foreach ($field in @('Health', 'foodLevel', 'foodSaturationLevel', 'foodExhaustionLevel', 'Pos', 'Motion', 'FallDistance')) {
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
}
function Keep-Fixture([string]$Course, [int]$Run, [string]$Recording, [string]$World, $Player, $ServerStart, $ServerEnd) {
  $directory = Join-Path $fixtures $Course
  $null = New-Item -ItemType Directory -Force -Path $directory
  $recordTarget = Join-Path $directory "$Course-$Run.jsonl"
  $worldTarget = Join-Path $directory "$Course-$Run.world.jsonl"
  foreach ($path in @($recordTarget, $worldTarget, "$recordTarget.tmp", "$worldTarget.tmp")) {
    if (Test-Path -LiteralPath $path) { throw "Fixture output already exists: $path" }
  }
  # Preserve the full client snapshot in the recording header. Resources used by Rust come
  # from the pre-replay server queries, with their timing limitation stated in the report.
  if ([single]$Player.health -ne [single]$ServerStart.Health -or
      $Player.foodLevel -ne $ServerStart.foodLevel -or
      [single]$Player.saturationLevel -ne [single]$ServerStart.foodSaturationLevel) {
    throw 'Initial server resources disagree with the settled client snapshot; pre-replay resources cannot initialize this fixture.'
  }
  Server-Resources $Player $ServerStart
  $end = [pscustomobject]@{ kind = 'serverEnd' }
  Server-Resources $end $ServerEnd
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
    if ($worldPublished) { Remove-Item -LiteralPath $worldTarget }
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
      $probe = Export-World "${prefix}_${name}_ready" $course.Min $course.Max
      $marker = $course.Marker -join ','
      $seen = @(Get-Content -LiteralPath $probe | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
        $_.kind -eq 'block' -and ($_.at -join ',') -eq $marker -and $_.block -eq 'minecraft:gold_block'
      })
      if ($seen.Count -ne 1) { throw 'Client readiness export did not see the final course marker.' }
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
        $null = Require (Invoke-Script (Join-Path $PSScriptRoot 'new-inputs.ps1') @{
          Name = $runName; Ticks = $Ticks; Seed = $runSeed; Start = $course.Start
        }) 'Generate inputs'
        Save-Artifact (Join-Path $botDir "pathtracing\inputs\$runName.jsonl")
        Assert-Absent
        $serverStart = Server-Snapshot "$runName-start"
        $activeRecording = $path
        $null = Require (Bot "replay $runName $runName" ("Saved ${runName}: $Ticks ticks\.") $timeout) 'Replay'
        $activeRecording = $null
        # First operation after the completion reply. Separate RCON reads are not tick-atomic.
        $serverEnd = Server-Snapshot "$runName-end"
        Save-Artifact $path
        $rows = @(Get-Content -LiteralPath $path | ForEach-Object { $_ | ConvertFrom-Json })
        $player = $rows[0].initialPlayer
        if ($null -eq $player -or $player.kind -ne 'player') { throw 'Recording lacks the pre-input initialPlayer snapshot; install the job 1.6 bot jar.' }
        $ticksRead = @($rows | Where-Object { $null -ne $_.PSObject.Properties['t'] })
        if ($ticksRead.Count -ne $Ticks) { throw "Expected $Ticks ticks, got $($ticksRead.Count)." }
        $positions = @($player.position) + @($ticksRead | ForEach-Object { $_.position })
        $min = @(); $max = @()
        foreach ($axis in @('x', 'y', 'z')) {
          $bounds = $positions | ForEach-Object { $_.$axis } | Measure-Object -Minimum -Maximum
          $min += [int][Math]::Floor($bounds.Minimum) - 3
          $max += [int][Math]::Floor($bounds.Maximum) + 3
        }
        $world = Export-World $runName $min $max
        $check = Require (Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $path } $timeout) 'Recording check'
        Keep-Fixture $name $i $path $world $player $serverStart $serverEnd
        $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $true; Reason = "Seed $runSeed; $($check.File); export bounds $($min -join ',') to $($max -join ',')." })
      } catch {
        $outcomes.Add([pscustomobject]@{ Course = $name; Run = $i; Kept = $false; Reason = $_.Exception.Message })
        if ($exitCode -eq 3) { throw }
      } finally {
        if ($activeRecording -and (Test-Path -LiteralPath "$activeRecording.tmp")) {
          $stop = Require (Bot 'rec stop' ("Saved ${runName}:")) 'Cancel own replay'
          $activeRecording = $null
        }
        Save-Artifact $path
        Save-Artifact "$path.tmp"
      }
    }
  }
  $completed = $true
} catch { Note $_.Exception.Message }
finally {
  if ($locked) {
    foreach ($path in $artifacts) {
      try { Save-Artifact $path; Save-Artifact "$path.tmp" } catch { Note "Artifact cleanup failed: $($_.Exception.Message)" }
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
    'Server timing is unaligned. Start resources are queried before replay teleport/settling. End queries start immediately',
    'after the Saved reply is observed. The seven sequential queries span an unbounded number of server ticks.',
    'There is no tick-atomic snapshot or measured client/server tick offset. serverEnd is provisional resource evidence.',
    'The initialPlayer header retains every client field. The world player line substitutes the pre-replay server resources.',
    'A run is rejected if initial health, food or saturation disagree with that settled client snapshot.',
    'No health, hunger, inventory or effect reset is performed. Courses are static snapshots after the replay.',
    'The water course includes still and flowing water. Bubble columns are omitted because the frozen world schema has no bubble hook.',
    'Recordings retain their original header and end trailer. The current wt/law replay reader needs end-trailer support.',
    'Seeds are Seed+courseIndex*Runs+run-1 in the courses.ps1 declaration order, stable under course selection.', '')
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
