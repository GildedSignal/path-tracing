# Overnight chain (17 November): Hydrogen writes port lenses -> commit and queue them -> GLM runs them -> Hydrogen digests.
# Started detached by the integrator; every step is logged to jobs\port-lenses\chain.log.
$ErrorActionPreference = 'Continue'
$Root = Split-Path (Split-Path $PSScriptRoot)
Set-Location $Root
$log = Join-Path $PSScriptRoot 'chain.log'
function Note($t) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $t" }

# 1. Wait for Hydrogen's port-lenses run to end.
while (Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'gemini\.ps1.*-Job port-lenses\b' }) { Start-Sleep 30 }
$report = Join-Path $PSScriptRoot 'report.md'
if (-not (Test-Path $report)) { Note 'no report.md: stop'; exit 1 }
$line = Select-String $report -Pattern '^queue:\s*(.+)$' | Select-Object -Last 1
if (-not $line) { Note 'no queue: line in report.md: stop'; exit 1 }
$names = $line.Matches[0].Groups[1].Value -split '\s+' | Where-Object { $_ -match '^[a-z0-9-]+$' }
$copied = @()
foreach ($n in $names) {
  $src = Join-Path $PSScriptRoot "lenses\$n.md"; $dst = Join-Path $Root "lab\lenses\$n.md"
  if ((Test-Path $src) -and -not (Test-Path $dst)) { Copy-Item $src $dst; $copied += $n } else { Note "skip lens $n (missing or name taken)" }
}
if (-not $copied) { Note 'no lens copied: stop'; exit 1 }

# 2. Commit the lenses (lanes run from a checkout of HEAD) and queue them.
git add -- ($copied | ForEach-Object { "lab/lenses/$_.md" }) "jobs/port-lenses/report.md" | Out-Null
git commit -q -m "Port lenses for GLM, written by Hydrogen (jobs/port-lenses/report.md), queued overnight`n`nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" | Out-Null
Add-Content -Encoding utf8 (Join-Path $Root 'lab\queue.txt') "# 17 November overnight: Hydrogen's port lenses (jobs/port-lenses/report.md), port only (S173).`n$($copied -join ' ')"
git add lab/queue.txt | Out-Null; git commit -q -m "Queue the overnight port lenses`n`nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" | Out-Null
Note "queued $($copied.Count) lenses: $($copied -join ' ')"

# 3. Run GLM on them, three lanes, and wait for the queue to finish.
& (Join-Path $Root 'scripts\glm-queue.ps1') -Queue lab\queue.txt -Lanes 3
Note 'glm queue finished'

# 4. Hydrogen digests the results.
& (Join-Path $Root 'scripts\launch.ps1') port-digest hydrogen 2>&1 | ForEach-Object { Note "$_" }
