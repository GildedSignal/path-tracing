# 1.8c live gate. See clock-full-demand.md for the frozen protocol.
param([string]$Jar = "$PSScriptRoot\..\..\bot\build\pathtracing.jar")
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
. "$PSScriptRoot\courses.ps1"
. "$PSScriptRoot\fixture-rig.ps1"

# Use the original course, controls, clock math and live capture without running their gates.
foreach ($entry in @(
  @{ File = 'clock-gate.ps1'; Names = @('Measure-Clocks', 'New-ClockInput', 'Distribution', 'Build-ClockCourse') },
  @{ File = 'clock-100x.ps1'; Names = @('Copy-RigPart', 'Set-Rate', 'Direct-Rcon', 'Read-ClientTick', 'Capture-Replay', 'Stop-Session') }
)) {
  $tokens = $null
  $parseErrors = $null
  $gate = [Management.Automation.Language.Parser]::ParseFile(
    (Join-Path $PSScriptRoot $entry.File), [ref]$tokens, [ref]$parseErrors)
  if ($parseErrors.Count) { throw "Cannot parse $($entry.File)." }
  foreach ($name in $entry.Names) {
    $definition = @($gate.EndBlock.Statements | Where-Object {
      $_ -is [Management.Automation.Language.FunctionDefinitionAst] -and $_.Name -ceq $name
    })
    if ($definition.Count -ne 1) { throw "Expected one definition of $name in $($entry.File)." }
    . ([scriptblock]::Create($definition[0].Extent.Text))
  }
}

$referenceRig = $Rig
$referenceEnv = $env:PATHTRACING_ROOT
$scripts = Join-Path $PSScriptRoot '..\..\scripts'
$powershell = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
$watch = [Diagnostics.Stopwatch]::StartNew()
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$prefix = 'clock1_8c_' + $stamp
$ticks = 800
$rates = @(20, 2000, 4000)
$result = Join-Path $PSScriptRoot "results\$stamp-clock-full-demand"
$snapshot = Join-Path $result 'scratch-rig'
$botDir = Join-Path $referenceRig 'clients\bot'
$null = New-Item -ItemType Directory -Path $result
$commands = [Collections.Generic.List[string]]::new()
$notes = [Collections.Generic.List[string]]::new()
$failures = [Collections.Generic.List[string]]::new()
$started = @{}
$lock = $null
$retainScratch = $false
$lockPath = Join-Path $referenceRig 'rig.lock'
$courses = @(Get-FixtureCourses | Where-Object { $_.Name -in @('full-blocks', 'ice', 'edges') })
function New-Run([string]$Course, [int]$Rate, [int]$ExpectedRows) {
  [pscustomobject]@{ Course = $Course; Rate = $Rate; ExpectedRows = $ExpectedRows; Status = 'unqualified'
    Recording = $null; Rows = $null; Checked = $false; Corrections = $null; Identical = $null; Clock = $null; ClockWindows = @()
    ReplaySeconds = $null; FullSeconds = $null; EndToEndTps = $null; Audit = $null; Reason = 'Not attempted' }
}
$runs = @(foreach ($course in $courses) { foreach ($rate in $rates) { New-Run $course.Name $rate $ticks } })
$sustained = New-Run 'full-blocks' 2000 12000

