# Port delta: PistonBaseBlock.triggerEvent and PistonMovingBlockEntity player push

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-update-order` (`local-20261111-1844-1-2`) already settled `checkIfExtend -> Level.blockEvent` queueing; do not re-ask how pistons queue block events.

**Consumer:** `docs/plan.md` step 5 & step 8 (redstone doors/platforms/slime launchers, `P2a`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `PistonBaseBlock.triggerEvent` and `PistonStructureResolver.resolve` (to locate): push limit (`12`), `PushReaction` (`NORMAL`/`DESTROY`/`BLOCK`/`IGNORE`/`PUSH_ONLY`), slime/honey stickiness (`canStickTo`), sticky retract-vs-extend behavior, and the `setBlock` flags (`68` / `18`?) creating `MovingPistonBlock` + `PistonMovingBlockEntity`;
- `PistonMovingBlockEntity.tick` and `moveCollidedEntities` / `fixEntityWithinPistonBase` (to locate): progress steps (`0.0 -> 0.5 -> 1.0`), `move(MoverType.PISTON, ...)` entity displacement, and **slime-block launch velocity** (`isSlimeBlock` -> `setDeltaMovement` on player!).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `PistonBaseBlock.triggerEvent`, `PistonStructureResolver.resolve`, and `PistonMovingBlockEntity.tick`/`moveCollidedEntities`, plus direct jar calls checking (1) the `12`-block push limit (`12` succeeds, `13` fails), and (2) `PistonMovingBlockEntity` progress steps and slime-launch velocity constants. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any push limit, stickiness rule, progress step, or slime-launch velocity constant that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
