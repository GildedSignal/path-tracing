# Port delta: FallingBlockEntity air flight, timeout and anvil/dripstone damage

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-collapse-schedule`, `port-collapse-due-tick`, and `port-falling-block-landing-place` already settled updateShape scheduling, due-tick `isFree(below)`, and grounded landing `setBlock`; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 & step 8 (sand/gravel/concrete/anvil falling trajectories and timing).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `FallingBlockEntity.tick` (`@0-241` before the already-settled `@241+` grounded placement): initial velocity/position when spawned by `FallingBlockEntity.fall(level, pos, state)` (`(x+0.5, y, z+0.5)`, `Vec3.ZERO`, `blockState` flag set in `fall`), `time++`, gravity `-0.04d` (`getDefaultGravity`) before `move(SELF)` vs `0.98d` drag after `move(SELF)` (`scale(0.98)`), and timeout discard/drop (`time > 100 && (y <= minY || y > maxY) || time > 600`);
- `ConcretePowderBlock.shouldSolidify` / `ConcretePowderBlock.onLand` (to locate) and `FallingBlockEntity.causeFallDamage` (to locate): anvil / pointed-dripstone entity damage formula and concrete powder solidification in flight/landing.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `FallingBlockEntity.fall`, `FallingBlockEntity.tick` (`@0-241`), and `causeFallDamage`, and step a real `FallingBlockEntity` (`SAND` and `ANVIL`) for 10 falling ticks in air printing raw `u64` `y` and `vy` bits each tick plus the `time > 100` / `time > 600` discard condition. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any gravity/drag constant or order (`move` vs `0.98`), spawn centering, or timeout tick that disagrees with `FallingBlockEntity` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
