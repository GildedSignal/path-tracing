# Vibe generator rows: inventory slot fill, stack merge, durability, and attribute equip

**Goal:** Bit- and server-exact parameter/scar table for 26.2 inventory mechanics (`Inventory`, `ItemStack` merge/stack caps, tool/armor durability loss + Unbreaking formula, attribute slot equip modifiers, offhand swap) executed with bound `DataComponent`s against `server-26.2.jar`.

Supplies Group 24 (`Inventory`) of `jobs/law-grouping-audit/table.md` with executed witnesses (`1537/1537` item holders bound).

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed witnesses):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `ServerPlayer` (`Inventory`, `Equipment`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after first runs pass, then extend cleanly.

**From bytecode + live execution:**
1. **`Inventory.add` & stacking (`ItemStack.isSameItemSameComponents`, max stack size `64 / 16 / 1`):** exact hotbar `0..8` then `9..35` order, partial fill -> remainder slot, pickup merge rules.
2. **Durability & Unbreaking (`ItemStack.hurtAndBreak`, `EnchantmentHelper.processDurabilityChange`):** armor Unbreaking `60 + 40/(lvl+1)%` vs tool `1/(lvl+1)` chance, break sound/zero stack, `ServerPlayer` break callback.
3. **Equipment attribute folds (`ItemAttributeModifiers`, `EquipmentSlot`):** equipping `DIAMOND_BOOTS` / `LEATHER_BOOTS` / `TURTLE_HELMET` / `SWIFT_SNEAK` / `DEPTH_STRIDER` / `SOUL_SPEED` updates `AttributeMap` + `Equipment` readers (`leather_boots` powder snow, `depth_strider`, `sneaking_speed`).

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	slot_or_limit	constants_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	slot	item	count	damage	attr_bits_hex	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any executed slot count, durability decrease, or equipped attribute hex bits differing from `server-26.2.jar`.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
