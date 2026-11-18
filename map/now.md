# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Astra 6.2, `jobs/flat-ground`** (worktree `D:\path-tracing-wt\flat-ground`, from `75b2b90`): first the new core flies in his game (`plan-track --mode solve` writes the tick script the mod plays), then flat ground as the map says: the gait stored once, the yaw put in, nothing relearned per command. A waiter wakes the integrator when it ends. Codex week at 87%: about one run left before its reset.
- **The overhaul** (`jobs/overhaul/plan.md`, approved S240): the map (this folder), the if-then graph in code, agency, less code, `main` merged often, the meters band in the app.
- **Hydrogen's port build A** (`wt/port-build-a`: mining speed, boat, elytra boost, ender pearl): builds and passes; its Sonnet judge could not run commands and judged by reading. Merge after a flight or a running judge.

## Next, in order

1. Fly the new core in his game once Astra's export builds: `straight100/300/800`, `stonesimple`, ten new flat goals. The clock's T must equal the predicted T, with no deviation.
2. Finish the overhaul: the hooks, the commit checks, the scoreboard, the companion contracts (`gpt.ps1`, `gemini.ps1`), memory cut to what is about him, delete the old A\* planner and dead `words` code.
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
