param(
  [string]$Evidence = 'D:\path-tracing-evidence\server-probe-20261118',
  [switch]$CheckOnly
)
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_ROOT = 'D:\path-tracing-lab'
$env:PATHTRACING_DEPS = 'D:\path-tracing-deps\data'
$env:PATHTRACING_SERVER_PORT = '25566'
. "$PSScriptRoot\..\..\scripts\common.ps1"

$tree = $Root
$reference = $Rig
$null = New-Item -ItemType Directory -Force $Evidence
$snapshot = Join-Path $Evidence 'scratch-rig'
$lock = $null
$process = $null

function Rcon([string]$Command) {
  & "$tree\scripts\rcon.ps1" -Commands @($Command) -SocketTimeoutMilliseconds 5000
}

function Await([scriptblock]$Condition, [int]$Seconds, [string]$Name) {
  $watch = [Diagnostics.Stopwatch]::StartNew()
  while ($watch.Elapsed.TotalSeconds -lt $Seconds) {
    if ($process -and $process.HasExited) {
      throw "Server exited ($($process.ExitCode)) during $Name"
    }
    if (& $Condition) { return }
    Start-Sleep -Milliseconds 250
  }
  throw "Timed out after $Seconds seconds: $Name"
}

function Stop-Server {
  if ($script:process -and -not $script:process.HasExited) {
    try { Rcon 'stop' | Out-Null } catch {}
    if (-not $script:process.WaitForExit(15000)) {
      $script:process.Kill()
      $script:process.WaitForExit()
    }
  }
  $script:process = $null
}

$cases = if ($CheckOnly) {
  @(
    @{ mode = 'packet'; layout = 'dense'; n = 1 }
  )
} else {
  @(
    @{ mode = 'packet'; layout = 'dense'; n = 1 },
    @{ mode = 'packet'; layout = 'dense'; n = 10 },
    @{ mode = 'packet'; layout = 'dense'; n = 50 },
    @{ mode = 'packet'; layout = 'dense'; n = 200 },
    @{ mode = 'packet'; layout = 'dispersed'; n = 1 },
    @{ mode = 'packet'; layout = 'dispersed'; n = 10 },
    @{ mode = 'packet'; layout = 'dispersed'; n = 50 },
    @{ mode = 'lazy'; layout = 'dense'; n = 50 },
    @{ mode = 'lazy'; layout = 'dense'; n = 200 },
    @{ mode = 'lazy'; layout = 'dense'; n = 1000 },
    @{ mode = 'virtual'; layout = 'dense'; n = 1000 },
    @{ mode = 'virtual'; layout = 'dense'; n = 10000 }
  )
}

