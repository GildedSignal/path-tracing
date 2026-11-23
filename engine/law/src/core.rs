//! The player as a causal chain: state + cause → effect, then the next cause.
//!
//! One input is the cause the server observes per tick, so the face is `tick`; inside it the
//! causes fire in the order the game fires them (fluids sensed, the fire clock, the flag
//! machines, the jump, the move, the cells crossed, the pose, the ledger). Between causes the
//! generators of the basis carry the state (`body` for travel, `sweep` for the move, `clock`
//! for counters, `ray` for the cells crossed). Every damage goes through one `hurt`.
use crate::{
    basis::{body::{input_vector, AffineBody, Moved, LIVING}, clock::{self, COOLDOWN}, ray::{cells_along, BlockSet, SeenBlocks}, sweep::axis_order},
    math, player::GameType, world::block_pos, Aabb, Axis, Block, BlockPos, CollisionContext, ContactDamage, FluidKind,
    GroundedContact, HurtCooldown, Input, Inside, PlayerState, Pose, Vec3, VoxelShape, World,
};

/// A rule the port lacks, met during a tick: the law refuses by name. The state is then
/// partly written, so the caller discards it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal(pub String);
macro_rules! refuse { ($($rule:tt)*) => { return Err(Refusal(format!($($rule)*))) } }

/// What one tick hands outward: the move asked of `Entity.collide` and what it gave, and the
/// contact damage requested on the server's resource timeline.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effect { pub collision: Option<(Vec3, Vec3)>, pub contact: Option<ContactDamage> }

/// The same tick as `tick`; an unported rule is returned instead of panicking.
pub fn try_tick(s: &mut PlayerState, world: &impl World, input: &Input) -> Result<(), Refusal> { step(s, world, input).map(|_| ()) }
fn unported(s: &mut PlayerState, world: &impl World, input: &Input) -> Effect {
    step(s, world, input).unwrap_or_else(|Refusal(rule)| panic!("not implemented: {rule}"))
}
/// Player.tick; Entity.baseTick; LocalPlayer.aiStep; LivingEntity.aiStep.
pub fn tick(s: &mut PlayerState, world: &impl World, input: &Input) { unported(s, world, input); }
/// The tick, returning the request and result at Entity.collide.
pub fn tick_with_collision(s: &mut PlayerState, world: &impl World, input: &Input) -> Option<(Vec3, Vec3)> { unported(s, world, input).collision }
/// The tick, returning its contact damage request (resolved on the server's timeline, never here).
pub fn tick_with_contact(s: &mut PlayerState, world: &impl World, input: &Input) -> Option<ContactDamage> { unported(s, world, input).contact }

fn modeled_effects(s: &PlayerState) -> bool {
    let p = &s.profile;
    p.jump_boost.is_some() || p.speed_effect.is_some() || p.slowness_effect.is_some() || p.blindness || p.fire_resistance
        || p.levitation.is_some() || p.slow_falling || p.dolphins_grace || p.weaving || p.resistance.is_some() || !s.effect_book.is_empty()
}

/// One input, every cause it fires, in the game's order.
pub fn step(s: &mut PlayerState, world: &impl World, input: &Input) -> Result<Effect, Refusal> {
    world.observe_tick();
    use crate::state_reads::{StateField, StateReads, TICK_WRITES_FIRST as FIRST};
    // The previous input is read only where creative flight or the sprint window asks for it.
    let input_read = s.profile.mayfly || s.sprint_window > 0 || s.sprint_trigger_time > 0;
    world.state_footprint(StateReads(StateReads::ALL.minus(FIRST).0 | if input_read { StateReads::of(&[StateField::input]).0 } else { 0 }), FIRST);
    if s.profile.unmodeled_effects || s.effects && !modeled_effects(s) { refuse!("LivingEntity: status effects other than none"); }
    if s.profile.boat { refuse!("Boat movement"); }
    if s.profile.riding { refuse!("LivingEntity.travelRidden"); }
    let (start, start_box) = (s.position, s.bounding_box());
    s.movement_this_tick = None;
    (s.yaw, s.pitch) = (input.yaw, input.pitch);
    s.was_in_powder_snow = std::mem::take(&mut s.in_powder_snow);
    s.was_eye_in_water = s.eye_in_water;
    // Cause: the fluids around the body.
    immerse(s, world);
    if s.in_lava { s.fall_distance *= 0.5; }
    s.swimming = !s.flying && if s.swimming { s.sprinting && s.in_water } else {
        s.sprinting && s.underwater && world.fluid_block(block_pos(s.position)).fluid.kind == FluidKind::Water
    };
    // Cause: the fire clock (%20 checked before the decrement; negative grace untouched).
    let fire = s.remaining_fire_ticks;
    let burn = (fire > 0 && fire % 20 == 0 && !s.in_lava).then_some(ContactDamage::OnFire);
    if fire > 0 { set_fire(s, fire - 1); }
    // Cause: the input. The flag machines read the previous input before it is replaced.
    let was_shift = s.shift_key_down;
    let was_jumping = s.profile.mayfly && s.input.jump;
    let was_forward = (s.sprint_window > 0 || s.sprint_trigger_time > 0) && s.input.forward && !s.input.back;
    s.crouching = !s.flying && !s.swimming && fits(s, world, Pose::Crouching) && (was_shift || !fits(s, world, Pose::Standing));
    s.shift_key_down = input.sneak;
    s.input = *input;
    if !s.profile.no_physics {
        let d = 0.6f32 as f64 * 0.35;
        for (x, z) in [(-d, d), (-d, -d), (d, -d), (d, d)] { push_out(s, world, s.position.x + x, s.position.z + z); }
    }
    let slowly = s.crouching || s.pose == Pose::Swimming && !s.in_water;
    sprint(s, input, slowly, was_shift, was_forward);
    if s.profile.game_mode == GameType::Spectator { s.flying = true; }
    else if s.profile.mayfly && !was_jumping && input.jump {
        if s.jump_trigger_time == 0 { s.jump_trigger_time = 7; } else { s.flying = !s.flying; s.jump_trigger_time = 0; }
    }
    if s.flying {
        let dir = input.jump as i32 - input.sneak as i32;
        if dir != 0 { s.velocity.y += ((dir as f32 * s.profile.flying_speed) * 3.0f32) as f64; }
    }
    COOLDOWN.cool(&mut s.jump_trigger_time);
    if s.flying { s.fall_distance = 0.0; }
    s.speed = s.movement_speed_value() as f32;
    if !s.flying && s.in_water && input.sneak { s.velocity.y -= 0.04f32 as f64; }
    COOLDOWN.cool(&mut s.no_jump_delay);
    if s.velocity.horizontal_sqr() < 9.0e-6 { (s.velocity.x, s.velocity.z) = (0.0, 0.0); }
    if s.velocity.y.abs() < 0.003 { s.velocity.y = 0.0; }
    let mut movement = steer(input, slowly, s);
    s.on_climbable = climbable(s, world);
    if input.jump && !s.flying && s.health > 0.0 {
        let height = if s.in_lava { s.lava_height } else { s.water_height };
        let water = s.in_water && height > 0.0;
        if water && (!s.on_ground || height > 0.4) || s.in_lava && (!s.on_ground || s.lava_height > 0.4) {
            s.velocity.y += 0.04f32 as f64;
        } else if (s.on_ground || water && height <= 0.4) && s.no_jump_delay == 0 {
            jump(s, world);
            s.no_jump_delay = 10;
        }
    } else { s.no_jump_delay = 0; }
    if s.health <= 0.0 { movement = Vec3::default(); }
    if s.fall_flying {
        s.fall_distance = crate::elytra::check_fall_distance_accumulation(s.velocity.y, s.fall_distance);
        if s.on_ground || s.profile.levitation.is_some() { s.fall_flying = false; }
    }
    if s.profile.slow_falling || s.profile.levitation.is_some() { s.fall_distance = 0.0; }
    // Cause: the move (the body generator, then the sweep).
    let collision = travel(s, world, movement)?;
    world.observe_sweep(start_box, s.bounding_box());
    // Cause: the cells the move crossed.
    let contact = burn.or(cross(s, world, start)?);
    let before_pose = s.bounding_box();
    s.position.x = s.position.x.clamp(-29999999.0, 29999999.0);
    s.position.z = s.position.z.clamp(-29999999.0, 29999999.0);
    if s.on_ground && s.flying && s.profile.game_mode != GameType::Spectator { s.flying = false; }
    pose(s, world);
    world.observe_sweep(before_pose, s.bounding_box());
    s.on_climbable = climbable(s, world);
    // Cause: the distance walked, then the food clock.
    walked(s, s.position.minus(start));
    food(s, world);
    s.t += 1;
    Ok(Effect { collision, contact })
}

