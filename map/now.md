# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **Flat ground: on `main`, flown in his game.** `plan-track --mode solve` writes the tick script the mod plays. The flights: `straight100` 288 predicted / 288 on the track clock, `straight300` 845 / 845, `straight800` 2,248 / 2,248, `stonesimple` 85 / 85 (his best 86). Per new command (`real_time.rs`, 60 each, every plan replayed by `law::try_tick` onto B at the predicted tick): from standing min 6.2 / median 13.5 / p90 18.6 / max 28.8 µs, while moving 7.0 / 12.7 / 16.1 / 23.1 µs, 0 refused, 0 law ticks prepared after the first command on a ground (119 µs, 15 law ticks). The track clock counts from the last tick on A (his 97 recorded times each +1).
- **Deltas the flights show (expected against seen):** (1) `plan-track` costs 1.8–2.3 s per click (a process and the world from disk). The stick runs `solve` in the bot's own process instead (`engine/native/src/resident.rs`: one living store per session, the corridor read from the chunks the bot has received), so a stick click costs microseconds. In his game (the bot driven by `goto` from the console): open ground reached (2 of 2); a way that crosses a fence is refused as `NeedsJoints` with the fence named, because a new site has no joints yet (step 3). Playback stops only on events that move the body (explosion, motion, position, rotation). (2) The planner's settled start has saturation 5.0, the bot has 20.0 (first deviation at tick 0 on every flight; T unaffected on these lengths, the food guard is not). (3) Ten new flat goals are not flown yet. Astra is back on the 23rd (Codex week reset, 20:34).
- **Sonnet judges on Antigravity: stopped by the budget guard.** Two at once took the Claude+GPT weekly meter from 87% to 74% left in about 10 minutes, before either reached a verdict; their sandbox cannot run commands ("non-absolute file path"). The hard-track judge's notes so far: the branch is ~450 lines of offline certificate the game does not judge plus ~56k lines of generated CSVs, and the witness T rose from 579 to 939 race ticks. The four prompts (`jobs/*-judge`) wait for his choice of judge.
- **Argon, `jobs/map-lint`:** the map against his verbatim turns, once.
- **GLM, `scripts/glm-chain.ps1`:** the 15 port prompts without a result (`jobs/glm-chain.txt`; entity push first, then boats, pearls, portals for step 8, then the rest), one at a time.
- **Hydrogen, port build B** (`wt/port-build-b`, on build A): twelve reproduced port results into the law (place, break, interact for step 5; sprint, lava, game mode, flying; elytra start and stop, the portal's base height), and build A's `break_ticks` fix.
- **Hydrogen's port build A** (`wt/port-build-a`, `055e7b7`: mining speed, boat, elytra boost, ender pearl in the law, not wired into `movement.rs` or the planner): no flight can show it yet, because nothing plays these mechanics; it merges when step 5 (mining) or step 8 (boat, elytra, pearl) consumes it. One judge finding open: `break_ticks` is `ceil(1/p)` in f64, the game breaks at `p·n ≥ 1` in f32 (obsidian by hand 5000, not 5001).

## The overhaul (`jobs/overhaul/plan.md`, approved S240): state

Built and on `main`: the map (this folder; 22 KB loaded whole, history in `archive/`), AGENTS.md and the README rewritten, the hooks of `work.md` (card and his words on every message, the launch guard, the job watcher, the map and net-line commit checks, the push guard), the scoreboard (`scripts/scoreboard.ps1`), companion agency in `gpt.ps1` and `gemini.ps1`, GLM without its checker or lenses, tests cut to what the game judges, memory cut to 11 files, 220 dead worktrees and 51 branches archived (`D:\path-tracing-archive`, `archive/*` tags), history stripped from `map/detail/`. The meters: `/meters` opens a pane in every session (the mod in `C:\Users\PASCAL\.claude\mods\meters-band`, loaded by `CLAUDE_CODE_PLUGIN_DIRS` in `~/.claude/settings.json`; the app's own Context panel takes no mod content).

Checks against these fixes: `jobs/overhaul/checks.md`.

Left: delete the old A\* planner (`plan/src/flight.rs` modes 3/5/b, `planner.rs`) and dead `words` code, after Astra's job; Argon lints `map/` against his verbatim turns once.

## Next, in order

1. The click in microseconds in his game: the living store in the server beside the bot, so a click costs ticks, not 1.8–2.3 s.
2. Delete the dead planner code (`flight.rs` modes 3/5/b, `planner.rs`) now that `solve` flies.
3. His tracks as inputs (Sol, `jobs/loop-tracks`), held until Astra's work is in.
4. The bot's runner: words × count in the client, re-anchoring at events, the goal changed mid-run (R-retarget).

## The order of work (steps of the one loop)

| Step | State |
|---|---|
| 1. the core: `solve`, words with counts, the word store | built, on `main` |
| 2. joints and sets | built, on `main` |
| 2b. the simplest case right | built, on `main`; per new command 68 ms against a ~10 µs floor |
| 3. the loop on his tracks | flat ground flies (T = predicted); his tracks as inputs next |
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
