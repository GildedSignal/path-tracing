# Vibe generator rows: consumables, chorus fruit, totem, and status effects

**Goal:** One parameter/scar table for 26.2 item use (`Consumable`, food eating durations `16/32/40`, nutrition/saturation, chorus fruit random-bounded teleport, `Totem of Undying` death save, milk/honey clear) and `MobEffect` ticks executed with bound `DataComponent`s against `server-26.2.jar`.

Feeds `wt/port-fold`'s `clock`, `ledger`, `attribute`, and wormhole inventories (`jobs/law-grouping-audit/table.md` groups 7, 18, 20, 24).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) AND `DataComponentInitializers` (`1537/1537` item holders bound) so real `ItemStack(Items.GOLDEN_APPLE/COOKED_BEEF/DRIED_KELP/HONEY_BOTTLE/CHORUS_FRUIT/TOTEM_OF_UNDYING/MILK_BUCKET/SPLASH_POTION)` construct and finish `finishUsingItem`.
- Use real `ServerLevel` + real `LevelChunk` + real `ServerPlayer`/`LivingEntity`.
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after core executed runs pass, then extend cleanly.

**From bytecode + live execution:**
1. **Food & drink components:** exact `useDuration` ticks, movement speed penalty while `usingItem` (`0.2f`), nutrition/saturation formula `min(food + n, 20)`, `min(sat + n * mod * 2, food)`, golden apple / enchanted golden apple effect application.
2. **Chorus fruit teleport (`ConsumeEffect` / `TeleportRandomlyConsumeEffect`):** `±8.0` x/z range, `±8` y search, `16` attempts, solid-below check, `cooldown 20` ticks, fall-distance reset check.
3. **Totem of Undying (`LivingEntity.checkTotemDeathProtection`):** `setHealth(1.0f)`, `removeAllEffects()`, regeneration II (`900`), absorption II (`100`), fire resistance I (`800`), works even against void/out-of-world (`BYPASSES_INVULNERABILITY` check!).
4. **Status effects (`MobEffectInstance.tickServer`, `MobEffect` attributes):** `SPEED`/`SLOWNESS` (`+20%`/`-15%` multiply_total), `JUMP_BOOST` (`+0.1f * (amp + 1)`), `REGENERATION`/`POISON`/`WITHER` interval `(50 >> amp)` / `(25 >> amp)` / `(40 >> amp)`, `LEVITATION` (`0.05 * (amp + 1) - vy) * 0.2`), `SLOW_FALLING` (`0.01`, `fallDistance = 0`).

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	duration_or_period	params_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	health_hex	food	saturation_hex	x_hex_or_speed	vy_hex	flags	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any divergence in use duration, post-eat food/saturation bits, totem effects/durations, or effect tick cadence/attributes against `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