/// The sprint flag machine (LocalPlayer.aiStep @0-484, canStartSprinting, shouldStopRunSprinting,
/// shouldStopSwimSprinting): the input's effect on `sprinting` and its double-tap window.
pub fn sprint(s: &mut PlayerState, input: &Input, slowly: bool, was_shift: bool, was_forward: bool) {
    let p = &s.profile;
    let mut trigger = s.sprint_trigger_time;
    COOLDOWN.cool(&mut trigger);
    let slow_item = p.using_item && !p.use_item_can_sprint.expect("used item canSprint");
    if was_shift || slow_item || input.back { trigger = 0; }
    let forward = input.forward && !input.back;
    // shouldStopSwimSprinting asks isSprintingPossible(true): food reads through blindness and mayfly.
    let food = if s.sprinting && s.swimming { if p.blindness { 0 } else if p.mayfly { 7 } else { s.food_level } } else { s.food_level };
    let enough = food as f32 > 6.0f32;
    let possible = !p.blindness && (enough || p.mayfly) && (s.flying || !(s.in_water && !s.underwater));
    let mut on = s.sprinting;
    if !on && forward && possible && !slow_item && (!s.fall_flying || s.underwater) && (!slowly || s.underwater) {
        if !was_forward { if trigger > 0 { on = true; } else { trigger = s.sprint_window; } }
        if !on && input.sprint { on = true; }
    }
    if on && if s.swimming { !enough || !s.in_water || !forward && !s.on_ground && !input.sneak }
        else { !possible || !forward || s.horizontal_collision && !s.minor_horizontal_collision } { on = false; }
    s.sprint_trigger_time = trigger;
    if on != s.sprinting { s.set_sprinting(on); }
}

// LocalPlayer.suffocatesAt; LocalPlayer.moveTowardsClosestSpace
fn push_out(s: &mut PlayerState, world: &impl World, x: f64, z: f64) {
    let body = s.bounding_box();
    let suffocates = |p: BlockPos| world.collides_with_suffocating_block(
        Aabb::new(p[0] as f64, body.min.y, p[2] as f64, p[0] as f64 + 1.0, body.max.y, p[2] as f64 + 1.0).deflate(1.0e-7), CollisionContext::of(s));
    let p = block_pos(Vec3::new(x, s.position.y, z));
    if !suffocates(p) { return; }
    let (dx, dz) = (x - p[0] as f64, z - p[2] as f64);
    let mut best = (f64::MAX, None);
    for (axis, step, distance) in [(Axis::X, -1, dx), (Axis::X, 1, 1.0 - dx), (Axis::Z, -1, dz), (Axis::Z, 1, 1.0 - dz)] {
        let mut q = p;
        q[axis as usize] += step;
        if distance < best.0 && !suffocates(q) { best = (distance, Some((axis, step))); }
    }
    if let (_, Some((axis, step))) = best { s.velocity = s.velocity.with(axis, 0.1 * step as f64); }
}

// KeyboardInput.tick; LocalPlayer.modifyInput; LocalPlayer.modifyInputSpeedForSquareMovement
fn steer(i: &Input, slowly: bool, s: &PlayerState) -> Vec3 {
    let (mut x, mut z) = ((i.left as i32 - i.right as i32) as f32, (i.forward as i32 - i.back as i32) as f32);
    let length = math::sqrt(x * x + z * z);
    if length < 1.0e-4 { return Vec3::default(); }
    (x, z) = (x / length * 0.98f32, z / length * 0.98f32);
    if s.profile.using_item { let m = s.profile.use_item_speed_multiplier.expect("used item speedMultiplier"); (x, z) = (x * m, z * m); }
    if slowly { (x, z) = (x * s.profile.sneaking_speed, z * s.profile.sneaking_speed); }
    let length = math::sqrt(x * x + z * z);
    if length > 0.0 {
        let r = 1.0f32 / length;
        (x, z) = (x * r, z * r);
        let (a, b) = (x.abs(), z.abs());
        let ratio = if b > a { a / b } else { b / a };
        let speed = (length * math::sqrt(1.0f32 + ratio * ratio)).min(1.0);
        (x, z) = (x * speed, z * speed);
    }
    Vec3::new(x as f64, 0.0, z as f64)
}

// Entity.getOnPos
fn on_pos(s: &PlayerState, world: &impl World, offset: f32) -> BlockPos {
    let Some(p) = s.main_supporting_block else { return block_pos(Vec3::new(s.position.x, s.position.y - offset as f64, s.position.z)) };
    if offset <= 1.0e-5f32 { return p; }
    let b = world.hook_block(p);
    if offset as f64 <= 0.5 && b.fence || b.wall_or_gate { p } else { [p[0], math::floor(s.position.y - offset as f64), p[2]] }
}

