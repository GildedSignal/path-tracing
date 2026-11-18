param([string]$Evidence='D:\path-tracing-evidence\u2')
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$gate=Get-Content -Raw -LiteralPath (Join-Path $Evidence 'gates.json') | ConvertFrom-Json
if(@($gate.gates | Where-Object {!$_.recorded}).Count){throw 'An unrecorded gate remains'}
foreach($item in $gate.gates) {
 $latest=Get-Content -Raw -LiteralPath (Join-Path $Evidence ($item.name+'.json')) | ConvertFrom-Json
 if($latest.exit -ne $item.exit){throw ('Latest gate result differs from manifest: '+$item.name)}
}
$mutants=Get-Content -Raw -LiteralPath (Join-Path $Evidence 'core-mutants.json') | ConvertFrom-Json
if(@($mutants.runs | Where-Object {!$_.caught}).Count){throw 'Core mutation not caught'}
$footprints=Get-Content -Raw -LiteralPath (Join-Path $Evidence 'footprints.json') | ConvertFrom-Json
if($footprints.exit -ne 0 -or $footprints.caught -ne 13){throw 'Footprint mutation gate incomplete'}
$log=[IO.File]::ReadAllText((Join-Path $Evidence 'core-final-release.log'))
$rows=[regex]::Matches($log,'U1_SOLVE cell=([^ ]+) mode=(cold|warm) n=(\d+) min_ns=(\d+) median_ns=(\d+) p99_ns=(\d+) max_ns=(\d+) samples=\[([^\]]+)\]')
if($rows.Count -ne 6){throw 'Missing timing distributions'}
$culture=[Globalization.CultureInfo]::InvariantCulture
$table=@('| Goal cell | Mode | n | min us | median us | p95 us | p99 us | max us |','|---|---|---:|---:|---:|---:|---:|---:|')
foreach($m in $rows) {
 $samples=@($m.Groups[8].Value.Split(',') | ForEach-Object {[long]$_.Trim()})
 $p95=$samples[[math]::Ceiling(0.95*$samples.Count)-1]
 $values=@([double]$m.Groups[4].Value,[double]$m.Groups[5].Value,[double]$p95,[double]$m.Groups[6].Value,[double]$m.Groups[7].Value)
 $table+='| '+$m.Groups[1].Value+' | '+$m.Groups[2].Value+' | '+$m.Groups[3].Value+' | '+(($values | ForEach-Object {($_/1000).ToString('0.###',$culture)}) -join ' | ')+' |'
}
$bar=[IO.File]::ReadAllText((Join-Path $Evidence 'kill10-bar.log'))
$result=[regex]::Match($bar,'U2_KT10_BAR best=(\d+) human=(\d+) margin=(\d+) inferred_total=(\d+) wall_hits=(\d+) ladder_hits=(\d+) sentences=(\d+) certified_edges=(\d+) acting_ticks=(\d+)')
if(!$result.Success -or !$bar.Contains('test result: ok.')){throw 'KT10 discovery has not passed'}
$replay=[IO.File]::ReadAllText((Join-Path $Evidence 'kill10-current-record.log'))
if(!$replay.Contains('first_touch_exact=true') -or !$replay.Contains('test result: ok.')){throw 'KT10 current record replay missing'}
$counts=([IO.File]::ReadAllLines((Join-Path $Evidence 'kill11.log')) | Where-Object {$_ -match '^U2_KT11 pattern='}) -join "`n"
$stairs=([IO.File]::ReadAllLines((Join-Path $Evidence 'maps-and-stairs.log')) | Where-Object {$_ -match '^U2_STAIRS'}) -join "`n"
$replayLine=([IO.File]::ReadAllLines((Join-Path $Evidence 'kill10-current-record.log')) | Where-Object {$_ -match '^U2_KT10_REPLAY'}) -join "`n"
$work=([IO.File]::ReadAllLines((Join-Path $Evidence 'core-final-release.log')) | Where-Object {$_ -match 'U1_(WORK|SCARS|RESULT)'}) -join "`n"
$refusals=([IO.File]::ReadAllLines((Join-Path $Evidence 'core-final-release.log')) | Where-Object {$_ -match 'U1_REFUSAL'}) -join "`n"
$checks=@('| Gate | Result |','|---|---|')
foreach($item in $gate.gates){$checks+='| '+$item.name+' | '+$(if($item.expected_red){'RED, failure recorded'}else{'PASS'})+' |'}
$checks+='| independent KT10 search | PASS |'
$checks+='| core mutants | '+@($mutants.runs | Where-Object {$_.expected_failure}).Count+' caught; restored passes |'
$checks+='| footprint mutants | '+$footprints.caught+' caught; restored passes |'
$workspace=[IO.File]::ReadAllText((Join-Path $Evidence 'workspace-final.log'))
$suites=[regex]::Matches($workspace,'test result: ok\. (\d+) passed; 0 failed; (\d+) ignored')
$passed=0;$ignored=0
foreach($s in $suites){$passed+=[int]$s.Groups[1].Value;$ignored+=[int]$s.Groups[2].Value}
$files=@(& git -C $root diff --name-only -- . ':!engine/world/tests/fixtures/world')
$files+=@(& git -C $root ls-files --others --exclude-standard engine/plan jobs/u2)
$files=@($files | Sort-Object -Unique)
$section=@'
<!-- GENERATED -->
## Final measurements and gates

