# Vibe live verification: elytra, firework boost, portals, and pistons executed headless

**Goal:** Execute 26.2 `Elytra` glide + `FireworkRocketEntity` boost, `NetherPortalBlock` / `PortalForcer` / `EndPortalBlock`, and `PistonBaseBlock` / `PistonStructureResolver` / `PistonMovingBlockEntity` LIVE on `ServerLevel` + `LevelChunk` + `LevelTicks` against `server-26.2.jar`.

Directly grounds the three non-affine kernels (`Pistons`, `Elytra`, `Portals`: `jobs/law-grouping-audit/table.md` groups 5, 14, 15) in bit-exact executed trajectories.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` (`FIREWORK_ROCKET`, `ELYTRA` components!) + Unsafe `ServerLevel` + real `LevelChunk` + `LevelTicks` + `BlockEvent` queue.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after first kernel executes, then extend cleanly.

**Live scenes to execute:**
1. **Elytra glide & firework (`ServerPlayer.travelFallFlying`, `FireworkRocketEntity.tick`):** pitch angles `-45°, 0°, +30°` with/without attached firework rocket (flight `1..3`), horizontal crash against stone wall (damage threshold), exact position & velocity hex per tick.
2. **Pistons (`PistonBaseBlock.triggerEvent`, `PistonStructureResolver`, `PistonMovingBlockEntity.tick`):** 12-block push OK vs 13-block refuse; sticky piston pull vs slime-to-honey non-stick; moving block `progress` `0.0 -> 0.5 -> 1.0`; entity pushed horizontally/vertically (and launched on slime block!).
3. **Portals (`NetherPortalBlock`, `PortalForcer.findClosestPortalPosition` / `createPortal`, `EndPortalBlock`):** survival `80`-tick vs creative `0`-tick delay, `8x` Overworld <-> Nether scaling, `PortalForcer` search radius `128`/`16`, `portalCooldown = 10`, End obsidian platform `(100, 48, 0)` build.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kernel	constants_hex	limits_or_delays	scars	source`
- `witnesses.tsv`: `case_id	tick	entity_or_pos	x_hex	y_hex	z_hex	vx_hex	vy_hex	vz_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed tick where glide velocity hex, firework boost, piston push cap/bounce, or portal scaling/cooldown disagrees with `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
