param([string]$Evidence='D:\path-tracing-evidence\u2')
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$env:PATH='D:\rust\cargo\bin;'+$env:PATH
$env:RUSTUP_HOME='D:\rust\rustup'
$env:CARGO_HOME='D:\rust\cargo'
$env:CARGO_TARGET_DIR='D:\path-tracing-target\u2'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass='Normal'
New-Item -ItemType Directory -Force $Evidence | Out-Null
$records=[Collections.Generic.List[object]]::new()
function Check([string]$name,[bool]$fail) {
    $old=$ErrorActionPreference
    try {
        $ErrorActionPreference='Continue'
        $out=@(& cargo test --offline --release -p plan --test zones_prereqs u1_kill_test_8 -- --nocapture 2>&1 | ForEach-Object {"$_"})
        $code=$LASTEXITCODE
    } finally {$ErrorActionPreference=$old}
    $out | Set-Content -Encoding UTF8 -LiteralPath (Join-Path $Evidence "$name.log")
    $ok=if($fail){$code -ne 0 -and ($out -join "`n").Contains('test result: FAILED')}else{$code -eq 0}
    $records.Add(@{name=$name;exit=$code;caught=$ok;expected_failure=$fail;command='cargo test --offline --release -p plan --test zones_prereqs u1_kill_test_8 -- --nocapture'})
    Write-Output "$name passed=$ok"
    if(!$ok){throw "$name did not produce the expected test result"}
}
$solve=Join-Path $root 'engine/plan/src/solve.rs'
$store=Join-Path $root 'engine/plan/src/store.rs'
$original=[IO.File]::ReadAllBytes($solve)
$originalStore=[IO.File]::ReadAllBytes($store)
$source=[IO.File]::ReadAllText($solve)
$site='let Kit::BareLegs=kit;'
if(!$source.Contains($site)){throw 'Mutation site missing'}
# A real answer bypass: retain the cold uniform answer and return its composed
# words directly on the next query, without reading the store or corridor.
# Stage labels and physical motion stay correct on unchanged uniform ground.
$shortcut=@'
thread_local! {static PLANTED:std::cell::RefCell<Option<SolveResult>>=std::cell::RefCell::new(None);}
if let Some(answer)=PLANTED.with(|p|p.borrow().clone()) {return answer;}
'@
$ticks=@'
let mut discarded=from.clone();
for _ in 0..1000 {law::try_tick(&mut discarded,source,&law::Input::default()).unwrap();}
'@
Push-Location (Join-Path $root 'engine')
try {
    Check 'core-restored-before' $false
    foreach($m in @(@{name='real-shortcut';text=$shortcut},@{name='uncounted-ticks';text=$ticks})) {
        try {
            $mutated=$source.Replace($site,$m.text+"`n"+$site)
            if($m.name -eq 'real-shortcut') {
                $return='SolveResult {outcome:PlanOutcome::Ready(Plan {ticks,words,end,bracket:Bracket {lb:bracket.lb,ub:Some(ticks)},label,deadline:deadline_ticks,lattice_cost,thinking}),stages,explanation:None}'
                if(!$mutated.Contains($return)){throw 'Missing cached-answer mutation site'}
                $mutated=$mutated.Replace($return,'{let answer='+$return+';PLANTED.with(|p|*p.borrow_mut()=Some(answer.clone()));answer}')
            }
            [IO.File]::WriteAllText($solve,$mutated)
            Check $m.name $true
        } finally {[IO.File]::WriteAllBytes($solve,$original)}
    }
    try {
        $text=[IO.File]::ReadAllText($store)
        $s='let result=inner.bundles.get(&(label,motor,program,pair[0],pair[1])).cloned();'
        if(!$text.Contains($s)){throw 'Bundle mutation site missing'}
        [IO.File]::WriteAllText($store,$text.Replace($s,'let result:Option<Arc<WordBundle>>=None;'))
        Check 'missing-bundle' $true
    } finally {[IO.File]::WriteAllBytes($store,$originalStore)}
    Check 'core-restored-after' $false
} finally {
    Pop-Location
    [IO.File]::WriteAllBytes($solve,$original)
    [IO.File]::WriteAllBytes($store,$originalStore)
    @{runs=$records.ToArray();inputs=@{solve=(Get-FileHash $solve).Hash;store=(Get-FileHash $store).Hash};worst_margin=@{warm_native_ticks=0;corridor_queries=1;bundle_hits=1};refusals=@()} | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 (Join-Path $Evidence 'core-mutants.json')
}
