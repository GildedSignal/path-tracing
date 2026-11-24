# Vibe live verification: BucketItem, SolidBucketItem, waterlogging, zero-spread pickup, and clutch fall-reset

**Goal:** Execute 26.2 `BucketItem.use`, `SolidBucketItem.useOn` (water, powder snow, lava, empty bucket pickup), `SimpleWaterloggedBlock` (`Slab`, `Stair`, `TrapDoor`, `Fence`, `Scaffolding`), `WATER_TICK_DELAY = 5` spread schedule (proving pickup at `t <= 4` leaves 0 cascaded edits), Nether `water_evaporates` vs powder snow survival, and impact-tick `fallDistance` reset live on `server-26.2.jar`.

Pins exact self-closing bucket clutch words (`0` open ledger debt when picked up within `<= 4` ticks), Nether fallback (`PowderSnowBucket`), and waterlogging legality for kit unfolding. See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe `ServerLevel` (Overworld & Nether dimension `EnvironmentAttributes.WATER_EVAPORATES`) + real `LevelChunk` + real `CollectingNeighborUpdater` + real `LevelTicks<Block>` + real `LevelTicks<Fluid>`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core bucket place/pickup/spread witnesses pass, then extend cleanly.

**Live scenes to execute:**
1. **Water clutch (`t = 0..6`) with empty-bucket pickup:** fall from height 25 (`fallDistance > 20`), clip & `BucketItem.use` (`ClipContext.Fluid.NONE` vs `SOURCE_ONLY` raycast, placement onto floor face, `FluidTicks` scheduled at `t + 5`), verify water contact resets `fallDistance = 0.0` at landing tick `t0`, then test picking back up at `t0 + 4` (`0` spread writes!) vs `t0 + 5` (first horizontal/downward `FLOWING_WATER` cascades entered into world journal).
2. **Waterlogging (`SimpleWaterloggedBlock`):** `BucketItem.emptyContents` on waterloggable `OAK_SLAB[type=bottom]`, `OAK_STAIRS`, `OAK_TRAPDOOR`, `OAK_FENCE`, `SCAFFOLDING` vs `type=double` / solid replacement failure; verify `BucketPickup` restores `WATERLOGGED=false` (never deleting the host slab/trapdoor!) and `FluidTicks` schedule on waterlogged unlock.
3. **Nether & extreme hazards:** execute water bucket (`WATER_EVAPORATES = true` -> vaporizes, empty bucket returned, `0` water placed, fatal fall) vs `POWDER_SNOW_BUCKET` (`SolidBucketItem.useOn`, succeeds in Nether, resets fall damage, leaves solid snow block to pick back up).
4. **Lava bucket:** `FluidTicks` `delay = 30` Overworld vs `10` Nether (`FAST_LAVA`), water+lava contact reaction (`OBSIDIAN` vs `COBBLESTONE` — proving irreversible/cascaded ledger rules).

**Write in your run folder:**
- `rows.tsv`: `rule_id	item	target_block	dim_flags	delay_ticks	result_state	fall_reset	spread_writes	source`
- `witnesses.tsv`: `case_id	tick	action	pos	state_after	fluid_ticks_queued	fall_distance_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where bucket placement target, waterlog bit, fluid spread tick count (`t = 4` vs `t = 5`), Nether evaporation branch, or `fallDistance` reset disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
