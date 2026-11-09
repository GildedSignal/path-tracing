param(
  [string]$Target = 'D:/path-tracing-target/l0-close-20261109-v5',
  [string]$Evidence = 'D:/path-tracing-evidence/l0-close-20261109',
  [ValidateSet('Preflight','Audit','Full')][string[]]$Stages = @('Preflight','Audit','Full'),
  [string]$PreflightCapture,
  [string]$AuditCapture
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$env:PATHTRACING_ROOT = 'D:/path-tracing-lab'
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
. "$root/scripts/common.ps1"
$reference = Join-Path $Rig 'server'
$clockEvidence = Join-Path $Evidence 'archived-clock'
$locked = $false
try {
  Enter-Rig 'job-1.3-l0-close'
  $locked = $true
  if (@(Get-Process -Name java,javaw -ErrorAction SilentlyContinue).Count -or
      (Test-Port '127.0.0.1' 25566) -or (Test-Port '127.0.0.1' 25576)) {
    throw 'Java or the reference rig is active. Refusing to disturb a live session.'
  }
  $null = New-Item -ItemType Directory -Force -Path $Evidence
  # Preserve unchanged archived controls from the retained rig after worktree cleanup.
  foreach ($stamp in @('20261106T1257229131311Z','20261106T1249512984107Z')) {
    $destination = Join-Path $clockEvidence "$stamp-fixtures"
    $null = New-Item -ItemType Directory -Force -Path $destination
    $name = "l0_${stamp}_edges_1"
    # The rig no longer holds them; an earlier attempt already preserved them here.
    if (-not (Test-Path -LiteralPath "$destination/$name.jsonl")) {
      Copy-Item -LiteralPath "$Rig/clients/bot/pathtracing/recordings/$name.jsonl" -Destination $destination
    }
    if (-not (Test-Path -LiteralPath "$destination/$name.world.jsonl")) {
      Copy-Item -LiteralPath "$Rig/clients/bot/pathtracing/fixtures/$name.world.jsonl" -Destination $destination
    }
  }
  foreach ($mode in $Stages) {
    $port = @{ Preflight = 25567; Audit = 25568; Full = 25569 }[$mode]
    $copy = Join-Path $Evidence $mode.ToLowerInvariant()
    if (-not (Test-Path -LiteralPath $copy)) {
      $null = New-Item -ItemType Directory -Force -Path "$copy/server", "$copy/clients/bot"
      # No process owns these fresh copies; reference files are read only.
      & robocopy.exe $reference "$copy/server" /E /XD logs players /XF session.lock /R:0 /W:0 /NFL /NDL /NJH /NJS | Out-Null
      if ($LASTEXITCODE -gt 7) { throw "Server copy failed: $LASTEXITCODE" }
      # 1.5h/1.6f gate: the client rig's matching Fabric API goes in the copied server's mods.
      Copy-Item -Path "$Rig/clients/bot/mods/fabric-api-*.jar" -Destination "$copy/server/mods"
      foreach ($part in @('mods','config')) {
        Copy-Item -LiteralPath "$Rig/clients/bot/$part" -Destination "$copy/clients/bot" -Recurse
      }
      $options = @(Get-Content -LiteralPath "$Rig/clients/bot/options.txt")
      foreach ($setting in @('renderDistance:4','simulationDistance:12','overrideWidth:480','overrideHeight:270','maxFps:260','enableVsync:false')) {
        $key = $setting.Split(':')[0]
        $options = @($options | Where-Object { -not $_.StartsWith($key + ':') }) + $setting
      }
      # PowerShell 5.1's UTF8 writes a BOM: the client then misreads "version", runs every options
      # datafix, fails on the key names and resets to defaults (accessibility onboarding, no quick play).
      $options | Set-Content -LiteralPath "$copy/clients/bot/options.txt" -Encoding ASCII
      $properties = @(Get-Content -LiteralPath "$copy/server/server.properties")
      foreach ($setting in @("server-port=$port", "rcon.port=$($port+10)", 'server-ip=127.0.0.1',
          'view-distance=4','simulation-distance=4','enable-rcon=true','online-mode=false','level-name=world')) {
        $key = $setting.Split('=')[0]
        $properties = @($properties | Where-Object { -not $_.StartsWith($key + '=') }) + $setting
      }
      $properties | Set-Content -LiteralPath "$copy/server/server.properties" -Encoding ASCII
      foreach ($marker in @('resource-pair.disposable','landing-motion.disposable','terrain-corpus.disposable')) {
        Set-Content -LiteralPath "$copy/$marker" -Value 'Job 1.3 disposable rig' -Encoding ASCII
      }
    }
    $arguments = @{ Mode=$mode; Target=$Target; Rig=$copy; Port=$port; ClockEvidence=$clockEvidence }
    if ($mode -eq 'Full') {
      $arguments.PreflightCapture = $PreflightCapture
      $arguments.AuditCapture = $AuditCapture
    }
    $output = @(& "$root/oracles/1.6g-terrain-corpus/run.ps1" @arguments)
    $output
    $report = $output | Where-Object { $_ -like '*report.md' } | Select-Object -Last 1
    if ($mode -eq 'Preflight') { $PreflightCapture = Split-Path -Parent $report }
    if ($mode -eq 'Audit') { $AuditCapture = Split-Path -Parent $report }
  }
} finally {
  if ($locked) { Exit-Rig }
}
