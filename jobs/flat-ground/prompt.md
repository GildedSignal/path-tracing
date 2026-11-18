# Flat ground as the docs say: the gait stored once, the yaw put in, nothing relearned per command

Builder: Astra 6.2. Worktree `D:\path-tracing-wt\flat-ground`, branch `wt/flat-ground`, from `wt/warm-floor` at `75b2b90` (your previous run's work, committed as it stood, then the integrator's test cut: S235, "cut down the tests by a lot … it's literally cheaper to fix than to prevent the issue"). The integrator commits; you never run `git commit`, `launch.ps1` or `gpt.ps1`, and use no network, game, server or client.

## His words (verbatim)

- S225: "each knew command on the flat ground should take µs IN TOTAL."
- S229: "astra should get closer to the fermi since last times we optimized only a portion thinking that getting hot was once per world and not once per command. because ms for a straight line is way too long"
- S230: "just change the input parameters and that's done a gait can be straight line one with no yaw change so you just input the yaw you care about"; "if there's relearning that means either the application of the learning is bad or the learning itself is bad"; "no over-precision we spend compute where we need it"
- S232 (on the stop and the three gaits): "it should use the biggest bracket and if it fits use it, that's it period"; "this is so much overthinking"

## Why this job exists (docs/reference/postmortem-2026-11-18.md)

Today a new command on flat ground costs 68 ms (55 ms while moving; 22 of 60 retargets refused). About 59 ms of it is `store.rs::prepare` re-running about 4,700 law ticks: about 83 small runs per command, because kernels are keyed by `(label, program, yaw_index)` plus the start state's exact bits, and `solve.rs:125-145` builds two yaw kernels, three rival words and stop rows for every phase of every gait before any decision. That breaks four rules of the living docs. **Your job is to make the code obey them, not to speed up the current structure.**

## The rules this job must obey (quote them in your report, with how the code now obeys each)

1. **Store the generator, not the generated** (`docs/engine.md`, "Continuous where it helps"): "What is stored is the recurrence and its parameters … keyed by those parameters and shared by every place and bot that uses them. Arcs and trajectories are derived when looked at, never stored tick by tick. Differences below `aiStep`'s thresholds … vanish in the law itself, so states that differ only there merge."
2. **The aim is an argument, never part of an entry key** (`docs/vocabulary.md` section 5, S202).
3. **Words live in velocity space** (`docs/representation.md` section 10): "the tick is `v ← d·v + a·u(yaw)`, a contraction onto the gait; position is its running sum, read out by exact integer maps per binade."
4. **Biggest word first, with its count** (`docs/learning.md` section 0, step 3; `docs/vocabulary.md` section 5): "Take the biggest word the kit holds for this displacement. One presence query over its worldtube returns its count and the first exception that breaks it." Not three rival words; not stop rows built in advance.
5. **One mechanism** (`docs/learning.md` section 0, kill test 8): no branch on terrain kind, flat ground, distance, axis or straight line. What you build is step 3 of the one loop for any uniform ground; an exception from the corridor query goes to the existing cut and joints machinery, unchanged.

## What the law says (checked by the integrator, `engine/law/src/movement.rs`)

On ground without horizontal contact, each horizontal axis is its own scalar recurrence: `v ← (v + input(yaw)) × f × drag` (`:438`, `:452-455`), plus `−sin(yaw)·0.2` / `+cos(yaw)·0.2` on a sprint-jump tick (`:379`), with `sin` and `cos` from the law's table (`math.rs`). The vertical motion, the on-ground flag and so `f` per tick are yaw-free on uniform ground. The axes meet only at `aiStep`'s zeroing (`:168`, h² < 9×10⁻⁶), which merges near-rest starts (rule 1). There is no horizontal collision, so no minor-collision test.

## What to build

1. **A gait is stored once per (ground's generator parameters, program):** the yaw-free per-tick coefficients (the multiplier, which ticks carry the jump term, the vertical cycle and flags), found by running the law once on the first command on that ground. Never per yaw, never per start velocity.
2. **A yaw is put in:** two table lookups give each axis's input term; each axis is then the stored scalar recurrence, run from the observed velocity (standing or moving: the same code) until it joins the gait's cycle, and read out by the existing exact integer maps. No `PlayerState` per tick, no world recording, no allocation.
3. **The command, per rule 4:** bracket the bearing by its two table yaws (`lattice::bracket`, kept); take the biggest word the kit holds (sprint-jump while food allows, the law's own guard); one corridor query gives its count to the first exception or to B; the counts on the two yaws come from the existing lattice; T is the first tick the body touches B (`Goal`, unchanged), read from the same recurrence. Nothing else is prepared.
4. **Delete what the rules forbid:** the yaw- and bits-keyed kernel entries, the incumbent and terminal bundles of `solve.rs:125-145`, and any code that only served them. Deleting is the point: report net lines.
5. **Keep:** your previous run's touch on both axes, the counters and `real_time.rs`. **Revert** the stop-at-touch-phase readout that moved the retarget to `[-388,-61,-317]` from T 1666 to 2103.
6. **Tests:** the suite is now only what the game judges (`law/tests/replay.rs`, `law/tests/jar.rs`, `plan/tests/hard_track.rs`, `plan/tests/deviation.rs`, `world/tests`) plus `real_time.rs`. Add no test of internal structure; extend `real_time.rs` for the gates below.

## Gates (they fail when a rule is broken)

- **The headline (S225):** with one living store, the world in memory, 60 new commands from standing and 60 retargets while moving (bearing uniform, distance log-uniform 10² to 10⁶ blocks, `real_time.rs`): thinking time per command, min / median / p90 / max, against the floor of about 1–10 µs. Target: microseconds in total for every command after the first on that ground. Report the gap in orders of magnitude and what the remaining time is spent on.
- **Nothing relearned (rule 1):** after the first command on a ground, law ticks prepared per command = 0, for every new yaw and every moving start. Counted, not estimated.
- **No refusals:** 0 of 60 moving retargets refused on flat ground.
- **Expected against seen (S230, S235):** every emitted plan is replayed by `law::try_tick` in `real_time.rs`, outside the timed region, and touches B on the predicted tick. A delta is a bug to fix, reported with the goal and both ticks. No sweep of all 65,536 yaws, no planted-fault test.
- **The suite stays green,** his hard-track shortcut at 158 ticks included.

## Stop and write `STATUS: BLOCKED`, never patch

If a rule above cannot hold without a branch, a tolerance or a second path, stop at once, write the case (inputs, the two values) and why, and do nothing else. Do not optimise around it.

## Output

`jobs/flat-ground/last.md` in the main checkout (`C:\Users\PASCAL\Desktop\path tracing\jobs\flat-ground\last.md`), written as you go: per rule, how the code now obeys it (file:line); each gate with its numbers; net lines. End with `STATUS: DONE` or `STATUS: BLOCKED: <why>`.
