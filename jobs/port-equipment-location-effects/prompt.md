# Port delta: equipment-driven movement enchantments (Soul Speed, Depth Strider, Swift Sneak)

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-movement-modes` (`local-20261114-1013-1-1`) already settled `EnchantmentHelper.runLocationChangedEffects` trigger timings (`baseTick`, `checkFallDamage`, `collectEquipmentChanges`, `setGameMode`); do not re-ask those triggers.

**Consumer:** `engine/law/src/attributes.rs`, `engine/law/src/movement.rs:469-470` (`water_movement_efficiency`, `sneaking_speed`), `map/now.md` ("kits and many bots"), batch `P4e`.

**From the 26.2 server bytecode and jar enchantment JSONs**, state with class, method and offsets:
- `data/minecraft/enchantment/soul_speed.json`, `depth_strider.json`, `swift_sneak.json`, and `feather_falling.json` (to locate): exact enchantment effect types, level-based value formulas, attribute IDs (`MOVEMENT_SPEED`, `WATER_MOVEMENT_EFFICIENCY`, `SNEAKING_SPEED`, `MOVEMENT_EFFICIENCY`), operations (`ADD_VALUE` vs `ADD_MULTIPLIED_TOTAL`), and slot filters;
- `LivingEntity.tryAddSoulSpeed` / `onChangedBlock` / `getBlockSpeedFactor` (to locate): exact soul-block speed modifier (`0.03 + level * 0.0105`? `BLOCK_SPEED_FACTOR` interaction: does Soul Speed also prevent `soul_sand`'s `0.4` speedFactor slowdown in `LivingEntity.getBlockSpeedFactor`?!) and durability wear (`4%` chance?).

**Build a jar-backed reproduction:** run `reproduce.sh` extracting the four enchantment JSONs from the server jar and disassembling `LivingEntity.getBlockSpeedFactor` and `EnchantmentAttributeEffect` / `EnchantmentHelper`, plus direct jar calculation of `MOVEMENT_SPEED`, `WATER_MOVEMENT_EFFICIENCY`, and `SNEAKING_SPEED` attribute values at levels `1..=3` on soul sand (`0.4f`) vs stone. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any attribute modifier constant, operation, or `soul_sand` speed-factor bypass condition that disagrees with the 26.2 jar JSONs and bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
