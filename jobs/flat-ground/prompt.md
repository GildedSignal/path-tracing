# The new core flies in his game, then flat ground as the map says: the gait stored once, the yaw put in

Builder: Astra 6.2. Worktree `D:\path-tracing-wt\flat-ground`, branch `wt/flat-ground`, at `main`. The map (`map/`, above and in `map/detail/`) is the truth; this job is one step of it. The integrator commits and flies; you never run `git commit`, `launch.ps1` or `gpt.ps1`, and use no network, game, server or client.

## His words (verbatim)

- S239: "always test in game that's the only kind of test that matters"
- S225: "each knew command on the flat ground should take µs IN TOTAL."
- S230: "just change the input parameters and that's done a gait can be straight line one with no yaw change so you just input the yaw you care about"; "if there's relearning that means either the application of the learning is bad or the learning itself is bad"; "no over-precision we spend compute where we need it"
- S232: "it should use the biggest bracket and if it fits use it, that's it period"; "this is so much overthinking"

## Part 0, first: the new core in his game

Today only the old planner reaches his game (`plan-track.rs` calls `flight::fly_with_budget`; the tick script writer is `flight.rs:62-69`; `scripts/play-track.ps1` hands the script to the mod). `solve` has never flown.

- Add `--mode solve` to `plan-track` (same track loading and settled start; `solve` with one living store, as `real_time.rs` uses it; words × counts expanded into the same tick script by running the law). Same file format, so the mod plays it unchanged. Print `SENTENCE <predicted ticks> thinking <µs>`. Add `solve` to `play-track.ps1`'s `-Mode`.
- **The clock, decided by the map** (`goal.md`: T counts from the click's tick). The predicted T counts ticks from the settled start on A to the first touch of B. The track clock (`bot/src/pathtracing/TrackDetector.java:40-45`) starts on the first sample *off* A, one tick late: `straight100` reads 299 against 300 predicted (your report of the last run). Make the detector's origin the last tick on A, so both count the same ticks. It is the same rule for his runs and the bot's, so his recorded times rise by one tick each; recompute them from `lab/tracks/runs/` and report the before and after. No −1 anywhere else.
- Gate (yours, offline): for `straight100`, `straight300`, `straight800` and `stonesimple`, the exported script replayed through `law::try_tick` touches B on the predicted tick.
- Gate (the integrator's, in his game): the track clock's T equals the predicted T. This flight is the gate of the whole job.

## Then flat ground: why

A new command on flat ground costs 68 ms standing and 55 ms moving, against a floor of 1–10 µs (`goal.md`), and 22 of 60 moving retargets are refused. About 59 ms is `store.rs::prepare` re-running about 4,700 law ticks per command, because kernels are keyed by `(label, program, yaw_index)` plus the start state's bits, and `solve.rs:125-145` builds two yaw kernels, three rival words and stop rows for every phase before any decision. That breaks principles 5, 6 and 7 of the map: store the generator (`detail/engine.md`, "Continuous where it helps"), the aim is an argument and never a key (`vocabulary.md`, "aim"), words live in velocity space (`detail/representation.md` section 10), the biggest word first with its count (`architecture.md` section 3, step 3), one mechanism. **Make the code obey the map, not the current structure faster.**

What the law says (`engine/law/src/movement.rs`): on ground without horizontal contact, each horizontal axis is its own scalar recurrence `v ← (v + input(yaw)) × f × drag` (`:438`, `:452-455`), plus `−sin(yaw)·0.2` / `+cos(yaw)·0.2` on a sprint-jump tick (`:379`), from the law's sine table. Vertical motion, the on-ground flag and so `f` are yaw-free on uniform ground. The axes meet only at `aiStep`'s zeroing (`:168`, h² < 9×10⁻⁶), which merges near-rest starts.

## What done looks like

1. A gait is stored once per (ground's parameters, program): the yaw-free coefficients, found by running the law once on the first command on that ground. Never per yaw, never per start velocity.
2. A yaw is put in: two table lookups give each axis's input term; each axis runs the stored recurrence from the observed velocity (standing or moving: the same code) until it joins the gait's cycle, read out by the existing exact integer maps. No `PlayerState` per tick, no allocation.
3. The command: bracket the bearing by its two table yaws (`lattice::bracket`); take the biggest word the kit holds (sprint-jump while food allows); one corridor query gives its count to the first exception or to B; T is the first tick the body touches B (`Goal`, unchanged). An exception goes to the existing cut and joints, unchanged. Nothing else is prepared.
4. What the map forbids is deleted: the yaw- and bits-keyed kernel entries, the incumbent and terminal bundles of `solve.rs:125-145`, and what only served them. Report net lines.
5. Kept: the touch on both axes, the counters, `real_time.rs`. Reverted: the stop-at-touch-phase readout that moved the retarget to `[-388,-61,-317]` from T 1666 to 2103.
6. Tests: only what the game judges (`law/tests/replay.rs`, `law/tests/jar.rs`, `plan/tests/hard_track.rs`, `plan/tests/deviation.rs`, `world/tests`) plus `real_time.rs`, extended for the numbers below.

## Numbers to report (from `real_time.rs` and `scripts/scoreboard.ps1`, raw)

- Thinking per new command, 60 from standing and 60 retargets while moving (bearing uniform, distance log-uniform 10² to 10⁶): min / median / p90 / max, beside the 1–10 µs floor, the gap in orders, and where the remaining time goes.
- Law ticks prepared per command after the first on a ground: 0, counted.
- Moving retargets refused: 0 of 60.
- Every emitted plan replayed by `law::try_tick` outside the timed region touches B on the predicted tick; a delta is a bug, fixed, reported with the goal and both ticks.
- The suite green, his hard-track shortcut at 158 ticks included.

## The one stop

Stop only if flat ground cannot be one mechanism: if obeying the map needs a branch on terrain kind, distance, axis or straight line, or a tolerance on a predicted tick. Then write the case (inputs, the two values) and stop. Anything else (a definition, a file, a choice the spec leaves open) you decide against the map, write down with its reason, and keep going.

## Output

Write your report as you go to `jobs/flat-ground/report.md` in your worktree: per point of "done", how the code now does it (file:line); the numbers; the clock change and his times before and after; net lines.
