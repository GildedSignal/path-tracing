# Judge: Clean dead planner salvage (`wt/clean-dead-planner`)

**Goal:** Adversarially judge Terra 6.1's `D:\path-tracing-wt\clean-dead-planner` working tree against `map/`, `jobs/clean-dead-planner/prompt.md`, and `D:\path-tracing-wt\clean-dead-planner\jobs\clean-dead-planner\last.md` with fresh eyes.

Judge: Claude Sonnet (fresh context; cross-family judge for OpenAI Terra 6.1). Worktree under review (READ-ONLY — do not modify files there): `D:\path-tracing-wt\clean-dead-planner`, compared against `main` (`C:\Users\PASCAL\Desktop\path tracing`).

**The judge's first question:** does this work obey `map/` (`C:\Users\PASCAL\Desktop\path tracing\map\{goal,principles,architecture}.md`: one mechanism, the generator stored once, precision only where a decision can flip, the game as the judge)? A diff that matches its spec but breaks the map is a FAIL.

## What to inspect directly (read code, don't trust `last.md` alone)

1. Run `git -C "D:\path-tracing-wt\clean-dead-planner" diff main -- ':!**/r.0.*.mca'` and inspect every changed hunk in `engine/`:
   - `engine/law/src/{lib.rs, movement.rs, world.rs}`
   - `engine/world/src/snapshot.rs` and `engine/world/tests/world.rs`
   - `engine/native/src/resident.rs`
   - `engine/plan/src/{source.rs, flight.rs, lib.rs, time.rs, walk.rs, bin/plan-track.rs}` and `engine/plan/tests/{deviation.rs, hard_track.rs}`
   - `engine/words/src/{lib.rs, segment.rs}`
2. Check every clause of `C:\Users\PASCAL\Desktop\path tracing\jobs\clean-dead-planner\prompt.md`:
   - Confirm ZERO `joints/cut.rs` or greedy forward rollout exists anywhere in `wt/clean-dead-planner`.
   - Confirm deleted files (`arith.rs`, `polish.rs`, `reach.rs`, `search.rs`, `words/src/{class,gait,table}.rs`) left no broken modes, stale feature paths, or dead test workarounds.
   - Check the saturation `20.0` settled-start change carefully: does `resident.rs` / `source.rs` / `deviation.rs` keep true tick-0 bit-exact parity without relaxing assertions?
   - Check the non-local shape / `unknown` block exception path (`ObservedWorld`, `load_for_plan`, `known_block`, `try_tick_with_collision`, corridor halo widening in `resident.rs` from `2 -> 4`): verify it does NOT hide missing chunks, silent air fallbacks, or weaken corridor guarantees on uniform ground.
   - Check whether the two modified `.mca` files in `wt/clean-dead-planner` match `main`'s pre-existing dirty working fixture (identical byte-for-byte with `C:\Users\PASCAL\Desktop\path tracing\engine\world\tests\fixtures\world\...\r.0.*.mca`, i.e., `launch.ps1` `robocopy` artifact, NOT touched by Terra).
   - Check `D:\path-tracing-evidence\clean-dead-workspace.log` and `D:\path-tracing-evidence\clean-dead-real-time.log` (or rerun `cargo test --release -p plan --test real_time` if needed) to verify `<= 15 µs` flat median and zero test regressions.
   - Check `map/now.md` diff in `wt/clean-dead-planner` vs `C:\Users\PASCAL\Desktop\path tracing\map\now.md` (note `wt/clean-dead-planner` branched before `b0d9ea3e` — note cleanly that integrator should drop/reconcile `map/now.md` upon merge).

## Deliverable

Write your concise adversarial verdict to `C:\Users\PASCAL\Desktop\path tracing\jobs\clean-dead-planner-judge\report.md`, opening with `Verdict: PASS` or `Verdict: FAIL` plus blocking findings (if any) and non-blocking notes (`file:line`), then print the same verdict clearly in stdout.
