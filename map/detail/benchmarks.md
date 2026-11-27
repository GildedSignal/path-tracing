# Outside references: what others measured, checked at the source

Julien's session goal for 2026-11-26: gather the metrics that other people publish, then beat the ones that matter to T. Every row here was read at its primary source on 2026-11-26. A pasted summary from another AI was the lead, and several of its numbers were wrong. Opponents are run by us on this machine before any comparison is claimed; a number from another CPU or version is a lead, never a result.

## What matters to us, in order

| # | Reference | Verified number | Our row it attacks | Fermi floor | Why it matters |
|---|---|---|---|---|---|
| 1 | Hielke maps, speedrun.com `mcm_hielke_maps`, Any% Java, version bucket `26.2+`, RTA (whole seconds, start and stop not stated in the rules), video-verified | first places: Town **2:19** (Power Ups, the only Town class), Spiral **6:51**, Spiral 2 **6:56**, Paradise **9:06**, Volcano **9:56**, Egg **10:09**, Dimension **11:30**, Spiral 3 **20:28**, Paradise 2 **22:38**, Pyramid **35:27**, Paradise 3 **46:24** (Spiral 6:51 = 8,220 ticks) | T against the best human | to be bracketed per map by the kit's light cone | an independently designed hard track with an elite human bar: the goal's own scoreboard row |
| 2 | BlockStep vs Baritone (Braam, Leiden 2022, thesis Table 5.1): Minecraft 1.16.1; 6 biomes (desert, hills with trees, ice, ice mountains, swamp, trees); start at a biome's middle point, goal at (±20, ±20), (±50, ±50) or (±100, ±100); 72 tests per algorithm | mean travel s, BlockStep / Baritone: **6.6 / 8.4** (28.3 b), **16.2 / 18.8** (70.7 b), **35.1 / 37.0** (141.4 b, only runs both finished); BlockStep faster in 85.4 %, 16 % faster on average | T on natural terrain, which no track of ours holds yet | bare-legs sprint-jump `0.356 b/tick` (`straight800`): 3.97 s, 9.93 s, 19.9 s | both sit `0.2–0.3` order above the flat floor; seeds and coordinates are not published, so the protocol is rerun on 26.2 seeds with Baritone run by us |
| 3 | SpectraPhysics (`trqxyz/SpectraPhysics`, GPL-3.0, mineflayer, 1.16.5–26.1), `bench/tick.js`: 50 players, stone floor with bottom smooth-stone slabs, oak fences and water strips, forward + sprint + jump, yaw +90° every 40 ticks, 200 warm-up ticks, 2,000 timed | **~3 µs per player-tick**, CPU unstated; look-ahead copy ~1 µs; a ~190-block chunk route ~2 ms | the law tick (0.6–0.95 µs) | 0.1–0.3 µs | the only published per-tick physics cost; matched only once both run here on the same course |
| 4 | ParkourCalculatorMod (`Leg0shii`, GPL-3.0, Fabric 26.3 / 1.21.3 / 1.20.4, Forge 1.8.9 / 1.12.2) | **no numbers published**: bit-exact X/Z stepper, yaw solver per tick under a time budget, re-verified on the real entity; its Stratfinder does one jump, at most one turn, no momentum chains | word quality: least ticks for one jump at matched budgets | the law's exact optimum for that jump | the specialist on precision; run its Minecraft-free `core/` on fixed jumps |
| 5 | MineOdyssey (HKUST, arXiv 2610.11328): 30 worlds (20 outdoor, 10 indoor), Minecraft 1.21.11, 180 tasks of ordered waypoints, success within 3.5 blocks (1.5 vertical), adventure mode, no flight; 2.60 GB zipped | metrics are success, coverage, SPL and path length; **no non-LLM baseline**; the claimed best of 85.6 % is not on the pages read | success and T through buildings: doors, stairs, ladders | — | an independent built world; the worlds keep their makers' terms |

## Measured here (Ryzen 7800X3D, 2026-11-26, both engines on this machine)

SpectraPhysics at `1ea12c8` (Node 22.23.5) against the law on our `16³` sections (`engine/world/src/bin/bench-course.rs`, `WorldSnapshot::from_fn`), µs per player-tick, medians over runs:

| Course | SpectraPhysics 1.21.4 | SpectraPhysics 26.1 | the law, 26.2 | ratio against 26.1 |
|---|---|---|---|---|
| `exact`: their `tick.js` (49 of 50 players fall off the ±30 floor, to y ≈ −7,967) | 5.9–7.2 | 11.0–11.1 | 0.93–1.16 | ~9–12× |
| `floored`: the same obstacles, stone to ±128, all 50 stay on the ground | 21.7–23.2 | 31.1–34.8 | 0.89–1.09 | ~28–39× |

