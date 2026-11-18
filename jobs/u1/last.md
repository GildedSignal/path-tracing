# U1: the kernel, including terminal-word preimages

**Result: all specified offline gates pass.** This is a branch implementation for the independent judge; nothing was merged, pushed or flown. The earlier counts-only counterexample remains a native-law regression. The integrator's correction is implemented: a bearing count does not assert arrival; the same extension chooses a terminal word from its carried state.

## Release million-line thinking distributions

Values are **microseconds**, measured around the complete `solve` call, including result construction and the harness's assertions. Cold uses a new codec and store: label interning, all law-driven preparation, phase transitions, arithmetic-map compilation, corridor qualification and the terminal preimage count. Warm reuses resident generators and arithmetic maps, not a cached goal answer. Both include event lists, certificates, stage trace and endpoints. Generator construction, settling the observed input and independent witness replay are outside the clock. Sorted samples are in `u1.log`; median averages the two central samples, p99 is nearest rank.

| B cell | Mode | n | Min us | Median us | p99 us | Max us |
|---|---|---:|---:|---:|---:|---:|
| 1000000,-61,0 | cold | 100 | 24304.7 | 25602.9 | 28811.7 | 29349 |
| 1000000,-61,0 | warm | 100 | 364.5 | 379.35 | 520.8 | 697.3 |
| 0,-61,1000000 | cold | 100 | 24645.6 | 24949 | 27203.4 | 28365.3 |
| 0,-61,1000000 | warm | 100 | 403.5 | 423.35 | 476.3 | 611 |
| 1000000,-61,330000 | cold | 100 | 26884.3 | 27052.15 | 28697.7 | 28749.8 |
| 1000000,-61,330000 | warm | 100 | 590.3 | 600.6 | 657 | 674.1 |

| B cell | Arrival T ticks | Cold preparation law ticks |
|---|---:|---:|
| 1000000,-61,0 | 2806325 | 2450 |
| 0,-61,1000000 | 2806325 | 2450 |
| 1000000,-61,330000 | 2955181 | 2394 |

Warm preparation is zero law ticks. Cold preparation uses the law's collision-returning entry to retain **actual position-addition operands and clamp bits on all axes**; it executes the same frozen tick as `try_tick`. Native witnesses independently use `try_tick`.

**What dominates warm time:** exact goal-preimage construction/intersection over stored word phases, followed by output/readout and its event certificates. The profile is measured:

| B cell | Stage | Min us | Median us | p99 us | Max us |
|---|---|---:|---:|---:|---:|
| 1000000,-61,0 | entry | 10.5 | 11.4 | 22.4 | 22.5 |
| 1000000,-61,0 | bearing | 13.4 | 13.6 | 19 | 21.5 |
| 1000000,-61,0 | inverse | 273 | 283.6 | 407 | 412.4 |
| 1000000,-61,0 | output | 64.2 | 67.2 | 119.5 | 265.2 |
| 0,-61,1000000 | entry | 11 | 12.8 | 18 | 22 |
| 0,-61,1000000 | bearing | 13.5 | 13.9 | 18.6 | 20.7 |
| 0,-61,1000000 | inverse | 310.6 | 325.8 | 363.6 | 475.3 |
| 0,-61,1000000 | output | 64.1 | 69.6 | 80.7 | 106.5 |
| 1000000,-61,330000 | entry | 12.2 | 12.9 | 14.9 | 23.9 |
| 1000000,-61,330000 | bearing | 49.6 | 50.2 | 61.1 | 66.7 |
| 1000000,-61,330000 | inverse | 443.1 | 449.8 | 498.9 | 505.7 |
| 1000000,-61,330000 | output | 82.9 | 86.7 | 94.5 | 104.2 |

The oblique warm median is **120.1 times** the plan's approximately 5 us Fermi floor. The floor has **not** been reached. The old diagnostic repeatedly rebuilt binary64 phase graphs and bisected whole readouts. This implementation caches maps by `(binade exponent, word phase, parity)` and uses exact offset extrema for boundary bounds. The remaining gap is inverse/allocation work, not hidden tick simulation or a cell scan. No performance threshold was invented or weakened.

## Counts, native witnesses and planted faults

| Gate | Exit | Passed |
|---|---:|---|
| lattice | 0 | True |
| u1 | 0 | True |
| t3 | 0 | True |
| prerequisites | 0 | True |
| workspace | 0 | True |
| footprints | 0 | True |
| kill8-planted | 101 | True |
| kill8-restored | 0 | True |

