# Port delta: BucketItem water placement and pickup

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `docs/plan.md` step 5 (editing the world / undo ledger), `docs/PRINCIPLES.md` #5 & #20 (water bucket climb and picking water back up cleanly), and `jobs/port-backlog/report.md` #59 (`item-use-writes`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `BucketItem.use` (to locate): raycast call (`Item.getPlayerPOVHitResult` with `ClipContext.Fluid.SOURCE_ONLY` when empty vs `NONE` when filled, to locate), range/permission gates;
- `BucketItem.emptyContents` (to locate): waterloggable `LiquidBlockContainer` (`canPlaceLiquid` / `placeLiquid`) vs placing `WATER` source at `hitPos` (if replaceable) vs `hitPos.relative(face)`, `setBlock` flags, fluid `scheduleTick`, and `SimpleWaterloggedBlock` interaction;
- `BucketPickup.pickupBlock` (on `LiquidBlock` and `SimpleWaterloggedBlock`, to locate): exact water source pickup, `WATERLOGGED=false` vs `AIR`, and item swap (`WATER_BUCKET` <-> `BUCKET`).

**Build a jar-backed reproduction:** run `reproduce.sh` over `BucketItem.use` / `emptyContents` / `pickupBlock` bytecode and execute jar-backed place-then-pickup witnesses on (1) an air face above stone, and (2) a waterloggable slab/trapdoor, confirming identical state restoration. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any raycast fluid-mode, waterloggable target-cell selection, `setBlock` flag, or item-swap mismatch against the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
