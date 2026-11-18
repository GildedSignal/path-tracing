# Judge: Hydrogen's port build A (mining speed, boat, elytra boost, ender pearl)

You judge in a fresh context. You did not build this. Your verdict is the only check before it merges into the law, so look for what is wrong, not for what is right.

**Read:**
- the spec: `C:\Users\PASCAL\Desktop\path tracing\jobs\port-build-a\prompt.md`;
- the builder's report: `C:\Users\PASCAL\Desktop\path tracing\jobs\port-build-a\report.md`;
- the change: the worktree `D:\path-tracing-wt\port-build-a` (uncommitted, on top of `e6de417`): `engine\law\src\mining.rs`, `boat.rs`, `lib.rs`, and the new `elytra.rs` and `pearl.rs`;
- the evidence, where the test values must come from: the six folders the spec names in `C:\Users\PASCAL\Desktop\path tracing\lab\results\` (each `check.md` is the reproduced grade; the raw witness outputs are beside it).

The integrator ran `cargo test --offline -p law` on the worktree: every suite passes.

**Check, per part:**
1. **Values from evidence.** Sample at least five test values per part and find each in the raw evidence file the comment names. A value that is not there, or was retyped from prose, is a finding.
2. **The law's order of operations.** The ported functions follow the bytecode's operation order and types (`f32` versus `f64`, widening points, association) as the dossier and `check.md` state them.
3. **The tests can fail.** Each part's test fails on the mutant the evidence falsified. Read the reported mutant runs and say whether they would really turn red.
4. **The rules:** no tolerance, epsilon, clamp or retry added to pass; no block kind given its own code path by name (one mechanism: a set of blocks is read from the block table's data); no new search or geometry code (the pearl's ray must use the law's existing block traversal); `movement.rs` and `state.rs` untouched; no pinned hash or existing expected value changed.
5. **Size.** About 2,260 lines were added for four laws. Say what could be deleted without losing a tested behaviour.

**Write** `C:\Users\PASCAL\Desktop\path tracing\jobs\port-build-a-judge\last.md`: per part, PASS, PASS WITH FINDINGS or FAIL, each finding with file:line, its severity (blocking, medium, low) and the fix. Then one overall verdict line: `VERDICT: PASS`, `VERDICT: PASS WITH FINDINGS` or `VERDICT: FAIL`.
