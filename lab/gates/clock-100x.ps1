# Separate live comparison. See clock-100x.md for the frozen protocol.
param(
  [string]$Jar = "$PSScriptRoot\..\..\bot\build\pathtracing.jar",
  [string]$Review,
  [switch]$Focused
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"
. "$PSScriptRoot\courses.ps1"
. "$PSScriptRoot\fixture-rig.ps1"

# Import only the frozen helper definitions, without executing the old gate.
$tokens = $null
$parseErrors = $null
$gate = [Management.Automation.Language.Parser]::ParseFile(
  (Join-Path $PSScriptRoot 'clock-gate.ps1'), [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw 'Cannot parse the 1.8 gate.' }
$helpers = @('Measure-Clocks', 'New-ClockInput', 'Distribution', 'Build-ClockCourse', 'Start-ClockBot')
foreach ($name in $helpers) {
  $definition = @($gate.EndBlock.Statements | Where-Object {
    $_ -is [Management.Automation.Language.FunctionDefinitionAst] -and $_.Name -ceq $name
  })
  if ($definition.Count -ne 1) { throw "Expected one 1.8 definition of $name." }
  . ([scriptblock]::Create($definition[0].Extent.Text))
}

$referenceRig = $Rig
$referenceEnv = $env:PATHTRACING_ROOT
$scripts = Join-Path $PSScriptRoot '..\..\scripts'
$powershell = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
$watch = [Diagnostics.Stopwatch]::StartNew()
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$prefix = 'clock100x_' + $stamp
$ticks = 800
$rates = @(20, 2000, 4000)
$result = Join-Path $PSScriptRoot "results\$stamp-clock-100x"
$evidence = if ($Review) { (Resolve-Path -LiteralPath $Review).Path } else { $result }
$snapshot = Join-Path $result 'scratch-rig'
$botDir = Join-Path $referenceRig 'clients\bot'
$null = New-Item -ItemType Directory -Path $result
$commands = [Collections.Generic.List[string]]::new()
$notes = [Collections.Generic.List[string]]::new()
$failures = [Collections.Generic.List[string]]::new()
$started = @{}
$lock = $null
$lockPath = Join-Path $referenceRig 'rig.lock'
$runs = @(foreach ($course in Get-FixtureCourses | Where-Object { $_.Name -in @('full-blocks', 'ice', 'edges') }) {
  foreach ($rate in $rates) {
    [pscustomobject]@{ Course = $course.Name; Rate = $rate; Status = 'unqualified'; Recording = $null
      Rows = $null; Checked = $false; Corrections = $null; Identical = $null; Clock = $null
      ReplaySeconds = $null; FullSeconds = $null; Reason = 'Not attempted' }
  }
})

function Copy-RigPart([string]$Part, [string[]]$Names) {
  $source = Join-Path $referenceRig $Part
  $destination = Join-Path $snapshot $Part
  $null = New-Item -ItemType Directory -Path $destination -Force
  foreach ($item in Get-ChildItem -LiteralPath $source -Force | Where-Object { $_.Name -in $Names }) {
    Copy-Item -LiteralPath $item.FullName -Destination $destination -Recurse -Force
  }
}

function Set-Rate([int]$Rate) {
  $reply = Require (Rcon "tick rate $Rate") "Set tick rate $Rate"
  if ($reply.Text.Trim() -cnotmatch ('^Set the target tick rate to ' + $Rate + '(?:\.0)? per second$')) {
    throw "Tick rate $Rate refused. See $($reply.File)."
  }
}

function Direct-Rcon([string]$Command) {
  & "$scripts\rcon.ps1" -Commands @($Command) -SocketTimeoutMilliseconds 2000
}

function Read-ClientTick([IO.StreamReader]$Reader, $Progress) {
  if ($null -eq $Reader) { return }
  $Progress.Pending += $Reader.ReadToEnd()
  $lines = $Progress.Pending.Split("`n")
  for ($i = $lines.Length - 2; $i -ge 0; $i--) {
    if ($lines[$i] -cmatch '^\{"t":\d+,"tickCount":(\d+),') {
      $Progress.Tick = [long]$Matches[1]
      break
    }
  }
  $Progress.Pending = $lines[-1]
  $Progress.Tick
}

function Capture-Replay($Run, [string]$InputName) {
  Set-Rate $Run.Rate
  Start-Sleep -Seconds 1
  Assert-Absent
  $name = "${InputName}_$($Run.Rate)"
  $path = Join-Path $botDir "pathtracing\recordings\$name.jsonl"
  $Run.Recording = [IO.Path]::GetFileName($path)
  $samples = [Collections.Generic.List[object]]::new()
  $discarded = [Collections.Generic.List[object]]::new()
  $reader = $null
  $progress = @{ Pending = ''; Tick = $null }
  $timer = [Diagnostics.Stopwatch]::StartNew()
  try {
    $reply = Direct-Rcon "msg $BotName replay $InputName $name"
    if (($reply -join ' ') -notlike "*replay $InputName $name*") { throw "Replay dispatch refused: $reply" }
    while (-not (Test-Path -LiteralPath $path)) {
      if ($timer.Elapsed.TotalSeconds -gt 60) { throw 'Replay exceeded 60 seconds.' }
      if ($null -eq $reader -and (Test-Path -LiteralPath "$path.tmp")) {
        try {
          $stream = [IO.File]::Open("$path.tmp", 'Open', 'Read', 'ReadWrite, Delete')
          $reader = [IO.StreamReader]::new($stream)
        } catch [IO.FileNotFoundException] {
          # The recorder may publish the file between the existence check and open.
          if (-not (Test-Path -LiteralPath $path)) { throw }
        }
      }
      $beforeSeconds = $watch.Elapsed.TotalSeconds
      $before = Read-ClientTick $reader $progress
      if ($null -ne $before) {
        $reply = Direct-Rcon 'time query gametime'
        $after = Read-ClientTick $reader $progress
        $afterSeconds = $watch.Elapsed.TotalSeconds
        $sample = [pscustomobject]@{ Server = $null; ClientBefore = $before; ClientAfter = $after
          BeforeSeconds = $beforeSeconds; AfterSeconds = $afterSeconds }
        $reason = $null
        if ($reply -cnotmatch '^The game time is (\d+) tick\(s\)$') {
          $discarded.Add([pscustomobject]@{ Reason = 'Invalid game time'; Sample = $sample; Reply = $reply })
          throw 'Unrecognized game-time reply.'
        }
        $sample.Server = [long]$Matches[1]
        if (Test-Path -LiteralPath $path) { $reason = 'Published during query' }
        elseif ($samples.Count -and $samples[-1].Server -eq $sample.Server) { $reason = 'Repeated server tick' }
        if ($reason) { $discarded.Add([pscustomobject]@{ Reason = $reason; Sample = $sample }) }
        else { $samples.Add($sample) }
      }
      if (-not (Test-Path -LiteralPath $path)) { [Threading.Thread]::Sleep(5) }
    }
    $Run.ReplaySeconds = $timer.Elapsed.TotalSeconds
  } finally {
    if ($null -ne $reader) { $reader.Dispose() }
    Save-Artifact $path
    Save-Artifact "$path.tmp"
    ConvertTo-Json -InputObject $samples.ToArray() -Depth 5 | Set-Content "$result\$name.clocks.json" -Encoding ASCII
    ConvertTo-Json -InputObject $discarded.ToArray() -Depth 6 | Set-Content "$result\$name.discarded.json" -Encoding ASCII
  }
}

function Check-Replay($Run, $Reference) {
  $path = Join-Path $evidence $Run.Recording
  $raw = if (Test-Path -LiteralPath $path) { $path } else { "$path.tmp" }
  $problems = [Collections.Generic.List[string]]::new()
  if (Test-Path -LiteralPath $raw) {
    $rows = @([IO.File]::ReadLines($raw) | Where-Object { $_ -cmatch '^\{"t":' })
    $Run.Rows = $rows.Count
    $Run.Corrections = @($rows | ForEach-Object { ($_ | ConvertFrom-Json).anomaly } | Where-Object { $null -ne $_ }).Count
    $check = Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $raw }
    $Run.Checked = $check.Code -eq 0 -and $Run.Rows -eq $ticks -and $raw -eq $path
    if (-not $Run.Checked) { $problems.Add("Expected 800 complete checked rows. See $($check.File).") }
    if ($Run.Corrections) { $problems.Add("$($Run.Corrections) anomaly entries.") }
  } else { $problems.Add('No recording.') }
  $samplePath = [IO.Path]::ChangeExtension($path, '.clocks.json')
  try {
    $clockSamples = Get-Content -LiteralPath $samplePath -Raw | ConvertFrom-Json
    $Run.Clock = Measure-Clocks @($clockSamples)
  }
  catch { $problems.Add($_.Exception.Message) }
  if ($null -ne $Run.Clock -and -not $Run.Clock.Lockstep) { $problems.Add('Empty offset intersection.') }
  if ($Run.Rate -ne 20) {
    if ($Reference.Status -ne 'qualified') { $problems.Add('20 TPS reference is unqualified.') }
    if ($Reference.Recording -and (Test-Path -LiteralPath $raw)) {
      $baseline = Join-Path $evidence $Reference.Recording
      if (Test-Path -LiteralPath $baseline) {
        $comparison = Invoke-Script (Join-Path $PSScriptRoot 'compare-recordings.ps1') @{ A = $baseline; B = $raw }
        $Run.Identical = $comparison.Code -eq 0
        if (-not $Run.Identical) { $problems.Add("$(Diagnostic $comparison) See $($comparison.File).") }
        if ($Run.Identical) {
          # Recorder writes these two counters immediately after t. Compare every other row byte.
          $pattern = '^(\{"t":\d+,)"tickCount":\d+,"gameTime":\d+,'
          $expected = @([IO.File]::ReadLines($baseline) | Where-Object { $_ -cmatch '^\{"t":' })
          for ($i = 0; $i -lt $rows.Count; $i++) {
            if ($expected[$i] -cnotmatch $pattern -or $rows[$i] -cnotmatch $pattern -or
                ($expected[$i] -creplace $pattern, '$1') -cne ($rows[$i] -creplace $pattern, '$1')) {
              $Run.Identical = $false
              $problems.Add("Raw row mismatch at tick $i after removing absolute counters.")
              break
            }
          }
        }
      }
    }
    if ($Run.Identical -ne $true) { $problems.Add('No identical comparison.') }
  }
  $problems
}

function Stop-Session {
  if (-not $started.ContainsKey('bot')) { return }
  try {
    if ($null -ne $script:originalMode) {
      $null = Require (Rcon "gamemode $script:originalMode $BotName") 'Restore bot mode'
      $script:originalMode = $null
    }
  } finally {
    Stop-Owned 'bot'
    $started.Remove('bot')
  }
  $deadline = [Diagnostics.Stopwatch]::StartNew()
  while ($BotName -in (Players (Rcon 'list'))) {
    if ($deadline.Elapsed.TotalSeconds -ge 30) { throw 'Owned bot did not leave the server.' }
    Start-Sleep -Milliseconds 50
  }
}

function Write-Report([string[]]$Source) {
  $report = @('# 1.8b clock comparison', '') + $Source + @('')
  foreach ($rate in $rates) {
    $group = @($runs | Where-Object Rate -eq $rate)
    $count = @($group | Where-Object Status -eq 'qualified').Count
    $report += "## $rate TPS ($count/$($group.Count) qualified)"
    foreach ($run in $group) {
      $c = $run.Clock
      $report += @('', "$($run.Course) $($run.Status). $($run.Reason)",
        "Rows $($run.Rows); checked $($run.Checked); corrections/impulses $($run.Corrections); identical $($run.Identical).",
        "Replay seconds $($run.ReplaySeconds); session seconds $($run.FullSeconds).")
      if ($null -ne $c) {
        $report += "Client TPS [$($c.ClientTpsMin), $($c.ClientTpsMax)]; server TPS [$($c.ServerTpsMin), $($c.ServerTpsMax)]; offsets [$($c.OffsetMin), $($c.OffsetMax)]; samples $($c.Samples)."
      }
    }
    $report += ''
    foreach ($metric in @('ClientTpsMin', 'ClientTpsMax', 'ServerTpsMin', 'ServerTpsMax')) {
      $report += "$metric $(Distribution @($group | Where-Object Clock | ForEach-Object { $_.Clock.$metric }))"
    }
    $report += "ReplaySeconds $(Distribution @($group | Where-Object ReplaySeconds | ForEach-Object ReplaySeconds))"
    $report += ''
  }
  $report += @('## Driver failures', '') + @($failures.ToArray())
  ConvertTo-Json -InputObject $runs -Depth 10 | Set-Content -LiteralPath "$result\runs.json" -Encoding ASCII
  Set-Content -LiteralPath "$result\report.md" -Value $report -Encoding UTF8
  Write-Output "$result\report.md"
}

if ($Review) {
  # Review operates on completed evidence only and leaves the original report intact.
  $source = Get-Content -LiteralPath "$evidence\report.md"
  $runs = Get-Content -LiteralPath "$evidence\runs.json" -Raw | ConvertFrom-Json
  $driverFailureIndex = [Array]::IndexOf($source, '## Driver failures')
  if ($driverFailureIndex -lt 0 -or $runs.Count -ne 9) { throw 'Expected a completed 1.8b evidence directory.' }
  foreach ($line in $source | Select-Object -Skip ($driverFailureIndex + 1) | Where-Object { $_.Trim() }) {
    $failures.Add($line)
  }
  foreach ($run in $runs) {
    if (-not $run.Recording -or $null -eq $run.ReplaySeconds) { continue }
    $reference = $runs | Where-Object { $_.Course -eq $run.Course -and $_.Rate -eq 20 }
    # The initial driver boxed parsed arrays on PowerShell 5.1. Recompute the verdict from raw
    # samples and discard only that parser error and the consequent unqualified-reference error.
    $retained = $run.Reason.Replace('Fewer than three clock samples inside the replay.', '').
      Replace('20 TPS reference is unqualified.', '').Trim()
    $problems = @(Check-Replay $run $reference)
    $run.Reason = (@($retained) + $problems | Where-Object { $_ }) -join ' '
    $run.Status = if ($run.Reason) { 'unqualified' } else { 'qualified' }
  }
  Write-Report (@("Rechecked raw evidence $evidence") + @($source | Where-Object {
    $_ -match '^(Reference rig|Fixture profile|Bot jar SHA-256|Full run seconds) '
  }))
  if ($failures.Count -or @($runs | Where-Object { $_.Rate -ne 4000 -and $_.Status -ne 'qualified' }).Count) { exit 1 }
  exit 0
}

try {
  if ($referenceEnv -ine 'D:\path-tracing-lab' -or $ServerPort -ne 25566) {
    throw 'Set PATHTRACING_ROOT=D:\path-tracing-lab and PATHTRACING_SERVER_PORT=25566.'
  }
  # Exclusive creation also refuses existing locks. Keep the handle until the owned rig exits.
  $lock = [IO.File]::Open($lockPath, 'CreateNew', 'Write', 'Read')
  $owner = @{ holder = 'clock-100x'; pid = $PID; since = (Get-Date).ToString('s') } | ConvertTo-Json
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
  $profileName = 'standard'
  if ($Focused) {
    $profileName = 'four-chunk radius, 480 x 270 client'
    $serverProperties = Join-Path $Rig 'server\server.properties'
    $settings = [IO.File]::ReadAllText($serverProperties)
    $settings = $settings -creplace '(?m)^(view-distance|simulation-distance)=\d+', '$1=4'
    [IO.File]::WriteAllText($serverProperties, $settings, [Text.UTF8Encoding]::new($false))
    $profileOptions = @($profileOptions | Where-Object {
      $_ -cnotmatch '^(renderDistance|simulationDistance|overrideWidth|overrideHeight):'
    }) + @('renderDistance:4', 'simulationDistance:12', 'overrideWidth:480', 'overrideHeight:270')
  }
  $null = Require (Invoke-Script (Join-Path $scripts 'start-server.ps1') @{} 30 'server') 'Reference server launch'
  if (-not $started.ContainsKey('server')) { throw 'No owned server process reported.' }
  $null = Wait-Rcon
  foreach ($course in Get-FixtureCourses | Where-Object { $_.Name -in $runs.Course }) {
    $inputName = New-ClockInput $course
    $reference = $null
    $courseReady = $false
    foreach ($run in $runs | Where-Object Course -eq $course.Name) {
      $session = [Diagnostics.Stopwatch]::StartNew()
      $errors = [Collections.Generic.List[string]]::new()
      $run.Reason = ''
      try {
        if ($run.Rate -ne 20 -and -not $courseReady) { throw 'Course readiness was not established.' }
        Set-Rate 20
        [IO.File]::WriteAllLines($optionsPath, $profileOptions, [Text.UTF8Encoding]::new($false))
        Start-ClockBot 'true'
        if ($run.Rate -eq 20) {
          Build-ClockCourse $course
          $courseReady = $true
        }
        Capture-Replay $run $inputName
        Assert-Absent
      } catch { $errors.Add($_.Exception.Message) }
      finally {
        try {
          try { Set-Rate 20 } finally { Stop-Session }
        } catch {
          $errors.Add($_.Exception.Message)
          $failures.Add("Cleanup failed for $($run.Course) at $($run.Rate): $($_.Exception.Message)")
        }
        if ($run.Recording) {
          try {
            # A failed capture can finish or flush a partial file before the owned bot exits.
            $source = Join-Path $botDir "pathtracing\recordings\$($run.Recording)"
            Save-Artifact $source
            Save-Artifact "$source.tmp"
            foreach ($problem in Check-Replay $run $reference) { $errors.Add($problem) }
            if (Test-Path -LiteralPath (Join-Path $botDir 'logs\latest.log')) {
              Copy-Item -LiteralPath (Join-Path $botDir 'logs\latest.log') -Destination "$result\$($run.Recording).client.log"
            }
          } catch { $errors.Add($_.Exception.Message) }
        }
        $run.FullSeconds = $session.Elapsed.TotalSeconds
        $run.Reason = $errors -join ' '
        if (-not $errors.Count) { $run.Status = 'qualified' }
        Note "$($run.Course) $($run.Rate) TPS $($run.Status) $($run.Reason)"
      }
      if ($run.Rate -eq 20) { $reference = $run }
      if ($failures.Count) { throw 'Stopping the matrix after a cleanup failure.' }
    }
  }
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
  if ((Test-Path -LiteralPath $snapshot) -and -not $started.Count -and -not $failures.Count) {
    $resolved = (Resolve-Path -LiteralPath $snapshot).Path
    if ($resolved -ceq (Join-Path (Resolve-Path -LiteralPath $result).Path 'scratch-rig')) {
      try { Remove-Item -LiteralPath $resolved -Recurse -Force } catch { $failures.Add($_.Exception.Message) }
    }
  }
  Write-Report @("Reference rig $referenceRig on port $ServerPort", "Fixture profile $profileName",
    "Bot jar SHA-256 $jarHash", "Full run seconds $($watch.Elapsed.TotalSeconds)")
}
if ($failures.Count -or @($runs | Where-Object { $_.Rate -ne 4000 -and $_.Status -ne 'qualified' }).Count) { exit 1 }
