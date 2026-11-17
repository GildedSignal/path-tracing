# Port Wave 2: Digest of 17 November Results & Next Wave Lenses

## Part 1: Digest of the 11 results of 17 November (`local-20261117-13*` through `16*`)

### 1. Per-result digest

#### 1. `port-mining-destroy-speed` (`local-20261117-1340-1-0-port-mining-destroy-speed`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `BlockBehaviour.getDestroyProgress` (`@0-49`) = `0.0f` if `hardness == -1.0f` (`fcmpl` `@9-19`; NaN falls through), else `(Player.getDestroySpeed(state) / hardness) / (hasCorrectToolForDrops ? 30 : 100)` (`@20-49`, two left-assoc `f32` divisions). `Player.getDestroySpeed` (`@0-175`): `Tool` component base speed (`1.0f` if absent), `+ (f32)MINING_EFFICIENCY` only if `base > 1.0f` (`@12-28`, Efficiency V = `26.0f` via `levels_squared + 1.0`), `* (1.0f + (max(HASTE, CONDUIT_POWER) + 1) * 0.2f)` (`@29-51`), `* {0: 0.3f, 1: 0.09f, 2: 0.0027f, default(>=3): 8.1E-4f}` for `MINING_FATIGUE` (`@52-125`; `MobEffectInstance` clamps amp to `[0, 255]`, no `2.43E-4` branch in 26.2), `* (f32)BLOCK_BREAK_SPEED` (`@126-136`, base `1.0`), `* (f32)SUBMERGED_MINING_SPEED` (`@137-160`, base `0.2`, Aqua Affinity `add_multiplied_total 4.0` -> `1.0f`) when `isEyeInFluid(WATER)`, `/ 5.0f` (`@161-173`) when `!onGround()`. `Tool.getMiningSpeed` and `Tool.isCorrectForDrops` scan `rules` independently (first matching rule with present field wins; e.g. iron pickaxe on obsidian gets `speed=6.0`, `drops=false` -> divisor `100`).
- **Engine state:** **Partly** — `engine\law\src\mining.rs:11-27` (`MiningSample::progress`) implements the exact `getDestroyProgress` `(speed / hardness) / (30.0 | 100.0)` and `hardness == -1.0` formula, but lacks `Player.getDestroySpeed` (the tool-rule evaluation, `> 1.0f` efficiency gate, Haste/Conduit/Fatigue multipliers, `BLOCK_BREAK_SPEED`, `SUBMERGED_MINING_SPEED`, and `!onGround` `/ 5.0f`).
- **Unblocks:** `docs\plan.md:49` Step 5 (`P2c`, mining T-cost in `mining.rs`).
- **Port delta:**
  - *Files:* `engine\law\src\mining.rs` (plus `attributes.rs` if mining attributes live on `PlayerState`).
  - *What changes:* Add `MiningModifiers` / `player_destroy_speed(tool_base_speed, mining_efficiency, haste_or_conduit_amp, fatigue_amp, block_break_speed, eye_in_water, submerged_mining_speed, on_ground) -> f32` implementing the exact `@0-175` `f32` operation order (`> 1.0f` gate -> `+ mining_efficiency as f32` -> `* (1.0f + (amp + 1) as f32 * 0.2f)` -> fatigue tableswitch -> `* block_break_speed as f32` -> `if eye_in_water { * submerged_mining_speed as f32 }` -> `if !on_ground { / 5.0f32 }`), feeding `MiningSample::destroy_speed`, plus `ToolRule` first-match speed/drops evaluation.
  - *Bit-exact test:* Assert raw `f32` bits and `k_break = ceil(1.0 / p)` across all 13 author + 10 checker cases (bare hand dirt `15`, obsidian+iron pick `speed=6.0, divisor=100, k=834`, Efficiency V `0x41d00000`, `hoe+eff5` ignored gate `k=150`, Haste I + Conduit III `max=2 -> *1.6`, Fatigue IV/V `8.1e-4 -> k=9260`, non-default `BLOCK_BREAK_SPEED = 3.5 -> k=24`, Aqua Affinity water `0x3f800000 -> k=8`, bedrock `0x00000000`, kitchen-sink `k=17`).

#### 2. `port-block-support-pop-family` (`local-20261117-1341-1-0-port-block-support-pop-family`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** Across all 331 real classes in `net.minecraft.world.level.block{,.piston}`, 108 override the 8-arg `updateShape`: **52 synchronous-pop / 20 delayed-block-tick / 36 pass-through (NONE) / 0 BOTH**. Live execution across 1,196 registered blocks x 6 facings finds 53 mismatches against static needles: 52 are **dead pop branches** whose `canSurvive` is identically `true` (`FlowerPotBlock` x39, `WallHangingSignBlock` x12, `GrindstoneBlock` x1), and 1 is `ObserverBlock` (helper-hidden `startSignal` schedules delay `2`). Pinned laws: `DoorBlock.updateShape` (`@12-147`) synchronously pops to `AIR` when the partner half is not `DoorBlock` or has same `HALF` (`@52-96`), synchronously mirrors an intact partner's state (`FACING/HINGE/OPEN/POWERED`, `@76-89`), and synchronously pops `LOWER` on `facing==DOWN && !canSurvive(below.isFaceSturdy(UP))` (`@97-129`); `ScaffoldingBlock.updateShape` (`@16-54`) never pops and unconditionally `scheduleTick(pos, this, 1)` on `!isClientSide`, deciding 1 tick later in `tick` (`@0-117`) via `getDistance` (`@0-161`: `DISTANCE==7` fresh -> `destroyBlock(pos, true)`, already-7 -> `FallingBlockEntity.fall`, else `setBlock(..., 3)`); `BaseRailBlock` pops synchronously in `neighborChanged`, not `updateShape`; `TrapDoorBlock` has no support-loss pop law.
- **Engine state:** **Lacks it** — `engine\law\src\world.rs:182-255` stores static block geometry/flags only; there is no `update_shape` support-loss pop table, `ScheduledTick` producer, or `ScaffoldingBlock::getDistance` evaluator.
- **Unblocks:** `docs\plan.md:49` Step 5 (edit/undo obligation ledger, `P2a`, `port-update-order`).
- **Port delta:**
  - *Files:* `engine\law\src\world.rs` (block traits), `engine\law\src\interact.rs` (Step 5 `update_shape` dispatch & scheduled ticks), `engine\world\tools\ModelTable.java`.
  - *What changes:* Classify blocks by `SupportRule { None, SyncPop(CanSurviveKind), ScheduledTick(delay), RailNeighborPop }` (mapping FlowerPot/WallHangingSign/Grindstone/TrapDoor to `None`), implement `door_update_shape` (partner pop + state mirror + lower `is_face_sturdy(UP)` pop), `scaffolding_update_shape` (unconditional server `schedule_tick(pos, Scaffolding, 1)`), and `scaffolding_tick` (`get_distance` -> `destroy` / `fall` / `set_block(..., 3)`).
  - *Bit-exact test:* Verify door lower-floor pop, door upper partner-gone pop, door partner `OPEN` mirror, scaffolding unconditional `delay=1` schedule on any neighbor change (no-op on client side), scaffolding `getDistance = {0, 4, 7}` + fresh-7 `destroy` vs already-7 `fall`, and dead-pop non-activation on wall-mounted grindstone, wall hanging sign, and flower pot.

