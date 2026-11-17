# Port delta: remaining ModelTable block hooks (berry bush, cactus, farmland, dripstone)

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-block-laws` already settled `WebBlock`, `PowderSnowBlock`, `HoneyBlock`, `BubbleColumnBlock`, and `MagmaBlock` (`stepOn`); do not re-ask those.

**Consumer:** `engine/world/tools/ModelTable.java:36-41` (`HOOKS` refusals on unmodeled `entityInside`, `stepOn`, and `fallOn` overriders), batch `P3a`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `SweetBerryBushBlock.entityInside` (to locate): `makeStuckInBlock(0.8f, 0.75, 0.8f)` condition (`AGE > 0`?), movement threshold (`0.003` dx/dz delta via `oldPosition` vs `xOld`!), and `hurtServer(sweetBerryBush(), 1.0f)`;
- `CactusBlock.entityInside` (to locate): `hurtServer(cactus(), 1.0f)` and its `getEntityInsideCollisionShape` vs `getCollisionShape`;
- `FarmlandBlock.fallOn` and `PointedDripstoneBlock.fallOn` (to locate): `fallDistance` / `causeFallDamage` multiplier (`2.0f` + `2.0f` for pointed dripstone!) and `FarmlandBlock` trample threshold (`fallDistance - 0.5f > random` -> `turnToDirt`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `SweetBerryBushBlock.entityInside`, `CactusBlock.entityInside`, `FarmlandBlock.fallOn`, and `PointedDripstoneBlock.fallOn` (plus listing any remaining `HOOKS` overriders in `ModelTable.java:36-41`), and invoke `PointedDripstoneBlock.fallOn` + `SweetBerryBushBlock.entityInside` directly on a `Player` printing exact damage/stuck constants. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any stuck multiplier, berry movement threshold, cactus damage, or dripstone/farmland fall formula that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
