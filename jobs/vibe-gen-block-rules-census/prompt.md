# Vibe live census: 26.2 BLOCK_RULES (1,196 blocks) and STATE_STATIC/STATE_DYNAMIC (32,366 states)

**Goal:** Dump and verify live against `server-26.2.jar` all 1,196 blocks' rule rows (`destroySpeed`, `requiresCorrectToolForDrops`, tool-tag bitmask `pickaxe/axe/shovel/hoe/shears/sword`, redstone conductor, `PushReaction`, `blocksMotion`, `canBeReplaced`, `isSignalSource`, `hasBlockEntity`, `canSurvive`/support-pop family, `restore_class` = `IdenticalDrop | SilkTouchOnly | WaterloggedLeavesWater | PartnerHalf | Irreversible`) plus all 32,366 states' `STATE_STATIC` (`sturdy_faces` 6-bit mask, `interact_kind`, `waterlogged`, `partner_half`) and packed `STATE_DYNAMIC` properties.

Feeds **both** `yarn-cold-fold` (replacing the 40-row partial `engine/law/src/basis/rules.tsv` with full 1,196 rows across `engine/world/data/states.tsv` & `blocks.tsv`) **and** kit unfolding (`BreakProjection` + `STATE_STATIC`). Note: `vibe-check-block-place-break` (`lab/results/local-20261124-130147-11356-1-0-vibe-check-block-place-break/`) solved headless `DataComponentInitializers` / tag loading so real `ItemStack` tool & block checks run cleanly.

Jar: `D:/path-tracing-lab/server/versions/26.2/server-26.2.jar`; javap: `D:/path-tracing-deps/data/java/java-runtime-epsilon/bin/javap.exe`.

**Harness requirements (mandatory executed checks):**
- Boot jar tags (`MappedRegistry.prepareTagReload`) + `DataComponentInitializers` + Unsafe `ServerLevel` + real `LevelChunk` where `canSurvive`/`isRedstoneConductor`/`isFaceSturdy` read `BlockGetter` / `LevelReader`.
- Iterate ALL `BuiltInRegistries.BLOCK` entries in registry order (all `1,196` blocks) and ALL `Block.BLOCK_STATE_REGISTRY` states (`0..32,366`, asserting contiguous alignment with `engine/world/data/states.tsv`).
- Write `dossier.md` (`Claim: ...`), `rows.tsv`, and `witnesses.tsv` early after the 1,196-block dump passes, then extend with state census and edge witnesses.

**Scenes and checks to execute:**
1. **Per-block rules (`1,196` rows):** query live `defaultDestroyTime()`, `requiresCorrectToolForDrops`, Mojang block tags (`MINEABLE_WITH_{PICKAXE,AXE,SHOVEL,HOE}`, `SWORDS`, `SHEARS`, `NEEDS_{STONE,IRON,DIAMOND}_TOOL`), `isSignalSource`, `EntityBlock` / `hasBlockEntity`, `PistonBehavior`/`PushReaction`, `canBeReplaced`, `blocksMotion`, support-pop classification (`None | SyncPop | ScheduledTick | RailNeighborPop`), and `restore_class` (`IdenticalDrop | SilkTouchOnly | WaterloggedLeavesWater | PartnerHalf | Irreversible` e.g. chest/shulker/spawner/beacon/bedrock).
2. **Per-state static & dynamic tables (`32,366` states):** `sturdy_faces` `[D,U,N,S,W,E]` from live `isFaceSturdy(..., SupportType.FULL)`, `interact_kind` (`None | Door | TrapDoor | FenceGate | Button | Lever | Container | Bed | Comparator | Repeater | Other`), `waterlogged`, `partner_half` (doors, beds, tall plants, pistons), plus packed `STATE_DYNAMIC` properties (`facing`, `half`, `open`, `powered`, `fluid_level`, `falling`, `slab_type`, `stair_shape`, `scaffolding_distance`, `extended`, `attached`).
3. **Spot witnesses:** verify granite/diorite/andesite/stone/obsidian/bedrock/glass/ice/glowstone/oak_door/waterlogged_oak_slab/scaffolding/iron_door against live `ItemStack` tool probes (hand, diamond pickaxe, silk-touch Eff-V).

**Write in your run folder:**
- `rows.tsv`: `block_id	block_name	default_state_id	destroy_speed_hex	requires_correct_tool	tool_tags_mask	push_reaction	blocks_motion	can_be_replaced	is_redstone_conductor	is_signal_source	has_block_entity	support_rule	restore_class`
- `state_rows.tsv` (or appended/linked static summary): `state_id	block_id	sturdy_faces_mask	interact_kind	waterlogged	partner_half	dynamic_packed_hex	properties`
- `witnesses.tsv`: `case_id	state_id	block_name	probe	expected_from_jar	actual_bits	notes`
- `dossier.md` (first line `Claim: `), `reproduce.sh`, and `wiki.md`.

**Falsifier:** Total block count != 1,196, total state count != 32,366, any `Block.BLOCK_STATE_REGISTRY` ID mismatch against `states.tsv`, or any row derived from guessing/names instead of live jar method execution.

Write only in your run folder. No game launch, network, or production edits; no `work.patch`.
