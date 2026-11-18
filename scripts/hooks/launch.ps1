# PreToolUse and PostToolUse hook on shell commands (the overhaul's C3 and C4).
#   pre:  a launch of a big model (astra, sol, opus, or argon with -Commands) is refused unless
#         jobs/<id>/shown.txt exists: Julien has seen the prompt in chat (S230).
#   post: every launch arms a watcher that exits 2 when the job's process ends, which wakes the model.
param([ValidateSet('pre','post')][string]$Phase)
try { $in = [Console]::In.ReadToEnd() | ConvertFrom-Json } catch { exit 0 }
$cmd = [string]$in.tool_input.command
$m = [regex]::Match($cmd, 'launch\.ps1\s+(\S+)\s+(\S+)')
if (-not $m.Success -or $cmd -match '-DryRun') { exit 0 }
$job = $m.Groups[1].Value; $who = $m.Groups[2].Value
$root = Split-Path (Split-Path $PSScriptRoot)
if ($Phase -eq 'pre') {
  $big = $who -in 'astra','sol','opus' -or ($who -eq 'argon' -and $cmd -match '-Commands')
  if ($big -and -not (Test-Path (Join-Path $root "jobs\$job\shown.txt"))) {
    [Console]::Error.WriteLine("Refused: $who is a big-model launch and jobs\$job\shown.txt does not exist. Show Julien the full prompt in chat first (S230), then create shown.txt.")
    exit 2
  }
  exit 0
}
# post: wait while a runner for this job is alive, then wake the model.
if (($in.tool_response | ConvertTo-Json -Depth 4 -Compress) -notmatch "launched $([regex]::Escape($job)) on") { exit 0 }   # only a real launch arms a watcher
Start-Sleep 30
while (Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match "(gpt|gemini)\.ps1.*-Job $([regex]::Escape($job))(\s|$)" }) { Start-Sleep 60 }
[Console]::Error.WriteLine("Job $job ($who) has ended at $(Get-Date -Format HH:mm). Read jobs\$job\last.md and judge it.")
exit 2
