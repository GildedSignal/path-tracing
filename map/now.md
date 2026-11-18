# Now

What is in flight and what comes next. Rewritten freely; history lives in git.

## In flight

- **One mind in the server (`wt/server-mind`, Stage 1 + Stage 2 live):**
  - *Boundary & scale spec (`oracles/server-probe/`, `D:\path-tracing-evidence\server-probe-20261118`):* 24 probe runs (`packet`, `lazy`, `virtual` at 20 and 100 TPS) run with **0 teleports** and **0 `"moved wrongly"`**. Observer fixes cut the 89.1% `Waypoint`+`ChunkMap` quadratic wall to < 1%; packet `ServerPlayer`s cost ~23.4 µs/bot at 100 TPS, dispersed chunk discs hit 37.15 TPS (`60,681` chunks, `4,094 MiB`) at 50 lone bots, `lazy` cuts CPU 33× (`~0.72 MiB/bot` shell RAM), and `virtual` runs 10,000 bots in `0.050 / 0.040 ms/tick` (`1,301 MiB` flat). Architecture in [server.md](detail/server.md).
  - *Stage 2 live runtime (`server-mod/` + `engine/mind/`, `D:\path-tracing-evidence\stage2-flight-20261118-b`):* dedicated Fabric 26.2 mod streams RLE chunks, `LevelChunk.setBlockState` deltas, and per-bot self-channels over Windows memory-mapped SPSC rings (`s2e` 16 MiB / `e2s` 4 MiB) to out-of-process `mind.exe` (`WorldMirror`, one shared `GeneratorStore`, `plan::solve`, `law::try_tick`). Four live flights run with **0 teleports** and **0 `"moved wrongly"`**: (1) 1 bot, 50-block sprint-jump (`actual_T == predicted_T = 144`, solve `141.4 µs`, server tick med/p99 `2.024 / 18.016 ms`); (2) 50 bots in grid formation, 80-block sprint-jump (`50/50` exact `T=228`, solve `4.8 / 7.5 / 11.1 µs`, server tick med/p99 `4.392 / 32.170 ms`); (3) hot-reload of `mind.exe` mid-session (`engine_epoch=2`, `841` chunks / `20,184` sections re-synced) + 50-bot return flight (`50/50` exact `T=204`, solve `4.0 / 4.1 / 139.7 µs`, server tick med/p99 `3.614 / 11.100 ms`); (4) live `setblock 20 -60 0 minecraft:oak_fence` delta -> `NeedsJoints at [20, -60, 0]` in `28.4 µs`. Next: Stage 3 (`Virtual` / `Visual Projection` / causal promotion + region-file preload).
- **Flat ground: on `main`, flown in his game.** `plan-track --mode solve` writes the tick script the mod plays. The flights: `straight100` 288 predicted / 288 on the track clock, `straight300` 845 / 845, `straight800` 2,248 / 2,248, `stonesimple` 85 / 85 (his best 86). Per new command (`real_time.rs`, 60 each, every plan replayed by `law::try_tick` onto B at the predicted tick): from standing min 6.2 / median 13.5 / p90 18.6 / max 28.8 µs, while moving 7.0 / 12.7 / 16.1 / 23.1 µs, 0 refused, 0 law ticks prepared after the first command on a ground (119 µs, 15 law ticks). The track clock counts from the last tick on A (his 97 recorded times each +1).
- **Deltas the flights show (expected against seen):** (1) `plan-track` costs 1.8–2.3 s per click (a process and the world from disk). The stick runs `solve` in the bot's own process instead (`engine/native/src/resident.rs`: one living store per session, the corridor read from the chunks the bot has received), so a stick click costs microseconds. In his game (the bot driven by `goto` from the console): open ground reached (5 of 5, up to 46 blocks; the bot slides past B on its momentum after the touch); a way that crosses a fence is refused as `NeedsJoints` with the fence named, because a new site has no joints yet (step 3). Playback stops only on events that move the body (explosion, motion, position, rotation) or a block edit inside the plan's corridor; chunks arriving while it walks do not stop it. The corridor is the straight box between the bot and B plus 2 blocks; a click mid-jump takes the first solid block below as the ground; a click on its own block answers "Already there." Open: (a) he sees the bot lag in the air while jump-running, though the flights land on the predicted tick: unmeasured; (b) a block whose shape the law refuses ("non-local shape", stairs) panics the corridor instead of becoming an exception; (c) steps, fences and stairs are refused as `NeedsJoints`. (2) The planner's settled start has saturation 5.0, the bot has 20.0 (first deviation at tick 0 on every flight; T unaffected on these lengths, the food guard is not). (3) Ten new flat goals are not flown yet. Astra is back on the 23rd (Codex week reset, 20:34).
- **Sonnet judges on Antigravity: stopped by the budget guard.** Two at once took the Claude+GPT weekly meter from 87% to 74% left in about 10 minutes, before either reached a verdict; their sandbox cannot run commands ("non-absolute file path"). The hard-track judge's notes so far: the branch is ~450 lines of offline certificate the game does not judge plus ~56k lines of generated CSVs, and the witness T rose from 579 to 939 race ticks. The four prompts (`jobs/*-judge`) wait for his choice of judge.
- **Argon, `jobs/map-lint`:** completed (`jobs/map-lint/report.md`, 60 findings across `map/` against his verbatim turns), ready to apply.
- **GLM, `scripts/glm-chain.ps1`:** actively running `jobs/glm-chain.txt` (`port-entity-push-and-crowd-loop`, `port-boat-place-and-break`, `port-pearl-throw-origin-and-item` dossiers done; `port-portal-teleport-transition` and remaining queue items running one at a time).
- **Hydrogen, port build B** (`wt/port-build-b`, on build A): completed in `jobs/port-build-b/last.md` (`159 passed`, twelve reproduced port results into `engine/law` plus `break_ticks` f32 fix; uncommitted in `D:\path-tracing-wt\port-build-b`, awaiting wiring/merge).
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