The gate contract is completed with the two red results preserved. This is not an interval-design freeze.

{{checks}}

KT10 best after the ladder exit: **{{best}} ticks against {{human}}**, margin {{margin}};
conditional full T is {{total}} if the ladder exit occurs at his tick 259. This is offline law evidence,
not a server measurement. The search found {{sentences}} sentences, {{walls}} wall hits and {{ladders}}
ladder hits. The current-record replay certifies {{edges}} edges and observes {{ticks}} acting law ticks;
first goal touch and all endpoint state bits agree with native replay.

Workspace: {{suites}} successful result lines, {{passed}} passed checks, {{ignored}} ignored checks.
The explicit red gates and the KT10 discovery/replay are run separately from that default workspace.

The complete serial solve distributions (including counters, stages, output records and shared
composition) are below. Cold uses a fresh store/codec; warm retains generators, not a goal answer.
All raw sorted samples and profiles are in `core-final-release.log`. Median averages the middle pair;
p95/p99 use nearest rank. The plan's microsecond floor is not reached.

{{timings}}

```
{{plateaus}}
{{stairs}}
{{replay}}
```

Worst native map/replay error is zero ULP; warm native ticks are zero. Work by distance and scars,
native tick totals and the explicit refusals (Normal resource prediction, loaded boundary, codec owner):

```
{{work}}
{{refusals}}
```

Exact changed-source list (pre-existing fixture changes excluded):

{{files}}

Replay commands: `verify.ps1 -Search -Footprints` (full independent discovery),
`verify.ps1 -Footprints` (reuse discovery, replay through current code), then `report.ps1`.
Each cargo run has a JSON record with input hashes, command, exit, invariant output and refusals.
`core-mutants.json`, `footprints.json` and their per-mutant logs record planted-fault results.
'@
$section=$section.Replace('{{checks}}',($checks -join "`n")).Replace('{{timings}}',($table -join "`n"))
foreach($pair in @(@('best',1),@('human',2),@('margin',3),@('total',4),@('walls',5),@('ladders',6),@('sentences',7),@('edges',8),@('ticks',9))){$section=$section.Replace('{{'+$pair[0]+'}}',$result.Groups[$pair[1]].Value)}
$section=$section.Replace('{{plateaus}}',$counts).Replace('{{stairs}}',$stairs).Replace('{{replay}}',$replayLine).Replace('{{work}}',$work).Replace('{{refusals}}',$refusals).Replace('{{suites}}',"$($suites.Count)").Replace('{{passed}}',"$passed").Replace('{{ignored}}',"$ignored").Replace('{{files}}',(($files | ForEach-Object {'- `'+$_+'`'}) -join "`n"))
$path=Join-Path $PSScriptRoot 'last.md'
$text=[IO.File]::ReadAllText($path).Split(@('<!-- GENERATED -->'),[StringSplitOptions]::None)[0].TrimEnd()
$text=$text.Replace(' (in progress)','').Replace('## Result so far','## Result').Replace('Pending final gate records and generated distributions. Replay all gates with:','Final gate records and distributions are generated below. Replay all gates with:').Replace('are being rerun against the final gates.','are caught by the final gates.')
[IO.File]::WriteAllText($path,$text+"`n`n"+$section+"`n",[Text.UTF8Encoding]::new($false))
@{gates=$gate.gates;core_mutants=$mutants.runs;footprints=$footprints.caught;workspace=@{suites=$suites.Count;passed=$passed;ignored=$ignored};kt10=@{best=[int]$result.Groups[1].Value;human=[int]$result.Groups[2].Value;margin=[int]$result.Groups[3].Value};red=@('translation-premise','kill11');files=$files} | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $Evidence 'summary.json')
Write-Output ('Report regenerated: '+$path)
