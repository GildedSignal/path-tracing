[CmdletBinding()]
param(
 [ValidateSet('Slice','CaptureBaseline','Prerequisites','MillionThinking','Straight800Live','Retarget','VariedRolling','Current21','T4Food')][string]$Case = 'Slice',
 [string]$Slice = 'A',
 [ValidateSet('Peaceful')][string]$Difficulty = 'Peaceful',
 [ValidateSet('Off')][string]$Mobs = 'Off',
 [string[]]$Tests = @('T1','T2','T3','T4'),
 [int]$Repeat = 5,
 [switch]$Live,
 [string]$Evidence = 'D:\path-tracing-evidence\3.2d-3-A'
)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\r0-channel'
New-Item -ItemType Directory -Force $Evidence | Out-Null
if ($Live) { throw 'S135: slice A launches no game/server/client. K supplies the live harness after its prerequisites.' }
if ($Case -eq 'CaptureBaseline') {
    & "$PSScriptRoot/build-observer.ps1"
    throw 'Observer-only artifact prepared. No live baseline is claimed: K must pair it with the frozen baseline binary, not the candidate jar, and supply actual hop/difficulty/restoration evidence.'
}
if ($Case -eq 'Prerequisites') {
    foreach ($test in $Tests) {
        $path = Join-Path $Evidence "$test/manifest.json"
        if (!(Test-Path -LiteralPath $path)) { throw "Missing prerequisite artifact $test" }
        $artifact = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json
        if (!$artifact.passed -or $artifact.difficulty -ne $Difficulty -or $artifact.mobs -ne $Mobs) { throw "Unqualified prerequisite $test" }
        foreach ($file in $artifact.files) { if ((Get-FileHash -LiteralPath $file.path).Hash -ne $file.sha256) { throw "Changed prerequisite file $($file.path)" } }
    }
    return
}
if ($Case -ne 'Slice' -or $Slice -ne 'A') { throw 'Dependent slices/live cases require their T1-T4 and production artifacts; this is the A skeleton.' }
$commands = @()
$pinPaths = @('engine/law/src/movement.rs','engine/law/src/pattern.rs','engine/law/src/footprint.rs','engine/law/src/world.rs','engine/law/src/shapes.rs','engine/words/src/class.rs','engine/words/src/table.rs')
$pins = @($pinPaths | ForEach-Object { @{path=$_;sha256=(Get-FileHash -LiteralPath (Join-Path $root $_)).Hash} })
function Check([string]$Name,[scriptblock]$Run) {
    $log = Join-Path $Evidence "$Name.log"
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & $Run *>&1 | ForEach-Object { "$_" } | Tee-Object -FilePath $log | Write-Output
        $code = $LASTEXITCODE
    } finally { $ErrorActionPreference = $previous }
    if ($code) { throw "$Name failed: $log" }
    $script:commands += @{ name=$Name; log=$log; sha256=(Get-FileHash -LiteralPath $log).Hash; exit=0 }
}
Push-Location $root
try {
    Check 'click-path' { & powershell -NoProfile -ExecutionPolicy Bypass -File ./bot/tests/test-resident-path.ps1 }
    Check 'native-java' { & powershell -NoProfile -ExecutionPolicy Bypass -File ./bot/tests/test-native.ps1 -Evidence $Evidence }
    Check 'workspace' { Push-Location engine; try { & cargo test --offline } finally { Pop-Location } }
    # This ORIGINAL gate plants its frozen mutants and restores source in finally.
    Check 'footprints' { & powershell -NoProfile -ExecutionPolicy Bypass -File ./jobs/1.2-footprints/verify.ps1 }
    foreach ($pin in $pins) { if ((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256) { throw "Original footprint gate did not restore $($pin.path)" } }
    $mutants = @([regex]::Matches((Get-Content -Raw ./jobs/1.2-footprints/verify.ps1),"(?m)^\s*Mutant '([^']+)'") | ForEach-Object { $_.Groups[1].Value })
    $caught = @([regex]::Matches((Get-Content -Raw (Join-Path $Evidence 'footprints.log')),'(?m)^([a-z-]+) CAUGHT') | ForEach-Object { $_.Groups[1].Value })
    if (@(Compare-Object ($mutants | Sort-Object) ($caught | Sort-Object)).Count) { throw 'Dynamic footprint mutant set differs from CAUGHT set' }
    Check 'bot-build' { & powershell -NoProfile -ExecutionPolicy Bypass -File ./bot/build.ps1 }
    Check 'bot-tracks' { & powershell -NoProfile -ExecutionPolicy Bypass -File ./bot/tests/test-tracks.ps1 }
    Check 'observer-build' { & powershell -NoProfile -ExecutionPolicy Bypass -File ./jobs/3.2d-3/build-observer.ps1 }
    $files = @(& git ls-files 'engine/*' 'bot/*' 'scripts/*' 'jobs/1.2-footprints/*') + @(& git diff --name-only) + @(& git ls-files --others --exclude-standard)
    $hashes = @($files | Sort-Object -Unique | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object { @{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash} })
    $totals = @{ passed=0; ignored=0 }
    foreach ($m in [regex]::Matches((Get-Content -Raw (Join-Path $Evidence 'workspace.log')),'test result: ok\. (\d+) passed; \d+ failed; (\d+) ignored')) { $totals.passed += [int]$m.Groups[1].Value; $totals.ignored += [int]$m.Groups[2].Value }
    @{ slice='A'; passed=$true; difficulty=$Difficulty; mobs=$Mobs; live=$false; prerequisites=@(); commands=$commands; files=$hashes; sourcePins=$pins; workspaceTotals=$totals; footprintMutants=$mutants; refusals=@('required unknown state words','stale version/source/observation','unreceived or unsupported source'); worstMargins='bit-exact equality only; no numerical tolerance'; baseline='D:/path-tracing-evidence/3.2d-3-A-baseline/manifest.json' } | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $Evidence 'manifest.json') -Encoding UTF8
    Write-Output 'Slice A OFFLINE PASS; no live T or baseline capture claimed.'
} finally { Pop-Location }
