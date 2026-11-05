param(
  [string]$Jar,
  [ValidateRange(1, 2147483647)][int]$Runs = 8,
  [ValidateRange(1, 2147483647)][int]$Ticks = 400,
  [int]$Seed = 1
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"

$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$result = Join-Path $PSScriptRoot "results\$stamp-gate-1.2"
$null = New-Item -ItemType Directory -Path $result
$report = Join-Path $result 'report.md'
$powershell = Join-Path $PSHOME 'powershell.exe'
$scripts = Join-Path $PSScriptRoot '..\..\scripts'
$botDir = Join-Path $Rig 'clients\bot'
$start = '0.5,-60,0.5,0,0'
$prefix = 'g12_' + $stamp
# Leave room for the helper's ten-second transport allowance in WaitForExit(int milliseconds).
$timeout = [int][Math]::Min(2147400, [Math]::Ceiling($Ticks / 10.0) + 60)
$locked = $false
$exitCode = 1
$started = @{}
$artifacts = [Collections.Generic.List[string]]::new()
$commands = [Collections.Generic.List[string]]::new()
$notes = [Collections.Generic.List[string]]::new()
$gates = @{}
foreach ($n in 1..4) { $gates[$n] = [pscustomobject]@{ Pass = $false; Reason = 'Not run.'; File = 'driver.log' } }

function Note([string]$Text) {
  $notes.Add($Text)
  Add-Content -LiteralPath (Join-Path $result 'driver.log') -Value $Text -Encoding UTF8
}
function Quote([string]$Value) { "'" + $Value.Replace("'", "''") + "'" }

# Helpers call exit, and RCON has no socket timeout. Isolate and bound every invocation.
function Invoke-Script([string]$Path, [hashtable]$Parameters = @{}, [int]$Seconds = 30, [string]$LaunchRole = '') {
  $name = '{0:D4}-{1}' -f ($commands.Count + 1), [IO.Path]::GetFileNameWithoutExtension($Path)
  $commands.Add("$name.log")
  $call = '& ' + (Quote $Path)
  foreach ($key in ($Parameters.Keys | Sort-Object)) { $call += ' -' + $key + ' ' + (Quote ([string]$Parameters[$key])) }
  $code = "`$ErrorActionPreference = 'Stop'; `$ProgressPreference = 'SilentlyContinue'; `$PSDefaultParameterValues['Start-Process:WindowStyle'] = 'Hidden'; "
  if ($LaunchRole) {
    # start-server does not request PassThru; capture ownership before waiting for readiness.
    $code += "`$PSDefaultParameterValues['Start-Process:PassThru'] = `$true; "
    $call += " | ForEach-Object { if (`$_ -is [Diagnostics.Process]) { 'GATE_PROCESS ' + `$_.Id + ' ' + `$_.StartTime.ToUniversalTime().Ticks } else { `$_ } }"
  }
  $code += "try { $call } catch { Write-Output `$_; exit 1 }"
  $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($code))
  $out = Join-Path $result "$name.stdout.txt"
  $err = Join-Path $result "$name.stderr.txt"
  $log = Join-Path $result "$name.log"
  Set-Content -LiteralPath $log -Value $call -Encoding UTF8
  $process = Start-Process -FilePath $powershell -WindowStyle Hidden -PassThru -ArgumentList @(
    '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', $encoded
  ) -RedirectStandardOutput $out -RedirectStandardError $err
  # Retain the handle: Windows PowerShell's Start-Process can otherwise lose ExitCode.
  $null = $process.Handle
  $timedOut = -not $process.WaitForExit($Seconds * 1000)
  if ($timedOut) { $process.Kill(); $process.WaitForExit() }
  $status = if ($timedOut) { 124 } else { $process.ExitCode }
  $stdout = [IO.File]::ReadAllText($out)
  $stderr = [IO.File]::ReadAllText($err)
  Add-Content -LiteralPath $log -Encoding UTF8 -Value @(
    "Exit: $status; timeout: $timedOut", 'STDOUT:', $stdout, 'STDERR:', $stderr
  )
  if ($LaunchRole -and $stdout -match '(?m)^GATE_PROCESS (\d+) (\d+)\r?$') {
    $started[$LaunchRole] = @{ Id = [int]$Matches[1]; StartTicks = [long]$Matches[2] }
  }
  $process.Dispose()
  [pscustomobject]@{ Code = $status; Text = $stdout; Error = $stderr; File = "$name.log" }
}
function Require($Reply, [string]$Action) {
  if ($Reply.Code -ne 0) { throw "$Action failed (exit $($Reply.Code)); see $($Reply.File)." }
  $Reply
}
function Rcon([string]$Text) {
  Invoke-Script (Join-Path $scripts 'rcon.ps1') @{ Commands = $Text } 10
}
function Players($Reply) {
  $null = Require $Reply 'RCON list'
  if ($Reply.Text.Trim() -notmatch '^There are \d+ of a max of \d+ players online:\s*(.*)$') {
    throw "Unrecognised RCON list; see $($Reply.File)."
  }
  @($Matches[1].Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ })
}
function Wait-Rcon {
  $clock = [Diagnostics.Stopwatch]::StartNew()
  do {
    if (Test-Port $ServerHost $RconPort) {
      $reply = Rcon 'list'
      if ($reply.Code -eq 0) { return (Players $reply) }
    }
    Start-Sleep -Milliseconds 500
  } while ($clock.Elapsed.TotalSeconds -lt 120)
  $script:exitCode = 4
  throw 'Timed out waiting for server RCON.'
}
function Assert-Absent {
  $reply = Rcon 'list'
  if ($PlayerName -in (Players $reply)) {
    $script:exitCode = 3
    throw "$PlayerName is online. Refusing to disturb Julien; close his game connection. See $($reply.File)."
  }
}
function Bot([string]$Text, [string]$WaitFor, [int]$Seconds = 30, [string]$FailOn = '') {
  Assert-Absent
  $parameters = @{ Text = $Text; WaitFor = $WaitFor; TimeoutSeconds = $Seconds }
  if ($FailOn) { $parameters.FailOn = $FailOn }
  Invoke-Script (Join-Path $PSScriptRoot 'bot-command.ps1') $parameters ($Seconds + 10)
}
function Bot-Processes {
  # The launcher puts the role in an argument file, not in javaw's command line.
  $argPath = (Join-Path $botDir 'launch.args').Replace('\', '/')
  @(Get-CimInstance Win32_Process -Filter "Name = 'javaw.exe' OR Name = 'java.exe'" | Where-Object {
    $line = ([string]$_.CommandLine).Replace('\', '/')
    $line.IndexOf($argPath, [StringComparison]::OrdinalIgnoreCase) -ge 0 -or
      $line -match '(?:^|\s)-Dpathtracing\.role=bot(?:\s|$)'
  })
}
function Save-Artifact([string]$Path) {
  if (Test-Path -LiteralPath $Path -PathType Leaf) {
    Copy-Item -LiteralPath $Path -Destination (Join-Path $result ([IO.Path]::GetFileName($Path))) -Force
  }
}
function Gate([int]$Number, [bool]$Pass, [string]$Reason, [string]$File) {
  $gates[$Number] = [pscustomobject]@{ Pass = $Pass; Reason = $Reason; File = $File }
}
function Diagnostic($Reply) {
  $line = @((($Reply.Text + "`n" + $Reply.Error) -split '\r?\n') | Where-Object { $_.Trim() } | Select-Object -First 1)
  $text = ($line -join '').Trim()
  if ($text.Length -gt 240) { $text = $text.Substring(0, 240) + '...' }
  "Exit $($Reply.Code): $text"
}
function Read-Ticks([string]$Path) {
  if (-not (Test-Path -LiteralPath $Path)) { return }
  $stream = [IO.File]::Open($Path, 'Open', 'Read', [IO.FileShare]::ReadWrite)
  $reader = [IO.StreamReader]::new($stream)
  try {
    # Ignore a partially written last line; only complete LF-terminated rows count.
    $text = $reader.ReadToEnd()
    $last = $text.LastIndexOf("`n")
    if ($last -lt 0) { return }
    foreach ($line in $text.Substring(0, $last).Split("`n")) {
      $row = $line | ConvertFrom-Json
      if ($null -ne $row.PSObject.Properties['t']) { $row }
    }
  } finally { $reader.Dispose() }
}
function Wait-Ticks([string]$Path, [int]$Minimum) {
  $clock = [Diagnostics.Stopwatch]::StartNew()
  do {
    $rows = @(Read-Ticks $Path)
    if ($rows.Count -ge $Minimum) { return $rows }
    Start-Sleep -Milliseconds 100
  } while ($clock.Elapsed.TotalSeconds -lt 15)
  throw "Timed out waiting for $Minimum recorded ticks in $Path."
}
function Stop-Owned([string]$Role) {
  if (-not $started.ContainsKey($Role)) { return }
  $owned = $started[$Role]
  $process = Get-Process -Id $owned.Id -ErrorAction SilentlyContinue
  if ($null -eq $process) { Note "$Role already exited."; return }
  if ($process.StartTime.ToUniversalTime().Ticks -ne $owned.StartTicks) { throw "$Role PID was reused; refusing to stop it." }
  if ($Role -eq 'server') {
    $reply = Rcon 'stop'
    Note "Server stop: see $($reply.File)."
    if ($process.WaitForExit(15000)) { return }
  }
  # PID and start time identify only the process created by this invocation.
  $process.Kill()
  if (-not $process.WaitForExit(10000)) { throw "Started $Role process did not exit." }
  Note "Stopped owned $Role process $($owned.Id)."
}

Note "Rig: $Rig; server ${ServerHost}:$ServerPort; RCON $RconPort; bot $BotName; owner $PlayerName."
Note "Start: $start (x,y,z,yaw,pitch), lab superflat; survival; Runs=$Runs; Ticks=$Ticks; Seed=$Seed."
try {
  Enter-Rig 'gate-1.2'
  $locked = $true
  if ([long]$Seed + $Runs -gt [int]::MaxValue) { throw 'Seed + Runs must fit a signed 32-bit seed.' }

  $serverOpen = Test-Port $ServerHost $ServerPort
  $online = @()
  if ($serverOpen -or (Test-Port $ServerHost $RconPort)) {
    $online = @(Wait-Rcon)
    if ($PlayerName -in $online) { $exitCode = 3; throw "$PlayerName is online. Refusing to disturb Julien." }
  }
  $runningBots = @(Bot-Processes)
  $botRunning = $runningBots.Count -gt 0 -or $BotName -in $online
  if ($Jar) {
    $source = (Resolve-Path -LiteralPath $Jar).Path
    $target = Join-Path $botDir 'mods\pathtracing.jar'
    $same = (Test-Path -LiteralPath $target) -and
      ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -eq (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash)
    if ($botRunning -and -not $same) {
      $exitCode = 3
      throw 'The jar differs and the bot client is running. Close the bot client before supplying -Jar.'
    }
    if (-not $botRunning -and -not $same) { Copy-Item -LiteralPath $source -Destination $target -Force }
    Note "Bot jar SHA256: $((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash)."
  } else {
    $target = Join-Path $botDir 'mods\pathtracing.jar'
    if (-not (Test-Path -LiteralPath $target)) { $exitCode = 3; throw "Missing $target; supply -Jar." }
    Note "Installed bot jar SHA256: $((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash)."
  }
  $options = Join-Path $botDir 'options.txt'
  if (-not (Test-Path -LiteralPath $options)) { $exitCode = 3; throw "Missing bot options: $options; run scripts\setup.ps1." }
  Save-Artifact $options
  $lines = @(Get-Content -LiteralPath $options)
  foreach ($setting in @('autoJump:false', 'toggleSprint:false', 'toggleCrouch:false', 'sprintWindow:0')) {
    $key = $setting.Split(':')[0]
    $values = @($lines | Where-Object { $_ -cmatch ('^' + $key + ':') })
    if ($values.Count -ne 1 -or $values[0] -cne $setting) {
      $exitCode = 3
      throw "Bot options must contain exactly $setting. Run scripts\setup.ps1 with the bot closed."
    }
  }
  if (-not $serverOpen) {
    $null = Require (Invoke-Script (Join-Path $scripts 'start-server.ps1') @{} 30 'server') 'Server launch'
  }
  $online = @(Wait-Rcon)
  Assert-Absent
  if ($BotName -notin $online -and @(Bot-Processes).Count -eq 0) {
    $null = Require (Invoke-Script (Join-Path $scripts 'launch-client.ps1') @{ Role = 'bot' } 30 'bot') 'Bot launch'
  }
  $clock = [Diagnostics.Stopwatch]::StartNew()
  $ready = $false
  do {
    $reply = Rcon 'list'
    $online = @(Players $reply)
    if ($PlayerName -in $online) { $exitCode = 3; throw "$PlayerName is online. Refusing to disturb Julien." }
    if ($BotName -in $online) {
      # Unknown top-level words are silent in Bot.java; an unknown rec word returns Usage.
      $reply = Bot 'rec gate-ping' 'Usage: replay' 2 'Recorder/dumper:|Replay is running\.'
      if ($reply.Code -eq 0) { $ready = $true; break }
    }
    Start-Sleep -Milliseconds 500
  } while ($clock.Elapsed.TotalSeconds -lt 180)
  if (-not $ready) { $exitCode = 4; throw 'Timed out waiting for PathBot and its recorder usage reply.' }
  $null = Require (Bot 'stop' 'Stopped\.') 'Bot stop'
  Assert-Absent
  $null = Require (Rcon "gamemode survival $BotName") 'Survival mode'
  $null = Require (Rcon "tp $BotName 0.5 -60 0.5 0 0") 'Course teleport'

  $gate1 = $true
  $gate2 = $true
  $failure1 = $null
  $failure2 = $null
  $evidence1 = [Collections.Generic.List[string]]::new()
  $evidence2 = [Collections.Generic.List[string]]::new()
  for ($i = 1; $i -le $Runs; $i++) {
    $inputName = "${prefix}_$i"
    $generated = Invoke-Script (Join-Path $PSScriptRoot 'new-inputs.ps1') @{
      Name = $inputName; Ticks = $Ticks; Seed = ($Seed + $i); Start = $start
    }
    Save-Artifact (Join-Path $botDir "pathtracing\inputs\$inputName.jsonl")
    $pair = @()
    foreach ($suffix in @('a', 'b')) {
      $name = "${inputName}_$suffix"
      $path = Join-Path $botDir "pathtracing\recordings\$name.jsonl"
      $artifacts.Add($path)
      $pair += (Join-Path $result "$name.jsonl")
      if ($generated.Code -eq 0) {
        $reply = Bot "replay $inputName $name" ("Saved ${name}: $Ticks ticks\.") $timeout
      } else { $reply = $generated }
      $evidence1.Add("$name replay: exit $($reply.Code); $($reply.File)")
      if ($reply.Code -ne 0) {
        $gate1 = $false; $gate2 = $false
        if ($null -eq $failure1) { $failure1 = @{ Reason = "Replay $name did not complete. $(Diagnostic $reply)"; File = $reply.File } }
        if ($null -eq $failure2) { $failure2 = @{ Reason = "Recording $name did not complete. $(Diagnostic $reply)"; File = $reply.File } }
      }
      Save-Artifact $path
      Save-Artifact "$path.tmp"
      $check = Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $pair[-1] } $timeout
      $evidence2.Add("$name check: exit $($check.Code); $($check.File)")
      if ($check.Code -ne 0) {
        $gate2 = $false
        if ($null -eq $failure2) { $failure2 = @{ Reason = "Recording $name failed. $(Diagnostic $check)"; File = $check.File } }
      }
      # A missing completion may leave our replay running. Cancel only our own output.
      if ($reply.Code -ne 0 -and (Test-Path -LiteralPath "$path.tmp")) {
        $null = Require (Bot 'rec stop' ("Saved ${name}:")) 'Cancel failed replay'
        Save-Artifact $path
      }
    }
    $compared = Invoke-Script (Join-Path $PSScriptRoot 'compare-recordings.ps1') @{ A = $pair[0]; B = $pair[1] } $timeout
    $evidence1.Add("Pair $i comparison: exit $($compared.Code); $($compared.File)")
    if ($compared.Code -ne 0 -or $compared.Text.Trim() -cne 'identical') {
      $gate1 = $false
      if ($null -eq $failure1) { $failure1 = @{ Reason = "Pair $i was not identical. $(Diagnostic $compared)"; File = $compared.File } }
    }
    Set-Content -LiteralPath (Join-Path $result 'gate-1.txt') -Value $evidence1 -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $result 'gate-2.txt') -Value $evidence2 -Encoding UTF8
    if ($gate1) { Gate 1 $true "$i of $Runs replay pairs identical." 'gate-1.txt' }
    else { Gate 1 $false $failure1.Reason $failure1.File }
    if ($gate2) { Gate 2 $true "$($i * 2) recordings passed with no anomalies or structural errors." 'gate-2.txt' }
    else { Gate 2 $false $failure2.Reason $failure2.File }
  }

  $reply = Bot 'dump blocks' 'Dumped \d+ block states; independent registry count \d+\.' 120
  $dump = Join-Path $botDir 'pathtracing\dumps\blocks.jsonl'
  if ($reply.Code -eq 0) {
    Save-Artifact $dump
    $dumpCopy = Join-Path $result 'blocks.jsonl'
    Get-Content -LiteralPath $dumpCopy -Tail 1 | Set-Content -LiteralPath (Join-Path $result 'dump-summary.jsonl') -Encoding UTF8
    $check = Invoke-Script (Join-Path $PSScriptRoot 'check-dump.ps1') @{ File = $dumpCopy } 180
    Gate 3 ($check.Code -eq 0) (Diagnostic $check) $check.File
  } else {
    Save-Artifact "$dump.tmp"
    Gate 3 $false 'Dump command did not complete; any retained temporary dump is evidence.' $reply.File
  }

  Assert-Absent
  $null = Require (Rcon "tp $BotName 0.5 -60 0.5 0 0") 'Negative control reset'
  Start-Sleep -Milliseconds 1000
  $name = "${prefix}_negative"
  $path = Join-Path $botDir "pathtracing\recordings\$name.jsonl"
  $artifacts.Add($path)
  $reply = Require (Bot "rec start $name" ("Recording $name\.")) 'Negative recording start'
  $before = @(Wait-Ticks "$path.tmp" 5)
  if (@($before | Where-Object { $null -ne $_.PSObject.Properties['anomaly'] }).Count -gt 0) {
    throw 'Negative control has anomalies before the injected teleport; cannot attribute its failure.'
  }
  Assert-Absent
  $null = Require (Rcon "tp $BotName 8.5 -60 0.5 0 0") 'Negative control teleport'
  $afterCommand = @(Read-Ticks "$path.tmp").Count
  $null = Wait-Ticks "$path.tmp" ($afterCommand + 10)
  $null = Require (Bot 'rec stop' ("Saved ${name}:")) 'Negative recording stop'
  Save-Artifact $path
  $negative = Join-Path $result "$name.jsonl"
  $check = Invoke-Script (Join-Path $PSScriptRoot 'check-recording.ps1') @{ File = $negative }
  $lines = @($check.Text -split '\r?\n' | Where-Object { $_ })
  $injected = @(Read-Ticks $negative | Where-Object {
    $_.t -ge $before.Count -and @($_.anomaly | Where-Object {
      $_.type -eq 'ClientboundPlayerPositionPacket' -and $_.position.x -eq 8.5 -and
        $_.position.y -eq -60 -and $_.position.z -eq 0.5
    }).Count -gt 0
  })
  $onlyAnomalies = $lines.Count -gt 0 -and @($lines | Where-Object { $_ -notmatch '^Anomaly at line \d+, tick \d+: ' }).Count -eq 0
  $negativePass = $check.Code -eq 1 -and $onlyAnomalies -and $injected.Count -gt 0
  $reason = if ($negativePass) { 'Checker rejected only named anomalies and caught the injected teleport after clean baseline ticks.' }
    elseif ($check.Code -ne 1) { "Checker did not reject the teleport recording as required. $(Diagnostic $check)" }
    elseif (-not $onlyAnomalies) {
      'Checker failed for a reason other than anomalies: ' +
        (@($lines | Where-Object { $_ -notmatch '^Anomaly at line \d+, tick \d+: ' }) -join ' ')
    }
    else { 'Named anomalies did not include the injected teleport position packet after baseline ticks.' }
  Gate 4 $negativePass $reason $check.File
} catch {
  $reason = $_.Exception.Message
  Note $reason
  foreach ($n in 1..4) {
    if ($gates[$n].Reason -eq 'Not run.') { Gate $n $false "Not completed: $reason" 'driver.log' }
  }
} finally {
  if ($locked) {
    try {
      foreach ($path in $artifacts) {
        if (Test-Path -LiteralPath "$path.tmp") {
          # Do not stop someone else's recording. Only our unique temporary names qualify.
          $name = [IO.Path]::GetFileNameWithoutExtension($path)
          $reply = Bot 'rec stop' ("Saved ${name}:") 5
          Note "Final recorder stop: $($reply.File)."
        }
        Save-Artifact $path
        Save-Artifact "$path.tmp"
      }
    } catch { Note "Artifact/recording cleanup failed: $($_.Exception.Message)" }
    foreach ($role in @('bot', 'server')) {
      try { Stop-Owned $role } catch { Note "Cleanup failed: $($_.Exception.Message)" }
    }
    try { Exit-Rig } catch { Note "Lock release failed: $($_.Exception.Message)" }
  }
  $allPassed = @($gates.Values | Where-Object { -not $_.Pass }).Count -eq 0
  $clean = @($notes | Where-Object { $_ -match 'cleanup failed:|Cleanup failed:|Lock release failed:' }).Count -eq 0
  $overall = if ($allPassed -and $clean) { 'Overall: PASS' } else { 'Overall: FAIL' }
  if ($allPassed -and $clean) { $exitCode = 0 }
  $body = @('# Gate 1.2', '', "UTC: $stamp", "Rig: $Rig", "Start: $start (x,y,z,yaw,pitch); survival on lab superflat.",
    "Runs: $Runs; ticks per replay: $Ticks; seeds: Seed + i, i=1..Runs; Seed=$Seed.",
    "Replay timeout: $timeout seconds (60 + ceiling(Ticks / 10), capped at 2147400); exit: $exitCode.", '',
    'Assumptions: prepared flat course at y=-60, no nearby players; common.ps1 supplies ports and names.',
    'Usage ping is the unknown recorder subcommand rec gate-ping; unknown top-level words are silent.',
    'Uses the frozen comparison helper (ignores absolute tickCount). No health/food/inventory reset.', '')
  foreach ($n in 1..4) {
    $gate = $gates[$n]
    $status = if ($gate.Pass) { 'PASS' } else { 'FAIL' }
    $body += "Gate ${n}: $status - $($gate.Reason) Evidence: [$($gate.File)]($($gate.File))."
  }
  $body += @('', 'Driver and cleanup:', '') + @($notes | ForEach-Object { "- $_" })
  $body += @('', 'Command outputs (each log includes the invocation, exit, stdout and stderr):', '')
  $body += @($commands | ForEach-Object { "- [$_]($_)" })
  $body += @('', $overall)
  Set-Content -LiteralPath $report -Value $body -Encoding UTF8
}
Write-Output $report
Write-Output $overall
exit $exitCode
