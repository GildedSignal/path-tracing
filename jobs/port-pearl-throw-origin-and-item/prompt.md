# Port delta: EnderpearlItem.use spawn position, shootFromRotation and cooldown

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-pearl-raytrace-and-landing` already settled pearl flight and `onHit` teleport, and `port-wormhole-facts` settled speed `1.5f`, spread `±0.0172275`, and `20`-tick cooldown; do not re-ask those.

**Consumer:** `map/now.md` step 8 (`P4c`, deterministic pearl launch trajectory from player `(pos, pose, vel, yaw, pitch)`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `EnderpearlItem.use` (to locate) -> `Projectile.spawnProjectileFromRotation` -> `ThrownEnderpearl.<init>` (to locate): exact initial spawn coordinates (`getX()`, `getEyeY() - 0.1f`, `getZ()`?), `ItemCooldowns.addCooldown(stack, 20)` vs `isOnCooldown`, and `stack.consume(1, player)`;
- `Projectile.shootFromRotation` and `Projectile.shoot` (to locate): exact `f32`/`f64` order of `calculateViewVector(xRot + pitchOffset, yRot)` -> `normalize` -> triangular/Gaussian spread -> `* velocity (1.5f)` -> `shoot` (`entity.getDeltaMovement()` thrower momentum added with `(x, onGround ? 0.0 : y, z)` before or after spread/scale?), and whether `Projectile.spawnProjectile` ticks or moves the projectile on the spawn tick.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `EnderpearlItem.use`, `Projectile.spawnProjectileFromRotation`, `Projectile.shootFromRotation`, `Projectile.shoot`, and `ThrownEnderpearl.<init>`, and invoke them on a zero-RNG (or fixed-RNG) test to print raw `u64` initial `pos` and `vel` bits for a standing vs airborne moving player. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any spawn Y offset, thrower-momentum add order, or `f32`/`f64` cast order that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
