# The ladder

This file holds the scoreboard and the rungs. Each rung is frozen before its first run, with its course, numbers and pass line written down, and is never weakened afterwards. The first failing rung is the next work item. A rung that cannot fail is a control, not evidence.

## The scoreboard

- **T:** server ticks from the tick in which the server receives the goal command, to the tick in which the bot stands in the goal cell, with every edit restored and every temporary item back in its inventory.
- **The bracket:**
  - LB is the light cone's lower bound for the trip. It is admissible: no run may ever beat it.
  - UB is the best T found by an independent offline search over raw tick controls, never over the planner's own moves.
  - Each rung reports the distribution of T / UB and of the bracket width UB / LB.
- **Human baseline:** Julien runs the same A to B himself, and the time he takes is recorded "for grounding in T not in method" (S14).
  - **His time is the minimum threshold** (S25): on every track he has run, the bot's T must be at most his best T. Surpassing him is the aim.
  - Tracks (S25): he defines A and B on the superflat; T runs from the tick the body leaves A to the tick it touches B, measured by the same detector for him and for the bot. The bot's latency from command to first move is reported separately.
  - Only his time is used, never his path.
  - He runs it in survival with the same toolkit as the bot, without modifying the world.
  - His T is a real upper bound: a route that fast exists. Every rung reports the bot's T / his T next to T / UB.
  - For the harder rungs this is the ground truth that the offline search can't reach.
- **Latency:** ticks from the whisper's arrival at the bot's client to its first applied control. The p99 must be ≤ 1.
- **Failures:** failed edges, server corrections, restoration misses and irreversible losses. Each must be zero.
- **Scale:** planning cost against trip length, in work units and in wall time.
- **Swarm:** the planner will serve hundreds of bots (S17), so cost is also reported per bot, with many bots on one machine. How it is measured is frozen with Phase 10.
- **Reports** give n, min, median, p90, p99 and max, plus the worst seeds with their replays. Never an average alone, never a trimmed outlier. A result holds only inside the range it was observed in.

## Fixtures

- **Worlds:** pre-generated default worlds with fixed seeds, kept on D:.
- **Server:** the lab server runs on port 25566 (RCON on 25576).
- **Courses:** built on those worlds through RCON.
- **No superflat:** every superflat chunk is the same, so the memo, repair and scale tests could not fail there.
- **Isolation:** gates take the rig lock and run under `/tick rate`, so each test stays under 60 seconds. Julien's player stays away from the bot, because players push each other.

## How we test (from ES, adopted 5 November, S19)

Julien, in ES: "test like a spartan would train, harder than for the war"; "extremes inform the mean"; "even baby spartans started 'small'". Sources: ES `docs/test-ladder.md`, `docs/method.md`, PRINCIPLES 10 and 20.

The purpose of a test is to shorten the time until we notice we are fooling ourselves, not to prevent every issue.

1. **Every capability gets a ladder.** Rungs are frozen before they run, each about 10× harder than the last on the one dimension it stresses. Climb until a rung fails.
   - The last passing rung is what the bot can do.
   - The first failing rung, with the mechanism that limits it, is the next work item.
   - A ladder with no failing rung has not been climbed far enough.
2. **Two kinds of rung, both required.**
   - **Extreme:** one dimension pushed to its limit, everything else trivial. It finds which dimension breaks, and how cost grows along it.
   - **Chaotic:** many mechanisms meeting at the scale of the goal. It finds what breaks when they meet. For this project, that means real terrain where ice, water, soul sand, ceilings and drops chain together, re-targets mid-air, and, once there are several bots, crowds of bots pushing each other.
3. **Worst cases are built on purpose:**
   - exact ties in the collision axis order;
   - grazing a block corner at the body's exact width;
   - simultaneous contacts, and step-ups on edges;
   - gaps at the sprint-jump limit;
   - coordinates far from the origin, where doubles lose precision;
   - the highest speeds the law allows;
   - chunk borders.
4. **A test that cannot fail is not evidence.** Every gate has a positive control that must pass, and a planted fault that must fail.
   - The planted fault is a mutant in rarely run code, chosen by the red-team at freeze, never by the builder.
   - Easy fixtures are controls, not rungs.
5. **Correctness before speed.** Small rungs compare against an independent reference: the bot's own client, or the offline search. Large rungs keep invariants, exact replay and digests. A fast wrong answer is a failure.
6. **Behaviour over many seeds.** Behaviour gates draw many seeds from a frozen sampler and judge the distribution. One seed's data is never the target. Determinism is checked seed by seed, bit for bit.
7. **Statistical etiquette** (as in the scoreboard).
   - Describe the behaviour before choosing the test: what should happen, what must never happen, what the tails look like.
   - Assume fat tails until thin tails are shown.
   - Verdicts are k of n beyond a frozen line, plus the worst case. Worst cases are compared at equal n only.
   - Count independent events, not runs that share a harness.