- Their README's "~3 µs" is not reproduced on this CPU: their own bench gives 5.9 µs at its default version, and the number is mostly free fall.
- `straight` (one player, plain stone, the `law` `bench` workload): `0.62 µs` median (`0.61–0.63 µs`) on our sections, matching `BoxWorld` (`0.57–0.98 µs`) and near the `0.1–0.3 µs` floor.
- Each section holds split 4³ collision masks (`collide` and `collide_beyond`), shift-packed `PaletteSlot` entries (direct `&'static Block`, reach, offset/unknown flags), and `WorldSnapshot` `visit_collisions` / `collides_with_suffocating_block` / `collision_cell_bounds` overrides paired with SSE2 inline `math::floor`/`ceil` and single-tick `core` hook/climb/push-out deduplication so halo-free floor and air queries resolve in bitmask checks.
- **BlockStep's protocol, offline on 26.2** (`engine/plan/src/bin/blockstep.rs`; evidence `D:\path-tracing-evidence\blockstep-20261126\`): one seed (`1162026`), each biome's interior found by `locate biome` grids (desert, windswept forest, snowy plains, snowy slopes, swamp, forest), goals at (±d, ±d), bare legs, cold store per task, every plan replayed through `law::try_tick`. Predicted, not yet flown; the terrain is not the thesis's.

| d | solved | mean T (s) | with thinking | BlockStep | Baritone | floor |
|---|---|---|---|---|---|---|
| 20 | 24/24 | 4.79 | 4.96 | 6.6 | 8.4 | 3.97 |
| 50 | 21/24 | 11.94 | 12.41 | 16.2 | 18.8 | 9.93 |
| 100 | 20/24 | 25.27 | 26.88 | 35.1 | 37.0 | 19.9 |

  Per biome at 20 and 50 blocks, ours is below BlockStep everywhere except the desert (5.80 against 5.1; 12.50 against 12.0), where `desert-20-ne` takes 187 ticks against a floor of 80 and `desert-100-sw` 792 against 398. Seven of 72 return `NeedsJoints` (3 at 50, 4 at 100). Thinking, cold: p50 0.40 s, p90 1.48 s, max 3.57 s, 1–2 orders above the new-joint floor.
- **Why the seven fail:** every one is `defect uncrossed` after learning spends its cap (~98,000 or ~240,000 raw ticks, 0–6 proposals). Single strips on flat stone (`WorldSnapshot::from_fn`, bare legs) separate the causes. Leaf litter, a snow layer and a 2-block drop cross at once (T 84, 84, 83). Water 1 deep crosses but costs 869 ms of thinking against 0.4 ms on stone. **Water 3 deep × 10 wide and a powder-snow floor × 6 return `NeedsJoints`**, though both are crossable in the game. Swimming and sinking are not in the field, so every water or powder-snow defect falls to raw learning. That is three failures (desert-100-ne, swamp-50-se, swamp-100-se) and powder snow is a fourth (icemountain-100-se). The remaining three (hills-50-sw, forest-50-sw, forest-100-se) meet 1–3,845 exceptions among trees, cliffs and leaf litter and spend the 240,000-tick cap: not yet separated.
- **The desert outliers:** `desert-20-ne` climbs a 14-block dune (y 78 → 92 → 78) on the straight diagonal; the flat floor ignores height, so 187 ticks is not shown to be poor. `desert-100-sw`'s goal lies in a warm ocean (18,190 water cells in its box): its 792 ticks are a swim, priced by the same gap.
- Every Hielke map ships only for 26.3; Parkour Spiral 3.0.5 is DataVersion 5023, the law is 4903 (26.2), and a 26.2 game cannot open a newer world. Spiral is dropped (Julien, 2026-11-26); the other ten open only once the law is 26.3.
- Few other parkour maps have a human board on speedrun.com: Run Swim Fly Parkour (Noxcrew, Full Course 4:58.95) and Unusual Parkour Map (8:08, 2021); their download versions are not yet checked.
- MineOdyssey's 30 worlds (SHA-256 checked) are DataVersion 4671 (1.21.11), older than the law's: a 26.2 server's `--forceUpgrade` on copies brings them to 4903; its tasks name ordered waypoints by id (`tasks.json`).
- Opponents and maps stay outside git: `D:\path-tracing-bench\opponents\` (SpectraPhysics, Baritone v1.19.0 for 26.2, SHA-1 checked), `D:\path-tracing-maps\` (Hielke, MineOdyssey).

Not used now: Curryocity/Stratfinder (single-axis jumps, no numbers, no licence); Gazelka's hybrid A\* post and SwarmBot (no numbers); JumpCraft and parkour.gg maps (server-hosted, no world files); SoulFire, Botcraft, mineflayer-pathfinder (no published numbers, a later many-bots row).

## Corrections to the pasted summary

- Hielke records hold for Java 26.2+, except Dimension, which is 11:30 (2026-11-24), not 11:39. Across all versions, faster and now-obsolete records exist (Town 1:12 on Bedrock, Paradise 2:52 by an exploit); only the 26.2+ Java board matches our law.
- Baritone v1.19.0 ships for Minecraft 26.2 (Fabric, LGPL-3.0, `#goto x y z`), so the BlockStep protocol reruns with Baritone on our exact version.
- Hielke maps now ship for Minecraft 26.3 (Town v1.4.7, Spiral v3.0.5); the law is 26.2. The 26.2+ bucket rejects a run whose map version is not the latest.
- Hielke's terms allow personal, non-commercial use and forbid redistribution or modified maps: world files never enter git.
- SpectraPhysics' tick course has no cobwebs or honey (those are in its trajectory hash, not its timing).
- BlockStep's 28.3 b row is 28.2 b in the thesis, and the thesis does not say whether planning time is inside travel time.