// Entity.getBlockJumpFactor (both cells read); Entity.getBlockSpeedFactor (water stops at the feet)
fn jump_factor(s: &PlayerState, world: &impl World) -> f32 {
    let (here, below) = (world.hook_block(block_pos(s.position)).jump_factor, world.hook_block(on_pos(s, world, 0.500001)).jump_factor);
    if here as f64 == 1.0 { below } else { here }
}
fn speed_factor(s: &PlayerState, world: &impl World) -> f32 {
    let here = world.hook_block(block_pos(s.position));
    if !here.water_block && here.speed_factor as f64 == 1.0 { world.hook_block(on_pos(s, world, 0.500001)).speed_factor } else { here.speed_factor }
}

// LivingEntity.jumpFromGround; ServerPlayer.jumpFromGround
fn jump(s: &mut PlayerState, world: &impl World) {
    let power = s.profile.jump_strength as f32 * 1.0f32 * jump_factor(s, world) + s.jump_boost_power();
    if power > 1.0e-5f32 {
        s.velocity.y = (power as f64).max(s.velocity.y);
        if s.sprinting {
            let yaw = (s.yaw * 0.017453292f32) as f64;
            s.velocity = s.velocity.plus(Vec3::new((-math::sin(yaw)) as f64 * 0.2, 0.0, math::cos(yaw) as f64 * 0.2));
        }
    }
    exhaust(s, if s.sprinting { 0.2 } else { 0.05 });
}

// LivingEntity.onClimbable; LivingEntity.trapdoorUsableAsLadder
fn climbable(s: &PlayerState, world: &impl World) -> bool {
    let p = block_pos(s.position);
    let b = world.hook_block(p);
    b.climbable || b.open_trapdoor_facing.is_some_and(|f| world.hook_block([p[0], p[1] - 1, p[2]]).ladder_facing == Some(f))
}

// LivingEntity.getEffectiveGravity
fn gravity(s: &PlayerState, vy: f64) -> f64 {
    if vy <= 0.0 && s.profile.slow_falling { s.profile.gravity.min(0.01) } else { s.profile.gravity }
}

// Player.travel; LivingEntity.travel
fn travel(s: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    if s.swimming {
        let look_y = -math::sin((s.pitch * 0.017453292f32) as f64) as f64;
        let factor = if look_y < -0.2 { 0.085 } else { 0.06 };
        if look_y <= 0.0 || s.input.jump || world.fluid_block(block_pos(Vec3::new(s.position.x, s.position.y + 1.0 - 0.1, s.position.z))).fluid.kind != FluidKind::Empty {
            s.velocity.y += (look_y - s.velocity.y) * factor;
        }
    }
    let flying_vy = s.flying.then_some(s.velocity.y);
    let collision = if !s.flying && (s.in_water || s.in_lava) { travel_fluid(s, world, movement)? }
        else if s.fall_flying && !climbable(s, world) { glide(s, world)? }
        else {
            let collision = travel_air(s, world, movement)?;
            if s.fall_flying { s.fall_flying = false; }
            collision
        };
    if let Some(vy) = flying_vy { s.velocity.y = vy * 0.6; }
    Ok(collision)
}

// LivingEntity.travelFallFlying: the elytra kernel, then the wall's hit.
fn glide(s: &mut PlayerState, world: &impl World) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    let pre_h = s.velocity.horizontal_sqr().sqrt();
    let look = math::calculate_view_vector(s.yaw, s.pitch);
    s.velocity = crate::elytra::update_fall_flying_movement(s.velocity, look, s.pitch, gravity(s, s.velocity.y));
    let collision = move_body(s, world, s.velocity)?;
    if let Some(amount) = crate::elytra::fly_into_wall_damage(s.horizontal_collision, pre_h, s.velocity.horizontal_sqr().sqrt()) {
        hurt(s, Hit { amount, fall: false, ..Hit::FALL });
    }
    Ok(collision)
}

// LivingEntity.travelInAir; handleRelativeFrictionAndCalculateMovement; handleOnClimbable
fn travel_air(s: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    let below = on_pos(s, world, 0.500001);
    let f = if s.on_ground { math::modified_friction(world.hook_block(below).friction, s.profile.friction_modifier) } else { 1.0 };
    let acceleration = if !s.on_ground {
        if s.flying { s.profile.flying_speed * if s.sprinting { 2.0f32 } else { 1.0 } } else if s.sprinting { 0.025999999f32 } else { 0.02f32 }
    } else if f as f64 > 0.6 { s.speed * (0.21600002f32 / (f * f * f)) } else { s.speed };
    s.velocity = s.velocity.plus(input_vector(movement, acceleration, s.yaw));
    if climbable(s, world) {
        s.fall_distance = 0.0;
        let c = 0.15f32 as f64;
        let v = &mut s.velocity;
        (v.x, v.z, v.y) = (v.x.clamp(-c, c), v.z.clamp(-c, c), v.y.max(-c));
        if v.y < 0.0 && !world.hook_block(block_pos(s.position)).scaffolding && s.shift_key_down { s.velocity.y = 0.0; }
    }
    let collision = move_body(s, world, s.velocity)?;
    let mut v = s.velocity;
    let pre_bump_vy = v.y;
    if (s.horizontal_collision || s.input.jump) && (climbable(s, world) || s.was_in_powder_snow && s.profile.leather_boots) { v.y = 0.2; }
    // The unloaded-chunk fallback of travelInAir is client-only: the server goes to gravity.
    let vy = match s.profile.levitation {
        Some(amp) => v.y + (0.05 * ((amp as i32 + 1) as f64) - v.y) * 0.2,
        None => v.y - gravity(s, pre_bump_vy),
    };
    let m = s.profile.air_drag_modifier;
    let row = AffineBody { air_drag: math::modified_friction(LIVING.air_drag, m), drag_y: math::modified_friction(LIVING.drag_y, m), ..LIVING };
    world.observe_air_travel(movement.z * acceleration as f64, f * row.air_drag);
    s.velocity = row.friction_drag(v, vy, f);
    Ok(collision)
}

