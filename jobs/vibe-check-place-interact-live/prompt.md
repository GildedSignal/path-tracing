# Vibe live verification: 26.2 useItemOn, placement obstruction, pillar-under-feet, and door/gate/trapdoor toggles

**Goal:** Independently execute live 26.2 `useItemOn` dispatch, `isUnobstructed` player overlap checks (standing in cell vs pillar under feet), sneak-bypass, and `DoorBlock` / `TrapDoorBlock` / `FenceGateBlock` toggles on `server-26.2.jar`.

Cross-checks GLM's `gen-place-interact-echo-rows` and directly verifies the live `doorcorridor`, `doorparkourblue`, and `fencemaze` mechanics from `map/now.md`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Mandatory first boot step: `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + `ServerPlayer` + `ServerPlayerGameMode`.
- Inspect `javap -c -p -constants` for `ServerPlayerGameMode.useItemOn`, `BlockItem.place`, `DoorBlock.useWithoutItem`, `TrapDoorBlock.useWithoutItem`, `FenceGateBlock.useWithoutItem`, and execute all scenes live.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Door / trapdoor / gate toggles & sneak bypass:** click `oak_door` (upper vs lower half, open -> closed -> open, both halves matching), `iron_door` (`PASS`), `oak_trapdoor`, `oak_fence_gate` (clicked while holding `cobblestone` standing: toggles without placing; clicked while sneaking: places against face!).
2. **`isUnobstructed` entity box overlap:** place `cobblestone` while player legs occupy target cell (FAIL), feet `0.0` above support placing below feet while airborne (SUCCESS — pillar), feet overlapping top `1e-7` (verify exact edge!), and placing `carpet` vs `slab` inside/beside player.
3. **Self-edit echo state identity:** verify `setBlock` + `useItemOn` post-state matches client state for replaceable (`short_grass`, `water`) vs solid (`stone`) supports.

**Write in your run folder:**
- `rows.tsv`: `case_id	method	result	sneaking	item_consumed	state_change	scars	source`
- `witnesses.tsv`: `case_id	player_y_hex	target_pos	face	held_item	result	block_after	inv_after	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed `InteractionResult`, placement overlap edge, door-half sync, or inventory decrement that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
