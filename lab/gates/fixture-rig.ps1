# Rig transport and owned-process handling follow run-gate-1.2.ps1.
function Note([string]$Text) {
  $notes.Add($Text)
  Add-Content -LiteralPath (Join-Path $result 'driver.log') -Value $Text -Encoding UTF8
}

function Quote([string]$Value) { "'" + $Value.Replace("'", "''") + "'" }

function Invoke-Script([string]$Path, [hashtable]$Parameters = @{}, [int]$Seconds = 30, [string]$LaunchRole = '') {
  $name = '{0:D4}-{1}' -f ($commands.Count + 1), [IO.Path]::GetFileNameWithoutExtension($Path)
  $commands.Add("$name.log")
  $call = '& ' + (Quote $Path)
  foreach ($key in ($Parameters.Keys | Sort-Object)) {
    $values = @($Parameters[$key] | ForEach-Object { Quote ([string]$_) })
    $argument = if ($Parameters[$key] -is [array]) { '@(' + ($values -join ',') + ')' } else { $values[0] }
    $call += ' -' + $key + ' ' + $argument
  }
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
  if ($Reply.Code -ne 0) { throw "$Action failed. $(Diagnostic $Reply) See $($Reply.File)." }
  $Reply
}

function Rcon([string[]]$Text) {
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

function Diagnostic($Reply) {
  $line = @((($Reply.Text + "`n" + $Reply.Error) -split '\r?\n') | Where-Object { $_.Trim() } | Select-Object -First 1)
  $text = ($line -join '').Trim()
  if ($text.Length -gt 240) { $text = $text.Substring(0, 240) + '...' }
  "Exit $($Reply.Code): $text"
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
