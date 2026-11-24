# Yarn-ball fold: delete-whole generator compression onto basis + rows

**Goal:** Untangle `engine/law/src/` on `wt/port-fold` by replacing whole entangled Mojang-shaped modules with table-driven `basis/` generators (`body`, `clock`, `cell`, `sweep`, `ray`) and unified `ledger` built from GLM's audited row packs, unpinning `tests/jar.rs` onto public faces, and deleting every replaced silo (`>= 1,500` net code lines deleted / `>= 0.30` decades across targeted groups) with zero bit or speed regressions.

Builder: Claude Opus. Worktree: `D:\path-tracing-wt\port-fold`, branch `wt/port-fold` (currently at `43f43730`). Commit each clean verified slice directly in `D:\path-tracing-wt\port-fold` (never push, never edit `main` or other worktrees, never touch the live game/server/client).

## Operating Philosophy (no nibbling in place)

Read these on the main checkout first:
- `C:\Users\PASCAL\Desktop\path tracing\map\goal.md`
- `C:\Users\PASCAL\Desktop\path tracing\map\principles.md`
- `C:\Users\PASCAL\Desktop\path tracing\map\architecture.md`
- `C:\Users\PASCAL\Desktop\path tracing\map\detail\engine.md`
- `C:\Users\PASCAL\Desktop\path tracing\jobs\law-grouping-audit\table.md`
- `C:\Users\PASCAL\Desktop\path tracing\jobs\hot-core-rebuild\report.md` (how `0.36 decades` was actually achieved)
- `C:\Users\PASCAL\Desktop\path tracing\jobs\law-basis-3\report.md` (the trap to avoid: `0.025 decades`, zero files deleted)

**Non-negotiable rules learned from `port-compress`, `law-basis-3`, and `hot-core-rebuild`:**
1. **Treat old modules as the frozen differential oracle — never migrate in place, never add wrapper/bridge layers.** Build the compact table-driven replacement beside the old module, run a differential suite (`core` / `basis` + rows vs. old module on extreme + recording inputs) until `0` anomalies, switch callers in one cut, and **delete the old files whole** in the same commit.
2. **Cut the yarn-ball anchor FIRST (`tests/jar.rs` + internal helper paths):** Per `law-basis-3` and `hot-core-rebuild`, ~60 helper paths in `tests/jar.rs` call internal per-class functions and pin legacy silos in place. Re-route `tests/jar.rs` assertions onto the public faces (`law::core`, `law::face::advance`, `basis::{body, clock, cell, sweep, ray}`, `WorldMut`) **without modifying a single TSV witness row or expected bit**.
3. **One binary rule for deletion:** A commit that adds helper wrappers, dual representations, or shrinks targeted groups by only tens of lines is a **FAIL** (`growth or churn, not progress`). Either replace a whole family + delete its silo, or leave frozen cold kernels (`sim.rs` world-sim kernels, pistons, fluids, portals, elytra, rails, wire graph) untouched until a flight needs them — do not burn turns half-nibbling `sim.rs`.
4. **Tests follow `AGENTS.md`:** keep what the game and jar judge (`cargo test -p law --test jar`, `tests/replay.rs`, `real_time.rs`, differential gates). Delete redundant in-file per-class Rust unit tests once their family is covered by `tests/jar.rs` + differential checks.

## Audited GLM Data Packs (on `C:\Users\PASCAL\Desktop\path tracing\lab\results\`)

1. **Affine body & kick regimes** (`29` rows / `15` witnessed bodies, `45/45` exact):
   `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261123-215021-18424-1-0-gen-affine-body-rows\{dossier.md, rows.tsv}`
2. **Player & server resource clocks** (`41` clock rows / `21` active clocks + `order.tsv`, `36,400` ticks, `0` mismatches):
   `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261123-222540-17108-1-0-gen-clock-rows\{dossier.md, rows.tsv, order.tsv, src/GenericClock.java}`
3. **Ground classes & motion scars** (`154` classes, `17,040` ticks, `0` mismatches):
   `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261123-211113-3024-1-0-gen-ground-rows\{dossier.md, rows.tsv, stuck.tsv}`