#### 3. `port-boat-land-and-collision` (`local-20261117-1342-1-0-port-boat-land-and-collision`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `AbstractBoat` non-`IN_WATER` law: `getGroundFriction()` (`@5-340`) averages `block.getFriction()` in `f32` over non-empty collision shapes intersecting `[minY - 0.001, minY]` across `[floor(minX)-1 .. ceil(maxX)+1)` (skipping corner columns, edge-column first/last y, and `LilyPadBlock`; `n == 0 -> NaN -> IN_AIR`). `floatBoat()` (`@0-359`): `d1 = -0.04` (`getDefaultGravity`), `IN_AIR -> f = 0.9f`, `ON_LAND -> f = landFriction` (with dead `landFriction /= 2` overwritten next tick by `getStatus`), `UNDER_WATER -> f = 0.45f, d3 = (double)0.01f`, `UNDER_FLOWING_WATER -> f = 0.9f, d1 = -7.0E-4d`, tail `vx *= (double)f; vz *= (double)f; vy += d1; deltaRotation *= f`, then `if d3 > 0 { vy = (vy + d3 * 0.04 / 0.65) * 0.75 }`. `tick()` (`@0-75`) increments `outOfControlTicks += 1.0f` in `UNDER_WATER | UNDER_FLOWING_WATER` (else `= 0.0f`) and ejects passengers server-side when `outOfControlTicks >= 60.0f` (`fcmpg/ifge`, non-strict). `AbstractBoat.maxUpStep() = 0.0f` (0.5 slab is a wall), and `Entity.restituteMovementAfterCollisions` with `b = 0.0` writes raw `-0.0` (`0x8000000000000000`) on a blocked `+axis` wall (`+0.0` on `-axis` wall) and `+0.0` on falling landing `vy`.
- **Engine state:** **Partly** — `engine\law\src\boat.rs:1-48` models only established `IN_WATER` occupied-boat motion without collision; it lacks `getGroundFriction`, `ON_LAND`/`IN_AIR`/`UNDER_WATER`/`UNDER_FLOWING_WATER`, `maxUpStep = 0.0f` collision/restitution (`-0.0`/`+0.0`), and `outOfControlTicks >= 60.0f` ejection.
- **Unblocks:** `docs\plan.md:52` Step 8 (carried movement, ice-boat tracks, batch `P4a`).
- **Port delta:**
  - *Files:* `engine\law\src\boat.rs`.
  - *What changes:* Add `BoatStatus { InWater, UnderWater, UnderFlowingWater, OnLand, InAir }`, `land_friction: f32`, `out_of_control_ticks: f32`, `get_status` / `get_ground_friction` (with ring-skip & `LilyPadBlock` skip, `n=0 -> NaN -> InAir`), per-status `float_boat` drag/gravity/buoyancy, `outOfControlTicks >= 60.0f` server ejection, and `Entity::move` collision with `max_up_step = 0.0f` + `-v_axis * 0.0` sign-preserving restitution.
  - *Bit-exact test:* Assert raw `u64` velocity/position bits from `evidence.txt` and `check-extra.out` for `ON_LAND` stone (`0x3fc70a3d59999999`), blue ice (`0x3fd2fd2206666666`), packed ice 1-ulp grid drift (`0x3f7ae147`/`0x3f7ae148`), `IN_AIR` (`0x3fd147ae0ccccccd`, `0x3f847ae147ae147c`), `UNDER_WATER` (`0x3fc147ae0ccccccd`, `0xbf9e3f54994dfb47`), `UNDER_FLOWING_WATER` (`0x3fa93dd97f62b6af`), `+x`/`-x` wall `-0.0`/`+0.0` bits, bottom-slab stop at `101.3125`, lily-pad `NaN -> IN_AIR`, and tick-60 ejection.

#### 4. `port-boat-mount-dismount` (`local-20261117-1519-1-0-port-boat-mount-dismount`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `AbstractBoat.interact` (`@0-63`) mounts a non-sneaking player (`!isSecondaryUseActive()`) iff `outOfControlTicks < 60.0f` (`fcmpg; ifge`), `passengers.size() < 2`, and `!isEyeInFluid(WATER)`. `Entity.startRiding` (`@0-173`) forces `setPose(STANDING)` only (no `setPos`/`setYRot`/`setSprinting(false)`), and `Entity.addPassenger` (`@40-94`) inserts a server `Player` at seat `0` if seat `0` is non-Player. `AbstractBoat.positionRider` (`@0-119`) + `clampRotation` (`@0-64`) applies to **players too** (`CAN_TURN_IN_BOATS = ["minecraft:breeze"]` only in 26.2): `rider.yRot += boat.deltaRotation; rider.yHeadRot += deltaRotation; yBodyRot = hull.yRot; f = wrapDegrees(rider.yRot - hull.yRot); f1 = clamp(f, -105.0f, 105.0f); yRotO += (f1 - f); yRot += (f1 - f); yHeadRot = yRot`. Dismount (`Player.rideTick` -> `stopRiding` -> `AbstractBoat.getDismountLocationForPassenger` `@0-271`) computes one rider-yaw-fixed escape vector `(-sin/max, 0, +cos/max) * (1.375 * sqrt(2) + 0.6 + 1e-5) / 2` (`= 1.272277` for player+boat); if `!isWaterAt(below)` probes floor heights at `blockpos` then `below` (`isBlockFloorValid` strict `< 1.0`, empty-cell below floor `< 1.0 -> -inf`) across `[STANDING, CROUCHING, SWIMMING]` (with `setPose(pose)` side effect), falling back on failure or water to boat center `(x, bb.maxY, z)` (hard snap `teleportTo`, no velocity carry).
- **Engine state:** **Lacks it** — `engine\law\src\movement.rs:124-129` refuses `riding`/`boat`; `engine\law\src\boat.rs` has no mount, `clampRotation`, or dismount logic.
- **Unblocks:** `docs\plan.md:52` Step 8 (carried movement, batch `P4a`).
- **Port delta:**
  - *Files:* `engine\law\src\boat.rs`, plus later `movement.rs` / `state.rs` wiring when the `boat` refusal lifts.
  - *What changes:* Add `boat_can_mount(out_of_control_ticks, passengers_len, eye_in_water, sneaking)`, `boat_clamp_rotation(hull_yaw, delta_rot, rider_y_rot, rider_y_rot_o)`, and `boat_dismount_location(world, boat_pos, boat_bb, rider_yaw, rider_width) -> (Vec3, Option<Pose>)` implementing the exact 1-vector escape, `!is_water_at(below)` gate, 2-floor candidate height check (`< 1.0`), `[STANDING, CROUCHING, SWIMMING]` pose selection, and `(x, bb.maxY, z)` fallback.
  - *Bit-exact test:* Assert `outOfControlTicks = 59.9999f` mount vs `60.0f` refuse, `[player, cow]` seat-0 priority, `clampRotation` sweep (`105->105`, `105.1->105`, `+120->+105`, `-120->-105`, `+180->255`, `480->465`, hull `200` + rider `0 -> -55`), and dismount exits: open ground `(0, 64, 1.272277) STANDING`, walled fallback `(0, 64.5625, 0)`, `1.5`-gap ceiling `SWIMMING`, water-below `(0, 64.25, 0)`, and bottom-slab own-cell `y=64.5`.

