# Job 3.2f-b: the stalled joints, one measurement, finished

**Why.** The planner gets a sentence on 8 of Julien's 18 tracks. Three — `fences`, `fencemaze`, `redfakemaze` — stall at a joint the cell field claims and no exact state reaches. The planner's rewrite (job 3.2d) waits on one fact: **is the cell the bottleneck, or is it the run-up?** Job 3.2f measured half of it and said so honestly; this job finishes exactly that half and nothing else.

**Consumer.** Job 3.2d, the planner rewrite. One fact, then the spec is written.

**Read.** `AGENTS.md`, `docs/learning.md` section 1, `docs/HANDOFF.md` ("Now", item 3), `jobs/3.2e-probes/prompt.md`, `jobs/3.2f-stalled-joints/prompt.md` and `jobs/3.2f-stalled-joints/last.md` (what it found and what it left). Its records: `D:\path-tracing-evidence\probes\joints\` — `inputs.txt`, `flight.txt`, `probe.patch`. The patch is your instrumentation; reapply it rather than writing a second one.

**Branch.** `D:\path-tracing-wt\joints` on `wt/joints`, cut from `7713f9d`.

**Measure, and only this.** 3.2f established: the stored hop at each claimed joint begins at a pre-accelerated contact, so a centre-at-rest replay does not reproduce it. For `fences` and `fencemaze`, for **every** sampled state it already collected (not a subset), report the **first departing tick** — the first tick where the replayed path leaves the stored path — and **what the body meets on that tick**: the collision face by axis, or `none` if the paths part with no collision. Then the one thing 3.2f skipped: for the single `fencemaze` sample that did arrive, run the **next joint** from the state it arrives in, and say whether the chain continues or dies, and on what. For `redfakemaze`, 3.2f showed all 13,549 forward expansions carry no anchor; do not re-run it, just carry that line forward.

**The question your final message answers, in one sentence each:**
1. Do the stalls part from the stored path on a collision, or without one?
2. Is a valid run-up reachable from the search's own states at these joints — yes, no, or not determined by this measurement?
3. Does the `fencemaze` chain continue past its one arriving sample?

**Must not.** Change the law, the planner, the search order, or any gate. Add a tolerance, an epsilon or a retry anywhere. Widen the measurement to other tracks, other joints, or a fix. Commit, push or merge. If the instrumentation cannot answer a question, say "not determined by this measurement" and say what would determine it — that is a complete answer, and a guess is not.

**Done means.** Records under `D:\path-tracing-evidence\probes\joints-b\` with the command that replays them; `cargo test --offline` from `engine/` unchanged; a final message with the three answers, a table of first-departing-tick and contact per sample, and anything left out.
