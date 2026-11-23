# The law's grouping table

Opus, 23 November. Read-only audit of `D:\path-tracing-wt\port-fold` at `38a49808`, `engine/law/src/`: **20,185 code lines / 29,087 total** (`perl scripts/law-lines.pl`), 48 files, 309 struct/enum types, 79 `*tick*` functions, 40 `rules.tsv` blocks over the 1,196-row census. Nothing was edited. One probe ran outside the worktree (a scratch crate with `law` as a path dependency, target dir `D:\rust\scratch-layout-target`) to read `PlayerState`'s layout.

**Method.** Every line of every file is assigned to exactly one group, by line range, with the counting rules of `law-lines.pl`. The ranges are in the appendix. The groups plus Glue sum to 20,185 with no line uncovered and no line counted twice (`alloc.pl`, checked). Groups follow the transition shape (what recurrence or state machine runs), never the Mojang class or the file. "Rebuilt" is a bottom-up Fermi estimate of the Rust code lines the group costs on the generator basis, tests excluded. Rows are inert data (constants, shapes, flags, tags) and cost 0 code lines. Guards that are logic stay Rust and are counted.

## 1. Inventory (what the law holds)

- **Bodies:** the player; boat; minecart; falling block (sand, gravel, anvil, concrete powder); item entity; arrow; throwables (snowball, egg, ender pearl); wind charge; firework (attached boost); fishing hook (cast and pull only); trident riptide and spin; mounts (horse, pig, strider, camel, happy ghast); a generic living body (the crowd tests' pig); shulker (as a collider).
- **Blocks with behaviour:** redstone wire, lever, buttons (stone, wood), pressure plates (standard, weighted), tripwire and hook, repeater, comparator, observer, torch, target, daylight detector, copper bulb, dispenser (edge only), doors, trapdoors, fence gates, piston, sticky piston, head, moving piston, water, lava, scaffolding, slab, stairs, ladder, falling blocks, bubble column, cauldrons, big dripleaf, shulker box, bamboo and dripstone (offset shapes), cobweb, powder snow, honey, slime, soul sand, ice, magma, fire, soul fire, campfire, cactus, berry bush, wither rose, nether and end portals, hopper (suck box), detector rail, turtle egg, farmland, sculk sensor and shrieker, redstone ore, lily pad, frogspawn, eyeblossom, bed, respawn anchor, hay.
- **Tick routines:** `movement::tick_movement` (the player); `SimWorld::step_tick` (scheduled block ticks, then fluid ticks, then block events, then entities, then block entities); one free tick per body class; `MobEffectInstance::tick_server`; `food_tick`; `tick_air`; `LevelTicks::tick`; `MovingPiston::advance`; `Minecart::tick`; boat `tick_body`; the effect, freeze and resource events.
- **Planner interface inside `law`:** codec, footprint, pattern, proof, planes, state answers, state reads, and the solve meter. It is not dynamics, but it lives here and counts here.

## 2. The groups

### Summary

| # | Group (transition shape) | Maps to | Exactness | Lines now | Rebuilt |
|---|---|---|---|---:|---:|
| 1 | Cell rewrite rules: (cell, neighbourhood, trigger) → (cell', schedules, neighbour updates) | `cell` rows + guards | behaviour-exact (redstone); server-exact (breaking, placing) | 3,666 | 950 |
| 2 | Body recurrence: v += input + g; sweep; v ×= drag·friction | `body` | bit-exact (player, boat, projectiles); behaviour-exact (falling block and item in the world) | 1,570 | 600 |
| 3 | Planner interface: canonical bytes, footprints, patterns, proofs | plumbing | exact encoding (not a tier) | 1,418 | 730 |
| 4 | Calendar and neighbour updates: the order in which rewrites run | `cell` (the engine) | behaviour-exact | 1,316 | 400 |
| 5 | Pistons: push structure, moving block, entity push | kernel | behaviour-exact | 1,221 | 500 |
| 6 | Contact, body–cell: the swept cells, overlap → effect rows | `contact` (cell face) | bit-exact (motion); server-exact (damage, fire) | 1,007 | 270 |
| 7 | Ledger: amount through a mitigation chain, cooldown window, food chain | **`ledger` (new)** | server-exact | 923 | 300 |
| 8 | World interface and block tables | plumbing | — | 881 | 320 |
| 9 | Fluids: spread, level, current, immersion | kernel | behaviour-exact (spread); bit-exact (current, immersion) | 866 | 510 |
| 10 | Sweep and shape algebra: AABB through boxes, step-up, edge back-off | `sweep` | bit-exact | 855 | 540 |
| 11 | Contact, body–body: pair push over a spatial index, cramming | `contact` (pair face) | behaviour-exact | 818 | 250 |
| 12 | Kicks: v' = a·v + b (launch at a = 0; knockback, boosts, pushes) | `body` (its input term) | server-exact (server RNG, server knockback); bit-exact (firework) | 736 | 270 |
| 13 | Player tick core: phase order and the flag machines (sprint, crouch, swim, pose, flight) | composes the generators | bit-exact | 664 | 330 |
| 14 | Elytra glide | kernel | bit-exact | 626 | 220 + table as data |
| 15 | Portals: transit transform, process clock, PortalForcer | kernel | behaviour-exact; server-exact (transit) | 481 | 240 |
| 16 | Shape rows of state: offsets, lid progress, tilt, fill | `cell` model rows + `clock` | bit-exact (they feed collision) | 446 | 110 |
| 17 | Numeric scars: sine table, `Mth`, `JavaRandom`, `atan2`, `Vec3`, `Direction` | plumbing | bit-exact | 404 | 250 |
| 18 | Clocks: counter, step, threshold, reset, firing | `clock` | server-exact | 391 | 150 |
| 19 | Rails: motion projected onto a track polyline | kernel (new name) | behaviour-exact | 380 | 220 |
| 20 | Attribute fold: base ⊕ ordered modifiers (speed, enchantments, mining) | profile rows + one fold | bit-exact (speed); server-exact (mining) | 379 | 130 |
| 21 | Ray: DDA through cells, clip against shapes | `ray` | bit-exact | 328 | 220 |
| 22 | Wire graph: redstone power propagation | kernel | behaviour-exact | 254 | 180 |
| 23 | Game mode, abilities, death and respawn | rows + small guards | server-exact | 228 | 80 |
| 24 | Inventory: slot fill, merge, durability, consumables | rows + one fold | server-exact | 206 | 90 |
| — | Glue / unclassified: `lib.rs`, `basis/mod.rs`, `bin/bench.rs` | — | — | 121 | 60 |
| | **Total** | | | **20,185** | **7,920** |

**24 groups plus Glue.** 12 map to the generator basis, 6 are named kernels, 4 are plumbing (planner interface, world interface, numeric scars, Glue), and 2 are rows with small guards (game mode, inventory).

### Changes the shapes demand of the basis

- **Add `ledger`.** One shape is spread across 9 files: an amount goes through a chain of monotone maps (armour and toughness, then resistance, then the enchantment EPF, then absorption, then health), gated by the invulnerability window (`accept_raw_hit`: the full hit when the cooldown is 10 or less; above it, only the excess over the last hit). It pairs with the food chain (exhaustion over 4 costs saturation, then food). Rows: damage types (`bypasses_armor`, `bypasses_resistance`, `is_fire`, `is_fall`, exhaustion), fall multipliers, food rows. Today two hurt pipelines exist: `movement::apply_server_damage → player::actually_hurt`, and `player::hurt_server` / `hurt_server_mitigated`, which only the tests call. Boat hurt and combat each carry their own.
- **Merge launches and impulses into `body`.** A launch is a kick with a = 0. Knockback, extra knockback, punch, wind burst, explosion push, firework boost, riptide, piston and shulker push are all v' = a·v + b·dir (+ cap). They become one kick row family on the body's input term, not a generator.
- **`contact` has two faces:** body–cell (the move's swept cells walked in `cells_along` order, effects collected in step order) and body–body (pairs from a spatial index, in Mojang's section order). Both are "overlap → row".
- **`cell` is two things:** the engine (calendar, neighbour queue, set-with-flags, storage: group 4) and the rules on it (group 1). A rule's guard stays Rust. Only its constants become rows.
- **The player's flag machines are the same shape as a block's state machine** (state + guards → new state + timers), in a different coat. They stay Rust guards (no rule scripts), with their timers on `clock`.
- **Rails is a kernel.** The minecart's position is projected onto a piecewise-linear track whose exits depend on the rail shape. Velocity is redirected along the segment, and slope, boost and brake are per-shape scars. That is not affine in v: it needs a projection and a direction state.
- **The attribute fold is one function, not a generator.** `AttributeInstance`'s base plus modifiers runs in add, multiply-base, multiply-total order, sorted by id hash. It derives the rows' parameters (speed, jump strength, mining speed) once per profile change.

### Per group: members, parameter columns, named scars, sources, witnesses

**1. Cell rewrite rules (3,666 → 950).**
- *Members:* signal toggles (door, trapdoor, fence gate, lever, copper bulb, dispenser edge); diodes (repeater, comparator with modes and container analog); observer; torch with burnout; target; daylight; presence sensors (pressure plates, buttons, detector rail, tripwire hook and wire line, up to 42 blocks); placement (slab, stairs shape, door hinge, trapdoor, ladder, scaffolding distance and placement); buckets (pick up, empty, waterlog); breaking (break-packet protocol, destroy side effects, partner halves); survival rules; falling-block landing (concrete, anvil degrade); big dripleaf tilt; boat placement.
- *Parameter columns:* block id, delay, tick priority, signal map (count → strength), sensor box, recheck period, toggle flags, survival rule, push reaction, hardness, drop flags.
- *Named scars:* comparator subtract mode; repeater locking; torch burnout (8 toggles in 60 ticks); tripwire "disarmed" with shears; the weighted plate's `ceil(min(count, max) / max × 15)`; stairs inner/outer corners; the door hinge's 4-way neighbour vote; scaffolding distance 7; waterlogged trapdoor fluid tick 5; partner removal for door and bed halves.
- *Sources:* `world_hooks.rs` 1,313, `sim.rs` 755 (`on_remove`, `on_place`, `handle_neighbor_event`, `can_survive`, `run_block_tick`, lever, button, plate, tripwire and bucket application, bucket view), `place.rs` 434, `mining.rs` 360, `redstone.rs` 345, `interact.rs` 164, `falling.rs` 98, `boat.rs` 96, `block_hooks.rs` 83, `neighbor.rs` 18.
- *Rebuilt:* signal toggles one function + rows (~40); diodes, observer, torch, target, daylight (~185); one presence-sensor mechanism (~60) + the tripwire line (~70); placement (~165); buckets (~80; sound and game-event emission dropped, see below); breaking (~120); survival and dispatch (~150); landing (~40); dripleaf (~20); boat place (~30).
- *Witnesses:* `world_hooks.tsv` (PLATE, BUTTON, TRIPWIRE traces and BUCKET_CASE are boundary rows: an edit in, the ordered SET/SCHED/NEIGHBOR trace out); `world_sim.tsv` A_CASCADE (boundary, multi-tick); `cauldron_redstone.tsv` and `projectiles_redstone.tsv` (REPEATER, OBSERVER, TORCH, COMP, DAYLIGHT, TARGET: pinned to helper signatures); `place.rs` and `interact.rs` in-file tests (internal types). Placement, game-mode restriction, the break protocol and most redstone helpers have no caller but tests.

**2. Body recurrence (1,570 → 600).**
- *Members:* the player's travel (air, ground friction, water, lava, creative flight, climbing, levitation, slow falling, dolphin's grace, depth strider, soul speed); mounts (ridden input and speed per kind); boat (status, buoyancy, ground friction scan, paddles); item entity; falling block; arrow; throwable and pearl; wind charge; the world sim's per-class entity loops.
- *Parameter columns:* gravity, water gravity, air drag, vertical drag, water drag, order (G-M-D, G-D-M, M-G-D), acceleration, friction rule, jump power, step height, speed factor, the efficiency lerp target (0.546 drag, `speed` acceleration).
- *Named scars:* `0.21600002 / f³`; the `aiStep` zeroing (9e-6, 0.003); the fluid-falling `-0.003` snap; the jump out of water (0.3 when the lifted box is free); lava height 0.4; the climb clamp at 0.15; the arrow's drag split around the move; the item's rest cadence (moves every 4th tick at rest) and fluid rise; the boat's underwater, in-water, on-land and in-air status with its water-level scan; the minecart's 0.005 water gravity.
- *Sources:* `boat.rs` 385, `sim.rs` 300 (falling blocks, boats, pearls), `movement.rs` 265, `mounts.rs` 171, `basis/body.rs` 95, `pearl.rs` 86, `projectiles.rs` 80, `items.rs` 79, `player.rs` 69 (`travel_in_air_forward_tick` and `get_friction_influenced_speed`: twins of `movement.rs` kept for in-file tests), `explosion.rs` 23, `falling.rs` 17.
- *Rebuilt:* generator 110; living travel scars 180; boat 100; mounts 40; free-body scars 60; one entity loop over body rows 60; falling landing hand-off 30; rest for pearl, arrow and wind charge 20.
- *Witnesses:* `replay.rs` (29 recordings, per-tick PlayerState in → out: boundary); `falling.tsv`, ARROW_TRAJ, THROW_TRAJ, MC_TICK, BOAT_PADDLE, BOAT_STATUS, FLY_TRAVEL, EFFECT_DOLPHIN, EFFECT_SLOWFALL_* (boundary trajectories); `mounts_breath_void.tsv` horse_input, strider_speed, ghast_in (pinned to `mounts::*`, which only tests call).

**3. Planner interface (1,418 → 730).**
- *Members:* `codec.rs` 272, `pattern.rs` 362, `footprint.rs` 309, `state.rs` 153 (wire and bits), `codec_proof.rs` 145, `codec_planes.rs` 68, `codec_state.rs` 68, `solve_meter.rs` 21, `state_reads.rs` 20.
- *Rebuilt:* the state's wire and answers are derived from the one `player_state!` declaration (~90); codec and interner (~150); footprint recording (~150); pattern table (~200); proof (~80); planes (~40); reads and meter (~20).
- *Witnesses:* `plan` crate tests (not counted here).

**4. Calendar and neighbour updates (1,316 → 400).**
- *Members:* `LevelTicks` and `LevelChunkTicks` (time, priority, sub-tick order, the per-chunk drain); `NeighborQueue` / `CollectingNeighborUpdater` (stack-then-queue, chain limit); update flags and update-or-destroy; `SimWorld` storage, scheduling, `set_cell_with_flags`, `step_tick`'s phase order; `SimCell` / `CellState` / `WorldMut`.
- *Parameter columns:* tick priority, delay, update flags, the chain limit (512, `DEFAULT_LIMIT`), neighbour order (W, E, D, U, N, S), shape-update order.
- *Sources:* `sim.rs` 536, `ticks.rs` 326, `neighbor.rs` 302, `basis/cell.rs` 152.
- *Rebuilt:* calendar (~100), neighbour queue and flags (~95), storage, set and phases (~150), cell = census row + packed properties (~60). The class-major calendar the map names would replace `LevelTicks`'s HashMap containers, keeping the drain order.
- *Witnesses:* `world_sim.tsv` (63 multi-tick boundary rows); `ticks.rs` and `neighbor.rs` in-file tests (internal types).

**5. Pistons (1,221 → 500), kernel.**
- *Why a kernel:* the push structure is a graph search: a line, then sticky branches, with 12 blocks at most and a destroy list in Mojang's order. Entities are pushed by a moving box over 0.5-progress steps. Neither is a recurrence over a row.
- *Parameter columns:* push reaction, sticky pairs (slime, honey), max push 12, progress step 0.5, extend ticks 2, push offset 0.01.
- *Named scars:* slime and honey don't stick to each other; the head's short flag; the base-fix of entities inside the base; the moving piston's two-box shape; the `final_tick` orphan restore.
- *Sources:* `piston.rs` 714, `sim.rs` 507 (check-extend, block events, insert and execute move, final tick, block-entity tick, moving-piston boxes and entity motion, `SimPistonView`, head boxes, piston signal).
- *Rebuilt:* resolver 150, trigger 60, moving block entity 50, entity push 120, world binding 120.
- *Witnesses:* `world_sim.tsv` C_PISTON (boundary); `piston.rs` 12 in-file tests (internal types).

**6. Contact, body–cell (1,007 → 270).**
- *Members:* entityInside, stepOn and fallOn: cobweb, powder snow, honey slide, bubble column (inside and above), fire, soul fire, campfire, lava, cactus, berry bush, wither rose, end and nether portal refusal, magma (`DamageUnlessCareful`), slime bounce, the step-ordered effect collector (freeze, clear freeze, ignite, lava ignite, extinguish, lava hurt); non-player hooks (turtle egg, farmland trample, detector rail, hopper box, sculk, redstone ore, lily pad, frogspawn, eyeblossom, crop ravager); `check_world_hooks`.
- *Parameter columns:* stuck multiplier (x, y, z), velocity clamp (min, max, add), effect op, damage type, schedule delay, entity kinds, sensor box.
- *Named scars:* the honey slide's `old_y` reconstruction (`vy / 0.98 + 0.08`) and the 0.4375 + width/2 edge; the bubble column's open top (1.8 / 0.7, -0.9 / -0.3); powder snow sticks only when the feet cell is powder snow; the berry bush hurts only past 0.003 of movement; the 16-step budget and the extra `budget ≤ 0` pass; the collector's per-step flush order; `fire_grace_tail`.
- *Sources:* `block_hooks.rs` 354, `movement.rs` 315, `player.rs` 158, `world_hooks.rs` 73, `world.rs` 60, `sim.rs` 47.
- *Rebuilt:* traversal 70, scars 110, non-player hook rows and the evaluator 60, sensors 30.
- *Witnesses:* BUBBLE (171) and BUBBLE_COLUMN (72), COLLECTOR_*, FIRE_*, CAMPFIRE_INSIDE, HURT_BERRY, HURT_CACTUS, `landings.tsv`, HOOK_* (mostly pinned to `evaluate_entity_inside` / `evaluate_step_on`, which only tests call); `replay.rs` (boundary).

**7. Ledger (923 → 300), new generator.**
- *Members:* `accept_raw_hit`, the mitigation chain, `apply_server_damage`; fall power and damage; the food chain and regeneration; peaceful regeneration; server resource events; eat; melee attack resolution (crit, sweep, strength scale, shield angle and disable, mace smash and density); arrow damage; explosion damage; boat hurt and decay; falling-block damage (anvil, stalactite).
- *Parameter columns:* damage type flags, exhaustion per type, crit multiplier 1.5, shield angle 90°, fall multiplier per block, nutrition, saturation modifier.
- *Named scars:* `amount - last_hurt` inside the window; the NaN knockback that does not return; the bed halves fall distance; the dripstone ×2 with its fall distance; starvation stops at 10 / 1 health by difficulty; `Math.round` in the movement exhaustion centimetres.
- *Sources:* `movement.rs` 250, `combat.rs` 254, `player.rs` 238, `boat.rs` 62, `projectiles.rs` 34, `explosion.rs` 26, `falling.rs` 21, `items.rs` 21, `mining.rs` 17.
- *Rebuilt:* generator 70, chain scars 60, food 45, combat 90, rest 35.
- *Witnesses:* MITIGATION, ATK_SCALE, KB, ARROW_DMG, EXP_*, REGEN_GUARD, EXHAUSTION, FALL_* (pinned to helpers); `support/server_resources.rs` cadences.

**8. World interface and block tables (881 → 320).**
- *Members:* the `World` trait (162 lines, with the footprint `observe_*` hooks), `Block` (35 fields), `BoxWorld`, `BlockMap`, `CollisionContext`; `basis/cell.rs`'s loading of `models.tsv`, `maps.tsv`, `blocks.tsv`, `census.tsv` and `rules.tsv`; `WorldOverlay` (the edit overlay `sim` reads through).
- *Sources:* `world.rs` 436, `sim.rs` 231, `basis/cell.rs` 214.
- *Rebuilt:* one `World`/`WorldMut` over cell ids with reversible deltas (the overlay folds into it) ~140, the table loader ~100, the `Block` row ~40, the test world ~40.

**9. Fluids (866 → 510), kernel.**
- *Why a kernel:* the spread is a bounded search (slope distance 4 for water, 2 or 4 for lava, with a hole cache) over a level field, then a write with water–lava reactions. The current is a gradient of surface heights over 4 neighbours. Neither is a row on one generator.
- *Parameter columns:* drop-off, slope distance, tick delay (5 water, 30 or 10 lava), source conversion, current scale (0.014 water, 0.007 / 0.00233 lava).
- *Named scars:* the 0.0045 minimum current when nearly still; the height < 0.4 scaling; falling fluid height 8/9; lava into water → stone/cobblestone/obsidian; the `can_pass_through_wall` faces.
- *Sources:* `fluid.rs` 354, `sim.rs` 329, `movement.rs` 93 (`update_fluid`, `contains_fluid`), `world.rs` 90 (`Fluid::surface_height`, `current`).
- *Rebuilt:* spread 250, current and height 70, immersion 70, world binding 120.
- *Witnesses:* `world_sim.tsv` B_FLUID (boundary); `oracles/1.9e-fluid-flow-oracle` example; `replay.rs` water runs (boundary).

**10. Sweep and shape algebra (855 → 540).**
- *Members:* `basis::sweep` (axis order, the kernel, step-up candidates), `VoxelShape` / `ShapeView` / `Aabb`, back-off from the edge, `can_fall`, pose fits, supporting block, closest-space push-out, the minor-collision angle, boat dismount and rider rotation, pose dimensions.
- *Parameter columns:* step height, pose heights (1.8, 1.5, 0.6), contact epsilons (1e-7, 1e-5f).
- *Named scars:* `Mth.equal` on x and z but exact on y; the 32-box inline set; back-off step 0.05; the supporting block's tie-break order (y, z, x); `acos` in the minor-collision angle (`movement.rs` near line 893).
- *Sources:* `shapes.rs` 358, `movement.rs` 187, `boat.rs` 147, `basis/sweep.rs` 137, `interact.rs` 26.
- *Rebuilt:* kernel 130, shapes 200, player scars 150, dismount 60.
- *Witnesses:* `shapes.tsv` 1,024 and `epsilons.tsv` 4 (the generator's own face: request → result, boundary); `replay.rs`.

**11. Contact, body–body (818 → 250).**
- *Members:* `push_pair`, `pushable_by` (team rules, spectators, the boat gate), `do_push`, cramming, `SpatialCrowdIndex` (347 lines: a hash grid kept for Mojang's section iteration order), the sim's living loop.
- *Parameter columns:* push factor 0.05, the 0.01 threshold, cramming max 24, damage 6.
- *Rebuilt:* pair push 80, index on the world's entity store with the order scar 120, cramming 15, loop 35.
- *Witnesses:* E_BOAT_CROWD (boundary); `crowd_witness` in-file tests; `perf_core.rs`.

**12. Kicks: launches and impulses (736 → 270).**
- *Members:* `basis::body::launch` (shoot, spread, facing); bow power curve; crossbow charge, power and multishot fan (JOML quaternion); fishing cast and pull; snowball; pearl use; item drop velocities; trident strength, riptide and spin; living knockback, extra knockback, punch, wind burst, mace knockback; explosion entity effect; firework boost; the resource-event knockback.
- *Parameter columns:* power, inaccuracy, roll offset, spawn offset, a (velocity keep: 0, 0.5, 1), b (direction scale), y cap 0.4, RNG draw order.
- *Named scars:* the `+0.0` add at zero spread; the projectile's facing taken before the thrower's motion is added; the multishot quaternion in f32; the knockback degenerate-direction epsilon.
- *Sources:* `projectiles.rs` 211, `items.rs` 133, `combat.rs` 116, `explosion.rs` 116, `pearl.rs` 74, `elytra.rs` 52, `basis/body.rs` 22, `movement.rs` 12.
- *Rebuilt:* kernel 30, rows' scars 240.
- *Witnesses:* BOW_*, XBOW_*, FISH_*, TRIDENT_*, PEARL_LAUNCH, WC_SHOOT, KB, EXTRA_* (pinned to helpers, mostly called only by tests); THROW_TRAJ (boundary).

**13. Player tick core (664 → 330).**
- *Members:* `tick_movement`'s phase order (base tick: fluids, fire; `aiStep`: crouch, closest space, sprint, flight toggle, cooldowns, jump; travel; block effects; pose; exhaustion; food); the sprint machine; swimming transitions; crouch and pose selection; refusals; the `player_state!` declaration.
- *Named scars:* creative flight's double tap (7 ticks); the sprint machine reads the previous input only inside the sprint window; `shouldStopSwimSprinting` reads food through blindness and mayfly.
- *Sources:* `movement.rs` 327, `state.rs` 219, `player.rs` 118.
- *Rebuilt:* orchestration 120, flag machines 90, state declaration 110, refusals 10. Today `update_sprinting` copies 22 fields into `SprintTickInput` each tick to call `step_ai_sprint`.
- *Witnesses:* `sprint.tsv` 4,096 (an exhaustive 12-bit table, pinned to the internal `update_sprinting`); ABILITY_SWITCH, CRAWL_POSE, EFFECT_SPEED (helpers); `replay.rs` (boundary).

**14. Elytra (626 → 220 + data), kernel.**
- *Why a kernel:* `updateFallFlyingMovement` is trigonometric in pitch and look (`Math.cos`, `sqrt`, lift ∝ cos²), not affine. It needs HotSpot's `dcos` bit for bit.
- *Sources:* `elytra.rs` 626, of which **254 lines are the `DCOS_DELTA` table**: data, counted as code today.
- *Rebuilt:* glide update 40, fdlibm cos 130, glide gates 50; the table becomes a data file.
- *Witnesses:* ELYTRA_TRAJ 116 (boundary), ELYTRA_DCOS 32 (pinned to `jvm_dcos`), ELYTRA_WALL, ELYTRA_FALL_ACC.

**15. Portals (481 → 240), kernel.**
- *Why a kernel:* PortalForcer is a search: the closest portal over POIs in a radius, then a spiral search for a frame site, then a fallback base. The transit is an affine map (relative position, rotation, scale, border clamp) and could be a row, but it lives with the search.
- *Rebuilt:* transit 60, process clock 25, search and create 150.
- *Witnesses:* PORTAL_TRANSIT and PORTAL_BORDER (boundary); `portal`, `poi`, `spiral` in `mounts_breath_void.tsv` (helpers). 255 of its 481 lines are reached only by tests.

**16. Shape rows of state (446 → 110).**
- *Members:* the bamboo and dripstone offset from the position seed; the shulker lid (progress clock 0.1 a tick, contracted box, can-open, push vector, place and break); cauldron fill shapes and the fire-inside rule; big dripleaf tilt shapes.
- *Rebuilt:* offsets 25, shulker 55, cauldron and dripleaf rows 30.
- *Witnesses:* OFFSET_*, SHULKER_*, CAULDRON_* (pinned to helpers, called only by tests).

**17. Numeric scars (404 → 250).**
- *Members:* the 65,536-entry sine table (`include_bytes`), `floor`, `ceil`, `equal`, `wrap_degrees`, `JavaRandom`, `Mth.atan2` with its table, `fast_inv_sqrt`, `Vec3`, `Axis`, `Direction` (two `impl` blocks, `neighbor.rs` and `interact.rs`), `face_comparisons`.
- *Sources:* `math.rs` 152, `neighbor.rs` 102, `interact.rs` 81, `shapes.rs` 69.

**18. Clocks (391 → 150).**
- *Members:* `basis::clock` (COOLDOWN, FOOD_SATURATED, FOOD_FED, AIR); the effect book (duration, hidden stack, interval per kind and amplifier, merge on update); air supply; fire ticks; item use countdown and consume ticks; attack strength; the wither interval.
- *Parameter columns:* step, at, reset, numeric type (i32 or f32), firing action.
- *Named scars:* fire damage on `%20` checked before the decrement; negative fire grace untouched; the effect interval `base >> amplifier` (poison 25, wither 40, regeneration 50; at least 1 tick).
- *Sources:* `effects.rs` 205, `breath.rs` 55, `player.rs` 51, `items.rs` 40, `combat.rs` 23, `basis/clock.rs` 17.
- Other clocks counted in their own groups: pearl, portal and boat cooldowns; shulker and moving-piston progress; the item age and merge cadence; mount jump charge; bow draw; torch burnout.
- *Witnesses:* `air` 120, WITHER_EFFECT_TICK 63, EFFECT_*, ITEM_COUNTDOWN (helpers).

**19. Rails (380 → 220), kernel.**
- *Members:* `Minecart::tick`, `move_along_track` (rail exits per shape, slope acceleration 0.0078125, powered boost 0.06, brake, max speed 0.4 on land and 0.2 in water, snap to the rail, `get_pos`, `get_pos_offs`), `come_off_track`.
- *Witnesses:* MC_TICK 151 and MC_GET_POS (boundary).

**20. Attribute fold (379 → 130).**
- *Members:* speed modifier order by id hash; effect → modifier (speed, slowness, jump boost); soul speed with its location check; swift sneak; depth strider; feather falling and protection EPF; mining destroy speed (tool rules, haste, conduit, fatigue, water, air).
- *Sources:* `attributes.rs` 230, `mining.rs` 149.
- *Witnesses:* `attributes.tsv` 12, ENCH_*, EFFECT_SPEED_ORD (helpers).

**21. Ray (328 → 220).**
- *Members:* `basis::ray` (walk, `cells_along`, `clip_face`, `clip_shape`, `clip`); the pearl's raytrace with its inside-box margin; explosion exposure (`seen_percent`); interaction range and the looking directions; the fall-reset clip; break range.
- *Sources:* `basis/ray.rs` 166, `interact.rs` 62, `pearl.rs` 45, `explosion.rs` 35, `mining.rs` 16, `movement.rs` 4.
- *Witnesses:* `rays.tsv` 512 and `intersections.tsv` 512 (the generator's own face, boundary); BOAT_CLIP.

**22. Wire graph (254 → 180), kernel.**
- *Why a kernel:* power is a graph relaxation whose update order is observable (it drives neighbour updates and timing). Today it is the default wire evaluator: `update_redstone_wire`, connections, `get_signal`, `get_direct_signal`, best neighbour signal.
- *Sources:* `sim.rs` 193, `neighbor.rs` 32, `interact.rs` 29.
- *Witnesses:* A_CASCADE (boundary).

**23. Game mode, abilities, death (228 → 80).** Game types (rows), abilities per mode, the game-mode switch packet, below-world, respawn, bed rule gates, respawn anchor. *Source:* `player.rs` 228. *Witnesses:* ABILITY_SWITCH, `void` 21, `bed_const`.

**24. Inventory (206 → 90).** Consumable rows (already `const` rows), slot fill (`add`, `add_resource`), merge rules, unbreaking's binomial removal, durability. *Source:* `items.rs` 206. *Witnesses:* ITEM_* (helpers).

### What the grouping finds, beyond the counts

- **197 law functions (~2,890 lines) have no caller outside tests** in any crate: `block_hooks` 391 lines, `portal` 255, `place` 216, `player` 184, `projectiles` 163, `boat` 163, `redstone` 157, `sim` 155, `mining` 145, `items` 145, and the rest. They are the jar-pinned internal API. They include twins of the live path that exist only to be tested (`player::travel_in_air_forward_tick`, `get_friction_influenced_speed` and `apply_collector_effects` beside `movement.rs`'s live versions; `elytra::update_fall_flying`; `player::hurt_server_mitigated` beside `movement::apply_server_damage`). A rebuild drives those witness rows through the boundary instead, and the twins go.
- **Sound and game events are emitted** (`BucketSound`, `BucketGameEvent`, SOUND and GAME_EVENT in the bucket traces). They cannot change T. Question the requirement: the rebuilt cell log keeps SET, SCHED and NEIGHBOR, and the trace witnesses drop the sound columns.
- **One entity loop, not four:** `tick_falling_blocks` (164), `tick_boats`, `tick_pearls` and `tick_living_and_crowd_push` (89) are one loop over body rows with per-class scars.

## 3. Witnesses: boundary rows and pinned internals

- **Boundary rows (state in → state out of a tick, an edit, or a generator's own face): 3,170 of the 9,117 jar rows.** `shapes` 1,024, `rays` 512, `intersections` 512 (the sweep and ray faces); `world_sim` 63 (multi-tick world); `falling` 20; ELYTRA_TRAJ 116, ARROW_TRAJ 40, THROW_TRAJ 40, MC_TICK 151 (trajectories); PLATE, BUTTON and TRIPWIRE traces and BUCKET_CASE (edit traces); BUBBLE, FLY_TRAVEL, FREEZE_STEP, COLLECTOR_*, EFFECT_* motion rows; PORTAL_TRANSIT; BOAT_PADDLE and BOAT_STATUS; `air` and `void` sequences. Add `replay.rs`'s 29 recordings (5 ignored): per-tick `PlayerState` in → out, the strongest boundary witness. They survive any rebuild unchanged.
- **Rows pinned to an internal helper (one Mojang method's inputs → output): 5,947 rows,** including `sprint.tsv`'s 4,096 (pinned to `movement::update_sprinting`) and every `*_CURVE`, `*_MATH`, `*_SHAPE`, `OFFSET_*`, `SHULKER_*`, `HOOK_*`, `jump_scale`, `ELYTRA_DCOS` row. The values stay valid as data. What binds them to today's types is the harness: `tests/jar.rs` (4,965 lines, 18 tests, calling 66 distinct `law::module::fn` paths) and 101 in-file `#[test]`s (~7.7K test lines inside `src/`). A rebuild keeps every row and rewrites these harnesses to feed each row through the generator's row or the nearest boundary. That cost is test code, outside the floor, but it is the largest single piece of rebuild work: the `jar_world_hooks` harness alone is 939 lines.
- **Planner-side boundary:** `plan`'s `real_time` (364 CMD/END/REFUSED lines byte-identical across passes) and the flights.

## 4. The floor, bottom-up

| Bucket | Groups | Rebuilt |
|---|---|---:|
| Generators (kernels of the basis) | `body` 110, kick 30, `sweep` 130 + shapes 200, `ray` 150, `clock` 25, `cell` engine 400, `contact` 190 (cell face 70, pair face 120), `ledger` 70 | 1,305 |
| Rust on the generators (scars, guards, rule logic; constants are rows) | core 330, body scars 490, kicks 240, sweep scars 210, shape rows 110, ray uses 70, contact scars 330, clocks 125, ledger 230, attribute fold 130, cell rules 950, inventory 90, game mode 80 | 3,385 |
| Named kernels | pistons 500, fluids 510, portals 240, elytra 220, rails 220, wire graph 180 | 1,870 |
| World and state plumbing | planner interface 730, world interface 320, numeric scars 250, glue 60 | 1,360 |
| **Floor** | | **7,920** |

- **The floor is ~7.9K code lines** (an honest band 5.5K–10K), against 20,185 today: **0.41 decades**. Without the planner interface, which is not dynamics, it is ~7.2K (0.45 decades).
- It is **0.72 decades above** the ~1,500 figure in [law-basis-3](../law-basis-3/report.md). That figure is reachable only by moving rule logic into tables as scripts or opcodes, which this table refuses. The mass is real logic: 991 of 1,106 functions are witnessed, and the cell rules alone are ~950 lines after every constant becomes a row.
- **Where the reduction comes from:** Java-shaped scaffolding (per-family structs, event enums, trait views, adapters, `Default` impls) ~4.5K; the 2.9K of witness-only twins and helpers folded into boundaries; one entity loop and one hurt pipeline; the 254-line table moved to data.
- **Where it does not:** the kernels (1.9K) are already near Mojang's own size; the cell rules (950) are guards that do not become data; the sweep, shapes and ray (~700) are tuned and stay.

## 5. Hot player core and cold world layer

- **Hot** (a plain movement tick on dry stone, survival, no effects; ~1.5K lines rebuilt, ~3.7K today): the tick core (13), living travel and the body kernel (2), sweep and shapes with the player's scars (10), the cooldown clocks (18), the attribute fold's speed (20, refreshed per tick today), food and exhaustion (7), the `World` read path (8), numeric scars (17), and the early-outs of `update_fluid` and the block effects (9 and 6).
- **Lukewarm** (entered only when a flag or the world says so): immersion and current (9), body–cell contact (6), elytra (14), fall damage (7).
- **Cold** (the world layer): `cell` engine and rules (4, 1, 16), the wire graph, fluid spread, pistons, portals and rails (22, 9, 5, 15, 19), free bodies and kicks (2, 12), body–body contact (11), the planner interface (3).

**`PlayerState`: 296 B** (align 8; the `Profile` behind its `Arc` is 152 B more). Field offsets were measured with `offset_of!`.

| Class | Fields (bytes) | Bytes |
|---|---|---:|
| Hot: read and changed by a plain tick | `position` 24, `velocity` 24, `movement_this_tick` 32, `input` 16, `main_supporting_block` 16, `t` 8, `fall_distance` 8, `profile` pointer 8, `yaw` 4, `pitch` 4, `no_jump_delay` 4, `sprint_trigger_time` 4, `speed` 4, `exhaustion` 4, ten flags 1 each (`on_ground`, `horizontal_collision`, `vertical_collision`, `vertical_collision_below`, `minor_horizontal_collision`, `sprinting`, `sprint_speed_modifier`, `shift_key_down`, `crouching`, `pose`), `on_ground_no_blocks` 1 | 171 |
| Lukewarm: touched every tick but constant or re-derived on dry ground | `stuck_speed_multiplier` 24, `effect_book` 24 (only `is_empty` read), `water_height` 8, `lava_height` 8, `health` 4, `food_level` 4, `saturation` 4, `food_tick_timer` 4, `remaining_fire_ticks` 4, `jump_trigger_time` 4, `sprint_window` 4, nine fluid and climb flags 1 each (`in_water`, `in_lava`, `eye_in_water`, `was_eye_in_water`, `underwater`, `swimming`, `on_climbable`, `in_powder_snow`, `was_in_powder_snow`), `flying`, `fall_flying`, `effects` 1 each | 104 |
| Cold: untouched by a plain tick | `hurt_cooldown` 8, `ticks_frozen` 4, `absorption` 4, `max_health` 4 | 20 |
| Padding | | 1 |

**What the split shows:**
- `movement_this_tick` (32 B, the largest field) is this tick's output, which the planner reads afterwards (`plan/src/arith.rs`). It is not state.
- `yaw` and `pitch` copy `input.yaw` and `input.pitch`.
- 9 of the fluid flags and both heights are zeroed and recomputed every tick. Only `was_eye_in_water` and `was_in_powder_snow` carry memory.
- `effect_book` (a `Vec`) and `stuck_speed_multiplier` are cold contents behind hot reads.
- A plain tick's true hot core fits about 96 B:
  - position and velocity: 48
  - flags as bits: 2
  - `yaw`: 4
  - `fall_distance`: 8
  - supporting block: 12
  - three small counters: 3
  - `exhaustion`: 4
  - `t`: 8
  - input bits: 2

  That is 1.5 cache lines against 4.6 today.

## Appendix: line ranges per group (code lines by `law-lines.pl` rules)

| Group | File: code lines |
|---|---|
| Cell rules | world_hooks.rs 1–1535: 1,313; sim.rs 929–1037, 1082–1305, 1333–1364, 2404–2514, 2875–3104, 3268–3339: 755; place.rs all: 434; mining.rs 581–759, 785–end: 360; redstone.rs all: 345; interact.rs 211–310, 346–end: 164; falling.rs 64–end: 98; boat.rs 177–283: 96; block_hooks.rs 956–1058: 83; neighbor.rs 540–end: 18 |
| Body | boat.rs 1–106, 284–618: 385; sim.rs 371–431, 2515–2774: 300; movement.rs 393–690: 265; mounts.rs: 171; basis/body.rs 1–151: 95; pearl.rs 105–177, 233–end: 86; projectiles.rs 224–328: 80; items.rs 352–453: 79; player.rs 825–1174: 69; explosion.rs 225–end: 23; falling.rs 1–33: 17 |
| Planner interface | pattern.rs 362; footprint.rs 309; codec.rs 272; state.rs 246–end: 153; codec_proof.rs 145; codec_planes.rs 68; codec_state.rs 68; solve_meter.rs 21; state_reads.rs 20 |
| Calendar | sim.rs 1–53, 330–362, 543–928, 1038–1063, 1306–1332, 3166–3225, 3244–3267: 536; ticks.rs 326; neighbor.rs 136–497: 302; basis/cell.rs 255–end: 152 |
| Pistons | piston.rs 714; sim.rs 363–370, 1487–1504, 1925–2403, 3226–3243, 3340–end: 507 |
| Contact body–cell | block_hooks.rs 496–876, 1059–end: 354; movement.rs 1048–1382: 315; player.rs 154–259, 382–401, 610–674: 158; world_hooks.rs 1536–end: 73; world.rs 146–215: 60; sim.rs 3105–3165: 47 |
| Ledger | combat.rs 43–84, 210–590: 254; movement.rs 890–942, 1383–1491, 1505–end: 250; player.rs 260–323, 413–609: 238; boat.rs 107–176: 62; projectiles.rs 329–378: 34; explosion.rs 92–123: 26; falling.rs 34–63: 21; items.rs 163–196: 21; mining.rs 760–784: 17 |
| World interface | world.rs 1–37, 216–end: 436; sim.rs 54–329: 231; basis/cell.rs 1–254: 214 |
| Fluids | fluid.rs 354; sim.rs 1571–1924: 329; movement.rs 943–1042: 93; world.rs 38–145: 90 |
| Sweep | shapes.rs 97–end: 358; movement.rs 357–392, 722–889: 187; boat.rs 619–end: 147; basis/sweep.rs 137; interact.rs 111–141: 26 |
| Contact body–body | entity_contact.rs 625; sim.rs 432–542, 2775–2874: 193 |
| Kicks | projectiles.rs 1–223, 379–end: 211; items.rs 197–351, 482–515: 133; combat.rs 85–209, 591–end: 116; explosion.rs 1–54, 124–224: 116; pearl.rs 1–104: 74; elytra.rs 1–81: 52; basis/body.rs 152–end: 22; movement.rs 1492–1504: 12 |
| Player core | movement.rs 1–356, 691–721: 327; state.rs 1–245: 219; player.rs 1–153: 118 |
| Elytra | elytra.rs 82–end: 626 |
| Portals | portal.rs 481 |
| Shape rows | block_hooks.rs 1–495, 877–955: 446 |
| Numeric | math.rs 152; neighbor.rs 1–135: 102; interact.rs 16–110: 81; shapes.rs 1–96: 69 |
| Clocks | effects.rs 205; breath.rs 55; player.rs 324–381, 402–412: 51; items.rs 103–162: 40; combat.rs 1–42: 23; basis/clock.rs 17 |
| Rails | minecart.rs 380 |
| Attribute fold | attributes.rs 230; mining.rs 1–561: 149 |
| Ray | basis/ray.rs 166; interact.rs 1–15, 142–210: 62; pearl.rs 178–232: 45; explosion.rs 55–91: 35; mining.rs 562–580: 16; movement.rs 1043–1047: 4 |
| Wire graph | sim.rs 1064–1081, 1365–1486, 1505–1570: 193; neighbor.rs 498–539: 32; interact.rs 311–345: 29 |
| Game mode, death | player.rs 675–824, 1175–end: 228 |
| Inventory | items.rs 1–102, 454–481, 516–end: 206 |
| Glue | lib.rs 86; bin/bench.rs 30; basis/mod.rs 5 |
| **Sum** | **20,185** |
