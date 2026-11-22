# Port delta: Shield blocking/disable, attack crit/sweep and Mace smash/wind-burst

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `combat.rs` already covers `Player.attack` cooldown scaling, `causeExtraKnockback`, and `LivingEntity.knockback`; do not duplicate those.

**Consumer:** `engine/law/src/combat.rs`, `engine/law/src/player.rs`, `map/now.md` (melee/tool combat laws).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.applyItemBlocking` & `BlocksAttacks`: exact warm-up delay (`getItemBlockingWithDelay` / `useDuration`), horizontal view-angle cosine dot-product gate (`blockedByShield` vs bypassing damage tags), blocked amount, and axe/attack `disableShield` cooldown (`5 * 20` = `100` ticks, `BlocksAttacks.disableBlockingForSeconds`);
- `Player.attack` crit & sweep branches: exact crit conditions (`attackStrengthScale > 0.9f`, `fallDistance > 0.0`, `!onGround`, `!onClimbable`, `!isInWater`, `!hasEffect(BLINDNESS)`, `!isPassenger`, `target instanceof LivingEntity`, `!isSprinting()`), `1.5f` damage multiplier, and sweep AOE (`AABB.inflate(1.0, 0.25, 1.0)`, distance `< 9.0`, `1.0f + SWEEPING_DAMAGE_RATIO * baseDamage`, `0.4` knockback);
- `MaceItem.getAttackDamageBonus` & `hurtEnemy` / `postHurtEnemy`: exact fall distance gate (`canSmashAttack`: `fallDistance > 1.5 && !isFallFlying`), tiered bonus (`4 * d1 + 2 * d2 + 1 * d3` over thresholds `3.0` and `8.0`), `Density` (`0.5 * level * fallDistance`), `WindBurst` enchantment upward impulse, and `resetFallDistance` / `setIgnoreFallDamageFromCurrentImpulse`!

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity`, `BlocksAttacks`, `Player.attack`, and `MaceItem`, plus executing the 26.2 jar across shield angle dots (`-0.01`, `0.0`, `+0.01`, `1.0`), axe disable ticks, crit edge flags, sweep distances, and mace falls (`1.5`, `1.51`, `3.0`, `8.0`, `25.0`), outputting raw hex bits. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` extending `engine/law/src/combat.rs`.

**Falsifier:** any angle dot boundary, crit flag, sweep distance cutoff (`< 9.0`), or mace smash damage bit mismatch against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
