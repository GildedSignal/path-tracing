param([string]$Evidence = 'D:\path-tracing-evidence\stage3-flight-20261118')
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

function Sample-Mem {
  $jvmMb = 0.0
  $mindMb = 0.0
  if ($script:serverProc -and -not $script:serverProc.HasExited) {
    $script:serverProc.Refresh()
    $jvmMb = [math]::Round($script:serverProc.WorkingSet64 / 1MB, 1)
  }
  if ($script:mindProc -and -not $script:mindProc.HasExited) {
    $script:mindProc.Refresh()
    $mindMb = [math]::Round($script:mindProc.WorkingSet64 / 1MB, 1)
  }
  $sumMb = [math]::Round($jvmMb + $mindMb, 1)
  return @{ jvm = $jvmMb; mind = $mindMb; sum = $sumMb; text = "mem_MB=jvm:$jvmMb,mind:$mindMb,sum:$sumMb" }
}

function Assert-TickP99Below([string]$Status, [string]$FlightName, [double]$LimitMs) {
  if ($Status -notmatch 'tick_ms=[\d.]+/[\d.]+/[\d.]+/([\d.]+)/[\d.]+\(') {
    throw "$FlightName missing tick_ms: $Status"
  }
  $p99 = [double]$Matches[1]
  if ($p99 -ge $LimitMs) {
    throw "$FlightName tick p99=${p99}ms exceeded ${LimitMs}ms limit: $Status"
  }
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
  $bytes = [Text.Encoding]::ASCII.GetBytes((@{ holder = 'stage3-flight'; pid = $PID; since = (Get-Date).ToString('s') } | ConvertTo-Json))
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
    '-Xms2G', '-Xmx6G',
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

  # Flight 1: 1,000 Dense Bots (auto, 8 audit canaries)
  Rcon 'swarm config 128 32 16 8 100' | Out-Null
  Rcon 'swarm spawn 1000 1.0 auto -200 -60 -200' | Out-Null
  Rcon 'swarm goto -120 -60 -200' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=1000/1000 settled=1000/1000' } 60 'Flight 1 1000 dense'
  $f1 = Rcon 'swarm status'
  $m1 = Sample-Mem
  Write-Output "FLIGHT 1: $f1 $($m1.text)"
  if ($f1 -notmatch 'exact_T=1000/1000' -or $f1 -notmatch 'teleports=0') { throw "Flight 1 failed: $f1" }
  if ($f1 -notmatch 'tiers=.*mat:8\b') { throw "Flight 1 expected 8 audit canaries: $f1" }
  Assert-TickP99Below $f1 'Flight 1' 50.0

  # Flight 2: 10,000 Dense Bots (auto, 8 audit canaries)
  Rcon 'swarm remove' | Out-Null
  Rcon 'swarm profile reset' | Out-Null
  Rcon 'swarm spawn 10000 1.0 auto -200 -60 -200' | Out-Null
  Rcon 'swarm goto -120 -60 -200' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=10000/10000 settled=10000/10000' } 90 'Flight 2 10000 dense'
  $f2 = Rcon 'swarm status'
  $m2 = Sample-Mem
  Write-Output "FLIGHT 2: $f2 $($m2.text)"
  if ($f2 -notmatch 'exact_T=10000/10000' -or $f2 -notmatch 'teleports=0') { throw "Flight 2 failed: $f2" }
  Assert-TickP99Below $f2 'Flight 2' 50.0
  if ($m2.sum -ge 28000) { throw "Flight 2 RAM exceeded safety budget: $($m2.text)" }

  # Flight 3: 1,000 Dispersed Bots Across Unloaded .mca Chunks (auto, 4 audit canaries)
  Rcon 'swarm remove' | Out-Null
  Rcon 'swarm profile reset' | Out-Null
  Rcon 'swarm config 128 32 16 4 100' | Out-Null
  Rcon 'swarm spawn 1000 6.0 auto 5250 -60 3960 75' | Out-Null
  Rcon 'swarm goto 5310 -60 3960' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=1000/1000 settled=1000/1000' } 90 'Flight 3 1000 dispersed'
  $f3 = Rcon 'swarm status'
  $m3 = Sample-Mem
  Write-Output "FLIGHT 3: $f3 $($m3.text)"
  if ($f3 -notmatch 'exact_T=1000/1000' -or $f3 -notmatch 'teleports=0') { throw "Flight 3 failed: $f3" }
  if ($f3 -notmatch 'disk_sections=([1-9]\d*)') { throw "Flight 3 expected disk_sections > 0: $f3" }
  if ($f3 -notmatch 'tickets=([1-9]\d*)') { throw "Flight 3 expected tickets > 0: $f3" }
  if ($f3 -match 'loaded_chunks=(\d+)' -and [int]$Matches[1] -ge 2500) { throw "Flight 3 loaded_chunks too high: $f3" }
  Assert-TickP99Below $f3 'Flight 3' 50.0

  # Flight 4: Dynamic Mid-Flight Promotion & Demotion Around an Observer
  Rcon 'swarm remove' | Out-Null
  Rcon 'swarm profile reset' | Out-Null
  Rcon 'swarm config 64 16 8 0 100' | Out-Null
  Rcon 'swarm spawn 200 1.5 auto -200 -60 -200' | Out-Null
  Await { (Rcon 'swarm status') -match 'bots=200 .*tiers=virt:200,vis:0,mat:0' } 15 'Flight 4 initial virtual'
  $f4Init = Rcon 'swarm status'
  Write-Output "FLIGHT 4 (init): $f4Init"

  Rcon 'swarm observer add TestHuman -200 -60 -200 64' | Out-Null
  Await {
    $s = Rcon 'swarm status'
    ($s -match 'tiers=virt:\d+,vis:([1-9]\d*),mat:([1-9]\d*)') -and ($s -match 'vis_pkts=([1-9]\d*)')
  } 15 'Flight 4 observer promotion'
  $f4Obs = Rcon 'swarm status'
  Write-Output "FLIGHT 4 (obs): $f4Obs"

  Rcon 'swarm goto -140 -60 -200' | Out-Null
  Start-Sleep -Milliseconds 1500
  Rcon 'swarm observer move TestHuman -160 -60 -200' | Out-Null
  Start-Sleep -Milliseconds 1500
  Rcon 'swarm observer remove TestHuman' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=200/200 settled=200/200.*tiers=virt:200,vis:0,mat:0' } 60 'Flight 4 arrive & demote'
  $f4 = Rcon 'swarm status'
  $m4 = Sample-Mem
  Write-Output "FLIGHT 4 (end): $f4 $($m4.text)"
  if ($f4 -notmatch 'exact_T=200/200' -or $f4 -notmatch 'teleports=0') { throw "Flight 4 failed: $f4" }
  if ($f4 -notmatch 'prom=([1-9]\d*)' -or $f4 -notmatch 'dem=([1-9]\d*)' -or $f4 -notmatch 'vis_pkts=([1-9]\d*)') {
    throw "Flight 4 missing prom/dem/vis_pkts: $f4"
  }

  # Flight 5: Hooked-Block Causal Promotion (oak_pressure_plate + copper_bulb)
  Rcon 'swarm remove' | Out-Null
  Rcon 'swarm observer clear' | Out-Null
  Rcon 'swarm config 64 16 8 0 100' | Out-Null
  Rcon 'forceload add 0 0 31 15' | Out-Null
  Await { (Rcon 'swarm status') -match 'loaded_chunks=([1-9]\d*)' } 15 'Flight 5 forceload setup'
  $fill1 = Rcon 'fill 14 -60 0 19 -60 0 minecraft:oak_pressure_plate'
  if ($fill1 -notmatch 'Successfully filled') { throw "Flight 5 plate fill failed: $fill1" }
  $fill2 = Rcon 'fill 14 -60 1 19 -60 1 minecraft:copper_bulb'
  if ($fill2 -notmatch 'Successfully filled') { throw "Flight 5 bulb fill failed: $fill2" }
  Start-Sleep -Milliseconds 300
  Rcon 'forceload remove 0 0 31 15' | Out-Null
  Await { (Rcon 'swarm status') -match 'loaded_chunks=0\b' } 30 'Flight 5 unload after fill'
  Rcon 'swarm profile reset' | Out-Null
  Rcon 'swarm spawn 1 1.5 auto 0 -60 0' | Out-Null
  Await { (Rcon 'swarm status') -match 'bots=1 .*tiers=virt:1,vis:0,mat:0' } 15 'Flight 5 initial virtual'
  $f5Init = Rcon 'swarm status'
  Write-Output "FLIGHT 5 (init): $f5Init"

  Rcon 'swarm goto 30 -60 0' | Out-Null
  Await { (Rcon 'swarm status') -match 'arrived=1/1 settled=1/1' } 60 'Flight 5 arrive & settle'
  Start-Sleep -Milliseconds 300
  $f5 = Rcon 'swarm status'
  $m5 = Sample-Mem
  Rcon 'forceload add 0 0 31 15' | Out-Null
  Await { (Rcon 'swarm status') -match 'loaded_chunks=([1-9]\d*)' } 15 'Flight 5 forceload check'
  $litBulbs = @()
  foreach ($bx in 14..19) {
    $res = Rcon "execute if block $bx -60 1 minecraft:copper_bulb[lit=true]"
    if ($res -match 'Test passed') { $litBulbs += $bx }
  }
  Rcon 'forceload remove 0 0 31 15' | Out-Null
  Write-Output "FLIGHT 5 (end): $f5 $($m5.text) lit_bulbs=$($litBulbs -join ',')"
  if ($f5 -notmatch 'exact_T=1/1' -or $f5 -notmatch 'teleports=0') { throw "Flight 5 failed: $f5" }
  if ($f5 -notmatch 'hook_prom=([1-9]\d*)') { throw "Flight 5 expected hook_prom >= 1: $f5" }
  if ($litBulbs.Count -lt 1) { throw "Flight 5 expected at least one lit copper_bulb at 14..19 -60 1" }

  $stdout = Get-Content "$Evidence\server-stdout.log" -Raw
  $movedWrongly = ([regex]::Matches($stdout, 'moved wrongly')).Count
  Write-Output "MOVED_WRONGLY: $movedWrongly"
  if ($movedWrongly -ne 0) { throw "Unexpected moved wrongly warnings: $movedWrongly" }

  @(
    "FLIGHT 1: $f1 $($m1.text)",
    "FLIGHT 2: $f2 $($m2.text)",
    "FLIGHT 3: $f3 $($m3.text)",
    "FLIGHT 4 (init): $f4Init",
    "FLIGHT 4 (obs): $f4Obs",
    "FLIGHT 4 (end): $f4 $($m4.text)",
    "FLIGHT 5 (init): $f5Init",
    "FLIGHT 5 (end): $f5 $($m5.text) lit_bulbs=$($litBulbs -join ',')",
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
