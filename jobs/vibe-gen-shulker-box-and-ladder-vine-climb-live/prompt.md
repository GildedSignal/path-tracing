# Vibe live verification: 26.2 shulker-box collision shapes and ladder/vine climb gait

**Goal:** Execute live 26.2 `ShulkerBoxBlock.getCollisionShape` / `Shulker.getProgressDeltaAabb` (closed vs opening) and `LivingEntity.handleOnClimbable` (ladders, vines, scaffold-free trapdoor-above-ladder, sneak hold, top exit) on `server-26.2.jar`.

See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer`.
- Inspect bytecode with `javap -c -p -constants` for `ShulkerBoxBlock`, `ShulkerBoxBlockEntity`, `LivingEntity.handleRelativeFrictionAndCalculateMovement`, `LivingEntity.handleOnClimbable`, `LadderBlock`, and `VineBlock`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Colored shulker boxes (white, lime, magenta, orange, red, yellow):** query `getShape` / `getCollisionShape` closed vs lid progress (`0.0..1.0`) per facing, and test standing/walking onto a closed shulker box and falling on `hasSuffocatingBlock` / non-cubic top.
2. **Ladder & vine climb steady gait:** climb an 8-high ladder (`horizontalCollision` via forward key vs holding `jump`), hover via `sneak` (`shiftKeyDown=true`), descend without jump, and exit onto top solid lip; do the same on backed `vine` and open `trapdoor` over ladder (`LADDER` facing match). Record `(x, y, z, vx, vy, vz)` IEEE hex bits and `onClimbable` per tick.

**Write in your run folder:**
- `rows.tsv`: `case_id	block_id	state	tick	x_hex	y_hex	z_hex	vy_hex	flags	source`
- `witnesses.tsv`: `case_id	tick	x_hex	y_hex	z_hex	vy_hex	flags`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed shulker-box collision bound or climb velocity bit in `server-26.2.jar` that disagrees with the local-shape or `0.2` / `-0.15` clamp model.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
