# Runs GLM lens batches one after another, unattended, so the lab keeps working while Claude is out of usage (S100).
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-queue.ps1 -Queue lab\queue.txt
# The queue file holds one batch per line: lens names separated by spaces (two per line is what one
# 5-hour window sustains, S70). Lines starting with # are skipped. A finished line is appended to
# lab\results\queue.log with its exit state; a batch that fails does not stop the queue.
param([Parameter(Mandatory)][string]$Queue, [int]$Minutes = 60, [int]$PauseMinutes = 10)
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot\common.ps1"
$log = Join-Path $Root 'lab\results\queue.log'
function Note($text) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $text" }
# The file is re-read before every batch, so a line appended while the queue runs is picked up (S104).
$started = @()
Note "queue started from $Queue"
while ($true) {
  $batch = Get-Content (Join-Path $Root $Queue) | Where-Object { $_.Trim() -and -not $_.StartsWith('#') -and $_ -notin $started } | Select-Object -First 1
  if (-not $batch) { break }
  $started += $batch
  # One GLM run at a time: wait for any run already going (omp is its harness).
  while (Get-Process omp -ErrorAction SilentlyContinue) { Start-Sleep -Seconds 60 }
  # Two runs started in the same minute would share a run id.
  Start-Sleep -Seconds 61
  Note "start: $batch"
  & powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\glm-local.ps1" -Lenses $batch -Minutes $Minutes *> (Join-Path $Root 'lab\results\queue-last.txt')
  Note "end ($LASTEXITCODE): $batch"
  Start-Sleep -Seconds ($PauseMinutes * 60)
}
Note 'queue finished'