function Start-FullDemandBot([bool]$Audit) {
  Assert-Absent
  if ($BotName -in (Players (Rcon 'list')) -or @(Bot-Processes).Count -gt 0) {
    throw 'Close the existing bot before the clock gate. The gate only stops its own processes.'
  }
  $null = Require (Invoke-Script (Join-Path $scripts 'launch-client.ps1') @{
    Role = 'bot'; FollowTickRate = 'true'; FullTickDemand = 'true'; ClockAudit = $Audit.ToString().ToLowerInvariant()
  } 30 'bot') 'Bot launch'
  if (-not $started.ContainsKey('bot')) { throw 'No owned bot process reported.' }
  $deadline = [Diagnostics.Stopwatch]::StartNew()
  do {
    Assert-Absent
    if ($BotName -in (Players (Rcon 'list'))) { break }
    if ($deadline.Elapsed.TotalSeconds -ge 180) { throw 'Bot did not join.' }
    Start-Sleep -Milliseconds 500
  } while ($true)
  $arguments = Get-Content -LiteralPath (Join-Path $botDir 'launch.args')
  if ('"-Dpathtracing.followTickRate=true"' -cnotin $arguments -or
      '"-Dpathtracing.fullTickDemand=true"' -cnotin $arguments -or
      ('"-Dpathtracing.clockAudit=true"' -cin $arguments) -ne $Audit) { throw 'Launch property mismatch.' }
  $null = Require (Bot 'stop' 'Stopped\.') 'Bot readiness'
  $modeReply = Require (Rcon "data get entity $BotName playerGameType") 'Original bot mode'
  if ($modeReply.Text.Trim() -notmatch 'entity data: ([0-3])$') { throw 'Cannot read original bot mode.' }
  $script:originalMode = @('survival', 'creative', 'adventure', 'spectator')[[int]$Matches[1]]
  $null = Require (Rcon @("gamemode survival $BotName", "gamemode creative $BotName")) 'Clock fixture mode'
}

function New-SustainedInput([string]$Original) {
  $name = $Original + '_sustained'
  $directory = Join-Path $botDir 'pathtracing\inputs'
  $path = Join-Path $directory "$name.jsonl"
  $rows = [IO.File]::ReadAllLines((Join-Path $directory "$Original.jsonl"))
  if ($rows.Length -ne $ticks + 1) { throw 'Expected original start and 800 controls.' }
  $idle = $rows[-1] | ConvertFrom-Json
  foreach ($key in @('forward', 'back', 'left', 'right', 'jump', 'sneak', 'sprint')) {
    if ($idle.$key -isnot [bool] -or $idle.$key) { throw "Last control is not idle ($key)." }
  }
  $writer = [IO.StreamWriter]::new($path, $false, [Text.UTF8Encoding]::new($false))
  try {
    foreach ($line in $rows) { $writer.WriteLine($line) }
    for ($i = $ticks; $i -lt $sustained.ExpectedRows; $i++) {
      $idle.t = $i
      $writer.WriteLine(($idle | ConvertTo-Json -Compress))
    }
  } finally { $writer.Dispose() }
  Save-Artifact $path
  $name
}

function Read-Audit([string]$Log, [string]$Recording) {
  $name = [regex]::Escape([IO.Path]::GetFileNameWithoutExtension($Recording))
  $pattern = "Clock audit $name rows=(\d+) complete=(true|false) activeNanos=(\d+) frames=(\d+) scheduled=(\d+) executed=(\d+) dropped=(-?\d+) startMillis=(\d+)$"
  $entry = @(Select-String -LiteralPath $Log -Pattern $pattern -CaseSensitive)
  if ($entry.Count -eq 0) { return }
  if ($entry.Count -ne 1) { throw 'Multiple scheduler audit lines for this recording.' }
  $groups = $entry[0].Matches[0].Groups
  $activeSeconds = [long]$groups[3].Value / 1e9
  [pscustomobject]@{ Rows = [int]$groups[1].Value; Complete = $groups[2].Value -ceq 'true'
    ActiveSeconds = $activeSeconds; Frames = [long]$groups[4].Value
    Scheduled = [long]$groups[5].Value; Executed = [long]$groups[6].Value; Dropped = [long]$groups[7].Value
    StartMillis = [long]$groups[8].Value
    ActiveTps = if ($activeSeconds -gt 0) { [int]$groups[1].Value / $activeSeconds } }
}

function Wait-Audit($Run) {
  $timer = [Diagnostics.Stopwatch]::StartNew()
  $log = Join-Path $botDir 'logs\latest.log'
  do {
    $Run.Audit = Read-Audit $log $Run.Recording
    if ($null -ne $Run.Audit) { return }
    [Threading.Thread]::Sleep(20)
  } while ($timer.Elapsed.TotalSeconds -lt 30)
  throw 'Scheduler audit did not finish the final frame within 30 seconds of publication.'
}