// LivingEntity.travelInFluid; travelInWater; travelInLava; getFluidFallingAdjustedMovement
fn travel_fluid(s: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    let falling = s.velocity.y <= 0.0;
    let g = gravity(s, s.velocity.y);
    let old_y = s.position.y;
    let fall_adjust = |s: &mut PlayerState| if g != 0.0 && !s.sprinting {
        let y = s.velocity.y - g / 16.0;
        s.velocity.y = if falling && (s.velocity.y - 0.005).abs() >= 0.003 && y.abs() < 0.003 { -0.003 } else { y };
    };
    let collision = if s.in_water {
        let (mut drag, mut acceleration) = (if s.sprinting { 0.9f32 } else { 0.8f32 }, 0.02f32);
        let efficiency = s.profile.water_movement_efficiency * if s.on_ground { 1.0 } else { 0.5 };
        if efficiency > 0.0 { drag += (0.54600006f32 - drag) * efficiency; acceleration += (s.speed - acceleration) * efficiency; }
        if s.profile.dolphins_grace { drag = 0.96f32; }
        s.velocity = s.velocity.plus(input_vector(movement, acceleration, s.yaw));
        let collision = move_body(s, world, s.velocity)?;
        if s.horizontal_collision && climbable(s, world) { s.velocity.y = 0.2; }
        s.velocity = s.velocity.multiply(Vec3::new(drag as f64, 0.8f32 as f64, drag as f64));
        fall_adjust(s);
        collision
    } else {
        s.velocity = s.velocity.plus(input_vector(movement, 0.02, s.yaw));
        let collision = move_body(s, world, s.velocity)?;
        if s.lava_height <= 0.4 { s.velocity = s.velocity.multiply(Vec3::new(0.5, 0.8f32 as f64, 0.5)); fall_adjust(s); }
        else { s.velocity = s.velocity.scale(0.5); }
        if g != 0.0 { s.velocity.y += -g / 4.0; }
        collision
    };
    let v = s.velocity;
    let lifted = s.bounding_box().moved(Vec3::new(v.x, v.y + 0.6f32 as f64 - s.position.y + old_y, v.z));
    if s.horizontal_collision && world.no_collision(lifted, CollisionContext::of(s)) && !any_fluid(world, lifted) { s.velocity.y = 0.3f32 as f64; }
    Ok(collision)
}

// CollisionGetter.containsAnyLiquid
fn any_fluid(world: &impl World, b: Aabb) -> bool {
    let (lo, hi) = (block_pos(b.min), block_pos(b.max));
    (lo[0]..=hi[0]).any(|x| (lo[1]..=hi[1]).any(|y| (lo[2]..=hi[2]).any(|z| world.fluid_block([x, y, z]).fluid.kind != FluidKind::Empty)))
}

// Player.canPlayerFitWithinBlocksAndEntitiesWhen
fn fits(s: &PlayerState, world: &impl World, pose: Pose) -> bool {
    world.no_collision(Aabb::player(s.position, pose.height()).deflate(1.0e-7), CollisionContext::of(s))
}

// Player.updatePlayerPose; Player.getDesiredPose
fn pose(s: &mut PlayerState, world: &impl World) {
    if !fits(s, world, Pose::Swimming) { return; }
    let desired = if s.swimming { Pose::Swimming } else if s.fall_flying { Pose::FallFlying }
        else if s.shift_key_down && !s.flying { Pose::Crouching } else { Pose::Standing };
    s.pose = if s.profile.game_mode == GameType::Spectator || fits(s, world, desired) { desired }
        else if fits(s, world, Pose::Crouching) { Pose::Crouching } else { Pose::Swimming };
}

/// Player.canFallAtLeast.
pub fn can_fall(s: &PlayerState, world: &impl World, x: f64, z: f64, depth: f64) -> bool {
    world.state_footprint(crate::state_reads::StateReads::ALL, crate::state_reads::StateReads(0));
    let b = s.bounding_box();
    world.no_collision(Aabb::new(b.min.x + 1.0e-7 + x, b.min.y - depth - 1.0e-7, b.min.z + 1.0e-7 + z, b.max.x - 1.0e-7 + x, b.min.y, b.max.z - 1.0e-7 + z), CollisionContext::of(s))
}

// Player.maybeBackOffFromEdge; Player.isAboveGround
fn back_off(s: &PlayerState, world: &impl World, mut v: Vec3) -> Vec3 {
    let step = s.profile.step_height as f64;
    if v.y > 0.0 || !s.shift_key_down || s.flying || !(s.on_ground || s.fall_distance < step && !can_fall(s, world, 0.0, 0.0, step - s.fall_distance)) { return v; }
    let delta = if cfg!(feature = "mutant-edge-backoff") { 0.06 } else { 0.05 };
    let (dx, dz) = (v.x.signum() * delta, v.z.signum() * delta);
    let shrink = |c: &mut f64, d: f64| if c.abs() <= delta { *c = 0.0; } else { *c -= d; };
    while v.x != 0.0 && can_fall(s, world, v.x, 0.0, step) { if v.x.abs() <= delta { v.x = 0.0; break; } v.x -= dx; }
    while v.z != 0.0 && can_fall(s, world, 0.0, v.z, step) { if v.z.abs() <= delta { v.z = 0.0; break; } v.z -= dz; }
    while v.x != 0.0 && v.z != 0.0 && can_fall(s, world, v.x, v.z, step) { shrink(&mut v.x, dx); shrink(&mut v.z, dz); }
    v
}

// CollisionGetter.findSupportingBlock; Entity.checkSupportingBlock
fn support(s: &mut PlayerState, world: &impl World, moved: Vec3) {
    if !s.on_ground { (s.on_ground_no_blocks, s.main_supporting_block) = (false, None); return; }
    let b = s.bounding_box();
    let foot = Aabb::new(b.min.x, b.min.y - 1.0e-6, b.min.z, b.max.x, b.min.y, b.max.z);
    let find = |bounds: Aabb| {
        let (lo, hi) = world.collision_cell_bounds(bounds, block_pos(bounds.min).map(|v| v - 1), block_pos(bounds.max).map(|v| v + 1))?;
        let mut best: Option<(f64, BlockPos)> = None;
        for x in lo[0]..=hi[0] { for y in lo[1]..=hi[1] { for z in lo[2]..=hi[2] {
            let o = Vec3::new(x as f64, y as f64, z as f64);
            if !world.collision_shape([x, y, z], CollisionContext::of(s)).at(o).intersects(bounds) { continue; }
            let d = o.plus(Vec3::new(0.5, 0.5, 0.5)).minus(s.position).length_sqr();
            if best.is_none_or(|(old, q)| d < old || d == old && (y, z, x) > (q[1], q[2], q[0])) { best = Some((d, [x, y, z])); }
        } } }
        best.map(|(_, p)| p)
    };
    let found = find(foot);
    s.main_supporting_block = if found.is_some() || s.on_ground_no_blocks { found } else { find(foot.moved(Vec3::new(-moved.x, 0.0, -moved.z))) };
    s.on_ground_no_blocks = s.main_supporting_block.is_none();
}

