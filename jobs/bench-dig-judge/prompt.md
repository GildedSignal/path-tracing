# Judge the bench-dig diff: does it make the mechanism, or patch the benchmark?

**Goal:** a verdict per change in `D:\path-tracing-wt\bench-dig`'s uncommitted diff: KEEP, REWRITE (with the mechanism it should be) or DELETE, judged against the map's principles and the per-task numbers, not against how many tasks pass.

You are a judge. Read, run read-only commands, and write one file: `D:\path-tracing-wt\bench-dig\jobs\bench-dig-judge\verdict.md`. Do not edit code, do not commit, and do not run `git checkout`, `git stash`, `git reset` or `cargo clean` in that worktree: the diff is the only copy.

## Read first, whole

- `C:\Users\PASCAL\Desktop\path tracing\map\` — every `.md` file, `goal.md`, `principles.md`, `architecture.md`, `vocabulary.md`, `now.md`. Principles 4, 6, 11, 21, 22 and 27 are the ones this verdict turns on. Principle 22, his words: "never ever do that again no bandaids if unsure just ask me don't plow with half assed solutions they compound badly."
- `map\detail\representation.md`, the section **"Ways of moving are modes"**: it is the spec. Its "Killed if" line is the test: *a mode or class needs a block name or its own search code; a class's price differs from the law's gait there; deep water or a 6-wide shallow strip still returns `NeedsJoints`, or costs more cold thinking than a new joint pattern's floor (~12 ms).*
- `map\detail\benchmarks.md`, the BlockStep rows: what the benchmark is and why the seven tasks failed on `main`.

## What is being judged

- Branch `wt/bench-dig` at `8dfb7022` (committed: `2e6a40f3` prices field edges by the class of the cell left; `34927c46` makes swimming a zone). Read those two commit messages: `git -C D:\path-tracing-wt\bench-dig log -3`.
- The **uncommitted** diff on top of it, written by a builder who died before reading its own final result: `git -C D:\path-tracing-wt\bench-dig diff -- engine/` (`+399 / -108`; `engine/plan/src/defect.rs` carries most of it; also `cut.rs`, `solve.rs`, `source.rs`, `store.rs`, `bin/strip-probe.rs`, `law/src/codec.rs`, `mind/src/world_mirror.rs`). Ignore the two `.mca` fixture files.

## The numbers (already measured, do not rerun the 72 tasks unless a claim needs it)

All in `D:\path-tracing-evidence\blockstep-20261126\`:
- `results.txt` — `main`: 65/72 solved.
- `results-modes.txt` — `34927c46`: 62/72, with two `certify:touch-mismatch`.
- `results-bench-dig-final.txt` — the uncommitted diff: 71/72 replayed, 0 touch-mismatch; only `hills-50-sw` fails (`NeedsJoints` at its own start cell, 3,845 exceptions). The strip probes are at its top (`water1x6` … `sea5`, `powder_snow` fast-fails in 44 ms).
- `compare-main-vs-bench-dig.txt` — per task: T on main, T on the diff, thinking on each.

What the comparison already shows: 6 tasks newly solved (`desert-100-ne`, `forest-50-sw`, `forest-100-se`, `icemountain-100-se`, `swamp-50-se`, `swamp-100-se`). **On the 65 tasks both solve, T got worse on 40 and better on 20, +183 ticks in sum, worst +17 % (`forest-20-nw` 107 → 125)**, and cold thinking roughly doubles on many (`hills-20-ne` 152 → 436 ms, `ice-100-ne` 633 → 1,981 ms). A change that solves more tasks by making the solved ones slower is not progress by itself: T is the one currency, thinking included.

## Pieces that look like patches (check each; find any others)

1. `TopoField::bounds_for` (`defect.rs`): the field's box shrinks through a hand-picked ladder `(20, depth, 8) → (20, 8, 8) → (20, 4, 6) → (14, 4, 6) → (10, 3, 5) → (6, 2, 4) → (4, 2, 3)` until it fits `MAX_TOPO_CELLS = 120_000`. Is this a cap tuned until tests pass? Does a smaller field explain the T regressions above?
2. `Legs::learn`: a second cycle stop when velocity changes by `<= 1e-6` after `t >= 32`, beside the exact bit-key repeat. Is this precision thrown away where a decision can flip (principle 11), and a made-up number (27)? Why did the exact repeat not close for a swimmer?
3. `Legs::from_store` returns the hard-coded `fallback` (`h 2.81, up 6.19`) on more paths, including when either gait does not land on its floor. Is a non-walking mode silently priced as walking?
4. `has_step_support_with`: powder snow and leather boots enter step support. Law-true (`PowderSnowBlock` collides only for a boots-wearer descending), or a block name in search code (Killed if)?
5. `solve.rs`: the far-field switch moves from `horiz_dist > 26 || est_vol > 96_000` to `> 40 || > 72_000`. Tuned to the benchmark?
6. `store.rs` `Gait::learn`: a stop at `position.y > top + 2.5 && velocity.y >= 0`. Where does 2.5 come from?
7. `source.rs` `support_cell`: `(1..=3).contains(fy - floor_y)`. Where does 3 come from?
8. The `cell_flags` / `all_unit_xz` / `visit_extra_collisions` block in `TopoField::build` and `exhausted` + `cold_cap` near `defect.rs:2235`: an optimisation that keeps the wave exact, or a second path?

For each: read the code, find the law fact or need that fixes the constant (the 26.2 jar's mapped sources are under `D:\path-tracing-deps\`; the GLM and Vibe dossiers on swimming are in `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261127-0957*`), and say whether the number comes from the law or from the benchmark.

## You may run (read-only, own target dir)

```powershell
$env:CARGO_TARGET_DIR = 'D:\path-tracing-target\judge-bench-dig'
Set-Location 'D:\path-tracing-wt\bench-dig\engine'
cargo test --offline --release -p plan --test kit_unfold --test kit_field_edges --test generalize
$env:STAGES='1'; cargo run --offline --release -p plan --bin strip-probe
```

`PATH` needs `D:\rust\cargo\bin`; `RUSTUP_HOME=D:\rust\rustup`, `CARGO_HOME=D:\rust\cargo`. To test a hypothesis (for example: revert piece 1 alone and rerun the affected tasks), copy the worktree to `D:\path-tracing-run\judge-bench-dig\` and work in the copy; `blockstep.exe` takes its task list from `D:\path-tracing-evidence\blockstep-20261126\tasks.txt` (filter lines into a file of your own).

## The verdict file

`verdict.md`, in this order:
1. One paragraph: is the diff the mechanism the spec describes (modes as gaits, a cell priced by the cheapest mode the bot has there), and which part of it is?
2. A table, one row per piece above and any you found: file:line, what it does, law-sourced or benchmark-tuned (with the evidence), KEEP / REWRITE / DELETE, and for REWRITE the mechanism in one sentence.
3. The T regressions: their cause, shown on at least three tasks (raw ticks, not averages).
4. `hills-50-sw`: why it fails at its own start cell.
5. What you could not check, plainly.

Speak in his vocabulary (`map/vocabulary.md`): T, the law, word, gait, mode, zone, joint, defect, certificate, field, bracket. Raw numbers, no averages alone.
