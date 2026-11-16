# P2 (T2): failed inverse-stop premise

Consumer: 3.2d-3 exact inverse of `Goal::touched`. Offline, no game, client or server.

## Result

The specified inverse premise is false: P2 states that the player AABB is `[p - 0.3, p + 0.3]`. `engine/law/src/shapes.rs:112-113` instead uses `(0.6f32 / 2.0f32) as f64`, which is `0.30000001192092896`, and `shapes.rs:203-204` uses strict intersections. `engine/plan/src/goal.rs:25-26` tests the hull of the before and after boxes.

A stationary witness at `p=(-0.301000005, 0.0, 0.5)` with goal cell `(0,0,0)` gives zone minimum x `-0.001`. The P2 literal 0.3 slab does not touch; the actual body maximum x is `-0.0009999930790710598` and `Goal::touched` is true. The bit-exact equality assertion in `t2_binade` therefore fails, as recorded in `D:\path-tracing-evidence\3.2d-3-P2\t2.log` (exit 101). No tolerance was added. This does not establish that the binade map itself fails.

Under the instruction to stop on a false premise, no integer map, poison test, segment count, timing distribution, passing manifest, workspace/footprint gate, or prerequisite pass is claimed. The other requested assertions were not run; segment counts and timing min/median/max are not available. No refusal beyond this failed premise was encountered. The exact comparison has no numerical tolerance.

## Files changed

- `engine/plan/tests/zones_prereqs/t2.rs`: failing detector-edge premise witness.
- `jobs/3.2d-3-P2/last.md`: this record.

## Replay

```powershell
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\3.2d-3-P2'
Push-Location engine
cargo test --offline --release -p plan --test zones_prereqs t2_binade -- --nocapture
Pop-Location
```

STATUS: DONE
