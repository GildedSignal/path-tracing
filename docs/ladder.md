# The ladder

This file holds the scoreboard and the rungs. Each rung is frozen before its first run, with its course, numbers and pass line written down, and is never weakened afterwards. The first failing rung is the next work item. A rung that cannot fail is a control, not evidence.

## The scoreboard

- **T:** server ticks from the tick in which the server receives the goal command, to the tick in which the bot stands in the goal cell, with every edit restored and every temporary item back in its inventory.
- **The bracket:**
  - LB is the light cone's lower bound for the trip. It is admissible: no run may ever beat it.
  - UB is the best T found by an independent offline search over raw tick controls, never over the planner's own moves.
  - Each rung reports the distribution of T / UB and of the bracket width UB / LB.
- **Latency:** ticks from the whisper's arrival at the bot's client to its first applied control. The p99 must be ≤ 1.
- **Failures:** failed edges, server corrections, restoration misses and irreversible losses. Each must be zero.
- **Scale:** planning cost against trip length, in work units and in wall time.
- **Reports** give n, min, median, p90, p99 and max, plus the worst seeds with their replays. Never an average alone, never a trimmed outlier. A result holds only inside the range it was observed in.

## Fixtures

- **Worlds:** pre-generated default worlds with fixed seeds, kept on D:.
- **Server:** the lab server runs on port 25566 (RCON on 25576).
- **Courses:** built on those worlds through RCON.
- **No superflat:** every superflat chunk is the same, so the memo, repair and scale tests could not fail there.
- **Isolation:** gates take the rig lock and run under `/tick rate`, so each test stays under 60 seconds. Julien's player stays away from the bot, because players push each other.

## Rungs

| Rung | Phase | What it stresses | Status |
|---|---|---|---|
| L0 | 1 | **The law.** Fixtures for each terrain class: full blocks, slabs and stairs, step-ups, ceilings, ladders and vines, water, ice and blue ice, slime, soul sand, honey, cobweb, powder snow, and edges for the sneak back-off. Many random input sequences each. **Pass:** bit-identical positions and velocities on every tick; damage and exhaustion equal the server's; zero corrections; every mutant caught (axis tie order, step-up, the sneak back-off at edges, ceiling bonks) | to freeze before 1.5 runs |
| R1 | 2 | Flat trips, straight and diagonal, 10 to 200 blocks | to freeze before Phase 2 runs |
| R2 | 2 | Walls and mazes on legs alone: detours | to freeze before Phase 2 runs |
| R3 | 2 | Gaps of 1 to 4 blocks, step-ups, drops up to the 3-block safe fall. Damaging drops are barriers until tokens exist | to freeze before Phase 2 runs |
| R-retarget | 2 | A new goal at a random tick mid-trip. T counts from the new command; momentum carries over | to freeze before Phase 2 runs |
| R-scale | 2 | Trips of 10, 10², 10³ and 10⁴ blocks along a fixture corridor of default terrain. Frozen with Phase 2 and red until Phase 4 bends it | to freeze before Phase 2 runs |
| R4 | 3 | Walls against detours with a pickaxe; stone without Silk Touch as a barrier | when Phase 3 starts |
| R5 | 3 | Gaps and climbs with blocks; undo reachability | when Phase 3 starts |
| R6 | 5 | The bucket: falls, climbs, the Nether | when Phase 5 starts |
| R7 | 6 | Momentum ground: ice tunnels, slime, soul sand, honey | when Phase 6 starts |
| R8 | 7 | Tokens: pearls, food, health, durability | when Phase 7 starts |
| R9 | 8 | The moving world: fluids, falling blocks, mobs, redstone | when Phase 8 starts |
| R10 | 11 | Far travel: boats, the Nether sheet, elytra | when Phase 11 starts |
