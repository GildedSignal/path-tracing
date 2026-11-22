# Port delta: MobEffectInstance tickServer and periodic effect damage/heal/hunger/absorption

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-status-effect-kinematics` and `port-effect-levitation-slowfall-dolphin` already ported passive movement modifiers; do not re-ask those.

**Consumer:** `engine/law/src/movement.rs`, `engine/law/src/player.rs`, `map/now.md` (status effect tick loop & WitherRose / potion lifecycle).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.tickEffects` and `MobEffectInstance.tickServer(LivingEntity, Runnable)`: exact decrement/expiration order (`duration` check, `shouldApplyEffectTickThisTick`), hidden effects stack behavior, and `onEffectUpdated` / `onEffectRemoved`;
- `MobEffect.shouldApplyEffectTickThisTick(int duration, int amplifier)` and `applyEffectTick(ServerLevel, LivingEntity, int amplifier)` for:
  - `POISON` (`25 >> amp`, min `1` tick, `1.0f` `magic` damage clamped so `health` never drops below `1.0f`);
  - `WITHER` (`40 >> amp`, min `1` tick, `1.0f` `wither` damage, lethal);
  - `REGENERATION` (`50 >> amp`, min `1` tick, `heal(1.0f)`);
  - `HUNGER` (`0.005f * (float)(amp + 1)` exhaustion every tick via `Player.causeFoodExhaustion`);
  - `INSTANT_HEALTH`, `INSTANT_DAMAGE`, `SATURATION` (instant ticks & scaling), plus `ABSORPTION` (`onEffectAdded` / `removeAttributeModifiers` / `4 * (amp + 1)` absorption hearts) and `HEALTH_BOOST`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `MobEffectInstance`, `MobEffect`, and subclasses (`PoisonMobEffect`, `WitherMobEffect`, `RegenerationMobEffect`, `HungerMobEffect`, `HealOrHarmMobEffect`, `SaturationMobEffect`, `AbsorptionMobEffect`), plus executing 26.2 jar ticks across levels `0..=3` for each effect (including Poison floor at `1.0f` HP vs Wither lethality, Hunger exact `f32` exhaustion bits, and Absorption expiry). Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` implementing the pure periodic effect law in `engine/law`.

**Falsifier:** any period (`>> amp`), `f32` exhaustion bit, Poison `1.0f` clamp, or instant-heal/damage scaling mismatch against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
