# Replay the S203 object, followed by the untouched recorded old-object reds.
$ErrorActionPreference='Stop'
$evidence='D:\path-tracing-evidence\u2b'
$records=[Collections.Generic.List[object]]::new()
function Gate([string]$name,[string[]]$arguments,[string]$red='') {
    & (Join-Path $PSScriptRoot 'run.ps1') -Name $name -CargoArgs $arguments
    $r=Get-Content -Raw -LiteralPath (Join-Path $evidence ($name+'.json')) | ConvertFrom-Json
    $log=[IO.File]::ReadAllText((Join-Path $evidence ($name+'.log')))
    $ok=if($red){$r.exit -ne 0 -and $log.Contains('test result: FAILED') -and $log.Contains('panicked at')}else{$r.exit -eq 0 -and $log.Contains('test result: ok.')}
    $records.Add(@{name=$name;expected_red=[bool]$red;recorded=$ok;exit=$r.exit;reason=$red})
    Write-Output "U2B_GATE $name recorded=$ok exit=$($r.exit)"
}
try {
    Gate 's203-u2-final' @('test','--offline','--release','-p','plan','--test','u2','--','--nocapture','--test-threads=1','--skip','u2_pieces_computed_and_closed','--skip','u2_contact_oracle_before_freeze')
    Gate 's203-pieces-old-object' @('test','--offline','--release','-p','plan','--test','u2','u2_pieces_computed_and_closed','--','--nocapture') 'S188 erasing-pin/event-only object superseded by adopted S203 point-or-hold contact partition; frozen assertions unchanged'
    Gate 's203-pin-witness-old-object' @('test','--offline','--release','-p','plan','--test','u2','u2_pin_cut_neighbourhood_witness','--','--ignored','--nocapture') 'The native clamp/hold pair still falsifies the old erasing-pin premise; new gate preserves both exact values'
    Gate 's203-kill11-old-object' @('test','--offline','--release','-p','plan','--test','u2','u2_kill11','--','--ignored','--nocapture') 'Old input-signature plateaus grow under refinement; S203 carries flags in the output image'
    Gate 's203-translation-old-object' @('test','--offline','--release','-p','plan','--test','u2','u2_translation_premise','--','--ignored','--nocapture') 'Rounded addition is monotone, not a translation; exact maps retain parity'
    Gate 's203-kill10-current' @('test','--offline','--release','-p','plan','--test','u2','u2_kill10_discovered_program_current_record','--','--ignored','--nocapture')
    Gate 's203-core-final' @('test','--offline','--release','-p','plan','--test','zones_prereqs','--','--nocapture','--test-threads=1')
    Gate 's203-maps-stairs' @('test','--offline','--release','-p','plan','--lib','--test','u2_stairs','--','--nocapture')
    Gate 's203-workspace-full' @('test','--offline','--workspace','--','--nocapture') 'Workspace contains the frozen S188 old-object red; run independent workspace with only that test excluded'
    Gate 's203-workspace-independent' @('test','--offline','--workspace','--','--nocapture','--skip','u2_pieces_computed_and_closed','--skip','u2_contact_oracle_before_freeze')
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'core-mutants.ps1')
    $records.Add(@{name='s203-core-mutants';recorded=($LASTEXITCODE -eq 0);exit=$LASTEXITCODE})
    $lines=@(& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'footprints.ps1') 2>&1 | ForEach-Object {"$_"})
    $code=$LASTEXITCODE
    $lines | Set-Content -Encoding utf8 (Join-Path $evidence 's203-footprints.log')
    $caught=[regex]::Matches(($lines -join "`n"),'(?m)^\S+ CAUGHT\s*$').Count
    $records.Add(@{name='s203-footprint-mutants';recorded=($code -eq 0 -and $caught -eq 13);exit=$code;caught=$caught})
    $lines | Select-Object -Last 22
    Gate 's203-cost-final' @('test','--offline','--release','-p','plan','--test','u2b_cost','--','--nocapture','--test-threads=1')
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'review.ps1')
    $records.Add(@{name='s203-review';recorded=($LASTEXITCODE -eq 0);exit=$LASTEXITCODE})
} finally {
    @{gates=$records.ToArray();target='D:\path-tracing-target\u2b';priority='Normal';offline=$true} | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 (Join-Path $evidence 'gates-s203.json')
}
if($records.Where({!$_.recorded}).Count){exit 1}
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'finalize.ps1')
exit $LASTEXITCODE
