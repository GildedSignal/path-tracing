# Port delta: ender pearl water drag, raytrace and landing teleport

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-wormhole-facts` already settled throw speed `1.5`, spread `±0.0172275`, thrower momentum, `0.03` gravity, `0.99` drag, `5.0f` damage, and `20t` cooldown; do not re-ask those.

**Consumer:** `docs/plan.md` step 8 (carried movement: pearl), batch `P4c` (`jobs/port-backlog/report.md`), and `port-wormhole-facts`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ThrownEnderpearl.tick` / `ThrowableProjectile.tick` (or 26.2 superclass, to locate): exact tick order of `ProjectileUtil.getHitResultOnMoveVector` (raytrace block clip mode, `ClipContext.Block` & `ClipContext.Fluid`, entity AABB inflation, to locate), `setPos`, water detection & `0.8f` vs `0.99f` drag, and `applyGravity`;
- `ThrownEnderpearl.onHit` (to locate): exact teleport target coordinates (`hitResult.getLocation()`, `xo/yo/zo`, or `this.getX/Y/Z()`, to locate), `resetFallDistance`, whether `teleportTo` clamps or checks unloaded chunks, and order relative to `hurtServer(5.0f)`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ThrownEnderpearl.tick` / `onHit` and `ThrowableProjectile.tick`, plus a direct jar-backed trace of (1) a pearl entering water (drag `0.8f` vs `0.99f` and gravity order), and (2) hitting a solid block face (exact landed `(x, y, z)` bits and `fallDistance`). Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any raytrace clip mode, drag/gravity order, or landed teleport coordinate that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
