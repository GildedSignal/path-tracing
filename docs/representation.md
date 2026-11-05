# The representation (draft)

This file describes how the bot sees the world so that the search becomes trivial (principle 3). Each claim names the test that would kill it, and a claim becomes law only once its rung passes. A fix bolted onto a claim means the claim is wrong (ES principle 19).

The red-team's corrections were folded in on 4 November with Julien's approval (S14), together with his rule that getting around never modifies the world (S13, S14).

## The target

Make the search trivial:
- a time-to-go field rooted at the goal, built over the tables;
- the bot descends it greedily, which is his straight line and his clay mug (principle 7);
- no open-ended search over an exploding state.

Everything below exists to make that field cheap to build, cheap to repair and exact where decisions depend on it.

## The claims

1. **The bot is a point.** Its body is folded into the terrain.
   - **How:** each pose's collision box (0.6 × height × 0.6) inflates the world's collision boxes exactly.
     - Standing: 1.8 blocks tall.
     - Sneaking: 1.5 blocks.
     - Crawling: 0.6 blocks.
   - **Places:** a place is wherever the box fits, including standing 0.3 blocks out over an edge.
   - **Crawling:** this is not a mode the bot can choose on land. It needs a trapdoor or water that is already there, so it is a toolkit entry.
   - **The world:** sheets of places, joined by arcs (jumps, falls, throws). Full 3D is needed only in water and in elytra flight.
   - *Killed if:* a fixture with 1.5-high, 1-high and edge-overhang places shows the planner using a place the body doesn't fit, or missing one it does.

2. **The toolkit is part of the metric, and tool use is a priced edge.**
   - **Tool use:** placing or using an item is "just a movement" (X5): an ordinary edge with its own cost in ticks.
   - **Closure:** every tool use closes inside its own edge, so states on a border never carry an open edit.
   - **Toolkit:** what the bot carries decides which edges exist: water where a bucket can be used, a boat on ice.
   - *Killed if:* adding a capability needs new search or geometry code. The universal gate allows only new law and toolkit data.

3. **The route is the geodesic.** T is the minimum, over sequences of moves, of their summed ticks (min-plus).
   - Moves are partial maps on (state, world) that don't commute.
   - T only grows, so there are no negative cycles.
   - On positions alone the triangle inequality fails, because momentum matters. On the full state it holds.
   - *Killed if:* on R1–R3, T falls outside the bracket, or a run ever beats the lower bound.

4. **Speed is part of the state almost everywhere.**
   - **Fastest gait:** sprint-jumping (7.1 m/s) is the fastest gait even on stone.
   - **Coasting:** in the air the drag is 0.91, so speed builds over several jumps and coasting takes about 32 ticks to lose 95% of it.
   - **Braking:** pressing against the motion stops much sooner (S13). It is part of every local solve.
   - **Corners:** speed bleeds at corners, and each costs its own ticks.
   - **Facing:** the bot can turn in one tick, so facing costs nothing.
   - *Killed if:* courses with corners and runways show routes that ignore entry speed beating the planner's.

5. **States merge exactly at canonical points, and within a certified width between them.**
   - **Exact merges:**
     - `aiStep` zeroes horizontal velocity once its square falls below 9 × 10⁻⁶ (about 8 ticks after stopping on stone), and each vertical component below 0.003.
     - Collisions clamp the position to the face hit.
     - Landing snaps the height.
     - So bit-identical states recur at every stop, contact and landing.
   - **Between them:** states merge within the width the executor can correct.
   - **Memo:** every memo hit is verified by exact replay.
   - *Killed if:* a replayed memo hit diverges beyond its width, or two canonical states that should merge differ.

6. **Getting around never modifies the world** (principle 20; S13, S14).
   - **The rule:** a tool use is legal only if the world is identical once it is undone. A bucket's water must destroy nothing, create no source and change nothing once it is taken back. A trapdoor is never moved, even to crawl.
   - **No breaking:** nothing is broken or placed to get around, because "there's many ways to move in minecraft".
   - **Each use is checked:** the legality check runs against the law (flow, neighbours, what the water reaches before it is removed), never assumed.
   - *Killed if:* after any trip, one block state anywhere differs (including redstone state, fluids and trapdoors), or the inventory is short.

7. **Consumables need a budget; health and food are constraints.**
   - **Consumables:** pearls and durability are spent, and nothing is gathered or regenerated yet. Their price is open (S13). Until he decides, the bot spends only what a command grants, which is zero by default.
   - **Health and food:** these regenerate, so they bound which routes are feasible rather than being spent.
     - Sprinting stops at food 6.
     - A fall of 23 blocks does 20 damage, enough to kill from full health.
   - **Side effects:** a pearl can spawn an endermite (5%, except on peaceful), which modifies the world, so pearls also fall under claim 6.
   - *Killed if:* on the R8 rungs, a consumable is spent beyond its budget, or a route ends below a health or food constraint.

