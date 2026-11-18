[CmdletBinding()]
param([ValidateSet('All','Remaining')][string]$Stage = 'All')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence = 'D:\path-tracing-evidence\3.2d-3-P1'
$manifestRoot = 'D:\path-tracing-evidence\3.2d-3'
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\3.2d-3-P1'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass = 'Idle'
New-Item -ItemType Directory -Force $evidence, (Join-Path $manifestRoot 'T1') | Out-Null
function Check([string]$Name, [string]$Directory, [scriptblock]$Command) {
    Push-Location $Directory
    try {
        $old = $ErrorActionPreference
        try { $ErrorActionPreference = 'Continue'; & $Command *> (Join-Path $evidence "$Name.log"); $code = $LASTEXITCODE }
        finally { $ErrorActionPreference = $old }
        "exit=$code" | Add-Content (Join-Path $evidence "$Name.log")
        if ($code -ne 0) { throw "$Name failed with exit $code; see $evidence\$Name.log" }
    } finally { Pop-Location }
}
if ($Stage -eq 'All') {
    Check 'baseline' 'D:\path-tracing-wt\plain\engine' { cargo test --offline -p plan --test plain straight -- --nocapture }
    Check 't1' (Join-Path $root 'engine') { cargo test --offline -p plan --test zones_prereqs t1_observed_transient -- --nocapture }
    Check 'workspace' (Join-Path $root 'engine') { cargo test --offline --workspace }
} else {
    foreach ($name in 'baseline','t1','workspace') {
        $log = Join-Path $evidence "$name.log"
        if (!(Test-Path $log) -or !((Get-Content -Raw $log) -match '(?m)^exit=0\s*$')) { throw "Missing successful $name log" }
    }
}
$pins = @('movement.rs','world.rs','shapes.rs','footprint.rs','pattern.rs') | ForEach-Object { $p = Join-Path $root "engine/law/src/$_"; @{path=$p; hash=(Get-FileHash $p).Hash} }
Check 'footprints' $root { & powershell -NoProfile -ExecutionPolicy Bypass -File ./jobs/1.2-footprints/verify.ps1 }
foreach ($pin in $pins) { if ((Get-FileHash $pin.path).Hash -ne $pin.hash) { throw "Source not restored: $($pin.path)" } }
$caught = @([regex]::Matches((Get-Content -Raw (Join-Path $evidence 'footprints.log')), '(?m)^([a-z-]+) CAUGHT\s*$'))
if ($caught.Count -ne 13) { throw "Expected 13 mutants, caught $($caught.Count)" }
$files = @('engine/plan/Cargo.toml','engine/plan/tests/zones_prereqs/main.rs','engine/plan/tests/zones_prereqs/t1.rs','engine/plan/tests/zones_prereqs/t2.rs','engine/plan/tests/zones_prereqs/t3.rs','engine/plan/tests/zones_prereqs/t4.rs','bot/tests/fixtures/native-state.hex','bot/tests/fixtures/recorded-state.hex','engine/world/tests/fixtures/tracks.txt','engine/world/tests/fixtures/world/level.dat','engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/r.0.-1.mca','engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/r.0.0.mca') | ForEach-Object { $p = Join-Path $root $_; @{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash} }
$files += @('baseline.log','t1.log','workspace.log','footprints.log') | ForEach-Object { $p = Join-Path $evidence $_; @{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash} }
@{passed=$true; difficulty='Peaceful'; mobs='Off'; files=$files} | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 (Join-Path $manifestRoot 'T1/manifest.json')
Check 'prerequisites' $root { & powershell -NoProfile -ExecutionPolicy Bypass -File ./jobs/3.2d-3/verify.ps1 -Case Prerequisites -Tests T1 -Difficulty Peaceful -Mobs Off -Evidence $manifestRoot }
Write-Output 'T1 offline gates passed.'