// Entity.move; Entity.collide; Entity.restituteMovementAfterCollisions
fn move_body(s: &mut PlayerState, world: &impl World, mut wanted: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    if s.profile.no_physics { s.position = s.position.plus(wanted); return Ok(None); }
    if s.stuck_speed_multiplier.length_sqr() > 1.0e-7 {
        wanted = wanted.multiply(std::mem::take(&mut s.stuck_speed_multiplier));
        s.velocity = Vec3::default();
    }
    wanted = back_off(s, world, wanted);
    let step = s.profile.step_height;
    let step = if cfg!(feature = "mutant-step-up") { f32::from_bits(step.to_bits() + 1) } else { step };
    let m = Moved::new(wanted, crate::basis::sweep::collide(world, s.bounding_box(), CollisionContext::of(s), wanted, || s.on_ground, step));
    let moved = m.moved;
    if m.applies() {
        if s.fall_distance != 0.0 && moved.length_sqr() >= 1.0 {
            let end = s.position.plus(moved.normalize().scale(moved.length_sqr().sqrt().min(8.0)));
            if world.fall_reset_clip(s.position, end) { s.fall_distance = 0.0; }
        }
        s.movement_this_tick = Some(wanted);
        s.position = s.position.plus(moved);
    }
    (s.horizontal_collision, s.vertical_collision, s.vertical_collision_below, s.on_ground) = (m.horizontal(), m.y, m.below(), m.below());
    support(s, world, moved);
    s.minor_horizontal_collision = minor_collision(s, moved);
    if !s.in_water { immerse(s, world); }
    let block = world.hook_block(on_pos(s, world, 0.2));
    land(s, block, moved.y)?;
    if m.wanted.y.abs() > 0.0 && m.y || m.horizontal() {
        let shift = s.shift_key_down;
        let bounce = if shift { 0.0 } else { s.profile.bounciness };
        let vertical = if !m.below() { bounce } else if -s.velocity.y < s.profile.gravity || shift || block.suppresses_bounce { 0.0 } else { bounce.max(block.bounce as f64) };
        let mut v = s.velocity;
        m.restitute(&mut v, bounce, vertical, || (s.profile.gravity, math::modified_friction(0.98, s.profile.air_drag_modifier)));
        s.velocity = v;
    }
    let factor = speed_factor(s, world);
    let factor = (factor + s.profile.movement_efficiency * (1.0f32 - factor)) as f64;
    world.observe_speed_factor(factor);
    s.velocity = s.velocity.multiply(Vec3::new(factor, 1.0, factor));
    Ok(Some((wanted, moved)))
}

// LocalPlayer.isHorizontalCollisionMinor
fn minor_collision(s: &PlayerState, moved: Vec3) -> bool {
    if !s.horizontal_collision { return false; }
    let input = steer(&s.input, s.crouching || s.pose == Pose::Swimming && !s.in_water, s);
    let yaw = (s.yaw * 0.017453292f32) as f64;
    let (sin, cos) = (math::sin(yaw) as f64, math::cos(yaw) as f64);
    let intended = Vec3::new(input.x * cos - input.z * sin, 0.0, input.z * cos + input.x * sin);
    let (a, b) = (intended.horizontal_sqr(), moved.horizontal_sqr());
    if a < 1.0e-5f32 as f64 || b < 1.0e-5f32 as f64 { return false; }
    ((intended.x * moved.x + intended.z * moved.z) / (a * b).sqrt()).acos() < 0.13962633907794952
}

/// LivingEntity.calculateFallPower.
pub fn calculate_fall_power(s: &PlayerState, distance: f64) -> f64 { distance + 1.0e-6 - s.profile.safe_fall_distance }
/// LivingEntity.calculateFallDamage.
pub fn calculate_fall_damage(s: &PlayerState, distance: f64, multiplier: f32) -> i32 {
    math::floor(calculate_fall_power(s, distance) * multiplier as f64 * s.profile.fall_damage_multiplier)
}

// Entity.checkFallDamage; LivingEntity.checkFallDamage; Block.fallOn (bed halves, dripstone doubles plus 2.5)
fn land(s: &mut PlayerState, block: &Block, dy: f64) -> Result<(), Refusal> {
    if !s.in_water && dy < 0.0 { s.fall_distance -= (dy as f32) as f64; }
    if !s.on_ground { return Ok(()); }
    if s.fall_distance > 0.0 {
        if let Some(name) = &block.unsupported_fall_on { refuse!("BlockState.fallOn: {name}"); }
        if !s.profile.mayfly {
            let fall = s.fall_distance;
            let (distance, multiplier) = if block.slime || block.inside == Inside::PowderSnow { (fall, 0.0) }
                else if block.fall_on_multiplier == 2.0 { (fall + 2.5, 2.0) }
                else { (if block.bed { fall * 0.5 } else { fall }, block.fall_on_multiplier) };
            let damage = calculate_fall_damage(s, distance, multiplier).max(0);
            if damage > 0 { hurt(s, Hit { amount: damage as f32, ..Hit::FALL }); }
        }
    }
    s.fall_distance = 0.0;
    Ok(())
}

