# Job 3.2b: the planner (standard pathfinding over words, offline on his 18 tracks)

**Why.** The unified idea is approved (12 November, S93; `docs/learning.md`). Words exist in the code (`engine/words`, jobs 3.2a and 3.2c) but nothing chooses among them: no bot can yet go from A to B. The first flight needs a planner, and the flight is the test of the whole idea: the jumps where no sentence arrives, or where Julien's T beats ours, are the only demand the learning loop will work on. "we need to leverage proper path finding for short and long distances we're not remaking the wheel here" (S92).

**Consumer.** The executor and the first flight (HANDOFF roadmap, item 3); `docs/learning.md` section 6, kill test 1.

**Read.** `docs/PRINCIPLES.md` (goal, philosophy, From Emergent Space), `docs/learning.md` (all), `docs/vocabulary.md` (all; sections 3, 7, 9, 10 and 11 bind this job), `docs/representation.md` sections 2, 3 and 5, `docs/scales.md` sections 2, 4.3 and 4.4, `docs/footprints.md`, Julien's turns S83–S94 in `docs/reference/julien-2026-11-10.md`. Code: `engine/words` (all), `engine/world` (snapshots, the track fixtures and `tracks.txt`), `engine/law/src/movement.rs` (`travel_air`). Evidence: `lab/wiki/arc-certificate.md`, `gait-modulo.md`, `idiot-index.md`, `law-change-rebuild.md`, and HANDOFF "Start here" item 2 (what job 3.2c measured).

**Branch.** `wt/planner`, worktree `D:\path-tracing-wt\planner`, cut from `main`. A new crate `engine/plan`; changes to `engine/words` only where a step below names them.

## Build, in this order

1. **The table is looked up by class alone.** `ProposalTable::proposals` stops matching the world part at lookup (3.2c: that matching cost 18 to 20 replays per lookup, and nothing used its order). Each proposal keeps its footprint for one purpose: saying when its certificate is broken. Invention runs each distinct input once (54 programs give 24 pieces today).
2. **The law refuses instead of panicking**, where the planner would meet it: a segment that reaches an unported rule returns a named refusal. Remove the `catch_unwind` in `pieces`. This is the one change allowed in `engine/law`: the arithmetic does not change and no rule is added.
3. **The search: A\* over exact states.** A node is a concrete state. An edge is a word: a proposal of the node's class, certified by one replay. The cost is ticks. The closed set and every pruning use exact states or a proof of dominance under the law, never the class (vocabulary.md section 10). The heuristic is a lower bound nothing can beat: the distance to the goal over the terrain speed limit c (HANDOFF anomaly 3), stated with its premises. Where no proposal exists for a class, invent there (a miss) and count it. The goal is a region (stand on B), not a point.
4. **Gaits and arcs by arithmetic.** On uniform ground the search advances by a certified gait cycle and floor arithmetic, not a replay per piece (`gait-modulo`); a free arc is one edge whose crossings and landing tick are computed (`arc-certificate`), with its tube's cells checked against the world. One replay certifies the finished sentence end to end. If a case needs more than the two lanes proved, write it down: it is a finding.
5. **Sets backward, points forward** (S91, S94). From the goal, each tool's reach table mirrored about the goal gives the set of takeoff states, and its run-up says how far behind to look. Carry sets back through the chain; choose the point inside each set on the forward pass; then slide each joint inside its set and replay, tightest joints first, keeping a slide only when the sentence's T falls. Build this as a second way to produce a sentence, beside step 3, and measure both. The mirror is proven for free arcs inside one binade only: where it fails (a wall, a ceiling, an edge), report the case; do not special-case it.
6. **Four dimensions.** Every cell of a sentence's tube carries its tick. The world interface the planner reads takes a tick, even though his tracks are static, and one test uses a platform that exists only on a schedule.
7. **One objective.** The planner reports, per sentence, its T in ticks and its own thinking time, and the sum. It can be stopped at any time and returns the best certified sentence so far with the bracket it has (anytime): a lower bound from step 3's heuristic, an upper bound from the sentence.
8. **The flight report.** For each track: the sentence, its T, the thinking time, the bracket, and **every place where no sentence arrives**, with the state, the goal and what was tried. That list is this job's product as much as the sentences are.

## Measure

On his 18 tracks, from A to B: sentences found; T against Julien's time where `tracks.txt` or the fixtures record one (say where none is recorded); thinking time (n, min, p50, p90, p99, max) for step 3 alone, step 5 alone, and both; nodes expanded, replays, misses and their price; the share of ticks covered by arithmetic against by replay; the cost per certified tick against 3.2c's numbers; proposals replayed per decision; the failures of step 8. The machine's load, and every anomaly.

## Constraints

- No new search idea. A\* as in Hart, Nilsson and Raphael; if you take anything else (incremental replanning, jump-point skipping, interval search), name its source and say whether you read it or recall it.
- No tolerance, no special case per block, no catalogue of jumps or techniques, no number of ours. A capability that needs its own search or geometry is the representation being wrong: stop and describe it.
- The jar suite stays 6 of 6; `jobs/1.2-footprints/verify.ps1` still catches 12 of 12 mutants; `jobs/3.2a-segments-and-gaits/verify.ps1` passes or its changes are explained; the words tests pass, the pinned 27 of 40 included or its change explained.
- Every sentence the planner returns is certified by a replay from the exact start state. A sentence that is not certified is not returned.

## Must not

Touch `bot/`, the gates or `docs/`. Add or change a rule of the law. Build a learned ordering, a puzzle generator, compound words or the far field (`docs/learning.md` section 3: parked). Run anything live. Push or merge.

## Done means

A final message with what you built per step; the measurements above; the failure list of step 8 in full; where step 5's mirror failed and why; what you did not build and why; every anomaly; the commits on `wt/planner`, each with `Built-by:` and, where lines grow, `Bought:`.
