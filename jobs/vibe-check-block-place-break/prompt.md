# Vibe second-family verify & close open item/place/break/pickup component limits

**Goal:** Independently verify `gen-block-place-break-rows` against `server-26.2.jar` and resolve item-component initialization (`DataComponentInitializers`) so `BlockItem.place`, `BucketItem.pickupBlock`, and real `ItemStack` tool mining speeds (`getDestroySpeed`) run end-to-end.

Both `gen-block-place-break-rows` and `gen-fluid-bucket-rows` hit `ItemStack.<init>: Components not bound yet` because `Bootstrap.bootStrap()` alone didn't bind registry item components in those harnesses. This lane cracks headless `DataComponentInitializers` / frozen registry init and verifies placement/breaking/pickup end-to-end.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements:**
- Cross-check `C:/Users/PASCAL/Desktop/path tracing/lab/results/local-20261124-103201-18924-1-0-gen-block-place-break-rows/` against `server-26.2.jar`.
- Bind real jar tags AND initialize item `DataComponent`s cleanly via bytecode inspection (`Bootstrap` / `RegistryDataLoader` / `DataComponentInitializers.build` / `VanillaRegistries` — no skipping component checks).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after component boot + primary witnesses pass, then extend cleanly.

**Verify & close these specific items:**
1. **Item component boot recipe:** document exact Java calls to construct valid `ItemStack(Items.STONE)`, `ItemStack(Items.BUCKET)`, `ItemStack(Items.WATER_BUCKET)`, and `ItemStack(Items.DIAMOND_PICKAXE)` in a headless harness without `Components not bound yet`.
2. **Real `BlockItem.place` & `BlockPlaceContext`:** witness `STONE`, `OAK_SLAB` (top/bottom/double), `OAK_STAIRS` (facing/half/shape), `OAK_DOOR` (2-block hinge & neighbor), `SCAFFOLDING`, plus self-intersection (`CollisionContext.of(player)`) block.
3. **Real `ServerPlayerGameMode.handleBlockBreakAction` & mining ticks:** witness exact tick count and destroy progress float bits for `STONE` (hand vs wooden vs iron vs diamond pickaxe, Efficiency I–V, Haste, underwater/not-on-ground `0.2x` penalties) and `BreakAck`.
4. **Real `BucketItem.use` / `BucketItem.pickupBlock`:** verify water/lava pickup & placement return full `InteractionResult.SUCCESS` with updated `ItemStack` cleanly.

**Write in your run folder:**
- `rows.tsv`: `rule_id	kind	item_or_tool	block_or_target	result_or_ticks	constants_hex	scars	source`
- `witnesses.tsv`: `case_id	tick	item_stack	pos	block_state_after	progress_or_result	events`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Any witness tick where `ItemStack` component initialization fails, break ticks miss `ServerPlayerGameMode` by ±1 tick, or placement rules differ from the 26.2 jar.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
