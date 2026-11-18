# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Astra 6.2, `jobs/flat-ground`** (worktree `D:\path-tracing-wt\flat-ground`, from `75b2b90`, launched 15:40): first the new core flies in his game (`plan-track --mode solve` writes the tick script the mod plays), then flat ground as the map says: the gait stored once, the yaw put in, nothing relearned per command. She was launched before the job-watcher hook existed, so a background Bash waiter wakes the integrator; if it is gone, check `jobs\flat-ground\last.md` and the worktree. Codex week at 89%: this is the last Astra run before its reset.
- **Hydrogen's port build A** (`wt/port-build-a`: mining speed, boat, elytra boost, ender pearl): builds and passes; its Sonnet judge could not run commands and judged by reading. Merge after a flight.

## The overhaul (`jobs/overhaul/plan.md`, approved S240): state

Built and on `main`: the map (this folder; 22 KB loaded whole, history in `archive/`), AGENTS.md and the README rewritten, the hooks of `work.md` (card and his words on every message, the launch guard, the job watcher, the map and net-line commit checks, the push guard), the scoreboard (`scripts/scoreboard.ps1`), companion agency in `gpt.ps1` and `gemini.ps1`, GLM without its checker or lenses, tests cut to what the game judges, memory cut to 11 files, 220 dead worktrees and 51 branches archived (`D:\path-tracing-archive`, `archive/*` tags), history stripped from `map/detail/`. The meters: `/meters` opens a pane in the app (a mod in `C:\Users\PASCAL\.claude\dev-mods\…\meters-band`; the app's own Context panel takes no mod content).

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

- Branches: the clicked face (`wt/track-face`), the rebuilt hard track (`wt/hard-track-2`), freeze damage (`wt/l0-veto`), the preview renderer (`wt/build-preview`).
- Parked: the refuge (`wt/hearth-small`, S228).
- Open questions for him: whether the class (S67–S69) is where he wants it.

## Known anomalies (each blocks only what depends on it)

- Entity push has no producer in the law (five bots on one crossing).
- Sprint at an exactly axis-aligned wall depends on rounding in Mojang's own `acos` (`movement.rs:818`), ported faithfully.
- His bug S38: a track run deletes a sulfur spike from his inventory; never traced.
