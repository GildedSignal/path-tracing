# Port delta: ItemEntity tick/merge/pickup, Inventory slot fill order, drop and Unbreaking

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/law/src/items.rs`, `map/now.md` (inventory, drops, tool durability, resource accounting).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ItemEntity.tick`: exact gravity (`0.04`), air drag (`0.98`), ground friction (`getFriction() * 0.98` vs player's `* 0.91`!), water/lava float acceleration (`setUnderwaterMovement` / `setUnderLavaMovement`), block push (`checkInBlock` -> `moveTowardsClosestSpace`), `pickupDelay` decrement (`10` on player drop vs `40` throw? verify!), `6000`-tick lifetime despawn, and `mergeWithNeighbours` (`AABB.inflate(0.5, 0.0, 0.5)`, `areMergable` / `tryToMerge` stack merge + age = `min`);
- `ItemEntity.playerTouch(Player)`: `pickupDelay == 0`, owner lock at `age`, `inventory.add(itemStack)`, partial pickup remainder;
- `Inventory.add(int, ItemStack)` & `getFreeSlot` / `getSlotWithRemainingSpace`: hotbar (`0..9`) then main (`9..36`) vs offhand (`40`), stack size cap, and `Player.drop(ItemStack, boolean, boolean)` spawn origin/velocity (`0.3` drop / `0.4` throw);
- `ItemStack.hurtAndBreak` / `EnchantmentHelper.processDurabilityChange`: exact Unbreaking (`1 / (level + 1)` vs armor `0.6 + 0.4 / (level + 1)`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ItemEntity`, `Inventory`, `Player.drop`, and `ItemStack.hurtAndBreak`, plus executing 26.2 jar multi-tick `ItemEntity` motion on stone/ice/water, merge rules, inventory slot fills, and Unbreaking durability checks, printing raw hex bits. Write `dossier.md` starting with `Claim: ` on its first physical line and `work.patch` extending `engine/law/src/items.rs`.

**Falsifier:** any 1-ulp disagreement in `ItemEntity` drag/gravity, wrong pickup slot index, wrong `pickupDelay`/lifetime constant, or Unbreaking formula mismatch against the 26.2 jar.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
