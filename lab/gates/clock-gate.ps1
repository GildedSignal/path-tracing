# Live only. Close both clients first, keep the lab server running, and set PATHTRACING_ROOT.
# .\lab\gates\clock-gate.ps1 -Jar .\bot\build\pathtracing.jar
# Frozen clock classification: all observed client-minus-server offset intervals must intersect.
# RCON queries are bracketed by flushed recorder counts, never by ClientLevel's extrapolated time.
param([string]$Jar = "$PSScriptRoot\..\..\bot\build\pathtracing.jar")
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
. "$PSScriptRoot\courses.ps1"
. "$PSScriptRoot\fixture-rig.ps1"

$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$result = Join-Path $PSScriptRoot "results\$stamp-clock"
$null = New-Item -ItemType Directory -Path $result
$scripts = Join-Path $PSScriptRoot '..\..\scripts'
$powershell = Join-Path $PSHOME 'powershell.exe'
$botDir = Join-Path $Rig 'clients\bot'
$commands = [Collections.Generic.List[string]]::new()
$notes = [Collections.Generic.List[string]]::new()
$started = @{}
$runs = [Collections.Generic.List[object]]::new()
$failures = [Collections.Generic.List[string]]::new()
$locked = $false
$exitCode = 1
$watch = [Diagnostics.Stopwatch]::StartNew()
$ticks = 800
$prefix = 'clock_' + $stamp
$activeRecording = $null

