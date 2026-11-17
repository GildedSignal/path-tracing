# Port delta: placement of doors, ladders and scaffolding

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-block-place-commit` and `port-block-support-pop-family` already settled `BlockItem.place` and `updateShape`; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 (multi-block placement & scaffolding climbing/bridging, `carpetwater` `scaffolding:3`, `doorcorridor`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `DoorBlock.getStateForPlacement` and `DoorBlock.setPlacedBy` (to locate): exact upper-cell clearance (`pos.above().canBeReplaced` + `pos.y < maxY`), `HINGE` calculation (`left`/`right`), `FACING`, `POWERED`/`OPEN`, and the `setBlock` flags used to place the upper half;
- `LadderBlock.getStateForPlacement` and `LadderBlock.canSurvive` (to locate): face selection (`replacingClickedOnBlock` vs opposite directions), `isFaceSturdy` check, and `WATERLOGGED`;
- `ScaffoldingBlock.getStateForPlacement` and `ScaffoldingBlockItem.updatePlacementContext` (to locate): side-extension when `!isDescending` vs normal place, `DISTANCE`, `BOTTOM`, `WATERLOGGED`, and whether `ScaffoldingBlock.getStateForPlacement` schedules a `1`-tick update.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling those placement methods and invoke them directly for: (1) `OAK_DOOR` with solid vs replaceable `pos.above()` and left/right hinge neighbors, (2) `LADDER` on sturdy vs non-sturdy faces, and (3) `SCAFFOLDING` with `isDescending = true/false` on ground vs floating-next-to-column, printing the placed `BlockState`s and `setBlock` flags. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any hinge side, upper-half flag, ladder support face, or scaffolding side-extension rule that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
