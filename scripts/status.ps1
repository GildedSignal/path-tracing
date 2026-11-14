# Every lane on one screen: what runs, what the meters say, and which reports wait to be read.
#   .\scripts\status.ps1
. "$PSScriptRoot\common.ps1"
"== GPT (Codex exec jobs; four at most, one beside Astra)"
Get-CimInstance Win32_Process -Filter "Name='codex.exe'" | Where-Object { $_.CommandLine -match ' exec ' } | ForEach-Object {
  $m = [regex]::Match($_.CommandLine, '-m (\S+)').Groups[1].Value
  $c = [regex]::Match($_.CommandLine, '-C (\S+)').Groups[1].Value
  "  pid $($_.ProcessId) since $($_.CreationDate.ToString('HH:mm')) $m $c"
}
"== Antigravity (agy)"
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'gemini\.ps1' } | ForEach-Object {
  $j = [regex]::Match($_.CommandLine, '-Job (\S+)').Groups[1].Value
  $m = [regex]::Match($_.CommandLine, '-Model (\S+)').Groups[1].Value
  "  pid $($_.ProcessId) since $($_.CreationDate.ToString('HH:mm')) $j $m"
}
"== GLM"
$q = Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'glm-queue' }
"  queue runner: $(if ($q) { 'running' } else { 'NOT running' })"
Get-Content (Join-Path $Root 'lab\results\queue.log') -Tail 2 | ForEach-Object { "  $_" }
$key = Get-GlmKey
if ($key) {
  $env:ZAI_API_KEY = $key
  & D:\path-tracing-deps\omp\omp.exe usage -p zai 2>$null | Select-String '5 Hours' | ForEach-Object { "  " + ($_.Line -replace '[^\x20-\x7E]', '').Trim() }
  Remove-Item Env:ZAI_API_KEY
}
"== Terra queue"
Get-Content (Join-Path $Root 'jobs\queue.log') -Tail 2 -ErrorAction SilentlyContinue | ForEach-Object { "  $_" }
"== Reports newer than their last commit (read, judge, commit with Built-by:)"
Get-ChildItem (Join-Path $Root 'jobs') -Directory | ForEach-Object {
  $r = Join-Path $_.FullName 'last.md'
  if (Test-Path $r) {
    $committed = git -C $Root log -1 --format=%ct -- "jobs/$($_.Name)/last.md" 2>$null
    $t = [DateTimeOffset]::new((Get-Item $r).LastWriteTimeUtc).ToUnixTimeSeconds()
    if (-not $committed -or $t -gt [int64]$committed + 60) { "  $($_.Name)  ($((Get-Item $r).LastWriteTime.ToString('dd HH:mm')), $((Get-Item $r).Length) bytes)" }
  }
}
