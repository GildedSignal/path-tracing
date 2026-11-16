# P4: Peaceful food guard

Consumer: 3.2d-3 resource guard. No game, server, or client launched. Target `D:\path-tracing-target\3.2d-3-P4`; evidence `D:\path-tracing-evidence\3.2d-3-P4`.

## Results

- Offline release `t4_food`: PASS, `t4.log`. 240 ticks each: observed food 7 and 20 remained unchanged, with 2 and 1 exhaustion wraps respectively and sprint allowed throughout; observed food 6 remained 6 and sprint refused throughout. `state_answers` for food matched that guard every tick; levels 7 and 20 matched and 6 differed. Unknown food word was refused in all three starts. Eight health/saturation/exhaustion combinations round-tripped exact bits. No assertion predicts saturation across server regeneration.
- Under uninterrupted sprint from both spawn bars (20,5,0) and (20,20,0), without external resource changes, a food-drain crossing on Peaceful does not occur: `food_tick` skips food debit at difficulty 0, and Peaceful server regeneration can increase food. This agrees with `lab/results/local-20261115-1224-1-0-trial-food-crossing/check.md`, which reports no crossing on Peaceful; its 3949 and 7057 tick results are Normal, not Peaceful. These are reported lab results, not measurements by this test.
- Offline workspace tests: PASS, `workspace.log`. No guard-event solve or latency distribution was measured. A resource packet re-anchor and live assertions were not run. No passing T4 manifest was created.
- The frozen footprint mutant gate temporarily edits law source and so was not run under the explicit restriction to the named files. The all-T1–T4 prerequisite gate cannot pass without the other jobs' manifests and a live T4 manifest.

## Conflict

The P4 specification requires a live gate that launches a server and bot client (report.md P4: “launching hidden/background server + bot client” and `verify.ps1 -Case T4Food ... -Live`); this launch explicitly says “No game, server or client is launched. Everything is `cargo --offline`.” The live result and its passed manifest cannot be produced while obeying the launch restriction. The test above is offline and does not claim the live gate.

## Files changed

`engine/plan/tests/zones_prereqs/t4.rs`, `jobs/3.2d-3-P4/last.md`. Existing region modifications and backups predate this job.

## Replay

```powershell
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\3.2d-3-P4'
Push-Location .\engine
cargo test --offline --release -p plan --test zones_prereqs t4_food -- --nocapture
cargo test --offline --workspace
Pop-Location
```

