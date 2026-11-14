# Job 3.2g: a law state that reads back from its own bits

**Why.** `PlayerState::bits()` (`engine/law/src/state.rs:202`) is the law's identity of a state: exact-state dominance, footprints, the start-state check and every witness use it. But nothing reads it back. Job 3.2c left this open: a surprise's witness (`engine/words/src/table.rs:53`, `Surprise`) holds a `PlayerState` in memory, only 2,000 per track are written, and not in a form that replays. Vocabulary v2 (S81) makes every surprise a witness with its replay, so a witness that cannot be reloaded is not one.

**Consumer.** The witness log (vocabulary.md, the surprise counters; learning.md, Phase 3 learns from failures) and job 3.3c's start-state files and deviation report, which compare states field by field.

**Branch.** `D:\path-tracing-wt\from-bits` on `wt/3.2g-from-bits`, from `main`.

**Read.** `engine/law/src/state.rs` (the struct and `bits()`), `engine/words/src/table.rs` (the witness), `docs/footprints.md`.

**Build.**
1. `PlayerState::from_bits(&[u64]) -> Result<PlayerState, Refusal>`, the exact inverse of `bits()`: `from_bits(&s.bits()) == s` field for field, and `from_bits(b).bits() == b` for every `b` that `bits()` can produce. A vector of the wrong length, or a value no state encodes (an out-of-range enum tag, say), is a refusal naming the position, never a default. Floats travel as their raw bits, NaN payloads included.
2. Tests: the round trip on every state the law's existing tests and archived fixtures produce (collect them; do not invent a generator of ours), plus states with each optional field set and unset; one refusal test per refusal kind.
3. The witness, written so it replays: each surprise as one JSONL line with the start state's bits, the program and arguments, the predicted and actual end bits, and the world it happened on (its pattern key or the snapshot it came from, whichever the table already holds; say which). A loader that reads a line back and replays it through the law to the recorded actual end bit for bit, with a test on witnesses produced by the existing words tests. Drop the 2,000-per-track cap only if writing them all costs under one percent of the run that produces them; measure and say.

**Must not.** Change `bits()`, the order of its fields, or the law's arithmetic. Add a tolerance. Touch `engine/plan`, `bot/`, the gates or the fixtures. Commit, push or merge.

**Done means.** The round trip and refusal tests passing; a reloaded witness replaying bit for bit; the cost measurement; `cargo test --offline` from `engine/` passing; `jobs/1.2-footprints/verify.ps1` 13 of 13 mutants if run on `main` after 1.4k merges, or 12 of 12 on this branch; the net lines.
