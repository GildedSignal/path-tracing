# GLM never idles while it has port work (S241): runs a queue file top to bottom as a rolling worker pool.
# A finished job leaves the queue and immediately frees its slot; a job that hits Zai's 5-hour limit (1308)
# pauses new spawns until the reset the error names. Results land in lab\results\local-*.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\glm-chain.ps1 [-Parallel 7] [-QueueFile jobs\glm-chain.txt] [-Minutes 60] [-Model zai/glm-5.4] [-LogFile jobs\glm-chain.log]
param([int]$Parallel = 7, [string]$QueueFile = 'jobs\glm-chain.txt', [int]$Minutes = 60, [string]$Model = 'zai/glm-5.4', [string]$LogFile = 'jobs\glm-chain.log')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$list = if ([IO.Path]::IsPathRooted($QueueFile)) { $QueueFile } else { Join-Path $Root $QueueFile }
$log = if ([IO.Path]::IsPathRooted($LogFile)) { $LogFile } else { Join-Path $Root $LogFile }
$resultsDir = Join-Path $Root 'lab\results'
$glmLocal = Join-Path $PSScriptRoot 'glm-local.ps1'
$utf8NoBom = [Text.UTF8Encoding]::new($false)
function Write-Log($s) {
  for ($a = 0; $a -lt 5; $a++) {
    try { [IO.File]::AppendAllText($log, "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $s`r`n", $utf8NoBom); return } catch { Start-Sleep -Milliseconds 100 }
  }
}
function Read-Queue {
  if (-not (Test-Path $list)) { return @() }
  return @(Get-Content $list -ErrorAction SilentlyContinue | ForEach-Object { $_.Trim() } | Where-Object { $_ })
}

$active = @()
$failed = [System.Collections.Generic.HashSet[string]]::new()
$resumeAt = [datetime]::MinValue

while ($true) {
  $stillRunning = @()
  foreach ($slot in $active) {
    $slot.Proc.Refresh()
    if (-not $slot.Proc.HasExited) {
      $stillRunning += $slot
      continue
    }
    $job = $slot.Job
    $pat = "^local-.*-0-$([regex]::Escape($job))(-zai-limit)?$"
    $out = Get-ChildItem $resultsDir -Directory -Filter "local-*-$job*" -ErrorAction SilentlyContinue |
      Where-Object { $_.Name -match $pat -and $_.LastWriteTime -ge $slot.Start.AddSeconds(-5) } |
      Sort-Object LastWriteTime | Select-Object -Last 1
    $authorLog = if ($out) { Get-Content -Raw (Join-Path $out.FullName 'author.log') -ErrorAction SilentlyContinue } else { '' }
    if ($out -and ($out.Name -like '*-zai-limit' -or $authorLog -match '"code":"1308"|Usage limit reached|rate[_ -]?limit|HTTP 429|status 429|Too Many Requests')) {
      $m = [regex]::Match($authorLog, 'retry-after-ms=(\d+)')
      $wait = if ($m.Success) { [int]([long]$m.Groups[1].Value / 1000) + 60 } else { 1800 }
      $candidate = (Get-Date).AddSeconds($wait)
      if ($candidate -gt $resumeAt) { $resumeAt = $candidate }
      Write-Log "limit ($Model): $job -> $($out.Name); cooldown until $($resumeAt.ToString('HH:mm'))"
      continue
    }
    $dossierPath = if ($out) { Join-Path $out.FullName 'dossier.md' } else { '' }
    $firstLine = if ($dossierPath -and (Test-Path $dossierPath)) { (Get-Content $dossierPath -TotalCount 1 -ErrorAction SilentlyContinue) } else { '' }
    if ($out -and (Test-Path $dossierPath) -and ((Get-Item $dossierPath).Length -gt 200) -and ($firstLine -ne 'Claim: The author produced no dossier.')) {
      Write-Log "end: $job -> $($out.Name)"
      $current = @(Read-Queue | Where-Object { $_ -ne $job })
      [IO.File]::WriteAllLines($list, [string[]]$current, $utf8NoBom)
    } else {
      $outName = if ($out) { $out.Name } else { 'none' }
      Write-Log "failed: $job -> $outName (exit $($slot.Proc.ExitCode))"
      [void]$failed.Add($job)
    }
  }
  $active = $stillRunning

  $jobs = @(Read-Queue)
  if ($active.Count -eq 0 -and $jobs.Count -eq 0) {
    Write-Log "list empty ($QueueFile): done"
    break
  }
  $activeNames = @($active | ForEach-Object { $_.Job })
  $runnable = @($jobs | Where-Object { $_ -notin $activeNames -and -not $failed.Contains($_) })
  if ($active.Count -eq 0 -and $runnable.Count -eq 0) {
    Write-Log "chain stopped: no runnable jobs left in $QueueFile"
    break
  }

  if ((Get-Date) -ge $resumeAt) {
    while ($active.Count -lt $Parallel -and $runnable.Count -gt 0) {
      $job = $runnable[0]
      $runnable = @($runnable | Select-Object -Skip 1)
      $start = Get-Date
      try {
        $proc = Start-Process powershell -ArgumentList @(
          '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'scripts\glm-local.ps1', '-Lenses', $job, '-Minutes', "$Minutes", '-Model', $Model
        ) -WorkingDirectory $Root -WindowStyle Hidden -PassThru
        $active += [pscustomobject]@{ Job = $job; Proc = $proc; Start = $start }
        Write-Log "start ($Model): $job (active=$($active.Count)/$Parallel)"
      } catch {
        Write-Log "spawn error on ${job}: $_"
        [void]$failed.Add($job)
      }
      if ($active.Count -lt $Parallel -and $runnable.Count -gt 0) { Start-Sleep -Seconds 2 }
    }
  }

  Start-Sleep -Seconds 5
}
