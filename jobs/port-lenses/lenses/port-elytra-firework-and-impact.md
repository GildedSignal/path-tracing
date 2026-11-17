# Port delta: elytra firework boost and wall-impact damage

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-elytra-delta`, `port-glide-cosine-exact`, `port-elytra-climbable-branch`, and `port-movement-modes` (`1109-1554-1-4`, `1111-1823-1-4`) already settled free-glide and `START_FALL_FLYING`/`updateFallFlying`; do not re-ask those.

**Consumer:** `docs/plan.md` step 8 (carried movement: elytra) and batch `P4b` (`jobs/port-backlog/report.md`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `FireworkRocketItem.use` -> `FireworkRocketEntity.tick` (to locate): attached-entity flight duration (`lifeTime`, RNG bounds if any, to locate), and the exact per-tick velocity rewrite on the `isFallFlying()` rider (`lookAngle`, `1.5`, `0.1`, `0.5` or exact 26.2 bytecode constants) and where it runs relative to `LivingEntity.travelFallFlying`;
- `LivingEntity.travelFallFlying` -> `handleFallFlyingCollisions` (to locate): exact horizontal speed loss `d10 = d6 - horizontalLength`, threshold (`> 0.0`? `- 0.3`? `3.0f` multiplier? exact 26.2 constants, to locate), and `hurtServer(flyIntoWall)` damage.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `FireworkRocketEntity.tick` and `handleFallFlyingCollisions`, plus a jar-backed witness computing (1) 3 ticks of attached firework boost from a fixed `(yaw, pitch, v0)` with raw `f64` bits, and (2) wall-impact damage across speed drops spanning the exact threshold. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any firework velocity constant, tick ordering, or wall-impact damage threshold that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
