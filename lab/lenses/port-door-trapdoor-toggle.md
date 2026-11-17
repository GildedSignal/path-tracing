# Port delta: DoorBlock, TrapDoorBlock and FenceGateBlock useWithoutItem toggle

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. This replaces the unfinished `port-door-interaction` with only the right-click toggle law (`ServerPlayerGameMode.useItemOn` dispatch and pressure plates are separate lenses).

**Consumer:** Julien's tracks `doorcorridor`, `doorparkourblue`, `trapdoorparkour`, `amertystcoral` (`jobs/flight-corpus/report.md`), and `docs/plan.md` step 5 (`P2a`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `DoorBlock.useWithoutItem` (and `setOpen`, to locate), `TrapDoorBlock.useWithoutItem` (and `toggle`, to locate), and `FenceGateBlock.useWithoutItem` (to locate): every gate (`BlockSetType.canOpenByHand`, iron doors/trapdoors vs wooden/copper), which property (`OPEN`) flips, and which `setBlock` flags are used;
- for `DoorBlock`: which half (`HALF = LOWER / UPPER`, to locate) is written directly vs via `setBlock` / `updateShape`, and for `TrapDoorBlock`: whether toggling checks or avoids entity collision when closing onto the player's own AABB (`amertystcoral`).

**Build a jar-backed reproduction:** drive the jar's `DoorBlock`, `TrapDoorBlock`, and `FenceGateBlock` toggle logic (or `reproduce.sh` bytecode + direct jar invocation) showing: (1) wooden/copper door toggle writing both halves and toggling back bit-for-bit, (2) iron door rejection, and (3) trapdoor closing while a player box overlaps its cell. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any missed `canOpenByHand` gate, wrong `setBlock` flag, single-half door write, or wrong collision-overlap rule on trapdoor close.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
