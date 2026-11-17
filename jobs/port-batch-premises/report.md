# Port batches: premise check and exact edits

Checked against `jobs/port-digest/report.md` §4, the 10 `lab/wiki/*.md` entries, all 10 `lab/results/local-20261117-*` (`dossier.md` + `check.md`, checker winning on discrepancies), `docs/plan.md` steps 5 & 8, and the Rust/Java workspace on `main` (`engine/law/src/*.rs`, `engine/law/Cargo.toml`, `engine/law/tests/*.rs`, `engine/world/src/*.rs`, `engine/world/tools/ModelTable.java`, `engine/world/tests/*.rs`, `engine/words/src/*.rs`, `engine/plan/src/*.rs`, `jobs/1.2-footprints/verify.ps1`, `recordings/`, `lab/tracks/runs/`).

---

## Batch M1 — Player kinematics & resource guards

Scope: `engine/law/src/state.rs`, `engine/law/src/state_reads.rs`, `engine/law/src/attributes.rs`, `engine/law/src/movement.rs` (lenses `#2`, `#3`, `#4`, `#5`) — plus cross-crate consumers `engine/words/src/class.rs`, `engine/words/src/segment.rs`, `engine/plan/src/deviation.rs`.

### 1. Premises

| Item | Status | Evidence on `main` |
|---|---|---|
| `PlayerState` fields (`effects`, `flying`) | EXISTS | `engine/law/src/state.rs:116-117` (`effects: bool`, `flying: bool`); `Default` at `:186-187` |
| Creative-flight fields (`mayfly`, `spectator`, `was_jumping`, `jump_trigger_time`, `flying_speed`) | MISSING | Not found in `engine/law/src/state.rs:53-121` or `engine/law/src/state_reads.rs:29-95` |
| Jump-boost field (`jump_boost_level`) | MISSING | Not found in `engine/law/src/state.rs:53-121` or `engine/law/src/state_reads.rs:29-95` |
| `invulnerable: bool` field | MISSING | Not found in `engine/law/src/state.rs:53-121` or `engine/law/src/state_reads.rs:29-95` |
| `u8` / `Option<u8>` encoder & decoder for `StateField` / `bits()` | MISSING | `engine/law/src/state_reads.rs:122-135` (`encode_as!` covers `bool, i32, u32, u64, f32, f64, Pose, Vec3, [i32; 3], HurtCooldown, Input` + `Option<T>`, no `u8`); `engine/law/src/state.rs:256-279` (`Bits` has no `u8` helper) |
| `StateReads(pub u128)` capacity | EXISTS | `engine/law/src/state_reads.rs:29-95, 102`: 66 fields currently fit in `u128`; adding 7 fields gives 73 (< 128) |
| `AttributeInstance.calculateValue` + `Object2ObjectOpenHashMap` order | EXISTS | `engine/law/src/attributes.rs:18-47, 57-80` (`SpeedOperation`, `operation_order`, sequential `AddMultipliedTotal`, `[0.0, 1024.0]` clamp) |
| Speed/Slowness/JumpBoost effect helpers on `PlayerState` | MISSING | Not found in `engine/law/src/attributes.rs:49-81` |
| `effects` refusal & `flying` refusal | EXISTS | `engine/law/src/movement.rs:121-123` (`effects`), `:133-135` (`flying`) |
| Dry-land crawl cascade (`update_pose`, `fits`, `crouching`, `slowly`, `modify_input`, `sneaking_speed: 0.3`) | EXISTS (bit-exact) | `engine/law/src/movement.rs:147-149, 157, 174, 207, 272-306, 520-543`, `engine/law/src/state.rs:27-33, 179` (checked against `#2` checker correction on `LocalPlayer.aiStep:154` vs `KeyboardInput.tick:157-161`) |
| `crouching` / `getDesiredPose` `!flying` gate | DIFFERS (omitted) | `engine/law/src/movement.rs:147` (`crouching`) and `:531` (`update_pose` `desired`) omit `!read_state!(world, state.flying)` (jar `LocalPlayer.aiStep` `@92-154` & `Player.getDesiredPose` `@44-64` check `!abilities.flying`, previously masked by `:133-135` refusal) |
| `flying_speed` (`Player.getFlyingSpeed`) | DIFFERS (incomplete) | `engine/law/src/movement.rs:309-315` handles non-flying `0.025999999`/`0.02` only; lacks `flying && !riding` branch |
| `update_swimming` & `update_fluid` `flying` gates | EXISTS | `engine/law/src/movement.rs:82-84` (`flying -> false`), `:887` (`pushed = !flying`) |
| `jump_from_ground` jump-boost term | DIFFERS | `engine/law/src/movement.rs:373` hardcodes `+ 0.0f32` instead of `0.1f32 * (amp + 1)` |
| `jump_from_ground` exhaustion (`0.2`/`0.05`) & `calculate_fall_power` / `calculate_fall_damage` | EXISTS | `engine/law/src/movement.rs:381-385, 822-829` (matches `#4` checker correction) |
| `movement_exhaustion` & `add_exhaustion` | DIFFERS (missing 2 guards) | `engine/law/src/movement.rs:1099-1121` & `math.rs:45-47` match unmounted survival/adventure, but lack `riding`/`boat` early return (`@0-4`) and `abilities.invulnerable` gate (`@0-10`) |
| **Cross-crate premise:** `words::class` exhaustive `StateField` match | EXISTS (will fail compile) | `engine/words/src/class.rs:28-102` exhaustively matches every `StateField` variant |
| **Cross-crate premise:** `words::segment::events` cut list | EXISTS | `engine/words/src/segment.rs:66-83` |
| **Cross-crate premise:** `plan::deviation` hardcoded `PlayerState::bits()` offsets | EXISTS (will break silently) | `engine/plan/src/deviation.rs:20-29, 54-74` hardcodes prefix indices `0..57`, two `Option`s at `57..`, and `at(75)` for `speed_modifiers` |

### 2. Exact edits

