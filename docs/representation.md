# The representation (draft)

**Challenged on 4 November:** a red-team found errors in claims 1, 2, 4, 5, 8, 9 and 11 and in "Which math". They are listed in [HANDOFF.md](HANDOFF.md) under "Proposals awaiting Julien" and are folded in once he has read them.

This file covers how the bot sees the world so that the search becomes trivial (principle 3). Each claim names the test that would kill it, and a claim becomes law only once its rung passes. A fix bolted onto a claim means the claim is wrong (ES principle 19).

## The core

1. **The bot is a point.** The body is folded into the terrain, as in configuration space ("compact the space so that the player is one block tall", X4).
   - A place exists only where the body fits: standing needs 1.8 blocks of clearance, sneaking 1.5, crawling 0.6.
   - So a one-high hole is not in the space at all for a standing bot. For a crawling bot it is, at crawling speed.
   - The world becomes sheets of places the bot can be, joined by arcs (jumps, falls, throws). True 3D is needed only in water and in elytra flight.
   - *Killed if:* a fixture with 1.5-high and 1-high openings shows the planner missing a passage that exists and pays, or using one that doesn't exist.
2. **The toolkit is the metric** (principle 4). For a toolkit K, each place has a crossing time in each direction, with speed classes where they matter. A tool rewrites a whole class of places at once:
   - a pickaxe rewrites every stone;
   - a block rewrites every gap within reach;
   - a bucket rewrites every vertical drop or climb.

   Using a tool belongs to the local move, which is execution. It never appears in the search.
   - *Killed if:* adding a capability needs new search or geometry code. The universal gate says each capability must enter as law and toolkit data only.
3. **The route is the geodesic.** T is the minimum, over all paths, of the summed ticks: min-plus composition.
   - T only grows, so there are no negative cycles, and A* (with integer tick buckets) is valid on the full state.
   - The triangle inequality fails on positions alone, because stopping kills momentum. It holds on the full state.
   - *Killed if:* T on rungs R1–R3 falls outside the bracket's width, or the lower bound is ever beaten by a real run, which would mean it was not admissible.
4. **Speed is part of the state only where it matters.** Each tick, speed keeps a fraction friction × 0.91 of its past, so 95% of it is gone after about 5 ticks on stone and about 26 on ice. Speed therefore enters the state only:
   - where it is remembered: low-friction ground;
   - where it is required: take-offs and long jumps.

   Facing is free, because the bot turns within a single tick.
   - *Killed if:* an ice rung where routes depend on entry speed shows ice routes taken when they don't pay, or missed when they do.
5. **Equivalence is within a certified width, not in bits.** Bit-identical states almost never recur: a velocity difference takes about 60 ticks on stone to fall below one unit in the last place.
   - Two states merge when they reach the same future within the width the executor can correct.
   - Translations and the grid's 8 symmetries hold only within that width.
   - Every memo hit is verified by exact replay.
   - *Killed if:* a replayed memo hit diverges beyond its width.
6. **Edits are conjugates: A·B·A⁻¹** (open, cross, close), which leaves the world and the toolkit as they were (principle 20).
   - The undo must be reachable from wherever the move leaves the bot. A bucket's undo travels with the bot. A wall's hole is just behind it. A pillar's undo stays below, so restoring it costs the trip back down.
   - Restoring means the identical block and block state, orientation included (S12). Without Silk Touch in the toolkit, any block whose drop differs is a barrier: stone, glass, ice, grass, ores, leaves.
   - Irreversible side effects are barriers too: sand falling, attachments popping off, drops scattering.
   - *Killed if:* after any R4 or R5 trip, a block state differs or the inventory is short.
7. **Consumables are tokens.** Pearls, food, health and durability are spent, so they shrink the space rather than reshape it.
   - The search carries their counts as Pareto labels, the way EV routing carries battery level.
   - Exchange rates come from the game itself. A pearl costs 5 damage, which takes 7.5 food points to heal, about 300 m of sprinting. Sprinting stops at food 6.
   - *Killed if:* the R8 rungs show a pearl or a drop that taking damage spent when it does not save T.
8. **The hierarchy is min-plus tables** (FMM in the tropical semiring, principles 17 and 22).
   - Each region keeps a table of times between its border states, per toolkit, in the style of Customizable Route Planning.
   - Identical regions share one table, as in Hashlife.
   - One changed block re-solves its cell and the tables above it.
   - A toolkit change re-checks the tables that depend on it, so improvements are caught as well as breaks.
   - *Killed if:* the border-state probe (Phase 2.0) shows the number of border states per chunk exploding once speed classes are included, or the scale rung stays red after Phase 4.
9. **The light cone sets both what to shape and where plans can break.**
   - What to shape: where the bot can be before a deadline.
   - Where plans can break: where Julien or the world can change things before the bot gets there.
   - Its speed limit depends on terrain and toolkit: 7.1 m/s sprint-jumping, 19.3 in an ice tunnel, 78 falling, about 17.6 by pearl.
   - *Killed if:* any measured T is below the cone's lower bound.
10. **Edge width is "when we can make mistakes"** (principles 15 and 16).
    - Every edge records how far execution may miss and still land in the same future.
    - Narrow edges cost more once real jitter is priced in.
    - Irreversible losses are barriers.
    - *Killed if:* under real jitter (Phase 9), a certified edge fails within its width.
11. **Coordinates are chosen per regime; the system is unchanged** (principle 14). Surface coordinates on the sheets, arc parameters in flight, volumes in water and in elytra flight.
    - *Killed if:* any coordinate choice fails to reproduce the law within its width.
12. **The Nether is a second sheet** where each block counts for 8, joined to ours at portals.
    - *Killed if:* the R10 rung finds a Nether route that the planner misprices.

## The four questions (X3)

- **Breaking a wall instead of going around it:** with a pickaxe, a breakable wall is a slow material. Its cost is breaking, crossing and rebuilding with the identical block. The route goes through it whenever going around costs more, the same way light goes through glass. Nothing has to decide.
- **One-high holes:** for a standing bot they are not in the space at all. For a crawling bot they are, at crawling speed.
- **When a bucket is excellent:** the bucket rewrites the vertical metric.
  - A 100-block cliff becomes about a 3-second road: 61 ticks of falling, plus placing and picking up the water.
  - Walls become climbable through water.
  - The bucket's value at a given spot is T without it minus T with it.
- **When it must sprint for a jump:** the far side of a long gap connects only to states that are already fast. So the route has to pass through a run-up, and backs off for more runway when there isn't enough.

## Which math (X4)

His "certain type of math" is the word metric over moves: the distance between two places is the shortest word in the moves, weighted by ticks. So a tool is a new generator, and a new generator reshapes the space itself (his bike, X5).

- **Edits are conjugates.** Each one is a conjugate A·B·A⁻¹, which is how cubers build their algorithms.
- **Combined moves are commutators.** A move that no single tool can make comes from alternating two of them. Climbing with the bucket and pillaring work this way, the same way parallel parking produces sideways motion.

## Still open

- **Orientation on placement:** restoring stairs, logs and anything else oriented depends on how the bot faces when it places them.
- **Compactness:** whether border states stay compact once speed is included. The probe decides, before the geometry is designed.
