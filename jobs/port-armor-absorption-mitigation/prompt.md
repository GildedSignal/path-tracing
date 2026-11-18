# Port delta: armor, Feather Falling / Protection EPF, and absorption mitigation

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-contact-cooldown` and `port-regen-dead-guard-and-phase` already settled raw cooldown debit and regen; do not re-ask those.

**Consumer:** `engine/law/src/movement.rs:848` (`check_fall_damage`), `docs/PRINCIPLES.md` ("health is a resource"), `port-lava-entity-inside`, `port-pearl-raytrace-and-landing`, `port-elytra-firework-and-impact`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.actuallyHurt` (to locate): exact call order of `getDamageAfterArmorAbsorb(DamageSource, float)` -> `getDamageAfterMagicAbsorb(DamageSource, float)` -> absorption subtraction (`setAbsorptionAmount` vs `getAbsorptionAmount`, `hurt`-exhaustion `getFoodExhaustion()`) -> `setHealth(getHealth() - damage)`;
- `CombatRules.getDamageAfterAbsorb` and `CombatRules.getDamageAfterMagicAbsorb` (to locate): exact `f32` armor + armor-toughness formula (`min(20.0f, max(armor / 5.0f, armor - damage / (2.0f + toughness / 4.0f))) / 25.0f`?), `BYPASSES_ARMOR` / `BYPASSES_ENCHANTMENTS` tag checks (specifically for `FALL`, `ENDER_PEARL`, `FLY_INTO_WALL`, `LAVA`, `FREEZE`), and EPF cap `Mth.clamp(epf, 0, 20)` (`1.0f - epf / 25.0f`) including Feather Falling IV (`3 * level = 12` vs Protection `1 * level`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity.actuallyHurt`, `getDamageAfterArmorAbsorb`, `getDamageAfterMagicAbsorb`, `CombatRules`, and `data/minecraft/tags/damage_type/{bypasses_armor,bypasses_enchantments}.json`, plus direct jar calls computing post-mitigation `f32` health bits for `ENDER_PEARL` (`5.0f`), `FALL` (`10.0f`), and `LAVA` (`4.0f`) under diamond armor + Feather Falling IV + Protection IV + 4.0f absorption. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `f32` mitigation formula, EPF cap, damage-type bypass tag, or absorption subtraction order that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
