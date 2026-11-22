# Port delta: air supply, breath regeneration, drowning damage and Respiration

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/movement.rs`, `engine/law/src/player.rs`, `map/now.md` (full 26.2 player survival loop).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.baseTick` (breath section), `LivingEntity.decreaseAirSupply(int)`, `LivingEntity.increaseAirSupply(int)`, `Entity.getMaxAirSupply()` (`300`), `Entity.isEyeInFluid(TagKey<Fluid>)`, `LivingEntity.canDrownInFluidType` / `canBreatheUnderwater`:
  - Exact gate for losing breath vs recovering (`increaseAirSupply`: `+4` clamped to `maxAir`? Or only when not submerged / in bubble column? Exact `BubbleColumnBlock` air behavior vs waterlogged non-column blocks);
  - `decreaseAirSupply`: exact `Attributes.OXYGEN_BONUS` / Respiration enchantment `random.nextInt(bonus + 1) > 0` check, `WATER_BREATHING` and `CONDUIT_POWER` bypasses, and `invulnerable` / `Abilities.invulnerable` interaction;
  - Drowning threshold (`airSupply <= -20`?), counter reset (`airSupply = 0`), damage (`2.0f` `damageSources().drown()`), and exact tick ordering relative to `baseTick` fire/lava/travel.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling the breath/drowning methods in `Entity` and `LivingEntity` and executing the 26.2 jar across 10 multi-tick scenarios (submerged `300` -> `0` -> `-20` drown hit -> reset `0`; surfacing recovery; bubble column underwater; `WATER_BREATHING` vs Respiration RNG bypass; creative/invulnerable bypass), emitting raw tick-by-tick `airSupply`, `health`, and `hurt_res` bits. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` implementing the pure breath/drowning law in `engine/law`.

**Falsifier:** any 1-tick shift in drowning trigger tick, recovery rate (`+4`/tick), `airSupply` clamping, or damage amount (`2.0f`) against the 26.2 jar execution.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
