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
  # Newest record whose limit_id is "codex" and carries numbers: Astra's sessions log a separate "premium"
  # record with null windows, and the newest file may be one of those (14 November).
  $rolls = Get-ChildItem "$env:USERPROFILE\.codex\sessions" -Recurse -File -Filter '*.jsonl' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 20
  foreach ($roll in $rolls) {
    $last = Select-String -Path $roll.FullName -Pattern '"limit_id":"codex"' | Select-Object -Last 1
    if ($last) {
      $rl = ($last.Line | ConvertFrom-Json).payload.rate_limits
      if ($rl.primary) { $m['codex 5h'] = [double]$rl.primary.used_percent; $m['codex 5h resets'] = [DateTimeOffset]::FromUnixTimeSeconds($rl.primary.resets_at).LocalDateTime.ToString('dd HH:mm') }
      if ($rl.secondary) { $m['codex week'] = [double]$rl.secondary.used_percent }
      $m['codex as of'] = $roll.LastWriteTime.ToString('HH:mm')
      break
    }
  }
  # Antigravity: its own `/usage` slash command, run headless on stdin, costs no model call and prints
  # "<group>\t<label>\t<N>%\t<reset>" with N the share REMAINING (found 14 November, S112; the earlier
  # "unreadable until spent" was never tested). Claude and GPT models share one meter; Argon has its own.
  $agy = Join-Path $env:LOCALAPPDATA 'agy\bin\agy.exe'
  if (Test-Path $agy) {
    $in = [IO.Path]::GetTempFileName(); $out = [IO.Path]::GetTempFileName()
    Set-Content -LiteralPath $in '/usage' -Encoding ascii
    $p = Start-Process $agy -ArgumentList @('--output-format','text','--print-timeout','60s') -RedirectStandardInput $in `
      -RedirectStandardOutput $out -RedirectStandardError ([IO.Path]::GetTempFileName()) -NoNewWindow -Wait -PassThru
    foreach ($line in Get-Content -LiteralPath $out) {
      $f = $line -split "`t"
      if ($f.Count -ge 3 -and $f[2] -match '^(\d+)%$') {
        $used = 100 - [double]$Matches[1]   # read before any other -match overwrites $Matches
        $group = if ($f[0] -match 'Argon') { 'agy argon' } elseif ($f[0] -match 'Claude') { 'agy claude+gpt' } elseif ($f[0] -match 'Gemini') { 'agy gemini' } else { "agy $($f[0])" }
        $window = if ($f[1] -match 'Five Hour') { '5h' } else { 'week' }
        $m["$group $window"] = $used
      }
    }
    Remove-Item -LiteralPath $in, $out -ErrorAction SilentlyContinue
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
