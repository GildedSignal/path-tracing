# U2b: S203 builder record

Both requested parts are implemented on this branch. The adopted point-or-hold
gate passes; the warm full solve asserts zero PlayerState copies and zero heap
allocations. The ~5 us floor remains missed, with distributions and the remaining
work below. This is a builder result, awaiting the independent judge; no merge or flight claim.

## Representation and decisions

1. The law declares minor_horizontal_collision as its non-monotone field and
   reports Free/Clamp/Hold for each selected axis pass. CONTACT_CONSTANTS and
   approach/hit comparisons are under LAW_HASH. Collision arithmetic, visitation
   order, early zeroing and step selection are unchanged. Recording wrappers forward the report.
2. A clamp is a point per flag; a hold retains the earlier exact addition map.
   Silent clamps remain min-maps inside free pieces. Cuts come from law-owned
   face comparisons and exact-map bisection; no sample grid enters learn().
   Native samples are an independent judge. The face inventory uses the law's actual
   per-tick read-footprint hull, replacing the earlier fixed two-cell positional margin.
   Resource intervals enclose both flag tails;
   identical point images merge. The native second-wall check closes the hold witness.
   Identity follows the law's position-update flag, never an unchanged rounded
   endpoint. A 65-ULP native witness first failed at its odd neighbour, then passed
   with the actual addition operand retained. Explicit free-axis metadata is preserved.
3. EntryTemplate stores the ordered binary64 lattice on the free axis, with
   support relative to floor(body position). Resources travel as use intervals
   with exact numeric entry guards; broader unproved numeric reuse refuses.
   Yaw, pitch and old input values were removed from EntryTemplate, motor_words
   and the legacy pattern start index. Point route/wave entries and queued demands
   use the same canonical key, with numeric guards separate; raw point identity
   retains every witness bit. The native entry regression first failed, then passed;
   two-word alias composition also passes. Word aim arguments remain word data.
   Tests reuse a changed aim and still distinguish a one-ULP velocity change.
4. learn_family publishes the computed pieces with timed endpoint-hull read
   certificates for every declared flag. Legacy learn retains its exact-point
   API for arbitrary conditional programs. Held producers are reduced from the
   toolkit predicates, rather than assuming that their row count is one.
5. Piece exits are maps or point IDs; consume checks key, lattice, numeric guard
   and certificate, and returns the exit set and T range. The backward wave
   refines input intervals by exact preimage and applies min-plus to families
   and points until stable, including cross-axis singleton intersections.
   Alternative images retain [min,max]; an unknown branch leaves the upper open.
   Numeric-use intervals that vary cannot enter a downstream exact numeric guard.
   The legacy single-state route refuses unresolved family sets rather than
   choosing a representative. The set consumer exposes them to the caller.
6. The two-word family test checks pulled-back T, shifted exact readout, dead-aim
   reuse, distinct velocity, local certificate invalidation and no acting replay.
7. A terminal joint is learned from the face once in JointStore. It holds exact
   entry rectangles with constant T and next word, per floor label/gait/phase/
   floating chart and vertical face geometry. Interior charts share across
   different integer goals; binade boundaries retain their exact edge class.
   Warm terminal lookup performs no control search or inverse bisection. A test
   changes B, proves no new face table was built, and compares with a fresh solve.
   The first changed goal can compile cold parity graphs; warm zero is measured
   after these arithmetic graphs exist, independently of terminal-table reuse.
8. Player-state ownership is replaced by Arc phase handles plus exact readouts.
   Explicit materialize() is at execution/test boundaries. Inline lists spill
   on overflow without a semantic cap. The allocator counts alloc, alloc_zeroed
   and realloc; the state type counts Clone. Negative controls catch both.
   Inline iteration disarms ownership before a destructor can unwind; release
   and debug tests ensure no value is dropped twice after partial consumption.
9. The judge speed path: shared cached parity/binade graphs and streamed affine
   runs; borrowed hulls through the same Goal predicate; bound-ordered incumbents;
   no production re-fold assert; streamed boundary events reuse final readouts;
   the remaining terminal inverse is stored as face preimages, not goal answers.
   Query-specific run extents are still reconstructed: no claim of a 5 us implementation.
10. Bearing counts use exact 2x2 determinants, then an exact integer slice at
    the continuous lower bound. Thin strips or checked-width overflow retain
    the proved floor-sum fallback: blindly checking a fixed neighbour radius
    could discard a feasible optimum. The finite enumeration oracle still passes.
11. Generator label serialization reuses scratch space. Raw-model cache hits
    compare all native fields bit-exactly; shape-owner caches retain ownership
    to prevent address reuse. Canonical class IDs remain content-based.

## Frozen gates and evidence

