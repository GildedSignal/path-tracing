# Runs the GPT jobs listed in jobs\queue.txt, one at a time, never with more than two GPT jobs running at once.
# Each line: <job id> <worktree path>. Lines starting with # are skipped.
# A job counts as done when jobs\<id>\last.md exists; done jobs are skipped, so running this again after a
# shutdown resumes with the first unfinished job (its worktree keeps any half-done changes).
# After each job, its worktree is committed on its own branch (never main), for the integrator to judge and merge.
#   Double-click "Run queue.cmd"; "Stop GPT.cmd" stops every GPT job.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$queue = Join-Path $Root 'jobs\queue.txt'

function Count-Gpt { @(Get-CimInstance Win32_Process -Filter "Name = 'codex.exe'" -ErrorAction SilentlyContinue |
  Where-Object { $_.CommandLine -match '\bexec\b' }).Count }

foreach ($line in Get-Content $queue) {
  $line = $line.Trim()
  if (-not $line -or $line.StartsWith('#')) { continue }
  $job, $worktree = $line -split '\s+', 2
  $last = Join-Path $Root "jobs\$job\last.md"
  if (Test-Path $last) { Write-Host "Done already: $job"; continue }
  while ((Count-Gpt) -ge 2) { Write-Host "Two GPT jobs are running; waiting..."; Start-Sleep -Seconds 60 }
  Write-Host "$(Get-Date -Format HH:mm) Starting $job in $worktree"
  & "$PSScriptRoot\gpt.ps1" -Job $job -Worktree $worktree
  if (Test-Path $last) {
    git -C $worktree add -A
    git -C $worktree commit -q -m "Job ${job} (queued run; not yet judged)`n`nBuilt-by: gpt-6.1-sol"
    Write-Host "$(Get-Date -Format HH:mm) Finished $job; committed on its branch."
  } else {
    Write-Host "$job stopped without a final report; run the queue again to retry it."
    break
  }
}
Write-Host 'Queue finished. You can close this window.'
