# Port delta: updateShape support-loss pop catalogue across traversable blocks

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-update-order` (`1114-1158-1-0`) already pinned `LadderBlock`; do not re-ask ladders.

**Consumer:** `docs/plan.md` step 5 ("every edit owes its undo (the ledger)"), batch `P2a` (`jobs/port-backlog/report.md`), and `port-update-order`.

**From the 26.2 server bytecode**, state with class, method and offsets:
- scan all block subclasses in `net.minecraft.world.level.block` overriding `updateShape` (to locate): identify which movement/traversability blocks call `Blocks.AIR.defaultBlockState()` / `Block.updateOrDestroy` synchronously on support loss (`TorchBlock`, `WallTorchBlock`, `CarpetBlock`, `SnowLayerBlock`, `BasePressurePlateBlock`, `ButtonBlock`, `DoorBlock`, `ScaffoldingBlock`, `RailBlock` / `BaseRailBlock`, `VineBlock`, `HangingRootsBlock`, etc., to locate) vs scheduling a delayed tick (like `ScaffoldingBlock.tick` / `FallingBlock.tick`);
- for `DoorBlock.updateShape` and `ScaffoldingBlock.updateShape` (to locate): state the exact `canSurvive` / `distance` / `UPDATE_SHAPE` rule when the supporting block below is broken.

**Build a jar-backed reproduction:** run `reproduce.sh` printing the machine-checked census of `updateShape` support-loss pop blocks with class, method, and offsets, plus direct jar `updateShape` witnesses on a broken support under (1) `DoorBlock` (`LOWER` + `UPPER` pop), (2) `CarpetBlock` / `PressurePlateBlock`, and (3) `ScaffoldingBlock` (`distance` / `scheduleTick`). Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any block in the census that pops synchronously vs schedules (or fails to pop) differently from direct jar `updateShape` execution.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
