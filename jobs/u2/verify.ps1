param([switch]$Search,[switch]$Footprints)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$env:PATH='D:\rust\cargo\bin;'+$env:PATH
$env:RUSTUP_HOME='D:\rust\rustup'
$env:CARGO_HOME='D:\rust\cargo'
$env:CARGO_TARGET_DIR='D:\path-tracing-target\u2'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass='Normal'
$evidence='D:\path-tracing-evidence\u2'
$gates=[Collections.Generic.List[object]]::new()
function Gate([string]$name,[string[]]$arguments,[string]$red='') {
    & (Join-Path $PSScriptRoot 'run.ps1') -Name $name -CargoArgs $arguments
    $record=Get-Content -Raw -LiteralPath (Join-Path $evidence ($name+'.json')) | ConvertFrom-Json
    $log=[IO.File]::ReadAllText((Join-Path $evidence ($name+'.log')))
    $ok=if($red){$record.exit -ne 0 -and $log.Contains('test result: FAILED') -and $log.Contains($red)}else{$record.exit -eq 0 -and $log.Contains('test result: ok.')}
    $gates.Add(@{name=$name;expected_red=[bool]$red;recorded=$ok;exit=$record.exit})
    if(!$ok){throw "$name did not produce its required result"}
}
try {
    Gate 'u2-final' @('test','--offline','--release','-p','plan','--test','u2','--','--nocapture','--test-threads=1')
    Gate 'maps-and-stairs' @('test','--offline','--release','-p','plan','--lib','--test','u2_stairs','--','--nocapture')
    Gate 'kill11' @('test','--offline','--release','-p','plan','--test','u2','u2_kill11','--','--ignored','--nocapture') 'plateau count grows under refinement'
    Gate 'translation-premise' @('test','--offline','--release','-p','plan','--test','u2','u2_translation_premise','--','--ignored','--nocapture') 'same signature and binade do not imply pure translation'
    if($Search){Gate 'kill10-bar' @('test','--offline','--release','-p','plan','--test','u2','u2_kill10_post_ladder_bar','--','--ignored','--nocapture')}
    if(!(Test-Path (Join-Path $evidence 'kill10-bar.log'))){throw 'Run with -Search to generate KT10 evidence first'}
    Gate 'kill10-current-record' @('test','--offline','--release','-p','plan','--test','u2','u2_kill10_discovered_program_current_record','--','--ignored','--nocapture')
    Gate 'core-final-release' @('test','--offline','--release','-p','plan','--test','zones_prereqs','u1_','--','--nocapture','--test-threads=1')
    Gate 'core-two-yaw' @('test','--offline','--release','-p','plan','--test','zones_prereqs','t3_two_yaw','--','--nocapture')
    Gate 'workspace-final' @('test','--offline','--workspace','--','--nocapture')
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'core-mutants.ps1')
    if($LASTEXITCODE -ne 0){throw 'Core mutation gates failed'}
    if($Footprints) {
        $lines=@(& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'jobs/u1/run-footprints.ps1') 2>&1 | ForEach-Object {"$_"})
        $code=$LASTEXITCODE
        $lines | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $evidence 'footprints.log')
        $caught=[regex]::Matches(($lines -join "`n"),'(?m)^\S+ CAUGHT\s*$').Count
        if($code -ne 0 -or $caught -ne 13){throw "Footprint gate failed: exit=$code caught=$caught"}
        $directory=[regex]::Match(($lines -join "`n"),'U1_FOOTPRINT_OUTPUT=(.+)').Groups[1].Value.Trim()
        if(!$directory){throw 'Missing footprint evidence directory'}
        $copy=Join-Path $evidence 'footprints'
        New-Item -ItemType Directory -Force -Path $copy | Out-Null
        Get-ChildItem -LiteralPath $directory -File | Copy-Item -Destination $copy -Force
        @{caught=$caught;exit=$code;source_directory=$directory;command='powershell -NoProfile -File jobs/u1/run-footprints.ps1';target=$env:CARGO_TARGET_DIR;inputs=(Get-Content -Raw (Join-Path $evidence 'workspace-final.json') | ConvertFrom-Json).inputs} | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 (Join-Path $evidence 'footprints.json')
    }
} finally {
    @{gates=$gates.ToArray();target=$env:CARGO_TARGET_DIR;priority='Normal';offline=$true;red=@('translation-premise','kill11')} | ConvertTo-Json -Depth 7 | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $evidence 'gates.json')
}
