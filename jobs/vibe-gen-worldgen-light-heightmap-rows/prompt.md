# Vibe generator rows: 16³ section masks, heightmaps, fluid reactions, and fire/farmland scars

**Goal:** Exact 26.2 specification and live jar witnesses for `LevelChunkSection` (`hasFluid`, tick/random-tick counts), `Heightmap` predicates (`MOTION_BLOCKING`, `MOTION_BLOCKING_NO_LEAVES`, `WORLD_SURFACE`), water–lava block reactions, fire tick cadences, and farmland trample/hydration.

Supports step 3's unified `16³` chunk section + `4×4×4` mask spine (`map/now.md`) and closes fluid-reaction & farmland edge cases in `jobs/law-grouping-audit/table.md` groups 1, 4, and 9.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot real jar tags (`MappedRegistry.prepareTagReload`) and real `ServerLevel` + `LevelChunk` + `LevelChunkSection` + `LevelTicks`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early once core executed runs pass, then extend cleanly.

**From bytecode + live execution:**
1. **`LevelChunkSection` & `Heightmap` (`recalcBlockCounts`, `Heightmap.Types` predicates, `setBlockState`):** exact bit/index layout inside a `16³` section (`(y & 15) << 8 | z << 4 | x` vs palette order), what `nonEmptyBlockCount` / `tickingBlockCount` / `tickingFluidCount` count, and exact heightmap predicates over solid, fluid, leaves, carpet, slab, cobweb.
2. **Water–lava reactions (`LiquidBlock.shouldSpreadLiquid`, `LavaFluid.spreadTo`, `FrostWalker` / `Obsidian` / `Cobblestone` / `Basalt` / `Stone`):** exact `level == 0` -> Obsidian vs flowing -> Cobblestone; `Lava` onto Water -> Stone; Blue Ice + Soul Soil -> Basalt scar; neighbor reaction direction order.
3. **Fire & Farmland (`FireBlock.tick`, `BaseFireBlock`, `FarmBlock.fallOn`):** fire schedule `30 + random.nextInt(10)`, burn/encourage tables, rain extinguish, farmland `fallOn` exact `random.nextFloat() < fallDistance - 0.5f` + `0.75` crop trample scar -> `Blocks.DIRT`.

**Write in your run folder:**
- `rows.tsv`: `rule_id	subsystem	constants_or_predicate	order_or_delay	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	state_before	state_after	counts_or_height	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any difference in section counter transitions, heightmap top Y, fluid reaction block result, or fire/farmland update against `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
