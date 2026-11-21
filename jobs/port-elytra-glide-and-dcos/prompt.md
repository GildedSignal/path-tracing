# Port delta: LivingEntity.travelFallFlying complete tick recurrence and HotSpot Math.cos

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-elytra-firework-and-impact` and `port-elytra-start-and-stop` are already in `engine/law/src/elytra.rs` (`wt/port-build-b`); `port-elytra-delta` and `port-elytra-bits` settled the cosine calls. This job ports the complete `travelFallFlying` + `updateFallFlyingMovement` recurrence into `engine/law` with a bit-exact cosine implementation.

**Consumer:** `engine/law/src/movement.rs:130-132` (`state.fall_flying` refusal), `engine/law/src/elytra.rs`, `map/detail/engine.md:39`, `map/now.md` step 8.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.travelFallFlying` (`@0-73`) and `LivingEntity.updateFallFlyingMovement(Vec3)` (`@0-262`):
  - Exact branch order: `onClimbable()` -> `travelInAir` + `stopFallFlying` (`@0-15`); else `checkSlowFallDistance()`, save pre-glide `oldVel = getDeltaMovement()` and `d1 = oldVel.horizontalDistance()`, compute `newVel = updateFallFlyingMovement(oldVel)`, `setDeltaMovement(newVel)`, `move(SELF, newVel)`, post-move `d3 = getDeltaMovement().horizontalDistance()`, `handleFallFlyingCollisions(d1, d3)`, and `onGround() && !isClientSide -> stopFallFlying()`;
  - Inside `updateFallFlyingMovement`: `look = getLookAngle()`, `f = getXRot() * ((float)Math.PI / 180F)`, `d0 = Math.sqrt(look.x*look.x + look.z*look.z)`, `d1 = vec.horizontalDistance()`, `d2 = look.length()`, `d3 = Math.cos((double)f)`, `d3 = d3 * d3 * Math.min(1.0D, d2 / 0.4D)`, gravity `vec.add(0.0, d4 * (-1.0 + d3 * 0.75), 0.0)`, dive boost (`vec.y < 0.0 && d0 > 0.0`), pull-up conversion (`f < 0.0f && d0 > 0.0`), and final drag `vec.multiply(0.9900000095367432D, 0.9800000190734863D, 0.9900000095367432D)`.

**Build a jar-backed reproduction:** run `reproduce.sh` executing `updateFallFlyingMovement` in the 26.2 JVM across 20 multi-tick trajectories (diving `+45°`, pulling up `-30°`, level `0°`, steep `+89.5° / -89.5°`, exact `±90°`, and adversarial pitches where UCRT `f64::cos` diverges from HotSpot `Math.cos`), and implement the Rust `update_fall_flying_movement` (and exact `jvm_dcos` for the `f32 * 0.017453292f` domain) as `work.patch` matching every trajectory bit-for-bit (`u64::to_bits`). Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any 1-ulp difference in `(vx, vy, vz)` between the Rust implementation and the 26.2 JVM across all 20 trajectories and the adversarial pitch table.
