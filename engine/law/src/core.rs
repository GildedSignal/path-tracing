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
/// contact damage requested on the server's resource timeline, and whether the body jumped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effect { pub collision: Option<(Vec3, Vec3)>, pub contact: Option<ContactDamage>, pub jumped: bool }

/// The same tick as `tick`; an unported rule is returned instead of panicking.
pub fn try_tick(s: &mut PlayerState, world: &impl World, input: &Input) -> Result<(), Refusal> { step(s, world, input).map(|_| ()) }
fn unported(s: &mut PlayerState, world: &impl World, input: &Input) -> Effect {
    step(s, world, input).unwrap_or_else(|Refusal(rule)| panic!("not implemented: {rule}"))
}
/// Player.tick; Entity.baseTick; LocalPlayer.aiStep; LivingEntity.aiStep.
pub fn tick(s: &mut PlayerState, world: &impl World, input: &Input) { unported(s, world, input); }
/// The tick, returning the request and result at Entity.collide.
pub fn tick_with_collision(s: &mut PlayerState, world: &impl World, input: &Input) -> Option<(Vec3, Vec3)> { unported(s, world, input).collision }
/// Same collision readout, returning an unported-rule refusal instead of panicking.
pub fn try_tick_with_collision(s: &mut PlayerState, world: &impl World, input: &Input) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    step(s, world, input).map(|e| e.collision)
}
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
    swim(s, world);
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
        let body = s.bounding_box();
        let context = CollisionContext::of(s);
        let mut safe = [[0i32; 3]; 3];
        let mut n_safe = 0usize;
        for (x, z) in [(-d, d), (-d, -d), (d, -d), (d, d)] {
            push_out_cached(s, world, body, context, &mut safe, &mut n_safe, s.position.x + x, s.position.z + z);
        }
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
    let mut climb_cache = (block_pos(s.position), climbable(s, world));
    s.on_climbable = climb_cache.1;
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
    let jumped = s.no_jump_delay == 10;
    if s.health <= 0.0 { movement = Vec3::default(); }
    if s.fall_flying {
        s.fall_distance = crate::elytra::check_fall_distance_accumulation(s.velocity.y, s.fall_distance);
        if s.on_ground || s.profile.levitation.is_some() { s.fall_flying = false; }
    }
    if s.profile.slow_falling || s.profile.levitation.is_some() { s.fall_distance = 0.0; }
    // Cause: the move (the body generator, then the sweep).
    let (collision, land_block) = travel(s, world, movement, &mut climb_cache)?;
    world.observe_sweep(start_box, s.bounding_box());
    // Cause: the cells the move crossed.
    let contact = burn.or(cross(s, world, start, collision.map(|(wanted, _)| wanted), land_block)?);
    let before_pose = s.bounding_box();
    s.position.x = s.position.x.clamp(-29999999.0, 29999999.0);
    s.position.z = s.position.z.clamp(-29999999.0, 29999999.0);
    if s.on_ground && s.flying && s.profile.game_mode != GameType::Spectator { s.flying = false; }
    pose(s, world);
    world.observe_sweep(before_pose, s.bounding_box());
    s.on_climbable = climbable_cached(s, world, &mut climb_cache);
    // Cause: the distance walked, then the food clock.
    walked(s, s.position.minus(start));
    food(s, world);
    s.t += 1;
    Ok(Effect { collision, contact, jumped })
}

