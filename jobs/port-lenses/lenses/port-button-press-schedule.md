# Port delta: ButtonBlock press, neighbor updates and scheduled release tick

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `docs/plan.md` step 5 ("interacting with doors, trapdoors and buttons", `P2a`) and `butoonpressureplates`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ButtonBlock.useWithoutItem` / `press` (to locate) and `ButtonBlock.tick` (to locate): what happens if `POWERED` is already `true` on click, `setBlock` flags, `scheduleTick` delay (`ticksToStayPressed` — exact tick counts for stone vs wooden buttons, to locate), and whether `ButtonBlock.entityInside` re-schedules or holds wooden buttons pressed for arrows/entities;
- `ButtonBlock.updateNeighbours` (to locate): exact directions notified (clicked cell + `getConnectedDirection(state).getOpposite()`, to locate) so adjacent doors/redstone react on the right tick.

**Build a jar-backed reproduction:** run `reproduce.sh` over `ButtonBlock` bytecode and invoke stone vs wooden `ButtonBlock` press/tick transitions directly from the jar, printing `POWERED` state and scheduled release tick (`t + 20` vs `t + 30` or exact jar values) and neighbor-update coordinates. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any stone vs wooden duration, re-click while powered behavior, or neighbor-update target direction that disagrees with `ButtonBlock` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
