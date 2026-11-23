# PreToolUse and PostToolUse hook on shell commands (the overhaul's C3 and C4).
#   pre:  a launch of a big model (astra or sol) is refused unless
#         jobs/<id>/shown.txt exists: Julien has seen the prompt in chat (S230).
#   post: every launch arms a watcher that exits 2 when the job's process ends, which wakes the model.
param([ValidateSet('pre','post')][string]$Phase)
try { $in = [Console]::In.ReadToEnd() | ConvertFrom-Json } catch { exit 0 }
$cmd = [string]$in.tool_input.command
# Fire only on an actual invocation of the launch script: a command segment (split on ; && || | and
# newlines) whose program is scripts/launch.ps1 itself, bare, after `&`, or as `powershell -File <it>`.
# A segment that merely names the script (sed, grep, cat, git add ...) is not a launch.
$job = $null; $who = $null; $dry = $false
foreach ($seg in ($cmd -split '\r?\n|;|&&|\|\||\|')) {
  $t = @([regex]::Matches($seg.Trim(), '"[^"]*"|''[^'']*''|\S+') | ForEach-Object { $_.Value.Trim('"', "'") })
  if ($t.Count -eq 0) { continue }
  $i = 0
  if ($t[0] -eq '&') { $i = 1 }
  if ($i -lt $t.Count -and $t[$i] -match '^(powershell|pwsh)(\.exe)?$') {
    $f = [array]::IndexOf(@($t | ForEach-Object { $_.ToLower() }), '-file')
    if ($f -lt 0) { continue }
    $i = $f + 1
  }
  if ($i -ge $t.Count -or $t[$i] -notmatch '(^|[\\/])launch\.ps1$') { continue }
  # A bare `launch.ps1` at the start of a line (a commit message body) is prose; a launch has a path.
  if ($t[$i] -notmatch '^(\.[\\/]|.*scripts[\\/])launch\.ps1$') { continue }
  if ($t.Count -lt $i + 3) { continue }
  $job = $t[$i + 1]; $who = $t[$i + 2]; $dry = $t -contains '-DryRun'
  break
}
if (-not $job -or $dry) { exit 0 }
$root = Split-Path (Split-Path $PSScriptRoot)
if ($Phase -eq 'pre') {
  # Every prompt opens with its title, then the goal in one line, so he can review it at a glance.
  $lines = @(Get-Content -Encoding UTF8 (Join-Path $root "jobs\$job\prompt.md") -ErrorAction SilentlyContinue | Where-Object { $_.Trim() })
  if ($lines.Count -lt 2 -or $lines[0] -notmatch '^# ' -or $lines[1] -notmatch '^\*\*Goal:\*\* \S') {
    [Console]::Error.WriteLine("Refused: jobs\$job\prompt.md must open with '# <title>' and then one line '**Goal:** <the goal>'.")
    exit 2
  }
  $big = $who -in 'astra','sol'
  if ($big -and -not (Test-Path (Join-Path $root "jobs\$job\shown.txt"))) {
    [Console]::Error.WriteLine("Refused: $who is a big-model launch and jobs\$job\shown.txt does not exist. Show Julien the full prompt in chat first (S230), then create shown.txt.")
    exit 2
  }
  exit 0
}
# post: wait while a runner for this job is alive, then wake the model.
if (($in.tool_response | ConvertTo-Json -Depth 4 -Compress) -notmatch "launched $([regex]::Escape($job)) on") { exit 0 }   # only a real launch arms a watcher
Start-Sleep 30
while (Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match "gpt\.ps1.*-Job $([regex]::Escape($job))(\s|$)" }) { Start-Sleep 60 }
[Console]::Error.WriteLine("Job $job ($who) has ended at $(Get-Date -Format HH:mm). Read jobs\$job\last.md and judge it.")
exit 2
