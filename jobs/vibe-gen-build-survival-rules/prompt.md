# Vibe live verification: 26.2 block survival, updateShape cascade, and hanging/attached/gravity support

**Goal:** Extract and execute live 26.2 `BlockStateBase.canSurvive`, `updateShape` cascade drops, `FallingBlock` triggers, and attached/hanging/fluid-basin support rules across survival blocks on `server-26.2.jar`.

Supplies the jar-truth rule table to unify `engine/build`'s hand-coded survival checks (`verify.rs` / `waves.rs` — gravity, water basin, hanging blocks noted in `jobs/branch-consolidation/report.md`) onto `engine/world/data/cell_rule_index.tsv` + `engine/law/src/basis/rules.tsv`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + `ServerLevel.tick` / neighbor updates.
- Inspect `javap -c -p -constants` for `TorchBlock.canSurvive`, `WallTorchBlock.canSurvive`, `LanternBlock.canSurvive`, `FaceAttachedHorizontalDirectionalBlock`, `CarpetBlock`, `LadderBlock`, `FallingBlock`, `DoorBlock.updateShape`, `RailBlock.canSurvive`, and `BambooStalkBlock`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Support-required blocks (`canSurvive` + `updateShape`):** test placing & removing support beneath/beside `torch`, `wall_torch`, `lantern` (`hanging=false/true`), `oak_door` (lower vs upper support), `oak_button` (floor/wall/ceiling), `rail`, `ladder`, `white_carpet`, `scaffolding` (horizontal reach `0..7`), `snow` layer, `lily_pad`, `big_dripleaf`.
2. **Gravity & fluid containment:** `sand` / `gravel` / `anvil` / `concrete_powder` over air vs solid vs `slab` vs `water`, plus `water` / `lava` source containment adjacent to solid vs air vs `waterloggable` slab/fence.
3. **Full enumeration summary:** sweep all representative support-rule families in `BLOCK_STATE_REGISTRY` (`canSurvive` on air vs solid bottom/side/top) to map every unique `survive_rule` class.

**Write in your run folder:**
- `rows.tsv`: `block_family	rule_kind	required_support_dir	requires_face_shape	update_effect	scars	source`
- `witnesses.tsv`: `case_id	block_state	support_pos	support_state	can_survive	post_update_state	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `canSurvive` verdict or `updateShape` pop/fall transition that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
