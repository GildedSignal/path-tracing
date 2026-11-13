# Job 3.3a-b: `play <track>`, so Julien can watch a certified sentence

**Why.** Nothing has ever been played in the game. The planner certifies sentences offline; Julien cannot see one. Every phase ends in a flight he can use (AGENTS, "Pace"). This is the smallest thing that is a flight.

**Consumer.** The first flight (HANDOFF roadmap item 2) and every deviation measurement after it: the deviation stream needs a run in the game to deviate from.

**Branch.** `D:\path-tracing-wt\playback` on `wt/playback`, where your earlier run left an uncommitted partial exporter in `engine/plan/tests/flight.rs`. Keep it; build on it.

**Read.** `AGENTS.md`, `docs/engine.md`, `docs/footprints.md`, `jobs/3.3a-playback/prompt.md` and `jobs/3.3a-playback/last.md` (your own report: the JSONL format you chose, and the four things you listed as not built). The recorder's input format and its start check, in `bot/`.

**Build, in this order, stopping at the first one you cannot finish:**
1. **Exact start state.** Your report says the recorder's start check does not verify the complete law state, so exact-state playback is not established. Establish it: the check compares the complete start-state bit vector, not a subset. Until this holds, a replayed sentence proves nothing, so it comes first.
2. **The exporter, finished.** `PLAN_RECORD` writes `<track>-<mode>.jsonl` for every sentence the flight harness certifies, for all 18 tracks, with the final state checked against the certificate before the file is written.
3. **`play <track>`.** The command feeds that JSONL to the bot and runs it in the game. It refuses, loudly, if the live start state does not match the file's start state bit for bit — a mismatch is a refusal, never a nudge toward the recorded state.
4. **The deviation report.** Per run: the first tick where the live state leaves the predicted state, what differed, and the count of corrections. Write it beside the run record.

**Must not.** Correct, nudge, re-anchor or retry the bot toward the plan — a deviation is the measurement, and hiding it destroys the only thing this job produces. Add a tolerance or an epsilon to make a replay match. Start a server or a client yourself: a live run needs a quiet machine and the rig lock, and Julien runs it. Touch the law, the planner or any gate. Commit, push or merge.

**Done means.** A final message naming which of the four steps are built and which are not, the exact command Julien types to play a track, what the refusal looks like when the start state differs, the tracks that export a sentence, `cargo test --offline` from `engine/` passing, and the net lines.
