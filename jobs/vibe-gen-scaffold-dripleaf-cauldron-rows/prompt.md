# Vibe generator rows: scaffolding, big dripleaf, cauldrons, and offset shapes

**Goal:** One bit- and behaviour-exact witness table for 26.2 `ScaffoldingBlock` (distance `0..7` & falling transition), `BigDripleafBlock` tilt schedule, `CauldronBlock` / `LayeredCauldronBlock`, and non-zero voxel offset blocks (`Bamboo`, `PointedDripstone`) executed against `server-26.2.jar`.

Covers special collision shapes & block schedules in `jobs/law-grouping-audit/table.md` groups 1, 6, and 16 (`census.tsv` shapes & scars).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot real jar tags (`MappedRegistry.prepareTagReload`) and real `ServerLevel` + `LevelChunk` + `LevelTicks<Block>`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early once core executed runs pass, then extend cleanly.

**From bytecode + live execution:**
1. **`ScaffoldingBlock` (`getCollisionShape`, `getDistance`, `tick`):** dynamic collision shape switch (`distance == 0 && bottom == true`, descending `isDescending()` / sneak pass-through vs above `1.0` shelf `isAbove`), `STABILITY_MAX_DISTANCE = 7` collapse into `FallingBlockEntity` after `1`-tick schedule.
2. **`BigDripleafBlock` (`entityInside`, `tick`, `TILT` state machine):** `NONE -> UNSTABLE (10 ticks) -> PARTIAL (10 ticks) -> FULL (100 ticks) -> NONE` cadence, which states remove collision shape, reset on redstone power (`setTiltAndScheduleTick`), and entity step detection box.
3. **`LayeredCauldronBlock` / `LavaCauldronBlock`:** fluid level `1..3`, exact water-level height formula `(6.0 + level * 3.0) / 16.0`, inner collision shape, fire-extinguish + level decrement scar, powder snow melt.
4. **Coordinate-seeded voxel offsets (`BlockStateBase.getOffset`, `BambooStalkBlock`, `PointedDripstoneBlock`):** exact `Mth.getSeed(x, 0, z)` bit hashing, `maxHorizontalOffset`, and resulting shifted AABB coords hex across test coordinates.

**Write in your run folder:**
- `rows.tsv`: `rule_id	block_id	schedule_or_offset	coords_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	pos	state_props	shape_or_aabb_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any tick where scaffolding stability distance, dripleaf tilt delay/collision shape, cauldron height, or offset AABB hex differs from `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
