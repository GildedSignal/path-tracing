$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence='D:\path-tracing-evidence\u2b'
$original=(& git -C $root show HEAD:engine/plan/tests/u2.rs) -join "`n"
$current=[IO.File]::ReadAllText((Join-Path $root 'engine/plan/tests/u2.rs')).Replace("`r`n","`n")
$assertions=@()
foreach($name in @('u2_translation_premise','u2_kill11_refined_bisection_diagnostic')) {
    $pattern='(?ms)^fn '+$name+'\(\) \{.*?^\}'
    $a=[regex]::Match($original,$pattern).Value
    $b=[regex]::Match($current,$pattern).Value
    if(!$a -or !$b -or $a -cne $b){throw "Frozen old assertion changed: $name"}
    $assertions+=@{name=$name;byte_equal_after_newline_normalization=$true}
}
$pieces=(Get-FileHash (Join-Path $root 'engine/plan/tests/u2b/pieces.rs')).Hash
if($pieces -ne '5C04C53881D0C797F3FE9EEC8AFB831B99781F5543E3790FC365D4565F98034D'){throw 'Frozen pieces test changed'}
$pins=(Get-FileHash (Join-Path $root 'engine/plan/tests/u2b/pins_point_hold.rs')).Hash
if($pins -ne 'B78EAAF3A53DE7B0A70E1A2F3DA12B5246437962F7909000301309190468FF82'){throw 'Frozen S203 pins test changed'}
$baseRecord=Get-Content -Raw (Join-Path $evidence 'cost-baseline.json') | ConvertFrom-Json
$baseCost=($baseRecord.inputs | Where-Object {$_.path -eq 'engine\plan\tests\u2b_cost.rs' -or $_.path -eq 'engine/plan/tests/u2b_cost.rs'} | Select-Object -First 1).sha256
$costFile=Join-Path $root 'engine/plan/tests/u2b_cost.rs'
$archiveFile=Join-Path (Join-Path $evidence 'inputs') $baseCost
if(!(Test-Path -LiteralPath $archiveFile)) {
    $previous=Get-Content -Raw (Join-Path $evidence 'final-inputs.json') | ConvertFrom-Json
    $archiveFile=($previous.inputs | Where-Object {$_.path -like '*u2b_cost.rs'} | Select-Object -First 1).blob
}
if(!(Test-Path -LiteralPath $archiveFile)){throw 'Missing archived cost assertion source'}
if(Test-Path -LiteralPath $archiveFile) {
    $baseText=[IO.File]::ReadAllText($archiveFile).Replace("`r`n","`n")
    $costText=[IO.File]::ReadAllText($costFile).Replace("`r`n","`n")
    foreach($name in @('u2_warm_zero_copies_and_allocations','u2b_counters_detect_work')) {
        $pattern='(?ms)^fn '+$name+'\(\) \{.*?^\}'
        if(![regex]::Match($baseText,$pattern).Success -or [regex]::Match($baseText,$pattern).Value -cne [regex]::Match($costText,$pattern).Value){throw "Cost gate changed: $name"}
    }
}
$parseErrors=@();$tokens=@()
Get-ChildItem $PSScriptRoot -Filter '*.ps1' | ForEach-Object {
    [Management.Automation.Language.Parser]::ParseFile($_.FullName,[ref]$tokens,[ref]$parseErrors) | Out-Null
    if($parseErrors){throw ($parseErrors | Out-String)}
}
$previous=$ErrorActionPreference; $ErrorActionPreference='Continue'
try {$diff=@(& git -C $root diff --check 2>&1 | ForEach-Object {"$_"});$diffExit=$LASTEXITCODE} finally {$ErrorActionPreference=$previous}
if($diffExit -ne 0){throw ($diff -join "`n")}
$initial=Get-Content -Raw (Join-Path $evidence 'premises.json') | ConvertFrom-Json
$fixtures=@($initial.inputs | Where-Object {$_.path -like '*.mca'} | ForEach-Object {
    $now=(Get-FileHash (Join-Path $root $_.path)).Hash
    if($now -ne $_.sha256){throw ('Pre-existing fixture changed: '+$_.path)}
    @{path=$_.path;sha256=$now;unchanged=$true}
})
$footprint=[IO.File]::ReadAllText((Join-Path $root 'jobs/1.2-footprints/verify.ps1'))
$adapter=[IO.File]::ReadAllText((Join-Path $PSScriptRoot 'footprints.ps1'))
$bodyPattern='(?ms)^function Mutant.*?^\}'
if([regex]::Match($footprint,$bodyPattern).Value -cne [regex]::Match($adapter,$bodyPattern).Value){throw 'Footprint mutation function changed'}
$mutationsA=[regex]::Matches($footprint,'(?m)^    Mutant .+$') | ForEach-Object {$_.Value}
$mutationsB=[regex]::Matches($adapter,'(?m)^    Mutant .+$') | ForEach-Object {$_.Value}
if(($mutationsA -join "`n") -cne ($mutationsB -join "`n")){throw 'Footprint mutation bodies changed'}
@{old_assertions=$assertions;pieces_sha256=$pieces;pins_sha256=$pins;fixture_hashes=$fixtures;scripts_parse=$true;diff_check=$true;footprint_mutation_lines_unchanged=$true} | ConvertTo-Json -Depth 7 | Set-Content -Encoding utf8 (Join-Path $evidence 'review.json')
Write-Output 'U2B_REVIEW frozen_assertions=true fixture_preserved=true scripts_parse=true diff_check=true footprint_mutants_unchanged=true'
