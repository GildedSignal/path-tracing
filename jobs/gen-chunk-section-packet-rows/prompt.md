# Generator rows: 26.2 LevelChunkSection palettes and Anvil .mca section encoding

**Goal:** Extract and execute live 26.2 `LevelChunkSection`, `PalettedContainer` bit-packing, `Block.BLOCK_STATE_REGISTRY` ordering, `RegionFile` / `.mca` NBT section loading, and fluid/empty counters on `server-26.2.jar`.

Supplies the exact Anvil & network-section oracle needed to delete the duplicate region/NBT decoder in `engine/mind/src/disk_world.rs` onto `engine/world` and verify `ResidentSource.chunk` single-palette sync (`map/now.md`).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Boot registries + `DataComponentInitializers` + `Bootstrap.bootStrap()`, inspect bytecode with `javap -c -p -constants` for `LevelChunkSection`, `PalettedContainer`, `PalettedContainer$Strategy`, `SerializableChunkData`, and `Block.BLOCK_STATE_REGISTRY` codec.
- Execute real chunk section round-trips (`setBlockState`, empty section, 1-state uniform palette, 4-bit..8-bit linear/hashmap palettes, global 15-bit palette, write to NBT + read back + Y/Z/X index order).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **`PalettedContainer` strategy (`SECTION_STATES`):** exact bits-per-entry transitions (`0` single-value, `4`, `5`, `6`, `7`, `8`, global `ceil(log2(registrySize))`), span alignment (`values_per_long = floor(64 / bits)`, no spanning across `u64` words!), and `BLOCK_STATE_REGISTRY` count (`32,366` states).
2. **`LevelChunkSection` counters & Y/Z/X layout:** exact index formula `((y & 15) << 8) | ((z & 15) << 4) | (x & 15)`, `nonEmptyBlockCount`, `tickingBlockCount`, `tickingFluidCount`, and `hasOnlyAir()`.
3. **Anvil `.mca` region section codec:** real serialized `block_states` (`palette` + packed `long[] data`), Y-section range (`-4..19` / `-64..319`), and upgrade/legacy empty-data single-palette rule on existing repo fixture `engine/world/tests/fixtures/world` (read-only).

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	bits_or_palette	stride_formula	index_order	scars	source`
- `witnesses.tsv`: `case_id	distinct_states	bits_per_entry	long_count	non_empty	fluid_ticking	roundtrip_hash	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any section bit-packing offset, palette resize threshold, registry ID, or NBT round-trip hash that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
