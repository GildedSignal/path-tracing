# Generator rows: damage mitigation ledger, food chain, and melee resolution

**Goal:** One parameter/scar table for 26.2 `ledger` mechanics (`accept_raw_hit`, armor/toughness/resistance/EPF/absorption chain, food/regeneration/starvation across difficulties 0–3, and melee crit/sweep/shield/mace smash), backed by bit-exact jar witnesses.

Supports the `ledger` generator (`map/now.md`, `jobs/law-grouping-audit/table.md` group 7) with one verified row/witness table across damage types and food transitions.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Bind real jar tags via `MappedRegistry.prepareTagReload(...).apply()` (`data/minecraft/tags/**/*.json`).
- Use real `LocalPlayer` (or override `isLocalPlayer() => true` and `isLocalClientAuthoritative() => true`) for client movement ticks and real `ServerPlayer` / `LevelChunk` (`hasFluid()`, sections) on server mechanics.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early as soon as core bytecode + primary witnesses pass, then extend cleanly so no timeout leaves an empty dossier.

**From the bytecode** (`class.method@offset`):
1. **`LivingEntity.hurtServer` / `actuallyHurt` / `getDamageAfterArmorAbsorb` / `getDamageAfterMagicAbsorb` / `CombatRules`:** invulnerability window (`invulnerableTime > 10` excess over `lastHurt` vs `<= 10` full hit), armor + toughness formula, Resistance potion `5 * (amp + 1)` / `25`, enchantment EPF cap `20`, absorption subtraction order, and damage-type flags (`BYPASSES_ARMOR`, `BYPASSES_EFFECTS`, `BYPASSES_ENCHANTMENTS`, `BYPASSES_SHIELD`, `IS_FIRE`, `IS_FALL`, exhaustion).
2. **`FoodData.tick` across difficulties 0–3:** exhaustion `>= 4.0f` drain (`saturation -= 1.0f` or `food -= 1`), fast regen (`food >= 20 && saturation > 0`, `>= 10` ticks), slow regen (`food >= 18`, `>= 80` ticks), starvation thresholds (`food == 0`: difficulty 1 stops at `10.0f`, difficulty 2 stops at `1.0f`, difficulty 3 kills to `0.0f`), peaceful regen (`food` + `health`).
3. **Melee resolution (`Player.attack`):** base attack strength scale, `0.2 + ((s + 0.5)/T)^2 * 0.8`, sprint knockback + extra knockback, `1.5f` critical multiplier (fall > 0, not grounded, not climbing/in-water/blind/passenger), sweep edge damage, shield block arc (`cosine`) + axe `5.0s` / `100` tick disable, and mace `getAttackDamageBonus` smash tiers.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	difficulty_or_type	params_hex	formula_order	scars	source`
- `witnesses.tsv`: `case_id	tick	health_hex	absorption_hex	food	saturation_hex	exhaustion_hex	last_hurt_hex	hurt_time	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick where the row model emits different post-hit health/absorption/food hex bits or invulnerability branching than the 26.2 jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
