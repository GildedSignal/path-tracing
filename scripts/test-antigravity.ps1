# The test an Antigravity run must pass before it may start (S129): two failures that should never recur.
#   1. A run ended EMPTY after one denied read_file (Argon, twice, ~10% of its week each).
#   2. An Opus run on Antigravity spent the Claude meter's week with no output.
#   .\scripts\test-antigravity.ps1            offline checks, then a small live Hydrogen probe (~1 point of the Gemini 5 h meter)
#   .\scripts\test-antigravity.ps1 -Offline   offline checks only
# On a pass it stamps jobs/_probe-agy/passed.txt with the hash of gemini.ps1 and launch.ps1; launch.ps1
# refuses every Antigravity job while that stamp does not match the scripts as they are now.
param([switch]$Offline)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$fail = @()
function Check($name, [scriptblock]$test) { try { if (& $test) { "PASS  $name" } else { "FAIL  $name"; $script:fail += $name } } catch { "FAIL  $name ($($_.Exception.Message))"; $script:fail += $name } }
# A refusal counts only with its own message: a script that fails to parse also "throws" (it did, once).
function Refuses([scriptblock]$s, [string]$because) { try { & $s *> $null; $false } catch { $_.Exception.Message -match $because } }

# Offline: every script parses, and Opus is refused at both doors before any model is called.
foreach ($f in 'gemini.ps1','launch.ps1','gpt.ps1','glm-queue.ps1','follow-up.ps1') {
  Check "$f parses" { $e = $null; [Management.Automation.Language.Parser]::ParseFile("$PSScriptRoot\$f", [ref]$null, [ref]$e) | Out-Null; -not $e }
}
Check 'launch.ps1 refuses opus' { Refuses { & "$PSScriptRoot\launch.ps1" anomaly-1-stepup opus -DryRun } 'Opus does not run on Antigravity' }
Check 'gemini.ps1 refuses an opus model' { Refuses { & "$PSScriptRoot\gemini.ps1" -Job anomaly-1-stepup -Model claude-opus-5-5-high } 'Opus does not run on Antigravity' }
Check 'every grant has its %20 form' {
  $src = Get-Content "$PSScriptRoot\gemini.ps1" -Raw; $src -match "replace ' ','%20'"
}

if (-not $Offline -and -not $fail) {
  # Live: one small Hydrogen job reads one file per path form that has been denied before.
  $job = '_probe-agy'; $dir = Join-Path $Root "jobs\$job"; New-Item -ItemType Directory -Force $dir | Out-Null
  Remove-Item (Join-Path $dir '*') -Exclude 'passed.txt' -Force -ErrorAction SilentlyContinue
  $wt = (Get-ChildItem 'D:\path-tracing-wt' -Directory | Where-Object { Test-Path (Join-Path $_.FullName 'AGENTS.md') } | Select-Object -First 1).FullName
  $forms = [ordered]@{
    'backslash'  = (Join-Path $Root 'docs\clan.md')
    'slash'      = ($Root -replace '\\','/') + '/AGENTS.md'
    'no drive'   = (($Root -replace '\\','/') -replace '^[A-Za-z]:','') + '/docs/HANDOFF.md'
    'relative'   = 'docs/PRINCIPLES.md'
    'worktree'   = (Join-Path $wt 'README.md')
    'file url'   = 'file:///' + (($Root -replace '\\','/') -replace ' ','%20') + '/README.md'
  }
  $first = @{ backslash = 'docs\clan.md'; slash = 'AGENTS.md'; 'no drive' = 'docs\HANDOFF.md'; 'file url' = 'README.md'; relative = 'docs\PRINCIPLES.md' }
  $lines = foreach ($k in $forms.Keys) { "- $k`: ``$($forms[$k])``" }
  $prompt = "# Probe: read six files by the exact paths given`n`nRead each file below. Some are given as a relative path (relative to C:\Users\PASCAL\Desktop\path tracing) or as a file:/// link: follow your instructions on paths. For each, write one line to your report: the label, a colon, then the file's first line verbatim. Nothing else.`n`n$($lines -join "`n")`n`nEnd your final message with the line PROBE DONE."
  [IO.File]::WriteAllText((Join-Path $dir 'prompt.md'), $prompt, [Text.UTF8Encoding]::new($false))
  & "$PSScriptRoot\gemini.ps1" -Job $job -Model 'gemini-4-hydrogen-low' -ReadDir $wt -Budget 5 -StallMinutes 6 -TimeoutMinutes 10 | Out-Host
  $report = (Get-Content (Join-Path $dir 'last.md') -Raw -ErrorAction SilentlyContinue) + (Get-Content (Join-Path $dir 'report.md') -Raw -ErrorAction SilentlyContinue)
  foreach ($k in $forms.Keys) {
    $path = if ($first.ContainsKey($k)) { Join-Path $Root $first[$k] } else { $forms[$k] }
    $expect = (Get-Content $path -TotalCount 1).Trim()
    Check "reads by the $k form" { $report -and $report.Contains($expect) }
  }
  Check 'no read was soft-denied' { -not (Select-String -Path (Join-Path $dir 'agy.log') -Pattern 'soft-denying' -Quiet) }
  Check 'not killed' { -not (Test-Path (Join-Path $dir 'killed.txt')) }
}

if ($fail) { "FAILED: $($fail -join '; ')"; exit 1 }
if ($Offline) { 'offline checks pass (no stamp: the live probe did not run)'; exit 0 }
$hash = (Get-FileHash "$PSScriptRoot\gemini.ps1").Hash + (Get-FileHash "$PSScriptRoot\launch.ps1").Hash
[IO.File]::WriteAllText((Join-Path $Root 'jobs\_probe-agy\passed.txt'), "$hash`n$(Get-Date -Format 'yyyy-MM-dd HH:mm')`n")
'all checks pass; stamped jobs\_probe-agy\passed.txt'
