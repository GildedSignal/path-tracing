# Job 3.3e: a planner you can call from a script

**Why and consumer.** Today the planner runs only inside an ignored test (`engine/plan/tests/flight.rs`, `the_flight_on_the_tracks`). The play driver (job 3.3g) must call it after the click, from the bot's live start state, so thinking time is inside T. Consumer: the first flight.

**Worktree** `D:\path-tracing-wt\plan-track`, branch `wt/3.3e-plan-track`.

**Read.** `engine/plan/tests/flight.rs` (`fly`, `write_script`, how a track loads and how the start state is built today; the exporter's JSONL format); `engine/plan/src/`.

**Deliverable.** A binary `engine/plan/src/bin/plan-track.rs`: `plan-track <track> --start <bits-file|settled> --mode <m> --out <file.jsonl> [--cap <s>]`. `--start settled` uses the harness's own settled start on A; a bits file holds one `u64` per line in `PlayerState::bits()` order (if `PlayerState::from_bits` does not exist on your branch, accept only `settled` and say so: job 3.2g is writing it). It loads the track exactly as the harness does, plans in the given mode, writes the same JSONL the exporter writes, and prints one line: `SENTENCE <ticks> thinking <wall ms>` or `NO SENTENCE <reason>`, exiting 0 or 3. Move the harness's track-loading and flying code into the library so the test and the binary share it (a move, not a copy; the test's output must not change). Wall time is measured from process start to file written and includes every table built after start. Test: on the track `grassjumpeasy` (or the first track the harness certifies in mode 5), the binary's JSONL is byte-identical to the harness's for the same start and mode.
**Branch.** A worktree from `wt/playback` (`1202d23`), named below. The main checkout is `C:\Users\PASCAL\Desktop\path tracing`; read by absolute path anything not on your branch.

**This job has one deliverable. Do not stop at an open choice:** take the stricter option (a refusal rather than a default, a smaller claim rather than a larger one), write the choice into your report, and finish. Stopping without the deliverable is the one outcome that wastes the run.

**Must not.** Add a tolerance, an epsilon, a retry or a nudge. Change the law's arithmetic, the planner's search, a gate or a fixture. Start a server or a client. Commit, push or merge.

**Done means.** The deliverable and its tests passing; `cargo test --offline` from `engine/` passing (fix the `walk.rs:4` doctest import if it blocks you, and say so); `jobs/1.2-footprints/verify.ps1` 12 of 12; the net lines; under 40 lines of report.