// Player.updateSwimming: the swim flag machine.
fn swim(s: &mut PlayerState, world: &impl World) {
    s.swimming = !s.flying && if s.swimming { s.sprinting && s.in_water } else {
        s.sprinting && s.underwater && world.fluid_block(block_pos(s.position)).fluid.kind == FluidKind::Water
    };
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
#[cfg(test)]
fn push_out(s: &mut PlayerState, world: &impl World, x: f64, z: f64) {
    let mut safe = [[0i32; 3]; 3];
    let mut n_safe = 0;
    push_out_cached(s, world, s.bounding_box(), CollisionContext::of(s), &mut safe, &mut n_safe, x, z);
}
#[inline]
fn push_out_cached(
    s: &mut PlayerState,
    world: &impl World,
    body: Aabb,
    context: CollisionContext,
    safe: &mut [[i32; 3]; 3],
    n_safe: &mut usize,
    x: f64,
    z: f64,
) {
    let p = block_pos(Vec3::new(x, s.position.y, z));
    if safe[..*n_safe].contains(&p) { return; }
    let suffocates = |p: BlockPos| world.collides_with_suffocating_block(
        Aabb::new(p[0] as f64, body.min.y, p[2] as f64, p[0] as f64 + 1.0, body.max.y, p[2] as f64 + 1.0).deflate(1.0e-7), context);
    if !suffocates(p) {
        if *n_safe < safe.len() {
            safe[*n_safe] = p;
            *n_safe += 1;
        }
        return;
    }
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
#[inline]
fn on_pos_block<'w>(s: &PlayerState, world: &'w impl World, offset: f32) -> (BlockPos, &'w Block) {
    let Some(p) = s.main_supporting_block else {
        let q = block_pos(Vec3::new(s.position.x, s.position.y - offset as f64, s.position.z));
        return (q, world.hook_block(q));
    };
    if offset <= 1.0e-5f32 { return (p, world.hook_block(p)); }
    let b = world.hook_block(p);
    if offset as f64 <= 0.5 && b.fence || b.wall_or_gate {
        (p, b)
    } else {
        let q = [p[0], math::floor(s.position.y - offset as f64), p[2]];
        (q, if q == p { b } else { world.hook_block(q) })
    }
}

// Entity.getBlockJumpFactor (both cells read); Entity.getBlockSpeedFactor (water stops at the feet)
fn jump_factor(s: &PlayerState, world: &impl World) -> f32 {
    let (here, below) = (world.hook_block(block_pos(s.position)).jump_factor, on_pos_block(s, world, 0.500001).1.jump_factor);
    if here as f64 == 1.0 { below } else { here }
}
fn speed_factor(s: &PlayerState, world: &impl World) -> f32 {
    let here = world.hook_block(block_pos(s.position));
    if !here.water_block && here.speed_factor as f64 == 1.0 { on_pos_block(s, world, 0.500001).1.speed_factor } else { here.speed_factor }
}

// LivingEntity.jumpFromGround; ServerPlayer.jumpFromGround
fn jump(s: &mut PlayerState, world: &impl World) {
    let power = s.profile.jump_strength as f32 * 1.0f32 * jump_factor(s, world) + s.jump_boost_power();
    if power > 1.0e-5f32 {
        s.velocity.y = (power as f64).max(s.velocity.y);
        if s.sprinting {
            let yaw = (s.input.yaw * 0.017453292f32) as f64;
            s.velocity = s.velocity.plus(Vec3::new((-math::sin(yaw)) as f64 * 0.2, 0.0, math::cos(yaw) as f64 * 0.2));
        }
    }
    exhaust(s, jump_cost(s));
}
pub(crate) fn jump_cost(s: &PlayerState) -> f32 { if s.sprinting { 0.2 } else { 0.05 } }

// LivingEntity.onClimbable; LivingEntity.trapdoorUsableAsLadder
fn climbable(s: &PlayerState, world: &impl World) -> bool {
    let p = block_pos(s.position);
    let b = world.hook_block(p);
    b.climbable || b.open_trapdoor_facing.is_some_and(|f| world.hook_block([p[0], p[1] - 1, p[2]]).ladder_facing == Some(f))
}
#[inline]
fn climbable_cached(s: &PlayerState, world: &impl World, cache: &mut (BlockPos, bool)) -> bool {
    let p = block_pos(s.position);
    if p == cache.0 { return cache.1; }
    let b = world.hook_block(p);
    let c = b.climbable || b.open_trapdoor_facing.is_some_and(|f| world.hook_block([p[0], p[1] - 1, p[2]]).ladder_facing == Some(f));
    *cache = (p, c);
    c
}

// LivingEntity.getEffectiveGravity
fn gravity(s: &PlayerState, vy: f64) -> f64 {
    if vy <= 0.0 && s.profile.slow_falling { s.profile.gravity.min(0.01) } else { s.profile.gravity }
}

// Player.travel; LivingEntity.travel
fn travel<'w>(
    s: &mut PlayerState,
    world: &'w impl World,
    movement: Vec3,
    climb_cache: &mut (BlockPos, bool),
) -> Result<(Option<(Vec3, Vec3)>, Option<&'w Block>), Refusal> {
    if s.swimming {
        let look_y = -math::sin((s.input.pitch * 0.017453292f32) as f64) as f64;
        let factor = if look_y < -0.2 { 0.085 } else { 0.06 };
        if look_y <= 0.0 || s.input.jump || world.fluid_block(block_pos(Vec3::new(s.position.x, s.position.y + 1.0 - 0.1, s.position.z))).fluid.kind != FluidKind::Empty {
            s.velocity.y += (look_y - s.velocity.y) * factor;
        }
    }
    let flying_vy = s.flying.then_some(s.velocity.y);
    let result = if !s.flying && (s.in_water || s.in_lava) { travel_fluid(s, world, movement, climb_cache)? }
        else if s.fall_flying && !climbable_cached(s, world, climb_cache) { glide(s, world)? }
        else {
            let result = travel_air(s, world, movement, climb_cache)?;
            if s.fall_flying { s.fall_flying = false; }
            result
        };
    if let Some(vy) = flying_vy { s.velocity.y = vy * 0.6; }
    Ok(result)
}

// LivingEntity.travelFallFlying: the elytra kernel, then the wall's hit.
fn glide<'w>(s: &mut PlayerState, world: &'w impl World) -> Result<(Option<(Vec3, Vec3)>, Option<&'w Block>), Refusal> {
    let pre_h = s.velocity.horizontal_sqr().sqrt();
    let look = math::calculate_view_vector(s.input.yaw, s.input.pitch);
    s.velocity = crate::elytra::update_fall_flying_movement(s.velocity, look, s.input.pitch, gravity(s, s.velocity.y));
    let result = move_body(s, world, s.velocity)?;
    if let Some(amount) = crate::elytra::fly_into_wall_damage(s.horizontal_collision, pre_h, s.velocity.horizontal_sqr().sqrt()) {
        hurt(s, Hit { amount, fall: false, ..Hit::FALL });
    }
    Ok(result)
}

/// Player.getFlyingSpeed: the acceleration off the ground.
pub fn air_speed(s: &PlayerState) -> f32 {
    if s.flying && !s.profile.riding { s.profile.flying_speed * if s.sprinting { 2.0f32 } else { 1.0 } } else if s.sprinting { 0.025999999f32 } else { 0.02f32 }
}

// LivingEntity.travelInAir; handleRelativeFrictionAndCalculateMovement; handleOnClimbable
fn travel_air<'w>(
    s: &mut PlayerState,
    world: &'w impl World,
    movement: Vec3,
    climb_cache: &mut (BlockPos, bool),
) -> Result<(Option<(Vec3, Vec3)>, Option<&'w Block>), Refusal> {
    let f = if s.on_ground {
        math::modified_friction(on_pos_block(s, world, 0.500001).1.friction, s.profile.friction_modifier)
    } else {
        1.0
    };
    let acceleration = if !s.on_ground { air_speed(s) } else if f as f64 > 0.6 { s.speed * (0.21600002f32 / (f * f * f)) } else { s.speed };
    s.velocity = s.velocity.plus(input_vector(movement, acceleration, s.input.yaw));
    if climbable_cached(s, world, climb_cache) {
        s.fall_distance = 0.0;
        let c = 0.15f32 as f64;
        let v = &mut s.velocity;
        (v.x, v.z, v.y) = (v.x.clamp(-c, c), v.z.clamp(-c, c), v.y.max(-c));
        if v.y < 0.0 && !world.hook_block(block_pos(s.position)).scaffolding && s.shift_key_down { s.velocity.y = 0.0; }
    }
    let result = move_body(s, world, s.velocity)?;
    let mut v = s.velocity;
    let pre_bump_vy = v.y;
    if (s.horizontal_collision || s.input.jump) && (climbable_cached(s, world, climb_cache) || s.was_in_powder_snow && s.profile.leather_boots) { v.y = 0.2; }
    // The unloaded-chunk fallback of travelInAir is client-only: the server goes to gravity.
    let vy = match s.profile.levitation {
        Some(amp) => v.y + (0.05 * ((amp as i32 + 1) as f64) - v.y) * 0.2,
        None => v.y - gravity(s, pre_bump_vy),
    };
    let m = s.profile.air_drag_modifier;
    let row = AffineBody { air_drag: math::modified_friction(LIVING.air_drag, m), drag_y: math::modified_friction(LIVING.drag_y, m), ..LIVING };
    world.observe_air_travel(movement.z * acceleration as f64, f * row.air_drag);
    s.velocity = row.friction_drag(v, vy, f);
    Ok(result)
}

