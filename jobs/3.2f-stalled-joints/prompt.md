# Job 3.2f: why three tracks stall at one joint (a measurement, nothing merges)

**Why.** On `fences`, `fencemaze` and `redfakemaze` the backward field over cells says a hop arrives, and the forward search over exact states never makes it: all its expansions go to one joint. Job 3.2e removed the two skipped tube checks for walked hops (they are removed in this worktree) and the tracks still stall. The next planner spec depends on the cause: is the field's claim wrong because it stands at cell centres and takes off from rest, or for another reason?

**Consumer.** The rewrite of `jobs/3.2d-sets/prompt.md` (the integrator).

**Branch.** You are in `D:\path-tracing-wt\probes` on `wt/probes`. Set `CARGO_TARGET_DIR=D:/path-tracing-target/probes`. Read `C:\Users\PASCAL\Desktop\path tracing\jobs\3.2e-probes\last.md` (how the flight was run, where the records are), `engine/plan/src/reach.rs`, `src/search.rs`, `tests/flight.rs`.

**Measure, for each of the three tracks (mode 5, cap 1).**
1. The stalled joint: the set the search was anchored on when it stopped, the next set its hop claims (`via`), and the hop (tool, aim, ticks, run-up contacts). Print the blocks around both cells with their collision boxes.
2. From the cell's centre, at rest, as the field assumed: replay the hop's program at its aim through the law in the real world. Does it arrive on the next set? If not, the first tick it departs from the reach table's path, and what it meets (the cell, the block, the box, which face).
3. From the exact states the forward search actually had on that set (take up to 20, spread over the expansions): the same replay. For each: its position inside the cell, its velocity, whether it arrives, and if not the first departing tick and what it meets.
4. For the arriving states, if any: why the search did not go on from them (the next joint's claim, the same analysis one level further).

**Must not.** Change the planner's behaviour beyond the instrumentation (keep it in a patch file beside the records). Add a tolerance or a special case. Touch the law, `engine/words`, `bot/`, `docs/`. Run anything live. Commit, push or merge.

**Done means.** A final message with one table per track (the rows of steps 2 and 3), one sentence per track naming the cause as the measurements show it, and what they do not show. Records under `D:/path-tracing-evidence/probes/joints/`.
