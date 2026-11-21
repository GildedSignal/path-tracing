# Port delta: AbstractMinecart rail kinematics, slopes, curves and PoweredRailBlock

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/movement.rs:127-129` (`state.riding` refusal), `jobs/port-wave-2/report.md` (non-boat vehicles), `map/detail/plan.md` step 8.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `net.minecraft.world.entity.vehicle.minecart.AbstractMinecart` and `OldMinecartBehavior` (vs `NewMinecartBehavior` — check which is default when `FeatureFlags.MINECART_IMPROVEMENTS` is off!):
  - `tick()` order: gravity (`isNoGravity ? 0 : (isInWater ? -0.005 : -0.04)`), rail detection at `floor(x), floor(y), floor(z)` (and `y - 1` if `BaseRailBlock.isRail(below)`!), `moveAlongTrack` vs `comeOffTrack`;
  - Inside `OldMinecartBehavior.moveAlongTrack`: `PoweredRailBlock` brake (`!powered`: `horizontalDistance() < 0.03 -> Vec3.ZERO`, else `multiply(0.5, 0.0, 0.5)`) before rail step vs boost (`powered`: `> 0.01` adds `v / d * 0.06`, or wall push `±0.02`), slope gravity (`±0.0078125D`, halved in water), passenger forward steer (`ServerPlayer.getLastClientMoveIntent().forward()` when horizontal speed2 `< 0.01`: adds `look * 0.1`?), rail exit snap onto the 2-endpoint line `EXITS[RailShape]`, `maxSpeed` clamp (`0.4D`, `0.2D` in water), and `applyNaturalSlowdown` (`0.997D` with passenger vs `0.96D` empty);
  - Player mount (`RideableMinecart.interact` -> `startRiding`) and `getDismountLocationForPassenger`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `AbstractMinecart`, `OldMinecartBehavior`, and `PoweredRailBlock`, plus a jar-backed execution stepping an occupied and unoccupied minecart over: (1) unpowered->powered flat `PoweredRailBlock`, (2) ascending/descending slopes, (3) a 90° curve (`SOUTH_EAST`), and (4) off-rail landing, printing raw `u64` position and velocity bits per tick. Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust port delta (`engine/law/src/minecart.rs`) as `work.patch`.

**Falsifier:** any rail Y-lookup offset, slope acceleration, curve projection, passenger drag (`0.997` vs `0.96`), or powered-rail boost/brake threshold that disagrees with the 26.2 jar execution.
