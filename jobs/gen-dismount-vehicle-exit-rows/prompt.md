# Generator rows: 26.2 DismountHelper and vehicle/bed exit across Boat, Minecart, Horse, Pig, Strider, Camel

**Goal:** Extract and execute live 26.2 `DismountHelper`, `AbstractBoat.getDismountLocationForPassenger`, `AbstractMinecart.getDismountLocationForPassenger`, `AbstractHorse`, `Pig`, `Strider`, `Camel`, and `BedBlock.findStandUpPosition` on `server-26.2.jar`.

Supplies complete jar-backed cross-vehicle ground truth for the unified `engine/law/src/dismount.rs` primitive so boat, minecart, mount, and bed exits run through one tested generator without per-noun duplication.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements:**
- Mandatory first boot step: load tags via `MappedRegistry.prepareTagReload(...).apply()` + `DataComponentInitializers` + live `ServerLevel` + `LevelChunk` + real entities (`Boat`, `Minecart`, `Horse`, `Pig`, `Strider`, `Camel`, `ServerPlayer`).
- Inspect bytecode with `javap -c -p -constants` for `DismountHelper`, `AbstractBoat.getDismountLocationForPassenger`, `AbstractMinecart.getDismountLocationForPassenger`, `AbstractHorse.getDismountLocationForPassenger`, `Pig.getDismountLocationForPassenger`, `Strider.getDismountLocationForPassenger`, and `BedBlock.findStandUpPosition`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Cases & rules to extract and witness:**
1. **`DismountHelper` core:** `nonSuffocatingOffsets` (2D offset ring order for 1x1 / 2x2 boxes), `findSafeDismountLocation` (`[Standing, Crouching, Swimming]` pose order vs entity width/height), `isBlockFloorValid` (`!isInfinite && < 1.0`), `canDismountTo` (world border + `noCollision` with entity).
2. **Per-vehicle candidate & fluid guards:** `AbstractBoat` single escape vector + water check (`y = boatMaxY` fallback), `AbstractMinecart` clockwise/counter-clockwise side offsets + upright check, `AbstractHorse` side offsets (`left -> right -> forward/back`) + upright/crouch/fallback to saddle top, `Strider` lava tolerance vs standing over lava.
3. **Obstruction & partial-block scenes:** open stone, 1-block slab step, ceiling forcing crouch vs swim pose, surrounded by walls fallback, water below boat, lava below strider vs horse.

**Write in your run folder:**
- `rows.tsv`: `vehicle_id	method	offset_order	pose_order	fluid_guard	fallback_rule	scars	source`
- `witnesses.tsv`: `case_id	vehicle	scene	yaw_hex	exit_x_hex	exit_y_hex	exit_z_hex	pose	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed vehicle exit coordinate (exact IEEE bits) or pose verdict that disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
