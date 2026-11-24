# Port: canonical 16³ StateId world spine and sim.rs deduplication

**Goal:** replace lossy ModelId world snapshots and sim.rs's WorldOverlay/SimCell duplication with one canonical 2-byte StateId 16³ paletted section + u64 4×4×4 mask + reversible WorldDelta spine, proving zero behavioral change across all 9,117 jar witnesses, replays, and bench.

**Worktree & build rules:**
- Work ONLY in `D:\path-tracing-wt\port-state-spine` (branch `wt/port-state-spine`).
- Set `$env:CARGO_TARGET_DIR='D:\path-tracing-target\port-state-spine'`, `$env:RUSTUP_HOME='D:\rust\rustup'`, `$env:CARGO_HOME='D:\rust\cargo'`, `$env:PATHTRACING_DEPS='D:\path-tracing-deps\data'`.
- Write your final report to `D:\path-tracing-wt\port-state-spine\jobs\port-state-spine\report.md` (and update progress there as you go).
- Never run `git commit` or `git push`. Never open `server/` or touch Julien's world.

## Read first
1. `map/principles.md`, `map/architecture.md`, `map/detail/engine.md`.
2. `jobs/kit-edit-ledger-redteam/report.md` (especially Pillar 1, C1, C2, C6).
3. `jobs/law-grouping-audit/table.md` and `jobs/hot-state-event-face/report.md`.
4. `engine/world/src/snapshot.rs`, `engine/law/src/basis/cell.rs`, `engine/law/src/sim.rs`, `engine/law/src/world.rs`, `engine/law/src/face.rs`.

## What to build (full S237 agency — decide, build, verify)
1. **Canonical 2-byte `StateId(u16)` + `BlockId(u16)` + `ModelId(u16)`** across `engine/world` and `engine/law`:
   - Every cell holds canonical Mojang 26.2 state identity (or sidecar bit for dynamic/block-entity states) instead of erasing `stone == dirt == planks == obsidian` into `ModelId`.
   - Keep `World::model_id(pos)` and `World::state_id(pos)` working everywhere — NEVER return `None` on an edited cell!
   - Expose static O(1) lookup tables generated/derived from the census/cell tables (`STATE_TO_MODEL`, `STATE_TO_BLOCK`, rules).
2. **`16³` paletted section + `u64` `4×4×4` summary mask + `O(1)` uniform-box query:**
   - Replace/back dense or ad-hoc world storage with `16³` sections `(y << 8) | (z << 4) | x` supporting uniform sections (0-bit payload) and compact paletted storage + a 64-bit `4×4×4` subcell occupancy/non-uniformity mask.
   - Add an `O(1)` uniform-box query on `World` (or `WorldSection` / `WorldSnapshot`) so `law::face::advance` (and future corridor checks) can test whether an AABB corridor box is uniform without per-cell loops.
3. **One reversible `WorldDelta` / `WorldMut` interface & deduplicate `sim.rs` (net-negative LOC!):**
   - Replace `SimWorld`'s duplicated `WorldOverlay` (`HashMap<BlockPos, Block>`) + `SimCell` (`Block` with 6 `VoxelShape`s) copies with canonical `StateId` storage and reversible inline delta journaling (checkpoint / rollback, O(1) ref-counted capability flags rather than sticky `|=` bools that leak on undo).
   - Delete dead overlay/palette code from `sim.rs` and `basis/cell.rs` so total non-test LOC across `engine/law` + `engine/world` strictly **decreases** (no bloat).

## Gates (all must pass with raw numbers in `report.md`)
- `cargo test --release --offline -p law --test jar` (all 9,117+ jar witnesses, 0 failures)
- `cargo test --release --offline -p law --test replay`
- `cargo test --release --offline -p law --lib`
- `cargo test --release --offline -p world`
- `cargo run --release --offline -p law --bin bench` (report raw before/after ns/tick; must not regress beyond measurement noise)
- Exact code-line count before vs after across `engine/law/src/` and `engine/world/src/`.

End with `STATUS: DONE` or `STATUS: BLOCKED: <reason>`.
