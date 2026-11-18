param([string]$Evidence = 'D:\path-tracing-evidence\u1')
$ErrorActionPreference = 'Stop'
$manifest=Get-Content -Raw -Encoding UTF8 (Join-Path $Evidence 'manifest.json') | ConvertFrom-Json
if (!$manifest.kernel_passed) {throw 'U1 gates are not all green; preserve the failing run record.'}
$log=[IO.File]::ReadAllText((Join-Path $Evidence 'u1.log'))
$t3=[IO.File]::ReadAllText((Join-Path $Evidence 't3.log'))
$timings=[regex]::Matches($log,'U1_SOLVE cell=([^ ]+) mode=(cold|warm) n=(\d+) min_ns=(\d+) median_ns=(\d+) p99_ns=(\d+) max_ns=(\d+)')
if ($timings.Count -ne 6) {throw 'Missing cold/warm million-line distributions'}
$table=($timings | ForEach-Object {
    $m=$_
    '| '+$m.Groups[1].Value+' | '+$m.Groups[2].Value+' | '+$m.Groups[3].Value+' | '+((4..7 | ForEach-Object { ([double]$m.Groups[$_].Value/1000).ToString('0.###',[Globalization.CultureInfo]::InvariantCulture) }) -join ' | ')+' |'
}) -join "`n"
$profiles=[regex]::Matches($log,'U1_PROFILE cell=([^ ]+) stage=([^ ]+) min_ns=(\d+) median_ns=(\d+) p99_ns=(\d+) max_ns=(\d+)')
$profileTable=($profiles | ForEach-Object {
    $m=$_
    '| '+$m.Groups[1].Value+' | '+$m.Groups[2].Value+' | '+((3..6 | ForEach-Object { ([double]$m.Groups[$_].Value/1000).ToString('0.###',[Globalization.CultureInfo]::InvariantCulture) }) -join ' | ')+' |'
}) -join "`n"
$results=[regex]::Matches($log,'U1_RESULT cell=([^ ]+) ticks=(\d+) cold_law_ticks=(\d+) warm_law_ticks=0 reads=([^\r\n]+)')
$resultTable=($results | ForEach-Object { '| '+$_.Groups[1].Value+' | '+$_.Groups[2].Value+' | '+$_.Groups[3].Value+' |' }) -join "`n"
$runs=($manifest.runs | ForEach-Object { '| '+$_.name+' | '+$_.exit_code+' | '+$_.passed+' |' }) -join "`n"
$work=([regex]::Matches($log,'(?m)^U1_WORK .+$') | ForEach-Object Value) -join "`n"
$scars=([regex]::Matches($log,'(?m)^U1_SCARS .+$') | ForEach-Object Value) -join "`n"
$native=([regex]::Matches($t3,'(?m)^T3_NATIVE .+$') | ForEach-Object Value) -join "`n"
$singleYaw=[regex]::Match($t3,'(?m)^T3_SINGLE_YAW .+$').Value
if (!$singleYaw) {throw 'Missing full-domain single-yaw exclusion certificate'}
$counter=[regex]::Match($log,'U1_STREAM law_ticks=(\d+) witness_states=(\d+) hits=0')
$control=[regex]::Match($log,'U1_CONTROL release_tick=(\d+) arrival_tick=(\d+) goal_touched=true')
$margin=[regex]::Match($log,'U1_MARGIN vertical_gap=([^\s]+)')
$warmOblique=$timings | Where-Object { $_.Groups[1].Value -eq '1000000,-61,330000' -and $_.Groups[2].Value -eq 'warm' }
$ratio=([double]$warmOblique.Groups[5].Value/5000).ToString('0.0',[Globalization.CultureInfo]::InvariantCulture)
$text=[IO.File]::ReadAllText((Join-Path $PSScriptRoot 'report-template.md'))
$values=@{timings=$table;profiles=$profileTable;results=$resultTable;runs=$runs;work=$work;scars=$scars;native=$native;single_yaw=$singleYaw;counter_ticks=$counter.Groups[1].Value;vertical_gap=$margin.Groups[1].Value;release=$control.Groups[1].Value;arrival=$control.Groups[2].Value;floor_ratio=$ratio}
foreach ($key in $values.Keys) {$text=$text.Replace('{{'+$key+'}}',[string]$values[$key])}
if ($text.Contains('{{')) {throw 'Unexpanded report value'}
[IO.File]::WriteAllText((Join-Path $PSScriptRoot 'last.md'),$text+"`n",[Text.UTF8Encoding]::new($false))
Write-Output 'Regenerated jobs/u1/last.md from the passing run record.'
