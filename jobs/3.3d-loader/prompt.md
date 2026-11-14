# Job 3.3d: the start-state loader

**Why and consumer.** The first flight plans from the bot's live state on A, so the planner's start must be the server's and client's state, bit for bit, or a refusal naming the field that is not established. Consumer: `scripts/play-track.ps1` (job 3.3g) and the deviation report.

**Worktree** `D:\path-tracing-wt\loader`, branch `wt/3.3d-loader`.

**Read.** `jobs/3.3c-start-and-plan/prompt.md` (its Continuation table is your specification, field by field) and `jobs/3.3c-start-and-plan/last.md` (the classification as built so far; `Player.hurtServer` returns at offsets 108-115 on a zero amount, so a zero-amount hit cannot reset the counter); `engine/law/tests/replay.rs` `load_state` (the precedent; leave it untouched); `bot/src/pathtracing/PlayerSnapshot.java` (the client fields) and `scripts/server-state.ps1` (the server sample, now with Fire, FallFlying, RootVehicle, active_effects, abilities).

**Deliverable.** A library function, in `engine/plan/src/start.rs` (new), `start_state(client: &str, first: &str, second: &str, ticks_between: u32) -> Result<PlayerState, Refusal>`: the settled client snapshot, two server samples, and the ticks between them. Every field of `PlayerState::bits()` is set by exactly one rule of the Continuation table, written as a comment beside it. A sample pair whose resources differ in any bit, `ticks_between < 20`, a non-survival game mode, a present vehicle, effect, flight or glide, or a missing tag is a `Refusal` naming the field. Tests: on real inputs from `D:\path-tracing-evidence\l0-veto-20261111` (a settled snapshot and its server samples; find them), the loaded state equals the one `replay.rs` builds for the same recording on every field the old loader sets; one test per refusal kind, each from a real input with one value edited.
**Branch.** A worktree from `wt/playback` (`1202d23`), named below. The main checkout is `C:\Users\PASCAL\Desktop\path tracing`; read by absolute path anything not on your branch.

**This job has one deliverable. Do not stop at an open choice:** take the stricter option (a refusal rather than a default, a smaller claim rather than a larger one), write the choice into your report, and finish. Stopping without the deliverable is the one outcome that wastes the run.

**Must not.** Add a tolerance, an epsilon, a retry or a nudge. Change the law's arithmetic, the planner's search, a gate or a fixture. Start a server or a client. Commit, push or merge.

**Done means.** The deliverable and its tests passing; `cargo test --offline` from `engine/` passing (fix the `walk.rs:4` doctest import if it blocks you, and say so); `jobs/1.2-footprints/verify.ps1` 12 of 12; the net lines; under 40 lines of report.

---

## Continuation (your first run wrote no code)

1. **The gate's log was a shared file** that parallel jobs locked. On your branch, change the one line in `jobs/1.2-footprints/verify.ps1` that sets `$output` to `$output = Join-Path 'D:\path-tracing-deps\footprints-verification' (Split-Path -Leaf $root)` (exactly as on `main` now). Nothing else in the gate changes.
2. **Write the loader first; tests second. A missing real input never blocks the code.**
3. **Test inputs:** the archived snapshots predate `isUsingItem`, and no saved `server-state.ps1` pairs exist yet. So build each test's inputs from a real archived snapshot (name the file) with the new fields added at their settled values, and a server-sample pair built from that same recording's recorded server resources (name the fields you took and from where). Say in the report that the first live play replaces these with real pairs. One edited copy per refusal kind, as before.
4. Delete the untracked `target-loader/` your first run left.