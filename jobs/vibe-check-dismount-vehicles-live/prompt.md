# Vibe live verification: 26.2 vehicle & bed dismount locations (Horse, Minecart, Boat, Strider, Bed)

**Goal:** Independently execute live 26.2 vehicle and bed dismount searches (`AbstractHorse`, `AbstractMinecart`, `AbstractBoat`, `Strider`, `BedBlock`) across obstructed, slab, ceiling, water, and lava scenes on `server-26.2.jar`.

Cross-checks GLM's `gen-dismount-vehicle-exit-rows` from a second model family and supplies independent witnesses for `engine/law/src/dismount.rs`.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real entities (`Boat`, `Minecart`, `Horse`, `Strider`, `ServerPlayer`).
- Inspect `javap -c -p -constants` for `DismountHelper`, `AbstractBoat.getDismountLocationForPassenger`, `AbstractMinecart.getDismountLocationForPassenger`, `AbstractHorse.getDismountLocationForPassenger`, `BedBlock.findStandUpPosition`, and execute live dismounts (`dismountTo` / `stopRiding`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Open ground + rotated yaw (`0°`, `90°`, `180°`, `270°`):** boat, minecart on rail, horse, bed facing north/east — record exact `(x, y, z)` double bits and ending `Pose`.
2. **Tight & obstructed exits:** 2-high ceiling forcing crouch (`1.5m`) and 1-high gap forcing swim (`0.6m`), slab floor (`y + 0.5`), step-down drop (`y - 1`), and fully walled fallback.
3. **Hazard avoidance:** water below boat escape vector (fallback to boat roof), lava adjacent to strider vs horse (`isDismountable` block/fluid check).

**Write in your run folder:**
- `rows.tsv`: `case_id	vehicle	method	yaw_deg	pose_selected	fallback_used	scars	source`
- `witnesses.tsv`: `case_id	vehicle	scene	exit_x_hex	exit_y_hex	exit_z_hex	pose	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed dismount `(x, y, z)` IEEE bits or `Pose` that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
