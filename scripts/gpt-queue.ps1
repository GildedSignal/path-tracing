# Keeps Terra working down a list, unattended, so the build lane does not idle waiting for the integrator
# to notice it is free (S103: "terra is very often idle you should have a list for it").
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\gpt-queue.ps1 -Queue jobs\queue.txt
#
# The queue file holds one job per line: "<job-id> <worktree-path> [base-commit]". Lines starting with #
# are skipped. The worktree is created from the base commit (default `main`) if it does not exist yet, so a
# queued line needs nothing prepared by hand. A job that fails does not stop the queue.
#
# It runs up to -Slots jobs at once (four fresh first prompts, AGENTS.md; never more, and never two on the
# same files - the queue's order is the dependency order, so put a job after anything it needs).
# It does NOT commit: GPT cannot write git metadata, so the integrator reads jobs/<id>/last.md and commits
# with `Built-by:`. Finished lines go to jobs/queue.log with their exit state and a `needs commit` marker.
param(
  [Parameter(Mandatory)][string]$Queue,
  [int]$Slots = 4,
  [string]$Model = 'gpt-6.1-terra',
  [int]$PollSeconds = 60
)
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot\common.ps1"
$log = Join-Path $Root 'jobs\queue.log'
function Note($text) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $text" }
# Count our own launched wrappers, not codex.exe: a wrapper takes a while to spawn codex, and counting
# codex alone made the queue declare itself finished before the first job had started.
$script:launched = @()
function Running {
  $script:launched = @($script:launched | Where-Object { $_ -and -not $_.HasExited })
  $script:launched.Count
}

$lines = Get-Content (Join-Path $Root $Queue) | Where-Object { $_.Trim() -and -not $_.StartsWith('#') }
Note "queue started: $($lines.Count) jobs, $Slots slots, model $Model"
foreach ($line in $lines) {
  $parts = $line.Trim() -split '\s+'
  $job = $parts[0]; $wt = $parts[1]; $base = if ($parts.Count -gt 2) { $parts[2] } else { 'main' }
  if (-not (Test-Path (Join-Path $Root "jobs\$job\prompt.md"))) { Note "skip (no prompt.md): $job"; continue }

  # Wait for a free slot. Astra takes the whole lane (AGENTS: only one GPT job while Astra runs).
  $cap = if ($Model -match 'astra') { 1 } else { $Slots }
  while ((Running) -ge $cap) { Start-Sleep -Seconds $PollSeconds }

  if (-not (Test-Path $wt)) {
    & git -C $Root worktree add -b "wt/$job" $wt $base 2>&1 | Out-Null
    if (-not (Test-Path $wt)) { Note "skip (worktree failed): $job -> $wt"; continue }
  }
  Note "start: $job in $wt (from $base)"
  # Start-Process joins arguments with spaces and does not quote them, and this repo's path has a space,
  # so -File "...\path tracing\scripts\gpt.ps1" silently becomes two arguments and nothing runs. Quote
  # anything with whitespace, exactly as gpt.ps1 and gemini.ps1 already do.
  $wrapArgs = @('-NoProfile','-ExecutionPolicy','Bypass','-File',"$PSScriptRoot\gpt.ps1",
                '-Job',$job,'-Worktree',$wt,'-Model',$Model) |
              ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
  $script:launched += Start-Process powershell -ArgumentList $wrapArgs -WorkingDirectory $Root -WindowStyle Hidden -PassThru
  Start-Sleep -Seconds 10
}
# Drain, so the log records every job's end rather than stopping while jobs are still in flight.
while ((Running) -gt 0) { Start-Sleep -Seconds $PollSeconds }
foreach ($line in $lines) {
  $job = ($line.Trim() -split '\s+')[0]
  $rep = Join-Path $Root "jobs\$job\last.md"
  if (Test-Path $rep) { Note "done: $job ($((Get-Item $rep).Length) bytes) - needs commit with Built-by:" }
  else { Note "no report: $job" }
}
Note 'queue finished'
