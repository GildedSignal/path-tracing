param([ValidateSet('All','Baseline','Measurements','Pins','Inherited','Counters')][string]$Mode='All')
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence='D:/path-tracing-evidence/warm-floor'
$env:CARGO_TARGET_DIR='D:/path-tracing-target/warm-floor'
$env:CARGO_NET_OFFLINE='true'
New-Item -ItemType Directory -Force $evidence | Out-Null
# SHA256 of the three files in the e6de417 checkout.
$files=@{
    'pins_point_hold.rs'='B78EAAF3A53DE7B0A70E1A2F3DA12B5246437962F7909000301309190468FF82'
    'pin_gate.rs'='9471A461C09A3B629416912F22AC06421D67C6362CED3DB6495C548EFF20D9A5'
    'premises.rs'='D322629D5BEBCAB8E666AE5FF0438FFB37D581C667B0C790C7A855FB4C0E8130'
}
foreach($name in $files.Keys) {
    $path=Join-Path $root "engine/plan/tests/u2b/$name"
    if((Get-FileHash -LiteralPath $path).Hash -ne $files[$name]) {throw "Frozen oracle changed in $name"}
}
if($Mode -eq 'Pins'){return}
if($Mode -eq 'All'){
    $failed=@()
    foreach($name in 'Baseline','Measurements','Counters','Inherited'){
        & powershell -NoProfile -ExecutionPolicy Bypass -File $PSCommandPath -Mode $name
        if($LASTEXITCODE){$failed+=$name}
    }
    & (Join-Path $PSScriptRoot 'score.ps1') -Series Measurements -Gate
    if($failed.Count){throw "Failed replay stages: $failed"}
    return
}
if($Mode -eq 'Inherited') {
    # The inherited driver writes fixed filenames, including the previous judge's
    # records. Isolate its output and build target, preserving every assertion.
    $scripts=Join-Path $evidence 'u2b-driver'
    $records=Join-Path $evidence 'u2b'
    New-Item -ItemType Directory -Force $scripts,$records | Out-Null
    $old='D:\path-tracing-evidence\u2b'
    foreach($name in 'cost-baseline.json','premises.json','final-inputs.json') {
        Copy-Item -LiteralPath (Join-Path $old $name) -Destination $records
    }
    $archive=Join-Path $old 'inputs'
    Copy-Item -LiteralPath $archive -Destination $records -Recurse -Force
    foreach($script in (Get-ChildItem (Join-Path $root 'jobs/u2b') -Filter '*.ps1')) {
        $text=[IO.File]::ReadAllText($script.FullName)
        $text=$text.Replace("Join-Path `$PSScriptRoot '../..'","Join-Path '$root' '.'")
        $text=$text.Replace($old,$records).Replace('D:\path-tracing-target\u2b','D:\path-tracing-target\warm-floor')
        $text=$text.Replace("Join-Path 'D:\path-tracing-evidence'","Join-Path '$evidence'")
        [IO.File]::WriteAllText((Join-Path $scripts $script.Name),$text)
    }
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $scripts 'verify.ps1')
    exit $LASTEXITCODE
}

function Run([string]$name,[string[]]$arguments,[string[]]$options=@()) {
    $out=Join-Path $evidence "$Mode-$name.log"
    $ErrorActionPreference='Continue'
    & cargo test --offline --release -p plan @arguments -- --nocapture --test-threads=1 @options 2>&1 |
        ForEach-Object {"$_"} | Out-File -FilePath $out -Encoding utf8 -Width 32767
    if($LASTEXITCODE){throw "cargo failed ($LASTEXITCODE), see $out"}
}
$buildRoot=$root
if($Mode -eq 'Baseline') {
    # Rebuild the historical implementation, even when the current worktree has changed.
    $buildRoot=Join-Path $evidence 'base-e6de417'
    New-Item -ItemType Directory -Force $buildRoot | Out-Null
    $archive=Join-Path $evidence 'base-e6de417.tar'
    & git -C $root archive --format=tar -o $archive e6de417
    if($LASTEXITCODE){throw 'Could not export exact base'}
    & tar -xf $archive -C $buildRoot
    if($LASTEXITCODE){throw 'Could not extract base'}
    foreach($path in 'engine/plan/tests/real_time.rs','engine/plan/tests/u2.rs') {
        Copy-Item -LiteralPath (Join-Path $root $path) -Destination (Join-Path $buildRoot $path)
    }
    $env:CARGO_TARGET_DIR='D:/path-tracing-target/warm-floor-baseline'
}
Push-Location (Join-Path $buildRoot 'engine')
try {
    if($Mode -eq 'Counters') {
        Run 'commands' @('--features','solve-counters','--test','real_time')
        return
    }
    if($Mode -eq 'Measurements'){
        Run 'witnesses' @('--test','warm_floor')
        Run 'contact' @('--test','u2','contact_observation_preserves_native_recurrence')
        Run 'arithmetic' @('--lib')
    }
    Run 'commands' @('--test','real_time')
    Run 'hot' @('--test','u2b_cost')
    Run 'interleaved' @('--test','zones_prereqs','u1_kill_test_17_million_line_peaceful')
    Run 'seen' @('--test','u2','u2_kill10_discovered_program_current_record') @('--ignored')
} finally {Pop-Location}
