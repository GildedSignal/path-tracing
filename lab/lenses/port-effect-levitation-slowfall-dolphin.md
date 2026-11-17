# Port delta: Levitation, Slow Falling, Dolphins Grace and Weaving kinematics

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-status-effect-kinematics` already settled `JUMP_BOOST`, `SPEED`, and `SLOWNESS`; do not re-ask those.

**Consumer:** `engine/law/src/movement.rs:121-123` (`state.effects` refusal), `jobs/port-lenses/report.md:52`, `jobs/port-backlog/report.md` (`WebBlock` `WEAVING` gap in `port-block-laws`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.travelInAir` (to locate): `MobEffects.LEVITATION` (`(0.05 * (amp + 1) - vy) * 0.2`, `resetFallDistance`, gravity skip, interaction with `0.98` vertical drag);
- `LivingEntity.getEffectiveGravity` / `travelInAir` / `checkFallDamage` (to locate): `MobEffects.SLOW_FALLING` (`vy <= 0.0` gravity cap `0.01`, `resetFallDistance`?);
- `LivingEntity.travelInWater` (to locate): `MobEffects.DOLPHINS_GRACE` (`0.96f` drag replacing `0.8f`/`0.9f` and `water_movement_efficiency` interaction);
- `WebBlock.entityInside` (`@4-60,67`): `MobEffects.WEAVING` `(0.5, 0.25, 0.5)` stuck multiplier vs `(0.25, 0.05f, 0.25)`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity.travelInAir`, `getEffectiveGravity`, `travelInWater`, and `WebBlock.entityInside`, plus direct jar calls / bit-exact trace checks of 3 ticks each under `LEVITATION` (amp 0/1), `SLOW_FALLING` (falling vs jumping `vy > 0`), `DOLPHINS_GRACE` (with/without Depth Strider), and `WEAVING` cobweb. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any gravity constant, drag constant, `resetFallDistance` timing, or `WEAVING` multiplier that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
