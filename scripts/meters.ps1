# Every meter we can read, as % of its window (S104: per-companion 5-hour %, nothing more).
#   .\scripts\meters.ps1                 print them
#   .\scripts\meters.ps1 -WatchAt 80     poll every 5 minutes; EXIT as soon as one reaches 80%, naming it
# Run the watch mode in the background from the integrator's session: its exit is the alarm (S106).
#
# Zai (GLM): `omp usage -p zai`, with the stored key. Codex (Terra, Astra): the newest rollout under
# ~/.codex/sessions carries `rate_limits` on every token_count event, primary = 5 h, secondary = weekly.
# Antigravity has no readout until a meter is spent; then a run's result.json or stderr says
# RESOURCE_EXHAUSTED and names the reset, so a spent meter is reported as 100%.
param([int]$WatchAt = 0, [int]$PollMinutes = 5)
. "$PSScriptRoot\common.ps1"

function Read-Meters {
  $m = [ordered]@{}
  $key = Get-GlmKey
  if ($key) {
    $env:ZAI_API_KEY = $key
    $line = & D:\path-tracing-deps\omp\omp.exe usage -p zai 2>$null | Select-String '5 Hours' | Select-Object -First 1
    Remove-Item Env:ZAI_API_KEY
    $pct = [regex]::Match([string]$line, '([\d.]+)% used').Groups[1].Value
    if ($pct) { $m['zai 5h'] = [double]$pct }
  }
  $roll = Get-ChildItem "$env:USERPROFILE\.codex\sessions" -Recurse -File -Filter '*.jsonl' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
  if ($roll) {
    $last = Select-String -Path $roll.FullName -Pattern '"rate_limits"' | Select-Object -Last 1
    if ($last) {
      $rl = ($last.Line | ConvertFrom-Json).payload.rate_limits
      if ($rl.primary) { $m['codex 5h'] = [double]$rl.primary.used_percent }
      if ($rl.secondary) { $m['codex week'] = [double]$rl.secondary.used_percent }
    }
  }
  # A 429 in the last five hours means that Antigravity meter is spent until the reset it names.
  $since = (Get-Date).AddHours(-5)
  Get-ChildItem (Join-Path $Root 'jobs') -Recurse -File -Include result.json,stderr.txt -ErrorAction SilentlyContinue |
    Where-Object { $_.LastWriteTime -gt $since } | ForEach-Object {
      $hit = Select-String -Path $_.FullName -Pattern 'RESOURCE_EXHAUSTED' -List
      if ($hit) {
        # Which meter is read from the model the job was launched with (jobs/launch.log), never from the
        # error text, which does not name it: guessing from the text called a spent Claude meter Gemini.
        $job = Split-Path -Leaf $_.DirectoryName
        $launched = Select-String -Path (Join-Path $Root 'jobs\launch.log') -Pattern " $([regex]::Escape($job)) " -ErrorAction SilentlyContinue | Select-Object -Last 1
        $meter = if ([string]$launched -match 'claude-') { 'agy claude' } elseif ([string]$launched -match 'gemini-') { 'agy gemini' } else { 'agy unknown' }
        $m[$meter] = 100
      }
    }
  $m
}

if (-not $WatchAt) { (Read-Meters).GetEnumerator() | ForEach-Object { '{0,-12} {1,5:N1}%' -f $_.Key, $_.Value }; return }
# Alarm on a crossing: a meter already over the line when the watch starts (a spent weekly meter) is
# reported once here and then ignored, or the watch would exit at once forever.
$already = @((Read-Meters).GetEnumerator() | Where-Object { $_.Value -ge $WatchAt } | ForEach-Object { $_.Key })
if ($already) { "already at $WatchAt%+ when the watch began (ignored): $($already -join ', ')" }
while ($true) {
  $m = Read-Meters
  $over = @($m.GetEnumerator() | Where-Object { $_.Value -ge $WatchAt -and $_.Key -notin $already })
  if ($over) {
    "$(Get-Date -Format HH:mm) METER AT $WatchAt%+: " + (($over | ForEach-Object { "$($_.Key) $($_.Value)%" }) -join ', ')
    $m.GetEnumerator() | ForEach-Object { '{0,-12} {1,5:N1}%' -f $_.Key, $_.Value }
    exit 0
  }
  Start-Sleep -Seconds ($PollMinutes * 60)
}
