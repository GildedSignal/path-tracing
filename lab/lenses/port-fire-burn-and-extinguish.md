# Port delta: burn ticking, water/rain extinguish and fire-block ignition

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-lava-entity-inside` / `port-lava-hurt-and-collector` cover lava contact itself; this lens covers the downstream burn timer (`remainingFireTicks` decrement/damage/extinguish) and `BaseFireBlock`/`CampfireBlock` ignition.

**Consumer:** `engine/law/src/movement.rs:1053` (`remaining_fire_ticks`), `engine/world/tools/ModelTable.java:37` (`BaseFireBlock`, `CampfireBlock` refused), `port-lava-entity-inside` Limits:47.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `Entity.baseTick` (to locate): `remainingFireTicks` decrement (`-1`/tick when `> 0`, vs `+1` up to `0` when `< 0`?), `isInWaterOrRain()` / `isInPowderSnow` extinguish (`EXTINGUISH` / `clearFire`, `playExtinguishSound`, `setRemainingFireTicks(0)` vs `-20`?), and `remainingFireTicks % 20 == 0` -> `hurtServer(damageSources().onFire(), 1.0f)`;
- `BaseFireBlock.entityInside` and `CampfireBlock.entityInside` (to locate): exact `inFire` damage (`1.0f` vs campfire `1.0f`), `FIRE_IGNITE` (`igniteForSeconds(8.0f)` -> `160` ticks?), and `FIRE_RESISTANCE` / `fireImmune` gate.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `Entity.baseTick`, `BaseFireBlock.fireIgnite` / `entityInside`, and `CampfireBlock.entityInside`, plus direct jar calls verifying: (1) `remainingFireTicks` countdown across `21 -> 20 -> 19` (`onFire` damage at `20`), (2) `isInWaterOrRain` clearing fire ticks, and (3) `FIRE` vs `CAMPFIRE` `entityInside` fire ticks & damage. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `% 20` fire-damage phase, tick decrement/extinguish target, or `BaseFireBlock`/`CampfireBlock` damage/ignite constant that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