// LivingEntity.travelInFluid; travelInWater; travelInLava; getFluidFallingAdjustedMovement
fn travel_fluid<'w>(
    s: &mut PlayerState,
    world: &'w impl World,
    movement: Vec3,
    climb_cache: &mut (BlockPos, bool),
) -> Result<(Option<(Vec3, Vec3)>, Option<&'w Block>), Refusal> {
    let falling = s.velocity.y <= 0.0;
    let g = gravity(s, s.velocity.y);
    let old_y = s.position.y;
    let fall_adjust = |s: &mut PlayerState| if g != 0.0 && !s.sprinting {
        let y = s.velocity.y - g / 16.0;
        s.velocity.y = if falling && (s.velocity.y - 0.005).abs() >= 0.003 && y.abs() < 0.003 { -0.003 } else { y };
    };
    let result = if s.in_water {
        let (mut drag, mut acceleration) = (if s.sprinting { 0.9f32 } else { 0.8f32 }, 0.02f32);
        let efficiency = s.profile.water_movement_efficiency * if s.on_ground { 1.0 } else { 0.5 };
        if efficiency > 0.0 { drag += (0.54600006f32 - drag) * efficiency; acceleration += (s.speed - acceleration) * efficiency; }
        if s.profile.dolphins_grace { drag = 0.96f32; }
        s.velocity = s.velocity.plus(input_vector(movement, acceleration, s.input.yaw));
        let result = move_body(s, world, s.velocity)?;
        if s.horizontal_collision && climbable_cached(s, world, climb_cache) { s.velocity.y = 0.2; }
        s.velocity = s.velocity.multiply(Vec3::new(drag as f64, 0.8f32 as f64, drag as f64));
        fall_adjust(s);
        result
    } else {
        s.velocity = s.velocity.plus(input_vector(movement, 0.02, s.input.yaw));
        let result = move_body(s, world, s.velocity)?;
        if s.lava_height <= 0.4 { s.velocity = s.velocity.multiply(Vec3::new(0.5, 0.8f32 as f64, 0.5)); fall_adjust(s); }
        else { s.velocity = s.velocity.scale(0.5); }
        if g != 0.0 { s.velocity.y += -g / 4.0; }
        result
    };
    let v = s.velocity;
    let lifted = s.bounding_box().moved(Vec3::new(v.x, v.y + 0.6f32 as f64 - s.position.y + old_y, v.z));
    if s.horizontal_collision && world.no_collision(lifted, CollisionContext::of(s)) && !any_fluid(world, lifted) { s.velocity.y = 0.3f32 as f64; }
    Ok(result)
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
pub(crate) fn support(s: &mut PlayerState, world: &impl World, moved: Vec3) {
    if !s.on_ground { (s.on_ground_no_blocks, s.main_supporting_block) = (false, None); return; }
    let b = s.bounding_box();
    let context = CollisionContext::of(s);
    let foot = Aabb::new(b.min.x, b.min.y - 1.0e-6, b.min.z, b.max.x, b.min.y, b.max.z);
    let find = |bounds: Aabb| {
        let (lo, hi) = world.collision_cell_bounds(bounds, block_pos(bounds.min).map(|v| v - 1), block_pos(bounds.max).map(|v| v + 1))?;
        let mut best: Option<(f64, BlockPos)> = None;
        for x in lo[0]..=hi[0] { for y in lo[1]..=hi[1] { for z in lo[2]..=hi[2] {
            let o = Vec3::new(x as f64, y as f64, z as f64);
            if !world.collision_shape([x, y, z], context).at(o).intersects(bounds) { continue; }
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
fn move_body<'w>(s: &mut PlayerState, world: &'w impl World, mut wanted: Vec3) -> Result<(Option<(Vec3, Vec3)>, Option<&'w Block>), Refusal> {
    if s.profile.no_physics { s.position = s.position.plus(wanted); return Ok((None, None)); }
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
    let (_, block) = on_pos_block(s, world, 0.2);
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
    Ok((Some((wanted, moved)), Some(block)))
}

// LocalPlayer.isHorizontalCollisionMinor
fn minor_collision(s: &PlayerState, moved: Vec3) -> bool {
    if !s.horizontal_collision { return false; }
    let input = steer(&s.input, s.crouching || s.pose == Pose::Swimming && !s.in_water, s);
    let yaw = (s.input.yaw * 0.017453292f32) as f64;
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
    s.fall_distance = fallen(s, s.fall_distance, dy);
    if !s.on_ground { return Ok(()); }
    if s.fall_distance > 0.0 {
        if let Some(name) = &block.unsupported_fall_on { refuse!("BlockState.fallOn: {name}"); }
        if !s.profile.mayfly {
            let (distance, multiplier) = fall_on(block, s.fall_distance);
            let damage = calculate_fall_damage(s, distance, multiplier).max(0);
            if damage > 0 { hurt(s, Hit { amount: damage as f32, ..Hit::FALL }); }
        }
    }
    s.fall_distance = 0.0;
    Ok(())
}
/// The fall after a move of `dy`.
pub(crate) fn fallen(s: &PlayerState, fall: f64, dy: f64) -> f64 { if !s.in_water && dy < 0.0 { fall - (dy as f32) as f64 } else { fall } }
/// Block.fallOn's row: the distance and multiplier a landing on `block` is priced at (slime and
/// powder snow free, a bed halves, a stalagmite's tip adds 2.5 and doubles).
pub fn fall_on(block: &Block, fall: f64) -> (f64, f32) {
    if block.slime || block.inside == Inside::PowderSnow { (fall, 0.0) }
    else if block.fall_on_multiplier == 2.0 { (fall + 2.5, 2.0) }
    else { (if block.bed { fall * 0.5 } else { fall }, block.fall_on_multiplier) }
}

// EntityFluidInteraction.update; Entity.updateFluidInteraction; Player.isPushedByFluid
fn immerse(s: &mut PlayerState, world: &impl World) {
    let b = s.bounding_box().deflate(0.001);
    let (lo, hi) = (block_pos(b.min), [b.max.x.ceil() as i32 - 1, b.max.y.ceil() as i32 - 1, b.max.z.ceil() as i32 - 1]);
    (s.water_height, s.lava_height, s.in_water, s.in_lava, s.eye_in_water, s.underwater) = (0.0, 0.0, false, false, false, false);
    world.observe_region(crate::footprint::Query::Fluid, lo, hi);
    if !world.has_fluid_in(lo, hi) { return; }
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
    if s.profile.no_physics || !s.on_ground { return None; }
    step_on_block(s, on_pos_block(s, world, 0.2).1)
}
#[inline]
fn step_on_block(s: &mut PlayerState, block: &Block) -> Option<ContactDamage> {
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
    /// StepBasedCollector.applyAndClear on a server player, while alive; the first fire hurt is a request.
    fn apply(mut self, s: &mut PlayerState) -> Option<ContactDamage> {
        self.flush();
        let mut fire_hit = None;
        for op in self.run {
            if s.health <= 0.0 { break; }
            match op {
                Op::Kind(FREEZE) => {
                    s.in_powder_snow = true;
                    if s.profile.game_mode != GameType::Spectator && !s.profile.leather_boots { s.ticks_frozen = (s.ticks_frozen + 1).min(140); }
                }
                Op::Kind(CLEAR_FREEZE) => s.ticks_frozen = 0,
                // BaseFireBlock.fireIgnite: a server player gains one tick, then burns at least 8 seconds.
                Op::Kind(FIRE_IGNITE) => {
                    let next = crate::player::base_fire_ignite_ticks(s.remaining_fire_ticks, false, true);
                    set_fire(s, next);
                    if next >= 0 { s.ticks_frozen = 0; }
                }
                // Entity.lavaIgnite: 15 seconds, max-only.
                Op::Kind(LAVA_IGNITE) => {
                    if s.remaining_fire_ticks < 300 { set_fire(s, 300); }
                    s.ticks_frozen = 0;
                }
                Op::Kind(_) => set_fire(s, crate::player::clear_fire_ticks(s.remaining_fire_ticks)),
                Op::LavaHurt => { hurt(s, Hit::LAVA); }
                Op::FireHurt(soul) => { fire_hit.get_or_insert(ContactDamage::InFire { soul }); }
            }
        }
        fire_hit
    }
}

// Entity.applyEffectsFromBlocks; Entity.checkInsideBlocks; the fire grace tail
fn cross(
    s: &mut PlayerState,
    world: &impl World,
    start: Vec3,
    wanted: Option<Vec3>,
    land_block: Option<&Block>,
) -> Result<Option<ContactDamage>, Refusal> {
    if s.profile.no_physics { return Ok(None); }
    let contact = if s.on_ground {
        world.state_footprint(crate::state_reads::StateReads::ALL, crate::state_reads::StateReads(0));
        let block = land_block.unwrap_or_else(|| on_pos_block(s, world, 0.2).1);
        step_on_block(s, block)
    } else {
        None
    };
    let fire_before = s.remaining_fire_ticks;
    let swept = s.bounding_box().expand(start.minus(s.position));
    world.observe_region(crate::footprint::Query::Hooks, block_pos(swept.min), block_pos(swept.max));
    let mut inside = None;
    if world.has_inside_effects_in(block_pos(swept.min), block_pos(swept.max)) {
        let delta = s.position.minus(start);
        let (mut seen, mut collector, mut budget, mut from) = (SeenBlocks::new(), Collector::default(), 16, start);
        if let Some(original) = wanted.filter(|_| delta.length_sqr() > 0.0) {
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
        inside = inside.or(collector.apply(s));
    }
    // Server players not on fire and not newly lit return to the 20-tick grace.
    let after = s.remaining_fire_ticks;
    if fire_before < 0 && crate::player::apply_effects_fire_tail(after, fire_before, true, false, true) != after { set_fire(s, -20); }
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
            crate::basis::body::bubble_column_contact(s, world, p, drag, intersects);
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
pub struct Hit { pub amount: f32, pub fire: bool, pub armor: bool, pub effects: bool, pub fall: bool, pub exhaustion: f32 }
impl Hit {
    /// data/minecraft/damage_type/fall.json: bypasses armour, exhaustion 0.
    pub const FALL: Hit = Hit { amount: 0.0, fire: false, armor: false, effects: true, fall: true, exhaustion: 0.0 };
    /// Entity.lavaHurt: 4, fire, exhaustion 0.1, never scaled by difficulty.
    pub const LAVA: Hit = Hit { amount: 4.0, fire: true, armor: true, effects: true, fall: false, exhaustion: 0.1 };
    /// A status effect's tick (wither, poison): no armour, resistance or enchantment term.
    pub const MAGIC: Hit = Hit { amount: 0.0, fire: false, armor: false, effects: false, fall: false, exhaustion: 0.0 };
    pub fn contact(c: ContactDamage) -> Hit {
        Hit { amount: c.raw_amount(), fire: c.is_fire(), armor: !c.bypasses_armor(), effects: true, fall: c.is_fall(), exhaustion: c.exhaustion() }
    }
}
/// The hurt itself; false when a gate (invulnerable, dead, fire resistance, the window) stops it.
pub fn hurt(s: &mut PlayerState, hit: Hit) -> bool {
    if s.profile.invulnerable || s.health <= 0.0 || hit.fire && s.profile.fire_resistance { return false; }
    let raw = if hit.amount < 0.0 { 0.0 } else if !hit.amount.is_finite() { f32::MAX } else { hit.amount };
    window(&mut s.hurt_cooldown, raw).map(|debit| mitigate(s, debit, hit)).is_some()
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
    if hit.effects {
        if let Some(amp) = p.resistance { amount = (amount * ((25 - (amp as i32 + 1) * 5) as f32) / 25.0f32).max(0.0f32); }
        if amount <= 0.0f32 { amount = 0.0f32; }
        else {
            let epf = p.protection_epf + if hit.fall { p.feather_falling_epf } else { 0.0f32 };
            if epf > 0.0f32 { amount *= 1.0f32 - math::clamp_f32(epf, 0.0f32, 20.0f32) / 25.0f32; }
        }
    }
    let to_health = (amount - s.absorption).max(0.0f32);
    s.absorption -= amount - to_health;
    if to_health != 0.0f32 {
        if !p.invulnerable { s.exhaustion = (s.exhaustion + hit.exhaustion).min(40.0f32); }
        s.health = (s.health - to_health).max(0.0f32);
    }
}

// Player.causeFoodExhaustion; FoodData.addExhaustion
fn exhaust(s: &mut PlayerState, amount: f32) { s.exhaustion = exhausted(s, s.exhaustion, amount); }
pub(crate) fn exhausted(s: &PlayerState, e: f32, amount: f32) -> f32 { if s.profile.invulnerable { e } else { (e + amount).min(40.0f32) } }

// ServerPlayer.checkMovementStatistics: centimetres walked, swum or sprinted, rounded.
fn walked(s: &mut PlayerState, delta: Vec3) { if let Some(cost) = walk_cost(s, delta) { exhaust(s, cost); } }
pub(crate) fn walk_cost(s: &PlayerState, delta: Vec3) -> Option<f32> {
    let (distance, rate) = if s.swimming || s.eye_in_water { (delta.length_sqr().sqrt(), 0.01f32) }
        else if s.in_water { (delta.horizontal_sqr().sqrt(), 0.01f32) }
        else if !s.on_climbable && s.on_ground && s.sprinting { (delta.horizontal_sqr().sqrt(), 0.1f32) } else { return None };
    let cm = math::round(distance as f32 * 100.0);
    (cm > 0).then(|| rate * cm as f32 * 0.01f32)
}

// FoodData.tick: exhaustion over 4 costs saturation, then food; one timer whose row the food state picks.
fn food(s: &mut PlayerState, world: &impl World) {
    (s.exhaustion, s.saturation, s.food_level) = drain(world, s.exhaustion, s.saturation, s.food_level);
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
/// FoodData.tick's first cause: exhaustion over 4 costs saturation, then food.
pub(crate) fn drain(world: &impl World, e: f32, saturation: f32, food: i32) -> (f32, f32, i32) {
    if e <= 4.0 { (e, saturation, food) }
    else if saturation > 0.0 { (e - 4.0, (saturation - 1.0).max(0.0), food) }
    else { (e - 4.0, saturation, if world.difficulty() != 0 { (food - 1).max(0) } else { food }) }
}

/// Ordered causes at the authoritative server resource boundaries, between the ticks.
#[derive(Clone, Debug)]
pub enum ServerResourceEvent {
    /// A resolved LivingEntity.knockback: `x`, `z` are the direction after the server's jitter
    /// loop (x² + z² ≥ 1e-5f), since the law draws no randomness; `resistance` is the attribute.
    Knockback { strength: f64, resistance: f64, x: f64, z: f64 },
    Exhaustion(f32),
    /// Observed health debit; any observed exhaustion comes separately.
    Damage(f32),
    /// ServerPlayer.tick, once before the connection's ServerPlayer.doTick.
    HurtCooldownTick,
    /// LivingEntity.onBelowWorld below minY - 64: the void's 4.0 after the cooldown tick,
    /// bypassing armour, resistance and invulnerability (creative included); exhaustion 0.
    FellOutOfWorld,
    /// The contact request and its exhaustion predicted in place of observed events (normal
    /// survival, stock damage data, no gear, effects or absorption; anchor the cooldown first).
    UnmodifiedContactDamage(ContactDamage),
    Regeneration(i32),
    FoodTick,
}

pub fn apply_server_resource_event(s: &mut PlayerState, world: &impl World, event: ServerResourceEvent) {
    world.state_footprint(crate::state_reads::StateReads::ALL, crate::state_reads::StateReads(0));
    match event {
        // LivingEntity.knockback; a NaN strength does not return (dcmpg; ifgt), as in vanilla.
        ServerResourceEvent::Knockback { strength, resistance, x, z } => {
            let strength = strength * (1.0 - resistance);
            if strength <= 0.0 { return; }
            debug_assert!(!(x * x + z * z < 1.0e-5f32 as f64), "knockback direction must include server jitter");
            let (impulse, v) = (Vec3::new(x, 0.0, z).normalize().scale(strength), s.velocity);
            let y = if !s.on_ground { v.y } else { let y = v.y / 2.0 + strength; if y.is_nan() { y } else { 0.4f64.min(y) } };
            s.velocity = Vec3::new(v.x / 2.0 - impulse.x, y, v.z / 2.0 - impulse.z);
        }
        ServerResourceEvent::Exhaustion(amount) => s.exhaustion = (s.exhaustion + amount).min(40.0f32),
        ServerResourceEvent::Damage(amount) => s.health = (s.health - amount).max(0.0),
        ServerResourceEvent::HurtCooldownTick => COOLDOWN.cool(&mut s.hurt_cooldown.remaining_ticks),
        ServerResourceEvent::FellOutOfWorld => {
            if s.health > 0.0 && let Some(amount) = window(&mut s.hurt_cooldown, crate::player::death::FELL_OUT_OF_WORLD) {
                s.health = (s.health - amount).max(0.0);
            }
        }
        ServerResourceEvent::UnmodifiedContactDamage(request) => {
            assert!(world.difficulty() == 2 && !s.profile.unmodeled_effects && !(s.effects && !modeled_effects(s)),
                "contact damage prediction requires unmodified normal survival");
            hurt(s, Hit::contact(request));
        }
        ServerResourceEvent::Regeneration(tick) => regenerate(s, world, tick),
        ServerResourceEvent::FoodTick => food(s, world),
    }
}

/// Resources for a living, unmounted server player with default max health, at the server's
/// Player.aiStep tickCount, with its exhaustion and damage already applied.
pub fn tick_server_resources(s: &mut PlayerState, world: &impl World, server_player_tick: i32) {
    world.state_footprint(crate::state_reads::StateReads::ALL, crate::state_reads::StateReads(0));
    regenerate(s, world, server_player_tick);
    food(s, world);
}

// ServerPlayer.tickRegeneration (peaceful with natural regeneration), before FoodData.tick.
fn regenerate(s: &mut PlayerState, world: &impl World, tick: i32) {
    if world.difficulty() != 0 || !world.natural_regeneration() { return; }
    if tick % 20 == 0 {
        if s.health > 0.0 && s.health < 20.0 { s.health = (s.health + 1.0).min(20.0); }
        if s.saturation < 20.0 { s.saturation += 1.0; }
    }
    if tick % 10 == 0 && s.food_level < 20 { s.food_level += 1; }
}

/// The server's freeze clock after travel (LivingEntity.aiStep:697-768, removeFrost/tryAddFrost,
/// getPercentFrozen, isFullyFrozen): the counter, the speed modifier and the damage request.
pub fn server_freeze_step(ticks_frozen: i32, in_powder_snow: bool, can_freeze: bool, feet_non_air: bool, tick_count: i32) -> (i32, Option<f64>, Option<f32>) {
    let frozen = if in_powder_snow && can_freeze { ticks_frozen } else { (ticks_frozen - 2).max(0) };
    let frost = (feet_non_air && frozen > 0).then(|| (-0.05f32 * (frozen.min(140) as f32 / 140.0f32)) as f64);
    (frozen, frost, (tick_count % 40 == 0 && frozen >= 140 && can_freeze).then_some(1.0f32))
}

#[cfg(test)]
mod witnesses {
    use super::*;
    use crate::{BoxWorld, Fluid};

    #[test]
    fn swim_machine_oracle_cases() {
        // Eight direct 26.2 oracle cases (flying, swimming, sprinting, underwater, in water, water at the feet → swimming).
        // The two riding rows are refused by the tick before the swim machine runs (travelRidden).
        for (flying, swimming, sprinting, underwater, in_water, water, expected) in [
            (false, false, true, true, false, true, true), (false, false, true, true, false, false, false),
            (false, true, true, false, true, true, true), (false, true, false, false, true, true, false),
            (true, false, true, true, false, true, false), (true, true, true, false, true, true, false),
        ] {
            let mut s = PlayerState { flying, swimming, sprinting, underwater, in_water, ..Default::default() };
            let mut world = BoxWorld::default();
            if water { world.set(block_pos(s.position), Block { fluid: Fluid { kind: FluidKind::Water, ..Default::default() }, ..Block::default() }); }
            swim(&mut s, &world);
            assert_eq!(s.swimming, expected, "flying={flying} swimming={swimming}");
        }
        let mut s = PlayerState { profile: crate::Profile { riding: true, ..Default::default() }.into(), ..Default::default() };
        assert_eq!(try_tick(&mut s, &BoxWorld::default(), &Input::default()), Err(Refusal("LivingEntity.travelRidden".into())));
    }

    #[test]
    fn jar_sprint_latch_corpus() {
        for row in include_str!("../tests/data/sprint.tsv").lines() {
            let (bits, expected) = row.split_once(' ').unwrap();
            let bits: u32 = bits.parse().unwrap();
            let mut s = PlayerState {
                sprinting: bits & 1 != 0, food_level: if bits & 4 != 0 { 20 } else { 6 }, in_water: bits & 8 != 0, underwater: bits & 16 != 0,
                swimming: bits & 32 != 0, on_ground: bits & 64 != 0, horizontal_collision: bits & 256 != 0, minor_horizontal_collision: bits & 512 != 0,
                ..PlayerState::default()
            };
            let input = Input { forward: bits & 2 != 0, sneak: bits & 128 != 0, sprint: bits & 2048 != 0, ..Input::default() };
            sprint(&mut s, &input, bits & 1024 != 0, false, false);
            assert_eq!(s.sprinting, expected == "true", "jar sprint case {bits}");
        }
    }

    #[test]
    fn sprint_window_food_and_item_witnesses() {
        // check-reproduce-out.txt:225-299 and checkwitness-out.txt:1-66, as states and inputs.
        let base = || PlayerState { sprint_window: 7, on_ground: true, ..PlayerState::default() };
        let using = |s: &mut PlayerState, can_sprint: bool| { let p = &mut *s.profile; p.using_item = true; p.use_item_can_sprint = Some(can_sprint); };
        let forward = Input { forward: true, ..Input::default() };
        let idle = Input::default();
        // S2: the double tap is killed by a bow or food.
        let mut s = base();
        sprint(&mut s, &forward, false, false, false);
        assert_eq!((s.sprint_trigger_time, s.sprinting), (7, false));
        using(&mut s, false);
        sprint(&mut s, &forward, false, false, false);
        assert_eq!((s.sprint_trigger_time, s.sprinting), (0, false));
        // S3: it survives a spear (canSprint).
        let mut s = base();
        sprint(&mut s, &forward, false, false, false);
        using(&mut s, true);
        sprint(&mut s, &idle, false, false, true);
        assert_eq!((s.sprint_trigger_time, s.sprinting), (6, false));
        sprint(&mut s, &forward, false, false, false);
        assert_eq!((s.sprint_trigger_time, s.sprinting), (5, true));
        // S5 and S6: food 6 refuses in survival, mayfly allows; food 7 is enough.
        let mut s = PlayerState { food_level: 6, ..base() };
        sprint(&mut s, &Input { sprint: true, ..forward }, false, false, false);
        assert_eq!((s.sprint_trigger_time, s.sprinting), (0, false));
        let mut s = PlayerState { food_level: 7, ..base() };
        sprint(&mut s, &Input { sprint: true, ..forward }, false, false, false);
        assert!(s.sprinting);
        let mut s = PlayerState { food_level: 6, profile: crate::Profile { mayfly: true, ..Default::default() }.into(), ..base() };
        sprint(&mut s, &forward, false, false, false);
        assert_eq!(s.sprint_trigger_time, 7);
        sprint(&mut s, &idle, false, false, true);
        sprint(&mut s, &forward, false, false, false);
        assert!(s.sprinting);
        // A1: a bow mid-sprint zeroes the window and keeps sprinting.
        let mut s = PlayerState { sprint_trigger_time: 5, sprinting: true, ..base() };
        using(&mut s, false);
        sprint(&mut s, &forward, false, false, true);
        assert_eq!((s.sprint_trigger_time, s.sprinting), (0, true));
        // A3: a backward tap resets the window.
        let mut s = PlayerState { sprint_trigger_time: 6, ..base() };
        sprint(&mut s, &Input { back: true, ..idle }, false, false, false);
        assert_eq!(s.sprint_trigger_time, 0);
    }

    #[test]
    fn lava_hurt_and_collector_witnesses() {
        // probe-output.txt:3-18 and mycheck-output.txt:1-7, on a server player.
        let lava = |s: &mut PlayerState, cells: &[i32]| {
            let mut c = Collector::default();
            for &step in cells { c.at(step); c.lava(); }
            c.apply(s);
        };
        let mut s = PlayerState::default();
        lava(&mut s, &[0]);
        assert_eq!((s.health.to_bits(), s.remaining_fire_ticks), (0x4180_0000, 300));
        // The window: strict > 10 ticks, then only the excess over the last raw hit.
        let mut s = PlayerState { hurt_cooldown: HurtCooldown { remaining_ticks: 10, last_raw_hit: 100.0 }, ..Default::default() };
        assert!(hurt(&mut s, Hit::LAVA));
        assert_eq!((s.health, s.hurt_cooldown.remaining_ticks, s.hurt_cooldown.last_raw_hit, s.exhaustion), (16.0, 20, 4.0, 0.1));
        let mut s = PlayerState { hurt_cooldown: HurtCooldown { remaining_ticks: 11, last_raw_hit: 4.0 }, ..Default::default() };
        assert!(!hurt(&mut s, Hit::LAVA));
        assert_eq!((s.health, s.hurt_cooldown.remaining_ticks, s.exhaustion), (20.0, 11, 0.0));
        let mut s = PlayerState { hurt_cooldown: HurtCooldown { remaining_ticks: 11, last_raw_hit: 2.0 }, ..Default::default() };
        assert!(hurt(&mut s, Hit::LAVA));
        assert_eq!((s.health, s.hurt_cooldown.remaining_ticks, s.hurt_cooldown.last_raw_hit, s.exhaustion), (18.0, 11, 4.0, 0.1));
        // Two cells in one step or in two: two hurts, the window takes one.
        for cells in [&[0, 0][..], &[0, 1][..]] {
            let mut s = PlayerState::default();
            lava(&mut s, cells);
            assert_eq!(s.health, 16.0);
        }
        // FREEZE runs before CLEAR_FREEZE in one flush.
        let mut c = Collector::default();
        c.at(0);
        c.add(CLEAR_FREEZE);
        c.add(FREEZE);
        let mut s = PlayerState { ticks_frozen: 10, ..Default::default() };
        c.apply(&mut s);
        assert_eq!(s.ticks_frozen, 0);
        // Fire resistance blocks the damage and exhaustion while ignite and clearFreeze run.
        let mut s = PlayerState { ticks_frozen: 10, profile: crate::Profile { fire_resistance: true, ..Default::default() }.into(), ..Default::default() };
        lava(&mut s, &[0]);
        assert_eq!((s.health, s.remaining_fire_ticks, s.ticks_frozen, s.hurt_cooldown.remaining_ticks, s.exhaustion), (20.0, 300, 0, 0, 0.0));
        // Ignite is max-only.
        for (before, after) in [(400, 400), (299, 300)] {
            let mut s = PlayerState { remaining_fire_ticks: before, profile: crate::Profile { fire_resistance: true, ..Default::default() }.into(), ..Default::default() };
            lava(&mut s, &[0]);
            assert_eq!(s.remaining_fire_ticks, after);
        }
    }

    #[test]
    fn flying_speed_witnesses() {
        // evidence/harness-output.txt:6-11: getFlyingSpeed by flying and sprinting.
        for (flying, sprinting, bits) in [(true, false, 0x3d4c_cccd), (true, true, 0x3dcc_cccd), (false, true, 0x3cd4_fdf3), (false, false, 0x3ca3_d70a)] {
            assert_eq!(air_speed(&PlayerState { flying, sprinting, ..Default::default() }).to_bits(), bits);
        }
    }

    #[test]
    fn push_out_direction_order_and_suffocation() {
        let mut world = BoxWorld::default();
        world.set([0, 0, 0], Block::full());
        let mut s = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), velocity: Vec3::new(0.7, 0.2, 0.8), ..PlayerState::default() };
        push_out(&mut s, &world, 0.5, 0.5);
        assert_eq!(s.velocity, Vec3::new(-0.1, 0.2, 0.8));
        world.set([-1, 0, 0], Block::full());
        push_out(&mut s, &world, 0.5, 0.5);
        assert_eq!(s.velocity, Vec3::new(0.1, 0.2, 0.8));
        world.set([0, 0, 0], Block { suffocating: false, ..Block::full() });
        s.velocity = Vec3::default();
        push_out(&mut s, &world, 0.5, 0.5);
        assert_eq!(s.velocity, Vec3::default());
    }

    #[test]
    fn fluid_only_intersection_is_visited_and_extinguishes() {
        let mut world = BoxWorld::default();
        let wet = Block { air: false, inside_shape: Some(VoxelShape::default()), fluid: Fluid { kind: FluidKind::Water, height: 1.0, ..Fluid::default() }, ..Block::default() };
        world.set([0, 0, 0], wet);
        let mut s = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), remaining_fire_ticks: 10, ..PlayerState::default() };
        let p = s.position;
        let mut seen = std::collections::BTreeSet::new();
        let mut cross = |s: &mut PlayerState, world: &BoxWorld, seen: &mut std::collections::BTreeSet<BlockPos>| {
            let mut c = Collector::default();
            segment(s, world, p, p, seen, &mut c, 16, Vec3::default(), &mut None).unwrap();
            c.apply(s);
        };
        cross(&mut s, &world, &mut seen);
        assert!(seen.contains(&[0, 0, 0]));
        assert_eq!(s.remaining_fire_ticks, 0);
        s.remaining_fire_ticks = 10;
        cross(&mut s, &world, &mut seen);
        assert_eq!(s.remaining_fire_ticks, 10);
        seen.clear();
        world.set([0, 0, 0], Block { air: false, inside_shape: Some(VoxelShape::default()), ..Block::default() });
        cross(&mut s, &world, &mut seen);
        assert!(seen.is_empty());
    }

    // Jar anchors: Player.isPushedByFluid = !abilities.flying (Player.java.txt:4122); the feet cell
    // water[amount=7] beside water[amount=4] gives one contributor, current (1,0,0) x 0.014.
    #[test]
    fn flying_takes_no_current() {
        let water = |amount: u8| Block { fluid: Fluid { kind: FluidKind::Water, raw_amount: Some(amount), raw_falling: Some(false), ..Fluid::default() }, ..Block::default() };
        let mut world = BoxWorld::default();
        world.set([0, 0, 0], water(7));
        world.set([1, 0, 0], water(4));
        for (flying, x) in [(false, 0.014f64), (true, 0.0)] {
            let mut s = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), flying, ..PlayerState::default() };
            immerse(&mut s, &world);
            assert!(s.in_water);
            assert_eq!(s.velocity.x.to_bits(), x.to_bits(), "flying={flying}");
        }
    }

    // LivingEntity.knockback from the 26.2 bytecode: one case per branch.
    #[test]
    fn knockback_branches() {
        let hit = |on_ground: bool, velocity: Vec3, strength: f64, resistance: f64, x: f64, z: f64| {
            let mut s = PlayerState { on_ground, velocity, ..Default::default() };
            apply_server_resource_event(&mut s, &BoxWorld::default(), ServerResourceEvent::Knockback { strength, resistance, x, z });
            s.velocity
        };
        const S: f64 = 0.4f32 as f64;
        let v = hit(true, Vec3::new(0.3, -0.08, 0.2), S, 0.0, 1.0, 0.0);
        assert_eq!((v.x.to_bits(), v.y.to_bits(), v.z.to_bits()), ((0.3f64 / 2.0 - S).to_bits(), (-0.08f64 / 2.0 + S).to_bits(), (0.2f64 / 2.0).to_bits()));
        assert_eq!(hit(true, Vec3::new(0.0, 0.2, 0.0), S, 0.0, 1.0, 0.0).y.to_bits(), 0.4f64.to_bits());
        assert_eq!(hit(false, Vec3::new(0.0, 0.3, 0.0), S, 0.0, 1.0, 0.0).y.to_bits(), 0.3f64.to_bits());
        let v0 = Vec3::new(0.3, -0.08, 0.2);
        let v = hit(true, v0, S, 1.0, 1.0, 0.0);
        assert_eq!((v.x.to_bits(), v.y.to_bits(), v.z.to_bits()), (v0.x.to_bits(), v0.y.to_bits(), v0.z.to_bits()));
        let v = hit(false, Vec3::default(), S, 0.0, 1.0, 1.0);
        let unit = 1.0f64 / 2.0f64.sqrt();
        assert_eq!((v.x.to_bits(), v.z.to_bits()), ((0.0 - unit * S).to_bits(), (0.0 - unit * S).to_bits()));
        assert!(hit(true, Vec3::default(), f64::NAN, 0.0, 1.0, 0.0).x.is_nan());
    }

    #[test]
    fn server_resource_cadences_precede_food_data() {
        let resources = |s: &PlayerState| (s.health, s.food_level, s.saturation, s.exhaustion, s.food_tick_timer);
        for (difficulty, natural_regeneration, around_ten, around_twenty) in [
            (0, true, [(19.0, 19, 19.5, 0.0, 9), (20.0, 20, 19.5, 6.0, 0), (20.0, 20, 18.5, 2.0, 0)],
                [(19.0, 20, 20.0, 4.0, 9), (20.0, 20, 19.0, 0.25, 0), (20.0, 20, 19.0, 0.25, 0)]),
            (2, true, [(19.0, 19, 19.5, 0.0, 9), (19.0, 19, 19.5, 0.0, 10), (19.0, 19, 19.5, 0.0, 11)],
                [(19.0, 20, 20.0, 4.0, 9), (20.0, 20, 19.0, 6.25, 0), (20.0, 20, 18.0, 2.25, 0)]),
            (0, false, [(19.0, 19, 19.5, 0.0, 0); 3],
                [(19.0, 20, 20.0, 4.0, 0), (19.0, 20, 19.0, 0.25, 0), (19.0, 20, 19.0, 0.25, 0)]),
        ] {
            let mut world = BoxWorld::default();
            world.difficulty = difficulty;
            world.natural_regeneration = natural_regeneration;
            let mut s = PlayerState { health: 19.0, food_level: 19, saturation: 19.5, food_tick_timer: 8, ..Default::default() };
            for (tick, expected) in (9..=11).zip(around_ten) {
                tick_server_resources(&mut s, &world, tick);
                assert_eq!(resources(&s), expected, "difficulty {difficulty}, regen {natural_regeneration}, tick {tick}");
            }
            let mut s = PlayerState { health: 19.0, saturation: 20.0, exhaustion: 4.0, food_tick_timer: 8, ..Default::default() };
            for (tick, expected) in (19..=21).zip(around_twenty) {
                if tick == 20 { s.exhaustion += 0.25; }
                tick_server_resources(&mut s, &world, tick);
                assert_eq!(resources(&s), expected, "difficulty {difficulty}, regen {natural_regeneration}, tick {tick}");
            }
        }
        let mut world = BoxWorld::default();
        world.difficulty = 0;
        let mut s = PlayerState { saturation: 19.5, ..Default::default() };
        tick_server_resources(&mut s, &world, 20);
        assert_eq!(s.saturation, 20.5);
    }
}
