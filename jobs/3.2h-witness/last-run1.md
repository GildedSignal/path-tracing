**Job 3.2h is incomplete.** [table.rs](D:/path-tracing-wt/witness/engine/words/src/table.rs) now retains actual end bits, the footprint, and a representative predicted end state: the first state observed for the predicted outcome.

The footprint’s cell contents, JSONL writer and loader, replay refusal test, and writer cost measurement remain undone. The existing 2,000-witness cap has not been evaluated or changed.

The workspace suite passed through the verification script: **151 passed, 11 ignored**. The footprint gate caught **12 of 12 mutants** and passed its restoration and randomness checks. Its original log path was denied; [verify.ps1](D:/path-tracing-wt/witness/jobs/1.2-footprints/verify.ps1) now uses a fresh writable output directory.

Net change: **21 insertions, 10 deletions** across two files. No commit, push, or merge was made.