function Check-FullReplay($Run, $Reference, [bool]$IsSustained) {
  $path = Join-Path $result $Run.Recording
  $raw = if (Test-Path -LiteralPath $path) { $path } else { "$path.tmp" }
  $rows = @()
  if (Test-Path -LiteralPath $raw) {
    $lines = [IO.File]::ReadAllLines($raw)
    $rows = @($lines | Where-Object { $_ -cmatch '^\{"t":' })
    $Run.Rows = $rows.Count
    $Run.Corrections = 0
    foreach ($line in $rows | Where-Object { $_ -cmatch '"anomaly":' }) {
      $Run.Corrections += @(($line | ConvertFrom-Json).anomaly).Count
    }
    $check = Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $raw } 60
    $Run.Checked = $check.Code -eq 0 -and $Run.Rows -eq $Run.ExpectedRows -and $raw -ceq $path
    if (-not $Run.Checked) { "Expected $($Run.ExpectedRows) complete checked rows. See $($check.File)." }
    if ($Run.Corrections) { "$($Run.Corrections) correction/impulse packets." }
    $completeTrailer = '{"type":"end","ticks":' + $Run.ExpectedRows + ',"complete":true}'
    if ($raw -ceq $path -and $Run.Rows -eq $Run.ExpectedRows -and $lines[-1] -ceq $completeTrailer -and
        $Run.ReplaySeconds -gt 0) { $Run.EndToEndTps = $Run.Rows / $Run.ReplaySeconds }
  } else { 'No recording.' }

  try {
    $samplePath = [IO.Path]::ChangeExtension($path, '.clocks.json')
    $samples = @(Get-Content -LiteralPath $samplePath -Raw | ConvertFrom-Json | ForEach-Object { $_ })
    $Run.Clock = Measure-Clocks $samples
    if ($IsSustained) {
      # Reuse the same bounds on successive windows, each spanning at least 1000 client ticks.
      $first = 0
      $Run.ClockWindows = @(for ($last = 2; $last -lt $samples.Count; $last++) {
        if ($samples[$last].ClientBefore - $samples[$first].ClientAfter -lt 1000 -or $last - $first -lt 2) { continue }
        Measure-Clocks @($samples[$first..$last])
        $first = $last + 1
        if ($first -ge $samples.Count) { break }
      })
    }
  } catch { "Clock measurement refused. $($_.Exception.Message)" }

  if ($Run.Rate -ne 20) {
    if ($Reference.Status -ne 'qualified') { '20 TPS reference is unqualified.' }
    if ($Reference.Recording -and $rows.Count) {
      $baseline = Join-Path $result $Reference.Recording
      if (Test-Path -LiteralPath $baseline) {
        $comparisonPath = $raw
        if ($IsSustained -and $rows.Count -ge $ticks) {
          $comparisonPath = "$path.first-800.jsonl"
          $prefixLines = @($lines[0]) + @($rows | Select-Object -First $ticks) +
            ('{"type":"end","ticks":' + $ticks + ',"complete":true}')
          [IO.File]::WriteAllLines($comparisonPath, [string[]]$prefixLines, [Text.UTF8Encoding]::new($false))
          $rows = @($rows | Select-Object -First $ticks)
        }
        $comparison = Invoke-Script (Join-Path $PSScriptRoot 'compare-recordings.ps1') @{ A = $baseline; B = $comparisonPath }
        $Run.Identical = $comparison.Code -eq 0
        if (-not $Run.Identical) { "$(Diagnostic $comparison) See $($comparison.File)." }
        if ($Run.Identical) {
          $pattern = '^(\{"t":\d+,)"tickCount":\d+,"gameTime":\d+,'
          $expected = @([IO.File]::ReadLines($baseline) | Where-Object { $_ -cmatch '^\{"t":' })
          for ($i = 0; $i -lt $rows.Count; $i++) {
            if ($expected[$i] -cnotmatch $pattern -or $rows[$i] -cnotmatch $pattern -or
                ($expected[$i] -creplace $pattern, '$1') -cne ($rows[$i] -creplace $pattern, '$1')) {
              $Run.Identical = $false
              "Raw row mismatch at tick $i after removing absolute counters."
              break
            }
          }
        }
      }
    }
    if ($Run.Identical -ne $true) { 'No identical comparison.' }
  }

  if ($IsSustained) {
    $audit = $Run.Audit
    if ($null -eq $audit -or -not $audit.Complete -or $audit.Rows -ne $Run.ExpectedRows -or
        $audit.Executed -lt $audit.Rows -or $audit.Frames -le 0 -or $audit.ActiveSeconds -le 0 -or
        $audit.Dropped -ne ($audit.Scheduled - $audit.Executed) -or $audit.Dropped -lt 0) {
      'Missing or inconsistent completed scheduler audit.'
    } elseif ($audit.Dropped -gt 0) { "Dropped $($audit.Dropped) ticks of timer demand." }
  }
}

