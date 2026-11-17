# Judge: the simplest case right (step 2b)

You judge GPT Astra's build, commit `e6de417` (final; `95552dc` was the state the integrator swept) on branch `wt/u2b-astra`, worktree `D:\path-tracing-wt\u2b-astra`, against its base `fef8c6e` (step 2). Fresh context; you are Sonnet, the other family. You edit nothing in the worktree: run mutations on a scratch copy (`D:\path-tracing-scratch\u2b-judge\`), build with `CARGO_TARGET_DIR=D:\path-tracing-target\u2b-judge`, `--offline --release`.

Read first: `docs/PRINCIPLES.md` (goal, philosophy), `docs/plan.md` (step 2b's row), `AGENTS.md` (no patches, no new search code, seal only what can't be corrected), then the spec `jobs/u2b/prompt.md` and the adopted re-derivation it now follows, `jobs/pin-rederive/proposal.md` with `jobs/pin-rederive/addendum.md` (S203: the pinned exit is a point or a hold; S202: the aim is never in an entry key). The living statements: `docs/representation.md` line ~179 and `docs/learning.md` kill test 11. Astra's record: `jobs/u2b/last.md`.

## Evidence already run (do not rerun the 12-minute workspace sweep)

The integrator ran `jobs/u2b/verify.ps1` on 17 November: log `D:\path-tracing-evidence\u2b\integrator-sweep-20261117.log`, record `D:\path-tracing-evidence\u2b\gates-s203.json`. Every gate matched its expectation. Four gates stay red by design: the old objects `u2_pieces_computed_and_closed`, `u2_pin_cut_neighbourhood_witness`, `u2_kill11_refined_bisection_diagnostic`, `u2_translation_premise`. The integrator checked that the last two are byte-equal to `fef8c6e`, and that the new frozen gate `engine/plan/tests/u2b/pins_point_hold.rs` hashes to `B78EAAF3…FF82`.

## What to decide

1. **The frozen gate can fail.** Mutate the contact outcome (turn a hold into a clamp, drop a cut, shift an exit by 1 ulp) on the scratch copy; each must turn `u2_pins_point_and_hold` red. Is the gate's oracle the native law (`law::try_tick`), not the family code it judges?
2. **The old reds are superseded, not weakened.** `pieces.rs` is new in this commit (written under S188 before S203 was adopted). Confirm its assertions encode the disproved erasing-pin object and that nothing passing now depends on it.
3. **Zero copies and zero allocations are counted, not asserted by fiat.** Add one `PlayerState` clone and one `Vec` allocation in the warm path on the scratch copy; `u2_warm_zero_copies_and_allocations` must catch each.
4. **The stop on B is looked up, never re-searched,** in a warm solve (`terminal.rs`, `JointStore`). Find any per-goal search left.
5. **One mechanism.** No terrain-kind, straight-line, distance or axis branch in `solve`; no tolerance, epsilon or retry added to pass a check (`CONTACT_ZERO` must be the law's 1e-7, taken from Mojang's comparison, not a new number). The aim does not enter any entry key.
6. **The timings.** `U2B_WARM` reports 57 µs on the axis while the core's `U1_SOLVE` warm reports 112/124/167 µs (x, z, oblique) in the same sweep. Say what each measures and which is the warm million line the plan's gate means. Against the ~5 µs floor, name what remains, with the profile lines.
7. **Merge risk.** List the law files changed (`movement.rs`, `state.rs`, `shapes.rs`, `contact.rs`…) and whether the recurrence itself changed or only observation was added; any change to the recurrence is a finding unless it is bit-exact with Mojang.

## Output

Write `jobs/u2b-judge/last.md` in `C:\Users\PASCAL\Desktop\path tracing`: first line the verdict, **PASS**, **PASS WITH FINDINGS** or **FAIL**; then findings ranked by severity (HIGH/MEDIUM/LOW/INFO), each with `file:line`, the concrete failure, and the mutation or command that shows it. Plain names; a code only in backticks after its name. No edits to the worktree, no commits, no pushes. Under 120 lines.