#### 5. `port-elytra-firework-and-impact` (`local-20261117-1520-1-0-port-elytra-firework-and-impact`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `FireworkRocketItem.use` (`@0-4,49-57`) gates on `player.isFallFlying()` and constructs an attached `FireworkRocketEntity` (`<init>` `@47-141`: `lifetime = 10 * (1 + flightDuration) + nextInt(6) + nextInt(7)`). `FireworkRocketEntity.tick` boosts `@40-174` before `life++` `@420-427` and explodes iff `life > lifetime` `@510-540` (boost ticks = `lifetime + 1`, e.g. `fd=1 -> 21..=32`). Exact per-tick velocity rewrite (`@75-174`, `dmul 0.1; dmul 1.5; dsub; dmul 0.5; dadd` `@90-170`): `v' = v + look * 0.1 + (look * 1.5 - v) * 0.5` (`look = Entity.getLookAngle -> calculateViewVector` via `Mth.sin/cos(D)F`). Because `EntityTickList` (`Int2ObjectLinkedOpenHashMap`) iterates insertion order, tick `N`'s rocket boost writes **after** the rider's same-tick `LivingEntity.travelFallFlying` (`@17-73`) and takes effect on tick `N + 1`. Wall-impact damage (`LivingEntity.handleFallFlyingCollisions(d3, d5)` `@0-58`): if `horizontalCollision`, `d5 = d1 - d3` (pre-glide minus post-move horizontal speed), `f7 = (float)(d5 * 10.0 - 3.0)` (`@12-23`), calls `hurt(flyIntoWall, f7)` iff `f7 > 0.0f` (`fcmpl; ifle` `@25-29`; `d5 = 0.3 -> 0.0` silent, first firing `0x3fd3333333333334` = `4.44e-16` damage). (Checker note: `f7 = NaN` also passes `fcmpl; ifle`; `tickConnection` runs before `tickChildren`.)
- **Engine state:** **Lacks it** — `engine\law\src\movement.rs:130-132` refuses `state.fall_flying`; no elytra glide, firework boost, or `flyIntoWall` exists in `engine\law\src\`.
- **Unblocks:** `docs\plan.md:52` Step 8 (`R-brach`, `P4b`, elytra courses).
- **Port delta:**
  - *Files:* `engine\law\src\elytra.rs` (new module, exported in `engine\law\src\lib.rs`), later wired from `movement.rs:130-132`.
  - *What changes:* Implement `firework_boost(vel, yaw, pitch)` using exact bytecode association `v + look * 0.1 + (look * 1.5 - v) * 0.5` (`look` from `calculate_view_vector` via `math::sin/cos`), `firework_lifetime_bounds(fd) -> (10*(1+fd), 10*(1+fd)+11)` (`+1` boost ticks), post-rider rocket tick ordering (`tick N` boost consumed at `tick N+1`), and `fly_into_wall_damage(horizontal_collision, pre_h_speed, post_h_speed) -> Option<f32>` (`let f7 = ((pre - post) * 10.0 - 3.0) as f32; if f7 > 0.0 || f7.is_nan() { Some(f7) } else { None }`).
  - *Bit-exact test:* Assert raw `u64` bits for `(yaw=-30, pitch=15, v0=(0.4,-0.1,0.05))` `look` (`0x3fdee88fe0000000`, `0xbfd08f7ac0000000`, `0x3feac4fc40000000`) and ticks 1–3 (`0x3fe3893d2599999a` / `0x3fe6e77552000000` / `0x3fe8969168333333` etc. — distinct from folded `0.5*v + 0.85*look` at tick 2), `fd=1..=3` boost bounds (`21..=32`, `31..=42`, `41..=52`), and `flyIntoWall` threshold `d5 = 0.3` (`None`) vs `0x3fd3333333333334` (`Some(4.44e-16)`) vs `0.5` (`Some(2.0)`).

#### 6. `port-pearl-raytrace-and-landing` (`local-20261117-1521-1-0-port-pearl-raytrace-and-landing`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `ThrowableProjectile.tick` (`@1-89`) applies gravity (`Entity.applyGravity` `@0-26`, `getDefaultGravity = 0.03d`) **before** drag (`applyInertia` `@0-118`: `isInWater ? (double)0.8f : (double)0.99f` via `v.scale((double)f)`), raytraces via `ProjectileUtil.getHitResultOnMoveVector` (`@0-32`, `ClipContext.Block.COLLIDER`, `ClipContext.Fluid.NONE`, `Level.clipIncludingBorder`, entity sweep `bb.expandTowards(delta).inflate(1.0)` with margin `max(0f, min(0.3f, (tickCount - 2) / 20f))`, entity hit wins), and calls `setPos(hit_or_end)` (`@54-55`). On impact `ThrownEnderpearl.onHit` (`@0-409`) teleports a valid server owner to `this.oldPosition()` (`xo/yo/zo` set by `ServerLevel.tickNonPassenger @1`, i.e. start-of-impact-tick position, **not** the ray hit point) through `ServerPlayer.teleport` -> `Entity.teleportSetPosition` (`@8906-8984`) with **no** coordinate clamp and **no** chunk check, then calls `resetFallDistance()` (`@302`) strictly before `resetCurrentImpulseContext()` (`@307`) and `hurtServer(enderPearl, 5.0f)` (`@310-327`), and `discard()` (`@409`). Pearl `fallDistance` stays `0.0`.
- **Engine state:** **Lacks it** — `engine\law\src\lib.rs` has no projectile/pearl module.
- **Unblocks:** `docs\plan.md:52` Step 8 (`P4c`, `port-wormhole-facts`, pearl stasis/teleports).
- **Port delta:**
  - *Files:* `engine\law\src\pearl.rs` (new module, exported in `engine\law\src\lib.rs`).
  - *What changes:* Implement `PearlState { pos, old_pos, vel, tick_count, fall_distance }`, `pearl_tick(world, pearl)` (`old_pos = pos`, `vel.y -= 0.03`, `vel *= if in_water { 0.8f32 as f64 } else { 0.99f32 as f64 }`, collider-only / fluid-none raycast via `blocks::traverse_blocks` + entity sweep margin `((tick_count - 2) as f32 / 20.0).clamp(0.0, 0.3)`, `pos = hit_or_end`), and `PearlLanding { teleport_pos: pearl.old_pos, reset_fall_before_damage: 5.0f32 }`.
  - *Bit-exact test:* Assert 20-tick air and 12-tick water bit-exact velocity/position traces (wet t4 `vy = 0x3fd2c986197d8450`, falsifying drag-before-gravity and double-`0.8`), `computeMargin` for `t=0..=12`, clip-mode corridor (`COLLIDER+NONE` hits stone at `4.0` passing grass `2.125` & water `3.0`), and fast-drop/horizontal-wall landings (`teleport_pos == start_of_tick != face`, `pearl.fall_distance == 0.0`).

#### 7. `port-minor-collision-acos` (`local-20261117-1604-1-0-port-minor-collision-acos`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** In `minecraft-26.2-client.jar`, `LocalPlayer.isHorizontalCollisionMinor(Vec3)` (`@0-156`) passes the **unclamped** quotient `dot / Math.sqrt(a * b)` into `Math.acos` (`@127-138`, no clamp/min/max instruction) and tests `< 0.13962633907794952` via `dcmpg; ifge 156` (`@143-149`), so `q > 1.0` (e.g. `0x3ff0000000000001`) or `q < -1.0` produces `NaN -> false`, stopping sprint (`shouldStopRunSprinting` `@24-35`). A `[-1, 1]` clamp would return `true` on `q > 1.0` and diverge from Mojang.
- **Engine state:** **Already in and bit-exact** — `engine\law\src\movement.rs:802-819` (`minor_collision`) matches the jar's float radian conversion (`:808`), `math::sin/cos` `(D)F` widened (`:809-810`), `1.0e-5f32 as f64` guards (`:814-816`), `(a * b).sqrt()` + unclamped `.acos()` (`:817`), and strict `< 0.13962633907794952` (`:818`). (Only the `horizontal_collision` gate sits inside `minor_collision` at `:804` instead of at the `Entity.move` call site — identical behavior.)
- **Unblocks:** Closes Anomaly 12 (`docs\HANDOFF.md:46`) and batch `P1a` (`monotone-tick` 14/255,150 escapes confirmed real Mojang behavior; forbids any clamp patch).
- **Port delta:**
  - *Files:* `engine\law\src\movement.rs` (tests only; **no production code change**).
  - *What changes:* Add a `#[cfg(test)]` regression witness in `movement.rs` driving `minor_collision` with the jar witness (`yaw=5.1076984`, `xxa=0.19115508`, `zza=-0.68441665`, `moved` producing `q = 0x3ff0000000000001`), plus `q == 1.0` (`true`), `q < -1.0` (`false`), and `0.9902680688920994` vs `nextUp` threshold flip.
  - *Bit-exact test:* Assert `minor_collision == false` on the `q = 0x3ff0000000000001` witness and `q < -1.0` case, and `true` on `q == 1.0`.

