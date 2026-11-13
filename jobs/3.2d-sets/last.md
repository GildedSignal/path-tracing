**Job 3.2d is incomplete.** I made the first part of Step 0: the measured `19.902886` b/s average is now used only for ordering, the general lower bound is zero, and bound-based node discards have been removed. Exact-state dominance now uses the recorded state bits, including the tick.

The existing backward field is still a field over cells. It reconstructs read scans and caches premise answers by epoch, so it does not meet the required arriving-state representation or tick-recorded footprint check. The three-answer spine, backward-state solver, timing and expansion-cap accounting, restoration proof, halo refusals, and held-out transformations remain unimplemented. The existing planner must not be treated as satisfying this spec.

I did **not** run the old flight as the requested comparison: its timing excludes reach-table construction, its cap is wall time, and its certification does not establish restoration. Consequently, there is no per-track scoreboard, mover or kill-table verdict, or held-out result. No optimality or completeness claim is supported.

Verification of the partial change:

- `cargo test --offline` passed.
- The footprint gate caught **12 of 12 mutants** and passed its restored checks.
- Net change: **18 lines added, 34 removed**. No commit, push, or merge.

The [partial run record](D:/path-tracing-evidence/3.2d-sets-partial/record.txt) links the inputs and limitations; the test logs and captured diff are beside it. The checkout was clean initially, at `wt/3.2d-sets`, commit `5f486f3`; the three planner files now contain the uncommitted partial changes.