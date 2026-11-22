# Port delta: Bow/Crossbow charge, AbstractArrow, Snowball/Egg and FishingHook kinematics

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/items.rs`, `engine/law/src/pearl.rs`, `map/now.md` (projectile & tool mechanics).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BowItem.getPowerForTime(int)` & `BowItem.releaseUsing`: exact charge curve `f = t / 20.0f; f = (f*f + f*2.0f) / 3.0f`, `> 0.1f` min threshold, `1.0f` clamp, `power * 3.0f` launch speed, and crit flag at `power == 1.0f`;
- `CrossbowItem`: charge duration (`getChargeDuration`, Quick Charge reduction), `shootFromRotation` pitch/yaw/multishot spread (`-10f`, `0f`, `+10f` around `upVector`), arrow `3.15f` vs firework `1.6f` velocity;
- `AbstractArrow.tick`: exact air drag (`0.99f` in `f64`), water drag (`getWaterInertia` = `0.6f`), gravity (`0.05` `getDefaultGravity`), block-clip `onHitBlock` (`inGround`, `shakeTime`), and `onHitEntity` velocity-scaled damage `ceil(clamp(vel.length() * baseDamage, 0, max))` + Punch knockback;
- `ThrowableProjectile` (`Snowball`, `ThrownEgg` `0.99` drag / `0.03` gravity) and `FishingHook` cast velocity + `pullEntity(Entity)` delta `(owner - target) * 0.1` impulse.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `BowItem`, `CrossbowItem`, `AbstractArrow`, `ThrowableProjectile`, and `FishingHook`, plus executing the 26.2 jar over charge times `0..=25`, 20-tick air & water arrow trajectories, arrow damage/punch outputs, and `FishingHook.pullEntity`, printing raw `u32`/`u64` hex bits. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` implementing these laws in `engine/law`.

**Falsifier:** any 1-ulp disagreement in bow power curve, arrow/throwable air+water trajectory bits, damage rounding, or fishing-hook pull impulse against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
