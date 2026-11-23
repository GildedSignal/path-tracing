# The plan

The one place to see what we are building, in what order, and where each piece stands. It is rewritten in place as things change. Plain names only: a code (a branch, a job folder) appears in backticks after its plain name, never alone (S179).

## The goal and the loop

The goal, the two extremes and the one mechanism are in [goal.md](../goal.md); the loop is in [architecture.md](../architecture.md) section 3.

## Where the design lives

| What | Where |
|---|---|
| **The approved design** (S165; Julien: "this is a beautiful plan i approve it") | the full write-up: [`jobs/unified-plan/plan.md`](../../jobs/unified-plan/plan.md); folded into [learning.md](learning.md) section 0 (the one loop), [representation.md](representation.md) sections 10–11 (the space, a crystal with defects), [scales.md](scales.md) (how the scales talk), [vocabulary.md](vocabulary.md) (every word takes a count) |
| **Learning on top of it** (S169) | [learning.md](learning.md) section 2; the review behind it: [`jobs/unified-plan/learning-review.md`](../../jobs/unified-plan/learning-review.md) |
| **The kill tests** (each step's gate) | [learning.md](learning.md) section 6 |

## What exists

| Piece | What it is |
|---|---|
| The direct channel (`slice A`) | the bot's mod and the planner in one process (JNI), reading the bot's real state bits |
| The class codec (`slice B`) | one way to say what a block is to the kit's verbs (legs, break, place) |
| The stored-opening test (`P1`) | a straight run's opening ticks are computed once, then reused with zero ticks (48–78 µs in debug, floor lookup included) |
| The exact-integer-maps test (`P2`) | inside one floating-point binade, a gait's position is an exact integer map |
| The food-guard test (`P4`) | the law's own food > 6 sprint guard as an event; on Peaceful, food never stops sprinting |

## Step gates (order and state: [now.md](../now.md))

| # | Step | What it is | Gate |
|---|---|---|---|
| 1 | **The core** | the word store keyed by zone label; a count on every word; binade crossings and food as events; two neighbouring yaws for any bearing (the two-yaw test, `P3`, is folded in); `solve`, the single entry point | the million-block line thought offline, cold and warm; the same work at 10³, 10⁴ and 10⁶ blocks; a planted shortcut path for flat ground must fail |
| 2 | **Joints and sets** | at each joint, the set of arriving states from which a known route reaches the goal (families, points, premises, prices); reach between surfaces derived without searching inputs | pieces stay few as the search refines; the reach map holds his 428 shortcut |
| 2b | **The simplest case right** (S186, S187) | the warm straight line with zero player-state copies and zero allocations (counted, so it cannot be faked); the stop on B stored as a joint and looked up; "it is important to get even the simplest case right, if we can't make them right the rest will compound badly" | the warm million line against the ~5 µs floor, with what remains |
| 3 | **The loop on real tracks** | `solve` on his tracks through one binary, against his recorded times | **the Diversity practice tracks are neo jumps over a floor: the bot must match or beat his T there, since the floor lets a slower non-neo route reach B** (S189; his scores are not all perfect); his tracks and the 21 tracks; thinking mirrors terrain |
| 4 | **The bot's runner** | the client plays word × count, checks at events, re-anchors on the server's packets, and changes goal mid-run | **the goal changed again and again during a run** (S189), and first move never waits on thinking; a new goal at tick 1000 |
| 5 | **Editing the world** | place, break and interact as words; every edit owes its undo (the ledger) | his five tracks that need it |
| 6 | **The far map** | long trips: shared tables by content, landmarks, far detail kept by its effect on T | a planted one-block bridge 1,000 blocks away is kept |
| 7 | **Other players** | their light cones on the timing wheel; only the parts they can reach are rethought (two-regime singularity boundary once contact is entered: pure crystal outside, dense `k = 1` solver inside; reference [pvp-strategy-2026-11-22.txt](../../archive/research/pvp-strategy-2026-11-22.txt)) | a player crosses the corridor, one segment is rethought |
| 8 | **Carried movement and wormholes** | boat, elytra, pearl, Nether portal | a route with a pearl, boat or elytra never beats the kit's lower bound |
| 9 | **The whole machine** | threads for the loop's parts, then the GPU for learning's batches, each only after a measured gain | cost per bot at N bots |

## Tracks, the port, and current floors

- **His maps:** imported as dimensions, switched with `/map`: `infinity_tower`, `diversity_parkour`, `parkour_race`, `frosted_ices`, `easy_dropper`, `the_drop_2`, `delightful_dropper_2`, `elytra_challenge_2`, `elytra_challenge_3`, `parkour_rectangle_3`, `race_the_rainbow`. Wanted next: droppers, elytra courses and maps whose progression survives the import (S192).
- **The port, by a better method** (S207–S211): the game itself and `server-26.2.jar` bit-witnesses judge the port; that witness suite proves compression and deletion onto shared generators + data tables + thin Mojang scars lossless. GLM extracts parameter rows, scars, jar witnesses, and short `lab/wiki/` notes on demand; builders compress and wire; live flights decide merges to `main`.
- **Every new command on flat ground in microseconds, in total** (S225): [goal.md](../goal.md) holds the current floor and gap; [`jobs/hundred-ns/report.md`](../../jobs/hundred-ns/report.md) (S212–S214) gives the how (a yaw-free generator, data shaped for the read, one walk per axis) and the floors (~10 µs for a new direction, ~1 µs after; 300 ns keeps 10⁴ bots under the server's ~3 ms).

Who does what and what is in flight live in [work.md](../work.md) and [now.md](../now.md).

## Later, in order, once most of the above works (S189)

- **Death as a tool.** Death is not game over: it is a teleport to the respawn point, with a toll (items dropped, unless kept in an ender chest or shulker boxes, and experience). It is T = ∞ only where a constraint forbids it, as in a fight (S189). It joins the wormholes in step 8, beside ender-pearl stasis chambers, a pearl held in water and released on demand: a portal built in advance.
- **Kits and many bots.** A new tool (a silk-touch efficiency V shovel) changes T everywhere. The design keeps that cheap: the world is stored once as blocks, each kit has a projection table from block state to class, and a kit change swaps a pointer. A hill of one material stays one class with a dig rate, not a thousand blocks to update; certificates are touched only through their footprints. Bots with the same kit class share fields. The flow of tools between bots that minimises total T is prices as fields (representation.md section 8, S96). This needs its own red-team before step 5.
- **Seed cracking** for unknown servers: plan the unseen terrain from the cracked seed, with seen chunks always overriding it (S189).
- **The real end goal:** from an omniscient bot toward robotics-like constraints, with less information, sensing only what a player senses, "step after step" (S189).
- **A ranking of the hardest maneuvers by error margin** (S190): every word carries its width (clearance and delay tolerance, vocabulary.md section 7), so the store can rank the game's narrowest maneuvers. That is hardness for the law, not for a human.
- **Stasis networks** (S190): a network of ender-pearl stasis chambers that summons a whole division of bots at once, triggered by one bot, a redstone clock or other tech. "very powerful and cheap tool which is an alien like behavior that i love and extremely suited to a swarm." It should be able to emerge, not be catalogued.
- **Redstone as the litmus test for intelligence, not efficiency** (S190, with S32's litmus): if words compound, redstone's convoluted, indirect chains should emerge more and more complex. A rung for it waits for editing the world and interaction.
- **The swarm-building benchmark** (S197): a pagoda benchmark (Hydrogen, S205), 797 commands and 7,951 blocks, kept as a blueprint in `jobs/pagoda/build.txt`, with the placed copy at x 60–92, z −16–16. When the swarm builds (representation.md section 9: a blueprint is a list of edits; reverse demolition; least slack first), the bots build this pagoda, and their makespan is measured against the critical path and work ÷ bots (scales.md 4.4).
