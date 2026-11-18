# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Astra 6.2, `jobs/flat-ground`** (worktree `D:\path-tracing-wt\flat-ground`, at `main`): the new core flies in his game, then flat ground in microseconds. The clock is decided by the map (T from the click): the track detector's origin is the last tick on A, for his runs and the bot's alike. Codex week at 89%: the last Astra run before its reset.
- **Sonnet judges** (Antigravity, two at a time): `hard-track-2-judge` and `track-face-judge` running; `freeze-damage-judge` and `build-preview-judge` written, launched when a slot frees. Each gives MERGE, MERGE PART or ARCHIVE.
- **Argon, `jobs/map-lint`:** the map against his verbatim turns, once.
- **Hydrogen's port build A** (`wt/port-build-a`, `055e7b7`: mining speed, boat, elytra boost, ender pearl in the law, not wired into `movement.rs` or the planner): no flight can show it yet, because nothing plays these mechanics; it merges when step 5 (mining) or step 8 (boat, elytra, pearl) consumes it. One judge finding open: `break_ticks` is `ceil(1/p)` in f64, the game breaks at `p·n ≥ 1` in f32 (obsidian by hand 5000, not 5001).

## The overhaul (`jobs/overhaul/plan.md`, approved S240): state

Built and on `main`: the map (this folder; 22 KB loaded whole, history in `archive/`), AGENTS.md and the README rewritten, the hooks of `work.md` (card and his words on every message, the launch guard, the job watcher, the map and net-line commit checks, the push guard), the scoreboard (`scripts/scoreboard.ps1`), companion agency in `gpt.ps1` and `gemini.ps1`, GLM without its checker or lenses, tests cut to what the game judges, memory cut to 11 files, 220 dead worktrees and 51 branches archived (`D:\path-tracing-archive`, `archive/*` tags), history stripped from `map/detail/`. The meters: `/meters` opens a pane in the app (a mod in `C:\Users\PASCAL\.claude\dev-mods\…\meters-band`; the app's own Context panel takes no mod content).

Checks against these fixes: `jobs/overhaul/checks.md`.

Left: delete the old A\* planner (`plan/src/flight.rs` modes 3/5/b, `planner.rs`) and dead `words` code, after Astra's job; Argon lints `map/` against his verbatim turns once.

## Next, in order

1. Fly the new core in his game once Astra's export builds: `straight100/300/800`, `stonesimple`, ten new flat goals. The clock's T must equal the predicted T, with no deviation. `scripts/scoreboard.ps1` reads the flight log.
2. Judge and merge Astra's work; then delete the dead planner code.
3. His tracks as inputs (Sol, `jobs/loop-tracks`), held until Astra's work is in.
4. The bot's runner: words × count in the client, re-anchoring at events, the goal changed mid-run (R-retarget).

## The order of work (steps of the one loop)

| Step | State |
|---|---|
| 1. the core: `solve`, words with counts, the word store | built, on `main` |
| 2. joints and sets | built, on `main` |
| 2b. the simplest case right | built, on `main`; per new command 68 ms against a ~10 µs floor |
| 3. the loop on his tracks | Astra's job, then Sol's track inputs |
| 4. the bot's runner | next |
| 5. editing the world (place, break, interact; the ledger) | the port feeds it |
| 6. the far map | after 4 |
| 7. other players' light cones | after 6 |
| 8. boats, elytra, pearls, portals | the port feeds it |
| 9. the whole machine: threads, then the GPU | last |

## Waiting for a judge or for him

- Branches under the judges above: the clicked face (`wt/track-face`), the rebuilt hard track (`wt/hard-track-2`), freeze damage (`wt/l0-veto`), the preview renderer (`wt/build-preview`).
- Parked: the refuge (`wt/hearth-small`, S228).

## Known anomalies (each blocks only what depends on it)

- Entity push has no producer in the law (five bots on one crossing).
- Sprint at an exactly axis-aligned wall depends on rounding in Mojang's own `acos` (`movement.rs:818`), ported faithfully.
- His bug S38: a track run deletes a sulfur spike from his inventory; never traced.
