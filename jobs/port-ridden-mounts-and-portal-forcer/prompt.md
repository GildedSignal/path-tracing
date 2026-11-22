# Port delta: LivingEntity.travelRidden mount kinematics and PortalForcer search/create

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/movement.rs:165` (`LivingEntity.travelRidden` refusal!), `engine/law/src/portal.rs`, `map/now.md` (vehicles & portals).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.travelRidden(Player, Vec3)` & `tickRidden(Player, Vec3)` & `getRiddenInput` / `getRiddenSpeed`:
  - Exact execution order when `player.isPassenger() && controllingPassenger == player` (mounted `AbstractHorse`, `Pig`, `Strider`, `Camel`, `HappyGhast`): rotation sync, `getRiddenInput(player, travelVector)` (`xxa * 0.5f`, backward `zza <= 0 -> * 0.25f`), `getRiddenSpeed(player)`, `executeRidersJump` (horse jump charge `>= 90` vs `< 90`, `0.4f + 0.4f * power`, forward boost along yaw `Mth.sin/cos`), `STEP_HEIGHT` (`1.0` for horses), and fall-damage split between rider & mount;
- `PortalForcer.findClosestPortalPosition(BlockPos, boolean isNether, WorldBorder)` & `PortalForcer.createPortal(BlockPos, Direction.Axis)`:
  - Exact search radius (`128` overworld, `16` nether), `PoiManager` distance tie-breaker (`distSqr`, lowest `y`), `BlockUtil.getLargestRectangleAround`, and fallback `createPortal` scan order (`2` loops: 3x1 air clearance vs 1x1 clearance, obsidian floor + 4x5 frame `setBlock` writes with flags `3` & `18`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity.travelRidden`, `AbstractHorse`, `Pig`, `Strider`, and `PortalForcer`, plus executing 26.2 jar witnesses for 10 ticks of horse/pig riding + jump charge (`0`, `50`, `90`, `100`) and `PortalForcer` closest-POI selection + fallback frame coordinates, emitting raw hex bits. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` implementing `travelRidden` / `PortalForcer` in `engine/law`.

**Falsifier:** any 1-ulp mismatch in ridden speed/jump boost/backward penalty or wrong `PortalForcer` tie-break / frame block position against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
