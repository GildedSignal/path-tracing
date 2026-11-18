# The scoreboard (the overhaul's C5): the only numbers reported as progress, each beside its floor.
#   .\scripts\scoreboard.ps1 [-Flight <live play log>] [-SkipTiming]
# Thinking per new command comes from engine/plan/tests/real_time.rs (release build); T in his game from the
# latest flight log, through flight-scoreboard.ps1.
param([string]$Flight, [switch]$SkipTiming)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"
$env:PATH = "D:\rust\cargo\bin;$env:PATH"; $env:RUSTUP_HOME = 'D:\rust\rustup'; $env:CARGO_HOME = 'D:\rust\cargo'
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\scoreboard'
"== Thinking per new command (one living store, every command a new goal), against the floor"
if (-not $SkipTiming) {
  Push-Location (Join-Path $Root 'engine')
  $ErrorActionPreference = 'Continue'   # PowerShell 5.1 throws on cargo's progress lines under Stop
  $out = & cargo test --offline --release -p plan --test real_time -- --nocapture 2>$null
  $ErrorActionPreference = 'Stop'
  Pop-Location
  $out | Select-String '^REAL_TIME ' | ForEach-Object {
    $l = $_.Line
    $name = [regex]::Match($l, 'series="([^"]+)"').Groups[1].Value
    $g = { param($k) [regex]::Match($l, "$k=([0-9.]+)").Groups[1].Value }
    '{0,-34} n {1,3}  refused {2,2}  median {3,10:N1} us  p90 {4,10:N1} us  max {5,10:N1} us  floor {6,6:N1} us  gap {7} orders' -f $name, (& $g 'n'), (& $g 'refused'), ([double](& $g 'median_ns')/1000), ([double](& $g 'p90_ns')/1000), ([double](& $g 'max_ns')/1000), ([double](& $g 'floor_ns')/1000), (& $g 'median_gap_oom')
  }
}
"== T in his game (the track clock) against the prediction and his best"
if (-not $Flight) { $Flight = Get-ChildItem 'D:\path-tracing-evidence' -Recurse -Filter 'live*.txt' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime | Select-Object -Last 1 -ExpandProperty FullName }
if ($Flight) { "log: $Flight"; & "$PSScriptRoot\flight-scoreboard.ps1" -Log $Flight } else { "no flight log yet" }
