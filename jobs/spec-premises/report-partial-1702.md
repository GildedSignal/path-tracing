# Spec Premises Verification Report: `D:\path-tracing-wt\sets\jobs\3.2d-3\last.md`

*In-progress incremental log of premises checked against the codebase.*

## Summary (pending completion)

- EXISTS: 0
- DIFFERS: 0
- MISSING: 0

## Findings Log

### Batch 1 (files read: `HANDOFF.md`, `plain.rs`, `footprint.rs`, `pattern.rs`, `state.rs`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\footprint.rs`: `Query` enum (`Block=1, Shape=2, Fluid=4, Hooks=8, Loaded=16` at L12-18) EXISTS; `RecordingWorld` L161-346; no reverse footprint index exists.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\pattern.rs`: `in_scope` EXISTS at L340-342; `cell_digest` L70-78 and `PatternTable` L135-281 use `u128` FNV-1a digests (`intern` L323-329 keys `HashMap<u128, u16>` by digest only without byte comparison).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\state.rs`: `PlayerState::bits` (L202-249) and `PlayerState::from_bits` (L282-373) ALREADY EXIST, though `PlayerState` lacks known-field/provenance masks, kit/inventory facts, and interaction inputs (`Input` L7-17).
- Checked `D:\path-tracing-wt\plain\engine\plan\src\plain.rs`: uses `law::pattern::law_digest` (note: not present in `main`'s `pattern.rs`), `words::fed`, `words::recurrence`, `Goal::touched`, `Goal::gap`, `settle`, `half_ulp = f32::EPSILON * 16.0` (L332), `NEAR`/`HALO`/`OUTSIDE` (L93-95).

### Batch 2 (files read: `deviation.rs`, `goal.rs`, `arith.rs`, `reach.rs`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\deviation.rs`: `compare(plan: &str, recording: &str)` EXISTS (L35), with `FIELDS` (L20-29) and `NEVER` (L30); `compare_event` does NOT exist yet (spec asks to extract it from `plan::deviation`). Note `FIELDS` omits `NEVER` fields (including `food_tick_timer`, `hurt_cooldown`, `no_jump_delay`, `using_item`, etc.) and parses JSONL text rather than `PlayerState` / observation bits directly.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\goal.rs`: `Goal::touched` EXISTS (L19-21), `Goal::gap` (L23-28), `Goal::lower_bound` (L33-36).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\arith.rs`: `free_tick` (L32-55) and `Cycle` (L67-125) EXIST (note spec line 43/115 says replace `arith::Cycle`/`free_tick`).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\reach.rs`: `Reach` hop/path tables (L19-40, L49-127) and `turn` with `f64::to_radians().cos()/sin()` (L42-45) EXIST.

### Batch 3 (files read: `flight.rs`, `search.rs`, `walk.rs`, `time.rs`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\flight.rs`: `write_script` (L59-79) writes JSONL tape; `fly_with_budget` (L88-218) uses `settle`, `Reach::compile`, `Field::build`, fresh `ProposalTable::default()` (L134), `Planner`, and `slide` (L152).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\search.rs`: `Planner` (L115-513) has `Order::Admissible`/`Field`, `Entry::cmp` (L92-94) uses `other.key[0]`, `other.key[1]`, `self.node` (LIFO, NOT FIFO! Spec H says "g in EVERY ordering key/FIFO ties"), `fed()` (L132-134), `identity()` (L169-174), `arc()` calling `arith::free_tick`, `gait()` calling `arith::Cycle`. Note spec H says "Scope the existing Planner per exceptional span... g in EVERY ordering key/FIFO ties" and "Arc certificate replaces free_tick on its domain".
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\walk.rs`: `settle` (L43-54), `certify` (L75-111).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\time.rs`: `Timeline`, `Still`, `Scheduled`, `Free`.

### Batch 4 (files read: `class.rs`, `gait.rs`, `program.rs`, `segment.rs`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\class.rs`: `class()` (L25-104) encodes `StateReads` via comparison answers on accumulating fields + raw bits on others.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\gait.rs`: `recurrence()` EXISTS (L55-64), `Discovery` private `memo: PatternTable<Vec<usize>>` (L37), `fed()` (L72-77) sets `food_level=20, saturation=20.0, exhaustion=0.0, food_tick_timer=0`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\program.rs`: `Predicate` (L4-19) has 14 movement predicates (`Always`, `OnGround`, `HorizontalCollision`, `VerticalCollision`, `InWater`, `InLava`, `WaterDeep`, `Climbable`, `JumpReady`, `Sprinting`, `SupportEnds`, `Since`, `All`, `Not`) — no break/place/use/item predicates; `Arguments` (L84-103) uses 64-way `yaw_index` + `refined_yaw` + `pitch` + `max_ticks`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\segment.rs`: `events` (L66-83) cuts on discrete state + `BodyBlock`; `support()` (L84-86) uses `cell_digest(..., 1 | 16)`.

### Batch 5 (files read: `table.rs`, `mining.rs`, `region.rs`, `snapshot.rs`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\words\src\table.rs`: `ProposalTable` (L143-320) uses `PatternTable` (`worlds`), `ClassOf` (`class::class`), `learn`, `proposals`, `replay`, `holds`, `invent`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\mining.rs`: `MiningClock` / `MiningSample` (L10-217) has `destroy_speed`, `hardness`, `correct_tool` — NOT wired to `World` or `PlayerState`; no drop generator, silk-touch, or block-place legality/replaceability in `law`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\world\src\region.rs`: `WorldReader` (L55-126) is static `.mca` Anvil reader ("Reopen to observe edits", L54), Overworld `-64..=319` only (L96-99).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\world\src\snapshot.rs`: `WorldSnapshot` (L103-220) loads a dense `u16` model-id box (NOT fine block-state identity; `BlockState` is converted to `u16` model id via `lookup` at L153), checks `version.txt` & `vanilla` datapack, panics outside bounds (`assert_covered`, L167-169). Note `world/data/models.tsv` has the 518-model table, no place/break/silk-touch data.

### Batch 6 (files read: `movement.rs` [1-400, 1100-1300], `world.rs`, `math.rs`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\movement.rs`: `update_sprinting` (L217-233) checks `food_level > 6`; `food_tick` (L1271-1300) skips food_level decrement when `world.difficulty() == 0`; `tick_movement` (L118-214) calls `food_tick` (L211) without `server_regeneration`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\world.rs`: `World` trait (L308-537) has `block`, `hook_block`, `model_id`, `fluid_block`, `has_fluid`, `has_inside_effects`, `visit_collision_cells`, `loaded`, `min_y`, `difficulty`, `natural_regeneration`, etc. — NO break/place/drop-generator/silk-touch queries.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\math.rs`: `sin`/`cos` 65536-entry `sin.bin` table (L1-17).

### Batch 7 (files read: `wt/plain/pattern.rs`, `state_reads.rs`, `items.rs`, `polish.rs`)
- Checked `D:\path-tracing-wt\plain\engine\law\src\pattern.rs`: `law_digest` exists at L85-90 in the unmerged `wt/plain-path` branch, NOT in `main`'s `pattern.rs`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\law\src\state_reads.rs`: `StateField` enum (L29-95) and `StateReads(pub u128)` (L102).
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\world\src\items.rs`: `use_effects` (L8-14) reads `use-effects.tsv` for `can_sprint` & `speed_multiplier`.
- Checked `C:\Users\PASCAL\Desktop\path tracing\engine\plan\src\polish.rs`: `slide` (L27-64).

### Batch 8 (files read: `Recorder.java`, `PathTracingMod.java`, `ClockAudit.java`, `ClockQualification.java`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\bot\src\pathtracing\Recorder.java`: `playPlan` (L41-92) reads JSONL plan file and feeds `planInputs` tick-by-tick (`startTick` L315-322); `positionPacket`/`motionPacket`/`rotationPacket`/`explosionPacket` (L510-560) record anomalies during recordings. No JNI channel, no QPC clock, no section/block delta packet hooks.
- Checked `PathTracingMod.java`, `ClockAudit.java`, `ClockQualification.java`: client-only Fabric mod, `System.nanoTime()`/`currentTimeMillis()` only (no QPC clock).

### Batch 9 (files read: `Tracks.java`, `Bot.java`, `PlayerSnapshot.java`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\bot\src\pathtracing\Tracks.java`: `Tracks.java` uses `System.nanoTime()` (L93, L289, L534) and `clock` (client tick counter, L226); `plan <path>` reads JSONL tape via `Recorder.playPlan` (L100-112); `TrackRcon.execute` prepares kit/tp over RCON (`L480`).
- Checked `C:\Users\PASCAL\Desktop\path tracing\bot\src\pathtracing\Bot.java`: `PLANNER = new StraightLine()` (L23) called synchronously on client thread (`goTo` L108-122).
- Checked `C:\Users\PASCAL\Desktop\path tracing\bot\src\pathtracing\PlayerSnapshot.java`: captures player state as JSON (`JsonObject`) — NOT a `u64[]` bit schema matching `PlayerState::bits()` (`use_item_speed_multiplier`, `use_item_can_sprint`, `remaining_fire_ticks`, `hurt_cooldown`, `no_physics`, `fall_flying`, `riding`, `boat`, `effects`, `flying`, `movement_this_tick`, `difficulty`, `natural_regeneration`, `min_y`, kit/inventory are NOT in `PlayerSnapshot.capture`).

### Batch 10 (files read: `Pointer.java`, `WorldExporter.java`, `BlockDumper.java`, `TickInput.java`, `MinecraftClockMixin.java`)
- Checked `Pointer.java`, `WorldExporter.java`, `BlockDumper.java`, `TickInput.java`, `MinecraftClockMixin.java`: `BlockDumper` dumps collision shapes & `defaultDestroyTime`/`requiresCorrectToolForDrops` (no drop generator or place legality); `TickInput` has movement + yaw/pitch only.

### Batch 11 (files read: `ClientPacketListenerMixin.java`, `TrackDetector.java`, `TrackRcon.java`)
- Checked `C:\Users\PASCAL\Desktop\path tracing\bot\src\pathtracing\mixin\ClientPacketListenerMixin.java`: EXISTS — hooks `handleUpdateAttributes`, `handleSetEntityData`, `handleSetTime`, `handleMovePlayer`, `handleSetEntityMotion`, `handleSystemChat`, `handleRotatePlayer`, `handleExplosion`, `handleEntityPositionSync`, `handleTeleportEntity`, `handleRespawn`, `handleBlockUpdate` (L106), `handleChunkBlocksUpdate` (L111), `handleLevelChunkWithLight` (L116), `handleForgetLevelChunk` (L121). Currently guarded by `LandingClientTrace.CORPUS`/`ENABLED` and only notify `LandingClientTrace`/`Recorder`/`Tracks` (no block-state/palette delta forwarded to Rust).
- Checked `TrackDetector.java` (touchesB L5-9) and `TrackRcon.java`.

