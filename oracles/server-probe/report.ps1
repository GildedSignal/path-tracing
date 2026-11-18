param([string]$Evidence = 'D:\path-tracing-evidence\server-probe-20261118')
$ErrorActionPreference = 'Stop'

function Percentile($Values, [double]$P) {
  $ordered = @($Values | Sort-Object)
  if ($P -le 0) { return $ordered[0] }
  $ordered[[Math]::Min($ordered.Count - 1, [int][Math]::Ceiling($P * $ordered.Count) - 1)]
}

$matrix = Get-Content "$Evidence\matrix.json" -Raw | ConvertFrom-Json
$report = [Collections.Generic.List[string]]::new()
$report.Add('| Mode | Layout | N | Target TPS | ms/tick min / median / p90 / p99 / max | Sustained TPS | Heap med / max MiB | Resident MiB | Chunks | Teleports | Moved wrongly |')
$report.Add('|---|---|---:|---:|---|---:|---|---:|---:|---:|---:|')

foreach ($row in $matrix) {
  foreach ($rate in @(20, 100)) {
    $caseDir = Join-Path $Evidence $row.case
    $file = Join-Path $caseDir "$rate.csv"
    if (-not (Test-Path $file)) {
      $report.Add("| $($row.mode) | $($row.layout) | $($row.n) | $rate | refused: $($row.reason) | n/a | n/a | n/a | n/a | n/a | $($row.movedWrongly) |")
      continue
    }
    $data = @(Import-Csv $file)
    if ($data.Count -ne 600) { throw "Expected 600 samples in $file, got $($data.Count)" }
    $ns = @($data | ForEach-Object { [double]$_.ns / 1e6 })
    $mem = @($data | ForEach-Object { [double]$_.heap_bytes / 1MB })
    $dist = @(0, 0.5, 0.9, 0.99, 1 | ForEach-Object { [string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:F3}', (Percentile $ns $_)) }) -join ' / '
    $tps = 599e9 / ([double]$data[-1].wall_ns - [double]$data[0].wall_ns)
    $tp = if ($rate -eq 20) { $row.teleports20 } else { $row.teleports100 }
    $chunks = if ($rate -eq 20) { $row.loadedChunks20 } else { $row.loadedChunks100 }
    $report.Add(([string]::Format([Globalization.CultureInfo]::InvariantCulture,
      '| {0} | {1} | {2} | {3} | {4} | {5:F2} | {6:F0} / {7:F0} | {8:F0} | {9} | {10} | {11} |',
      $row.mode, $row.layout, $row.n, $rate, $dist, $tps,
      (Percentile $mem 0.5), (Percentile $mem 1), ($row.workingSetBytes / 1MB),
      $chunks, $tp, $row.movedWrongly
    )))
  }
}

$report | Set-Content "$Evidence\table.md" -Encoding UTF8
$report
