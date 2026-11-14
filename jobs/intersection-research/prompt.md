# Research job: are we representing what the player can touch the smart way?

You read and research; you edit nothing but your report. Every claim about our code cites file and line; every claim about another field names its primary source (author, title, year) and says whether you read it in this run or recall it.

## Julien's question (S116, verbatim)

> are we properly representing things? because we represent them by basically the shape that they have in how they interact with the player but in the way we are doing it are we smart? gaming must have extremely smart solutions to this that we may have missed, as basically we're trying to see if there's intersections or not, same research principle as always, based of constraints and searching in fields with skin in the game and so on.

## Why he asks now

This afternoon the planner crouch-walked 170 ticks over buttons and pressure plates that are air to the law (`jobs/butoon-diagnosis/last.md`): the planner's own free-space test (`engine/plan/src/arith.rs:18-22`, `plain`) and its cycle test (`arith.rs:90-106`, `Cycle::layered`, which compares cells by pointer) disagreed with the law about which cells are the same. On straight flat tracks the planner makes one decision per jump (430 on 800 blocks) and thinks 2–4 orders of magnitude above the Fermi floor of ~10–100 µs for a straight line (`docs/HANDOFF.md`, "Now"). Both say the representation of "what the body can touch" may be the wrong shape.

## What we do now (read these; the main repo is `C:\Users\PASCAL\Desktop\path tracing`, the planner on `D:\path-tracing-wt\play`)

- **A block, to the law:** `engine/law/src/world.rs:182-252` (`Block`: a `VoxelShape` of boxes plus ~30 fields: friction, speed and jump factors, fluid, inside effect, climbable, stuck multiplier, bounce...), loaded from the jar-derived table `engine/world/data/models.tsv` by `engine/world/src/snapshot.rs` (~500 distinct models, one shared pointer per model).
- **The tick:** `engine/law/src/movement.rs`, a bit-exact port of Mojang's 26.2 movement: the player's box swept per axis against the boxes of the cells it overlaps, clamped in doubles.
- **What a tick read:** footprints (`jobs/1.2-footprints/`), and `engine/law/src/pattern.rs` (`cell_digest` under a read mask, `PatternTable` keyed by the digests of the cells a word read).
- **Classes and words:** `engine/words/src/class.rs`, `segment.rs`, `table.rs`; the framework `docs/representation.md` (sections 1–5) and `docs/scales.md` 4.7.
- **The planner's free-space shortcuts:** `engine/plan/src/arith.rs`, `reach.rs`, `search.rs` on the play worktree.

## The constraints any answer must meet (from `docs/PRINCIPLES.md`, `AGENTS.md`)

1. **Bit-exact at the tick.** The law reproduces Mojang's doubles; an approximation is allowed only above the tick (words, the medium) as a stated bound, never as a tolerance or epsilon.
2. **Scale:** 10⁴ bots, ~10⁶ distinct patterns over a 10k × 10k area; compute scales with what is new, not bots × ticks × places.
3. **No per-block special cases and no numbers of ours:** two cells are the same exactly when every comparison the law makes on them answers the same.
4. **No new search code:** a capability enters as law and data; needing new geometry or search means the representation is wrong.
5. **Fermi floors before benchmarks.**

## The work

1. **Name the problem in each field's terms.** We ask: which cells can a moving box touch, and does touching them change its motion? State that as the collision-detection people, the motion-planning people and the voxel-rendering people each would.
2. **Search fields with skin in the game**, where a wrong or slow answer costs money or lives, and bring back the ideas that change our design. Leads only, chase what pays: game physics (broad phase, sweep-and-prune, swept and continuous collision, voxel engines), robotics motion planning (configuration-space obstacles, Minkowski sums: shrink the body to a point and grow the world), GPU ray traversal of voxel grids, sparse voxel structures, chip layout rule checking, Minecraft's own speedrun, TAS and pathfinding tools, and the equivalence side (state minimization, hash-consing, coarsest partitions). Add fields we have missed.
3. **For each idea that survives the constraints:** what it is, its primary source, how it maps onto our objects (cite our file and line), a Fermi estimate against what we do now, and *Killed if*. Say plainly which ideas we already have under another name.
4. **Treat the extremes:** a 800-block straight run, the button-and-plate floor, a 1-pixel gap at the limit, a staircase of slabs, scaffolding, powder snow, water columns.

## Your report (under 120 lines)

1. What we do now, in one paragraph, with its weak points cited.
2. The ranked ideas (at most 7), each with the four parts of step 3.
3. The one change you would make first, and the measurement that would tell us it worked.
4. What you could not read or verify, by name.