1. **`engine/law/src/state.rs` & `engine/law/src/state_reads.rs`:**
   - Add to `PlayerState` (`state.rs:53-121`), `Default` (`:124-191`), `bits()` (`:202-249`), `from_bits()` (`:282-373`), and `state_fields!` (`state_reads.rs:29-95`):
     - `pub mayfly: bool` (default `false`), `pub spectator: bool` (default `false`), `pub was_jumping: bool` (default `false`), `pub jump_trigger_time: i32` (default `0`, `Player.java` `protected int`), `pub flying_speed: f32` (default `0.05f32`, `Abilities.<init>` `@10-18`).
     - `pub jump_boost_level: Option<u8>` (default `None`, clamped `0..=255` via `Mth.clamp(amp, 0, 255)` in `MobEffectInstance.<init>` `@25-34`). Add `u8` `encode_as!` in `state_reads.rs:122-130` and `Bits::u8` in `state.rs:256-279` (or store `Option<u32>`/`u8` adapter).
     - `pub invulnerable: bool` (default `false`; `GameType.updatePlayerAbilities` `@17-19, 42-44` sets `true` for Creative/Spectator, `false` otherwise at `@65-67`).
   - Also update `engine/words/src/class.rs:28-102` for the 7 new `StateField` variants (`jump_trigger_time > 0` as running/counter or bits, `mayfly`/`spectator`/`was_jumping`/`flying_speed`/`jump_boost_level`/`invulnerable` as bits or guard), `engine/words/src/segment.rs:66-83` (`events` cuts for `mayfly`/`spectator`/`invulnerable`/`jump_boost_level` and running counter `jump_trigger_time == 0`), and `engine/plan/src/deviation.rs:54-63` (`at()` offsets for `PlayerState::bits()`).
2. **`engine/law/src/attributes.rs`:**
   - Add helper methods on `PlayerState` (around `:49-81`) for Speed, Slowness, and Jump Boost, following `MobEffects.<clinit>` (`@0-34`, `@37-71`, `@227-259`), `AttributeTemplate.create` (`@0-24`), and `MobEffect.addAttributeModifiers` (`@60-94`, remove-by-id then add so `speed_modifiers` never accumulates duplicate IDs):
     - `set_speed(amp: Option<i32>)`: remove any modifier with `id == "minecraft:effect.speed"`; if `Some(a)`, clamp `a` to `0..=255` and push `SpeedModifier { id: "minecraft:effect.speed".into(), amount: 0.20000000298023224f64 * (a.clamp(0, 255) + 1) as f64, operation: SpeedOperation::AddMultipliedTotal }`.
     - `set_slowness(amp: Option<i32>)`: remove `"minecraft:effect.slowness"`; if `Some(a)`, push `-0.15000000596046448f64 * (a.clamp(0, 255) + 1) as f64` with `AddMultipliedTotal`.
     - `set_jump_boost(amp: Option<i32>)`: update `jump_boost_level` (`Some(a.clamp(0, 255) as u8)` or `None`), remove old Jump Boost contribution from `safe_fall_distance` and add `+1.0f64 * (amp + 1) as f64` when `Some`, clamping `safe_fall_distance` to `[-1024.0, 1024.0]` (`Attributes.<clinit>` `@792-808`) and recomputing `speed` (`movement_speed_value() as f32`) if Speed/Slowness changed.