function Client-Profile([string[]]$Arguments, [string]$Log, [string]$Expected) {
  $reply = & "$Jdk\jcmd.exe" $started.bot.Id @Arguments 2>&1
  $status = $LASTEXITCODE
  $reply | Set-Content -LiteralPath (Join-Path $result $Log) -Encoding UTF8
  if ($status -ne 0 -or ($reply -join "`n") -cnotmatch $Expected) { throw "Client JFR command failed. See $Log." }
}

function Run-Cell($Run, $Course, [string]$InputName, $Reference, [bool]$BuildCourse, [bool]$IsSustained) {
  $session = [Diagnostics.Stopwatch]::StartNew()
  $errors = [Collections.Generic.List[string]]::new()
  $profiling = $false
  try {
    Set-Rate 20
    [IO.File]::WriteAllLines($optionsPath, $profileOptions, [Text.UTF8Encoding]::new($false))
    Start-FullDemandBot $IsSustained
    if ($BuildCourse) {
      Build-ClockCourse $Course
      $script:courseReady = $true
    }
    if ($IsSustained) {
      try {
        Client-Profile @('JFR.start', 'name=clock', 'settings=profile') 'jfr-start.log' '(?m)^Started recording '
        $profiling = $true
      } catch { $errors.Add($_.Exception.Message) }
    }
    Capture-Replay $Run $InputName
    if ($IsSustained) { Wait-Audit $Run }
    Assert-Absent
  } catch { $errors.Add($_.Exception.Message) }
  finally {
    if ($profiling) {
      try { Client-Profile @('JFR.stop', 'name=clock', "filename=$result\sustained-client.jfr") 'jfr-stop.log' '(?m)^Stopped recording ' }
      catch { $errors.Add($_.Exception.Message) }
      if (-not (Test-Path -LiteralPath "$result\sustained-client.jfr")) { $errors.Add('Missing sustained client JFR.') }
    }
    try {
      try { Set-Rate 20 } finally { Stop-Session }
    } catch {
      $errors.Add($_.Exception.Message)
      $failures.Add("Cleanup failed for $($Run.Course) at $($Run.Rate) TPS. $($_.Exception.Message)")
    }
    try {
      $log = Join-Path $botDir 'logs\latest.log'
      if (Test-Path -LiteralPath $log) {
        $label = if ($IsSustained) { 'sustained' } else { "$($Run.Course)-$($Run.Rate)" }
        Copy-Item -LiteralPath $log -Destination "$result\$label.client.log"
        if ($IsSustained -and $Run.Recording -and $null -eq $Run.Audit) { $Run.Audit = Read-Audit $log $Run.Recording }
      }
      if ($Run.Recording) {
        $source = Join-Path $botDir "pathtracing\recordings\$($Run.Recording)"
        Save-Artifact $source
        Save-Artifact "$source.tmp"
        foreach ($problem in Check-FullReplay $Run $Reference $IsSustained) { $errors.Add($problem) }
      }
    } catch {
      $script:retainScratch = $true
      $errors.Add($_.Exception.Message)
    }
    $Run.FullSeconds = $session.Elapsed.TotalSeconds
    $Run.Reason = $errors -join ' '
    if (-not $errors.Count) { $Run.Status = 'qualified' }
    Note "$($Run.Course) $($Run.Rate) TPS ($($Run.ExpectedRows) rows) $($Run.Status) $($Run.Reason)"
  }
  if ($failures.Count) { throw 'Stopping after a cleanup failure.' }
}