function Read-Progress([string]$Path) {
  $candidate = if (Test-Path -LiteralPath $Path) { $Path } else { "$Path.tmp" }
  if (-not (Test-Path -LiteralPath $candidate)) { return }
  try {
    $stream = [IO.FileStream]::new($candidate, [IO.FileMode]::Open, [IO.FileAccess]::Read,
      [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)
  } catch [IO.FileNotFoundException] {
    # Publication atomically changes the temporary name to the immutable final name.
    if ($candidate -ceq $Path) { throw }
    $stream = [IO.FileStream]::new($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
  }
  $reader = [IO.StreamReader]::new($stream)
  try { $text = $reader.ReadToEnd() } finally { $reader.Dispose() }
  # A writer may be between write(row) and write(LF). Only closed lines are observations.
  $lines = $text.Split("`n")
  for ($index = $lines.Length - 2; $index -ge 0; $index--) {
    if ($lines[$index] -match '^\{"t":') { return ($lines[$index] | ConvertFrom-Json) }
  }
}

function Measure-Clocks([object[]]$Samples) {
  if ($Samples.Count -lt 3) { throw 'Fewer than three clock samples inside the replay.' }
  for ($index = 1; $index -lt $Samples.Count; $index++) {
    if ($Samples[$index].Server -le $Samples[$index-1].Server -or
        $Samples[$index].ClientBefore -lt $Samples[$index-1].ClientAfter) {
      throw 'Clock samples wrapped or moved backwards.'
    }
  }
  $first = $Samples[0]
  $last = $Samples[-1]
  $serverTicks = $last.Server - $first.Server
  $minSeconds = $last.BeforeSeconds - $first.AfterSeconds
  $maxSeconds = $last.AfterSeconds - $first.BeforeSeconds
  if ($serverTicks -le 0 -or $minSeconds -le 0) { throw 'Clock sample span is not positive.' }
  $offsetMin = ($Samples | ForEach-Object { $_.ClientBefore - $_.Server } | Measure-Object -Maximum).Maximum
  $offsetMax = ($Samples | ForEach-Object { $_.ClientAfter - $_.Server } | Measure-Object -Minimum).Minimum
  $clientMin = $last.ClientBefore - $first.ClientAfter
  $clientMax = $last.ClientAfter - $first.ClientBefore
  if ($clientMin -le 0) { throw 'Clock query bounds consume the measured client tick span.' }
  [pscustomobject]@{
    Samples = $Samples.Count; ServerTicks = $serverTicks
    ClientTicksMin = $clientMin; ClientTicksMax = $clientMax
    RatioMin = $clientMin / [double]$serverTicks; RatioMax = $clientMax / [double]$serverTicks
    ClientTpsMin = $clientMin / $maxSeconds; ClientTpsMax = $clientMax / $minSeconds
    ServerTpsMin = $serverTicks / $maxSeconds; ServerTpsMax = $serverTicks / $minSeconds
    OffsetMin = $offsetMin; OffsetMax = $offsetMax; Lockstep = $offsetMin -le $offsetMax
  }
}

function New-ClockInput($Course) {
  $name = "${prefix}_$($Course.Name)"
  $path = Join-Path $botDir "pathtracing\inputs\$name.jsonl"
  $null = New-Item -ItemType Directory -Force (Split-Path -Parent $path)
  $rows = [Collections.Generic.List[string]]::new()
  $rows.Add('{"start":[' + $Course.Start + ']}')
  for ($tick = 0; $tick -lt $ticks; $tick++) {
    # Held jump produces repeated sprint jumps without creative flight double-taps.
    # The edge course uses held sneak to exercise its back-off. The final 200 ticks coast.
    $moving = $tick -lt 600
    $edge = $Course.Name -eq 'edges'
    $yaw = if ($edge) { 0 } else { 180 * ([int][Math]::Floor($tick / 50.0) % 2) }
    $rows.Add((@{ t = $tick; forward = $moving; back = $false; left = $moving -and $edge; right = $false
      jump = $moving -and -not $edge; sneak = $moving -and $edge; sprint = $moving -and -not $edge
      yaw = $yaw; pitch = 0 } | ConvertTo-Json -Compress))
  }
  [IO.File]::WriteAllLines($path, $rows, [Text.UTF8Encoding]::new($false))
  Save-Artifact $path
  $name
}

function Compare-ClockRecording([string]$Baseline, [string]$Recording) {
  # Compare movement state after removing absolute clock coordinates. The shared comparator
  # excludes tickCount. Fixtures 1.6c also writes gameTime; remove only that absolute counter.
  $paths = @()
  foreach ($path in @($Baseline, $Recording)) {
    $target = Join-Path $result ([IO.Path]::GetFileNameWithoutExtension($path) + '.compare.jsonl')
    $lines = [IO.File]::ReadAllLines($path) | ForEach-Object { $_ -replace '"gameTime":\d+,', '' }
    [IO.File]::WriteAllLines($target, [string[]]$lines, [Text.UTF8Encoding]::new($false))
    $paths += $target
  }
  Invoke-Script (Join-Path $PSScriptRoot 'compare-recordings.ps1') @{ A = $paths[0]; B = $paths[1] }
}

function Start-ClockBot([string]$Follow) {
  Assert-Absent
  if ($BotName -in (Players (Rcon 'list')) -or @(Bot-Processes).Count -gt 0) {
    throw 'Close the existing bot before the clock gate. The gate only stops its own processes.'
  }
  $null = Require (Invoke-Script (Join-Path $scripts 'launch-client.ps1') @{
    Role = 'bot'; FollowTickRate = $Follow
  } 30 'bot') 'Bot launch'
  $deadline = [Diagnostics.Stopwatch]::StartNew()
  do {
    Assert-Absent
    if ($BotName -in (Players (Rcon 'list'))) { break }
    if ($deadline.Elapsed.TotalSeconds -ge 180) { throw 'Bot did not join.' }
    Start-Sleep -Milliseconds 500
  } while ($true)
  $arguments = Get-Content -LiteralPath (Join-Path $botDir 'launch.args')
  if ('"-Dpathtracing.followTickRate=' + $Follow + '"' -cnotin $arguments) { throw 'Launch property mismatch.' }
  $null = Require (Bot 'stop' 'Stopped\.') 'Bot readiness'
  $modeReply = Require (Rcon "data get entity $BotName playerGameType") 'Original bot mode'
  if ($modeReply.Text.Trim() -notmatch 'entity data: ([0-3])$') { throw 'Cannot read original bot mode.' }
  $script:originalMode = @('survival', 'creative', 'adventure', 'spectator')[[int]$Matches[1]]
  # Creative keeps health/food constant across the clock experiment. Flight is cleared by
  # entering survival first. Inputs never double-tap jump, use items, attack or break blocks.
  $null = Require (Rcon @("gamemode survival $BotName", "gamemode creative $BotName")) 'Clock fixture mode'
}

function Stop-ClockBot {
  $owned = $started.ContainsKey('bot')
  try {
    if ($owned -and $null -ne $script:originalMode) {
      $null = Require (Rcon "gamemode $script:originalMode $BotName") 'Restore bot mode'
      $script:originalMode = $null
    }
  } finally { Stop-Owned 'bot' }
  if ($owned) {
    $deadline = [Diagnostics.Stopwatch]::StartNew()
    while ($BotName -in (Players (Rcon 'list'))) {
      if ($deadline.Elapsed.TotalSeconds -ge 30) { throw 'Owned bot did not leave the server after exit.' }
      Start-Sleep -Milliseconds 50
    }
  }
}

function Distribution([double[]]$Values) {
  $sorted = @($Values | Sort-Object)
  if ($sorted.Count -eq 0) { return 'n=0' }
  $n = $sorted.Count
  "n=$n min=$($sorted[0]) median=$($sorted[[Math]::Ceiling(0.5*$n)-1]) p90=$($sorted[[Math]::Ceiling(0.9*$n)-1]) p99=$($sorted[[Math]::Ceiling(0.99*$n)-1]) max=$($sorted[-1])"
}

function Build-ClockCourse($Course) {
  Assert-Absent
  $null = Require (Rcon ('tp ' + $BotName + ' ' + $Course.Start.Replace(',', ' '))) 'Course load'
  Start-Sleep -Seconds 2
  $built = Require (Rcon $Course.Commands) 'Course build'
  if ($built.Text -match '(?im)unknown|incorrect|not loaded|too many|outside|invalid|error') { throw 'Course construction refused.' }
  $null = Require (Rcon ('tp ' + $BotName + ' ' + $Course.Start.Replace(',', ' '))) 'Course start'
  Start-Sleep -Seconds 2
  $name = "${prefix}_$($Course.Name)_ready"
  $bounds = ($Course.Min + $Course.Max) -join ' '
  $null = Require (Bot "export $name $bounds" ("Exported ${name}: \d+ blocks\.")) 'Course readiness export'
  $path = Join-Path $botDir "pathtracing\fixtures\$name.world.jsonl"
  Save-Artifact $path
  $marker = $Course.Marker -join ','
  $found = @(Get-Content -LiteralPath $path | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
    $_.kind -eq 'block' -and ($_.at -join ',') -eq $marker -and $_.block -eq 'minecraft:gold_block'
  })
  if ($found.Count -ne 1) { throw 'Client did not receive the course marker.' }
}

function Replay-Clock($Course, [string]$InputName, [int]$Rate, [string]$Follow) {
  Assert-Absent
  $null = Require (Rcon "tick rate $Rate") 'Set tick rate'
  # Rate packets must arrive before the initial teleport and settling. This is setup only.
  Start-Sleep -Seconds 1
  $name = "${InputName}_${Follow}_$Rate"
  $path = Join-Path $botDir "pathtracing\recordings\$name.jsonl"
  $script:activeRecording = $path
  $samples = [Collections.Generic.List[object]]::new()
  $null = Require (Bot "replay $InputName $name" ("Replay ${InputName}: waiting for")) 'Replay start'
  $deadline = [Diagnostics.Stopwatch]::StartNew()
  while (-not (Test-Path -LiteralPath $path)) {
    if ($deadline.Elapsed.TotalSeconds -gt 60) { throw "Replay $name exceeded 60 seconds." }
    $beforeSeconds = $watch.Elapsed.TotalSeconds
    $before = Read-Progress $path
    if ($null -ne $before) {
      $reply = Require (Rcon 'time query gametime') 'Server game time'
      $after = Read-Progress $path
      $afterSeconds = $watch.Elapsed.TotalSeconds
      if ($reply.Text.Trim() -cnotmatch '^The game time is (\d+) tick\(s\)$') { throw 'Unrecognized game-time reply.' }
      $server = [long]$Matches[1]
      # Drop a final query if completion occurred during it. Samples cover only replay ticks.
      if (-not (Test-Path -LiteralPath $path) -and $null -ne $after) {
        $samples.Add([pscustomobject]@{ Server = $server; ClientBefore = [long]$before.tickCount
          ClientAfter = [long]$after.tickCount; BeforeSeconds = $beforeSeconds; AfterSeconds = $afterSeconds })
      }
    }
    Start-Sleep -Milliseconds 50
  }
  Save-Artifact $path
  $samples | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $result "$name.clocks.json") -Encoding ASCII
  $clock = Measure-Clocks $samples.ToArray()
  $check = Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $path }
  $rows = @(Get-Content -LiteralPath $path | ForEach-Object { $_ | ConvertFrom-Json })
  $tickRows = @($rows | Where-Object { $null -ne $_.PSObject.Properties['t'] })
  if ($tickRows.Count -ne $ticks) { throw "Expected $ticks replay rows." }
  $corrections = @($tickRows | ForEach-Object { $_.anomaly } | Where-Object { $null -ne $_ })
  $run = [pscustomobject]@{ Course = $Course.Name; Rate = $Rate; Follow = $Follow; Path = $path
    Clock = $clock; Corrections = $corrections.Count; Identical = $null; Comparison = '' }
  $runs.Add($run)
  if ($check.Code -ne 0) { $failures.Add("${name}: $(Diagnostic $check)") }
  if ($corrections.Count -gt 0) {
    $failures.Add("${name}: $($corrections.Count) server correction/impulse packets.")
    $corrections | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $result "$name.corrections.json") -Encoding ASCII
  }
  Note "$name ratio [$($clock.RatioMin), $($clock.RatioMax)]; lockstep $($clock.Lockstep); corrections $($run.Corrections)."
  $run
}

