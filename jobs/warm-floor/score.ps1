param([string]$Series='Measurements',[switch]$Gate)
$ErrorActionPreference='Stop'
$dir='D:/path-tracing-evidence/warm-floor'
$fail=[Collections.Generic.List[string]]::new()
$rows=[Collections.Generic.List[string]]::new()
$checks=[Collections.Generic.List[string]]::new()
$raw=@{}
$values=@{}
function Numbers([string]$text) {
    @($text -split ',' | ForEach-Object {[double]$_.Trim()})
}
function Stats([double[]]$v) {
    if(!$v.Length) {throw 'Missing samples'}
    $s=@($v | Sort-Object)
    $n=$s.Length
    @($s[0],(($s[[int][Math]::Floor(($n-1)/2)]+$s[[int][Math]::Floor($n/2)])/2),
        $s[[int][Math]::Ceiling(0.9*$n)-1],$s[-1])
}
function Row([string]$run,[string]$name,[double[]]$v,[double]$floor,[string]$refusals) {
    $stats=@(Stats $v)
    $us=@($stats | ForEach-Object {($_/1000).ToString('0.###',[cultureinfo]::InvariantCulture)})
    $oom=[Math]::Log10($stats[1]/(1000*$floor)).ToString('0.00',[cultureinfo]::InvariantCulture)
    $rows.Add("| $run | $name | $($v.Length) | $($us -join ' / ') | $refusals | $floor | $oom |")
    $values["$run/$name"]=$stats
}
foreach($run in @('Baseline',$Series)) {
    $commands=@((Get-Content -LiteralPath (Join-Path $dir "$run-commands.log")) -replace '^test real_time_per_command \.\.\. ','')
    $raw[$run]=$commands
    foreach($line in $commands) {
        if($line -match 'REAL_TIME series="([^"]+)" n=(\d+) refused=(\d+).* samples_ns=\[([^\]]+)\]') {
            $name=$Matches[1];$n=[int]$Matches[2];$refused=[int]$Matches[3];$v=@(Numbers $Matches[4])
            if($n -lt 60 -or $v.Count -ne $n){throw "Incomplete command series ($name)"}
            $repeat=$name -match 'repeat'
            $floor=if($repeat){1}else{10}
            Row $run $name $v $floor $refused
            if($run -eq $Series) {
                # The initial unseen-ground preparation is allowed exactly once.
                $samples=if($name -eq 'new goal, from standing') {
                    @($commands -match '^CMD from=standing' | Select-Object -Skip 1 | ForEach-Object {[void]($_ -match ' ns=(\d+)');[double]$Matches[1]})
                } else {$v}
                if($refused){$fail.Add("$refused refusals in $name")}
                if(($samples|Measure-Object -Maximum).Maximum -gt 2000*$floor){$fail.Add("$name exceeds 2x its floor")}
            }
        }
    }
    foreach($kind in 'hot','interleaved','seen') {
        $log=Get-Content -LiteralPath (Join-Path $dir "$run-$kind.log")
        foreach($line in $log) {
            if($line -match 'U2B_WARM cell=\[([^\]]+)\].*samples_ns=\[([^\]]+)\]') {
                Row $run "repeat hot $($Matches[1] -replace ' ','')" @(Numbers $Matches[2]) 1 0
            } elseif($line -match 'U1_SOLVE cell=(\S+) mode=(\w+).*samples=\[([^\]]+)\]') {
                $cell=$Matches[1];$mode=$Matches[2];$v=@(Numbers $Matches[3])
                if($mode -eq 'cold'){Row $run "fresh store $cell" $v 10 0} else {Row $run "repeat interleaved $cell" $v 1 0}
            } elseif($line -match 'SEEN_TRACK_REPEAT.*samples_ns=\[([^\]]+)\]') {
                Row $run 'repeat learned shortcut' @(Numbers $Matches[1]) 0.05 0
            }
        }
    }
}
if($rows.Count -ne 26){throw "Expected 13 complete series per build, found $($rows.Count) rows"}
$end=@{}
foreach($line in $raw[$Series]) {
    if($line -match '^END (from=\w+ goal=\[[^\]]+\]) (T=.*?) profile=') {$end[$Matches[1]]=$Matches[2]}
}
$changed=@(foreach($line in $raw.Baseline) {
    if($line -match '^END (from=\w+ goal=\[[^\]]+\]) (T=.*?) profile=') {
        if($end[$Matches[1]] -cne $Matches[2]) {$Matches[1]}
    }
})
if($changed.Count){$fail.Add("Baseline T or end bits changed for $($changed -join ', ')")}
foreach($cell in '1000000,-61,0','0,-61,1000000','1000000,-61,330000') {
    $ratio=$values["$Series/repeat interleaved $cell"][1]/$values["$Series/repeat hot $cell"][1]
    foreach($temp in 'hot','interleaved'){
        if($values["$Series/repeat $temp $cell"][3] -gt 2000){$fail.Add("Repeat $temp $cell exceeds the 2 us ceiling")}
    }
    $checks.Add("Interleaved/hot repeat median for $cell = "+$ratio.ToString('0.00',[cultureinfo]::InvariantCulture)+'x.')
    if($ratio -gt 2){$fail.Add("Interleaved repeat exceeds twice hot for $cell")}
}
$counters=@((Get-Content -LiteralPath (Join-Path $dir 'Counters-commands.log')) -replace '^test real_time_per_command \.\.\. ','')
foreach($origin in 'standing','moving','repeat'){
    $cmd=@($raw[$Series] -match "^CMD from=$origin ")
    $work=@($counters -match "^WORK from=$origin ")
    if($work.Count -ne $cmd.Count){throw "Missing counter samples for $origin"}
    foreach($field in 'ns','allocated_bytes','law_ticks','kernel_ns','stop_builds','stop_build_ns','stop_lookups','max_bundle_bytes','probes','distinct_phases','x_walks','z_walks') {
        $data=$cmd;$pattern=" $field=(\d+)"
        if($field -in 'probes','distinct_phases'){ $data=$work;$pattern="${field}: (\d+)" }
        if($field -in 'x_walks','z_walks'){ $data=$work;$pattern=if($field -eq 'x_walks'){'walks: \[(\d+),'}else{'walks: \[\d+, (\d+)\]'} }
        if($field -in 'stop_build_ns','stop_lookups','max_bundle_bytes'){ $data=@($counters -match "^LAYOUT from=$origin ") }
        $samples=@(foreach($line in $data){if($line -match $pattern){[double]$Matches[1]}})
        if(!$samples.Count){throw "Missing $field for $origin"}
        $stats=@(Stats $samples)
        $checks.Add("$origin $field = $($stats -join ' / ')")
        if($field -eq 'law_ticks' -and ($origin -ne 'standing' -or $samples.Length -gt 1)){
            $paid=@($samples | Select-Object -Skip ([int]($origin -eq 'standing')) | Where-Object {$_ -gt 0})
            if($paid.Count){$fail.Add("$($paid.Count) $origin commands prepared native ticks after the first")}
        }
        if($field -eq 'max_bundle_bytes' -and $stats[3] -gt 1024){$fail.Add("Bundle exceeds 1 KiB for $origin")}
        if($field -in 'x_walks','z_walks' -and ($stats[0] -ne 1 -or $stats[3] -ne 1)){$fail.Add("$origin $field differs from one")}
        if($field -eq 'allocated_bytes' -and $origin -eq 'repeat' -and $stats[3]){$fail.Add('Resident repeated solve allocated')}
    }
}
$out=@('## Complete public solve distributions','','All times are microseconds. The four values are minimum, median, nearest-rank p90, and maximum. The gap is log10(median / floor). The fresh-store rows use the new-direction floor as a comparator and include first preparation. The seen-track floor is the high end of the 20-50 ns estimate. The two moving-command rows include refused commands.','',
'| Build | Series | n | Min / median / p90 / max (us) | Refused | Floor (us) | Gap (orders) |',
'|---|---|---:|---|---:|---:|---:|')+$rows.ToArray()+@('','## Cost and work','','Counter rows have minimum / median / p90 / maximum. Allocation bytes come from the normal build. Probes, phase identities, chart walks and stop build times come from a separate instrumented pass. A chart walk crosses binary64 charts between its endpoints. Full end-state bit vectors, sampled times, refusal causes and counters are preserved per command in the logs.','')+@($checks|ForEach-Object {"- $_"})+@('','## Unmet gates','')+@($fail|ForEach-Object {"- $_"})
[IO.File]::WriteAllLines((Join-Path $dir 'score.md'),[string[]]$out)
$report=Join-Path $PSScriptRoot 'last.md'
$marker='<!-- Generated from the public solve and counter logs -->'
$text=[IO.File]::ReadAllText($report)
$before=($text -split [regex]::Escape($marker),2)[0].TrimEnd()
$after=if($text -match '(?m)^STATUS: .+ -and $fail.Count){throw "$($fail.Count) acceptance checks failed; see $dir/score.md"}
) { "`n`n"+$Matches[0].TrimEnd() } else {''}
[IO.File]::WriteAllText($report,$before+"`n`n"+$marker+"`n`n"+($out -join "`n")+$after+"`n",[Text.UTF8Encoding]::new($false))
$out
if($Gate -and $fail.Count){throw "$($fail.Count) acceptance checks failed; see $dir/score.md"}
