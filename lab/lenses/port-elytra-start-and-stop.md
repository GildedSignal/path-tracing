# Port delta: client elytra start trigger, GLIDER component and durability wear

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-elytra-delta`, `port-elytra-climbable-branch`, `port-elytra-firework-and-impact`, and `port-movement-modes` already settled `travelFallFlying`, `tryToStartFallFlying`, and `!canGlide() -> stop`; do not re-ask those.

**Consumer:** `docs/plan.md` step 8 (`P4b`), `elytra_challenge_2`, `elytra_challenge_3`, and `R-brach`.

**From the 26.2 client and server bytecode**, state with class, method and offsets:
- `LocalPlayer.aiStep` (to locate) -> `tryToStartFallFlying`: exact client predicate that sends `ServerboundPlayerCommandPacket.Action.START_FALL_FLYING` (`!wasJumping && jump`, `!onGround`, `!isFallFlying`, `!isInWater`, `!abilities.flying`, `canGlide` / `EquipmentSlot` `DataComponents.GLIDER`, to locate), and whether `tryToStartFallFlying` checks `MobEffects.LEVITATION` (`!hasEffect(LEVITATION)`);
- `LivingEntity.canGlide` / `EquipmentSlot` / `LivingEntity.updateFallFlying` (to locate): in 26.2 (`DataComponents.GLIDER`), which armor slots are checked, what durability condition (`nextDamageWillBreak` at `damage >= maxDamage - 1`) stops gliding, and the exact tick interval (`fallFlyingTicks % 20 == 0`?) at which elytra loses 1 durability.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LocalPlayer` start-fall-flying trigger, `LivingEntity.canGlide`, `LivingEntity.updateFallFlying`, and `Player.tryToStartFallFlying`, plus a direct jar probe verifying `canGlide` across `ELYTRA` at `damage = maxDamage - 2` vs `maxDamage - 1`, `onGround`, `isInWater`, and `LEVITATION`. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any start condition, slot check, durability break threshold, or durability wear period that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
