# Vibe live verification: 26.2 LevelChunkSection palettes, bit-packing, and Anvil .mca decoding

**Goal:** Independently execute live 26.2 `LevelChunkSection`, `PalettedContainer` bit-packing, `Block.BLOCK_STATE_REGISTRY` indexing, and `.mca` `RegionFile` section reads against `engine/world/tests/fixtures/world` on `server-26.2.jar`.

Cross-checks GLM's `gen-chunk-section-packet-rows` from a second model family to verify unifying `engine/mind/src/disk_world.rs` onto `engine/world` (`jobs/branch-consolidation/report.md`).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + inspect `javap -c -p -constants` for `LevelChunkSection`, `PalettedContainer`, `PalettedContainer$Strategy`, and `RegionFile` / `SerializableChunkData`.
- Read actual `.mca` sections (read-only) from `engine/world/tests/fixtures/world/dimensions/minecraft/overworld/region/` and verify decoded state names + properties match `ServerLevel` chunk loading and `PalettedContainer` round-trips.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **`PalettedContainer` bit-width transitions:** `1` distinct state (`0` bits, empty `long[]`), `2..16` (`4` bits, `16` per `u64`), `17..32` (`5` bits, `12` per `u64`), `257+` (global `15` bits, `4` per `u64`), proving zero entries straddle `u64` boundaries.
2. **Fixture `.mca` cross-check:** decode sampled chunks/sections from `engine/world/tests/fixtures/world` (read-only), verify `BLOCK_STATE_REGISTRY.size() == 32366`, Y-index order `((y & 15) << 8) | ((z & 15) << 4) | (x & 15)`, and non-empty/fluid counts.

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	distinct_range	bits_per_entry	per_long	scars	source`
- `witnesses.tsv`: `case_id	chunk_xz	section_y	bits	long_len	non_empty	sample_state	hash`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any palette bit-width threshold, `u64` packing offset, registry state index, or fixture `.mca` decoded state that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
