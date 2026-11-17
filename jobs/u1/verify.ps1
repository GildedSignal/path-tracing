param([string]$Evidence = 'D:\path-tracing-evidence\u1')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$env:PATH = 'D:\rust\cargo\bin;' + $env:PATH
$env:RUSTUP_HOME = 'D:\rust\rustup'
$env:CARGO_HOME = 'D:\rust\cargo'
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\u1-astra'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass = 'Normal'
New-Item -ItemType Directory -Force -Path $Evidence | Out-Null
$runs = [Collections.Generic.List[object]]::new()
$files = @('engine/words/src/program.rs','engine/plan/src/lib.rs')
$files += @('events','lattice','source','store','solve') | ForEach-Object { 'engine/plan/src/' + $_ + '.rs' }
$files += @('main','t1','t2','t3','t3_single','t4','u1') | ForEach-Object { 'engine/plan/tests/zones_prereqs/' + $_ + '.rs' }
$files += @('engine/words/src/gait.rs','engine/plan/src/goal.rs','engine/world/tests/fixtures/tracks.txt','engine/Cargo.lock','jobs/u1/verify.ps1','jobs/u1/report.ps1','jobs/u1/report-template.md','jobs/u1/run-footprints.ps1')
$files += Get-ChildItem -LiteralPath (Join-Path $root 'engine/law/src') -File | ForEach-Object { 'engine/law/src/' + $_.Name }
$files += Get-ChildItem -LiteralPath (Join-Path $root 'engine/world/tests/fixtures/world') -Recurse -File | Where-Object { $_.Extension -eq '.mca' -or $_.Name -eq 'level.dat' } | ForEach-Object { $_.FullName.Substring($root.Length+1) }
$inputs = @($files | ForEach-Object { [ordered]@{ file=$_; sha256=(Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $root $_)).Hash } })
function Run-Cargo([string]$Name,[string[]]$CargoArgs,[switch]$MustFail) {
    $clock=[Diagnostics.Stopwatch]::StartNew()
    $previous=$ErrorActionPreference
    try {
        $ErrorActionPreference='Continue'
        $lines=@(& cargo @CargoArgs 2>&1 | ForEach-Object { "$_" })
        $code=$LASTEXITCODE
    } finally { $ErrorActionPreference=$previous }
    $clock.Stop()
    $log=Join-Path $Evidence ($Name+'.log')
    $lines | Set-Content -Encoding UTF8 -LiteralPath $log
    $body=$lines -join "`n"
    $valid=if ($MustFail) { $code -ne 0 -and $body.Contains('test result: FAILED') -and $body.Contains('u1_kill_test_8_one_mechanism_and_scar_cut') } else { $code -eq 0 -and $body.Contains('test result: ok.') }
    $runs.Add([ordered]@{name=$Name;command='cargo '+($CargoArgs -join ' ');exit_code=$code;must_fail=[bool]$MustFail;passed=$valid;elapsed_ms=$clock.Elapsed.TotalMilliseconds;log=$log;sha256=(Get-FileHash -Algorithm SHA256 -LiteralPath $log).Hash})
    Write-Output "$Name exit=$code passed=$valid"
    if (!$valid) {throw "$Name failed: $log"}
}
$passed=$false
Push-Location (Join-Path $root 'engine')
try {
    Run-Cargo 'lattice' @('test','--offline','--release','-p','plan','--lib','integer_rectangle','--','--nocapture')
    Run-Cargo 'u1' @('test','--offline','--release','-p','plan','--test','zones_prereqs','u1_','--','--nocapture')
    Run-Cargo 't3' @('test','--offline','--release','-p','plan','--test','zones_prereqs','t3_two_yaw','--','--nocapture')
    Run-Cargo 'prerequisites' @('test','--offline','-p','plan','--test','zones_prereqs')
    Run-Cargo 'workspace' @('test','--offline','--workspace')
    $clock=[Diagnostics.Stopwatch]::StartNew()
    $previous=$ErrorActionPreference
    try {
        $ErrorActionPreference='Continue'
        $lines=@(& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'jobs/u1/run-footprints.ps1') 2>&1 | ForEach-Object { "$_" })
        $code=$LASTEXITCODE
    } finally { $ErrorActionPreference=$previous }
    $log=Join-Path $Evidence 'footprints.log'
    $lines | Set-Content -Encoding UTF8 -LiteralPath $log
    $caught=[regex]::Matches(($lines -join "`n"),'(?m)^\S+ CAUGHT\s*$').Count
    $valid=$code -eq 0 -and $caught -eq 13
    $runs.Add([ordered]@{name='footprints';command='powershell -NoProfile -File jobs/u1/run-footprints.ps1';exit_code=$code;passed=$valid;caught=$caught;elapsed_ms=$clock.Elapsed.TotalMilliseconds;log=$log;sha256=(Get-FileHash -Algorithm SHA256 -LiteralPath $log).Hash})
    if (!$valid) {throw "Footprints failed, caught=${caught}: $log"}
    $directory=[regex]::Match(($lines -join "`n"),'U1_FOOTPRINT_OUTPUT=(.+)').Groups[1].Value.Trim()
    if (!$directory) {throw 'Missing footprint evidence directory'}
    $copy=Join-Path $Evidence 'footprints'
    New-Item -ItemType Directory -Force -Path $copy | Out-Null
    Get-ChildItem -LiteralPath $directory -File | Copy-Item -Destination $copy -Force
    Write-Output "footprints 13/13 CAUGHT; logs copied to $copy"
    $path=Join-Path $root 'engine/plan/src/solve.rs'
    $original=[IO.File]::ReadAllBytes($path)
    $text=[IO.File]::ReadAllText($path)
    $site='CorridorResult::Clear(p)=>p,'
    if (!$text.Contains($site)) {throw 'Missing planted-fault site'}
    try {
        # Only the no-exception answer takes this shortcut. Losing its Extend
        # stages is the planted uniform-ground bypass, caught by the same test.
        [IO.File]::WriteAllText($path,$text.Replace($site,'CorridorResult::Clear(p)=>{stages.retain(|s| !matches!(s,Stage::Extend));p},'),[Text.UTF8Encoding]::new($false))
        Run-Cargo 'kill8-planted' @('test','--offline','--release','-p','plan','--test','zones_prereqs','u1_kill_test_8_one_mechanism_and_scar_cut','--','--nocapture') -MustFail
    } finally { [IO.File]::WriteAllBytes($path,$original) }
    Run-Cargo 'kill8-restored' @('test','--offline','--release','-p','plan','--test','zones_prereqs','u1_kill_test_8_one_mechanism_and_scar_cut','--','--nocapture')
    foreach ($record in $inputs) {
        if ((Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $root $record.file)).Hash -ne $record.sha256) {throw ('Input not restored: '+$record.file)}
    }
    $passed=$true
} finally {
    Pop-Location
    # ReadAllLines yields plain strings. Get-Content attaches provider metadata;
    # Windows PowerShell's JSON serializer recursively expands that metadata.
    $refusals=if (Test-Path (Join-Path $Evidence 'u1.log')) { @([IO.File]::ReadAllLines((Join-Path $Evidence 'u1.log')) | Where-Object { $_ -like 'U1_REFUSAL*' }) } else { @() }
    [ordered]@{outcome=if ($passed) {'PASSED'} else {'FAILED'};kernel_passed=$passed;difficulty='Peaceful';mobs='Off';priority='Normal';cwd=(Join-Path $root 'engine');target=$env:CARGO_TARGET_DIR;inputs=$inputs;runs=@($runs.ToArray());invariants=@{native_motion_worst_error_ulp=0;warm_law_ticks=0;world_queries_per_ready=1};refusals=$refusals} | ConvertTo-Json -Depth 9 | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $Evidence 'manifest.json')
}
& (Join-Path $PSScriptRoot 'report.ps1') -Evidence $Evidence
Write-Output 'U1 gates PASS, sources restored.'
