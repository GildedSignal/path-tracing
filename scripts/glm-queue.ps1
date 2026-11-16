# Keeps GLM lanes full, unattended, so the lab works while Claude is away (S100).
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-queue.ps1 -Queue lab\queue.txt [-Lanes 3]
# The queue file holds lens names, one or several per line; lines starting with # are skipped. Each lens is
# its own lane: up to -Lanes run at once, and a new one starts as soon as one ends (S104: the old runner
# ran one two-lens batch at a time, waited for the slower lane and paused ten minutes, so GLM averaged
# ~1.5 lanes while its 5-hour window holds ~3 in parallel). The file is re-read before every start, so a
# lens appended while the queue runs is picked up. A lens already started by this runner, or named in a
# "start:" line of the log since the queue began, is not started twice. Lanes run at Normal priority (S158).
# A lens must be COMMITTED before it starts: lanes run from a checkout of HEAD.
param([Parameter(Mandatory)][string]$Queue, [int]$Lanes = 3, [int]$MaxLanes = 8, [int]$Minutes = 60)
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot\common.ps1"
$log = Join-Path $Root 'lab\results\queue.log'
function Note($text) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $text" }
$began = Get-Date
$started = @{}
$running = @()
# Restart-safe (S123, "glm is not used enough"): a lens named in a "start:" line since the last runner
# began is never started twice, and lanes are counted from the running glm-local processes, so lanes a
# previous runner left running count against -Lanes.
$previousLog = @(Get-Content $log -ErrorAction SilentlyContinue)
$from = [Array]::FindLastIndex([string[]]$previousLog, [Predicate[string]]{ param($l) $l -match 'queue started' })
if ($from -ge 0) { $previousLog[$from..($previousLog.Count - 1)] | ForEach-Object { if ($_ -match 'start: (\S+)$') { $started[$Matches[1]] = $true } } }
# Paced to the meter (S123): Zai's 5-hour window should be spent by its reset, not before. A lane above
# -Lanes may start while the share used is below the share of the window elapsed (minus one lane's ~10%
# of headroom); at or above the line no lane starts, so running lanes drain. -MaxLanes caps it.
function Get-ZaiPace {
  $key = Get-GlmKey; if (-not $key) { return $null }
  $env:ZAI_API_KEY = $key
  $line = [string](& D:\path-tracing-deps\omp\omp.exe usage -p zai 2>$null | Select-String '5 Hours' | Select-Object -First 1)
  Remove-Item Env:ZAI_API_KEY
  $used = [regex]::Match($line, '([\d.]+)% used').Groups[1].Value
  $r = [regex]::Match($line, 'resets in (?:(\d+)h)?(?:(\d+)m)?')
  if (-not $used -or -not $r.Success) { return $null }
  $left = [int]('0' + $r.Groups[1].Value) * 60 + [int]('0' + $r.Groups[2].Value)
  [pscustomobject]@{ Used = [double]$used; Elapsed = 100 * (300 - $left) / 300 }
}
function Get-Lanes { @(Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'glm-local\.ps1' }).Count }
Note "queue started from $Queue, $Lanes lanes ($($started.Count) lenses already started by the previous runner)"
while ($true) {
  $running = @($running | Where-Object { -not $_.Process.HasExited })
  foreach ($done in @($script:previous | Where-Object { $_ -and $_.Process.HasExited })) { Note "end ($($done.Process.ExitCode)): $($done.Lens)" }
  $script:previous = $running
  $lanesNow = Get-Lanes
  $room = $lanesNow -lt $Lanes
  if (-not $room -and $lanesNow -lt $MaxLanes) {
    $pace = Get-ZaiPace
    $room = $pace -and ($pace.Used + 10 -lt $pace.Elapsed)
  }
  if ($room) {
    $lenses = Get-Content (Join-Path $Root $Queue) | Where-Object { $_.Trim() -and -not $_.StartsWith('#') } | ForEach-Object { $_.Trim() -split '\s+' }
    $next = $lenses | Where-Object { -not $started.ContainsKey($_) } | Select-Object -First 1
    if ($next) {
      if (-not (git -C $Root ls-files "lab/lenses/$next.md")) { Note "skip (lens not committed): $next"; $started[$next] = $true; continue }
      # A lens with a finished result folder never runs again, whatever the log says: on 15 November a
      # restart re-ran port-freeze-step-fluid, finished the day before, because the queue was not pruned.
      if (Get-ChildItem (Join-Path $Root 'lab\results') -Directory | Where-Object { $_.Name -match "^local-\d{8}-\d{4}-\d+-\d+-$([regex]::Escape($next))$" -and (Test-Path (Join-Path $_.FullName 'DONE')) }) { Note "skip (already done): $next"; $started[$next] = $true; continue }
      $started[$next] = $true
      $q = @('-NoProfile','-ExecutionPolicy','Bypass','-File',"$PSScriptRoot\glm-local.ps1",'-Lenses',$next,'-Minutes',"$Minutes") |
           ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
      $p = Start-Process powershell -ArgumentList $q -WorkingDirectory $Root -WindowStyle Hidden -PassThru
      # Normal priority (S158); before a live gate, lower the lanes to Idle (S106).
      $running += [pscustomobject]@{ Lens = $next; Process = $p }
      $script:previous = $running
      Note "start: $next"
      Start-Sleep -Seconds 61   # two runs started in the same minute would share a run id
      continue
    }
    if ((Get-Lanes) -eq 0) { break }
  }
  Start-Sleep -Seconds 60
}
Note 'queue finished'
