# The plan

The one place to see what we are building, in what order, and where each piece stands. It is rewritten in place as things change. Plain names only: a code (a branch, a job folder) appears in backticks after its plain name, never alone (S179).

## The goal

The bot reaches the point Julien clicks in the least time T, counted from the click's server tick to arrival, thinking included (PRINCIPLES, S6, S124). It is held against two extremes at once (S131):
- a million-block straight line with bare legs, thought in microseconds;
- a hard track: "many types of mechanics that are very precise and chained in convoluted 3D space over great spans but to the bot it should not look like that" (S166).

**One mechanism for every terrain** (S161, S165). There is no "easy" path beside a "hard" fallback. Compute grows with what the terrain holds, never with its length.

## Where the design lives

| What | Where |
|---|---|
| **The approved design** (S165; Julien: "this is a beautiful plan i approve it") | the full write-up: [`jobs/unified-plan/plan.md`](../../jobs/unified-plan/plan.md); folded into [learning.md](learning.md) section 0 (the one loop), [representation.md](representation.md) sections 10–11 (the space, a crystal with defects), [scales.md](scales.md) (how the scales talk), [vocabulary.md](vocabulary.md) version 3 (every word takes a count) |
| **Learning on top of it** (S169) | [learning.md](learning.md) section 2; the review behind it: [`jobs/unified-plan/learning-review.md`](../../jobs/unified-plan/learning-review.md) |
| **The kill tests** (each step's gate) | [learning.md](learning.md) section 6 |
| **Why the old spec was dropped** | [reference/postmortem-2026-11-16.md](../../archive/postmortems/postmortem-2026-11-16.md): it forked easy from hard |

**The loop in one breath:**
1. Bracket the time to B from the kit's light cone.
2. If the next word is already certain, move.
3. Otherwise take the biggest word the kit holds and extend it until its certificate breaks.
4. Cut there, and solve each part the same way, in parallel.
5. Compose the answers backward; the bot goes forward.
6. Spend thinking where it can change the decision due soonest.

## What already exists

| Piece | What it is | State |
|---|---|---|
| The direct channel (`slice A`) | the bot's mod and the planner in one process (JNI), reading the bot's real state bits | built, judged pass with findings, on `wt/kernel` |
| The class codec (`slice B`) | one way to say what a block is to the kit's verbs (legs, break, place) | built, judged pass with findings, on `wt/kernel` |
| The stored-opening test (`P1`) | a straight run's opening ticks are computed once, then reused with zero ticks | pass after fixes; 48–78 µs in debug, the floor lookup included |
| The exact-integer-maps test (`P2`) | inside one floating-point binade, a gait's position is an exact integer map | pass with findings |
| The food-guard test (`P4`) | the law's own food > 6 sprint guard as an event; on Peaceful, food never stops sprinting | pass offline; the live check comes with the bot's runner |

## The order of work

| # | Step | What it is | Gate | State |
|---|---|---|---|---|
| 0 | Fold the plan into the docs | design and learning into the living docs | — | done 16 November |
| 1 | **The core** | the word store keyed by zone label; a count on every word; binade crossings and food as events; two neighbouring yaws for any bearing (the two-yaw test, `P3`, is folded in); `solve`, the single entry point | the million-block line thought offline, cold and warm; the same work at 10³, 10⁴ and 10⁶ blocks; a planted shortcut path for flat ground must fail | **built** (`wt/u1-astra`): all gates pass; oblique million line ~600 µs warm; judged pass with findings (two gates that cannot fail, fixed first in step 2) |
| 2 | **Joints and sets** | at each joint, the set of arriving states from which a known route reaches the goal (families, points, premises, prices); reach between surfaces derived without searching inputs | plateaus stay few as the search refines; the reach map holds his 428 shortcut | built (`wt/u2-astra`): the reach map holds his shortcut (158 ticks after the ladder against his 169); pieces re-derived by Opus as families and points (S188), now built in step 2b |
| 2b | **The simplest case right** (S186, S187) | after the set re-derivation, in one job: the warm straight line with zero player-state copies and zero allocations (counted, so it cannot be faked); the stop on B stored as a joint and looked up; "it is important to get even the simplest case right, if we can't make them right the rest will compound badly" | the warm million line against the ~5 µs floor, with what remains | **built, judged pass with findings** (`wt/u2b-astra`, `e6de417`; `jobs/u2b-judge/last.md`: one dominant-axis branch in solve, the frozen gate's fixtures unhashed, the recurrence guard ignored, fixed first in Astra's `jobs/warm-floor`): families and points with the pinned exit as a point or a hold (S203); zero copies and allocations reached. **Thinking per real command** (S223; a new goal each time, the world in memory, one living store, `D:\path-tracing-evidence\real-time-20261117.log`): **median 233 ms from standing** (160–450 ms, about 4,700 law ticks re-prepared every command), **253 ms while moving, and 33 of 60 mid-run retargets refused**. The 57 µs once reported as "warm" is the same goal asked twice. |
| 3 | **The loop on real tracks** | `solve` on his tracks through one binary, against his recorded times | **the Diversity practice tracks are neo jumps over a floor: the bot must match or beat his T there, since the floor lets a slower non-neo route reach B** (S189; his scores are not all perfect); his tracks and the old 21; thinking mirrors terrain | spec written and checked by the integrator (`jobs/loop-tracks`; his test set from `jobs/flight-corpus`, verified): **Sol 6.2 makes his tracks inputs** (terrain from region files of any dimension, the clicked face, the settled start, a harness counting like his track clock) from 18 November 00:35; **Astra closes the loop in `solve`** after her per-command work is judged |
| 4 | **The bot's runner** | the client plays word × count, checks at events, re-anchors on the server's packets, and changes goal mid-run | **the goal changed again and again during a run** (S189), and first move never waits on thinking; a new goal at tick 1000 | waits for step 3 |
| 5 | **Editing the world** | place, break and interact as words; every edit owes its undo (the ledger) | his five tracks that need it | the port feeds it (GLM) |
| 6 | **The far map** | long trips: shared tables by content, landmarks, far detail kept by its effect on T | a planted one-block bridge 1,000 blocks away is kept | after step 4 |
| 7 | **Other players** | their light cones on the timing wheel; only the parts they can reach are rethought | a player crosses the corridor, one segment is rethought | after step 6 |
| 8 | **Carried movement and wormholes** | boat, elytra, pearl, Nether portal | a route with a pearl, boat or elytra never beats the kit's lower bound | the port feeds it |
| 9 | **The whole machine** | threads for the loop's parts, then the GPU for learning's batches, each only after a measured gain | cost per bot at N bots | last |

## Also in flight

- **The clicked face as B's target** (S170): the mod part is in his game. The planner part waits for a judge (`wt/track-face`).
- **The rebuilt hard track** (`wt/hard-track-2`): built, waiting for a judge. Julien rates it easy-medium.
- **Freeze damage in the law** (`wt/l0-veto`, job `1.3e`): built, waiting for a judge since 15 November.
- **His maps:** eight imported as dimensions, switched with `/map`. The four with tracks are kept (S193): `infinity_tower`, `diversity_parkour`, `parkour_race`, `frosted_ices`; the four unused were removed (S194). Added on 17 November: `easy_dropper`, `the_drop_2`, `delightful_dropper_2`, `elytra_challenge_2`, `elytra_challenge_3`, `parkour_rectangle_3`, `race_the_rainbow` (spawns checked and corrected, `jobs/track-ux`, merged 17 November with autocomplete, no `/track restore`, faster arming and the run-will-count messages). 51 tracks raced on 16 November with their inputs recorded. Wanted next: droppers, elytra courses and maps whose progression survives the import (S192; `jobs/map-scout-2`).

- **The port, by a better method** (S207–S211): the game itself judges a port, through a differential harness running Mojang's methods headless against the Rust law. A coverage map from the jar measures what is done. Every meter works a ranked backlog, and a speed pass is certified by the same harness. GLM writes port code against the harness; Argon builds the tooling once the launcher lets Antigravity run commands (`jobs/agy-commands`). Wave 2: 28 lenses queued (`jobs/port-wave-2`).
- **Every new command on flat ground in microseconds, in total** (S225, the first gate of Astra's `jobs/warm-floor`, from 18 November 00:33): today a median 233 ms per new command and 33 of 60 retargets refused while moving (S223), because each command re-prepares its gaits, filed under the exact direction and speed. Opus's idiot-index report ([`jobs/hundred-ns/report.md`](../../jobs/hundred-ns/report.md), S212–S214) gives the how (a yaw-free generator, data shaped for the read, one walk per axis) and the floors (about 10 µs for a new direction, 1 µs after); 100 ns matters only when bots decide every tick (300 ns keeps 10⁴ bots under the server's ~3 ms).
- **Builds for him:** the refuge, pagoda-sized, in five passes; Hydrogen renders and judges its own views between them (`jobs/hearth-small`, the renderer on `wt/build-preview`). Passes 2–4 done; critique and repair next, with the rock made less boxy.

## Who does what (AGENTS.md, S174, S173, S209, S210)

| Work | Who |
|---|---|
| Specs | the integrator, checked against this plan |
| Important design and the core | Opus (red-teams, one-way doors; none until 23 November, S226) and Astra 6.2 (the core's build; the Codex window is hers) |
| Strong builds beside the core | Sol 6.2 (S217, S225) |
| The leftover ~20% | Terra |
| Building tooling, reading, our docs linted against his words | Argon, on its own meter |
| Premise checks, digests, builds | Hydrogen (reports to the integrator; a smaller model never writes for a bigger one) |
| The port's code, against the harness | GLM |
| Judging GPT's and Google's work | Sonnet |

## Later, in order, once most of the above works (S189)

- **Death as a tool.** Death is not game over: it is a teleport to the respawn point, with a toll (items dropped, unless kept in an ender chest or shulker boxes, and experience). It is T = ∞ only where a constraint forbids it, as in a fight (S189). It joins the wormholes in step 8, beside ender-pearl stasis chambers, a pearl held in water and released on demand: a portal built in advance.
- **Kits and many bots.** A new tool (a silk-touch efficiency V shovel) changes T everywhere. The design keeps that cheap: the world is stored once as blocks, each kit has a projection table from block state to class, and a kit change swaps a pointer. A hill of one material stays one class with a dig rate, not a thousand blocks to update; certificates are touched only through their footprints. Bots with the same kit class share fields. The flow of tools between bots that minimises total T is prices as fields (representation.md section 8, S96). This needs its own red-team before step 5.
- **Seed cracking** for unknown servers: plan the unseen terrain from the cracked seed, with seen chunks always overriding it (S189).
- **The real end goal:** from an omniscient bot toward robotics-like constraints, with less information, sensing only what a player senses, "step after step" (S189).
- **A ranking of the hardest maneuvers by error margin** (S190): every word carries its width (clearance and delay tolerance, vocabulary.md section 7), so the store can rank the game's narrowest maneuvers. That is hardness for the law, not for a human.
- **Stasis networks** (S190): a network of ender-pearl stasis chambers that summons a whole division of bots at once, triggered by one bot, a redstone clock or other tech. "very powerful and cheap tool which is an alien like behavior that i love and extremely suited to a swarm." It should be able to emerge, not be catalogued.
- **Redstone as the litmus test for intelligence, not efficiency** (S190, with S32's litmus): if words compound, redstone's convoluted, indirect chains should emerge more and more complex. A rung for it waits for editing the world and interaction.
- **The swarm-building benchmark** (S197): Hydrogen's pagoda, 797 commands and 7,951 blocks, kept as a blueprint in `jobs/pagoda/build.txt`, with the placed copy at x 60–92, z −16–16. When the swarm builds (representation.md section 9: a blueprint is a list of edits; reverse demolition; least slack first), the bots build this pagoda, and their makespan is measured against the critical path and work ÷ bots (scales.md 4.4).
