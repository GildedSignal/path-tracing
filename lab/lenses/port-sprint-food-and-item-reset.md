# Port delta: double-tap sprint timer reset on item use and food gate

Use the staged Mojang 26.2 client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, the server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-sprint-window` already settled `Options.sprintWindow` and basic `sprintTriggerTime` double-tap countdown; do not re-ask those.

**Consumer:** `docs/HANDOFF.md:44` Anomaly 10 (`LocalPlayer.aiStep:348` resetting `sprintTriggerTime` through `isSlowDueToUsingItem`), `engine/law/src/movement.rs:217-233`, and `trial-food-crossing` / `parkour_race`.

**From the 26.2 client bytecode**, state with class, method and offsets:
- every write to `LocalPlayer.sprintTriggerTime` (to locate) across the whole `LocalPlayer` class: specifically `isSlowDueToUsingItem` (to locate), `hasEnoughFoodToStartSprinting` (`food > 6` vs `hasEnoughImpulseToStartSprinting` `0.8f`), `isUsingItem`, `isUnderWater`, and `isPassenger`;
- exact order inside `LocalPlayer.aiStep` (to locate) between `sprintTriggerTime` reset/decrement, `setSprinting(true)`, and `isSlowDueToUsingItem` when starting to use an item (or switching items) on the same tick as a double-tap forward press;
- whether `food_level <= 6` in Creative (`mayfly` / `instabuild` / `hasEnoughFoodToStartSprinting`, to locate) bypasses the `food > 6` sprint gate (`movement.rs:219`).

**Build a jar-backed reproduction:** run `reproduce.sh` listing every write offset to `sprintTriggerTime` in `LocalPlayer` and `hasEnoughFoodToStartSprinting`, plus a jar-backed test of (1) a 2-tick double-tap interrupted by `isSlowDueToUsingItem = true` (bow/food vs spear from `UseEffects`), and (2) `foodLevel = 6` in Survival vs Creative (`mayfly = true`), printing `sprintTriggerTime` and `isSprinting()` each tick. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any missed `sprintTriggerTime` writer or Creative `food <= 6` sprint outcome that disagrees with `LocalPlayer` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
