# Job 1.3c: L0's frozen counter, the controls the judge found missing

**Why.** Job 1.3b (commit `b70127b` on this branch) records the server's frozen-counter packets and replays them. Its judge passed it with findings (`C:\Users\PASCAL\Desktop\path tracing\jobs\1.3b-frozen-counter\prompt.md` is the spec): only a wrong value is planted, never a wrong phase, and no test runs a `freezeSync` row through the real `replay_text`.

**Consumer.** L0's live gate (docs/ladder.md: a test that cannot fail is not evidence).

**Branch.** You are in `D:\path-tracing-wt\l0-veto` on `wt/l0-veto`. Read `engine/law/tests/replay.rs` (`freeze_events`, `freeze_fixture`, `replay_text`) and `oracles/1.6f-landing-motion/verify.rs`.

**Build.** Tests only.
1. A planted phase: the tick 111 event moved from `post` to `pre` must be refused (it gives 5 against a recorded 4 at END); the same event moved to `between` must be refused at START; the tick 112 `between` event moved to `pre` must be refused at START.
2. One test that feeds a small fixture text containing `freezeSync` rows through the real `replay_text` (the row parser and the `startTicksFrozen` check included): it passes as recorded and refuses with one value edited.
3. An old fixture without the `freezeSync` field still replays as before.

**Must not.** Change the recorder, the replay's behaviour, the law, L0's pass line, mutants or comparison fields. Add a tolerance. Run anything live. Commit, push or merge.

**Done means.** `cargo test --offline` from `engine/` passes with the new tests named in the final message, each with what it refuses.
