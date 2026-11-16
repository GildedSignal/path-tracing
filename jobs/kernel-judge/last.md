# Kernel judge: P2 (T2) and P4 (T4)

Target dir `D:\path-tracing-target\kernel-judge`; offline; nothing launched; worktrees restored (only the pre-existing dirty region .mca files remain).
Diff base: `git diff wt/3.2d-3-P1...wt/3.2d-3-PN` (merge-base 08ccc41; P1 has since moved to 024a39a, so a two-dot diff shows noise).

## P2 (T2): binade integer maps and inverse stop

VERDICT: PASS WITH FINDINGS

1. Diff: P2 touches only `engine/plan/tests/zones_prereqs/t2.rs` and `jobs/3.2d-3-P2/{derivation.md,last.md,report.ps1,verify.ps1}`. No forbidden path.
2. Test read in full (373 lines):
   - Reference is independent: `law::try_tick` replay of the real world (t2.rs:331-334) compared bit-for-bit (position bits, motor words, support block, t) with the fold, at 594 witnesses (positive 2^19-100..2^21+100, negative, -50..+50, +-0.0, +-1 ULP at 2^19/2^20/2^21, 144 slab/body edge neighbours at all 12 vertical phases). Inverse first-touch compared against native touch for 42 goals at y=-61/-60/-59.
   - Cycles by quotient/remainder (Graph::displacement 79-85), events by integer division (safe_cycles 86-100, inverse_edge 163-179), trillion-count test (299-301) proves no q-loop. Tie parity witnesses with odd/even N and negative d (277-281). Subnormal/signed-zero covered (283-291, 302).
   - inverse_edge is checked against an independent bisection oracle (`first_true`, 306-317) over 384 thresholds.
   - Dimensions come from the law: `Aabb::player(.., Pose::Standing.height()).max.x` (304, 196, 216, 350), no 0.3/1.8 literal in the code. Radius printed 0.30000001192092896.
3. Run: `cargo test --offline -p plan --test zones_prereqs -- --nocapture`: 2 passed (t1, t2), 59.8 s. Workspace: 211 passed, 0 failed.
   My timings: 100 blocks 278 ticks, 8 segments, median 80 us; 10^4: 28061 ticks, 15 seg, 162 us; 10^6: 2806320 ticks, 21 seg, 238 us (last.md reports 17/36/131 us; machine load differs, shape is the same: segments grow with log distance, not distance).
4. Mutations (reverted, file clean):
   - M1 `safe_cycles` bound `(hi-1-v)` -> `(hi-v)` (t2.rs:94): test STAYED GREEN. See finding 1.
   - M2 perturbed remainder in `quotient` (rem+1 when n odd): FAILED at t2.rs:281 (tie-parity witness). Poison switch (env, test-only) also exists per last.md.
Findings:
- (low, kept) M1 survived. That bound is only reached at the exact region edge; the pair of edge-exactness checks does not pin the `-1`. Smallest fix: add one witness where `hi-1-v` is divisible by the step (start at HIDDEN..LIMIT-k*step, assert bits against a stream of one binade crossing). Not blocking: result bits are checked at every crossing already; M1 may be an equivalent mutant at these operands.
- (low) The three measured distances (100, 10^4, 10^6 from x=0.5, t2.rs:362-371) are not cross-checked against a native stream; the 280-tick and 4.4M-tick streams check the same code at other starts. Fix: stream-check distance 100 and 10^4 (cheap, 28k ticks).
- (low) t2.rs:359 witness `goal.touched(&witness,&witness)` plus a hand-typed -0.301000005 is a weak, typed-constant sanity line; the real detector coverage is the 144 edge neighbours. Delete it or derive the position from the law's radius.
- (info) The fold asserts a positive-axis domain (t2.rs:191); negative axis is covered only by the position fold, not by inverse families. Declared, in scope for +x.
- (info) The `assert_eq!((first,period),(263,12))` (255) pins P1's measured cycle; it is a measurement guard, not a law constant.

## P4 (T4): the food guard

VERDICT: PASS WITH FINDINGS

1. Diff: only `engine/plan/tests/zones_prereqs/t4.rs` (+42) and `jobs/3.2d-3-P4/last.md`. No forbidden path. `jobs/3.2d-3/verify.ps1 -Case T4Food` NOT added, so the spec's live harness is absent (accepted: moves to U4); the footprint frozen gate and prerequisites verifier were not run either (declared).
2. Test read in full (42 lines):
   - Starts from `from_observed_bits` with the spec's three bars (7/0.0/3.99, 6/0.0/0.0, 20/5.0/0.0), 240 sprint-jump `law::try_tick` ticks on straight800 (difficulty asserted 0, t4.rs:10). Asserts food unchanged, `sprinting == (food>6)` every tick, wraps>0 for food 7 and 20 (2 and 1 observed), `state_answers` food answer stable, answers 7==20, 6 differs. Unknown food word refused for each start (19-20).
   - The Peaceful claim matches the law: `food_tick` skips the food debit when `difficulty()==0` (`engine/law/src/movement.rs:1263-1267`), and `update_sprinting` gates on `food_level > 6` (movement.rs:219). The test itself cites neither; last.md cites food_tick and a lab result (reported, not measured).
   - Weaknesses: (a) the reference for the guard is the typed literal `food>6` (t4.rs:30,33,34) instead of reading the law's threshold; (b) the health/saturation/exhaustion perturbation block (37-40) only checks a bits round trip, weaker than spec step 2 (no check that unknown health/sat/exh words are refused, no `fed` rejection); (c) `state_answers == answers.last()` (31) is near-trivial since food never changes; (d) food stays constant 240 ticks only because `try_tick` applies no server regeneration events (regeneration is an event, movement.rs:1173); that is consistent with the test's "no saturation prediction" and live behaviour is U4's.
3. Run: `cargo test --offline -p plan --test zones_prereqs -- --nocapture`: 2 passed (t1, t4), food=7 wraps=2, food=6 wraps=0 guard=false, food=20 wraps=1. Workspace: 211 passed, 0 failed.
4. Mutation (reverted, file clean): threshold `food>6` -> `food>7` in the assertions: FAILED at t4.rs:30 (guard tick). Good. (Mutating the law threshold was not done: forbidden path.)
Findings:
- (medium) Typed threshold 6 in the assertions. Smallest fix: derive both sides from the law, e.g. compute the expected sprint from a probe state that differs only in `food_level` (7 vs 6) through `state_answers`/`update_sprinting`, or cite movement.rs:219 and read the constant via the law if exposed.
- (low) Complete spec step 2: add refusal checks for unknown health/saturation/exhaustion words and a negative control (flip difficulty to 1 in a copy of the world, assert the food drains at the wrap), proving the guard is the law's Peaceful rule and not an artifact.
- (low) Add a code comment citing movement.rs:1263-1267 where the Peaceful claim is made.
- (info) Live part not run: moves to U4.
