# The idiot index first, then a Fermi estimate of a swarm on a map, then the 100 ns red-team (S212–S214)

You are Opus, the designer, in a fresh context. You read, measure if a number is missing, and write one report. You build nothing and edit nothing outside `jobs/hundred-ns/`.

## Why this job exists (his words, verbatim, docs/reference/julien-2026-11-10.md)

- **S212:** "what would it take to make our straight line 1M block track to compute in 100 ns warm?"
- **S213:** "this requires to rethink the whole execution and maybe the principles. So i'm speaking any straight line in any case while thinking that it should help the even the most convoluted 3D tool heavy parkour."
- **S214:** "that extreme thinking will be helpful even if not used as is, 1µs is already very good, it's just that it forces us to think in ways that could help us and force to remove useless things or being smarter about how we do them … the goal is basically to reduce the T total spent on a whole map on which a swarm is operating it so it has bound in overall time and size and members that'll help us anchor numbers more properly for the tradeoff of compute vs storing … a world doesn't change that much so maybe we could be smarter BUT tools do change how you move in that world, an elytra is game changing especially when paired with farms. So maybe 100ns changes the balance too much or maybe not. But before we're really trading the compute vs lookup we need to dissolve the idiot index."
- **S83** defines the pass: "an idiot index pass based on fermi estimates in micro meso macro but also holistically meaning to we have too much work where it doesn't matter with input data that doesn't matter, duplicate work, not leveraging things", weighted "by how often the thing is used and fundamental it is"; and "the server will no matter what compute all of those things … might as well leverage that already done computation" (the guided missile).

**The order is his: Part 1 before Parts 2 and 3.** Trading compute against lookup comes only after the useless work is gone.

## Read first

- `docs/PRINCIPLES.md` (the goal, the philosophy, the register rows S83, S124, S131, S165, S186, S191, S212–S214), `AGENTS.md` (no patches, no new search code, no made-up numbers).
- `docs/plan.md` (the steps; step 2b is where we are), `docs/learning.md` sections 0 and 6 (the one loop, the kill tests), `docs/representation.md` section 10 (families, points and holds), `docs/scales.md` (Fermi floors, the swarm, every scale under one rule), `docs/engine.md` (the word-plan output contract), `docs/vocabulary.md`.
- What was measured: `jobs/u1-judge/last.md` item 8 (where the core's 600 µs went, and five fixes); `jobs/u2b/last.md`; the sweep log `D:\path-tracing-evidence\u2b\integrator-sweep-20261117.log` (lines `U2B_WARM`, `U2B_PROFILE`, `U1_SOLVE`, `U1_PROFILE`, `U1_RESULT`). Warm million line on an axis: 57 µs median, zero copies, zero allocations; the core's own timing test reports 112–167 µs warm on the same build. The plan's floor is ~5 µs.
- The code: `D:\path-tracing-wt\u2b-astra\engine\plan\src\` (solve, store, events, lattice, families, terminal, joints, inline, state_view) and what it calls in `engine\law\src\`. Commit `95552dc`.
- The old census: `lab/wiki/idiot-index.md` (12 November: the proposal lookup at ~49× its floor).

## Part 1: dissolve the idiot index

For the warm million line, then for a warm decision on a seen track, then for the cold path: each stage's measured cost against its floor, the ratio (the idiot index), weighted by how often it runs and how fundamental it is (S83). Floors are Fermi estimates with every input stated (operations, cache lines, dependent latencies on this machine), never a guess.

Classify each excess under S83's four heads: work where it doesn't matter; input data that doesn't matter; duplicate work; not leveraging (stored results, symmetry, the server's own computation). For each, say what to delete or the smarter form, and what it saves. Do it at four grains: micro (one solve), meso (one trip, with the goal changed again and again, S189), macro (a swarm on a map for a session), holistic (where the whole system spends work that does not lower T).

## Part 2: a Fermi estimate of a swarm on a bounded map

Anchor the numbers he asked for. State the bounds as inputs you can change: the map's size, the number of bots, the session's length, how often blocks change, how often kits change (a tool, an elytra with fireworks from a farm). From them derive:
- total T over the map and how much of it is thinking on the path from click to move, at 57 µs, 5 µs, 1 µs and 100 ns per decision;
- the CPU each bot costs per trip, beside the server's 50 ms tick;
- what can be stored (fields, sets, tables per goal and per kit class) against what is recomputed: bytes, how often each is invalidated, what fits in which cache;
- where 100 ns changes that balance and where it does not. A kit change (elytra) changes the light cones everywhere: say what that costs to store versus recompute.

Give distributions or ranges, never one average alone.

## Part 3: red-team the 100 ns rethink

The integrator proposed four ideas (S213 answer, 17 November). Attack each and give a verdict (keep, change, kill), with a concrete case:
1. **Thinking has a light cone.** `solve` returns a proven first word, a bracket on T and a queue of deadlines; each decision is thought before the bot could reach the place it concerns. Attack: can "the first word is proven" be stated for every word, not only a straight line in the cone? What about run-ups (a jump 30 blocks ahead decides the first word), goals changed again and again (S189: the deadline queue thrown away each time), elytra (the cone itself changes)? Does it open a second path beside the one loop (S161)?
2. **The answer is a field read at the bot's state.** T toward B as a field; clear terrain nearly closed form (distance over the cone's speed plus a stop family looked up by face class); joints store it as families and points. Attack its size, its invalidation, its keying.
3. **Content keys and data shaped like the read**: direct indices, one or two cache lines, integers, no branches.
4. **Binade crossings under a certificate.** Inside a binade each tick moves a fixed whole number of grid steps; at each power of two the step changes and the landing depends on the cycle's phase (about 20 dependent crossings over a million blocks). Proposal: compute each binade independently, exact unless a crossing or the touch of B lands within one grid step of a tick boundary, and walk the chain only then. Estimate how often a real goal lands in such a sliver, and whether the rounding-parity tie (the translation premise's red) breaks it.

Also say which principle or contract would change (the word-plan output contract in engine.md, the scoreboard's two measures: T end to end, CPU per bot), and whether it is a one-way door.

## Output

`jobs/hundred-ns/report.md`, plain names (a code only in backticks after its name), at most 250 lines:
1. The idiot-index table, ranked by weighted index, with the deletion for each.
2. The Fermi numbers, inputs first.
3. The verdict on each of the four ideas, with the case that decides it.
4. What to build next, in order, as layers from the bottom (S191), for the integrator to write as specs. No code.

Write the report as you go, section by section, so a stop loses nothing.
