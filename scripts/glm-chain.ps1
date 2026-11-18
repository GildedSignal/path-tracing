# GLM never idles while it has port work (S241): runs jobs\glm-chain.txt top to bottom, one job at a time.
# A job that hits Zai's 5-hour limit (1308) is retried after the reset the error names; a finished job
# leaves the list. Results land in lab\results\local-*. Start it detached; close its window to stop it.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-chain.ps1
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$list = Join-Path $Root 'jobs\glm-chain.txt'
$log = Join-Path $Root 'jobs\glm-chain.log'
function Write-Log($s) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $s" }
while ($true) {
  $jobs = @(Get-Content $list | Where-Object { $_.Trim() })
  if (-not $jobs) { Write-Log 'list empty: done'; break }
  $job = $jobs[0]
  Write-Log "start: $job"
  try { & "$PSScriptRoot\glm-local.ps1" -Lenses $job -Minutes 60 } catch { Write-Log "run error: $_" }
  $out = Get-ChildItem (Join-Path $Root 'lab\results') -Directory -Filter "local-*-$job*" | Sort-Object LastWriteTime | Select-Object -Last 1
  if ($out -and $out.Name -like '*-zai-limit') {
    $ms = [regex]::Match((Get-Content -Raw (Join-Path $out.FullName 'author.log') -ErrorAction SilentlyContinue), 'retry-after-ms=(\d+)').Groups[1].Value
    $wait = if ($ms) { [int]([long]$ms / 1000) + 60 } else { 1800 }
    Write-Log "zai limit on ${job}: retry in $([int]($wait / 60)) min"
    Start-Sleep $wait
    continue
  }
  # A job leaves the list only when its run finished with a dossier; a broken run stops the chain to be fixed.
  if (-not $out -or -not (Test-Path (Join-Path $out.FullName 'dossier.md'))) { Write-Log "failed: $job -> $($out.Name); chain stopped"; break }
  Write-Log "end: $job -> $($out.Name)"
  Set-Content -Encoding utf8 $list ($jobs | Select-Object -Skip 1)
}