#### 8. `port-boat-trig-index` (`local-20261117-1617-1-0-port-boat-trig-index`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `net.minecraft.util.Mth` declares **only** `sin(D)F` (`@0-15`, index `(long)(v * 10430.378350470453) & 65535`) and `cos(D)F` (`@0-19`, adds `16384.0d` in double) — no `Mth.sin(F)F`/`cos(F)F` exists (`GivensParameters` uses `org.joml.Math.sin(F)F` only). Every boat trig site widens an `f32` product `(±yaw) * 0.017453292f` via `f2d` and calls `Mth.sin/cos(D)F` (`Vec3.yRot` `@2/@8`, `AbstractBoat.getPassengerAttachmentPoint` `@77-85`, `AbstractBoat.controlBoat` `@123-148`). Across `k/32°` (`11,520` yaws), a float-index mutant diverges on `149` yaws (`49` in the seat domain, explaining the `port-boat-rider-attachment` checker's `49/11520` note) and drifts on the 800-tick W+A turn trace (`4596625426450759510` vs `4596625426452742353`).
- **Engine state:** **Already in and bit-exact** — `engine\law\src\math.rs:1-17` (`sin.bin` FNV-1a64 identical to `Mth.SIN`, `((value * SIN_SCALE) as i64 & 65535)`) and `engine\law\src\boat.rs:5-11,44-45` widen `(-yaw * 0.017453292f32) as f64` before `math::sin/cos`, matching the jar on all 448/448 witness lines.
- **Unblocks:** Closes batch `P4a` trig-index verification (`port-boat-rider-attachment` 49/11520 note).
- **Port delta:**
  - *Files:* `engine\law\src\boat.rs` (tests only; **no production code change**).
  - *What changes:* Add a `#[cfg(test)]` regression test checking `boat::seat` and `boat::tick` on key divergent `k/32°` yaws (e.g. `k=45` negative-wrap index `65280`) and the 800-tick W+A turn trace.
  - *Bit-exact test:* Assert 800-tick W+A horizontal speed raw bits equal `4596625426452742353` (`to_bits()`) and `k=45` (`1.40625°`) `math::cos` bits equal `1065348123` (`f32::to_bits()`).

