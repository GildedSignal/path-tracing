# Port delta: ServerPlayerGameMode.destroyBlock side effects and waterlogged cleanup

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-breaking-threshold`, `port-break-stop-boundary`, `port-mining-destroy-speed`, and `port-update-order` already settled progress and `Level.setBlock`; do not re-ask those.

**Consumer:** `docs/plan.md` step 5 ("leave no damage" edit/undo ledger, `engine/law/src/mining.rs` `DestroyIntent` commit, `P2a`/`P2c`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `ServerPlayerGameMode.destroyBlock(BlockPos)` (to locate): exact call order of `Block.playerWillDestroy`, `Level.removeBlock` / `setBlock` (`getFluidState().createLegacyBlock()` — i.e., whether breaking a waterlogged block leaves a water source or air!), `Block.destroy`, `ItemStack.mineBlock` (tool durability cost + `hasCorrectToolForDrops` / `playerDestroy`), and `Player.causeFoodExhaustion(0.005f)` (to locate);
- `DoorBlock.playerWillDestroy` / `preventDropFromBottomPart` (to locate): how breaking either half of a door removes the other half in Survival vs Creative (`setBlock` flags, `35` vs `3`?), and whether any other multi-block (`BedBlock`?) does the same;
- Creative vs Survival branch inside `destroyBlock` (or `isCreative` early return).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ServerPlayerGameMode.destroyBlock`, `Level.removeBlock`, `Block.playerWillDestroy`, `Block.playerDestroy`, and `DoorBlock.playerWillDestroy`, plus direct jar calls checking (1) a waterlogged slab/trapdoor broken (resulting `BlockState` = water source vs air), (2) `DoorBlock` lower/upper break in Creative vs Survival, and (3) exhaustion `0.005f` added per broken block. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any post-break `BlockState` (waterlogged or door partner), `setBlock` flag, or exhaustion increment that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