function Rate-Status([double]$Min, [double]$Max, [int]$Target) {
  if ($Max -lt $Target) { 'below target' }
  elseif ($Min -ge $Target) { 'reaches target' }
  else { 'includes target' }
}

function Describe-Run($Run) {
  "$($Run.Course) $($Run.Status). $($Run.Reason)"
  "Rows $($Run.Rows); checked $($Run.Checked); corrections/impulses $($Run.Corrections); identical $($Run.Identical)."
  "Replay seconds $($Run.ReplaySeconds); session seconds $($Run.FullSeconds); replay ticks per end-to-end second $($Run.EndToEndTps)."
  if ($null -ne $Run.Clock) {
    $c = $Run.Clock
    "Client TPS [$($c.ClientTpsMin), $($c.ClientTpsMax)] ($(Rate-Status $c.ClientTpsMin $c.ClientTpsMax $Run.Rate))."
    "Server TPS [$($c.ServerTpsMin), $($c.ServerTpsMax)] ($(Rate-Status $c.ServerTpsMin $c.ServerTpsMax $Run.Rate))."
    "Old offset intersection [$($c.OffsetMin), $($c.OffsetMax)]; lockstep $($c.Lockstep); samples $($c.Samples)."
  }
  ''
}

function All-Qualified([int]$Rate) {
  @($runs | Where-Object { $_.Rate -eq $Rate -and $_.Status -eq 'qualified' }).Count -eq 3
}

