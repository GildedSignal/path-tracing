# Vibe live verification: 26.2 FlowingFluid spread timing, slope drop BFS, infinite source, and push vector

**Goal:** Execute live 26.2 `FlowingFluid` tick scheduling (`5t` water, `30t` overworld lava / `10t` nether lava), slope-distance BFS toward drop-offs, infinite water source creation (`>= 2` adjacent sources), and entity fluid flow push vector on `server-26.2.jar`.

Supplies live tick-by-tick ground truth for `engine/law/src/fluid.rs` and bucket/lava clutch timing (`map/now.md`). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + `ServerLevel.tick(...)` for water/lava fluid ticks + `LivingEntity` flow push.
- Inspect `javap -c -p -constants` for `FlowingFluid.tick`, `FlowingFluid.getNewLiquid`, `FlowingFluid.getSlopeDistance`, `FlowingFluid.getFlow`, and `WaterFluid` / `LavaFluid`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Water spread + drop-off attraction:** place water source on stone ledge 3 blocks from a drop-off, step `ServerLevel` `0..40` ticks, record `level` (`1..8` / `falling`) per cell and exact tick when edge falls off.
2. **Lava Overworld vs Nether (`fast_lava`) + cobble/obsidian reaction:** step lava source in `overworld` (`30t` tick delay, drop-off `2`) vs `the_nether` (`10t` tick delay, drop-off `1`), plus water-lava contact creating cobblestone vs obsidian.
3. **Infinite water source & fluid push vector:** `2×2` and `3×1` source tests (`canConvertToSource`), plus normalized flow push vector `FluidState.getFlow` on `ServerPlayer` across levels `1..8`.

**Write in your run folder:**
- `rows.tsv`: `fluid_id	dimension	tick_delay	drop_off	push_scale_hex	infinite_source	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	fluid_level	falling	flow_vx_hex	flow_vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed fluid tick delay, spread cell level, cobblestone/obsidian transform, or `getFlow` vector bit that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