// EntityFluidInteraction.update; Entity.updateFluidInteraction; Player.isPushedByFluid
fn immerse(s: &mut PlayerState, world: &impl World) {
    let b = s.bounding_box().deflate(0.001);
    let (lo, hi) = (block_pos(b.min), [b.max.x.ceil() as i32 - 1, b.max.y.ceil() as i32 - 1, b.max.z.ceil() as i32 - 1]);
    (s.water_height, s.lava_height, s.in_water, s.in_lava, s.eye_in_water, s.underwater) = (0.0, 0.0, false, false, false, false);
    world.observe_region(crate::footprint::Query::Fluid, lo, hi);
    if !world.has_fluid() { return; }
    // Level.hasChunksAt over the columns, asked cell by cell.
    if !(lo[2] - 1..=hi[2] + 1).all(|z| (lo[0] - 1..=hi[0] + 1).all(|x| world.loaded([x, lo[1], z]))) { return; }
    let (mut heights, mut eyes, mut currents, mut counts) = ([0.0f64; 2], [false; 2], [Vec3::default(); 2], [0; 2]);
    let center = block_pos(s.position);
    let eye_y = s.position.y + s.pose.eye_height() as f64;
    for x in lo[0]..=hi[0] { for y in lo[1]..=hi[1] { for z in lo[2]..=hi[2] {
        let f = world.fluid_block([x, y, z]).fluid;
        let a = match f.kind { FluidKind::Empty => continue, FluidKind::Water => 0, FluidKind::Lava => 1 };
        let top = y as f64 + f.surface_height(world, [x, y, z]) as f64;
        if top < b.min.y { continue; }
        if x == center[0] && z == center[2] && eye_y >= y as f64 && eye_y <= top { eyes[a] = true; }
        heights[a] = heights[a].max(top - s.bounding_box().min.y);
        if !s.flying {
            let flow = f.current(world, [x, y, z]);
            currents[a] = currents[a].plus(if heights[a] < 0.4 { flow.scale(heights[a]) } else { flow });
            counts[a] += 1;
        }
    } } }
    (s.water_height, s.lava_height, s.in_water, s.in_lava, s.eye_in_water) = (heights[0], heights[1], heights[0] > 0.0, heights[1] > 0.0, eyes[0]);
    s.underwater = s.was_eye_in_water && s.in_water;
    if s.in_water { s.fall_distance = 0.0; }
    for a in 0..2 {
        if counts[a] == 0 || currents[a].length_sqr() < 1.0e-5f32 as f64 { continue; }
        let scale = if a == 0 { 0.014 } else if world.fast_lava() { 0.007 } else { 0.0023333333333333335 };
        let mut flow = currents[a].scale(1.0 / counts[a] as f64).scale(scale);
        if s.velocity.x.abs() < 0.003 && s.velocity.z.abs() < 0.003 && flow.length_sqr().sqrt() < 0.0045000000000000005 { flow = flow.normalize().scale(0.0045000000000000005); }
        s.velocity = s.velocity.plus(flow);
    }
}

// Entity.makeStuckInBlock; Player.makeStuckInBlock (not while flying)
fn stuck(s: &mut PlayerState, multiplier: Vec3) {
    if !s.flying { s.fall_distance = 0.0; s.stuck_speed_multiplier = multiplier; }
}

/// The grounded stepOn cause on the already-moved player, before the cells crossed: slime's
/// damping and magma's request (resolved on the server's timeline).
pub fn apply_grounded_step(s: &mut PlayerState, world: &impl World) -> Option<ContactDamage> {
    world.state_footprint(crate::state_reads::StateReads::ALL, crate::state_reads::StateReads(0));
    step_on(s, world)
}
fn step_on(s: &mut PlayerState, world: &impl World) -> Option<ContactDamage> {
    if s.profile.no_physics || !s.on_ground { return None; }
    let block = world.hook_block(on_pos(s, world, 0.2));
    if block.slime && s.velocity.y.abs() < 0.1 && !s.shift_key_down {
        let f = 0.4 + s.velocity.y.abs() * 0.2;
        s.velocity = s.velocity.multiply(Vec3::new(f, 1.0, f));
    }
    match block.grounded_contact { GroundedContact::DamageUnlessCareful(request) if !s.shift_key_down => Some(request), _ => None }
}

// Player.setRemainingFireTicks: creative/invulnerable caps fire at 1 tick.
fn set_fire(s: &mut PlayerState, ticks: i32) { s.remaining_fire_ticks = if s.profile.invulnerable { ticks.min(1) } else { ticks }; }

/// InsideBlockEffectApplier.StepBasedCollector: per step a set of effect kinds (in APPLY_ORDER:
/// freeze, clear freeze, fire ignite, lava ignite, extinguish) and the hurts that run after
/// their kind; flushed when the step changes, run after the whole crossing.
#[derive(Default)]
struct Collector { step: Option<i32>, kinds: u8, fire: Vec<bool>, lava: u8, run: Vec<Op> }
#[derive(Clone, Copy)]
enum Op { Kind(u8), LavaHurt, FireHurt(bool) }
const FREEZE: u8 = 0; const CLEAR_FREEZE: u8 = 1; const FIRE_IGNITE: u8 = 2; const LAVA_IGNITE: u8 = 3; const EXTINGUISH: u8 = 4;
impl Collector {
    fn at(&mut self, step: i32) { if self.step != Some(step) { self.step = Some(step); self.flush(); } }
    fn add(&mut self, kind: u8) { self.kinds |= 1 << kind; }
    fn flush(&mut self) {
        for kind in 0..5 {
            if self.kinds & 1 << kind != 0 { self.run.push(Op::Kind(kind)); }
            if kind == FIRE_IGNITE { self.run.extend(self.fire.drain(..).map(Op::FireHurt)); }
            if kind == LAVA_IGNITE { self.run.extend((0..std::mem::take(&mut self.lava)).map(|_| Op::LavaHurt)); }
        }
        self.kinds = 0;
    }
    fn lava(&mut self) { self.add(CLEAR_FREEZE); self.add(LAVA_IGNITE); self.lava += 1; }
}

// Entity.applyEffectsFromBlocks; Entity.checkInsideBlocks; the fire grace tail
fn cross(s: &mut PlayerState, world: &impl World, start: Vec3) -> Result<Option<ContactDamage>, Refusal> {
    if s.profile.no_physics { return Ok(None); }
    let contact = apply_grounded_step(s, world);
    let fire_before = s.remaining_fire_ticks;
    let swept = s.bounding_box().expand(start.minus(s.position));
    world.observe_region(crate::footprint::Query::Hooks, block_pos(swept.min), block_pos(swept.max));
    let mut inside = None;
    if world.has_inside_effects() {
        let delta = s.position.minus(start);
        let (mut seen, mut collector, mut budget, mut from) = (SeenBlocks::new(), Collector::default(), 16, start);
        if let Some(original) = s.movement_this_tick.filter(|_| delta.length_sqr() > 0.0) {
            for axis in axis_order(original) {
                if delta.get(axis) == 0.0 { continue; }
                let to = from.plus(Vec3::default().with(axis, delta.get(axis)));
                budget -= segment(s, world, from, to, &mut seen, &mut collector, budget, delta, &mut inside)?;
                from = to;
            }
        } else {
            budget -= segment(s, world, start, s.position, &mut seen, &mut collector, 16, delta, &mut inside)?;
        }
        if budget <= 0 { segment(s, world, s.position, s.position, &mut seen, &mut collector, 1, delta, &mut inside)?; }
        collector.flush();
        let mut fire_hit = None;
        for op in collector.run {
            if s.health <= 0.0 { break; }
            match op {
                Op::Kind(FREEZE) => {
                    s.in_powder_snow = true;
                    if s.profile.game_mode != GameType::Spectator && !s.profile.leather_boots { s.ticks_frozen = (s.ticks_frozen + 1).min(140); }
                }
                Op::Kind(CLEAR_FREEZE) => s.ticks_frozen = 0,
                // BaseFireBlock.fireIgnite: a server player gains one tick, then burns at least 8 seconds.
                Op::Kind(FIRE_IGNITE) => {
                    let next = s.remaining_fire_ticks + 1;
                    let next = if next >= 0 { next.max(160) } else { next };
                    set_fire(s, next);
                    if next >= 0 { s.ticks_frozen = 0; }
                }
                Op::Kind(LAVA_IGNITE) => {
                    if s.remaining_fire_ticks < 300 { set_fire(s, 300); }
                    s.ticks_frozen = 0;
                }
                Op::Kind(_) => set_fire(s, s.remaining_fire_ticks.min(0)),
                Op::LavaHurt => hurt(s, Hit { amount: 4.0, fire: true, armor: true, fall: false, exhaustion: 0.1, ..Hit::FALL }),
                Op::FireHurt(soul) => { fire_hit.get_or_insert(ContactDamage::InFire { soul }); }
            }
        }
        inside = inside.or(fire_hit);
    }
    // Server players not on fire and not newly lit return to the 20-tick grace.
    if fire_before < 0 && s.remaining_fire_ticks <= fire_before && s.remaining_fire_ticks != -20 { set_fire(s, -20); }
    Ok(contact.or(inside))
}

