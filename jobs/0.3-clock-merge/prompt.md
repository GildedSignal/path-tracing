# Job 0.3: bring the 100× clock into the engine line

**Why.** Phase 0 (HANDOFF): one line of truth. The Sonnet judge passed `wt/clock` (388a1bc) as MERGE WITH FIXES. The integrator has started `git merge wt/clock` in this worktree (`wt/engine`) and left the conflicts for you.

**Resolve the conflicts, keeping both features working.**
- `bot/pathtracing.mixins.json`: take wt/engine's list and put `"MinecraftClockMixin"` first. Keep `defaultRequire` 1. Check that `LandingMoveMixin` doesn't inject into `Minecraft.runTick`.
- `bot/src/pathtracing/Recorder.java`:
  - Keep one copy of each hunk both sides share (awaitStart, collisionSteps, continueReplay, the replay wait, the settle condition, the gameTime/collisionSteps/speedModifiers/ticksFrozen fields, collisionStep).
  - Imports: the union.
  - `finish()` keeps both `ClockAudit.finish(...)` and `LandingClientTrace.replayEnd(...)`.
  - In the replay start block, `if (tick == 0) ClockAudit.begin(name);` goes first, before `apply`.
  - Keep every engine-only addition (SprintDiagnostic, LandingClientTrace, the start* fields).
- `Bot.java`, `PlayerSnapshot.java`, `TraceJson.java`, `WorldExporter.java`, `scripts/common.ps1`, `scripts/server-state.ps1`: one copy of each shared hunk, both sides' unique hunks.

**The fixes.**
1. A runtime guard: a recording started while the client follows a tick rate above the verified 2000 TPS ceiling refuses, or marks itself unqualified in its header, and says so in chat. Add an offline test that fails without the guard.
2. Rebuild (`bot/build.ps1`), then run `bot/tests/test-clock.ps1`, `bot/tests/test-full-tick-demand.ps1`, `bot/tests/test-tracks.ps1` (the mixin audit; update its injection count only if you can account for every added injection), `cargo test --offline` from `engine/`, and the `lab/gates/tests` self-tests.
3. Don't run the live full-demand gate yourself. The integrator runs it on the merged HEAD under the rig lock.

**Must not.** Change law or movement behaviour. Weaken or delete a check. Commit: leave the resolved files in the worktree.

**Done means.** Your final message lists each conflict and how you resolved it, the guard and its test, and every suite with its result.