8. **Anomalies are the most important data.**
   - Every run writes a record that replays exactly: inputs, seeds, commit, law version, each invariant's worst margin, every refusal.
   - A broken invariant, a new worst margin, a server correction, or two representations disagreeing goes to the anomaly log automatically. Logging is free and blames no one.
   - An unexplained anomaly blocks only the claims that depend on it, which then show as "unexplained", not "pass".
   - Once explained, it becomes a generator for its family, or a fix to the law.
9. **Unknown unknowns: neither ignore nor hunt them.** Meet them with fewer parts, records that replay exactly, independent judges once a part is load-bearing, and fast recovery. Julien's play is the sensor we cannot build: when he and a metric disagree, the metric is presumed broken.
10. **Every test runs in under 60 seconds** (under `/tick rate` where needed). A known red is baselined: it has an owner, it may only get better, and it is never called a pass.

Not yet adopted from ES's method: the merge command and the commit hooks (`method.md`, M7 to M10). They come when merges to `main` become frequent.

## Rungs

When a rung below is frozen, it is frozen as a ladder: its extreme tracks, each with rungs about 10× apart, plus its chaotic track.

| Rung | Phase | What it stresses | Status |
|---|---|---|---|
| L0 | 1 | **The law.** Fixtures for each terrain class: full blocks, slabs and stairs, step-ups, ceilings, ladders and vines, water, ice and blue ice, slime, soul sand, honey, cobweb, powder snow, and edges for the sneak back-off. Many random input sequences each. **Pass:** bit-identical positions and velocities on every tick; damage and exhaustion equal the server's; zero corrections; every mutant caught (axis tie order, step-up, the sneak back-off at edges, ceiling bonks) | to freeze before 1.5 runs |
| L0-x | 1 | **The law at its extremes**, a second rung beside L0, which was frozen before this method arrived: coordinates at 10⁴, 10⁶ and 3×10⁷; exact axis-order ties; grazing corners at the body's width; step-ups on edges and corners; elytra-scale speeds into walls; chunk borders. Same pass line as L0, with a red-team-chosen mutant. It is climbed beside the first flight and never blocks it | to freeze before L0's first run |
| Legs ladder | 2 | Frames R1 to R-scale as one ladder. **Extreme tracks:** trip length, maze depth, gap width up to the sprint-jump limit, corner precision, distance from the origin. **Chaotic track:** default terrain from fixed seeds with re-targets. The tracks and their rungs are frozen with R1 to R-scale | to freeze before Phase 2 runs |
| R1 | 2 | Flat trips, straight and diagonal, 10 to 200 blocks | to freeze before Phase 2 runs |
| R2 | 2 | Walls and mazes on legs alone: detours | to freeze before Phase 2 runs |
| R3 | 2 | Gaps of 1 to 4 blocks, step-ups, drops up to the 3-block safe fall. Damaging drops are barriers until tokens exist | to freeze before Phase 2 runs |
| R-retarget | 2 | A new goal at a random tick mid-trip. T counts from the new command; momentum carries over | to freeze before Phase 2 runs |
| R-scale | 2 | Trips of 10, 10², 10³ and 10⁴ blocks along a fixture corridor of default terrain. Frozen with Phase 2 and red until Phase 4 bends it | to freeze before Phase 2 runs |
| R-brach | after 2 | **The brachistochrone, as a falsifier of the method** (S17). Elytra with no fireworks, from rest on one pillar to a lower one, in still air. Needs only the elytra law (pitch and yaw controls; its `Math.cos` exactness class) added as law data, with **no new search or geometry code**. **Pass:** T within the bracket, whose UB is the independent time-optimal search over raw controls for this exact law; and the dive-then-flatten curve appears without being asked for. The reference is that UB curve, not the textbook cycloid, because the law has drag and pitch-dependent lift. **Kills:** no dive means speed was dropped from the state or something other than T was minimised; a kinked dive means states were merged too coarsely at speed | frozen once Phase 2 passes, before its first run |
| R4 | 3 | Movement in a world left unmodified: sneak edges, ladders and vines, water, slime, crawling under existing trapdoors. Pass includes every block state unchanged after the trip | when Phase 3 starts |
| R5 | 3 | Courses Julien designs and runs himself, judged against his time | when Phase 3 starts |
| R6 | 5 | The bucket where its water leaves no trace: falls, climbs; the Nether (no water) | when Phase 5 starts |
| R7 | 6 | Momentum ground: ice tunnels, slime, soul sand, honey | when Phase 6 starts |
| R8 | 7 | Tokens: pearls, food, health, durability | when Phase 7 starts |
| R9 | 8 | The moving world: fluids, falling blocks, mobs, redstone | when Phase 8 starts |
| R10 | 11 | Far travel: boats, the Nether sheet, elytra | when Phase 11 starts |
