# Port delta: FlowingFluid.spread slope distance, dropOff and beforeDestroyingBlock

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`. `port-fluid-tick-delay`, `port-water-source-neighbor-count`, `port-fluid-height-witness`, and `port-fluid-rules` (`canHoldAnyFluid` / `mergedFaceOccludes` / `SimpleWaterloggedBlock`) already settled delays, source count `>=2`, and waterlogging; do not re-ask those.

**Consumer:** `map/now.md` step 5 & step 8 (`P2b`, "water may destroy if repaired" undo check in `PRINCIPLES` S32, `watertunnel`).

**From the 26.2 server bytecode**, state with class, method and offsets:
- `FlowingFluid.tick`, `FlowingFluid.spread`, `FlowingFluid.getNewLiquid`, `FlowingFluid.spreadToSides`, and `FlowingFluid.getSlopeDistance` (to locate): `getSlopeFindDistance` (`4` water / `2` lava / `4` fast-lava), `getDropOff` (`1` water / `2` lava / `1` fast-lava), falling-below priority over horizontal spread (`canSpreadTo`), and `spreadTo` replacing non-source blocks;
- `WaterFluid.beforeDestroyingBlock` and `LavaFluid.spreadTo` / `LiquidBlock.shouldSpreadLiquid` (to locate): which blocks water washes away with drops (`beforeDestroyingBlock -> dropResources` vs air), and exact lava + water contact writes (flowing/source water vs lava -> `COBBLESTONE`, `OBSIDIAN`, `STONE`).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `FlowingFluid.tick`/`spread`/`spreadToSides`/`getSlopeDistance` and `WaterFluid.beforeDestroyingBlock`/`LavaFluid.spreadTo`, plus a direct jar simulation of (1) water flowing down a 1-block drop vs flat channel (`level` sequence `8(s)->7->6...` and slope shortest-path pull toward a hole within 4 blocks), and (2) water flowing onto a torch/redstone vs into lava. Write `dossier.md` starting with `Claim: ` on its first physical line.

**Falsifier:** any slope search depth, drop-off level, falling-over-horizontal priority, or wash-away / stone-obsidian outcome that disagrees with the 26.2 bytecode.

Write only in your run folder. Leave the rig alone; no game launch, network, or production edits.
