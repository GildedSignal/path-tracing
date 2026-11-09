param(
  [Parameter(Mandatory)][ValidateSet('Preflight','Audit','Full')][string]$Mode,
  [Parameter(Mandatory)][string]$Target,
  [Parameter(Mandatory)][string]$Rig,
  [Parameter(Mandatory)][ValidateRange(1,65525)][int]$Port,
  [string]$PreflightCapture,
  [string]$AuditCapture,
  [string]$EvidenceRoot = 'D:/path-tracing-evidence',
  [string]$ClockEvidence = 'D:/path-tracing-wt/clock/lab/gates/results'
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot/../..").Path
$Target = (Resolve-Path -LiteralPath $Target).Path
$Rig = (Resolve-Path -LiteralPath $Rig).Path
$reference = Get-Content -LiteralPath "$Target/reference-server.txt"
if ((Join-Path $Rig 'server') -eq $reference) { throw 'Use a disposable copy of the reference server.' }
foreach ($entry in (Get-Content -LiteralPath "$Target/frozen-sha256.json" -Raw | ConvertFrom-Json)) {
  if ((Get-FileHash -LiteralPath $entry.Path).Hash -cne $entry.Hash) {
    throw "Frozen source or binary changed: $($entry.Path). Prepare a new evidence directory."
  }
}
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$wall = [Diagnostics.Stopwatch]::StartNew()
if ($Mode -eq 'Full') {
  if (-not $PreflightCapture -or -not $AuditCapture) { throw 'Full requires independently verified preflight and stock audit captures.' }
  & "$PSScriptRoot/verify.ps1" -Mode Preflight -Capture $PreflightCapture -Target $Target -ReferenceServer $reference |
    Tee-Object -FilePath "$Target/$stamp-prerequisites.log"
  & "$PSScriptRoot/verify.ps1" -Mode Audit -Capture $AuditCapture -Target $Target -ReferenceServer $reference |
    Tee-Object -FilePath "$Target/$stamp-prerequisites.log" -Append
  & "$PSScriptRoot/controls.ps1" -EvidenceRoot $EvidenceRoot -ClockEvidence $ClockEvidence -ReferenceServer $reference -Target $Target |
    Tee-Object -FilePath "$Target/$stamp-controls.log"
}

$oldRoot = $env:PATHTRACING_ROOT
$oldPort = $env:PATHTRACING_SERVER_PORT
try {
  $env:PATHTRACING_ROOT = $Rig
  $env:PATHTRACING_SERVER_PORT = [string]$Port
  $driver = "$root/lab/gates/record-fixtures.ps1"
  $output = @(& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $driver -CorpusMode $Mode `
    -CorpusTarget $Target -CorpusReferenceServer $reference -CorpusLauncher "$Target/client-source/scripts/launch-client.ps1" `
    -Jar "$Target/client-source/bot/build/pathtracing.jar" -ResourceProbeJar "$Target/probe/phase-probe.jar")
  $code = $LASTEXITCODE
} finally {
  $env:PATHTRACING_ROOT = $oldRoot
  $env:PATHTRACING_SERVER_PORT = $oldPort
}
$output
$report = $output | Where-Object { $_ -like '*report.md' } | Select-Object -Last 1
$summary = "$Target/$stamp-$Mode.log"
$output | Set-Content -LiteralPath $summary
if ($code -ne 0 -or -not $report) { throw "$Mode refused. See $summary and the capture report." }
$capture = Split-Path -Parent $report

function Distribution([string]$Label, [double[]]$Values) {
  $sorted = @($Values | Sort-Object)
  $n = $sorted.Count
  if ($n -eq 0) { throw "Missing $Label metrics." }
  $median = ($sorted[[Math]::Floor(($n - 1) / 2)] + $sorted[[Math]::Floor($n / 2)]) / 2
  $p90 = $sorted[[Math]::Ceiling($n * 0.90) - 1]
  $p99 = $sorted[[Math]::Ceiling($n * 0.99) - 1]
  '{0} n={1} min={2:F2} median={3:F2} p90={4:F2} p99={5:F2} max={6:F2}' -f $Label, $n, $sorted[0], $median, $p90, $p99, $sorted[-1]
}
$culture = [Globalization.CultureInfo]::InvariantCulture
$logs = @(Get-ChildItem -LiteralPath $capture -Filter '*-verify.stdout.txt')
$metrics = foreach ($log in $logs) {
  $body = Get-Content -LiteralPath $log.FullName -Raw
  $client = [regex]::Match($body, 'client TPS ([0-9.]+);')
  $offline = [regex]::Match($body, 'offline movement TPS ([0-9.]+);')
  if (-not $client.Success -or -not $offline.Success) { throw "Missing replay metrics in $($log.FullName)." }
  [pscustomobject]@{
    Client = [double]::Parse($client.Groups[1].Value, $culture)
    Offline = [double]::Parse($offline.Groups[1].Value, $culture)
    Witness = $log.Name
  }
}
. "$root/lab/gates/courses.ps1"
. "$PSScriptRoot/manifest.ps1"
$manifest = Get-TerrainManifest $Mode
if ($logs.Count -ne ($manifest.Runs * $manifest.Courses.Count)) { throw 'Capture and verifier counts disagree.' }
$verdict = @(
  (Distribution 'Client ticks per recorder wall second' $metrics.Client)
  (Distribution 'Rust movement ticks per wall second' $metrics.Offline)
  ('Gate wall seconds {0:F3}' -f $wall.Elapsed.TotalSeconds)
  "Slowest client replay: $(($metrics | Sort-Object Client | Select-Object -First 1).Witness)"
  "Slowest Rust replay: $(($metrics | Sort-Object Offline | Select-Object -First 1).Witness)"
  $(if ($Mode -eq 'Full') { 'L0 PASS. Full corpus, prerequisites and frozen movement controls passed.' }
    else { "$Mode verified. L0 still requires the full gate." })
)
$verdict | Tee-Object -FilePath $summary -Append
