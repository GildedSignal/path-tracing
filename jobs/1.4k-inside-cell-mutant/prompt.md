# Job 1.4k: the footprint key must notice the fifth collision context

**Why.** Job 1.4e added a fifth collision context, the shape a cell presents with the feet inside it (the scaffolding bottom cap; 14 of 32,366 states differ), and keyed it in `engine/law/src/pattern.rs` (`for shape in [shape, above, below, sneak, inside_cell] {`, about line 397). Its judge found that **removing `inside_cell` from that list still passes the footprint gate 12 of 12**: no test edits a cell so that only its inside-cell answer changes. So the gate cannot see that key, and a cell that differs only there would share a pattern key with one that does not — two different laws under one key.

**Consumer.** Job 3.2d's premise check (its Step 0, fix 4: premises are checked against recorded read footprints) and every cached table keyed by pattern: a key that misses an answer the tick reads makes a premise silently wrong.

**Branch.** `D:\path-tracing-wt\inside-cell` on `wt/1.4k-inside-cell-mutant`, from `main`.

**Read.** `jobs/1.2-footprints/verify.ps1` (the gate: each `Mutant` line edits one site and requires a running test to fail); `engine/law/tests/footprints.rs`, test `edits_that_keep_a_cells_answers_cannot_alter_tick` and its neighbours (the `sneak-unkeyed` mutant is the exact precedent); `engine/law/src/pattern.rs`; `jobs/1.4e-climbable/last.md`.

**Build.**
1. A test case, beside the existing ones and in their style, with two cells whose answers are identical in the four old contexts and differ **only** in the inside-cell context (the scaffolding `bottom=true` cap against a state without it is the known pair; take both from the exported model table, never invent a shape). It must fail when `inside_cell` is dropped from the key and pass on `main` as it is. Record both runs.
2. A thirteenth `Mutant` line in `verify.ps1`, `inside-cell-unkeyed`, dropping `inside_cell` from that list, run against the test from step 1.

**Must not.** Change the key, the law, or any existing mutant or test. Add a tolerance or an `#[ignore]`. Commit, push or merge.

**Done means.** The test red with the mutant and green without (both outputs quoted); `verify.ps1` reporting **13 of 13** mutants caught; `cargo test --offline` from `engine/` passing; the net lines. Under 30 lines.