// Entity.checkInsideBlocks: one axis segment of the move, cell by cell in the ray's order.
#[allow(clippy::too_many_arguments)]
fn segment(s: &mut PlayerState, world: &impl World, from: Vec3, to: Vec3, seen: &mut impl BlockSet, collector: &mut Collector,
    budget: i32, delta: Vec3, damage: &mut Option<ContactDamage>) -> Result<i32, Refusal> {
    let bounds = Aabb::player(to, s.pose.height()).deflate(1.0e-5f32 as f64);
    let large = from.minus(to).length_sqr() > (1.0 - 1.0e-5f32 as f64) * (1.0 - 1.0e-5f32 as f64);
    let (body, movement) = (Aabb::player(from, s.pose.height()), to.minus(from));
    let (mut last, mut refused) = (0, None);
    cells_along(from, to, bounds, |p, step| {
        if s.health <= 0.0 || step >= budget { return false; }
        last = step;
        let block = world.hook_block(p);
        if block.air { return true; }
        let o = Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
        let inside = block.inside_shape.as_ref().is_none_or(|shape| body.collided_along_view(movement, shape.at(o)));
        let fluid = block.fluid.kind != FluidKind::Empty && body.collided_along(movement,
            &VoxelShape::boxes(&[Aabb::new(o.x, o.y, o.z, o.x + 1.0, (p[1] as f32 + block.fluid.surface_height(world, p)) as f64, o.z + 1.0)]));
        if !(inside || fluid) || !seen.insert(p) { return true; }
        if inside {
            collector.at(step);
            let cube = Aabb { min: o, max: o.plus(Vec3::new(1.0, 1.0, 1.0)) };
            match inside_block(s, world, p, large || bounds.intersects(cube), collector, delta) {
                Ok(d) => { if let Some(d) = d { damage.get_or_insert(d); } }
                Err(rule) => { refused = Some(rule); return false; }
            }
        }
        if fluid {
            collector.at(step);
            if block.fluid.kind == FluidKind::Water { collector.add(EXTINGUISH); } else { collector.lava(); }
        }
        true
    });
    match refused { Some(rule) => Err(rule), None => Ok(last + 1) }
}

// WebBlock, PowderSnowBlock, HoneyBlock, BubbleColumnBlock, BaseFireBlock, CampfireBlock,
// SweetBerryBushBlock, CactusBlock, WitherRoseBlock .entityInside
fn inside_block(s: &mut PlayerState, world: &impl World, p: BlockPos, intersects: bool, collector: &mut Collector, delta: Vec3) -> Result<Option<ContactDamage>, Refusal> {
    let block = world.hook_block(p);
    if let Some(m) = block.stuck_multiplier { stuck(s, m); }
    Ok(match &block.inside {
        Inside::None => None,
        Inside::Cobweb => { stuck(s, if s.profile.weaving { Vec3::new(0.5, 0.25, 0.5) } else { Vec3::new(0.25, 0.05f32 as f64, 0.25) }); None }
        Inside::PowderSnow => {
            if world.hook_block(block_pos(s.position)).inside == Inside::PowderSnow { stuck(s, Vec3::new(0.9f32 as f64, 1.5, 0.9f32 as f64)); }
            collector.add(FREEZE);
            collector.add(EXTINGUISH);
            None
        }
        Inside::Honey => {
            // The slide reconstructs the pre-drag vertical speed; 0.4375 + width/2 is the edge.
            let old_y = s.velocity.y / 0.98f32 as f64 + 0.08;
            let edge = 0.4375 + (0.6f32 / 2.0f32) as f64;
            if !s.on_ground && s.position.y <= p[1] as f64 + 0.9375 - 1.0e-7 && old_y < -0.08
                && ((p[0] as f64 + 0.5 - s.position.x).abs() + 1.0e-7 > edge || (p[2] as f64 + 0.5 - s.position.z).abs() + 1.0e-7 > edge) {
                if old_y < -0.13 { let f = -0.05 / old_y; s.velocity.x *= f; s.velocity.z *= f; }
                s.velocity.y = (-0.05 - 0.08) * 0.98f32 as f64;
                s.fall_distance = 0.0;
            }
            None
        }
        Inside::BubbleColumn { drag } => {
            if intersects && !s.flying {
                let above = world.block([p[0], p[1] + 1, p[2]]);
                let vy = s.velocity.y;
                if above.shape.is_empty() && above.fluid.kind == FluidKind::Empty {
                    s.velocity.y = if *drag { (-0.9f64).max(vy - 0.03) } else { 1.8f64.min(vy + 0.1) };
                } else {
                    s.velocity.y = if *drag { (-0.3f64).max(vy - 0.03) } else { 0.7f64.min(vy + 0.06) };
                    s.fall_distance = 0.0;
                }
            }
            None
        }
        Inside::Fire { soul } => { collector.add(CLEAR_FREEZE); collector.add(FIRE_IGNITE); collector.fire.push(*soul); None }
        Inside::Campfire { soul } => Some(ContactDamage::Campfire { soul: *soul }),
        Inside::BerryBush { hurt } => {
            stuck(s, Vec3::new(0.8f32 as f64, 0.75, 0.8f32 as f64));
            (*hurt && (delta.x.abs() >= 0.003f32 as f64 || delta.z.abs() >= 0.003f32 as f64)).then_some(ContactDamage::SweetBerryBush)
        }
        Inside::Cactus => Some(ContactDamage::Cactus),
        Inside::Extinguish => { collector.add(EXTINGUISH); None }
        Inside::Lava => { collector.lava(); None }
        Inside::EndPortal => refuse!("EndPortalBlock/EndGatewayBlock.entityInside: setAsInsidePortal leaves the dimension"),
        Inside::NetherPortal => refuse!("NetherPortalBlock.entityInside: setAsInsidePortal enters portal"),
        Inside::WitherRose => {
            // The server adds WITHER (40 ticks, amplifier 0) to the book, not on peaceful, not if invulnerable.
            if world.difficulty() != 0 && !s.profile.invulnerable {
                s.add_effect(crate::EffectKind::Wither, 40, 0);
                s.effects = true;
            }
            None
        }
        Inside::Unsupported(name) => refuse!("BlockState.entityInside: {name}"),
    })
}