try {
  Enter-Rig 'clock-gate'
  $locked = $true
  Assert-Absent
  if ($BotName -in (Players (Rcon 'list')) -or @(Bot-Processes).Count -gt 0) { throw 'Close the bot before running clock-gate.' }
  foreach ($setting in @('toggleSprint:false', 'toggleCrouch:false', 'autoJump:false', 'sprintWindow:0')) {
    $key = $setting.Split(':')[0]
    $values = @(Get-Content -LiteralPath (Join-Path $botDir 'options.txt') | Where-Object { $_ -cmatch ('^' + $key + ':') })
    if ($values.Count -ne 1 -or $values[0] -cne $setting) { throw "Bot options must contain exactly $setting." }
  }
  Copy-Item -LiteralPath (Resolve-Path -LiteralPath $Jar).Path -Destination (Join-Path $botDir 'mods\pathtracing.jar') -Force
  $null = Require (Rcon 'tick rate 20') 'Initial tick rate'
  $courses = @(Get-FixtureCourses | Where-Object { $_.Name -in @('full-blocks', 'ice', 'edges') })
  if ($courses.Count -ne 3) { throw 'Missing clock courses.' }
  Start-ClockBot 'true'
  foreach ($course in $courses) {
    Build-ClockCourse $course
    $inputName = New-ClockInput $course
    $baseline = $null
    foreach ($rate in @(20, 100, 200)) {
      $run = Replay-Clock $course $inputName $rate 'true'
      if ($rate -eq 20) { $baseline = $run }
      $comparison = Compare-ClockRecording $baseline.Path $run.Path
      $run.Identical = $comparison.Code -eq 0
      $run.Comparison = $comparison.Text.Trim()
      if ($run.Clock.Lockstep -and -not $run.Identical) { $failures.Add("$($course.Name) at ${rate}: $(Diagnostic $comparison)") }
      if ($rate -eq 20 -and -not $run.Clock.Lockstep) { $failures.Add("$($course.Name): 20 TPS baseline is out of lockstep.") }
    }
  }
  $null = Require (Rcon 'tick rate 20') 'Rate before off control'
  Stop-ClockBot
  Start-ClockBot 'false'
  $course = $courses[0]
  $control = Replay-Clock $course "${prefix}_$($course.Name)" 100 'false'
  $clock = $control.Clock
  if ($clock.ClientTpsMin -gt 20 -or $clock.ClientTpsMax -lt 20 -or $clock.ClientTpsMax -ge 100 -or
      $clock.ServerTpsMin -gt 100 -or $clock.ServerTpsMax -lt 100 -or $clock.Lockstep) {
    $failures.Add('Property-off control did not establish 20 client TPS against 100 server TPS.')
  }
  $exitCode = 0
} catch { $failures.Add($_.Exception.Message) }
finally {
  if ($locked) {
    # Restore before cancelling a replay or stopping the owned client. Cleanup failures stay red.
    try { $null = Require (Rcon 'tick rate 20') 'Restore tick rate 20' } catch { $failures.Add($_.Exception.Message) }
    try { Stop-ClockBot } catch { $failures.Add($_.Exception.Message) }
    if ($script:activeRecording) {
      try { Save-Artifact $script:activeRecording; Save-Artifact "$script:activeRecording.tmp" }
      catch { $failures.Add("Recording evidence copy failed: $($_.Exception.Message)") }
    }
    try { Exit-Rig } catch { $failures.Add($_.Exception.Message) }
  }
  if ($failures.Count -gt 0) { $exitCode = 1 }
  $highest = @(@(20, 100, 200) | Where-Object {
    $rate = $_
    $entries = @($runs | Where-Object { $_.Follow -eq 'true' -and $_.Rate -eq $rate })
    $entries.Count -eq 3 -and @($entries | Where-Object { -not $_.Clock.Lockstep }).Count -eq 0
  } | Measure-Object -Maximum).Maximum
  $body = @('# Client clock gate', '', "Rig $Rig", "Elapsed seconds $($watch.Elapsed.TotalSeconds)",
    "Highest tested lockstep setting across all three courses: $highest TPS", '',
    'Frozen pass line: bit-identical tick rows wherever lockstep holds; zero corrections; highest lockstep rate reported.',
    'Out-of-lockstep rates are reported limits. Every replay is compared, including limits.',
    'Lockstep means the intersection of all bracketed client-minus-server tick offsets is nonempty.',
    'Samples use server RCON game time modulo Integer.MAX_VALUE and flushed client tickCount.',
    'A wrap during a replay refuses measurement. ClientLevel gameTime extrapolates locally between server updates.',
    'Only absolute tickCount and gameTime are excluded from comparison. Original recordings are retained.',
    'Creative fixture mode holds resources constant. Survival then creative clears flight; jump stays held.',
    'Inputs run for 600 ticks and coast for 200. Edge inputs sneak; flat and ice inputs sprint-jump.',
    'Rate bounds include the complete RCON query duration. All raw samples are in *.clocks.json.', '')
  foreach ($run in $runs) {
    $c = $run.Clock
    $body += @("## $($run.Course), property $($run.Follow), setting $($run.Rate) TPS", '',
      "Samples $($c.Samples); server ticks $($c.ServerTicks); client ticks [$($c.ClientTicksMin), $($c.ClientTicksMax)].",
      "Client/server ratio [$($c.RatioMin), $($c.RatioMax)]; lockstep $($c.Lockstep).",
      "Client TPS [$($c.ClientTpsMin), $($c.ClientTpsMax)]; server TPS [$($c.ServerTpsMin), $($c.ServerTpsMax)].",
      "Corrections $($run.Corrections); identical $($run.Identical); $($run.Comparison)", '')
  }
  $body += @('## Distributions across courses', '')
  foreach ($rate in @(20, 100, 200)) {
    $entries = @($runs | Where-Object { $_.Follow -eq 'true' -and $_.Rate -eq $rate })
    $body += "Setting $rate ratio lower bounds: $(Distribution @($entries | ForEach-Object { $_.Clock.RatioMin }))"
    $body += "Setting $rate ratio upper bounds: $(Distribution @($entries | ForEach-Object { $_.Clock.RatioMax }))"
    $body += "Setting $rate client TPS lower bounds: $(Distribution @($entries | ForEach-Object { $_.Clock.ClientTpsMin }))"
    $body += "Setting $rate client TPS upper bounds: $(Distribution @($entries | ForEach-Object { $_.Clock.ClientTpsMax }))"
  }
  $body += @('## Failures', '') + @($failures) + @('', '## Driver log', '') + @($notes)
  $body += @('', "Exit code $exitCode")
  $runs | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $result 'runs.json') -Encoding ASCII
  Set-Content -LiteralPath (Join-Path $result 'report.md') -Value $body -Encoding ASCII
}
Write-Output (Join-Path $result 'report.md')
exit $exitCode