8. **The hierarchy is min-plus tables at sub-chunk cells.**
   - **Tables:** each cell keeps the times between its border states (CRP-style), keyed by toolkit class, not by item counts.
   - **Sharing:** identical cells share one table. On default terrain that pays at sub-chunk scale, not at chunk scale.
   - **Repair:** one changed block (changed by someone else, since the bot never edits) re-solves its cell and the tables above it. A toolkit change re-checks the tables that depend on it, so improvements are found as well as breaks.
   - *Killed if:* the border-state probe (Phase 2.0) shows the number of border states per cell exploding once speed is included, or the scale rung stays red after Phase 4.

9. **The light cone is computed per toolkit.**
   - **The bound:** the cone's speed limit is the fastest motion the toolkit allows:
     - 7.1 m/s sprint-jumping;
     - 19.3 m/s in an ice tunnel;
     - ice boats, since a boat reads the friction of the block under it;
     - pearls thrown while moving, which inherit the thrower's velocity;
     - falling at 78 m/s, within the health constraint;
     - the Nether at ×8.
   - **Its two jobs:** the cone bounds what to shape (where the bot can be before a deadline) and where plans can break (where Julien or the world can change things first).
   - *Killed if:* any measured T, the bot's or Julien's (see [ladder.md](ladder.md)), beats the cone's lower bound for that toolkit.

10. **Edge width measures when mistakes are allowed.**
    - **The width:** every edge records how far execution may miss and still land in the same future.
    - **Noise first:** the real noise is named and measured before any gate relies on it: tick misalignment between client and server, the server's RNG, block-update latency.
    - **Pricing:** narrow edges cost more once that noise is priced in.
    - *Killed if:* under the measured noise, a certified edge fails within its width.

11. **Coordinates are chosen per regime; the system is unchanged** (principle 14).
    - Surface coordinates on the sheets, arc parameters in flight, volumes in water and in elytra flight.
    - Each choice must reproduce the law within its width. This claim can only fail once a coordinate choice other than Cartesian exists.

12. **The Nether is reached only through portals that already exist.**
    - It is a 3D cave with horizontal distances divided by 8.
    - Portals wait 80 ticks.
    - Generating a new destination portal modifies the world, so it is ruled out.
    - *Killed if:* the R10 rung misprices a route through an existing portal.

13. **The field is certified by local consistency** (least time, S20 to S23).
    - **The check:** the field is right at a state s when T(s) = min over inputs u of [1 + T(next(s, u))], the Bellman equation, discrete Hamilton–Jacobi. Checking it reads one step ahead, so every state can be checked independently, on threads or the GPU, against the law.
    - **Repair in both directions,** as LPA* and D* Lite do. A slower block (a wall, a builder's new block) raises T only for states whose optimal route passed through it: its shadow. A faster one (an opened gap, a new tool, a block removed) spreads outward from that spot like a new light source and can lower T for states whose routes never touched it ("an old route can remain valid while becoming poor", X2). Only states made inconsistent are repaired.
    - **Where:** fields are kept where they are reused: goals shared by many bots, and a tube around each bot's route. A lone trip uses a forward search guided by the light cone.
    - *Killed if:* after a random block change on a fixture, the repaired field differs from a field rebuilt from scratch, or the repair touches states outside the inconsistent set.

14. **Ridges are where precision is spent.**
    - **The ridge:** where two families of optimal routes meet (left of the pillar or right), the field has a crease. Near it, a tiny error flips the plan.
    - **The executor** keeps the plan it holds near a ridge unless the other side is better by more than the noise (hysteresis), so the bot never dithers.
    - **Precision:** certified widths narrow at ridges; that is where refinement is spent (ES 21). Swarms split there for free.
    - *Killed if:* on a course built around a ridge, with the measured noise, the bot switches plans more than once, or loses more T than the gap between the two routes.

## The four questions (X3)

- **Breaking a wall versus going around:** the bot never breaks it. A wall leaves only the ways through that already exist: around it, over it, through an opening, or a bucket climb that leaves no trace.
- **One-high holes:**
  - For a standing or sneaking bot, they don't exist in the space.
  - With an existing trapdoor or water to enter the crawling pose, they exist at crawling speed.
- **When a bucket is excellent:** it is excellent wherever its water, placed and taken back, leaves no trace, and the fall or climb it allows saves T. A 100-block cliff becomes about a 3-second road: 61 ticks of falling, plus the time to place and take back the water. The bucket's value at a spot is T without it minus T with it.
- **When it must sprint for a jump:** the far side of a long gap connects only to states that are already fast. So the route passes through a run-up, and backs off for more runway if it needs it.

## Still open

- **Compactness:** whether border states stay compact once speed is included. The probe decides, before the geometry is designed.
- **Consumable prices:** Julien's call (S13).
- **Goals for builders** (S23): a builder's goal is rarely a cell. It is any state from which the block it must place is in reach and in sight, which is a region rooted at the block. Raised in chat on 5 November; not yet decided.
