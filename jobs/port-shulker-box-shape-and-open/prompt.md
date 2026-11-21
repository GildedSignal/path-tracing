# Port delta: ShulkerBoxBlock collision shape, open obstruction and item-preserving break

**Goal:** the jar's exact rule for this, with a reproduction that runs against Mojang's 26.2 jar: `dossier.md` (first line `Claim: `), `reproduce.sh`, and the Rust port delta as `work.patch`.

Use the staged Mojang 26.2 server jar at `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar` and staged javap at `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Consumer:** `engine/world/tools/ModelTable.java:46,110` (`non-local shape` refusal on all 17 `ShulkerBoxBlock` blocks in `engine/world/data/census.tsv`), `engine/law/src/interact.rs`, `engine/law/src/place.rs`, `map/now.md` delta 1b.

**From the 26.2 server bytecode**, state with class, method and offsets:
- `net.minecraft.world.level.block.ShulkerBoxBlock.getShape` and `net.minecraft.world.level.block.entity.ShulkerBoxBlockEntity.getBoundingBox(BlockState)` (to locate): exact collision shape when `getBlockEntity(pos)` is `null` or `AnimationStatus.CLOSED` (`Shapes.block()` `[0,0,0]..[1,1,1]`?) vs open/opening (`progress` -> `0.5f * progress` extension along `FACING` via `Shulker.getProgressAabb`?);
- `ShulkerBoxBlock.useWithoutItem` and `canOpen(BlockState, Level, BlockPos, ShulkerBoxBlockEntity)` (to locate): exact `noCollision` check on the `0.5`-extended physical box (`getProgressDeltaAabb(1.0f, facing, 0.0f, 0.5f)`deflated by `1.0E-7`?) when `AnimationStatus == CLOSED`;
- `ShulkerBoxBlock.getStateForPlacement` (`FACING = ctx.getClickedFace()`) and `ShulkerBoxBlock.playerWillDestroy`: exact Creative vs Survival item-retention rule (`saveToItem` / `CONTAINER` component drop even when broken in Creative if non-empty!).

**Build a jar-backed reproduction:** run `reproduce.sh` disassembling `ShulkerBoxBlock` (`getShape`, `useWithoutItem`, `canOpen`, `getStateForPlacement`, `playerWillDestroy`) and `ShulkerBoxBlockEntity` (`getBoundingBox`, `tick`, `updateAnimation`), plus a jar-backed execution printing: (1) `getCollisionShape` with `null` block entity vs closed vs open across all 6 `Direction`s, (2) `canOpen` against a block or entity in the 0.5 extension cell, and (3) what change in `ModelTable.java` (`LOCAL.getBlockEntity` returning `null` vs refusing `MovingPistonBlock`) models all 102 shulker-box states in `census.tsv`. Write `dossier.md` starting with `Claim: ` on its first physical line, and produce the Rust/Java port delta as `work.patch`.

**Falsifier:** any closed/open bounding box, `canOpen` obstruction box, or `ModelTable` state count that disagrees with the 26.2 bytecode.
