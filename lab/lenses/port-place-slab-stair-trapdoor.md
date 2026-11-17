# Port delta: Block.getStateForPlacement for slabs, stairs and trapdoors

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-block-place-commit` already settled `BlockItem.place` / `canPlace` / `isUnobstructed`; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 (world-editing placement words, `trapdoorparkour`, `stonestaircase`, `jobs/port-lenses/report.md:52`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `SlabBlock.getStateForPlacement` (to locate) and `SlabBlock.canBeReplaced` (to locate): how `TOP`, `BOTTOM`, and `DOUBLE` are chosen from `clickedFace` (`UP`/`DOWN`/horizontal) and `clickLocation.y - pos.y > 0.5` (strict vs non-strict), and how `WATERLOGGED` is initialized;
- `StairBlock.getStateForPlacement` (to locate): `HALF` (`TOP`/`BOTTOM`), `FACING`, `WATERLOGGED`, and initial `SHAPE` (`getStairsShape`, to locate);
- `TrapDoorBlock.getStateForPlacement` (to locate): `FACING`, `HALF` (`TOP`/`BOTTOM` from horizontal-face `clickLocation.y - pos.y > 0.5` vs `UP`/`DOWN`), `OPEN` / `POWERED` (redstone neighbor check on placement?), and `WATERLOGGED`.

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling all three `getStateForPlacement` methods and `SlabBlock.canBeReplaced`, and invoke them directly on `OAK_SLAB`, `STONE_STAIRS`, and `OAK_TRAPDOOR` across `clickLocation.y - pos.y ∈ {0.5, nextUp(0.5), nextDown(0.5)}`, faces `{UP, DOWN, NORTH}`, and waterlogged target cells, printing the resulting `BlockState` properties. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any `HALF`, `TYPE`, `FACING`, `SHAPE`, or `WATERLOGGED` property or `0.5` strictness boundary that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