/// One hit through the ledger: a damage source's row (LivingEntity.hurtServer's gates, the
/// invulnerability window, Player.actuallyHurt's armour, resistance, enchantment and absorption
/// chain). Every hurt of the player runs here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit { pub amount: f32, pub fire: bool, pub armor: bool, pub resistance: bool, pub fall: bool, pub exhaustion: f32 }
impl Hit {
    /// data/minecraft/damage_type/fall.json: bypasses armour, exhaustion 0.
    pub const FALL: Hit = Hit { amount: 0.0, fire: false, armor: false, resistance: true, fall: true, exhaustion: 0.0 };
    pub fn contact(c: ContactDamage) -> Hit {
        Hit { amount: c.raw_amount(), fire: c.is_fire(), armor: !c.bypasses_armor(), resistance: true, fall: c.is_fall(), exhaustion: c.exhaustion() }
    }
}
/// The hurt itself; `armor` and `resistance` say whether those steps of the chain apply.
pub fn hurt(s: &mut PlayerState, hit: Hit) {
    if s.profile.invulnerable || s.health <= 0.0 || hit.fire && s.profile.fire_resistance { return; }
    let raw = if hit.amount < 0.0 { 0.0 } else if !hit.amount.is_finite() { f32::MAX } else { hit.amount };
    if let Some(debit) = window(&mut s.hurt_cooldown, raw) { mitigate(s, debit, hit); }
}
/// LivingEntity.hurtServer's invulnerability window: the whole hit at 10 ticks or fewer, above
/// that only the excess over the last raw hit.
pub(crate) fn window(cooldown: &mut HurtCooldown, amount: f32) -> Option<f32> {
    let debit = if cooldown.remaining_ticks > 10 {
        if amount <= cooldown.last_raw_hit { return None; }
        amount - cooldown.last_raw_hit
    } else { cooldown.remaining_ticks = 20; amount };
    cooldown.last_raw_hit = amount;
    Some(debit)
}
/// Player.actuallyHurt with getDamageAfterArmorAbsorb and getDamageAfterMagicAbsorb.
pub(crate) fn mitigate(s: &mut PlayerState, mut amount: f32, hit: Hit) {
    let p = &s.profile;
    if hit.armor {
        let (armor, f) = (math::floor(p.armor as f64) as f32, 2.0f32 + p.armor_toughness / 4.0f32);
        amount *= 1.0f32 - math::clamp_f32(armor - amount / f, armor * 0.2f32, 20.0f32) / 25.0f32;
    }
    if hit.resistance && let Some(amp) = p.resistance { amount = (amount * ((25 - (amp as i32 + 1) * 5) as f32) / 25.0f32).max(0.0f32); }
    if amount <= 0.0f32 { amount = 0.0f32; }
    else {
        let epf = p.protection_epf + if hit.fall { p.feather_falling_epf } else { 0.0f32 };
        if epf > 0.0f32 { amount *= 1.0f32 - math::clamp_f32(epf, 0.0f32, 20.0f32) / 25.0f32; }
    }
    let to_health = (amount - s.absorption).max(0.0f32);
    s.absorption -= amount - to_health;
    if to_health != 0.0f32 {
        if !p.invulnerable { s.exhaustion = (s.exhaustion + hit.exhaustion).min(40.0f32); }
        s.health = (s.health - to_health).max(0.0f32);
    }
}

// Player.causeFoodExhaustion; FoodData.addExhaustion
fn exhaust(s: &mut PlayerState, amount: f32) { if !s.profile.invulnerable { s.exhaustion = (s.exhaustion + amount).min(40.0f32); } }

// ServerPlayer.checkMovementStatistics: centimetres walked, swum or sprinted, rounded.
fn walked(s: &mut PlayerState, delta: Vec3) {
    let (distance, rate) = if s.swimming || s.eye_in_water { (delta.length_sqr().sqrt(), 0.01f32) }
        else if s.in_water { (delta.horizontal_sqr().sqrt(), 0.01f32) }
        else if !s.on_climbable && s.on_ground && s.sprinting { (delta.horizontal_sqr().sqrt(), 0.1f32) } else { return };
    let cm = math::round(distance as f32 * 100.0);
    if cm > 0 { exhaust(s, rate * cm as f32 * 0.01f32); }
}

// FoodData.tick: exhaustion over 4 costs saturation, then food; one timer whose row the food state picks.
fn food(s: &mut PlayerState, world: &impl World) {
    if s.exhaustion > 4.0 {
        s.exhaustion -= 4.0;
        if s.saturation > 0.0 { s.saturation = (s.saturation - 1.0).max(0.0); }
        else if world.difficulty() != 0 { s.food_level = (s.food_level - 1).max(0); }
    }
    let (regen, hurt) = (world.natural_regeneration(), s.health > 0.0 && s.health < 20.0);
    let row = if regen && s.saturation > 0.0 && hurt && s.food_level >= 20 { (clock::FOOD_SATURATED, 0) }
        else if regen && s.food_level >= 18 && hurt { (clock::FOOD_FED, 1) }
        else if s.food_level <= 0 { (clock::FOOD_FED, 2) } else { s.food_tick_timer = 0; return };
    let mut timer = s.food_tick_timer as i32;
    let fired = row.0.tick(&mut timer);
    s.food_tick_timer = timer as u32;
    if !fired { return; }
    let health = s.health;
    match row.1 {
        0 => { let amount = s.saturation.min(6.0); s.health = (health + amount / 6.0).min(20.0); s.exhaustion = (s.exhaustion + amount).min(40.0); }
        1 => { s.health = (health + 1.0).min(20.0); s.exhaustion = (s.exhaustion + 6.0).min(40.0); }
        _ => if health > 10.0 || world.difficulty() == 3 || health > 1.0 && world.difficulty() == 2 { s.health = (health - 1.0).max(0.0); },
    }
}