The planted shortcut is confined to the no-exception answer and removes its `Extend` stages. Kill test 8 fails, the source is restored **byte for byte**, and the same test passes again. The frozen footprint verifier catches **all 13 mutants**, restores sources and passes its restored checks. Individual logs are copied under `D:\path-tracing-evidence\u1\footprints`. `manifest.json` pins inputs and logs; the driver checks every input hash after the mutation gates.

Clean-source work is identical at all requested distances, including cold law preparation and cold world reads; warm work has zero law ticks. Scar work grows with journal entries visited rather than path length:

```text
U1_WORK distance=1000 cold=ReadCost { blocks: 417178, loaded: 453075, corridors: 1, scars: 0 } warm=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } cold_ticks=2450 warm_ticks=0 lattice=LatticeCost { envelope_intervals: 64, quotient_queries: 31 }
U1_WORK distance=10000 cold=ReadCost { blocks: 417178, loaded: 453075, corridors: 1, scars: 0 } warm=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } cold_ticks=2450 warm_ticks=0 lattice=LatticeCost { envelope_intervals: 30, quotient_queries: 30 }
U1_WORK distance=1000000 cold=ReadCost { blocks: 417178, loaded: 453075, corridors: 1, scars: 0 } warm=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } cold_ticks=2450 warm_ticks=0 lattice=LatticeCost { envelope_intervals: 30, quotient_queries: 30 }
U1_SCARS k=1 costs=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 1 } warm_ticks=0
U1_SCARS k=8 costs=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 8 } warm_ticks=0
U1_SCARS k=32 costs=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 32 } warm_ticks=0
```

`CorridorSource` is queried once per tube. The generator plus a sparse journal never walks a million cells. Its lexicographic BTreeMap range is filtered in all three coordinates; an enclosing-box exception can be conservative. U1 returns `NeedsJoints`, never invokes another planner. The immutable source has no earlier loaded-lease expiry; the event type is retained for a future timed source.

P3's corrected acceptance is checked bit for bit at every word boundary and every preceding native tick, including axis/oblique million-line witnesses, quadrants and a negative axis:

```text
T3_NATIVE goal=[1000000, -61, 0] ticks=2806325 words=6 lattice=LatticeCost { envelope_intervals: 30, quotient_queries: 30 }
T3_NATIVE goal=[1000000, -61, 330000] ticks=2955181 words=6 lattice=LatticeCost { envelope_intervals: 370, quotient_queries: 31 }
T3_NATIVE goal=[1000, -61, 1000] ticks=3972 words=3 lattice=LatticeCost { envelope_intervals: 132, quotient_queries: 30 }
T3_NATIVE goal=[-1000, -61, 1000] ticks=3972 words=3 lattice=LatticeCost { envelope_intervals: 132, quotient_queries: 30 }
T3_NATIVE goal=[-1000, -61, -1000] ticks=3972 words=3 lattice=LatticeCost { envelope_intervals: 132, quotient_queries: 30 }
T3_NATIVE goal=[1000, -61, -1000] ticks=3972 words=3 lattice=LatticeCost { envelope_intervals: 132, quotient_queries: 30 }
T3_NATIVE goal=[0, -61, -1000] ticks=2809 words=3 lattice=LatticeCost { envelope_intervals: 120, quotient_queries: 30 }
```

The independent release reference also covers the full 65,536-yaw domain. Positive drag/input preserves each heading component's sign from rest, excluding headings outside the target quadrant. For the remaining headings it obtains the complete native prefix and exact motor recurrence, encloses all binary64 additions on the coarsest positional lattice, and proves the resulting slope cone misses even an enlarged goal rectangle. This is an outward arithmetic enclosure, not an acceptance tolerance. No yaw sweep enters `solve`.

```text
T3_SINGLE_YAW domain=65536 native_yaws=16383 sign_excluded=49153 cone_overlaps=0 law_ticks=4613633 ticks_min=267 ticks_median=279 ticks_p99=301 ticks_max=302 gap_num=1016843955266964965974 gap_den=1723924228 gap_unit_exp=-34
```

The retained counterexample checks **2806330 native ticks** with zero counts-only touches. Its minimum positive vertical miss during horizontal overlap is **0.7957356006686993 blocks**. The native full-state control stops jumping at tick **2806320** and touches at **2806325**. These remain evidence about the former spec premise; they no longer stop unrelated kernel work.

The tests also cover the law-derived food guard, observed resource bars, missing-bearing learning followed by zero-tick reuse, a subnormal switch, reuse across a vertical binade change, and a tube whose incoming velocity initially carries it away from B. A separate test interns 100,000 `MotorKeyId(u32)` keys without growing the codec's class table. One owner marker prevents accidental LabelId namespace reuse; motor keys never enter the 16-bit table.