4. **Step/fence/stair witnesses** (`8,280` `LocalPlayer` runs / `331,200` ticks, `0` deviations):
   `C:\Users\PASCAL\Desktop\path tracing\lab\results\local-20261123-211111-452-1-0-gen-step-fence-stair-witnesses\dossier.md`

## Ordered Execution Slices (each commit tested green before the next)

1. **Slice 1 — Unpin `tests/jar.rs` & cut `affine_body` + kicks (`table.md` groups 2 & 12):**
   - Fold throwables (`snowball`, `egg`, `ender pearl`), `arrow`, `wind_charge`, `falling_block`, `TNT`, `experience_orb`, off-rail `minecart`, and kicks (`v' = a*v + b*dir + cap`) onto `basis::body` + row tables (preserving exact jar operation order: arrow `move -> drag -> gravity`, minecart pre-move clamp).
   - Delete replaced per-class tick loops & kick duplicates across `projectiles.rs`, `pearl.rs`, `falling.rs`, `minecart.rs`, `explosion.rs`, `combat.rs`, and redundant in-file witnesses superseded by `tests/jar.rs`.
2. **Slice 2 — Fold `clock` + unified `ledger` (`table.md` groups 7 & 18):**
   - Implement unified `clock` + `ledger` (single mitigation/absorption/exhaustion/food pipeline + `order.tsv` phase ordering) covering `breath.rs`, `food_tick`, `fire`, `frozen`, effect timers, and invulnerability cooldown against `gen-clock-rows`.
   - Delete `breath.rs` and duplicate clock/hurt/food code paths across `player.rs`, `core.rs`, `effects.rs`, `combat.rs`, `boat.rs`.
3. **Slice 3 — Ground parameter/scar rows (`gen-ground-rows`) + clean jar harness through `law::face::advance`:**
   - Replace scattered hard-coded ground friction/speed/jump/scar branches (`block_hooks.rs`, `world_hooks.rs`, `world.rs`) with `gen-ground-rows` parameter/scar rows where whole switches delete cleanly.
   - Route the applicable jar witnesses cleanly through `law::core` and `law::face::advance` (preserving all `PlayerState` wire compatibility for `plan`).

## Done means (hard gates — no shipping without all of these)

1. **Quantitative deletion & decades check** (measured in `D:\path-tracing-wt\port-fold` via `perl scripts/law-lines.pl` and `bash scripts/engine-lines.sh` against `43f43730`):
   - **Net `engine/law/src/` code lines deleted >= `1,500` lines** (`>= 0.30` decades reduction on the targeted groups vs. `table.md`, with whole silos deleted and zero wrapper/dual-path bloat). If a pass stalls under that bar, stop cleanly with `STATUS: BLOCKED: <exact pin or blocker>` rather than shipping micro-trims.
2. **All gates pass in `D:\path-tracing-wt\port-fold` (`CARGO_TARGET_DIR = D:\path-tracing-target\job-yarn-ball-fold`):**
   - `cargo test --workspace` (every lib test passes, `0` failures).
   - `cargo test -p law --test jar` (all `9,117+` jar witnesses pass with `0` modified witness rows and `0` failures).
   - `cargo test -p law --test replay` (all gated recordings + `event_face` judge pass with `0` anomalies).
   - `cargo test --release -p plan --test real_time` passes (`0` regressions).
   - `bash scripts/bench-judge.sh c392524 12` AND `bash scripts/bench-judge.sh 43f43730 12` show `candidate/base <= 1.005` across walk, sprint-jump, ceiling, and bumpy.
3. Write exact before/after per-file line tables, decades per `table.md` group, exact test counts, and raw `bench-judge.sh` medians/ratios to `jobs/yarn-ball-fold/report.md` (and `C:\Users\PASCAL\Desktop\path tracing\jobs\yarn-ball-fold\report.md` if accessible, or note commit SHA cleanly in `last.md`), ending `last.md` with `STATUS: DONE` or `STATUS: BLOCKED: <reason>`.