Before freezing, the native oracle measured ice=2 and lane=2 at 2^6, 2^10 and
2^18; every other expected count agreed. See s203-native-oracle.log and
jobs/u2b/s203-oracle-counts.txt. Raw Free/Clamp/Hold counts also disclose silent
clamps; the adopted piece partition retains those inside free min-maps.

Frozen new gate SHA256: B78EAAF3A53DE7B0A70E1A2F3DA12B5246437962F7909000301309190468FF82.
Its first run (s203-first-gate.log) passed. Counts are 5/7/7/8/12/2/2 for gap270,
gap275.625, gap270 z=1, step270, step275.625, ice and lane. The gate covers all
2^6..2^18 grids, +/-8 ULP cut neighbours, 256 ULP hold endpoints, exact exit
membership, resource enclosure, native pin proofs and second-wall convergence.
The witness remains exact: clamp X=40592cccccc00000; hold X=40592cccccbfffff.

Old pieces SHA256: 5C04C53881D0C797F3FE9EEC8AFB831B99781F5543E3790FC365D4565F98034D.
The old pieces, cut-neighbour pin witness, KT11 and translation tests remain
unchanged recorded reds of the old object. The coarse-grid pin-erasure diagnostic
actually passes; it missed the hold, and is not misreported as red. Review also
checks the original full-solve cost assertions and footprint mutation bodies.

| Gate | Exit | Expected red | Correctly recorded | Reason |
|---|---:|---|---|---|
| s203-u2-final | 0 | False | True |  |
| s203-pieces-old-object | 101 | True | True | S188 erasing-pin/event-only object superseded by adopted S203 point-or-hold contact partition; frozen assertions unchanged |
| s203-pin-witness-old-object | 101 | True | True | The native clamp/hold pair still falsifies the old erasing-pin premise; new gate preserves both exact values |
| s203-kill11-old-object | 101 | True | True | Old input-signature plateaus grow under refinement; S203 carries flags in the output image |
| s203-translation-old-object | 101 | True | True | Rounded addition is monotone, not a translation; exact maps retain parity |
| s203-kill10-current | 0 | False | True |  |
| s203-core-final | 0 | False | True |  |
| s203-maps-stairs | 0 | False | True |  |
| s203-workspace-full | 101 | True | True | Workspace contains the frozen S188 old-object red; run independent workspace with only that test excluded |
| s203-workspace-independent | 0 | False | True |  |
| s203-core-mutants | 0 | False | True |  |
| s203-footprint-mutants | 0 | False | True |  |
| s203-cost-final | 0 | False | True |  |
| s203-review | 0 | False | True |  |
| s203-generator-map-final | 0 | False | True |  |
| s203-ownership-debug-final | 0 | False | True |  |
| s203-workspace-current-source | 0 | False | True |  |

All logs, input hashes, invariant output and refusals are in D:\path-tracing-evidence\u2b.
The pre-freeze oracle is now an explicit ignored diagnostic, superseded in the
normal loop by the frozen pin regression; its original run remains archived.
The footprint driver reuses a successful complete workspace baseline only if its
entire engine input hash set still matches. All mutation bodies and assertions run.
The full workspace includes the old frozen red. The independent workspace excludes
only that obsolete gate and the already-run pre-freeze oracle diagnostic; new checks stay enabled.
All 13 footprint mutants and all 3 core mutants are caught by running assertions.
The actual cached-answer bypass fails through effect counters; native-tick and bundle-hit
mutants also fail. Source bytes are restored and hash-checked. No trace-label-only substitution.

```text
U2_KT10_REPLAY best=158 human=169 margin=11 law_hash=[4a, f5, d1, 13, 61, 19, 85, 5f, a5, bd, 1c, bb, c0, ad, 65, d1, fd, d4, 11, 6b, 6, 93, 33, 57, 38, 4b, 8d, 29, 79, 83, c1, 49] cold_template_bytes=3082 cold_state_bytes=4080 edges=5 acting_native_ticks=0 first_touch_exact=true
```

## Warm million line: distributions, not averages

Microseconds; n=100 per goal. Median uses the two central samples; p99 is nearest rank.
The measured boundary is the complete public solve, including output and profile clocks.
Verification materializes states after the measured boundary. Warm native ticks are zero.

| B cell | Min us | Median us | p99 us | Max us | Copies max | Allocations max | Median / 5 us |
|---|---:|---:|---:|---:|---:|---:|---:|
| [1000000, -61, 0] | 56.6 | 57.2 | 95.4 | 100.1 | 0 | 0 | 11.44 |
| [0, -61, 1000000] | 67.9 | 69.4 | 108.8 | 113.4 | 0 | 0 | 13.88 |
| [1000000, -61, 330000] | 93 | 97.5 | 140.4 | 140.8 | 0 | 0 | 19.5 |

