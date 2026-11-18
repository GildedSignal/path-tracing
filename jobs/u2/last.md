# Joints and sets — builder record

Spec and prompt re-read from `C:\Users\PASCAL\Desktop\path tracing\jobs\u2\`.
Evidence: `D:\path-tracing-evidence\u2`. Offline, Normal priority, target `D:\path-tracing-target\u2`.
The pre-existing world-fixture diffs and backups are preserved. No game, server, client, launcher,
git metadata, merge or push was used.

## Result

- Core repairs and release tests pass. The cached-answer bypass, uncounted native ticks and
  missing-bundle mutations are caught by the final gates.
- KT10 topology passes through witnessed records. Its finite post-ladder learning search completed;
  final T and current-record replay are inserted by `report.ps1` below.
- **KT11 is RED.** Its assertion remains frozen. The law-side counterpart of the GLM gap pattern
  grows under refinement; see `kill11.log`. Ice and hardtrack signatures are also measured there.
- **Rule 3's translation premise is RED.** `translation-premise.log`: the same binade and discrete
  event signature can send adjacent input positions to output positions two ULPs apart. This is
  rounding parity, already represented by U1's integer maps; event signature alone does not remove it.
- Per the prompt's decision 2, only dependent work is stopped: interval-wide certification,
  endpoint-hull sufficiency, nonsingleton exit interpolation, and its downstream pullback composition.
  Joint facts currently certify exact singleton domains. These are not advertised as passing KT11
  or as a frozen interval representation. The integrator must re-derive the interval premise.

## Decisions and reasons

1. `World::observe_tick` is called at the shared native movement entry, before refusal. `MeteredWorld`
   and nested `RecordingWorld`s propagate it, so every entry point and refused tick is counted.
   Both U1's preparation meter and P1 consume this observation; there is no adjacent hand increment.
2. Kill 8 checks corridor calls, kernel reads, bundle hits, composed counts, native replay and ticks.
   Its planted shortcut reuses a physically correct cold uniform answer while bypassing both the
   corridor and resident store; its stage labels are preserved. The two other mutants plant discarded
   native calls and remove bundle hits. `jobs/u1/verify.ps1` now invokes these real mutants.
3. Program indices come from the toolkit's product definition. Tick/count widths are named as
   `Arguments` representation limits. The production map assertion moved into a native-addition test.
4. Inverse probes build the law's body boxes from borrowed phases and call `Goal::touched_boxes`.
   Affine families stream directly to the inverse; output streams boundary events. Incumbents run
   in lower-bound order with witnessed horizons. Monotone windows bracket exponentially from an
   accepted seed before bisection. No goal-answer cache, physics tolerance or approximation was added.
5. The build hash covers law sources, `sin.bin`, world model/registry data and decoding sources.
   It replaces the static codec schema and manual binding version. The two FNV-128 streams are
   version identifiers, not equality witnesses for interned data; all interning compares bytes.
6. Exact-state IDs are u32 and separate from cell-answer u16 IDs, including the older pattern and
   proposal consumers. The cold template/state bytes are reported separately from hot IDs.
7. Entry charts retain every field and phase exactly except s, plus all position binades and the
   exact perpendicular coordinate. If a chart cannot round-trip the input bits it remains an exact
   point. Rest has no free line. Certification uses the identity frame only.
8. Certificates bind pattern ID, plateau index and law hash. Consuming a fact checks the exact key,
   its singleton s domain and invalidation. The witness holds s bits and end digest. No class is used
   to reconstruct a witness. Program outputs are checked against their toolkit IDs while learning.
9. The goal-free record and goal-bound field are separate. Both generator chains and learned joint
   edges use `WordStep` and the same backward min-plus operation; counters observe that operation.
   Acting replays no joint edges. Missing/invalidated facts and created inventory faces enter one queue.
   Swept goal touches terminate edges, rather than giving an overshooting endpoint a zero-time goal.
10. Failed learning retains Searched or Refused, work, witness and native reads. A missed proposal
    never constructs Forbidden. Its proof-bearing type requires premises, separating cells, hash and
    read certificate; a kit-wide envelope producer is not claimed here. No false Forbidden is emitted.
11. Face inventory includes air reads and partial-shape top faces; it proposes candidates, while
    the law alone establishes grounded takeoffs. An edit repairs affected inventory records and queues
    newly created joints. Cell and environment changes invalidate the shared certified relation.
12. Goals are exact closed cell regions with no epsilon, including equality at the top surface.
    The legacy field and coarse near-goal query consume the same predicate. This fixed the workspace
    regressions exposed by epsilon removal. Clicked-face semantics wait for the stated track-face merge.
13. Field prices use two witnessed times when present and otherwise the difference of cone lower
    bounds. Slide queries require an upstream certified reachable interval; no sliding certificate is
    inferred from singleton facts. The monotone pullback primitive is tested separately; broad interval
    composition stays red with its premise.
14. KT10 reads the old build and toolkit, never Julien's inputs. The finite learning budget uses all
    held words and all prefix yaws in its toolkit. Uniform lane rows come from role classes and
    witnessed face reach, and the lane axis comes from geometry. Its reported best is an upper bound
    within that learning budget, not a proof of global optimality. The discovered program is replayed
    through the current record in a separate explicit gate, so learning need not be repeated for
    metadata changes. The aborted first run selected rows by length alone; the replacement derives
    their connection from the reach relation. Its log is preserved as `kill10-bar-aborted.log`.
15. Stairs are measured natively with exact normalized full-state comparisons. No recurrence found
    in that sample means no stair gait is certified; there is no terrain-specific rule in `solve`.
16. Epsilon removal exposed an i128 exponent-alignment overflow and an excluded binade endpoint in
    P2's independent inverse oracle. Its signed dyadic midpoint subtraction now spans binary64's
    entire exponent range; its interval inequalities match the closed goal. Native addition still
    checks every derived threshold and the original native streams/edge neighbours remain in force.

## Verification and timings

Final gate records and distributions are generated below. Replay all gates with:

`powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2/verify.ps1 -Search -Footprints`

`-Search` regenerates the independent learning result. Without it, its recorded result is replayed.
Both red gates run explicitly with their original failing assertions and are recorded as red, not skipped.
The new U2 ignores are limited to those explicit diagnostic and long discovery/replay runs.

## Files

Law: build hash, codec IDs/bindings, native tick observation, recording propagation, pattern consumer,
and affected codec tests. Words: product IDs/limits, joint event signature, proposal state IDs and tests.
Plan: generator store/source, exact maps, common goal predicate, solve, joints, face inventory,
legacy goal consumers, core/P1 tests, U2 gates and stair measurement. Jobs: U1 mutation invocation,
U2 runner/mutants/verifier/report and this record. HANDOFF: one anomaly pointer, within its line budget.

No claim that step 2's interval representation is ready to freeze or merge.

<!-- GENERATED -->
## Final measurements and gates

The gate contract is completed with the two red results preserved. This is not an interval-design freeze.

| Gate | Result |
|---|---|
| u2-final | PASS |
| maps-and-stairs | PASS |
| kill11 | RED, failure recorded |
| translation-premise | RED, failure recorded |
| kill10-current-record | PASS |
| core-final-release | PASS |
| core-two-yaw | PASS |
| workspace-final | PASS |
| independent KT10 search | PASS |
| core mutants | 3 caught; restored passes |
| footprint mutants | 13 caught; restored passes |

KT10 best after the ladder exit: **158 ticks against 169**, margin 11;
conditional full T is 417 if the ladder exit occurs at his tick 259. This is offline law evidence,
not a server measurement. The search found 9172 sentences, 13658 wall hits and 43800
ladder hits. The current-record replay certifies 5 edges and observes 0 acting law ticks;
first goal touch and all endpoint state bits agree with native replay.

Workspace: 51 successful result lines, 234 passed checks, 23 ignored checks.
The explicit red gates and the KT10 discovery/replay are run separately from that default workspace.

The complete serial solve distributions (including counters, stages, output records and shared
composition) are below. Cold uses a fresh store/codec; warm retains generators, not a goal answer.
All raw sorted samples and profiles are in `core-final-release.log`. Median averages the middle pair;
p95/p99 use nearest rank. The plan's microsecond floor is not reached.

| Goal cell | Mode | n | min us | median us | p95 us | p99 us | max us |
|---|---|---:|---:|---:|---:|---:|---:|
| 1000000,-61,0 | cold | 100 | 23233.1 | 24384.85 | 24690.1 | 24832.1 | 25274.4 |
| 1000000,-61,0 | warm | 100 | 222.4 | 244.35 | 267.7 | 299.1 | 301.8 |
| 0,-61,1000000 | cold | 100 | 24029.1 | 24343.7 | 25421.4 | 26527.5 | 30125.7 |
| 0,-61,1000000 | warm | 100 | 260.3 | 273.25 | 303.7 | 319.1 | 457.5 |
| 1000000,-61,330000 | cold | 100 | 26041.1 | 26289.5 | 27419 | 27869 | 28678.3 |
| 1000000,-61,330000 | warm | 100 | 390.9 | 399.85 | 434.2 | 448.7 | 477.7 |

```
U2_KT11 pattern=glm-ice divisions=64 plateaus=2 calls=107 cuts=[c057ee5506e671bf]
U2_KT11 pattern=glm-ice divisions=256 plateaus=2 calls=297 cuts=[c057ee5506e671bf]
U2_KT11 pattern=glm-ice divisions=1024 plateaus=2 calls=1063 cuts=[c057ee5506e671bf]
U2_KT11 pattern=glm-gap divisions=64 plateaus=7 calls=317 cuts=[c057ee5506e671c0, c058158f9357b5e6, c0581aee0418b453, c05823da7f7e7ff4, c058244088224005, c0582ef55db51f00]
U2_KT11 pattern=glm-gap divisions=256 plateaus=11 calls=657 cuts=[c057ee5506e671c0, c058158f9357b5e6, c0581aee0418b453, c0581b21fd97dcbf, c0581ed87b7dfbff, c0581f290418c005, c05823da7f7e7ff4, c058244088224005, c0582ef55db51f00, c0582f292dae757a]
U2_KT11 pattern=glm-gap divisions=1024 plateaus=17 calls=1633 cuts=[c057ee5506e671c0, c058158f9357b5e6, c0581a3eb3fee3f8, c0581a6400479b01, c0581aee0418b453, c0581b21fd97dcbf, c0581cabffff7ffe, c0581cc080000001, c0581ed87b7dfbff, c0581f290418c005, c05823da7f7e7ff4, c058244088224005, c0582541fe7c9e00, c05825dda2089005, c0582ef55db51f00, c0582f292dae757a]
U2_KT11 pattern=hardtrack-lane-wall divisions=64 plateaus=2 calls=104 cuts=[3faa62d0ef6b851e]
U2_KT11 pattern=hardtrack-lane-wall divisions=256 plateaus=2 calls=294 cuts=[3faa62d0ef6b851e]
U2_KT11 pattern=hardtrack-lane-wall divisions=1024 plateaus=2 calls=1060 cuts=[3faa62d0ef6b851e]
U2_STAIRS transitions=24 native_ticks=89 bit_exact_recurrence=None gait_certified=false reason=requires_recurrence_and_corridor_certificate
U2_KT10_REPLAY best=158 human=169 margin=11 law_hash=[e8, 2f, ef, ab, 75, ca, bf, b3, 62, 5c, 88, 7d, bf, fd, 3a, a8, c5, 9c, a0, c1, 6f, d8, 6, 8, 82, ed, cb, fe, 6c, 79, 4a, e3] cold_template_bytes=3962 cold_state_bytes=4080 edges=5 acting_native_ticks=0 first_touch_exact=true
```

Worst native map/replay error is zero ULP; warm native ticks are zero. Work by distance and scars,
native tick totals and the explicit refusals (Normal resource prediction, loaded boundary, codec owner):

```
U1_RESULT cell=1000000,-61,0 ticks=2806325 cold_law_ticks=2450 warm_law_ticks=0 reads=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } end_bits=[4696837145997503315, 13857013103465594880, 4605385597524425815]
U1_RESULT cell=0,-61,1000000 ticks=2806325 cold_law_ticks=2450 warm_law_ticks=0 reads=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } end_bits=[4596355305392900493, 13857013103465594880, 4696837145997503315]
U1_RESULT cell=1000000,-61,330000 ticks=2955181 cold_law_ticks=2394 warm_law_ticks=0 reads=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } end_bits=[4696837148080207338, 13856953993722331136, 4689412975974524754]
test u1::u1_work_tracks_events_and_scars ... U1_WORK distance=1000 cold=ReadCost { blocks: 417178, loaded: 453075, corridors: 1, scars: 0 } warm=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } cold_ticks=2450 warm_ticks=0 lattice=LatticeCost { envelope_intervals: 64, quotient_queries: 31 }
U1_WORK distance=10000 cold=ReadCost { blocks: 417178, loaded: 453075, corridors: 1, scars: 0 } warm=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } cold_ticks=2450 warm_ticks=0 lattice=LatticeCost { envelope_intervals: 30, quotient_queries: 30 }
U1_WORK distance=1000000 cold=ReadCost { blocks: 417178, loaded: 453075, corridors: 1, scars: 0 } warm=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 0 } cold_ticks=2450 warm_ticks=0 lattice=LatticeCost { envelope_intervals: 30, quotient_queries: 30 }
U1_SCARS k=1 costs=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 1 } warm_ticks=0
U1_SCARS k=8 costs=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 8 } warm_ticks=0
U1_SCARS k=32 costs=ReadCost { blocks: 4, loaded: 4, corridors: 1, scars: 32 } warm_ticks=0
test u1::u1_refusals_are_explicit ... U1_REFUSAL source=Normal reason=Some("U1 requires invariant resource guards; server food crossings are not predicted")
U1_REFUSAL source=LoadedBoundary reason=Some("loaded boundary during word preparation")
U1_REFUSAL source=OtherCodec reason=Schema
```

Exact changed-source list (pre-existing fixture changes excluded):

- `docs/HANDOFF.md`
- `engine/law/build.rs`
- `engine/law/src/codec.rs`
- `engine/law/src/codec_proof.rs`
- `engine/law/src/footprint.rs`
- `engine/law/src/lib.rs`
- `engine/law/src/movement.rs`
- `engine/law/src/pattern.rs`
- `engine/law/src/world.rs`
- `engine/law/tests/codec.rs`
- `engine/plan/src/events.rs`
- `engine/plan/src/faces.rs`
- `engine/plan/src/goal.rs`
- `engine/plan/src/joints.rs`
- `engine/plan/src/lattice.rs`
- `engine/plan/src/lib.rs`
- `engine/plan/src/reach.rs`
- `engine/plan/src/search.rs`
- `engine/plan/src/solve.rs`
- `engine/plan/src/source.rs`
- `engine/plan/src/store.rs`
- `engine/plan/tests/u2.rs`
- `engine/plan/tests/u2_stairs.rs`
- `engine/plan/tests/zones_prereqs/t1.rs`
- `engine/plan/tests/zones_prereqs/t2.rs`
- `engine/plan/tests/zones_prereqs/u1.rs`
- `engine/words/src/program.rs`
- `engine/words/src/segment.rs`
- `engine/words/src/table.rs`
- `engine/words/tests/codec.rs`
- `jobs/u1/verify.ps1`
- `jobs/u2/core-mutants.ps1`
- `jobs/u2/last.md`
- `jobs/u2/report.ps1`
- `jobs/u2/run.ps1`
- `jobs/u2/verify.ps1`

Replay commands: `verify.ps1 -Search -Footprints` (full independent discovery),
`verify.ps1 -Footprints` (reuse discovery, replay through current code), then `report.ps1`.
Each cargo run has a JSON record with input hashes, command, exit, invariant output and refusals.
`core-mutants.json`, `footprints.json` and their per-mutant logs record planted-fault results.
