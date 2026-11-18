# Regenerate the builder's final record from recorded distributions and gates.
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$evidence='D:\path-tracing-evidence\u2b'
$gates=Get-Content -Raw (Join-Path $evidence 'gates-s203.json') | ConvertFrom-Json
if(@($gates.gates | Where-Object {!$_.recorded}).Count){throw 'An unexpected gate result remains'}
$cost=[IO.File]::ReadAllText((Join-Path $evidence 's203-cost-final.log'))
$kt10=[IO.File]::ReadAllText((Join-Path $evidence 's203-kill10-current.log'))
$out=[Collections.Generic.List[string]]::new()
function Line([string]$s='') {$out.Add($s)}
function Us([string]$n) {([decimal]$n/1000).ToString('0.###',[Globalization.CultureInfo]::InvariantCulture)}
Line '# U2b: S203 builder record'
Line
Line 'Both requested parts are implemented on this branch. The adopted point-or-hold'
Line 'gate passes; the warm full solve asserts zero PlayerState copies and zero heap'
Line 'allocations. The ~5 us floor remains missed, with distributions and the remaining'
Line 'work below. This is a builder result, awaiting the independent judge; no merge or flight claim.'
Line
Line '## Representation and decisions'
Line
Line '1. The law declares minor_horizontal_collision as its non-monotone field and'
Line '   reports Free/Clamp/Hold for each selected axis pass. CONTACT_CONSTANTS and'
Line '   approach/hit comparisons are under LAW_HASH. Collision arithmetic, visitation'
Line '   order, early zeroing and step selection are unchanged. Recording wrappers forward the report.'
Line '2. A clamp is a point per flag; a hold retains the earlier exact addition map.'
Line '   Silent clamps remain min-maps inside free pieces. Cuts come from law-owned'
Line '   face comparisons and exact-map bisection; no sample grid enters learn().'
Line '   Native samples are an independent judge. The face inventory uses the law''s actual'
Line '   per-tick read-footprint hull, replacing the earlier fixed two-cell positional margin.'
Line '   Resource intervals enclose both flag tails;'
Line '   identical point images merge. The native second-wall check closes the hold witness.'
Line '   Identity follows the law''s position-update flag, never an unchanged rounded'
Line '   endpoint. A 65-ULP native witness first failed at its odd neighbour, then passed'
Line '   with the actual addition operand retained. Explicit free-axis metadata is preserved.'
Line '3. EntryTemplate stores the ordered binary64 lattice on the free axis, with'
Line '   support relative to floor(body position). Resources travel as use intervals'
Line '   with exact numeric entry guards; broader unproved numeric reuse refuses.'
Line '   Yaw, pitch and old input values were removed from EntryTemplate, motor_words'
Line '   and the legacy pattern start index. Point route/wave entries and queued demands'
Line '   use the same canonical key, with numeric guards separate; raw point identity'
Line '   retains every witness bit. The native entry regression first failed, then passed;'
Line '   two-word alias composition also passes. Word aim arguments remain word data.'
Line '   Tests reuse a changed aim and still distinguish a one-ULP velocity change.'
Line '4. learn_family publishes the computed pieces with timed endpoint-hull read'
Line '   certificates for every declared flag. Legacy learn retains its exact-point'
Line '   API for arbitrary conditional programs. Held producers are reduced from the'
Line '   toolkit predicates, rather than assuming that their row count is one.'
Line '5. Piece exits are maps or point IDs; consume checks key, lattice, numeric guard'
Line '   and certificate, and returns the exit set and T range. The backward wave'
Line '   refines input intervals by exact preimage and applies min-plus to families'
Line '   and points until stable, including cross-axis singleton intersections.'
Line '   Alternative images retain [min,max]; an unknown branch leaves the upper open.'
Line '   Numeric-use intervals that vary cannot enter a downstream exact numeric guard.'
Line '   The legacy single-state route refuses unresolved family sets rather than'
Line '   choosing a representative. The set consumer exposes them to the caller.'
Line '6. The two-word family test checks pulled-back T, shifted exact readout, dead-aim'
Line '   reuse, distinct velocity, local certificate invalidation and no acting replay.'
Line '7. A terminal joint is learned from the face once in JointStore. It holds exact'
Line '   entry rectangles with constant T and next word, per floor label/gait/phase/'
Line '   floating chart and vertical face geometry. Interior charts share across'
Line '   different integer goals; binade boundaries retain their exact edge class.'
Line '   Warm terminal lookup performs no control search or inverse bisection. A test'
Line '   changes B, proves no new face table was built, and compares with a fresh solve.'
Line '   The first changed goal can compile cold parity graphs; warm zero is measured'
Line '   after these arithmetic graphs exist, independently of terminal-table reuse.'
Line '8. Player-state ownership is replaced by Arc phase handles plus exact readouts.'
Line '   Explicit materialize() is at execution/test boundaries. Inline lists spill'
Line '   on overflow without a semantic cap. The allocator counts alloc, alloc_zeroed'
Line '   and realloc; the state type counts Clone. Negative controls catch both.'
Line '   Inline iteration disarms ownership before a destructor can unwind; release'
Line '   and debug tests ensure no value is dropped twice after partial consumption.'
Line '9. The judge speed path: shared cached parity/binade graphs and streamed affine'
Line '   runs; borrowed hulls through the same Goal predicate; bound-ordered incumbents;'
Line '   no production re-fold assert; streamed boundary events reuse final readouts;'
Line '   the remaining terminal inverse is stored as face preimages, not goal answers.'
Line '   Query-specific run extents are still reconstructed: no claim of a 5 us implementation.'
Line '10. Bearing counts use exact 2x2 determinants, then an exact integer slice at'
Line '    the continuous lower bound. Thin strips or checked-width overflow retain'
Line '    the proved floor-sum fallback: blindly checking a fixed neighbour radius'
Line '    could discard a feasible optimum. The finite enumeration oracle still passes.'
Line '11. Generator label serialization reuses scratch space. Raw-model cache hits'
Line '    compare all native fields bit-exactly; shape-owner caches retain ownership'
Line '    to prevent address reuse. Canonical class IDs remain content-based.'
Line
Line '## Frozen gates and evidence'
Line
Line 'Before freezing, the native oracle measured ice=2 and lane=2 at 2^6, 2^10 and'
Line '2^18; every other expected count agreed. See s203-native-oracle.log and'
Line 'jobs/u2b/s203-oracle-counts.txt. Raw Free/Clamp/Hold counts also disclose silent'
Line 'clamps; the adopted piece partition retains those inside free min-maps.'
Line
Line 'Frozen new gate SHA256: B78EAAF3A53DE7B0A70E1A2F3DA12B5246437962F7909000301309190468FF82.'
Line 'Its first run (s203-first-gate.log) passed. Counts are 5/7/7/8/12/2/2 for gap270,'
Line 'gap275.625, gap270 z=1, step270, step275.625, ice and lane. The gate covers all'
Line '2^6..2^18 grids, +/-8 ULP cut neighbours, 256 ULP hold endpoints, exact exit'
Line 'membership, resource enclosure, native pin proofs and second-wall convergence.'
Line 'The witness remains exact: clamp X=40592cccccc00000; hold X=40592cccccbfffff.'
Line
Line 'Old pieces SHA256: 5C04C53881D0C797F3FE9EEC8AFB831B99781F5543E3790FC365D4565F98034D.'
Line 'The old pieces, cut-neighbour pin witness, KT11 and translation tests remain'
Line 'unchanged recorded reds of the old object. The coarse-grid pin-erasure diagnostic'
Line 'actually passes; it missed the hold, and is not misreported as red. Review also'
Line 'checks the original full-solve cost assertions and footprint mutation bodies.'
Line
Line '| Gate | Exit | Expected red | Correctly recorded | Reason |'
Line '|---|---:|---|---|---|'
foreach($g in $gates.gates) {Line "| $($g.name) | $($g.exit) | $([bool]$g.expected_red) | $($g.recorded) | $($g.reason) |"}
Line
Line 'All logs, input hashes, invariant output and refusals are in D:\path-tracing-evidence\u2b.'
Line 'The pre-freeze oracle is now an explicit ignored diagnostic, superseded in the'
Line 'normal loop by the frozen pin regression; its original run remains archived.'
Line 'The footprint driver reuses a successful complete workspace baseline only if its'
Line 'entire engine input hash set still matches. All mutation bodies and assertions run.'
Line 'The full workspace includes the old frozen red. The independent workspace excludes'
Line 'only that obsolete gate and the already-run pre-freeze oracle diagnostic; new checks stay enabled.'
$core=Get-Content -Raw (Join-Path $evidence 'core-mutants.json') | ConvertFrom-Json
$coreCaught=@($core.runs | Where-Object {$_.expected_failure -and $_.caught}).Count
$footprintCaught=($gates.gates | Where-Object {$_.name -eq 's203-footprint-mutants'}).caught
Line ("All $footprintCaught footprint mutants and all $coreCaught core mutants are caught by running assertions.")
Line 'The actual cached-answer bypass fails through effect counters; native-tick and bundle-hit'
Line 'mutants also fail. Source bytes are restored and hash-checked. No trace-label-only substitution.'
Line
Line '```text'
Line (($kt10 -split "`r?`n" | Where-Object {$_ -like 'U2_KT10_REPLAY*'} | Select-Object -First 1))
Line '```'
Line
Line '## Warm million line: distributions, not averages'
Line
Line 'Microseconds; n=100 per goal. Median uses the two central samples; p99 is nearest rank.'
Line 'The measured boundary is the complete public solve, including output and profile clocks.'
Line 'Verification materializes states after the measured boundary. Warm native ticks are zero.'
Line
Line '| B cell | Min us | Median us | p99 us | Max us | Copies max | Allocations max | Median / 5 us |'
Line '|---|---:|---:|---:|---:|---:|---:|---:|'
$pattern='U2B_WARM cell=(\[[^\]]+\]) n=100 ns_min=(\d+) ns_median=(\d+) ns_p99=(\d+) ns_max=(\d+) copies_max=(\d+) allocations_max=(\d+)'
foreach($m in [regex]::Matches($cost,$pattern)) {Line "| $($m.Groups[1].Value) | $(Us $m.Groups[2].Value) | $(Us $m.Groups[3].Value) | $(Us $m.Groups[4].Value) | $(Us $m.Groups[5].Value) | $($m.Groups[6].Value) | $($m.Groups[7].Value) | $(([decimal]$m.Groups[3].Value/5000).ToString('0.###',[Globalization.CultureInfo]::InvariantCulture)) |"}
Line
Line '| B cell | Stage | Min us | Median us | p99 us | Max us |'
Line '|---|---|---:|---:|---:|---:|'
$pattern='U2B_PROFILE cell=(\[[^\]]+\]) stage=(\w+) ns_min=(\d+) ns_median=(\d+) ns_p99=(\d+) ns_max=(\d+)'
foreach($m in [regex]::Matches($cost,$pattern)) {Line "| $($m.Groups[1].Value) | $($m.Groups[2].Value) | $(Us $m.Groups[3].Value) | $(Us $m.Groups[4].Value) | $(Us $m.Groups[5].Value) | $(Us $m.Groups[6].Value) |"}
Line
Line 'The floor is missed. Remaining entry cost is motor/label lookup and exact transient'
Line 'readouts. Axis bearing still enters the checked floor-sum fallback (30 quotient'
Line 'queries in U1_WORK); the oblique solve takes the determinant path. Inverse cost'
Line 'is phase enumeration, repeated exact readouts, approach-run reconstruction and'
Line 'linear terminal-row lookups. Output still evaluates envelopes and walks binades'
Line 'for boundary events, although those event walks now also supply final coordinates.'
Line 'The oblique excess is therefore not reduced to the expected 1-2 us second-axis'
Line 'crossing cost. The tables show the larger inverse/output work still present;'
Line 'no claim that the extra work is required by nature, or that the floor was reached.'
Line
Line '## Files and exact replay'
Line
$previousPreference=$ErrorActionPreference
try {
    $ErrorActionPreference='Continue'
    $tracked=@(& git -C $root -c core.quotepath=false diff fef8c6ed0f287aef829a6b0ce63d8f307cec5032 --name-only 2>$null);$trackedExit=$LASTEXITCODE
    $untracked=@(& git -C $root -c core.quotepath=false ls-files --others --exclude-standard 2>$null);$untrackedExit=$LASTEXITCODE
} finally {$ErrorActionPreference=$previousPreference}
if($trackedExit -ne 0 -or $untrackedExit -ne 0){throw 'Cannot enumerate changed files'}
$changed=$tracked+$untracked
foreach($file in ($changed | Where-Object {$_ -notmatch '^engine/world/tests/fixtures/'} | Sort-Object -Unique)) {Line ('- `'+$file+'`')}
Line
Line 'The integrator saved the earlier implementation as 95552dc during the usage pause.'
Line 'The final native-footprint inventory, aim-entry and unwind ownership repairs'
Line 'are additional working-tree changes, with their regressions and final gate rerun.'
Line 'File changes above are measured from fef8c6e, the original U2 base.'
Line
Line 'Pre-existing MCA edits and backup files are untouched and hash-checked. No launcher,'
Line 'game, server, client, network or sub-agent was used by this builder. The builder'
Line 'made no commit, merge or push. All runs'
Line 'are offline at Normal priority with target D:\path-tracing-target\u2b.'
Line
Line '```powershell'
Line 'powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2b/verify.ps1'
Line 'powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2b/report.ps1'
Line 'powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2b/finalize.ps1'
Line '```'
Line
Line 'Run records use content hashes; final-inputs-s203.json maps replay inputs to'
Line 'content-addressed copies. Archived older records retain the original blocker'
Line 'and first failures; this file states the adopted object and current results.'
Line
Line 'STATUS: DONE'
[IO.File]::WriteAllLines((Join-Path $PSScriptRoot 'last.md'),$out,[Text.UTF8Encoding]::new($false))
Write-Output 'U2B_REPORT regenerated=true status=DONE'