## Frazzoli section 3.1 and representation

1. **Real yaw-switch transients: applied.** These are ordinary words with count one, prepared from the carried motor state by the same function. No velocity is rotated instantly. The terminal word is another store row, considered at every phase by the same preimage/extension mechanism.
2. **Verified partial symmetry only: applied.** Keys retain exact read-before-write fields, vertical phase, support-presence and law resource guards; they exclude t, horizontal position, numeric food/saturation/exhaustion and absolute support coordinates. Maps retain binade/sign/subnormal events and ties-to-even parity. Vertical readout integrates stored operands between collision resets, rather than assuming floating levels translate exactly.
3. **Separate trim nodes and maneuver edges: refused.** There is one program/arguments/count record. A gait differs from a transition through its count and exit event.
4. **Continuous inversion as a discrete goal-hit theorem: refused, per the integrator's ruling.** Euclidean quotients and floor sums construct the bearing lattice; the exact preimage supplies its terminal word. A continuous endpoint LP does not prove a discrete vertical-phase touch.

The lattice independently matches bounded integer enumeration in its unit gate. Native yaw bracketing uses binary search and exact dyadic cross-product signs, without sweeping the table. Envelope comparisons use full-width products and Euclidean floor sums instead of enumerating q. A complete phase cell is retained before the goal. Candidates are read out by exact binade maps and accepted only by the unchanged `Goal::touched`; its AABB/face predicate is not copied into `events.rs`.

The bracket keeps the kit lower bound and witnessed upper bound distinct. This does **not** claim global optimality over all Minecraft inputs or force LB=UB. Joints, sets, slide refinement, other toolkit terms and live resource events remain the later steps named in the spec.

## Files, judgments and scope

- `engine/words/src/program.rs`: count on every word, default one.
- `engine/words/src/gait.rs`: explicit deprecation of the old full-bar helper; the new entry point never calls it.
- `engine/plan/src/{store,events,lattice,source,solve}.rs`, `lib.rs`: shared store, maps, interval lattice, source proof and single loop.
- `engine/plan/tests/zones_prereqs/{main,t3,t3_single,t4,u1}.rs`: corrected P3, full-domain single-yaw exclusion, retained counterexample, law-derived P4 assertions and U1 gates. P1/P2 remain unchanged.
- `jobs/u1/{verify,run-footprints,report}.ps1`, `report-template.md`, `last.md`: replay, source-restoring faults and this generated report.
- `docs/HANDOFF.md`: anomaly/status pointer within its line budget.

**Decisions/departures:** the store uses interior mutability behind the specified shared reference so missing bearings can learn. These bare-legs rows prepare pitch zero. Cold preparation uses the collision-returning law entry, as P2 does, to capture exact clamped operands. Arithmetic maps compile lazily, with all cost included in thinking. A shift across an axis tie is a new bearing and learns; covered shifted bearings reuse their kernel. Immutable Peaceful guards are proved by the source; non-Peaceful server crossings refuse until their scheduled producer exists. Resource numerics in predicted endpoints remain observed values; the runner's server re-anchor owns their future values.

The generator exposes conservative collision occupancy from its floor and sparse journal, retaining the caller's halo for protruding shapes. It is checked against full scanning on 500 shape/suffocation queries and 200 bit-exact native ticks; this shortens witness feedback without adding a solver branch. An initial complete gate run passed its tests but stalled while serializing PowerShell provider annotations on log strings. The driver now reads plain strings before JSON serialization, and every gate was rerun on these final inputs.

There is no terrain-kind branch in `solve`, legacy-planner fallback, tolerance, per-block case or warm law replay. `engine/native`, `bot` and the frozen law are unchanged. Existing dirty fixture `.mca` files/backups were not edited. No game, server, client or forbidden launcher ran. Growth buys the counted-word kernel, exact preimages and replayable evidence; the integrator owns the commit and its `Bought:`/`Built-by:` trailers.

## Replay

From `D:\path-tracing-wt\u1-astra`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\jobs\u1\verify.ps1
```

The driver sets Rust homes, Normal priority and `CARGO_TARGET_DIR=D:\path-tracing-target\u1-astra`. It runs release U1/P3/lattice gates, debug prerequisites, workspace, footprint mutants, planted shortcut and restored test. Outputs land in `D:\path-tracing-evidence\u1`, and this file is regenerated. `report.ps1` regenerates tables from an existing passing manifest without re-running gates.

STATUS: DONE

