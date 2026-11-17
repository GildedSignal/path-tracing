# Port delta: LeverBlock toggle and redstone-driven Door/TrapDoor/FenceGate neighborChanged

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-door-trapdoor-toggle` and `port-button-press-schedule` already settled hand-toggling and buttons; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 (`P2a`, iron doors, levers, pressure-plate/button-opened doors and trapdoors).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `LeverBlock.useWithoutItem` and `LeverBlock.pull` (to locate): `POWERED` cycle, `setBlock` flags, and which neighbor cells receive `updateNeighboursAt` (`pos` and support cell via `getConnectedDirection`);
- `DoorBlock.neighborChanged`, `TrapDoorBlock.neighborChanged`, and `FenceGateBlock.neighborChanged` (to locate): whether `hasNeighborSignal` (checking both halves for `DoorBlock`!) toggles `OPEN` on `POWERED != hasSignal` (edge-driven) vs sets `OPEN = hasSignal`, exact `setBlock` flags (`2` vs `10`?), and whether iron doors/trapdoors share the exact same `neighborChanged` path as wooden/copper ones;
- whether `DoorBlock.neighborChanged` runs inside the player's own tick or deferred.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `LeverBlock.useWithoutItem`/`pull` and `DoorBlock`/`TrapDoorBlock`/`FenceGateBlock.neighborChanged`, and execute direct jar calls for (1) lever click on/off, (2) iron door receiving redstone signal on upper vs lower half (including a pre-opened copper/oak door receiving and losing signal), and (3) iron trapdoor receiving/losing signal, printing `OPEN`, `POWERED`, and `setBlock` flags. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `setBlock` flag, neighbor-update cell, edge-vs-level `OPEN` rule, or iron/wood difference that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
