# Clean dead planner salvage from Astra checkpoint

**Goal:** Delete the dead A\* planner (`flight.rs` modes `3`/`5`/`b` and orphaned modules) on top of `main` and salvage only settled saturation `20.0` plus non-local-shape exceptions from `c74c4a5c`, without importing `joints/cut.rs` or any `.mca` files.

Builder: Terra 6.1. Worktree: `D:\path-tracing-wt\clean-dead-planner`, branch `wt/clean-dead-planner`, branched from `main` (`b0d9ea3e`). Reference commit for the deletion list and clean fixes only: `c74c4a5c` (on `wt/flight-ladder`). You never run `git push`, never modify `D:\path-tracing-wt\flight-ladder`, never run the live game/server/client, and never modify or stage `.mca` / `.bak-20261114` region files (`engine/world/tests/fixtures/world/`). Copy git-ignored fixtures only if missing in your worktree.

## Decisions already made (do not re-open)

1. **Do NOT salvage `engine/plan/src/joints/cut.rs` or greedy `DemandAt::transfer`.** The real backward state-set wave is designed separately before Astra builds it. On `wt/clean-dead-planner`, `solve.rs` continues to emit `PlanOutcome::NeedsJoints` cleanly at corridor exceptions using existing `main` routing.
2. **Do NOT touch any `.mca` region files.** Leave `engine/world/tests/fixtures/world/` untouched in git.
3. **Delete dead A\* planner code cleanly (`main` only needs `--mode solve`):**
   - Inspect what only `flight.rs` modes `3`, `5`, and `b` reach (`search.rs`, `polish.rs`, `reach.rs`, `arith.rs`, `words/src/{class.rs, gait.rs, segment.rs, table.rs}`, and any unused items in `families.rs` / `faces.rs`; **keep** `family_wave.rs` or anything still called by live `solve.rs` / `joints.rs` on `main`), delete every orphaned module and legacy A\* test (`hard_track.rs` legacy cases), and slim `plan-track` to `--mode solve` only. Compare against `git show --stat c74c4a5c` (`:!**/*.mca` and `:!engine/plan/src/joints/cut.rs`) as your checklist, keeping `cargo test` green across the workspace.
4. **Salvage settled saturation `20.0`:**
   - Carry the `saturation = 20.0` settled start fix (`c74c4a5c` changes in `source.rs` / `resident.rs` / `solve.rs`) so `plan` matches the live bot at tick `0`, keeping `tests/deviation.rs` passing.
5. **Salvage non-local / unmodeled shape exception handling:**
   - Carry `law::try_tick_with_collision` (`engine/law/src/{lib.rs, movement.rs}`), `World::known_block` (`engine/law/src/world.rs`), and `WorldSnapshot::load_for_plan` (`engine/world/src/snapshot.rs` tracking `unknown` cells instead of panicking on unmodeled non-local shapes like stairs), turning them into corridor `Exception` / `Unknown` outcomes cleanly rather than panicking.

## Done means

1. `git diff --name-only main` contains **zero** `.mca` files and **zero** `joints/cut.rs` greedy rollout code.
2. Dead A\* planner modules and legacy modes are removed; `plan-track` serves `--mode solve` only; `saturation = 20.0` and non-local shape corridor exception handling pass.
3. Workspace builds and tests pass:
   - `cargo test --workspace` (from `engine/` with `CARGO_TARGET_DIR = D:\path-tracing-target\job-clean-dead-planner`) passes with zero failures (note: ignore-gated replay tests if `lab/fixtures` is empty in worktree, and explicitly report which suite tests ran).
   - `cargo test --release -p plan --test real_time` passes with flat-ground thinking median `<= 15 µs` and `0` refused.
   - Add or verify a targeted test proving a non-local shape in the corridor returns a corridor exception (`NeedsJoints` or `Unknown`) instead of panicking, and `saturation == 20.0` on settled start.
4. Print `git diff --stat main -- engine/ bot/` and confirm net lines across `engine/` + `bot/` are strongly negative (`~-2,000` lines or better).

Write your concise report to `jobs/clean-dead-planner/last.md` (with files touched, commands run, exact `real_time.rs` percentiles, and `git diff --stat` net lines), and end with `STATUS: DONE` or `STATUS: BLOCKED: <reason>`.
