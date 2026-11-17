# Port delta: pressure plate overlap, press tick and release schedule

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged JDK/javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** Julien's track `butoonpressureplates` (`jobs/flight-corpus/report.md`), `docs/plan.md` step 5 (interacting with world hooks), and `docs/HANDOFF.md` (first-flight 198t vs 70t planner slowdown).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BasePressurePlateBlock.entityInside` and `BasePressurePlateBlock.checkPressed` (and concrete wood/stone/weighted plate overrides, to locate): the exact entity bounding-box overlap test (`getInteractionShape` or AABB bounds, to locate), which entity types trigger it, and whether any plate modifies the overlapping player's velocity, pose, or resources;
- the signal calculation (`getSignalForState`, `setSignalForState`, to locate), `setBlock` update flags, and adjacent neighbor updates (`updateNeighbours`, to locate);
- the scheduled release tick (`getPressedTime`, `scheduleTick`, `BasePressurePlateBlock.tick`, to locate): exact delay in ticks for wooden vs stone vs weighted plates, and how continuous overlap re-schedules vs releases.

**Build a jar-backed reproduction:** drive the plate's press and release methods directly from the 26.2 server jar (or print bytecode + runnable `reproduce.sh` witness) showing: (1) tick of entry (state & scheduled tick delay), (2) tick after leaving (exact tick when `POWERED` / signal clears), and (3) confirmation that player velocity is untouched. Write `dossier.md` starting with `Claim: ` on its first physical line and `reproduce.sh` in the run folder.

**Falsifier:** any claim that a pressure plate changes player collision/velocity, or a release tick delay / overlap box that disagrees with a direct jar run.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