| B cell | Stage | Min us | Median us | p99 us | Max us |
|---|---|---:|---:|---:|---:|
| [1000000, -61, 0] | entry | 3.3 | 3.3 | 6.5 | 7.2 |
| [1000000, -61, 0] | bearing | 10.2 | 10.5 | 16.6 | 17.1 |
| [1000000, -61, 0] | inverse | 30.3 | 30.8 | 53.7 | 63.7 |
| [1000000, -61, 0] | output | 11.8 | 12 | 19.3 | 21.3 |
| [0, -61, 1000000] | entry | 4.1 | 4.2 | 8.8 | 10.3 |
| [0, -61, 1000000] | bearing | 10.1 | 10.5 | 14.7 | 14.9 |
| [0, -61, 1000000] | inverse | 41.1 | 42 | 67.8 | 69.8 |
| [0, -61, 1000000] | output | 11.7 | 12.1 | 19.5 | 20.7 |
| [1000000, -61, 330000] | entry | 4.7 | 4.9 | 9.8 | 10.9 |
| [1000000, -61, 330000] | bearing | 3 | 3 | 4.1 | 4.3 |
| [1000000, -61, 330000] | inverse | 61.2 | 64.3 | 95.8 | 98.9 |
| [1000000, -61, 330000] | output | 23 | 24.1 | 33.6 | 39 |

The floor is missed. Remaining entry cost is motor/label lookup and exact transient
readouts. Axis bearing still enters the checked floor-sum fallback (30 quotient
queries in U1_WORK); the oblique solve takes the determinant path. Inverse cost
is phase enumeration, repeated exact readouts, approach-run reconstruction and
linear terminal-row lookups. Output still evaluates envelopes and walks binades
for boundary events, although those event walks now also supply final coordinates.
The oblique excess is therefore not reduced to the expected 1-2 us second-axis
crossing cost. The tables show the larger inverse/output work still present;
no claim that the extra work is required by nature, or that the floor was reached.

## Files and exact replay

- `docs/HANDOFF.md`
- `engine/law/Cargo.toml`
- `engine/law/src/codec.rs`
- `engine/law/src/codec_state.rs`
- `engine/law/src/contact.rs`
- `engine/law/src/footprint.rs`
- `engine/law/src/lib.rs`
- `engine/law/src/movement.rs`
- `engine/law/src/pattern.rs`
- `engine/law/src/shapes.rs`
- `engine/law/src/state.rs`
- `engine/law/src/state_reads.rs`
- `engine/law/src/world.rs`
- `engine/plan/Cargo.toml`
- `engine/plan/src/events.rs`
- `engine/plan/src/families.rs`
- `engine/plan/src/family_wave.rs`
- `engine/plan/src/goal.rs`
- `engine/plan/src/inline.rs`
- `engine/plan/src/joints.rs`
- `engine/plan/src/lattice.rs`
- `engine/plan/src/lib.rs`
- `engine/plan/src/solve.rs`
- `engine/plan/src/source.rs`
- `engine/plan/src/state_view.rs`
- `engine/plan/src/store.rs`
- `engine/plan/src/terminal.rs`
- `engine/plan/tests/u2.rs`
- `engine/plan/tests/u2b/consumers.rs`
- `engine/plan/tests/u2b/pieces.rs`
- `engine/plan/tests/u2b/pin_gate.rs`
- `engine/plan/tests/u2b/pins_point_hold.rs`
- `engine/plan/tests/u2b/premises.rs`
- `engine/plan/tests/u2b_cost.rs`
- `engine/plan/tests/zones_prereqs/u1.rs`
- `engine/words/src/program.rs`
- `jobs/u2b/continue-hold.md`
- `jobs/u2b/core-mutants.ps1`
- `jobs/u2b/finalize.ps1`
- `jobs/u2b/footprints.ps1`
- `jobs/u2b/last.md`
- `jobs/u2b/prompt.md`
- `jobs/u2b/report.ps1`
- `jobs/u2b/review.ps1`
- `jobs/u2b/run.ps1`
- `jobs/u2b/s203-oracle-counts.txt`
- `jobs/u2b/verify.ps1`

The integrator saved the earlier implementation as 95552dc during the usage pause.
The final native-footprint inventory, aim-entry and unwind ownership repairs
are additional working-tree changes, with their regressions and final gate rerun.
File changes above are measured from fef8c6e, the original U2 base.

Pre-existing MCA edits and backup files are untouched and hash-checked. No launcher,
game, server, client, network or sub-agent was used by this builder. The builder
made no commit, merge or push. All runs
are offline at Normal priority with target D:\path-tracing-target\u2b.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2b/verify.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2b/report.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File jobs/u2b/finalize.ps1
```

Run records use content hashes; final-inputs-s203.json maps replay inputs to
content-addressed copies. Archived older records retain the original blocker
and first failures; this file states the adopted object and current results.

STATUS: DONE
