param([string]$Evidence = 'D:\path-tracing-evidence\stage2-flight-20261118')
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_ROOT = 'D:\path-tracing-lab'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
$env:PATHTRACING_SERVER_PORT = '25566'
. "$PSScriptRoot\..\scripts\common.ps1"

$tree = $Root
$reference = $Rig
$null = New-Item -ItemType Directory -Force $Evidence
$snapshot = Join-Path $Evidence 'scratch-rig'
$shmPath = Join-Path $Evidence 'server-mind.shm'
$mindExe = 'D:\path-tracing-target\server-mind\release\mind.exe'
$modJar = 'D:\path-tracing-target\server-mod-build\pathtracing-server.jar'

if (-not (Test-Path $mindExe)) { throw "Missing $mindExe" }
if (-not (Test-Path $modJar)) { throw "Missing $modJar" }

$lock = $null
$serverProc = $null
$mindProc = $null

function Rcon([string]$Command) {
  & "$tree\scripts\rcon.ps1" -Commands @($Command) -SocketTimeoutMilliseconds 5000
}

function Await([scriptblock]$Condition, [int]$Seconds, [string]$Name) {
  $watch = [Diagnostics.Stopwatch]::StartNew()
  while ($watch.Elapsed.TotalSeconds -lt $Seconds) {
    if ($script:serverProc -and $script:serverProc.HasExited) {
      throw "Server exited ($($script:serverProc.ExitCode)) during $Name"
    }
    if (& $Condition) { return }
    Start-Sleep -Milliseconds 250
  }
  throw "Timed out after $Seconds seconds: $Name (last status: $(try { Rcon 'swarm status' } catch { '<rcon failed>' }))"
}

