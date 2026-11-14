# Keeps GLM lanes full, unattended, so the lab works while Claude is away (S100).
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-queue.ps1 -Queue lab\queue.txt [-Lanes 3]
# The queue file holds lens names, one or several per line; lines starting with # are skipped. Each lens is
# its own lane: up to -Lanes run at once, and a new one starts as soon as one ends (S104: the old runner
# ran one two-lens batch at a time, waited for the slower lane and paused ten minutes, so GLM averaged
# ~1.5 lanes while its 5-hour window holds ~3 in parallel). The file is re-read before every start, so a
# lens appended while the queue runs is picked up. A lens already started by this runner, or named in a
# "start:" line of the log since the queue began, is not started twice. Lanes run at Idle CPU priority.
# A lens must be COMMITTED before it starts: lanes run from a checkout of HEAD.
param([Parameter(Mandatory)][string]$Queue, [int]$Lanes = 3, [int]$Minutes = 60)
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot\common.ps1"
$log = Join-Path $Root 'lab\results\queue.log'
function Note($text) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $text" }
$began = Get-Date
$started = @{}
$running = @()
Note "queue started from $Queue, $Lanes lanes"
while ($true) {
  $running = @($running | Where-Object { -not $_.Process.HasExited })
  foreach ($done in @($script:previous | Where-Object { $_ -and $_.Process.HasExited })) { Note "end ($($done.Process.ExitCode)): $($done.Lens)" }
  $script:previous = $running
  if ($running.Count -lt $Lanes) {
    $lenses = Get-Content (Join-Path $Root $Queue) | Where-Object { $_.Trim() -and -not $_.StartsWith('#') } | ForEach-Object { $_.Trim() -split '\s+' }
    $next = $lenses | Where-Object { -not $started.ContainsKey($_) } | Select-Object -First 1
    if ($next) {
      if (-not (git -C $Root ls-files "lab/lenses/$next.md")) { Note "skip (lens not committed): $next"; $started[$next] = $true; continue }
      $started[$next] = $true
      $q = @('-NoProfile','-ExecutionPolicy','Bypass','-File',"$PSScriptRoot\glm-local.ps1",'-Lenses',$next,'-Minutes',"$Minutes") |
           ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
      $p = Start-Process powershell -ArgumentList $q -WorkingDirectory $Root -WindowStyle Hidden -PassThru
      try { $p.PriorityClass = 'Idle' } catch {}
      $running += [pscustomobject]@{ Lens = $next; Process = $p }
      $script:previous = $running
      Note "start: $next"
      Start-Sleep -Seconds 61   # two runs started in the same minute would share a run id
      continue
    }
    if ($running.Count -eq 0) { break }
  }
  Start-Sleep -Seconds 60
}
Note 'queue finished'