try {
  $lock = [IO.File]::Open((Join-Path $reference 'rig.lock'), 'CreateNew', 'Write', 'Read')
  $bytes = [Text.Encoding]::ASCII.GetBytes((@{ holder = 'server-probe'; pid = $PID; since = (Get-Date).ToString('s') } | ConvertTo-Json))
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
  $probeJar = 'D:\path-tracing-target\server-probe-build\server-probe.jar'
  if (-not (Test-Path $probeJar)) { throw "Missing $probeJar; run build.ps1 first" }
  Copy-Item $api.FullName, $probeJar "$snapshot\server\mods" -Force

  $props = Join-Path $snapshot 'server\server.properties'
  $settings = [IO.File]::ReadAllText($props) `
    -replace '(?m)^max-players=.*$', 'max-players=1005' `
    -replace '(?m)^pause-when-empty-seconds=.*$', 'pause-when-empty-seconds=-1'
  [IO.File]::WriteAllText($props, $settings, [Text.UTF8Encoding]::new($false))

  $Rig = $snapshot
  $env:PATHTRACING_ROOT = $snapshot

  $rows = [Collections.Generic.List[object]]::new()
  foreach ($c in $cases) {
    $m = $c.mode
    $layout = $c.layout
    $n = $c.n
    $name = "$m-$layout-$n"
    $caseDir = Join-Path $Evidence $name
    $null = New-Item -ItemType Directory -Force $caseDir

    # Clean accumulated player files between cases.
    Remove-Item "$snapshot\server\world\players" -Recurse -Force -ErrorAction SilentlyContinue

    $row = [pscustomobject]@{
      case = $name
      mode = $m
      layout = $layout
      n = $n
      status = 'not-run'
      reason = ''
      workingSetBytes = [int64]0
      movedWrongly = 0
      teleports20 = -1
      teleports100 = -1
      loadedChunks20 = -1
      loadedChunks100 = -1
    }
    $rows.Add($row)

    try {
      $jvm = @(
        '-Xms2G', '-Xmx8G',
        "-Djava.io.tmpdir=$caseDir",
        "-XX:FlightRecorderOptions=repository=$caseDir",
        "-Dprobe.output=$caseDir",
        "-XX:StartFlightRecording=settings=profile,filename=$caseDir/profile.jfr,dumponexit=true",
        '-jar', 'fabric-server-launch.jar', 'nogui'
      )
      $script:process = Start-Process -FilePath "$Jdk\java.exe" -ArgumentList $jvm `
        -WorkingDirectory "$snapshot\server" -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput "$caseDir\stdout.log" -RedirectStandardError "$caseDir\stderr.log"

      Await { Test-Port $ServerHost $RconPort } 60 "server startup ($name)"
      Rcon 'difficulty peaceful' | Out-Null
      Rcon 'tick rate 20' | Out-Null
      if ($layout -eq 'dense' -and $m -ne 'virtual') {
        Rcon 'fill -2 -61 -2 35 -61 20 stone' | Out-Null
        Rcon 'fill -2 -60 -2 35 -54 20 air' | Out-Null
      }

      Rcon "probe set $m $n $layout" | Out-Null
      $spawnTimeout = if ($layout -eq 'dispersed' -or $n -ge 1000) { 180 } else { 120 }
      Await { (Rcon 'probe status') -match '^ready ' } $spawnTimeout "$name setup/loading"

      foreach ($rate in @(20, 100)) {
        Rcon "tick rate $rate" | Out-Null
        Rcon "probe measure $rate" | Out-Null
        Await { (Rcon 'probe status') -match '^done ' } 90 "$name at $rate TPS"
        $script:process.Refresh()
        $row.workingSetBytes = [Math]::Max($row.workingSetBytes, $script:process.WorkingSet64)

        $statsFile = Join-Path $caseDir "$rate.stats.json"
        if (Test-Path $statsFile) {
          $st = Get-Content $statsFile -Raw | ConvertFrom-Json
          if ($rate -eq 20) {
            $row.teleports20 = $st.teleports
            $row.loadedChunks20 = $st.loadedChunks
          } else {
            $row.teleports100 = $st.teleports
            $row.loadedChunks100 = $st.loadedChunks
          }
        }
        Write-Output "Measured $name at $rate TPS"
      }
      $row.status = 'measured'
    } catch {
      $row.status = 'refused'
      $row.reason = $_.Exception.Message
      Write-Output "REFUSED $name : $($row.reason)"
      if ($row.reason -match 'during server startup') {
        Stop-Server
        throw
      }
    } finally {
      Stop-Server
      $stdout = Join-Path $caseDir 'stdout.log'
      if (Test-Path $stdout) {
        $row.movedWrongly = @(Select-String -Path $stdout -Pattern 'moved wrongly' -SimpleMatch).Count
      }
      $rows | ConvertTo-Json -Depth 5 | Set-Content "$Evidence\matrix.json" -Encoding UTF8
    }
  }
} finally {
  Stop-Server
  if ($lock) {
    $lock.Dispose()
    Remove-Item -LiteralPath (Join-Path $reference 'rig.lock') -ErrorAction SilentlyContinue
  }
}
