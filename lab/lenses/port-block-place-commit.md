# Port delta: BlockItem.place, entity obstruction and stack decrement

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `place-reach-legality` and `port-place-replaceability` already settled reach and `BlockPlaceContext.replaceClicked`; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 (`P2c`), Julien's tracks `mud` (`itemsUsed:{"minecraft:mud":2}`), `ametwoodbox` (`oak_log:1`), and "leave no damage" inventory accounting.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BlockItem.place(BlockPlaceContext)` (to locate) through `getPlacementState`, `canPlace` (`BlockItem.canPlace`, `CollisionContext.of(player)`, `LevelReader.isUnobstructed`, to locate), `placeBlock` (`Level.setBlock` flags), `updateCustomBlockEntityTag` / `onPlace` / `setPlacedBy`, and item stack decrement (`consume` / `shrink` gated on Creative `hasInfiniteMaterials`, to locate);
- specifically verify whether `isUnobstructed(state, pos, CollisionContext.of(player))` blocks placing a solid block when the placing player's AABB overlaps the target cell (feet cell vs cell below feet vs grazing edge `1e-7`!).

**Build a jar-backed reproduction:** run `reproduce.sh` over `BlockItem.place`/`canPlace` bytecode and execute a jar-backed placement test for a solid block (`mud` / `oak_log`) at: (1) the player's feet cell (obstructed), (2) the cell directly below feet (allowed), and (3) a face-adjacent cell grazing the player AABB, reporting `setBlock` success and remaining stack count. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any obstruction outcome, `setBlock` flag, or stack-decrement condition that disagrees with `BlockItem.place` bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
