# Port delta: TripWireBlock and TripWireHookBlock trigger and reset

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-pressure-plate-hook` and `port-button-press-schedule` already settled plates and buttons; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 (`world_hook` completion), `engine/world/tools/ModelTable.java:38` (`TripWireBlock.class -> "world"`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `TripWireBlock.entityInside` and `TripWireBlock.checkPressed` (to locate): whether it changes any entity motion/pose or only block states, exact `AABB` / entity filter (`isIgnoringBlockTriggers`?), `setBlock` flags (`18` / `4`?), and `scheduleTick(pos, this, 10)` delay;
- `TripWireBlock.tick` and `TripWireBlock.updateSource` -> `TripWireHookBlock.calculateState` (to locate): scan distance (`42`?), `ATTACHED`/`POWERED` updates on hooks & wire, and neighbor notifications;
- `TripWireBlock.playerWillDestroy` (to locate): disarming with shears (`Items.SHEARS`) vs triggering when broken without shears (`DISARMED` flag).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `TripWireBlock` (`entityInside`, `checkPressed`, `tick`, `updateSource`, `playerWillDestroy`) and `TripWireHookBlock.calculateState`, plus a jar-backed execution of a player entering a hooked tripwire line (`[hook - wire - wire - hook]`) and leaving it, printing all block state writes, `setBlock` flags, scheduled-tick delays, and player velocity/pose invariance. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any delay other than the bytecode's, any entity velocity/pose mutation, or any hook/wire state/flag difference from `TripWireBlock` / `TripWireHookBlock` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
