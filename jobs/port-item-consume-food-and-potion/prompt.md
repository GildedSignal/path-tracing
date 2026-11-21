# Port delta: item consumption lifecycle (startUsingItem, useItemRemaining, FoodProperties and potions)

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-use-effects-dump` and `port-sprint-food-and-item-reset` already settled `use_item_speed_multiplier` (`0.2f` vs `1.0f`) and `canSprint` while `using_item == true`. This job settles when `using_item` starts, ticks down, aborts, and commits `FoodProperties` / `Consumable` / `PotionContents` writes.

**Consumer:** `engine/law/src/player.rs`, `engine/law/src/movement.rs:1271-1300` (`food_tick`), `map/goal.md` ("Health, food and the inventory are resources spent in T").

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LivingEntity.startUsingItem`, `updatingUsingItem`, `updateUsingItem`, `completeUsingItem`, and `stopUsingItem` (to locate): exact tick phase inside `LivingEntity.tick` (before `aiStep` / `travel`!), `useItemRemaining` initialization (`stack.getUseDuration(this)` -> `Consumable.consumeTicks() = (int)(consumeSeconds * 20.0f)`, default `1.6f -> 32` ticks, dried kelp `0.8f -> 16` ticks), per-tick `--useItemRemaining == 0 -> completeUsingItem()` transition, and `LIVING_ENTITY_FLAG_IS_USING` clear;
- `Consumable.canConsume` (`FoodProperties.canAlwaysEat` vs `FoodData.needsFood()` `foodLevel < 20 || abilities.instabuild`), `FoodProperties.onConsume` -> `FoodData.eat(int, float)` (`foodLevel = min(nutrition + foodLevel, 20)`, `saturationLevel = min(saturationLevel + (float)nutrition * saturationModifier * 2.0f, (float)foodLevel)`), and `ChorusFruitItem` / `TeleportRandomlyConsumeEffect` or `ApplyStatusEffectsConsumeEffect`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LivingEntity` (`startUsingItem`, `updatingUsingItem`, `updateUsingItem`, `completeUsingItem`), `Consumable`, `FoodProperties`, and `FoodData.eat`, plus a jar-backed execution stepping a player eating golden carrots, cooked beef, golden apples (`canAlwaysEat = true` at `food = 20`), and drinking a speed potion from start tick to completion, printing `useItemRemaining`, `using_item`, `foodLevel`, and `saturation` (`f32::to_bits`) each tick. Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust port delta as `work.patch`.

**Falsifier:** any off-by-one tick in `useItemRemaining` completion (`32` ticks), `canAlwaysEat` gate, or `FoodData.eat` `f32` saturation formula that disagrees with the 26.2 jar execution.
