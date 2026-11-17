# Wave 2 chain (17 November, S207): Hydrogen digests and writes lenses -> copy, commit, queue -> GLM runs them.
# Started detached by the integrator; every step is logged to jobs\port-wave-2\chain.log.
$ErrorActionPreference = 'Continue'
$Root = Split-Path (Split-Path $PSScriptRoot)
Set-Location $Root
$log = Join-Path $PSScriptRoot 'chain.log'
function Note($t) { Add-Content -Encoding utf8 $log "$(Get-Date -Format 'yyyy-MM-dd HH:mm') $t" }

# 1. Wait for Hydrogen's port-wave-2 run to end.
while (Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'gemini\.ps1.*-Job port-wave-2\b' }) { Start-Sleep 30 }
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
git add -- ($copied | ForEach-Object { "lab/lenses/$_.md" }) "jobs/port-wave-2/report.md" | Out-Null
git commit -q -m "Port lenses wave 2 for GLM, written by Hydrogen (jobs/port-wave-2/report.md), queued (S207)`n`nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" | Out-Null
Add-Content -Encoding utf8 (Join-Path $Root 'lab\queue.txt') "# 17 November evening (S207): Hydrogen's port lenses wave 2 (jobs/port-wave-2/report.md), port only (S173).`n$($copied -join ' ')"
git add lab/queue.txt | Out-Null; git commit -q -m "Queue the port lenses, wave 2`n`nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" | Out-Null
Note "queued $($copied.Count) lenses: $($copied -join ' ')"

# 3. Run GLM on them, three lanes, and wait for the queue to finish.
& (Join-Path $Root 'scripts\glm-queue.ps1') -Queue lab\queue.txt -Lanes 3
Note 'glm queue finished'

Note 'chain done; the integrator digests wave 2 next'
