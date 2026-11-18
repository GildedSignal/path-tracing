# Port delta: NetherPortalBlock.getPortalDestination and TeleportTransition

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-portal-entry-and-create` already settled the 81-tick counter and `PortalForcer.createPortal` footprints, and `port-wormhole-facts` settled `16`/`128` radii and `8:1` scale; do not re-ask those.

**Consumer:** `map/now.md` step 8 (`P4c`, exact exit coordinates, yaw/velocity preservation across Overworld <-> Nether portal transit).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `NetherPortalBlock.getPortalDestination` and `NetherPortalBlock.getDimensionTransitionFromExit` (to locate): how `DimensionType.getTeleportationScale` (`8.0` / `0.125`) + `WorldBorder.clampToBounds` computes `blockPos` (`BlockPos.containing(x * scale, y, z * scale)`?), how `PortalShape.getRelativePosition` and `PortalShape.findCollisionFreePosition` (to locate) place the player inside the target `BlockUtil.FoundRectangle`, and what happens to `yaw`, `pitch`, and `deltaMovement` (`Vec3` rotated on axis change `X <-> Z`?);
- `Entity.handlePortal` (`@64-145`): exact order of `setPortalCooldown()`, `getPortalDestination`, and `teleport(transition)` (notably when `getPortalDestination` returns `null`!).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `NetherPortalBlock.getPortalDestination`, `getDimensionTransitionFromExit`, `PortalShape.getRelativePosition`/`findCollisionFreePosition`, and `Entity.handlePortal`, plus direct jar execution of exit coordinate/rotation calculation across matching-axis (`X->X`) and rotated-axis (`X->Z`) portals. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any exit coordinate offset, axis-swap rotation, velocity rotation, or `null`-destination cooldown behavior that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
