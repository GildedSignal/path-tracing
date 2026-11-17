# Port delta: BoatItem.use placement and AbstractBoat breaking

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-boat-delta`, `port-boat-land-and-collision`, and `port-boat-mount-dismount` already settled boat physics and mounting; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 & step 8 (`P4a`, "keep what you have" placing a boat, crossing water/ice, breaking and picking it back up).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BoatItem.use` (to locate): raycast `ClipContext.Fluid.ANY` (water surface hit vs block!), entity-collision eye-ray check (`getEntities` `isPickable` obstructing placement?), boat spawn position `(hitLocation.x, hitLocation.y, hitLocation.z)` and `yRot = player.getYRot()`, `level.noCollision(boat, boat.getBoundingBox())`, and stack decrement (`!hasInfiniteMaterials`);
- `VehicleEntity.hurtServer` / `AbstractBoat` damage & break (to locate): `hurtDir`, `damage` decay `-1`/tick, `damage + attackerDamage > 40.0f` vs Creative `isCreative()` instant break, `dropItem` (`getDropItem`) and `player` pickup/recovery.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `BoatItem.use` and `VehicleEntity.hurtServer`/`tick` (to locate), plus direct jar calls verifying (1) `BoatItem.use` raycast hitting water vs stone (`ClipContext.Fluid.ANY`) and `noCollision` check on `1.375 x 0.5625` box, and (2) `VehicleEntity` damage accumulation to break threshold (`> 40.0f` vs Creative instant). Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any fluid raycast mode, spawn-box collision check, or boat break threshold that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