function Start-Mind {
  $script:mindProc = Start-Process -FilePath $mindExe -ArgumentList @('--shm', $shmPath) `
    -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput "$Evidence\mind-stdout.log" -RedirectStandardError "$Evidence\mind-stderr.log"
}

function Stop-Mind {
  if ($script:mindProc -and -not $script:mindProc.HasExited) {
    $script:mindProc.Kill()
    $script:mindProc.WaitForExit(5000) | Out-Null
  }
  $script:mindProc = $null
}

function Stop-Server {
  if ($script:serverProc -and -not $script:serverProc.HasExited) {
    try { Rcon 'stop' | Out-Null } catch {}
    if (-not $script:serverProc.WaitForExit(15000)) {
      $script:serverProc.Kill()
      $script:serverProc.WaitForExit()
    }
  }
  $script:serverProc = $null
}

try {
  $lock = [IO.File]::Open((Join-Path $reference 'rig.lock'), 'CreateNew', 'Write', 'Read')
  $bytes = [Text.Encoding]::ASCII.GetBytes((@{ holder = 'stage2-flight'; pid = $PID; since = (Get-Date).ToString('s') } | ConvertTo-Json))
  $lock.Write($bytes, 0, $bytes.Length)
  $lock.Flush()

  if ((Test-Port $ServerHost $ServerPort) -or (Test-Port $ServerHost $RconPort)) {
    throw 'Reference server already active'
  }
  $clients = @(Get-Process javaw -ErrorAction SilentlyContinue)
  if ($clients.Count) {
    throw 'A Minecraft client is already active; refusing shared rig use'
  }
  if (Test-Path $snapshot) {
    throw "Snapshot already exists at $snapshot; use a fresh evidence directory"
  }

  $null = New-Item -ItemType Directory -Force "$snapshot\server"
  foreach ($item in Get-ChildItem "$reference\server" -Force | Where-Object { $_.Name -notin @('logs', 'crash-reports') }) {
    Copy-Item -LiteralPath $item.FullName -Destination "$snapshot\server" -Recurse
  }
  $api = Get-ChildItem "$reference\clients\bot\mods" -Filter 'fabric-api-*.jar' | Select-Object -First 1
  Copy-Item $api.FullName, $modJar "$snapshot\server\mods" -Force

  $props = Join-Path $snapshot 'server\server.properties'
  $settings = [IO.File]::ReadAllText($props) `
    -replace '(?m)^max-players=.*$', 'max-players=64' `
    -replace '(?m)^pause-when-empty-seconds=.*$', 'pause-when-empty-seconds=-1'
  [IO.File]::WriteAllText($props, $settings, [Text.UTF8Encoding]::new($false))

  $Rig = $snapshot
  $env:PATHTRACING_ROOT = $snapshot

  $jvm = @(
    '-Xms2G', '-Xmx4G',
    "-Djava.io.tmpdir=$Evidence",
    "-Dpathtracing.shm=$shmPath",
    '-jar', 'fabric-server-launch.jar', 'nogui'
  )
  $script:serverProc = Start-Process -FilePath "$Jdk\java.exe" -ArgumentList $jvm `
    -WorkingDirectory "$snapshot\server" -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput "$Evidence\server-stdout.log" -RedirectStandardError "$Evidence\server-stderr.log"

  Await { Test-Port $ServerHost $RconPort } 60 'server startup'
  Start-Mind
  Await { (Rcon 'swarm status') -match 'engine=connected' } 30 'mind engine connect'

  Rcon 'difficulty peaceful' | Out-Null
  Rcon 'tick rate 20' | Out-Null

  # Flight 1: 1 bot, 50-block sprint-jump via solve
  Rcon 'swarm spawn 1' | Out-Null
  Rcon 'swarm goto 50 -60 0' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=1/1 settled=1/1' } 60 'Flight 1 arrive & settle'
  $f1 = Rcon 'swarm status'
  Write-Output "FLIGHT 1: $f1"
  if ($f1 -notmatch 'exact_T=1/1' -or $f1 -notmatch 'teleports=0') { throw "Flight 1 check failed: $f1" }

  # Flight 2: 50 bots, 80-block sprint-jump in grid via one shared GeneratorStore
  Rcon 'swarm remove' | Out-Null
  Rcon 'swarm profile reset' | Out-Null
  Rcon 'swarm spawn 50 1.5' | Out-Null
  Rcon 'swarm goto 80 -60 0' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=50/50 settled=50/50' } 90 'Flight 2 arrive & settle'
  $f2 = Rcon 'swarm status'
  Write-Output "FLIGHT 2: $f2"
  if ($f2 -notmatch 'exact_T=50/50' -or $f2 -notmatch 'teleports=0') { throw "Flight 2 check failed: $f2" }

  # Flight 3: Hot-reload mind.exe while server and 50 bots stay live
  Stop-Mind
  Start-Sleep -Seconds 2
  $f3Down = Rcon 'swarm status'
  Write-Output "FLIGHT 3 (down): $f3Down"
  if ($f3Down -notmatch 'engine=disconnected') { throw "Flight 3 expected disconnected: $f3Down" }

  Start-Mind
  Await {
    $s = Rcon 'swarm status'
    ($s -match 'engine=connected') -and ($s -match 'engine_epoch=2') -and ($s -notmatch 'sections=0\b')
  } 15 'Flight 3 mind hot-reload'
  $f3Up = Rcon 'swarm status'
  Write-Output "FLIGHT 3 (reconnected): $f3Up"

  Rcon 'swarm goto 10 -60 0' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=50/50 settled=50/50' } 90 'Flight 3 return flight'
  $f3Back = Rcon 'swarm status'
  Write-Output "FLIGHT 3 (returned): $f3Back"
  if ($f3Back -notmatch 'exact_T=50/50' -or $f3Back -notmatch 'teleports=0') { throw "Flight 3 return check failed: $f3Back" }

  # Flight 4: Live block delta (oak_fence placed in corridor -> NeedsJoints)
  Rcon 'swarm remove' | Out-Null
  $sb = Rcon 'setblock 20 -60 0 minecraft:oak_fence'
  Write-Output "FLIGHT 4 (setblock): $sb"
  Rcon 'swarm spawn 1' | Out-Null
  Rcon 'swarm goto 30 -60 0' | Out-Null
  Await { (Rcon 'swarm status') -match 'status=.*joints:1' } 15 'Flight 4 fence NeedsJoints'
  $f4 = Rcon 'swarm status'
  Write-Output "FLIGHT 4: $f4"
  if ($f4 -notmatch 'joints:1' -or $f4 -notmatch '\[20,_-60,_0\]') { throw "Flight 4 check failed: $f4" }

  $stdout = Get-Content "$Evidence\server-stdout.log" -Raw
  $movedWrongly = ([regex]::Matches($stdout, 'moved wrongly')).Count
  Write-Output "MOVED_WRONGLY: $movedWrongly"
  if ($movedWrongly -ne 0) { throw "Unexpected moved wrongly warnings: $movedWrongly" }

  @(
    "FLIGHT 1: $f1",
    "FLIGHT 2: $f2",
    "FLIGHT 3 (down): $f3Down",
    "FLIGHT 3 (reconnected): $f3Up",
    "FLIGHT 3 (returned): $f3Back",
    "FLIGHT 4: $f4",
    "MOVED_WRONGLY: $movedWrongly"
  ) | Set-Content "$Evidence\report.txt" -Encoding UTF8
} finally {
  Stop-Mind
  Stop-Server
  if ($lock) {
    $lock.Dispose()
    Remove-Item -LiteralPath (Join-Path $reference 'rig.lock') -ErrorAction SilentlyContinue
  }
}
