# P1: observed-start transient (T1)

Consumer: 3.2d-3 generator store; offline Peaceful fixture, mobs off. No game, server or client launched. Target: `D:\path-tracing-target\3.2d-3-P1`; evidence: `D:\path-tracing-evidence\3.2d-3-P1`.

## Result

- Native observed bits with all-known mask: PASS. Recorded fixture with its mask: PASS, refused `unknown word` as required.
- Native preparation: `transient_ticks=263`, `cycle_period=12`; cycle increment `(x,z)=(0.0,4.276060366589263)` blocks. One prepared entry; read certificate contains 23,650 cell reads.
- Nine starts (three tracks × x = 300.5, 304.5, 315.25; ticks = 0, 77, 1543 respectively; the specified two resource bars): recurrence key and floor label identical, 9/9 memo hits; prepared entries = 1. Negative on-ground control missed: PASS.
- Hot lookup/application: zero counted tick calls and zero counted world reads for every start. Debug-build duration over nine starts: min 4,900 ns, median 5,200 ns, max 5,700 ns (`t1.log`); timings are a small sample, not a latency guarantee.
- Fresh replay comparisons: all nine passed exact bits for x/y/z, velocity x/y/z, on-ground, sprinting, jump delay, fall distance, translated supporting block, and tick. No numerical tolerance.
- Baseline (unmerged plain reference): 3/3 passed, cycle 12 from tick 263 on each straight; `baseline.log`.
- Workspace: PASS (`workspace.log`). Footprint gate: 13/13 CAUGHT, restored checks PASS and pinned source hashes restored (`footprints.log`); detailed mutant logs in `D:\path-tracing-evidence\footprints-2a01459b2b2249959bda43135024872d`. Prerequisite verifier: PASS (`prerequisites.log`), manifest `D:\path-tracing-evidence\3.2d-3\T1\manifest.json`.
- Refusal: the intentionally incomplete recorded state is refused as above. No unexpected refusal or failing assertion. Worst exactness margin is equality of bits; no nonzero numeric margin applies.

The two fixture region files were already modified, with `.bak-20261114` files present, before this job. They were not altered by P1; their actual hashes are pinned in the manifest.

## Files changed

`engine/plan/Cargo.toml`; `engine/plan/tests/zones_prereqs/{main.rs,t1.rs,t2.rs,t3.rs,t4.rs}`; `jobs/3.2d-3-P1/{verify.ps1,last.md}`. The empty modules reserve disjoint files for P2/P3/P4.

## Replay

From this worktree, `powershell -NoProfile -ExecutionPolicy Bypass -File .\jobs\3.2d-3-P1\verify.ps1` runs, at Idle priority and offline, the baseline, T1, workspace, frozen footprint mutants, manifest generation and prerequisite verifier, writing each command's log. Equivalent gate commands:

```powershell
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\3.2d-3-P1'
[Diagnostics.Process]::GetCurrentProcess().PriorityClass = 'Idle'
Push-Location D:\path-tracing-wt\plain\engine
cargo test --offline -p plan --test plain straight -- --nocapture
Pop-Location
Push-Location .\engine
cargo test --offline -p plan --test zones_prereqs t1_observed_transient -- --nocapture
cargo test --offline --workspace
Pop-Location
& .\jobs\1.2-footprints\verify.ps1
& .\jobs\3.2d-3\verify.ps1 -Case Prerequisites -Tests T1 -Difficulty Peaceful -Mobs Off -Evidence D:\path-tracing-evidence\3.2d-3
```