#### 9. `port-regen-dead-guard-and-phase` (`local-20261117-1636-1-0-port-regen-dead-guard-and-phase`)
- **Checker grade:** `reproduced` (`check.md:1`).
- **Settled fact:** `ServerPlayer.tickRegeneration` (`@0-124`, called unconditionally on connected players via `ServerPlayer.doTick @15 -> Player.tick -> LivingEntity.tick (!isRemoved @170-178) -> Player.aiStep @17-18`, with `Entity.tickCount` incremented first in `ServerLevel.tickNonPassenger @8-15`) has **no** dead guard on its `%20` health leg (`@45-54` only checks `getHealth() < getMaxHealth()`), **but** calls `LivingEntity.heal(1.0f)` (`@0-18`), which returns without writing when `getHealth() <= 0.0f` (`fcmpl; ifle @5-8`). Meanwhile, the `%20` saturation leg (`@62-85`, raw `setSaturation(sat + 1.0f)`) and `%10` food leg (`@88-121`, `setFoodLevel(food + 1)`) still run when `health <= 0.0f`. In `FoodData.tick` (`@0-324`), both heal legs call `Player.isHurt()` (`health > 0.0f && health < getMaxHealth()`), matching `movement.rs:1280,1288` bit-for-bit.
- **Engine state:** **Partly (bug in `server_regeneration`)** — `engine\law\src\movement.rs:1257-1259` writes `health = (health + 1.0).min(20.0)` whenever `health < 20.0` without checking `health > 0.0`, resurrecting a dead Peaceful player (`0.0 -> 1.0`) at `tick % 20 == 0`. `movement.rs:1260-1266` (saturation/food) and `movement.rs:1271-1300` (`food_tick`) already match the jar.
- **Unblocks:** Batch `P3b` (`jobs\port-backlog\report.md`) and `port-peaceful-regen` wiring (`docs\HANDOFF.md:39` Anomaly 2).
- **Port delta:**
  - *Files:* `engine\law\src\movement.rs` (`server_regeneration`, line `1257`).
  - *What changes:* Change `if read_state!(world, state.health) < 20.0` at `engine\law\src\movement.rs:1257` to `if read_state!(world, state.health) > 0.0 && read_state!(world, state.health) < 20.0` (do NOT early-return the whole `server_regeneration` function when dead, because saturation and food legs still run).
  - *Bit-exact test:* Assert at `server_player_tick = 20`: `health = 0.0` stays `0x00000000` (and `-0.0f` stays untouched) while `saturation` `5.0 -> 6.0` (`0x40c00000`) and `food_level` `10 -> 11`; `health = f32::from_bits(1)` heals to `1.0` (`0x3f800000`); `health = 10.0` heals to `11.0`; `tick = 30` leaves dead `health = 0.0, sat = 5.0` and raises `food_level` `10 -> 11`.

#### 10. `port-portal-entry-and-create` (`local-20261117-1537-1-0-port-portal-entry-and-create`) — **short of `reproduced`**
- **Checker grade:** `lead` (`check.md:1`).
- **What went right vs what went wrong:** Almost everything reproduced (tick-phase order `baseTick -> handlePortal` before `aiStep -> checkInsideBlocks`, post-increment `portalTime++ >= wait` firing at entry `T + wait + 1`, `-4/tick` decay and processor drop on `portalTime <= 0`, `createPortal` 20-write hostable footprint and 44-write clear-box fallback with floor `max(minY + 1, 70)`). **One formula in the dossier failed adversarial testing:** the fallback Y **ceiling** was stated as `maxY - 9`, whereas `PortalForcer.createPortal` (`@33-57, @464-469`) actually computes `min(getMaxY(), getMinY() + getLogicalHeight() - 1) - 9`. The checker falsified `maxY - 9` on vanilla Nether dimensions (`minY=0, maxY=255, logicalHeight=128`: entry `y=200` clamps to `118`, not `200 < 246`) and custom dimensions (`C3`: `246` vs `247`).
- **Engine state:** **Lacks it** — `engine\law\src\world.rs:212` has only `end_portal_or_gateway: bool`, no `NetherPortalBlock` / `PortalProcessor` / `PortalForcer`.
- **Unblocks:** `docs\plan.md:52` Step 8 (`P4c`, Nether portal transit & creation).
- **Port delta:** Hold on `createPortal` ceiling until the narrower re-check below settles it to `reproduced`; or if wired after re-run, use `min(max_y, min_y + logical_height - 1) - 9`.
- **Narrower lens for wave 2:** `port-portal-create-ceiling` (below in Part 2) — verify ONLY `PortalForcer.createPortal`'s `topUsable` / fallback Y clamp (`floor`, `ceiling`, and empty return when `ceiling < floor`) across vanilla Overworld, Nether, and reduced-`logicalHeight` dimensions, plus `getExitPortal` / `findClosestPortalPosition` link if needed. (All counter/decay/20-vs-44 footprint facts are already verified by both author and checker!)

