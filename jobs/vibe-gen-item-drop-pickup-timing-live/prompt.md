# Vibe live verification: 26.2 ItemEntity drop spawn, 10t pickup delay & pickup AABB

**Goal:** Execute live 26.2 `Block.popResource` / `Player.drop`, `ItemEntity.pickupDelay` (`10` / `40`), `Player.aiStep`, and `ItemEntity.playerTouch` on `server-26.2.jar` so `ametwoodbox` break-and-restore pickup timings are jar-certified.

See `lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/` for clean `DataComponentInitializers` / tag boot.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`; java/javac: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/`.

**Harness requirements (mandatory executed witnesses):**
- Boot tags (`MappedRegistry.prepareTagReload(...).apply()`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` + real `ServerPlayer`.
- Inspect bytecode with `javap -c -p -constants` for `net.minecraft.world.entity.item.ItemEntity` (`tick`, `playerTouch`, `setDefaultPickUpDelay`, `setPickUpDelay`), `Block.popResource`, and `net.minecraft.world.entity.player.Player.aiStep` / `touch`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early, then extend cleanly.

**Live scenes to execute:**
1. **Block break item spawn & velocity:** break `oak_planks` and `amethyst_block` in `ServerLevel`, recording spawned `ItemEntity` `(x, y, z, vx, vy, vz)` and initial `pickupDelay` (`10` ticks default).
2. **Pickup AABB & tick window:** tick `ServerPlayer` + `ItemEntity` across ticks `0..=15` at horizontal offsets `(0.0, 0.8, 1.0, 1.3, 1.5)` until pickup succeeds or fails, recording `pickupDelay` countdown and exact `Inventory` delta tick.
3. **Player drop throw velocity:** execute `ServerPlayer.drop(item, false, false)` (`pickupDelay = 40`) and record exact throw `(vx, vy, vz)` and `pickupDelay`.

**Write in your run folder:**
- `rows.tsv`: `case_id	item	initial_delay	pickup_tick	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	item_x_hex	item_y_hex	item_z_hex	delay_rem	picked_up`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where item spawn position/velocity, `pickupDelay` threshold, or pickup AABB inflation differs from `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
