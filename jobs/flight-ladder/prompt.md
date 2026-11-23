# The flight ladder: steps, fences and stairs in his game

**Goal:** a click whose way crosses a step, a fence or a stair flies in his game with the track clock's T equal to the predicted T, and his tracks climb one rung at a time on `main`.

Builder: Astra 6.2. Worktree `D:\path-tracing-wt\flight-ladder`, branch `wt/flight-ladder`, at `main`. The map (`map/`, above and in `map/detail/`) is the truth; this job is step 3 of `map/now.md`. The integrator commits and flies; you never run `git commit`, `launch.ps1` or `gpt.ps1`, and use no network, game, server or client. Copy the git-ignored `engine/world/tests/fixtures/world` from the main checkout if your worktree lacks it.

## His words (verbatim)

- S239: "always test in game that's the only kind of test that matters"
- S166: "many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that"
- S161 and S131: one mechanism; "the terrains complexity should be a mirror of compute's complexity"
- S232: "it should use the biggest bracket and if it fits use it, that's it period"

## Where it stands (on `main`)

Flat ground flies: `straight100/300/800` and `stonesimple` land on the predicted tick; a new command costs ~13 µs. The stick runs `solve` in the bot's own process (`engine/native/src/resident.rs`). Everything else is refused: `solve.rs:269` turns the corridor's first exception into `PlanOutcome::NeedsJoints` and only enqueues it, because nothing builds the joints of a new site at a click. And a block whose shape the law calls non-local (stairs) panics the corridor instead of becoming an exception.

## What done looks like

1. **The cut and the compose of the one loop** (`architecture.md` section 3, steps 4–5; `detail/learning.md` section 0) run at a click: the exception's joints split the problem, each part is `solve`, sets come backward in min-plus, the point goes forward. A new site is learned by one backward wave, kept by content key, and never searched again (principle 5). No A\* in acting; no per-terrain branch.
2. **A non-local shape is an exception, never a panic**: the corridor reports it like any other different cell, and the cut handles it.
3. **The ladder**, each rung offline first: for each track below, `plan-track --mode solve` emits a script that `law::try_tick` replays onto B on the predicted tick. Climb in this order and report every rung you reach, with predicted T, his best T (from `lab/tracks/runs/`), thinking time (first command on the site, then the same command again), and law ticks prepared: `grassjumpeasy`, `flowers`, `stonestaircase`, `fences`, `fencestraightbridge`, `fencemaze`, then `practice1`–`practice10`, `birchparkourmedium`, `mangrovejumpeasy`, `doorcorridor`, `trapdoorparkour`. A rung you cannot reach is written as its first refusal (the cell, the word, the bracket) and you go to the next.
4. **The planner's settled start matches the bot**: saturation 20.0 as the bot has it, not 5.0 (first deviation at tick 0 on every flight today).
5. **The dead planner is deleted** once `solve` covers what it covered: `flight.rs` modes `3`, `5` and `b`, and what only they reach (`search.rs`, `polish.rs`, `family_wave.rs` and others: decide by what the compiler and the kept tests say). `plan-track` keeps only `--mode solve`. Report net lines; the map's principle 21 is "less code is better on every level".
6. **The ES audit wins that sit on this path** (`jobs/es-audit/report.md`, sections 1, 4 and 6): no tick-by-tick replay from tick 0 in a slide, the airborne arc by its certificate rather than per-tick boxes, no per-click settle loop. Take each only where it lies on the rungs you climb.
7. Tests: only what the game judges (`law/tests/replay.rs`, `law/tests/jar.rs`, `plan/tests/hard_track.rs`, `plan/tests/deviation.rs`, `world/tests`) plus `real_time.rs`. The flat-ground numbers must not regress (median ≤ 15 µs per new command, 0 refused).

## The one stop

Stop only if a rung needs a second mechanism: a branch on terrain kind, block name or track, or a tolerance on a predicted tick. Then write the case (inputs, the two values) and stop. Anything else you decide against the map, write down with its reason, and keep going.

## Output

Write your report as you go to `jobs/flight-ladder/report.md` in your worktree: per point of "done", how the code now does it (file:line); the ladder table (track, predicted T, his best, thinking, prepared ticks, or the first refusal); net lines. The integrator flies each rung you reach in his game; the flight is the gate.