3. **`engine/law/src/movement.rs`:**
   - **Creative flight (`#3`):**
     - Remove `if read_state!(world, state.flying) { refuse!("Player.travel: abilities.flying"); }` at `:133-135`.
     - Capture `let was_jumping = read_state!(world, state.was_jumping);` at entry (`LocalPlayer.aiStep` `@56-66`: reads previous tick's latched jump key before `input.tick` `@161`), and write `write_state!(world, state.was_jumping) = input.jump;` (`@56-66` semantics with held `Input`).
     - In `crouching` (`:147`) and `update_pose` (`:531`), gate with `!read_state!(world, state.flying)` (`LocalPlayer.aiStep` `@92-154`, `Player.getDesiredPose` `@44-64`).
     - Implement `LocalPlayer.aiStep` `@487-640` toggle block (before `super.aiStep`): if `read_state!(world, state.mayfly)`: if `read_state!(world, state.spectator)` force `write_state!(world, state.flying) = true` (`@498-529`); else if `!was_jumping && input.jump` (`@535-549`): if `read_state!(world, state.jump_trigger_time) == 0` set `jump_trigger_time = 7` (`@557-570`); else if `!read_state!(world, state.swimming)` (and `!riding` / non-jumpable vehicle, `@573-591`), toggle `flying = !flying` (`@594-609`), if now `flying && on_ground` call `jump_from_ground(state, world)` (`@612-628`), and set `jump_trigger_time = 0` (`@631-640`).
     - Implement `LocalPlayer.aiStep` `@815-894` vertical flight input before `travel`: if `read_state!(world, state.flying)`, compute `let dir = input.jump as i32 - input.sneak as i32; if dir != 0 { modify_state!(world, state.velocity).y += (dir as f32 * read_state!(world, state.flying_speed) * 3.0f32) as f64; }`.
     - Decrement `jump_trigger_time` (`Player.aiStep` `@0-14` via `super.aiStep` `@1095`): `if read_state!(world, state.jump_trigger_time) > 0 { modify_state!(world, state.jump_trigger_time) -= 1; }` (after toggle block, before/during `super.aiStep`).
     - In `flying_speed` (`:309-315`, `Player.getFlyingSpeed` `Player` `@4807-4835`): if `read_state!(world, state.flying) && !read_state!(world, state.riding)` return `read_state!(world, state.flying_speed) * if read_state!(world, state.sprinting) { 2.0f32 } else { 1.0f32 }`; else keep existing `0.025999999` / `0.02`.
     - In `travel` (`:409-427`, `Player.travel` `@121-163`): if `read_state!(world, state.flying)` (and not swimming? — check Open Question 1): save `let d0 = read_state!(world, state.velocity).y;` (`@131-138`) before running `travel_fluid`/`travel_air`, then overwrite `modify_state!(world, state.velocity).y = d0 * 0.6;` (`@144-160`).
     - Post-`super.aiStep` landing clear (`LocalPlayer.aiStep` `@1098-1136`, after `travel` and `update_pose`): `if read_state!(world, state.on_ground) && read_state!(world, state.flying) && !read_state!(world, state.spectator) { write_state!(world, state.flying) = false; }`.
   - **Status effects (`#4`):**
     - Retain `if read_state!(world, state.effects)` refusal at `:121-123` for unmodeled effects only (Speed, Slowness, Jump Boost operate through `speed_modifiers`, `safe_fall_distance`, `jump_boost_level`).
     - In `jump_from_ground` (`:373`), replace `+ 0.0f32` with `+ read_state!(world, state.jump_boost_level).map_or(0.0f32, |amp| 0.1f32 * (amp as f32 + 1.0f32))` (`LivingEntity.getJumpBoostPower` `@0-31`).
   - **Exhaustion guards (`#5`):**
     - At `movement_exhaustion` (`:1104`), add early return `if read_state!(world, state.riding) || read_state!(world, state.boat) { return; }` (`ServerPlayer.checkMovementStatistics` `@0-4` `isPassenger()`).
     - Gate `Player.causeFoodExhaustion` (`@0-10` `abilities.invulnerable`): return early when `read_state!(world, state.invulnerable)` on the player-caused exhaustion paths (`movement_exhaustion` `:1104` and `jump_from_ground` `:381-385`, NOT `food_tick` `:1285, 1292` — see Open Question 2).

### 3. Tests proving each edit bit-exact

- **Recorded run (`#3` creative flight):** `C:\Users\PASCAL\Desktop\path tracing\recordings\track-parkour_race-Brain_Forge-1.jsonl` lines `10495-10510` (`t=10493..10508`, `lab/tracks/runs/parkour_race.jsonl`): assert `t=10495` jump press #1 (`jumpTriggerTime=7->6`), `t=10497` release, `t=10499` (`s=4`) press #2 toggles `flying=true`, producing exact `requested.yBits = 0x3fcdd57e7f403bbe` and `velocity.yBits = 0x3fc1e67f19268a3f` (`t=10499..10508`). (Note: `parkour_race` has `"sprintWindow":7` and unrecorded gamemode switch, so test via direct `law::tick` witness over slice `t=10493..10508`.)
- **Unit witnesses (`#3`):** press-to-press sweep `s in 2..=8` (`s=2..=6` toggles & resets `jump_trigger_time=0`; `s=7,8` reads `0` and re-arms `7`); swimming suppression; Creative landing clear (`on_ground && flying && !spectator -> false`) vs Spectator exempt (`flying` stays `true`); 5-tick open-air ascend/descend/hover matching `d0 = 0x3fc3333340000000`, `vy' = 0.09000000357627869`, converging to `0.375`.
- **Unit witnesses (`#4`):** exact raw bits from `#4` dossier/checker: `Speed I` (`0x3fbeb851f47ae148`), `Speed II` (`0x3fc1eb85247ae148`), `Slowness I` (`0x3fb5c28f5f0a3d70`), `Slowness II` (`0x3fb1eb851e147ae0`), `Slowness VII` (`0x0`), `Speed II + Slowness I` (`0x3fbe76c8ba6e978d`), `amp 255` Speed (`0x40499999a0000000`), `getJumpBoostPower` `amp {0,1,9,128,255}` (`0x3dcccccd, 0x3e4ccccd, 0x3f800000, 0x414e6667, 0x41cccccd`), `jump_power` (`0x3f051eb8, 0x3f1eb852, 0x3fb5c28f`), `calculate_fall_power(5.0)` JB II (`0x3eb0c6f7a0c00000` -> `0` dmg) vs JB I (`1` dmg), and re-apply `set_speed` leaves 1 modifier.
- **Unit witnesses (`#5`):** `i=7` sprint bits (`0x3be56041`), 30-tick sprint accumulation (`0x3f199999`), `0.005` (`cm=1`) vs `0.004999999999999999` (`cm=0`), `-0.0` delta (`0`), `invulnerable=true` -> `0.0` on sprint & jump (while `food_tick` regen still exhausts), `riding=true` / `boat=true` -> `0.0`.
- **Unit witness (`#2`):** `#[cfg(test)]` in `movement.rs` replaying `S1/S2/S3` + `CheckPose` (`1.0` trapdoor `[0,0,0]..[1,0.1875,1]` at `y+1` -> `1.5` top slab -> `1.8` open; `!fits(Swimming)` freeze; `swimming==false`; `0.3` input scaling; 1-tick `crouching` shift ordering).

### 4. Conflicts & frozen gates

- **File conflict with Hook follow-up:** Both M1 and the hook follow-up edit `engine/law/src/movement.rs`. **M1 must merge before the hook follow-up** (or share one sequential merge). No file overlap with M2 or M3.
- **Frozen gates:**
  - `mutant-edge-backoff` (`movement.rs:558`), `mutant-step-up` (`:645`), `mutant-ceiling` (`:791`): untouched by M1.
  - `jobs/1.2-footprints/verify.ps1:66` (`chunk-corners` mutant matches `'if !world.loaded([x, lo[1], z]) {'` in `movement.rs:875`): untouched by M1.
  - `engine/plan/src/deviation.rs:54-63`: **breaks** if `PlayerState::bits()` shifts indices without updating `deviation.rs`.

### 5. Open questions for the integrator

1. **In `LocalPlayer.aiStep` / `LivingEntity.aiStep` while `abilities.flying == true`:** do `movement.rs:161-163` (water sneak `-0.04` sink), `movement.rs:176-191` (`jump_from_ground` / `jumpInLiquid` inside `LivingEntity.aiStep`), and `Player.travel` when `isSwimming()` (`Player.travel` `@0-120` before `@121`?) still execute while flying? (In `#3` dossier: `update_swimming` sets `swimming=false` when `flying` at `movement.rs:82-84`, and `LocalPlayer.aiStep:612-628` calls `jumpFromGround` on the takeoff tick, but `#3` did not dump `LivingEntity.aiStep`'s `jumping` gate or `LocalPlayer.aiStep`'s `inWater && sneak` gate when `flying == true`.)
   - *Options:* (A) Guard `movement.rs:161-163` and `movement.rs:176-191` with `!flying` by assumption, OR (B) have GLM/Argon verify those two offsets in `LocalPlayer.aiStep` / `LivingEntity.aiStep` before freezing, OR (C) keep `movement.rs:176-191` running as `LivingEntity.aiStep` does and check against `parkour_race` flight ticks.
   - *Recommendation:* **(B)** — one 10-second `javap` check on `LocalPlayer.aiStep` (`water && sneak` and `LivingEntity.aiStep` `jumping` + `Player.travel` `@0-120`) eliminates guessing on ground-contact Spectator / water-flying ticks.
2. **Where to put the `invulnerable` guard (`#5`):** inside `add_exhaustion` (`movement.rs:1099-1101`) vs in `cause_food_exhaustion` wrapping `movement_exhaustion` (`:1104`) and `jump_from_ground` (`:381-385`)?
   - *Fact:* `movement.rs:1285, 1292` (`food_tick`) calls `add_exhaustion` directly, and in the jar `FoodData.tick` calls `FoodData.addExhaustion` directly, NOT `Player.causeFoodExhaustion` (`#5` dossier `:17-19`). Putting `if invulnerable { return; }` inside `add_exhaustion` would wrongly silence `food_tick` and `ServerResourceEvent::Exhaustion`.
   - *Options:* (A) Put the guard inside `add_exhaustion`, OR (B) add `cause_food_exhaustion` gated on `!invulnerable` and call it from `movement_exhaustion` and `jump_from_ground`, leaving `add_exhaustion` raw for `food_tick` and `ServerResourceEvent::Exhaustion`.
   - *Recommendation:* **(B)** — matches the jar's `Player.causeFoodExhaustion` vs `FoodData.addExhaustion` split cleanly.

---

## Batch M2 — Step 5 world-edit & interact substrate

Scope: `engine/law/src/world.rs`, `engine/law/src/interact.rs` (new), `engine/law/src/lib.rs` (lenses `#6`, `#7`, `#8`, `#9`, `#10`, `#1`) — plus `engine/law/src/pattern.rs` and `CollisionContext` struct-literal callers if `Block` / `CollisionContext` change.

### 1. Premises

| Item | Status | Evidence on `main` |
|---|---|---|
| `interact.rs` module | MISSING | Not found in `engine/law/src/lib.rs:1-57` |
| `CollisionContext::is_placement` | MISSING | `engine/law/src/world.rs:291-307` has `feet_y, descending, fall_distance, leather_boots` only (`is_placement` not found) |
| `CollisionContext` struct-literal callers (no `..Default::default()`) | EXISTS (4 sites will fail compile) | `engine/law/tests/speed_differential.rs:39-40`, `engine/world/tests/blocks.rs:21, 141`, `engine/world/tests/collision_cells.rs:47-48`, `engine/plan/src/flight.rs:99` |
| `World::collision_shape` `Scaffolding` / `PowderSnow` `is_placement` branch | MISSING | `engine/law/src/world.rs:444-476` ignores placement mode (`PowderSnow` hardcoded at `:447-458`; no `Scaffolding` placement empty branch) |
| `VoxelShape::intersects` / `IndirectMerger` `1e-7` join (`Shapes.joinIsNotEmpty`) | EXISTS | `engine/law/src/shapes.rs:321-323, 461-525` (`ShapeView::intersects` + `overlap_range` with `1.0e-7` collapse); `Aabb::intersects` at `:203-205` is strict `<`/`>` |
| `traverse_blocks` ray stepper | EXISTS | `engine/law/src/blocks.rs:187-240` |
| `world_hook: bool`, `open_trapdoor_facing: Option<u8>`, `wall_or_gate: bool`, `Fluid` | EXISTS (static movement subset only) | `engine/law/src/world.rs:196, 197, 210, 214`; no mutable world, scheduled ticks, `set_block`, `can_open_by_hand`, `waterlogged`/`waterloggable`, `button`, or `plate` state/props |
| `Block` exhaustive destructuring in `pattern.rs` | EXISTS | `engine/law/src/pattern.rs:384-390` (`encode` destructures every field of `Block`) |
| `RecordingWorld` immutability contract | EXISTS | `engine/law/src/footprint.rs:1-2` ("wrapped world must be an immutable snapshot for the duration of a recording") |
| `FullScan` / `ReplayWorld` / `Scan` / `Scheduled` / `Free` `World` impls | EXISTS | `engine/law/tests/support/footprints.rs:51`, `engine/law/tests/replay.rs:247`, `engine/world/tests/collision_cells.rs:9`, `engine/plan/src/time.rs:44, 61` (new `World` methods need defaults) |

### 2. Exact edits

1. **`engine/law/src/world.rs` (& `CollisionContext` callers + `pattern.rs`):**
   - `CollisionContext` (`world.rs:291-307`): add `pub is_placement: bool`; update `CollisionContext::of` (`:299-306`) to set `is_placement: false`; add `CollisionContext::placement(state: &PlayerState) -> Self` (`CollisionContext.placementContext` `@75-101`: `descending: state.shift_key_down`, `is_placement: true`, `feet_y: state.position.y`, `fall_distance: state.fall_distance`, `leather_boots: state.leather_boots`). Derive `Default` or update the 4 struct-literal sites (`speed_differential.rs:39-40`, `blocks.rs:21,141`, `collision_cells.rs:47-48`, `plan/src/flight.rs:99`).
   - In `World::collision_shape` (`world.rs:444-476`): if `context.is_placement`: return empty `VoxelShape` when `block.scaffolding` (`ScaffoldingBlock.getCollisionShape` returns `Shapes.empty()` per `#9` check.md:21), and use exported `placement_shape` (see M3/Open Question 4 for `PowderSnowBlock`).
   - If `Block` (`world.rs:182-216`) adds `placement_shape: Option<VoxelShape>` (or any field), update `Block::default()` (`:219-254`), `Block::collision_shapes()` (`:258-260`) — **note:** do NOT include `placement_shape` in movement `collision_shapes()`/`never_collides()`! — and `pattern.rs:384-390` (`encode`).
   - Keep `BoxWorld` (`:540-679`) and `World` read methods intact without breaking `jobs/1.2-footprints/verify.ps1:48-53` exact match strings!
2. **`engine/law/src/interact.rs` (new, exported from `engine/law/src/lib.rs`):**
   - **Mutable Step-5 world/tick substrate** (kept separate from immutable `World` / `RecordingWorld`, or layered as an `InteractWorld` trait/struct holding block cells + `ScheduledTick` queue + entity list + event log):
     - `set_block(pos, new_state, flags, limit)` (`Level.setBlock` `@308-444`): writes state; when `(flags & 16) == 0 && limit > 0` (`@177-186`), runs `update_neighbour_shapes` in `UPDATE_SHAPE_ORDER = {W, E, N, S, D, U}` (`BlockStateBase.updateNeighbourShapes` `@814-851`); records neighbor notifications when `(flags & 1) != 0` (`@140-175`); records scheduled ticks (`(pos, kind, game_time + delay)`, first-wins dedup per `(type, pos)` per `port-scheduled-tick-dedup`).
   - **#6 `use_item_on` dispatch (`ServerPlayerGameMode.useItemOn` `@0-320` & `MultiPlayerGameMode.performUseItemOn` `@757-882`):**
     - Types: `InteractionHand { Main, Off }`, `InteractionResult { Success, SuccessServer, Consume, Fail, Pass, TryWithEmptyHand }` (`consumes_action()` true only for `Success | SuccessServer | Consume`), `BlockHitResult { pos, face, inside }`, `ItemStack` (kind, count, cooldown, `can_place_on`).
     - 5-stage `use_item_on`: (1) feature enabled else `Fail` (`@0-33`); (2) spectator -> menu `Consume`/`Pass` (`@34-73`); (3) global sneak skip `player.shift_key_down && (!main.is_empty() || !off.is_empty())` jumps to stage 5 (`@74-128`, no `doesSneakBypassUse`); (4) `block.use_item_on` (`@131-148`, default `TryWithEmptyHand`), if `consumes_action()` return (`@150-173`), else if `TryWithEmptyHand && hand == Main` run `block.use_without_item` (`@174-222`, consumes -> return); (5) if `stack.is_empty() || stack.on_cooldown` return `Pass` (`@223-244`), else `item_use_on` (`Abilities.mayBuild` / adventure check -> `block_item_place`, preserving count on `instabuild`, `@245-320`).
     - `client_start_use_item` (`Minecraft.startUseItem` `@4156-4346`): iterates `[Main, Off]`, sets `rightClickDelay=4` (on item use), runs `use_item_on`; stops on `consumes_action()` or `Fail`; on `Pass`, falls through to air-use (`bucket_use` if non-empty) then `Off`.
   - **#7 Doors, trapdoors, fence gates (`useWithoutItem`):**
     - `door_use_without_item` (`DoorBlock.useWithoutItem` `@0-82`): gate `can_open_by_hand` (false only for `iron`, `gold`; true for `copper`, `stone`, `polished_blackstone`, 12 woods) else `Pass`; cycle `OPEN`; `set_block(pos, cycled, 10, 512)`; shape wave invokes `DoorBlock::update_shape` (`@12-96`) which checks axis-Y partner half (`HALF` opposite, same door kind), mirrors `(FACING, HINGE, OPEN, POWERED)` while preserving own `HALF` (flag `10`), or returns `AIR` if partner missing/mismatched; `LOWER` also checks `can_survive(DOWN)` (`@97-129`); returns `Success`.
     - `trapdoor_use_without_item` (`TrapDoorBlock.useWithoutItem` `@0-26` -> `toggle` `@0-75`): gate `can_open_by_hand` else `Pass`; cycle `OPEN`; `set_block(pos, cycled, 2, 512)` with **0 entity queries**; if `waterlogged`, `schedule_tick(pos, Water, 5)`; returns `Success`.
     - `fence_gate_use_without_item` (`FenceGateBlock.useWithoutItem` `@0-189`): no `can_open_by_hand` gate; if `open` set `open=false` (`set_block(..., 10)`); if closed, if `facing == player_horizontal_dir.opposite()` rewrite `facing = player_horizontal_dir`, set `open=true`, `set_block(..., 10)`; returns `Success`.
   - **#8 Buttons (`ButtonBlock`):**
     - `button_use_without_item` (`@0-29`): if `powered` return `Consume` (0 writes); else `button_press` (`@16-54`): `set_block(pos, powered=true, 3)`, notify `pos` and `pos + connected_dir.opposite()` (`Floor -> Down`, `Ceiling -> Up`, `Wall(f) -> f.opposite()`), `schedule_tick(pos, Button, 20 /* stone | polished_blackstone */ | 30 /* 12 woods */)`, return `Success`.
     - `button_tick` (`@13-21`) & `button_entity_inside` (`@4-38`): `tick` gates on `powered`, `entity_inside` gates on server + `arrow_activatable && !powered`; both call `button_check_pressed` (`@7-164`): if `arrow_activatable`, query `AbstractArrow` intersecting **current-state** shape bounds moved to `pos` (`WALL(NORTH)` unpowered `z in [0.875, 1.0]`, powered `z in [0.9375, 1.0]` per `#8` check.md:15; floor unpowered `[0, 0.125]`, powered `[0, 0.0625]`); if `arrow_present == powered`: if `arrow_present` re-`schedule_tick`, else no-op; if changed: `set_block(pos, powered=arrow_present, 3)`, notify same 2 cells, re-schedule iff `arrow_present`.
   - **#9 Block placement (`BlockItem.place` `@0-271`):**
     - `can_place` (`@261-288`): `must_survive && can_survive` and `is_unobstructed(placed_state, clicked_pos, CollisionContext::placement(player))`.
     - `is_unobstructed` (`CollisionGetter.isUnobstructed` `@12-43` + `EntityGetter.isUnobstructed` `@28+`): obtain `placed_shape` in `CollisionContext::placement`; if empty -> `true`; otherwise over non-removed `blocks_building` non-spectator entities whose `Aabb::intersects` `placed_shape.bounds().moved(pos)` (including placing player), test `placed_shape.at(pos_vec).intersects(entity.bounding_box)` (reusing `shapes.rs:461-525` `IndirectMerger` `1e-7`!). Any hit -> `false`.
     - `place_block` (`@295-304`): `set_block(clicked_pos, state, 11, 512)` once; `if !instabuild { stack.count -= 1 }`; return `Success`.
   - **#10 Water bucket (`BucketItem.use` `@7-369`, `emptyContents` `@0-461`, `LiquidBlock`/`SimpleWaterloggedBlock`):**
     - Raycast (`@7-13`, `Item.getPlayerPOVHitResult` `@0-49`): `eye_pos` (`pos.y + pose.eye_height()`), `calculate_view_vector(pitch, yaw)` via `math::sin`/`cos`, length `block_interaction_range` (default `4.5`), `traverse_blocks` (`blocks.rs:187`) clipping `Outline` shape and `SOURCE_ONLY` (empty bucket) or `NONE` (filled bucket). Miss -> `Pass`.
     - Permission gate (`@67-93`): `may_interact(pos) && may_use_item_at(offset_pos, dir, stack)` else `Fail`.
     - Filled water bucket (`@94-152`): `target = if hit_block.liquid_block_container { pos } else { offset_pos }`; `empty_contents(player, target, Some(hit))`:
       - `can_replace = state.can_be_replaced_by_water` (`can_be_replaced || !is_solid`); `can_place = can_replace || (state.liquid_block_container && state.can_place_liquid_water)` (`SlabBlock` refuses `TYPE=DOUBLE` per `#10` check.md adv5).
       - `proceed = state.is_air || (can_place && (!player.shift_key_down || hit.is_none()))`.
       - If `!proceed && hit.is_some()`: recurse **once** on `offset_pos` with `hit = None` (`@145-183`).
       - If `nether_evaporates`: return `true` (evaporate).
       - If `state.liquid_block_container` (`SimpleWaterloggedBlock`): `!waterlogged` -> `set_block(target, waterlogged=true, 3)` + `schedule_tick(target, Water, 5)`, return `true`.
       - Else if `can_replace && !state.is_liquid`: `destroy_block(target, drop=true)` (no-op on air), `set_block(target, water[level=0], 11)`, return `true` (or if already water source).
       - On `empty_contents == true`: if `!instabuild`, replace stack with `Bucket` (`getEmptySuccessItem` + `createFilledResult`).
     - Empty bucket pickup (`@224-332`): if hit block is `LiquidBlock(Water)` with `level == 0` -> `set_block(pos, Air, 11)`, return `WaterBucket`; if `SimpleWaterloggedBlock` with `waterlogged == true` -> `set_block(pos, waterlogged=false, 3)` (+ `destroy_block` if `!can_survive`), return `WaterBucket`; else `Fail`. Swap via `createFilledResult` (`!instabuild`).
   - **#1 Pressure plates (`BasePressurePlateBlock`):**
     - `plate_entity_inside(world, pos, entity_box, ...)` (`@0-31`): if `plate.signal == 0`, call `plate_check_pressed(world, pos)`.
     - `plate_tick(world, pos)` (`@0-22`): if `plate.signal > 0`, call `plate_check_pressed(world, pos)`.
     - `plate_check_pressed(world, pos)` (`@0-168`): count non-spectator `!is_ignoring_block_triggers` (`Marker`, `Display`, `Interaction`, `OminousItemSpawner`, `Bat` always true; `ArmorStand.is_marker` conditional) entities strictly `Aabb::intersects` `pos + [0.0625, 0.0, 0.0625]..[0.9375, 0.25, 0.9375]` (`TOUCH_AABB`; wooden & weighted query `Entity`, stone & polished_blackstone query `LivingEntity`); `new_signal` = `15` (if `count > 0`, wooden/stone) or `((count.min(max_w) as f32 / max_w as f32) * 15.0f32).ceil() as u8` (`max_w`: gold `15`, iron `150`); if `new_signal != old_signal`: `set_block(pos, signal=new_signal, 2)`, notify `pos` & `pos - [0, 1, 0]`; if `new_signal > 0`, `schedule_tick(pos, Plate, 20 /* wood/stone */ | 10 /* weighted */)`; player motion/pose/resources untouched.

### 3. Tests proving each edit bit-exact

- **Recorded runs:** No L0/PathBot `.jsonl` recording on `main` exercises `interact` or `use_item_on` (`lab/tracks/runs/{amertystcoral,ametwoodbox,doorcorridor,doorparkourblue,trapdoorparkour,mud}.jsonl` all have `"recordingNote":"... Requires hold-style input"` and no `.jsonl` recording file, per `jobs/flight-corpus/report.md:1.1,4.2`). Thus M2 tests are jar-oracle unit witnesses (matching the `#1, #6, #7, #8, #9, #10` dossiers/checkers):
- **`#6`:** (1) empty main hand on oak trapdoor -> `useWithoutItem` toggles; (2) stone in main hand standing -> trapdoor opens, stone not placed; sneaking -> skips block stages, stone placed; (3) main-hand `Pass` -> `Off` runs `use_item_on` without entering `useWithoutItem` (zero double-toggle).
- **`#7`:** inconsistent-pair door sync (`Adversary A/A2`: clicking either half mirrors `FACING/HINGE/OPEN/POWERED` onto partner, `flags=10`); iron door/trapdoor `Pass` + `0` writes; trapdoor close onto overlapping player AABB (`1` write `flags=2`, `0` entity queries); waterlogged trapdoor open (`flags=2` + `schedule_tick(Water, 5)`); fence gate facing-opposite rewrite (`FACING` changed, non-identity round trip) vs perpendicular (identity round trip).
- **`#8`:** re-click while powered -> `Consume` + `0` writes; stone/polished_blackstone release `+20`, 12 woods `+30`; `FLOOR`/`CEILING`/`WALL(N/E/S/W)` notify exact 2 cells; arrow in `[0.875, 0.9375)` on `WALL(NORTH)` presses (`[0.875, 1.0]`), releases at `+30` (`[0.9375, 1.0]` miss), re-presses oscillation; stone ignores arrows.
- **`#9`:** feet-cell FAIL (`stack` unchanged); below-feet SUCCESS (`flags=11`, survival `64->63`, `1->0`, instabuild `64->64`); X/Z razor-edge bisect `g = 8.80790e-8` PASS vs `g = 8.80791e-8` FAIL (using `0.6f32 / 2.0`); vertical head touch `1.7999999523162842` PASS (`-4.77e-8` gap) vs `1.0002e-7` overlap FAIL; spectator entity ignored vs second non-spectator obstructs; `ScaffoldingBlock` placement shape empty (`CollisionContext::placement`).
- **`#10`:** air-face place (`flags=11`, `bucket` swap) & source pickup (`flags=11`, `water_bucket` swap, world identical); `level=1` & `level=3` pickup refused; bottom slab waterlog from top AND north face (`flags=3`, `delay=5`, dry pickup restoration); `TYPE=DOUBLE` slab non-sneaking redirect above (`flags=11`); sneaking slab redirect (`flags=11`); `SHORT_GRASS` `destroy_block(drop=true)` before `flags=11` water.
- **`#1`:** `OAK_PRESSURE_PLATE`, `STONE_PRESSURE_PLATE`, `LIGHT_WEIGHTED_PRESSURE_PLATE` (`15`), `HEAVY_WEIGHTED_PRESSURE_PLATE` (`150`) across counts `{0,1,11,15,16,150,151}`; `TOUCH_AABB` `y = pos.y + 0.25` (no press) vs `nextDown(0.25)` (press); `Entity` vs `LivingEntity` on stone; re-entry while powered `0` writes; weighted `1->2` re-power writes `flags=2`; `20t`/`10t` release; `player.bits()` unchanged.

### 4. Conflicts & frozen gates

- **Batch overlap:** M2 modifies `engine/law/src/world.rs` (adding `CollisionContext::is_placement`, possibly `Block::placement_shape`). Must keep `jobs/1.2-footprints/verify.ps1:48-53` exact string-replacement mutant sites untouched (`include(min,max,!self.blocks.unbounded)`, `self.visit_collisions(bounds, context, |_| false)`, `!self.visit_collision_cells(bounds,lo.map(|v|v-1),hi.map(|v|v+1), |p| {`, `self.blocks.fluids || self.regions.iter().any(|(_,_,b)|b.fluid.kind!=FluidKind::Empty)`, `self.blocks.inside_effects || self.regions.iter().any(|(_,_,b)|b.has_inside_effects())`), AND keep `blocks.rs:119` (`mutant-fluid-wall-above`) untouched.
- **Cross-batch dependency with M3:** If `Block` gains `placement_shape` in M2 AND `snapshot.rs` reads a 22nd column from `ModelTable.java` in M3, M2 + M3 must agree on `Block`'s schema (and `engine/law/src/pattern.rs:384-390` must destructure the new `Block` field!). **Recommendation:** Keep `Block` in `world.rs` focused on movement + `placement_shape` (if needed), and keep Step-5 block-state/interact metadata (`Door`, `TrapDoor`, `FenceGate`, `Button`, `PressurePlate`, `Slab` type/waterloggable) in `interact.rs` / a Step-5 block-state table rather than inside `models.tsv` / `Block`!

### 5. Open questions for the integrator

1. **Where Step-5 block-state identity & properties live (`interact.rs` / `world`):** `WorldSnapshot` (`engine/world/src/snapshot.rs:107`) currently stores only `u16` model IDs (deduplicated physical models) and discards block names/properties, AND `engine/world/tests/blocks.rs:101` asserts `stone_button[face=wall,facing=east,powered=true]` and `stone_button[face=floor,facing=north,powered=false]` share the same `Block` pointer!
   - *Options:* (A) Put `door`/`trapdoor`/`gate`/`button`/`plate`/`waterlogged` properties directly on `law::Block` and split rows in `models.tsv`, OR (B) keep `law::Block` + `models.tsv` purely physical (plus `placement_shape`), and give the Step-5 mutable world (`interact.rs` / `world` Step-5 loader) its own compact `BlockStateId` / interact-state table alongside `Block` model IDs.
   - *Recommendation:* **(B)** — preserves all 519 physical models, keeps `world/tests/blocks.rs:101`, `world/tests/pattern.rs:20,35`, and `law/tests/footprints.rs` green without inflating `PatternTable` or `models.tsv`.
2. **Who owns `CollisionContext` struct-literal fixes (`speed_differential.rs`, `blocks.rs`, `collision_cells.rs`, `flight.rs`)?**
   - *Options:* (A) M2 derives `Default` on `CollisionContext` and updates the 4 struct-literal call sites (one line each in `engine/world/tests/` and `engine/plan/src/flight.rs` — note: touches 1 file in `world/tests` and 1 in `plan/src`), OR (B) M2 adds `is_placement: bool` and updates all workspace callers, with M2 landing before M3's Rust edits if any.
   - *Recommendation:* **(A)** in M2 (none of those 4 lines are in `ModelTable.java` or `snapshot.rs`, so M2 and M3 remain file-disjoint!).

---

## Batch M3 — Export table placement shapes

Scope: `engine/world/tools/ModelTable.java`, `engine/world/src/snapshot.rs` (lens `#9`, plus optional Step-5 props) — and regenerated `engine/world/data/*.{tsv,txt}` if schema changes.

### 1. Premises

| Item | Status | Evidence on `main` |
|---|---|---|
| `ModelTable.Context` passes `placement = false` | EXISTS | `engine/world/tools/ModelTable.java:50-54` (`super(descending, false, feet, ItemStack.EMPTY, false, null)`) |
| `ModelTable` schema width | EXISTS (21 cols) | `engine/world/tools/ModelTable.java:125-133` outputs 21 tab-separated columns; `engine/world/src/snapshot.rs:71` asserts `f.len() == 21` |
| `ModelTable` hooks for `BasePressurePlateBlock`, `ButtonBlock` | EXISTS (`"world"`) | `engine/world/tools/ModelTable.java:37-38` |
| `ModelTable` flags (`ladder_facing`, `open_trapdoor_facing`, `wall_or_gate`, `scaffolding`, `water_block`, `fluid`) | EXISTS | `engine/world/tools/ModelTable.java:114-128`; `snapshot.rs:73-95` |
| `placement_shape` column in `ModelTable.java` / `snapshot.rs` | MISSING | Not found in `ModelTable.java:110-133` or `snapshot.rs:69-96` |
| `WorldSnapshot::load` block-state retention | MISSING | `engine/world/src/snapshot.rs:107, 152-157` maps `BlockState` -> `u16` model ID only |
| `blocks.rs:101` button sharing assertion | EXISTS (conflicts with optional button props in `models.tsv`) | `engine/world/tests/blocks.rs:100-101` asserts `std::ptr::eq` for `stone_button` across `face`/`facing`/`powered` |
| `pattern.rs:20,35` model symmetry assertions | EXISTS | `engine/world/tests/pattern.rs:20, 35` asserts 1-to-1 digest per model and exact set of asymmetric models (`cake`, `candle`, `sniffer_egg`) |

### 2. Exact edits

1. **`engine/world/tools/ModelTable.java`:**
   - Add a `PlacementContext` (or parameterize `Context(boolean descending, boolean placement, double feet)`) passing `true` as the 2nd argument to `EntityCollisionContext(...)` (`CollisionContext.placementContext` `@75-101`).
   - Add `placement` shape export column (`f[21]`): `shape(s.getCollisionShape(LOCAL, BlockPos.ZERO, PLACEMENT_CONTEXT))` (or only if `!= collision.get(0)`, or always as 22nd column; note `ScaffoldingBlock` returns `Shapes.empty()` and `PowderSnowBlock` reads `isPlacement()` per `#9` check.md:21).
   - **Do NOT** add `face`/`facing`/`powered`/`open` interact properties into `models.tsv` (see Open Question 1 / Conflicts). If Step-5 worlds need `can_open_by_hand` / `waterloggable` / `button` / `plate` lookup, export them to a separate sidecar TSV or build them in `world` without altering `models.tsv`'s physical-model equivalence classes.
2. **`engine/world/src/snapshot.rs`:**
   - Update `parse_model` (`:69-96`): change `assert_eq!(f.len(), 21, ...)` (`:71`) to `22` (or conditional) and parse `f[21]` into `Block::placement_shape: Option<VoxelShape>` (e.g. `(!s.is_empty() && s != f[15])` or always `Some(shape(f[21]))` / `None` when equal to `shape`).
   - Regenerate `engine/world/data/{models.tsv, maps.tsv, blocks.tsv, defaults.txt, version.txt, census.tsv}` using the 26.2 jar so `WorldSnapshot` and `model()` match the new schema.

### 3. Tests proving each edit bit-exact

- Existing `engine/world/tests/blocks.rs` (`census_puts_every_registry_state_in_one_bucket`, `rule_flags_and_reasons`, `julien_tracks_load`, `new_family_shapes_equal_the_jar_bit_for_bit`), `engine/world/tests/pattern.rs` (`the_key_separates_models_and_finds_their_symmetry`), and `engine/world/tests/collision_cells.rs` (`powder_snow_is_never_skipped_as_a_block_that_cannot_collide`).
- Add a witness test in `engine/world/tests/blocks.rs`: load `scaffolding[bottom=false,distance=0,waterlogged=false]` and `powder_snow`, verify `Block::shape` (movement `CollisionContext::of`) vs `placement_shape` (`CollisionContext::placement`) against `ModelTable.java --shapes` / jar `CollisionContext.placementContext` (specifically `scaffolding` movement top plate `[0, 0.875, 0]..[1, 1, 1]` vs placement `empty`, and `powder_snow` placement shape).

### 4. Conflicts & frozen gates

- **Schema lockstep with M2 (`Block::placement_shape`):** `snapshot.rs:78-96` constructs `law::Block { ... }`. If M3 writes `placement_shape: ...` on `Block`, **M2 must land `Block::placement_shape` first** (or M3 adds the column only after M2 defines the field in `world.rs` + `pattern.rs`).
- **Breaks `blocks.rs:101` & `pattern.rs:20,35` if optional props go into `models.tsv`:** Putting button `face`/`facing`/`powered` or door `open`/`facing`/`hinge` into `models.tsv` breaks `engine/world/tests/blocks.rs:101` (`std::ptr::eq` on `stone_button`) and `engine/world/tests/pattern.rs:20,35`.
- **`PowderSnowBlock` `never_collides()` gate (`collision_cells.rs:103-110`):** untouched as long as `placement_shape` does not alter `Block::collision_shapes()` or `never_collides()`.

### 5. Open questions for the integrator

1. **What does `PowderSnowBlock.getCollisionShape` return when `isPlacement() == true` (in bytecode) across `descending`/`above`/`leather_boots`?** (`#9` check.md:40 verified `PowderSnowBlock` calls `isPlacement()` in bytecode, not at runtime; `ModelTable` only runs fixed `CONTEXTS`).
   - *Options:* (A) Check `PowderSnowBlock.getCollisionShape` bytecode via `javap` first (takes 15s) to see whether `isPlacement()` returns `Shapes.empty()` / `Shapes.block()` unconditionally or still reads `entity`/`descending`, OR (B) have `ModelTable.java` check all 5 `CONTEXTS` with `placement=true` and verify whether `PowderSnowBlock` differs across them.
   - *Recommendation:* **(A) + (B)** — inspect `PowderSnowBlock.getCollisionShape` once before freezing M3's `ModelTable.java` column layout.
2. **Where do `can_open_by_hand` / `waterloggable` / `button` / `plate` metadata live (`ModelTable.java` sidecar vs Rust registry)?**
   - *Recommendation:* Separate sidecar TSV (e.g. `interact.tsv`) or Rust static table in `world`/`interact.rs` — never in `models.tsv`, so `models.tsv` keeps its 519 physical models and `blocks.rs:101` stays green.

---

## Post-M1+M2 follow-up — Hook invocation in `inside_segment`

Scope: `engine/law/src/movement.rs` (`inside_segment` `:1020-1061`, `inside_block` `:1064-1096`) (lenses `#1`, `#8`).

### 1. Premises

| Item | Status | Evidence on `main` |
|---|---|---|
| `inside_segment` swept walk & `inside_shape` test | EXISTS | `engine/law/src/movement.rs:1020-1061` (`for_each_intersected`, `visited.insert(p)`, `inside` check `:1038`) |
| `world_hook: bool` on `Block` (`BasePressurePlateBlock`, `ButtonBlock`, `TripWireBlock`) | EXISTS | `engine/law/src/world.rs:214` & `engine/world/tools/ModelTable.java:37-38` |
| `has_inside_effects` omits `world_hook` | DIFFERS (not flagged) | `engine/law/src/world.rs:257` `Block::has_inside_effects` checks `inside != None || stuck_multiplier.is_some() || fluid.kind != Empty` — `world_hook` plates/buttons have `has_inside_effects == false`, so `movement.rs:995` (`if !world.has_inside_effects() { return Ok(contact); }`) **skips `inside_segment` entirely** on worlds containing only pressure plates or buttons! |
| Mutable world argument to `tick_movement` / `inside_segment` | MISSING | `engine/law/src/movement.rs:119, 1020` take `&impl World` (immutable trait); `RecordingWorld` (`footprint.rs:1-2`) requires immutability |

### 2. Exact edits

1. **`engine/law/src/movement.rs` (`inside_segment` `:1045-1050` & `apply_block_effects` `:995`):**
   - Because `tick` (`:62`, `:119`) takes an immutable `&impl World`, the hook follow-up either:
     - emits `WorldHookEvent { pos, entity_box }` (parallel to `ContactDamage` returned from `tick_with_contact` `:75-79`) when `inside` is true (`BasePressurePlateBlock` uses default `Shapes.block()`, `ButtonBlock` uses button shape) and `block.world_hook` is true, OR
     - introduces a separate `tick_interact` / `Step5World` entry point calling `interact::plate_entity_inside` / `interact::button_entity_inside` from `inside_segment`.
   - **Critical fix in `world.rs:257` (`Block::has_inside_effects`):** When live hook dispatch is enabled on a world, `has_inside_effects` (or `World::has_inside_effects` on a Step-5 world) MUST NOT skip `world_hook` blocks at `movement.rs:995`! (In `BoxWorld` / `WorldSnapshot` for pure movement `law::tick`, keep `has_inside_effects` excluding `world_hook` so `speed_differential.rs` and `butoonpressureplates` fast-path movement remain unchanged when hooks aren't requested.)

### 3. Tests proving each edit bit-exact

- **Recorded run:** `C:\Users\PASCAL\Desktop\path tracing\recordings\track-butoonpressureplates-PathBot-1.jsonl` (`lab/tracks/runs/butoonpressureplates.jsonl:2`, `sprintWindow: 0`, `T=198`): proves that stepping across buttons and pressure plates leaves `PlayerState` movement bits untouched while (with hook recording enabled) firing `plate_entity_inside` synchronously on swept plate entry.
- **Unit witnesses (`#1`, `#8`):** player moving onto `OAK_PRESSURE_PLATE` (`TOUCH_AABB` intersect) fires `plate_entity_inside` on the entry tick (press latency `0`, `flags=2`, `schedule_tick +20`), releases only on scheduled tick after exit; moving over `STONE_BUTTON` / `OAK_BUTTON` does **not** press it (non-arrow `Player`); `player.bits()` matches movement-only `law::tick` bit for bit.

### 4. Conflicts & frozen gates

- **Merge order:** Must run **after both M1 and M2** merge (`M1` modifies `movement.rs`; `M2` creates `interact.rs`).
- **Frozen gates:**
  - `jobs/1.2-footprints/verify.ps1:53` (`inside-flag` mutant matches `'self.blocks.inside_effects || self.regions.iter().any(|(_,_,b)|b.has_inside_effects())'` in `world.rs:583`, tested by `speed_differential.rs:66` `tick_equals_forced_fluid_and_inside_hooks`!). Changing `Block::has_inside_effects` to include `world_hook` would not break `verify.ps1` string matching, **unless** a `world_hook` block changes `tick()`'s `PlayerState` — since plates/buttons never mutate `PlayerState`, `tick_equals_forced_fluid_and_inside_hooks` remains bit-equal, and `inside-flag` mutant still fails on cobwebs/powder snow/honey!

### 5. Open questions for the integrator

1. **Immutable `law::tick(&mut PlayerState, &impl World, &Input)` vs mutable Step-5 `World`:** `RecordingWorld` (`footprint.rs:1-2`) and `PatternTable` assume `&impl World` never mutates during `law::tick`.
   - *Options:* (A) Keep `law::tick` pure/immutable and return/collect `Vec<BlockPos>` (or callback) of `world_hook` intersections from `tick_with_hooks(...)` for the Step-5 runner/timeline to pass to `interact::plate_entity_inside` (and keep `has_inside_effects` unchanged in `law::tick`), OR (B) add a new `tick_mut(&mut PlayerState, &mut impl MutableWorld, &Input)` wrapper in `interact.rs` sharing `inside_segment` logic.
   - *Recommendation:* **(A)** — zero churn to `RecordingWorld`, `PatternTable`, `WorldSnapshot`, `words`, or `plan`, and trivially preserves bit-equality on `law::tick`.
