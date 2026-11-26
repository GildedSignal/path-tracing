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

Not used now: Curryocity/Stratfinder (single-axis jumps, no numbers, no licence); Gazelka's hybrid A\* post and SwarmBot (no numbers); JumpCraft and parkour.gg maps (server-hosted, no world files); SoulFire, Botcraft, mineflayer-pathfinder (no published numbers, a later many-bots row).

## Corrections to the pasted summary

- Hielke records hold for Java 26.2+, except Dimension, which is 11:30 (2026-11-24), not 11:39. Across all versions, faster and now-obsolete records exist (Town 1:12 on Bedrock, Paradise 2:52 by an exploit); only the 26.2+ Java board matches our law.
- Baritone v1.19.0 ships for Minecraft 26.2 (Fabric, LGPL-3.0, `#goto x y z`), so the BlockStep protocol reruns with Baritone on our exact version.
- Hielke maps now ship for Minecraft 26.3 (Town v1.4.7, Spiral v3.0.5); the law is 26.2. The 26.2+ bucket rejects a run whose map version is not the latest.
- Hielke's terms allow personal, non-commercial use and forbid redistribution or modified maps: world files never enter git.
- SpectraPhysics' tick course has no cobwebs or honey (those are in its trajectory hash, not its timing).
- BlockStep's 28.3 b row is 28.2 b in the thesis, and the thesis does not say whether planning time is inside travel time.
