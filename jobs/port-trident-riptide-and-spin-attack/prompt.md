# Port delta: TridentItem Riptide launch, Pose.SPIN_ATTACK and checkAutoSpinAttack

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/movement.rs`, `engine/law/src/player.rs`, `jobs/port-backlog/report.md` (`TridentItem.releaseUsing@348`), `jobs/port-wave-2/report.md`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `net.minecraft.world.item.TridentItem.use` and `releaseUsing` (to locate): `isInWaterOrRain()` gate, `getUseDuration(stack, entity) - timeLeft >= 10` (`THROW_THRESHOLD_TIME = 10`) hold requirement, `EnchantmentHelper.getTridentSpinAttackStrength` (`RIPTIDE` level `1..=3` formula `0.75f * level`?), exact `Mth.sin`/`Mth.cos` launch vector `(f1, f2, f3)` normalized by `sqrt(f1*f1 + f2*f2 + f3*f3)` and scaled by `tridentSpinAttackStrength`, `player.push(f1, f2, f3)`, `startAutoSpinAttack(20, 8.0f, stack)`, and when `onGround()` immediate `move(MoverType.SELF, new Vec3(0.0, 1.1999999284744263, 0.0))`;
- `Player.updatePlayerPose` (`isAutoSpinAttack() -> Pose.SPIN_ATTACK`, dimensions `0.6 x 0.6`, eye height `0.4f`) and `LivingEntity.aiStep` (`autoSpinAttackTicks` decrement before `checkAutoSpinAttack(oldBox, getBoundingBox())`): exact entity-hit loop, `setDeltaMovement(getDeltaMovement().multiply(-0.2, -0.2, -0.2))` bounce, and `horizontalCollision -> autoSpinAttackTicks = 0` termination.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `TridentItem.use`, `TridentItem.releaseUsing`, `Player.startAutoSpinAttack`, and `LivingEntity.checkAutoSpinAttack`, plus a jar-backed execution computing raw `u64` velocity and position bits for Riptide I/II/III launches from ground and water across 5 `(yaw, pitch)` pairs and an entity-hit bounce. Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust port delta as `work.patch`.

**Falsifier:** any Riptide strength formula, trig operation order, grounded `+1.1999999284744263` Y step, or spin-attack bounce/termination rule that disagrees with the 26.2 bytecode.
