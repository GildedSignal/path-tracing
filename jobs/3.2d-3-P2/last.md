# P2 (T2): exact binade word maps and inverse stop

Consumer: U1 shared zone kernel (S165), not a separate terrain planner. Offline Peaceful, mobs off; no game, server or client launched.

## Result

PASS. The test-local kernel composes the exact parity map, jumps cycles by quotient/remainder, finds binade/sign/subnormal events by integer division, and inverts the strict detector predicate using exact dyadic midpoint ownership. Safe remainders use stored integer phase offsets. Only boundary additions use native floating addition; the solve calls no native movement ticks.

The law owns dimensions: `PlayerState::bounding_box` at `engine/law/src/state.rs:196-197` calls `Aabb::player` at `engine/law/src/shapes.rs:111-113`; pose height comes from `state.rs:27-32`. The former spec literal radius was false; the corrected test calls these law functions. Eye height is not read by this detector. `Goal::zone`, `Goal::touched` and strict `Aabb::intersects` retain their native expansion, hull and endpoint ownership.

`movement_this_tick` records wanted, while position adds moved (`movement.rs:748-749`). Preparation obtains actual moved operands from `tick_with_collision` and proves horizontal equality before using the recorded values. It reproduces P1's transient and cycle with the native +x direction coefficients, without rotating a recorded path.

## Assertion results

- tie-even/odd and negative dyadics PASS
- normal/subnormal/signed-zero/negative-step PASS
- parity_graph ds=[0.5] prefix=1 period=1 PASS
- parity_graph ds=[1.5] prefix=1 period=1 PASS
- parity_graph ds=[1.0] prefix=0 period=2 PASS
- parity_graph ds=[0.5, 1.0] prefix=1 period=1 PASS
- quotient/remainder trillion-count and subnormal boundary PASS
- rounded detector edge integer thresholds=384 PASS
- stream positive ticks=4414527 segments=4 endpoint+motor PASS inverse=[None, Some(280), Some(280), None, Some(1471600), Some(1471600), None, Some(4414246), Some(4414242)]
- stream negative ticks=1471885 segments=3 endpoint+motor PASS inverse=[None, Some(280), Some(280), None, Some(1471600), Some(1471600)]
- stream zero ticks=400 segments=18 endpoint+motor PASS inverse=[None, Some(142), Some(140), None, Some(280), Some(280), None, None, None]
- stream minus-zero ticks=400 segments=11 endpoint+motor PASS inverse=[Some(1), Some(1), Some(1), None, Some(142), Some(140), None, Some(280), Some(280)]
- stream plus-zero ticks=400 segments=11 endpoint+motor PASS inverse=[Some(1), Some(1), Some(1), None, Some(142), Some(140), None, Some(280), Some(280)]
- detector law radius=0.30000001192092896 height=1.8 edge_neighbours=144 PASS
- comparisons positions+motor=594 inverse=42 PASS

- Poison: RED, exit 101 at the exact-half parity assertion; unpoisoned release run GREEN.
- Local T1 replay: PASS (`t1-local.log`).
- Workspace: PASS, 211 passed and 19 ignored across its test summaries (workspace.log).
- Frozen footprint gate: 13/13 CAUGHT; restored regression checks and source hash comparisons PASS (footprints.log).
- Canonical T1+T2 prerequisite verifier: PASS (`prerequisites.log`). T2 manifest is at `D:/path-tracing-evidence/3.2d-3/T2/manifest.json`; the task copy and all logs are in `D:/path-tracing-evidence/3.2d-3-P2`.

## Closed-form measurements

| Blocks | First-touch tick | Word remainder | Binade segments | Min ns | Median ns | Max ns | Samples |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 278 | 2 | 8 | 17200 | 17400 | 34800 | 101 |
| 10000 | 28061 | 5 | 15 | 35500 | 35900 | 87000 | 101 |
| 1000000 | 2806320 | 0 | 21 | 91000 | 130700 | 416500 | 101 |

The measured start is the prepared carried-cycle state anchored at x=0.5, not a replay from rest. The detector at y=-59 intersects every vertical phase, so this measures the horizontal inverse. Native witnesses separately cover detectors at y=-61, -60 and -59, including exact floor-detector misses while airborne. Ticks exclude P1 preparation and are not live click-to-arrival T. Segment counts extend to the inverse stop. Timings include deriving the query count bound from the law operands and detector edge, compressed-family construction, inverse stop, and direct map application at the stop; preparation and streaming validation are outside the interval. All individual duration samples are logged. Shared-machine Idle-priority timings are measurements, not latency bounds.

Worst exactness margin is equality of bits; no nonzero tolerance applies. No unexpected refusal occurred. A None inverse is an expected no-touch result for this finite word prefix. Resource numeric recurrence remains P4's responsibility; ending motor comparisons use P1's guard-preserving motion projection. No general bearing, other medium, or production integration is claimed.

## Files changed

- `engine/plan/tests/zones_prereqs/t2.rs`
- `jobs/3.2d-3-P2/{derivation.md,verify.ps1,report.ps1,last.md}`

The two fixture region files and their backups were already dirty when this worktree was supplied, as P1 recorded; they were not edited by P2. Frozen gate mutations are restored byte for byte.

## Replay

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\jobs\3.2d-3-P2\verify.ps1
```

The verifier uses `CARGO_TARGET_DIR=D:/path-tracing-target/3.2d-3-P2` and offline cargo, runs RED then GREEN, local T1, workspace and frozen footprint gates, writes file hashes, and verifies T1,T2. `report.ps1` regenerates this record from its logs. The derivation names the qualified domain and endpoint proof.

STATUS: DONE
