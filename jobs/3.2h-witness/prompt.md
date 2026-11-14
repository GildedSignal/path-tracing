# Job 3.2h: a surprise's witness that replays

**Why and consumer.** Vocabulary v2 (S81): every surprise is logged with its replay as witness, because surprises are where the port is incomplete and where learning starts (learning.md, Phase 3). Today a witness (`engine/words/src/table.rs`, `Surprise`) lives only in memory, holds an interned pattern key rather than a world, and does not keep the predicted end bits. `PlayerState::from_bits` now exists on your base (job 3.2g, `19d88db`), so a witness can be written and read back.

**Worktree** `D:\path-tracing-wt\witness`, branch `wt/3.2h-witness`, from `wt/3.2g-from-bits`.

**This job has one deliverable. Do not stop at an open choice:** take the stricter option, write the choice into your report, and finish.

**Deliverable.** `Surprise` keeps the predicted end state's bits beside the actual ones, and the cells the replay read (its footprint, with each cell's contents, which the table can already produce through `RecordingWorld`) instead of only the interned key. A writer emits one JSONL line per surprise: start bits, program and arguments, predicted end bits, actual end bits, and the footprint's cells. A loader reads a line, rebuilds the world from those cells alone, replays through the law, and must reproduce the actual end bits exactly; a line whose replay differs is a refusal naming the first differing field. Test on surprises produced by the existing words tests: every written witness reloads and replays bit for bit, and one edited cell makes its witness refuse. Measure the writer's cost against the run that produces the surprises and report it; keep the 2,000-per-track cap only if writing them all costs more than one percent.

**Must not.** Change `bits()`, the law's arithmetic, a gate or a fixture. Add a tolerance. Touch `engine/plan` or `bot/`. Commit, push or merge. If `jobs/1.2-footprints/verify.ps1` is denied its log, apply the one-line `$output` change from `main` to your copy.

**Done means.** The round trip passing; the cost; `cargo test --offline` from `engine/` passing; `verify.ps1` 12 of 12; the net lines; under 40 lines of report.