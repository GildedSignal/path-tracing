# Generator rows: 26.2 block placement, useItemOn dispatch, and door/gate/trapdoor toggles

**Goal:** Extract and execute live 26.2 `ServerPlayerGameMode.useItemOn`, `BlockItem.place`, `isUnobstructed`, directional/half/hinge/waterlog state choices, and `DoorBlock` / `TrapDoorBlock` / `FenceGateBlock` toggles on `server-26.2.jar`.

Supplies exact dispatch order and self-edit echo ground truth for `KIT_PLAN` clicks (`ResidentRunner` + `KitAction.java`) and field edge pricing (`jobs/kit-edges-in-field/prompt.md`). See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: load all tags via `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` and boot `ServerLevel` + `LevelChunk` + `ServerPlayer` + `ServerPlayerGameMode`.
- Inspect bytecode with `javap -c -p -constants` for `ServerPlayerGameMode.useItemOn`, `BlockItem.place`, `BlockItem.canPlace`, `DoorBlock.useWithoutItem`, `TrapDoorBlock.useWithoutItem`, `FenceGateBlock.useWithoutItem`, `SlabBlock.getStateForPlacement`, and `StairBlock.getStateForPlacement`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core witnesses pass, then extend cleanly.

**Cases & rules to extract and witness:**
1. **`useItemOn` precedence:** secondary use active (sneak + holding block) vs block `useItemOn` / `useWithoutItem` vs fallback `BlockItem.place` on clicked face and replaceable blocks (`short_grass`, `water`, `snow` layers).
2. **Entity intersection (`isUnobstructed`) & stack/bucket consume:** exact AABB collision test between placed shape and player bounding box (standing adjacent vs inside target cell vs airborne pillar right under feet), creative vs survival `count` decrement (`1 -> 0` clearing item).
3. **Interactive toggles & neighbor updates:** `oak_door` (lower vs upper click, both halves updated atomically), `iron_door` (`useWithoutItem` refuses!), `oak_trapdoor` (top/bottom half + open toggle), `oak_fence_gate` (facing away from clicker + waterlogged toggle preserved).

**Write in your run folder:**
- `rows.tsv`: `rule_id	method	branch_order	state_props	entity_box_check	consumes_item	scars	source`
- `witnesses.tsv`: `case_id	action	clicked_pos	clicked_face	sneaking	result	before_state	after_state	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `useItemOn` / `place` / toggle case whose `InteractionResult`, resulting block state, double-half update, or item consumption disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