function Write-FullReport {
  $report = @('# 1.8c clock comparison', '', "Reference rig $referenceRig on port $ServerPort",
    'Fixture profile four-chunk radius, 480 x 270 client', "Worktree HEAD $head", "Bot jar SHA-256 $jarHash",
    "Full run seconds $($watch.Elapsed.TotalSeconds)", '',
    "2000 movement and observation checks: $(All-Qualified 2000)",
    "4000 stress movement and observation checks: $(All-Qualified 4000)",
    "Measured 100x active client result: $measured100x", "Sustained end-to-end reaches 2000 TPS: $endToEnd100x", '')
  foreach ($rate in $rates) {
    $group = @($runs | Where-Object Rate -eq $rate)
    $report += @("## $rate TPS", '')
    foreach ($run in $group) { $report += @(Describe-Run $run) }
    foreach ($metric in @('ClientTpsMin', 'ClientTpsMax', 'ServerTpsMin', 'ServerTpsMax')) {
      $report += "$metric $(Distribution @($group | Where-Object Clock | ForEach-Object { $_.Clock.$metric }))"
    }
    foreach ($metric in @('ReplaySeconds', 'FullSeconds', 'EndToEndTps')) {
      $report += "$metric $(Distribution @($group | Where-Object { $null -ne $_.$metric } | ForEach-Object { $_.$metric }))"
    }
    $report += ''
  }
  $report += @('## 12,000-tick sustained sample at configured 2000 TPS', '') + @(Describe-Run $sustained)
  foreach ($metric in @('ClientTpsMin', 'ClientTpsMax', 'ServerTpsMin', 'ServerTpsMax')) {
    $report += "Sustained window $metric $(Distribution @($sustained.ClockWindows | ForEach-Object { $_.$metric }))"
  }
  if ($null -ne $sustained.Audit) {
    $a = $sustained.Audit
    $report += @("Active seconds $($a.ActiveSeconds); replay ticks per active second $($a.ActiveTps) ($($a.ActiveTps / 20)x).",
      "Replay ticks per end-to-end second $($sustained.EndToEndTps) ($($sustained.EndToEndTps / 20)x).",
      "Whole intersecting frames $($a.Frames); scheduled $($a.Scheduled); executed $($a.Executed); dropped $($a.Dropped).",
      "JFR replay start Unix milliseconds $($a.StartMillis).", '')
  }
  $report += @('## Driver failures', '') + @($failures.ToArray())
  $data = @{ Matrix = $runs; Sustained = $sustained; Measured100x = $measured100x; EndToEnd100x = $endToEnd100x
    Failures = $failures.ToArray(); Head = $head; JarHash = $jarHash }
  ConvertTo-Json -InputObject $data -Depth 10 | Set-Content -LiteralPath "$result\runs.json" -Encoding ASCII
  Set-Content -LiteralPath "$result\report.md" -Value $report -Encoding UTF8
  Write-Output "$result\report.md"
}
try {
  if ($referenceEnv -ine 'D:\path-tracing-lab' -or $ServerPort -ne 25566) {
    throw 'Set PATHTRACING_ROOT=D:\path-tracing-lab and PATHTRACING_SERVER_PORT=25566.'
  }
  # Exclusive creation also refuses existing locks. Keep the handle until the owned rig exits.
  $lock = [IO.File]::Open($lockPath, 'CreateNew', 'Write', 'Read')
  $owner = @{ holder = 'clock-full-demand'; pid = $PID; since = (Get-Date).ToString('s') } | ConvertTo-Json
  $bytes = [Text.Encoding]::ASCII.GetBytes($owner)
  $lock.Write($bytes, 0, $bytes.Length)
  $lock.Flush()
  if ((Test-Port $ServerHost $ServerPort) -or (Test-Port $ServerHost $RconPort) -or @(Bot-Processes).Count) {
    throw 'Stop the reference server and bot before starting the comparison.'
  }
  $jarPath = (Resolve-Path -LiteralPath $Jar).Path
  $jarHash = (Get-FileHash -LiteralPath $jarPath).Hash
  Copy-RigPart 'server' @('.fabric', 'config', 'libraries', 'mods', 'versions', 'world', 'banned-ips.json',
    'banned-players.json', 'eula.txt', 'fabric-server-launch.jar', 'ops.json', 'server.properties', 'usercache.json', 'whitelist.json')
  Copy-RigPart 'clients\bot' @('.fabric', 'config', 'mods', 'options.txt', 'resourcepacks', 'servers.dat')
  $properties = Get-Content -LiteralPath "$snapshot\server\server.properties"
  foreach ($setting in @('server-port=25566', 'rcon.port=25576', 'server-ip=127.0.0.1', 'enable-rcon=true', 'level-name=world')) {
    $key = [regex]::Escape($setting.Split('=')[0])
    $actual = @($properties | Where-Object { $_ -cmatch "^$key=" })
    if ($actual.Count -ne 1 -or $actual[0] -cne $setting) { throw "Expected $setting in reference server properties." }
  }
  $Rig = $snapshot
  $env:PATHTRACING_ROOT = $snapshot
  $botDir = Join-Path $Rig 'clients\bot'
  Copy-Item -LiteralPath $jarPath -Destination (Join-Path $botDir 'mods\pathtracing.jar') -Force
  $optionsPath = Join-Path $botDir 'options.txt'
  $options = Get-Content -LiteralPath $optionsPath
  foreach ($setting in @('toggleSprint:false', 'toggleCrouch:false', 'autoJump:false', 'sprintWindow:0')) {
    $key = $setting.Split(':')[0]
    $actual = @($options | Where-Object { $_ -cmatch "^$key`:" })
    if ($actual.Count -ne 1 -or $actual[0] -cne $setting) { throw "Expected exactly $setting in bot options." }
  }
  [string[]]$profileOptions = @($options | Where-Object { $_ -cnotmatch '^(maxFps|enableVsync):' }) +
    @('maxFps:260', 'enableVsync:false')

  $serverProperties = Join-Path $Rig 'server\server.properties'
  $settings = [IO.File]::ReadAllText($serverProperties)
  $settings = $settings -creplace '(?m)^(view-distance|simulation-distance)=\d+', '$1=4'
  [IO.File]::WriteAllText($serverProperties, $settings, [Text.UTF8Encoding]::new($false))
  $profileOptions = @($profileOptions | Where-Object {
    $_ -cnotmatch '^(renderDistance|simulationDistance|overrideWidth|overrideHeight):'
  }) + @('renderDistance:4', 'simulationDistance:12', 'overrideWidth:480', 'overrideHeight:270')
  $null = Require (Invoke-Script (Join-Path $scripts 'start-server.ps1') @{} 30 'server') 'Reference server launch'
  if (-not $started.ContainsKey('server')) { throw 'No owned server process reported.' }
  $null = Wait-Rcon

  if ($courses.Count -ne 3) { throw 'Missing clock courses.' }
  foreach ($course in $courses) {
    $inputName = New-ClockInput $course
    $reference = $null
    $script:courseReady = $false
    foreach ($run in $runs | Where-Object Course -eq $course.Name) {
      if ($run.Rate -ne 20 -and -not $script:courseReady) {
        $run.Reason = 'Course readiness was not established.'
        continue
      }
      Run-Cell $run $course $inputName $reference ($run.Rate -eq 20) $false
      if ($run.Rate -eq 20) { $reference = $run }
    }
    if ($course.Name -eq 'full-blocks') {
      $sustainedCourse = $course
      $sustainedReference = $reference
      if ($script:courseReady) { $sustainedInput = New-SustainedInput $inputName }
    }
  }
  if ($sustainedInput) {
    Run-Cell $sustained $sustainedCourse $sustainedInput $sustainedReference $false $true
  } else { $sustained.Reason = 'Full-blocks course readiness was not established.' }
} catch { $failures.Add($_.Exception.GetBaseException().Message) }
finally {
  if ($started.ContainsKey('server')) {
    try { Set-Rate 20 } catch { $failures.Add($_.Exception.Message) }
    try { Stop-Session } catch { $failures.Add($_.Exception.Message) }
    try {
      Stop-Owned 'server'
      $started.Remove('server')
      if (Test-Path -LiteralPath "$Rig\server\logs\latest.log") {
        Copy-Item -LiteralPath "$Rig\server\logs\latest.log" -Destination "$result\server.log"
      }
    } catch { $failures.Add($_.Exception.Message) }
  }
  $Rig = $referenceRig
  $env:PATHTRACING_ROOT = $referenceEnv
  if ($null -ne $lock) {
    $lock.Dispose()
    try { Remove-Item -LiteralPath $lockPath } catch { $failures.Add($_.Exception.Message) }
  }
  if ((Test-Path -LiteralPath $snapshot) -and -not $started.Count -and -not $failures.Count -and -not $retainScratch) {
    $resolved = (Resolve-Path -LiteralPath $snapshot).Path
    if ($resolved -ceq (Join-Path (Resolve-Path -LiteralPath $result).Path 'scratch-rig')) {
      try { Remove-Item -LiteralPath $resolved -Recurse -Force } catch { $failures.Add($_.Exception.Message) }
    }
  }
  $head = git -C $Root rev-parse HEAD
  $measured100x = -not $failures.Count -and (All-Qualified 2000) -and $sustained.Status -eq 'qualified' -and
    $sustained.Audit.ActiveTps -ge 2000
  $endToEnd100x = $null -ne $sustained.EndToEndTps -and $sustained.EndToEndTps -ge 2000
  Write-FullReport
}
if (-not $measured100x) { exit 1 }
