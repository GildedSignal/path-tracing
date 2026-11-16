[CmdletBinding()]
param([ValidateSet('All','Remaining')][string]$Stage='All')
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence='D:\path-tracing-evidence\3.2d-3-P2'
$manifestRoot='D:\path-tracing-evidence\3.2d-3'
$env:CARGO_TARGET_DIR='D:\path-tracing-target\3.2d-3-P2'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass='Idle'
New-Item -ItemType Directory -Force $evidence,(Join-Path $manifestRoot 'T2') | Out-Null
function Check([string]$Name,[scriptblock]$Command,[switch]$Red) {
    $log=Join-Path $evidence "$Name.log"
    $old=$ErrorActionPreference
    try { $ErrorActionPreference='Continue'; & $Command *> $log; $code=$LASTEXITCODE }
    finally {$ErrorActionPreference=$old}
    "exit=$code" | Add-Content $log
    if ($Red) {
        $text=Get-Content -Raw $log
        if ($code -eq 0 -or $text -notmatch 'tie parity witness.*poison=true' -or $text -notmatch 'test result: FAILED') {throw "Poison not caught by running parity assertion: $log"}
    } elseif ($code -ne 0) {throw "$Name failed with exit $code; see $log"}
    Write-Output "$Name exit=$code"
}
Push-Location $root
try {
    $inputPaths=@(& git ls-files engine) + @('jobs/3.2d-3-P2/verify.ps1','jobs/3.2d-3-P2/derivation.md','jobs/3.2d-3-P2/report.ps1','jobs/3.2d-3/verify.ps1','jobs/1.2-footprints/verify.ps1','lab/tracks/straight100.json')
    $inputFiles=@($inputPaths | Sort-Object -Unique | Where-Object {Test-Path -LiteralPath $_ -PathType Leaf} | ForEach-Object {
        $path=[IO.Path]::GetFullPath((Join-Path $root $_));@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}
    })
    if ($Stage -eq 'All') {
        @{head=(& git rev-parse HEAD);target=$env:CARGO_TARGET_DIR;difficulty='Peaceful';mobs='Off';files=$inputFiles;command='verify.ps1 -Stage All'} | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 (Join-Path $evidence 'run-inputs.json')
        Push-Location engine
        try {
            $savedPoison=$env:PATHTRACING_T2_POISON
            try {
                $env:PATHTRACING_T2_POISON='1'
                Check 'poison' {cargo test --offline --release -p plan --test zones_prereqs t2_binade -- --nocapture} -Red
                Remove-Item Env:PATHTRACING_T2_POISON
                Check 't2' {cargo test --offline --release -p plan --test zones_prereqs t2_binade -- --nocapture}
                Check 't1-local' {cargo test --offline -p plan --test zones_prereqs t1_observed_transient -- --nocapture}
                Check 'workspace' {cargo test --offline --workspace}
            } finally {
                if ($null -eq $savedPoison) {Remove-Item Env:PATHTRACING_T2_POISON -ErrorAction SilentlyContinue} else {$env:PATHTRACING_T2_POISON=$savedPoison}
            }
        } finally {Pop-Location}
    } else {
        $previous=Get-Content -Raw (Join-Path $evidence 'run-inputs.json') | ConvertFrom-Json
        foreach ($file in $previous.files) {if ((Get-FileHash -LiteralPath $file.path).Hash -ne $file.sha256) {throw "Changed replay input $($file.path)"}}
        foreach ($name in 't2','t1-local','workspace') {
            if ((Get-Content -Raw (Join-Path $evidence "$name.log")) -notmatch '(?m)^exit=0\s*$') {throw "Missing successful $name"}
        }
        if ((Get-Content -Raw (Join-Path $evidence 'poison.log')) -notmatch 'tie parity witness.*poison=true') {throw 'Missing poison assertion'}
    }
    $sources=@('engine/law/src/movement.rs','engine/law/src/world.rs','engine/law/src/shapes.rs','engine/law/src/footprint.rs','engine/law/src/pattern.rs')
    $pins=@($sources | ForEach-Object { @{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash} })
    Check 'footprints' {& powershell -NoProfile -ExecutionPolicy Bypass -File ./jobs/1.2-footprints/verify.ps1}
    foreach ($pin in $pins) {if ((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256) {throw "Unrestored source $($pin.path)"}}
    $caught=@([regex]::Matches((Get-Content -Raw (Join-Path $evidence 'footprints.log')),'(?m)^([a-z-]+) CAUGHT\s*$'))
    if ($caught.Count -ne 13) {throw "Expected 13/13 caught, got $($caught.Count)"}
    $paths=$inputPaths
    $files=@($paths | Sort-Object -Unique | Where-Object {Test-Path -LiteralPath $_ -PathType Leaf} | ForEach-Object {
        $path=[IO.Path]::GetFullPath((Join-Path $root $_));@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}
    })
    $files+=@('poison','t2','t1-local','workspace','footprints') | ForEach-Object {$path=Join-Path $evidence "$_.log";@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}}
    $inputRecord=Join-Path $evidence 'run-inputs.json'
    $files+=@{path=$inputRecord;sha256=(Get-FileHash -LiteralPath $inputRecord).Hash}
    $measurements=@([regex]::Matches((Get-Content -Raw (Join-Path $evidence 't2.log')),'(?m)^MEASURE (.+)$') | ForEach-Object {$_.Groups[1].Value.Trim()})
    $assertions=@([regex]::Matches((Get-Content -Raw (Join-Path $evidence 't2.log')),'(?m)^ASSERT (.+)$') | ForEach-Object {$_.Groups[1].Value.Trim()})
    @{passed=$true;difficulty='Peaceful';mobs='Off';live=$false;consumer='U1 shared zone kernel';files=$files;sourcePins=$pins;
      assertions=$assertions;measurements=$measurements;footprintMutants=$caught.Count;refusals=@();
      worstMargins='exact equality of bits; no numerical tolerance';scope='prepared carried +x sprint-jump cycle; resource numeric recurrence belongs to P4';
      prerequisite='T1 canonical manifest is verified independently; local T1 test also rerun'} | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 (Join-Path $evidence 'manifest.json')
    Copy-Item -LiteralPath (Join-Path $evidence 'manifest.json') -Destination (Join-Path $manifestRoot 'T2/manifest.json')
    Check 'prerequisites' {& ./jobs/3.2d-3/verify.ps1 -Case Prerequisites -Tests T1,T2 -Difficulty Peaceful -Mobs Off -Evidence $manifestRoot}
    $env:CARGO_TARGET_DIR='D:\path-tracing-target\3.2d-3-P2'
    & ./jobs/3.2d-3-P2/report.ps1
    Write-Output 'P2 offline gates passed; no live T claimed.'
} finally {Pop-Location}
