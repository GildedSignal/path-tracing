# Sends a follow-up message to a GPT job's session as soon as its current run ends (S121): his new turns
# reach the session that already holds the job's context, instead of a fresh run that re-reads everything.
#   Start-Process powershell -ArgumentList '-File','scripts\follow-up.ps1','-Job','3.2d-3-design','-Worktree','D:\path-tracing-wt\sets','-Message','jobs\3.2d-3-design\addendum.md' -WindowStyle Hidden
# Polls every 10 s, so the slot it frees is taken back before any retry loop (launch.ps1 every 5 min) sees it.
param(
  [Parameter(Mandatory)][string]$Job,
  [Parameter(Mandatory)][string]$Worktree,
  [Parameter(Mandatory)][string]$Message,
  [string]$Model = 'gpt-6-astra'
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$dir = Join-Path $Root "jobs\$Job"
# Wait for the job's own run and for every follow-up queued before this one: a follow-up runs gpt.ps1
# inside its own process, so matching gpt.ps1 alone let two resumes of one session run at once
# (15 November, 12:16 and 12:23: the second message was lost). Older follow-ups go first.
$pattern = "(gpt|follow-up)\.ps1.*-Job $([regex]::Escape($Job))\b"
$me = (Get-CimInstance Win32_Process -Filter "ProcessId=$PID").CreationDate
while (Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match $pattern -and $_.ProcessId -ne $PID -and $_.CreationDate -lt $me }) { Start-Sleep 10 }
$line = Get-Content (Join-Path $dir 'log.jsonl') -TotalCount 1
if ($line -notmatch '"thread_id":"([^"]+)"') { throw "No session id in $dir\log.jsonl" }
$session = $Matches[1]
$stamp = Get-Date -Format 'HHmm'
Copy-Item (Join-Path $dir 'last.md') (Join-Path $dir "last-before-follow-$stamp.md") -Force -ErrorAction SilentlyContinue
Copy-Item (Join-Path $dir 'log.jsonl') (Join-Path $dir "log-before-follow-$stamp.jsonl") -Force -ErrorAction SilentlyContinue
Add-Content -Encoding utf8 (Join-Path $Root 'jobs\launch.log') "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $Job follow-up ($Message) resumes session $session"
try { (Get-Process -Id $PID).PriorityClass = 'Idle' } catch {}
& "$PSScriptRoot\gpt.ps1" -Job $Job -Worktree $Worktree -Model $Model -Resume $session -Follow $(if ([IO.Path]::IsPathRooted($Message)) { $Message } else { Join-Path $Root $Message })