#### 11. `port-lava-entity-inside` (`local-20261117-1637-1-0-port-lava-entity-inside`) — **no check (`rate-limited`)**
- **Checker grade:** `rate-limited` (no `check.md`; author produced `dossier.md` and `reproduce.sh` at `16:37`, then the checker hit GLM's 5-hour quota before verifying it).
- **Author's claimed fact (unverified by checker):** `LavaFluid.entityInside` (`@0-35`) enqueues `CLEAR_FREEZE`, `LAVA_IGNITE` (`remainingFireTicks = max(old, 300)` unless `fireImmune()`), and `runAfter(LAVA_IGNITE, Entity::lavaHurt)` (`server-only`: `hurtServer(lava, 4.0f)` with `invulnerableTime=20, lastHurt=4.0`, blocked while `(float)invulnerableTime > 10.0f && amount <= lastHurt`, suppressed by `FIRE_RESISTANCE`, armor-reducible, difficulty-scaled). Executed by `StepBasedCollector.applyAndClear` at sweep end.
- **Engine state:** **Lacks it** — `engine\law\src\movement.rs:1054` explicitly refuses `FluidKind::Lava` (`"FluidState.entityInside: LavaFluid CLEAR_FREEZE, LAVA_IGNITE and deferred lava damage"`), and `movement.rs:1239-1251` (`accept_raw_hit`) is not yet wired to lava contact.
- **Unblocks:** `engine\law\src\movement.rs:1054` refusal, batch `P3a`.
- **What went wrong & narrower lens for wave 2:** Only the checker quota ran out, AND the author noted two untested bytecode-only spots: multi-lava-cell double-enqueue in `StepBasedCollector` (`checkInsideBlocks` derefs `debugSubscribers`) and `Player.hurtServer` difficulty/fire-resistance/armor scaling on lava. **Narrower lens:** `port-lava-hurt-and-collector` — re-ask specifically the unchecked `LavaFluid.entityInside` -> `StepBasedCollector` multi-cell enqueue order + `Player.hurtServer` lava damage path (difficulty scaling, `FIRE_RESISTANCE`, `invulnerableTime > 10.0f` cooldown) with direct jar execution so both author and checker finish in one short lane.

---

### 2. Wiki drafts

Wiki entries in `lab\wiki\`'s untruncated format (`jobs\port-digest\wiki\*.md`) for all 9 settled (`reproduced`) results are written to `jobs\port-wave-2\wiki\`:
- `jobs\port-wave-2\wiki\port-mining-destroy-speed.md`
- `jobs\port-wave-2\wiki\port-block-support-pop-family.md`
- `jobs\port-wave-2\wiki\port-boat-land-and-collision.md`
- `jobs\port-wave-2\wiki\port-boat-mount-dismount.md`
- `jobs\port-wave-2\wiki\port-elytra-firework-and-impact.md`
- `jobs\port-wave-2\wiki\port-pearl-raytrace-and-landing.md`
- `jobs\port-wave-2\wiki\port-minor-collision-acos.md`
- `jobs\port-wave-2\wiki\port-boat-trig-index.md`
- `jobs\port-wave-2\wiki\port-regen-dead-guard-and-phase.md`

---

### 3. Builder batches for today's settled results (disjoint by file)

Counting the 7 newly settled actionable deltas (items #7 `port-minor-collision-acos` and #8 `port-boat-trig-index` confirm the Rust code is already bit-exact and need tests only; #10 and #11 wait for wave-2 checker reproduction):

| Batch | Files touched (disjoint) | Settled lenses included | Unblocks |
|---|---|---|---|
| **Batch A (Ready now — Mining speed)** | `engine\law\src\mining.rs` | `#1` `port-mining-destroy-speed` (`player_destroy_speed` + `ToolRule` evaluation + 23 bit-exact cases) | `docs\plan.md:49` Step 5 (`P2c`) |
| **Batch B (Ready now — Step 8 Boat & Projectiles & Elytra)** | `engine\law\src\boat.rs`, `engine\law\src\elytra.rs` (new), `engine\law\src\pearl.rs` (new), `engine\law\src\lib.rs` | `#3` `port-boat-land-and-collision`, `#4` `port-boat-mount-dismount`, `#5` `port-elytra-firework-and-impact`, `#6` `port-pearl-raytrace-and-landing`, `#8` `port-boat-trig-index` (test only) | `docs\plan.md:52` Step 8 (`P4a`, `P4b`, `P4c`, `R-brach`) |
| **Batch C (After Step 5 M2/M3 or folded into M2/M3 — Support pops)** | `engine\law\src\world.rs`, `engine\law\src\interact.rs`, `engine\world\tools\ModelTable.java` | `#2` `port-block-support-pop-family` (`SupportRule`, `DoorBlock` pop+mirror, `ScaffoldingBlock` 1t schedule + `getDistance`, dead-pop exclusion) | `docs\plan.md:49` Step 5 (`P2a`, ledger) |
| **Batch D (After the sprint port — touches `movement.rs`)** | `engine\law\src\movement.rs` | `#9` `port-regen-dead-guard-and-phase` (`health > 0.0` guard at `movement.rs:1257`), `#7` `port-minor-collision-acos` (regression test only at `:803-819`) | Batch `P3b`, Anomaly 2 & Anomaly 12 |

*(Note: Batch B does **not** touch `movement.rs` or `state.rs` — it adds the standalone `elytra.rs` and `pearl.rs` laws and extends `boat.rs`, keeping `lib.rs` export-only; wiring `movement.rs:124-132` refusals waits until after the sprint port too, or joins Batch D.)*

---

## Part 2: Wave 2 Port Lenses for GLM

28 narrow, jar-backed port lenses in `jobs\port-wave-2\lenses\`, none colliding with `lab\lenses\` or re-asking facts already settled in `lab\wiki\index.md` or Part 1.

### 1. Lenses in priority order

#### Priority 1: What his recorded tracks need that the engine lacks (`Infinity Tower`, `Diversity`, `Parkour Race`)
1. `port-gamemode-ability-switch`: `GameType.updatePlayerAbilities` & `ServerPlayer.setGameMode` / `MultiPlayerGameMode.setLocalMode` exact writes (`mayfly`, `flying`, `invulnerable`, `instabuild`, `mayBuild`, `flyingSpeed`) and side effects across mode switches — `parkour_race` Creative/Survival switches, `docs\plan.md` step 3.
2. `port-flying-horizontal-speed`: `Player.getFlyingSpeed` (`flyingSpeed` vs `*2.0f` sprint) and `LivingEntity.travelInAir` ground-friction/speed-factor/sprint gates when `abilities.flying == true` — `parkour_race`, `engine\law\src\movement.rs:133-135`, batch `P4e`.
3. `port-sprint-food-and-item-reset`: Every write to `LocalPlayer.sprintTriggerTime` (`isSlowDueToUsingItem`, food, blindness, passenger) and Creative `hasEnoughFoodToStartSprinting` bypass — `docs\HANDOFF.md:44` Anomaly 10, `engine\law\src\movement.rs:217-233`, `parkour_race`.

#### Priority 2: Editing the world (`docs\plan.md` step 5)
4. `port-place-slab-stair-trapdoor`: `SlabBlock`, `StairBlock`, and `TrapDoorBlock` `getStateForPlacement` (`HALF`/`TYPE`/`FACING`/`SHAPE`/`WATERLOGGED` and `clickLocation.y - pos.y > 0.5`) plus `SlabBlock.canBeReplaced` double-slab merge — `docs\plan.md` step 5, `trapdoorparkour`, `stonestaircase`.
5. `port-place-door-ladder-scaffold`: `DoorBlock.getStateForPlacement`/`setPlacedBy`, `LadderBlock.getStateForPlacement`/`canSurvive`, and `ScaffoldingBlock.getStateForPlacement`/`ScaffoldingBlockItem.updatePlacementContext` side-extension & flags — `docs\plan.md` step 5, `carpetwater` (`scaffolding:3`), `doorcorridor`.
6. `port-destroy-block-side-effects`: `ServerPlayerGameMode.destroyBlock` -> `playerWillDestroy`, `removeBlock` (waterlogged block -> water source vs air!), `DoorBlock` creative/survival partner break, tool durability, and `0.005f` exhaustion — `docs\plan.md` step 5 ("leave no damage" ledger), `engine\law\src\mining.rs` `DestroyIntent`.
7. `port-interact-lever-and-redstone-door`: `LeverBlock.useWithoutItem`/`pull` (`setBlock` flags & neighbor cells) and `DoorBlock`/`TrapDoorBlock`/`FenceGateBlock` `neighborChanged` (`hasNeighborSignal` edge-vs-level `OPEN` rule for iron & wooden doors) — `docs\plan.md` step 5 (`P2a`).
8. `port-tripwire-hook-and-string`: `TripWireBlock.entityInside`/`checkPressed`/`tick`/`playerWillDestroy` and `TripWireHookBlock.calculateState` scan distance, `setBlock` flags, `10`-tick delay, and zero player mutation — `docs\plan.md` step 5 (`world_hook`), `engine\world\tools\ModelTable.java:38`.
9. `port-mining-creative-and-abort-gate`: `ServerPlayerGameMode.handleBlockBreakAction` Creative insta-break (ignoring `-1.0f` bedrock hardness), air START, non-matching/inactive STOP, and `canInteractWithBlock` reach/restriction gates — `engine\law\src\mining.rs:99-102`, `docs\plan.md` step 5 (`P2c`).
10. `port-interact-reach-and-clip`: `Player.blockInteractionRange()` (`BLOCK_INTERACTION_RANGE` default & `canInteractWithBlock` slack), `Entity.pick` raycast origin/direction across standing/crouching/crawling poses, and `BlockPlaceContext.getNearestLookingDirections` — `docs\plan.md` step 5, `engine\law\src\blocks.rs:187`.

#### Priority 3: Carried movement and wormholes (`docs\plan.md` step 8, incl. Part 1 narrower lenses)
11. `port-portal-create-ceiling`: `PortalForcer.createPortal` `topUsable` ceiling `min(getMaxY(), getMinY() + getLogicalHeight() - 1) - 9`, floor `max(getMinY() + 1, 70)`, `Optional.empty` gate, and `WorldBorder.clampToBounds` — `docs\plan.md` step 8 (`P4c`), closes `port-portal-entry-and-create` (`lead -> reproduced`).
12. `port-lava-hurt-and-collector`: `LavaFluid.entityInside` -> `StepBasedCollector` two-cell enqueue order and `Player.hurtServer`/`LivingEntity.hurtServer` lava difficulty scaling, `FIRE_RESISTANCE`, `invulnerableTime > 10.0f` cooldown, and `0.1f` exhaustion — `engine\law\src\movement.rs:1054`, batch `P3a`, closes unchecked `port-lava-entity-inside`.
13. `port-elytra-start-and-stop`: `LocalPlayer.aiStep` mid-air jump-to-glide packet predicate, `LivingEntity.canGlide` (`DataComponents.GLIDER`, armor slots, `nextDamageWillBreak`, `LEVITATION`), and `LivingEntity.updateFallFlying` durability wear interval — `docs\plan.md` step 8 (`P4b`), `elytra_challenge_2/3`, `R-brach`.
14. `port-boat-place-and-break`: `BoatItem.use` (`ClipContext.Fluid.ANY`, `isPickable` entity check, `noCollision` spawn box) and `VehicleEntity.hurtServer`/`tick` boat damage decay and `> 40.0f` / Creative break drop — `docs\plan.md` step 5 & step 8 (`P4a`).
15. `port-boat-water-snap-and-paddle`: `AbstractBoat.floatBoat` (`@13-135`) `IN_AIR -> water` snap (`getWaterLevelAbove - bbHeight + 0.101`, `noCollision`, early drag skip) plus `setInput`/`handleMoveVehicle` — `docs\plan.md` step 8 (`P4a`), closes bytecode-only `@13-135` in `port-boat-land-and-collision`.
16. `port-pearl-throw-origin-and-item`: `EnderpearlItem.use` -> `Projectile.spawnProjectileFromRotation`/`shootFromRotation`/`shoot` exact spawn Y (`eyeY - 0.1`), thrower-momentum add order, `ItemCooldowns(20)`, and stack decrement — `docs\plan.md` step 8 (`P4c`).
17. `port-portal-teleport-transition`: `NetherPortalBlock.getPortalDestination`/`getDimensionTransitionFromExit`, `PortalShape.getRelativePosition`/`findCollisionFreePosition` (`8:1` scale, axis-swap yaw & velocity rotation), and `Entity.handlePortal` cooldown when destination is `null` — `docs\plan.md` step 8 (`P4c`).

#### Priority 4: The world's own laws that a bot meets (fluids flowing, falling blocks, scheduled ticks)
18. `port-fluid-spread-distance`: `FlowingFluid.spread`/`spreadToSides`/`getSlopeDistance` (`getSlopeFindDistance`, `getDropOff`, falling-over-horizontal priority) and `WaterFluid.beforeDestroyingBlock` / `LavaFluid.spreadTo` wash-away & cobblestone/obsidian writes — `docs\plan.md` step 5 & step 8 (`P2b`, `watertunnel`).
19. `port-falling-block-flight-and-anvil`: `FallingBlockEntity.fall`/`tick` (`@0-241`: `(x+0.5, y, z+0.5)` spawn, `-0.04` gravity before `move`, `0.98` drag after `move`, `time > 100 / 600` discard), `ConcretePowderBlock` solidification, and anvil/dripstone `causeFallDamage` — `docs\plan.md` step 5 & step 8.
20. `port-scheduled-tick-drain-and-cap`: `ServerLevel.tick` phase order (`blockTicks` vs `fluidTicks` vs `runBlockEvents` vs entities/players) and `LevelTicks.tick`/`collectTicks`/`runCollectedTicks` `65536` per-tick cap, excess deferral, and `subTickOrder` lifetime — `docs\plan.md` step 5 & step 8 (`P2a`).
21. `port-piston-extend-retract-motion`: `PistonBaseBlock.triggerEvent`, `PistonStructureResolver.resolve` (`12`-block limit, `PushReaction`, slime/honey stickiness), and `PistonMovingBlockEntity.tick`/`moveCollidedEntities` entity push + slime-block launch velocity — `docs\plan.md` step 5 & step 8 (`P2a`).

#### Priority 5: Remaining backlog `NOT IN` / `PARTIAL` (`jobs\port-backlog\report.md`)
22. `port-effect-levitation-slowfall-dolphin`: `MobEffects.LEVITATION` (`(0.05*(amp+1) - vy)*0.2`), `SLOW_FALLING` (`0.01` cap when `vy <= 0`), `DOLPHINS_GRACE` (`0.96f` water drag), and `WebBlock.entityInside` `WEAVING` `(0.5, 0.25, 0.5)` — `engine\law\src\movement.rs:121-123`, `jobs\port-lenses\report.md:52`.
23. `port-fire-burn-and-extinguish`: `Entity.baseTick` `remainingFireTicks` countdown, `% 20` `onFire` `1.0f` damage, `isInWaterOrRain`/powder-snow extinguish, and `BaseFireBlock`/`CampfireBlock` `entityInside` ignition/damage — `engine\world\tools\ModelTable.java:37`, `port-lava-entity-inside` Limits:47.
24. `port-hurt-block-inside-family`: `SweetBerryBushBlock.entityInside`, `CactusBlock.entityInside`, `FarmlandBlock.fallOn` trample, and `PointedDripstoneBlock.fallOn` damage multiplier (plus listing any remaining `ModelTable.java:36-41` `HOOKS` overriders) — `engine\world\tools\ModelTable.java:36-41`, batch `P3a`.
25. `port-equipment-location-effects`: `soul_speed`, `depth_strider`, `swift_sneak`, and `feather_falling` jar enchantment JSONs + `LivingEntity.getBlockSpeedFactor` (`soul_sand` `0.4f` bypass under Soul Speed) — `engine\law\src\attributes.rs`, batch `P4e`.
26. `port-entity-push-and-crowd-loop`: `LivingEntity.pushEntities` (`aiStep @817`, `MAX_ENTITY_CRAMMING`), `AbstractBoat.push` Y-gate, and `Level.getEntityCollisions` solid entity colliders (`Boat`, `Shulker`) — `engine\law\src\entity_contact.rs`, batch `P4d`, `docs\HANDOFF.md:41` Anomaly 5.
27. `port-armor-absorption-mitigation`: `LivingEntity.actuallyHurt` -> `CombatRules.getDamageAfterAbsorb` / `getDamageAfterMagicAbsorb` (EPF cap `20`, Feather Falling / Protection, `bypasses_armor` / `bypasses_enchantments` tags) and absorption subtraction before `setHealth` — `engine\law\src\movement.rs:848`, `docs\PRINCIPLES.md`.
28. `port-unloaded-chunk-client-gate`: `LivingEntity.travelInAir` `@101-159` `isClientSide()` gate on unloaded-chunk `vy = -0.1 / 0.0` vs server `vy - gravity`, and `hasChunkAt`/`isLoaded` queried position — `engine\law\src\movement.rs:444-450`, `jobs\port-backlog\report.md:175-176`.

---

### 2. What is still left after this wave (count by area)

Once these 28 lenses run, the remaining unported or unverified 26.2 surface breaks down into **~53 discrete mechanics** across 6 areas (not counting pure Rust integration batches `P1a`–`P4e` already backed by settled lenses):

| Area | Remaining count | What remains (and why deferred beyond Wave 2) |
|---|---:|---|
| **Non-block item uses & combat (`item-use-writes` / PvP)** | ~18 | Bows/crossbows, Trident Riptide (`TridentItem.releaseUsing@348`), Wind Charge (`WindCharge.explode`), Fishing Rod, Snowball/Egg, Food/Potion consumption & `startUsingItem` duration, Shield blocking, Chorus Fruit, `Player.attack(Entity)` / `MaceItem` / `AbstractArrow` attacker slowdown & knockback resolvers |
| **Non-boat vehicles & mounts (`LivingEntity.travelRidden`)** | ~6 | `AbstractMinecart` kinematics, rail curves/slopes, powered/detector/activator rails, minecart mount/dismount, and rideable living mounts (horse jump/speed, pig/strider) |
| **Rare `ModelTable` block hooks & context edge cases** | ~10 | Remaining `hook-catalogue` overriders not in Wave 2 (`EndPortalBlock`, `EndGatewayBlock`, `WitherRoseBlock`, `PitcherCropBlock`, `TurtleEggBlock`, `RedstoneOreBlock`, `BigDripleafBlock`, `PowderSnowBlock` `fallDistance > 2.5` collision context, `ShulkerBoxBlock` peek AABB, `BubbleColumnBlock` spread) |
| **Redstone & complex block/world updates** | ~8 | Repeaters, comparators, observers, redstone torches, dispensers/droppers, hoppers, water-plant (`Kelp`/`Seagrass`) source rules, and chunk-border/ticket boundaries |
| **Mobs, AI & End/Environment laws (Phase 8+)** | ~8 | Zombie/skeleton/breeze/mob pathing & targeting (`AGENTS.md` / `docs\plan.md` Phase 8), Ender Dragon knockback, End Gateway/Portal return, weather/lightning/daytime rules |
| **Miscellaneous player edge cases** | ~3 | `Pose::SPIN_ATTACK` (Riptide) / sleeping pose transitions, `ServerPlayer` `tickCount` sync/death-respawn transition (`docs\plan.md` "Death as a tool"), and HotSpot `_dcos` Rust intrinsic transcription verification |

---

### 3. Queue

queue: port-gamemode-ability-switch port-flying-horizontal-speed port-sprint-food-and-item-reset port-place-slab-stair-trapdoor port-place-door-ladder-scaffold port-destroy-block-side-effects port-interact-lever-and-redstone-door port-tripwire-hook-and-string port-mining-creative-and-abort-gate port-interact-reach-and-clip port-portal-create-ceiling port-lava-hurt-and-collector port-elytra-start-and-stop port-boat-place-and-break port-boat-water-snap-and-paddle port-pearl-throw-origin-and-item port-portal-teleport-transition port-fluid-spread-distance port-falling-block-flight-and-anvil port-scheduled-tick-drain-and-cap port-piston-extend-retract-motion port-effect-levitation-slowfall-dolphin port-fire-burn-and-extinguish port-hurt-block-inside-family port-equipment-location-effects port-entity-push-and-crowd-loop port-armor-absorption-mitigation port-unloaded-chunk-client-gate


