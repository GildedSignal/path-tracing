**Stopped at step 1:** exact-state playback is not established. The client snapshot omits fields in `PlayerState::bits()`, including damage cooldown and movement history; food exhaustion and timer observations are client-side rather than authoritative server values. The current recorder also teleports and settles the bot. A complete comparison needs a way to capture the required state at the same playback boundary.

- **Exact start-state check:** not built.
- **Exporter:** the existing partial exporter remains in [flight.rs](/D:/path-tracing-wt/playback/engine/plan/tests/flight.rs); it is not finished or verified across tracks. No track export results are available.
- **`play <track>` and deviation report:** not built. There is no command to play a certified track and no exact-state mismatch refusal yet.

`cargo test --offline` from `engine/` **failed** in the `plan` doctest: [walk.rs](/D:/path-tracing-wt/playback/engine/plan/src/walk.rs:4) imports `law::Refusal`, which is unavailable to rustdoc. The preceding non-ignored tests passed; recorder fixture replay and the flight measurement were ignored.

I made no source changes. The existing uncommitted exporter remains at **+30 / −0 lines**; net lines added during this turn: **0**. The requested job prompt and prior report were absent from this worktree.