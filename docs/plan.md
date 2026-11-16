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
| **The approved design** (S165; Julien: "this is a beautiful plan i approve it") | the full write-up: [`jobs/unified-plan/plan.md`](../jobs/unified-plan/plan.md); folded into [learning.md](learning.md) section 0 (the one loop), [representation.md](representation.md) sections 10–11 (the space, a crystal with defects), [scales.md](scales.md) (how the scales talk), [vocabulary.md](vocabulary.md) version 3 (every word takes a count) |
| **Learning on top of it** (S169) | [learning.md](learning.md) section 2; the review behind it: [`jobs/unified-plan/learning-review.md`](../jobs/unified-plan/learning-review.md) |
| **The kill tests** (each step's gate) | [learning.md](learning.md) section 6 |
| **Why the old spec was dropped** | [reference/postmortem-2026-11-16.md](reference/postmortem-2026-11-16.md): it forked easy from hard |

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
| 1 | **The core** | the word store keyed by zone label; a count on every word; binade crossings and food as events; two neighbouring yaws for any bearing (the two-yaw test, `P3`, is folded in); `solve`, the single entry point | the million-block line thought offline, cold and warm; the same work at 10³, 10⁴ and 10⁶ blocks; a planted shortcut path for flat ground must fail | **Astra building** (`jobs/u1`, spec `jobs/u1/spec.md`, branch `wt/u1-astra`) |
| 2 | **Joints and sets** | at each joint, the set of arriving states from which a known route reaches the goal (plateaus, premises, prices); reach between surfaces derived without searching inputs | plateaus stay few as the search refines; the reach map holds his 428 shortcut | red-team: freeze after fixes (`jobs/u2-redteam`); **the integrator's spec is ready** (`jobs/u2/spec.md`); Astra builds it after the core |
| 3 | **The loop on real tracks** | `solve` on his tracks through one binary, against his recorded times | his tracks and the old 21; thinking mirrors terrain | waits for steps 1–2; his test set is being drawn up (`jobs/flight-corpus`) |
| 4 | **The bot's runner** | the client plays word × count, checks at events, re-anchors on the server's packets, and changes goal mid-run | first move never waits on thinking; a new goal at tick 1000 | waits for step 3 |
| 5 | **Editing the world** | place, break and interact as words; every edit owes its undo (the ledger) | his five tracks that need it | the port feeds it (GLM) |
| 6 | **The far map** | long trips: shared tables by content, landmarks, far detail kept by its effect on T | a planted one-block bridge 1,000 blocks away is kept | after step 4 |
| 7 | **Other players** | their light cones on the timing wheel; only the parts they can reach are rethought | a player crosses the corridor, one segment is rethought | after step 6 |
| 8 | **Carried movement and wormholes** | boat, elytra, pearl, Nether portal | a route with a pearl, boat or elytra never beats the kit's lower bound | the port feeds it |
| 9 | **The whole machine** | threads for the loop's parts, then the GPU for learning's batches, each only after a measured gain | cost per bot at N bots | last |

## Also in flight

- **The clicked face as B's target** (S170): the mod part is in his game. The planner part waits for a judge (`wt/track-face`).
- **The rebuilt hard track** (`wt/hard-track-2`): built, waiting for a judge. Julien rates it easy-medium.
- **Freeze damage in the law** (`wt/l0-veto`, job `1.3e`): built, waiting for a judge since 15 November.
- **His maps:** eight imported as dimensions, switched with `/map`; 51 tracks raced on 16 November with their inputs recorded.

## Who does what (AGENTS.md, S174, S173)

| Work | Who |
|---|---|
| Specs | the integrator, checked against this plan |
| Important design and the core | Opus (red-teams, one-way doors) and Astra (the core's build) |
| Well-specified pieces | Terra |
| Complex questions and wide reading | Hydrogen; Argon for primary sources |
| The port only | GLM |
| Judging GPT's work | Sonnet |
