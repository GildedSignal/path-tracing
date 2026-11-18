# Port delta: AbstractBoat fell-into-water snap and ServerboundPaddleBoatState

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`, the client jar at `D:/path-tracing-deps/minecraft-26.2-client.jar`, and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-boat-land-and-collision` left `floatBoat @13-135` bytecode-cited only; do not re-ask `ON_LAND`/`IN_AIR`/`UNDER_WATER` or `IN_WATER` drag.

**Consumer:** `map/now.md` step 8 (`P4a`, water-entry splash/snap transition and client-server boat packet ordering).

**From the 26.2 server and client bytecode**, state with class, method and offsets:
- `AbstractBoat.floatBoat` (`@13-135`): exact condition (`oldStatus == IN_AIR && status != IN_AIR && status != ON_LAND`), `waterLevel = getY(1.0)`, `target = (getWaterLevelAbove() - bbHeight) + 0.101`, `noCollision(box.move(0, target - y, 0))` Y-snap, `setDeltaMovement(vx, 0, vz)`, `lastYd = 0`, `status = IN_WATER`, and early return skipping drag that tick;
- `AbstractBoat.getWaterLevelAbove` (to locate) and `LocalPlayer.rideTick` -> `AbstractBoat.setInput` -> `ServerboundPaddleBoatPacket` / `ServerGamePacketListenerImpl.handleMoveVehicle` (to locate): how client boat inputs reach `controlBoat` and whether the server validates boat speed/position in `handleMoveVehicle`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `AbstractBoat.floatBoat` (`@0-140`), `getWaterLevelAbove`, `setInput`, and `handleMoveVehicle`, and execute a real `Boat.tick()` falling from `IN_AIR` (`oldStatus = IN_AIR`) into water both with clear ceiling (`noCollision = true`) and blocked ceiling (`noCollision = false`), printing raw `y` and `v` bits before/after the snap tick. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `target` formula, early-return drag skip, or `noCollision`-blocked fallback that disagrees with `AbstractBoat.floatBoat` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
