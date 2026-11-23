use crate::state_reads::{read_state, write_state, modify_state};
use crate::{
    math,
    player::{
        self,
        GameType,
        HurtEntityState,
        InsideBlockEffectType,
        InsideEffectOp,
        SprintMachineState,
        SprintTickInput,
        StepBasedCollector,
    },
    shapes::{
        input_vector,
        axis_step_order
    },
    world::block_pos,
    Aabb,
    Block,
    BlockPos,
    CollisionContext,
    ContactDamage,
    FluidKind,
    GroundedContact,
    HurtCooldown,
    Input,
    Inside,
    PlayerState,
    Pose,
    Vec3,
    VoxelShape,
    World
};

// The state's own methods read and write fields without a world to tell. The tick
// calls them through these, which report the fields first.
fn bounding_box(state: &PlayerState, world: &impl World) -> Aabb {
    let _ = (read_state!(world, state.position), read_state!(world, state.pose));
    state.bounding_box()
}
fn collision_context(state: &PlayerState, world: &impl World) -> CollisionContext {
    let _ = (read_state!(world, state.position), read_state!(world, state.pose), read_state!(world, state.shift_key_down),
        read_state!(world, state.fall_distance), read_state!(world, state.leather_boots));
    CollisionContext::of(state)
}
fn movement_speed_value(state: &PlayerState, world: &impl World) -> f64 {
    let _ = (
        &read_state!(world, state.speed_modifiers),
        read_state!(world, state.sprint_speed_modifier),
        read_state!(world, state.movement_speed_base),
        read_state!(world, state.speed_effect),
        read_state!(world, state.slowness_effect),
    );
    state.movement_speed_value()
}
fn jump_boost_power(state: &PlayerState, world: &impl World) -> f32 {
    let _ = read_state!(world, state.jump_boost);
    state.jump_boost_power()
}
fn set_sprinting(state: &mut PlayerState, world: &impl World, sprinting: bool) {
    world.write_state(crate::state_reads::StateField::sprinting);
    world.write_state(crate::state_reads::StateField::sprint_speed_modifier);
    state.set_sprinting(sprinting);
}

/// A rule the port lacks, met during a tick: the law refuses by name. The state is then
/// partly written, so the caller discards it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal(pub String);
macro_rules! refuse { ($($rule:tt)*) => { return Err(Refusal(format!($($rule)*))) } }
fn unported<T>(tick: Result<T, Refusal>) -> T {
    tick.unwrap_or_else(|Refusal(rule)| panic!("not implemented: {rule}"))
}
/// The same tick as `tick`; an unported rule is returned instead of panicking.
pub fn try_tick(state: &mut PlayerState, world: &impl World, input: &Input) -> Result<(), Refusal> {
    tick_movement(state, world, input).map(|_| ())
}

// Player.tick; Entity.baseTick; LocalPlayer.aiStep; LivingEntity.aiStep
// The coupled food update estimates resources from local movement without a server schedule.
pub fn tick(state: &mut PlayerState, world: &impl World, input: &Input) {
    tick_with_collision(state, world, input);
}

/// Runs the same tick and returns the request and result at Entity.collide.
pub fn tick_with_collision(
    state: &mut PlayerState, world: &impl World, input: &Input,
) -> Option<(Vec3, Vec3)> {
    unported(tick_movement(state, world, input)).0
}

/// Runs the local movement tick and reports its grounded contact request.
/// The request is emitted at stepOn. The local tick never resolves it as damage.
pub fn tick_with_contact(
    state: &mut PlayerState, world: &impl World, input: &Input,
) -> Option<ContactDamage> {
    unported(tick_movement(state, world, input)).1
}

fn update_swimming(state: &mut PlayerState, world: &impl World) {
    write_state!(world, state.swimming) = if read_state!(world, state.flying) {
        false
    } else if read_state!(world, state.swimming) {
        read_state!(world, state.sprinting) && read_state!(world, state.in_water) && !read_state!(world, state.riding)
    } else {
        read_state!(world, state.sprinting) && read_state!(world, state.underwater) && !read_state!(world, state.riding) && world.fluid_block(block_pos(read_state!(world, state.position))).fluid.kind == FluidKind::Water
    };
}

#[cfg(test)]
mod swimming_witness {
    use super::*;
    use crate::{BoxWorld, Fluid};
    #[test]
    fn player_and_entity_swimming_transitions() {
        // Eight direct 26.2 oracle cases, in dossier order.
        let cases = [
            (false,false,true,true,false,false,true,true),
            (false,false,true,true,false,false,false,false),
            (false,true,true,false,true,false,true,true),
            (false,true,false,false,true,false,true,false),
            (true,false,true,true,false,false,true,false),
            (true,true,true,false,true,false,true,false),
            (false,true,true,false,true,true,true,false),
            (false,false,true,true,false,true,true,false),
        ];
        for (flying, swimming, sprinting, underwater, in_water, riding, water, expected) in cases {
            let mut state = PlayerState { flying, swimming, sprinting, underwater, in_water, riding, ..Default::default() };
            let mut world = BoxWorld::default();
            if water { let mut block = Block::default(); block.fluid = Fluid { kind: FluidKind::Water, ..Default::default() }; world.blocks.insert(block_pos(read_state!(world, state.position)), block); }
            update_swimming(&mut state, &world);
            assert_eq!(read_state!(world, state.swimming), expected, "flying={flying} swimming={swimming} riding={riding}");
        }
    }
}

fn has_modeled_effect(state: &PlayerState, world: &impl World) -> bool {
    read_state!(world, state.jump_boost).is_some()
        || read_state!(world, state.speed_effect).is_some()
        || read_state!(world, state.slowness_effect).is_some()
        || read_state!(world, state.blindness)
        || read_state!(world, state.fire_resistance)
        || read_state!(world, state.levitation).is_some()
        || read_state!(world, state.slow_falling)
        || read_state!(world, state.dolphins_grace)
        || read_state!(world, state.weaving)
        || read_state!(world, state.resistance).is_some()
}

fn tick_movement(
    state: &mut PlayerState, world: &impl World, input: &Input,
) -> Result<(Option<(Vec3, Vec3)>, Option<ContactDamage>), Refusal> {
    world.observe_tick();
    let has_modeled_effect = has_modeled_effect(state, world);
    if read_state!(world, state.unmodeled_effects)
        || (read_state!(world, state.effects) && !has_modeled_effect)
    {
        refuse!("LivingEntity: status effects other than none");
    }
    if read_state!(world, state.boat) {
        refuse!("Boat movement");
    }
    if read_state!(world, state.riding) {
        refuse!("LivingEntity.travelRidden");
    }
    let start = read_state!(world, state.position);
    let start_box = bounding_box(state, world);
    write_state!(world, state.movement_this_tick) = None;
    write_state!(world, state.yaw) = input.yaw;
    write_state!(world, state.pitch) = input.pitch;
    write_state!(world, state.was_in_powder_snow) = read_state!(world, state.in_powder_snow);
    write_state!(world, state.in_powder_snow) = false;
    write_state!(world, state.was_eye_in_water) = read_state!(world, state.eye_in_water);
    update_fluid(state, world);
    if read_state!(world, state.in_lava) { modify_state!(world, state.fall_distance) *= 0.5; }
    update_swimming(state, world);
    // Entity.baseTick @125-184 (server branch): check %20 before decrementing; negative grace untouched.
    let mut burn_damage = None;
    let cur_fire = read_state!(world, state.remaining_fire_ticks);
    if cur_fire > 0 {
        if cur_fire % 20 == 0 && !read_state!(world, state.in_lava) {
            burn_damage = Some(ContactDamage::OnFire);
        }
        set_remaining_fire_ticks(state, world, cur_fire - 1);
    }
    let was_shift = read_state!(world, state.shift_key_down);
    let was_jumping = if read_state!(world, state.mayfly) {
        read_state!(world, state.input).jump
    } else {
        false
    };
    let pre_move_vector_y = if read_state!(world, state.sprint_window) > 0
        || read_state!(world, state.sprint_trigger_time) > 0
    {
        let prev = read_state!(world, state.input);
        (prev.forward as i32 - prev.back as i32) as f32
    } else {
        0.0
    };
    write_state!(world, state.crouching) = !read_state!(world, state.flying)
        && !read_state!(world, state.swimming)
        && !read_state!(world, state.riding)
        && fits(state, world, Pose::Crouching)
        && (was_shift || !fits(state, world, Pose::Standing));
    write_state!(world, state.shift_key_down) = input.sneak;
    write_state!(world, state.input) = *input;
    if !read_state!(world, state.no_physics) {
        let offset = 0.6f32 as f64 * 0.35;
        for (x, z) in [(-offset, offset), (-offset, -offset), (offset, -offset), (offset, offset)] {
            move_towards_closest_space(state, world, read_state!(world, state.position).x+x, read_state!(world, state.position).z+z);
        }
    }
    let slowly = read_state!(world, state.crouching) || (read_state!(world, state.pose) == Pose::Swimming && !read_state!(world, state.in_water));
    update_sprinting(state, world, input, slowly, was_shift, pre_move_vector_y);
    // LocalPlayer.aiStep @487-607: creative flight toggle and vertical thrust before super.aiStep().
    if read_state!(world, state.game_mode) == GameType::Spectator {
        if !read_state!(world, state.flying) {
            write_state!(world, state.flying) = true;
        }
    } else if read_state!(world, state.mayfly) && !was_jumping && input.jump {
        if read_state!(world, state.jump_trigger_time) == 0 {
            write_state!(world, state.jump_trigger_time) = 7;
        } else {
            write_state!(world, state.flying) = !read_state!(world, state.flying);
            write_state!(world, state.jump_trigger_time) = 0;
        }
    }
    if read_state!(world, state.flying) {
        let mut dir = 0i32;
        if input.sneak { dir -= 1; }
        if input.jump { dir += 1; }
        if dir != 0 {
            let dy = ((dir as f32 * read_state!(world, state.flying_speed)) * 3.0f32) as f64;
            modify_state!(world, state.velocity).y += dy;
        }
    }
    // Player.aiStep @0-14, @28-49
    if read_state!(world, state.jump_trigger_time) > 0 {
        modify_state!(world, state.jump_trigger_time) -= 1;
    }
    if read_state!(world, state.flying) && !read_state!(world, state.riding) {
        write_state!(world, state.fall_distance) = 0.0;
    }
    // Player.getSpeed reads the attribute after LocalPlayer updates the sprint modifier.
    write_state!(world, state.speed) = movement_speed_value(state, world) as f32;
    if !read_state!(world, state.flying) && read_state!(world, state.in_water) && input.sneak {
        modify_state!(world, state.velocity).y -= 0.04f32 as f64;
    }
    if read_state!(world, state.no_jump_delay) > 0 {
        modify_state!(world, state.no_jump_delay) -= 1;
    }
    if read_state!(world, state.velocity).horizontal_sqr() < 9.0e-6 {
        modify_state!(world, state.velocity).x = 0.0;
        modify_state!(world, state.velocity).z = 0.0;
    }
    if read_state!(world, state.velocity).y.abs() < 0.003 {
        modify_state!(world, state.velocity).y = 0.0;
    }
    let movement = modify_input(input, slowly, read_state!(world, state.sneaking_speed), item_multiplier(state, world));
    write_state!(world, state.on_climbable) = climbable(state, world);
    if input.jump && !read_state!(world, state.flying) && read_state!(world, state.health) > 0.0 {
        let height = if read_state!(world, state.in_lava) {
            read_state!(world, state.lava_height)
        } else {
            read_state!(world, state.water_height)
        };
        let water = read_state!(world, state.in_water) && height > 0.0;
        if water && (!read_state!(world, state.on_ground) || height > 0.4) || read_state!(world, state.in_lava) && (!read_state!(world, state.on_ground) || read_state!(world, state.lava_height) > 0.4) {
            modify_state!(world, state.velocity).y += 0.04f32 as f64;
        } else if (read_state!(world, state.on_ground) || water && height <= 0.4) && read_state!(world, state.no_jump_delay) == 0 {
            jump_from_ground(state, world);
            write_state!(world, state.no_jump_delay) = 10;
        }
    } else {
        write_state!(world, state.no_jump_delay) = 0;
    }
    let movement = if read_state!(world, state.health) <= 0.0 {
        Vec3::default()
    } else {
        movement
    };
    if read_state!(world, state.fall_flying) {
        let clamped = crate::elytra::check_fall_distance_accumulation(
            read_state!(world, state.velocity).y,
            read_state!(world, state.fall_distance),
        );
        write_state!(world, state.fall_distance) = clamped;
        if read_state!(world, state.on_ground)
            || read_state!(world, state.riding)
            || read_state!(world, state.levitation).is_some()
        {
            write_state!(world, state.fall_flying) = false;
        }
    }
    if read_state!(world, state.slow_falling) || read_state!(world, state.levitation).is_some() {
        write_state!(world, state.fall_distance) = 0.0;
    }
    let collision = travel(state, world, movement)?;
    world.observe_sweep(start_box, bounding_box(state, world));
    let contact = apply_block_effects(state, world, start)?;
    let contact = burn_damage.or(contact);
    let before_pose = bounding_box(state, world);
    modify_state!(world, state.position).x = read_state!(world, state.position).x.clamp(-29999999.0, 29999999.0);
    modify_state!(world, state.position).z = read_state!(world, state.position).z.clamp(-29999999.0, 29999999.0);
    // LocalPlayer.aiStep @608-640: clear creative flight upon touching ground, then Player.tick calls updatePlayerPose.
    if read_state!(world, state.on_ground)
        && read_state!(world, state.flying)
        && read_state!(world, state.game_mode) != GameType::Spectator
    {
        write_state!(world, state.flying) = false;
    }
    update_pose(state, world);
    world.observe_sweep(before_pose, bounding_box(state, world));
    write_state!(world, state.on_climbable) = climbable(state, world);
    movement_exhaustion(state, world, read_state!(world, state.position).minus(start));
    food_tick(state, world);
    modify_state!(world, state.t) += 1;
    Ok((collision, contact))
}

// LocalPlayer.aiStep; LocalPlayer.canStartSprinting; LocalPlayer.shouldStopRunSprinting; LocalPlayer.shouldStopSwimSprinting
fn update_sprinting(
    state: &mut PlayerState,
    world: &impl World,
    input: &Input,
    slowly: bool,
    was_shift: bool,
    pre_move_vector_y: f32,
) {
    let prev_sprint_trigger = read_state!(world, state.sprint_trigger_time);
    let was_sprinting = read_state!(world, state.sprinting);
    let using_item = read_state!(world, state.using_item);
    let use_effects_can_sprint = if using_item {
        read_state!(world, state.use_item_can_sprint).expect("used item canSprint")
    } else {
        true
    };
    let blindness = read_state!(world, state.blindness);
    let mayfly = read_state!(world, state.mayfly);
    let food_level = read_state!(world, state.food_level);
    let swimming = read_state!(world, state.swimming);
    // Player.rs `step_ai_sprint` uses `food_data_has_enough_food` in `shouldStopSwimSprinting`,
    // whereas 26.2 `LocalPlayer.shouldStopSwimSprinting` calls `!isSprintingPossible(true)`.
    let sprint_food = if was_sprinting && swimming {
        if blindness { 0 } else if mayfly { 7 } else { food_level }
    } else {
        food_level
    };
    let mut machine = SprintMachineState {
        sprint_trigger_time: prev_sprint_trigger,
        is_sprinting: was_sprinting,
    };
    let sprint_input = SprintTickInput {
        pre_shift_key_down: was_shift,
        pre_move_vector_y,
        post_move_vector_y: (input.forward as i32 - input.back as i32) as f32,
        post_backward: input.back,
        post_sprint_key: input.sprint,
        sprint_window: read_state!(world, state.sprint_window),
        using_item,
        use_effects_can_sprint,
        is_passenger: read_state!(world, state.riding),
        vehicle_can_sprint: false,
        food_level: sprint_food,
        mayfly,
        flying: read_state!(world, state.flying),
        in_water: read_state!(world, state.in_water),
        underwater: read_state!(world, state.underwater),
        fall_flying: read_state!(world, state.fall_flying),
        moving_slowly: slowly,
        swimming,
        on_ground: read_state!(world, state.on_ground),
        sneak_key: input.sneak,
        horizontal_collision: read_state!(world, state.horizontal_collision),
        minor_horizontal_collision: read_state!(world, state.minor_horizontal_collision),
        blindness,
    };
    player::step_ai_sprint(&mut machine, &sprint_input);
    if machine.sprint_trigger_time != prev_sprint_trigger {
        write_state!(world, state.sprint_trigger_time) = machine.sprint_trigger_time;
    }
    if machine.is_sprinting != was_sprinting {
        set_sprinting(state, world, machine.is_sprinting);
    }
}

// LocalPlayer.suffocatesAt
fn suffocates_at(state: &PlayerState, world: &impl World, p: BlockPos) -> bool {
    let body = bounding_box(state, world);
    let bounds = Aabb::new(p[0] as f64, body.min.y, p[2] as f64,
        p[0] as f64+1.0, body.max.y, p[2] as f64+1.0).deflate(1.0e-7);
    world.collides_with_suffocating_block(bounds, collision_context(state, world))
}

// LocalPlayer.moveTowardsClosestSpace
fn move_towards_closest_space(state: &mut PlayerState, world: &impl World, x: f64, z: f64) {
    let p = block_pos(Vec3::new(x, read_state!(world, state.position).y, z));
    if !suffocates_at(state, world, p) {
        return;
    }
    let dx = x-p[0] as f64;
    let dz = z-p[2] as f64;
    let mut nearest = f64::MAX;
    let mut direction = None;
    for (axis, step, distance) in [(crate::Axis::X, -1, dx), (crate::Axis::X, 1, 1.0-dx),
        (crate::Axis::Z, -1, dz), (crate::Axis::Z, 1, 1.0-dz)] {
        let mut neighbor = p;
        neighbor[axis as usize] += step;
        if distance < nearest && !suffocates_at(state, world, neighbor) {
            nearest = distance;
            direction = Some((axis, step));
        }
    }
    if let Some((axis, step)) = direction {
        write_state!(world, state.velocity) = read_state!(world, state.velocity).with(axis, 0.1*step as f64);
    }
}

// KeyboardInput.tick; LocalPlayer.modifyInput; LocalPlayer.modifyInputSpeedForSquareMovement
fn item_multiplier(state: &PlayerState, world: &impl World) -> Option<f32> {
    read_state!(world, state.using_item).then(|| read_state!(world, state.use_item_speed_multiplier).expect("used item speedMultiplier"))
}

fn modify_input(i: &Input, slowly: bool, sneaking_speed: f32, item_multiplier: Option<f32>) -> Vec3 {
    let mut x = (i.left as i32 - i.right as i32) as f32;
    let mut z = (i.forward as i32 - i.back as i32) as f32;
    let length = math::sqrt(x*x+z*z);
    if length < 1.0e-4 {
        return Vec3::default();
    }
    x /= length;
    z /= length;
    x *= 0.98f32;
    z *= 0.98f32;
    if let Some(multiplier) = item_multiplier { x *= multiplier; z *= multiplier; }
    if slowly {
        x *= sneaking_speed;
        z *= sneaking_speed;
    }
    let length = math::sqrt(x*x+z*z);
    if length > 0.0 {
        let reciprocal = 1.0f32 / length;
        x *= reciprocal;
        z *= reciprocal;
        let a = x.abs();
        let b = z.abs();
        let ratio = if b > a {
            a/b
        } else {
            b/a
        };
        let distance = math::sqrt(1.0f32+ratio*ratio);
        let speed = (length*distance).min(1.0);
        x *= speed;
        z *= speed;
    }
    Vec3::new(x as f64, 0.0, z as f64)
}

// Player.getFlyingSpeed
fn flying_speed(state: &PlayerState, world: &impl World) -> f32 {
    player::player_get_flying_speed(
        read_state!(world, state.flying),
        read_state!(world, state.riding),
        read_state!(world, state.sprinting),
        read_state!(world, state.flying_speed),
    )
}

// LivingEntity.getFrictionInfluencedSpeed
fn friction_speed(state: &PlayerState, world: &impl World, f: f32) -> f32 {
    if read_state!(world, state.on_ground) {
        if f as f64 > 0.6 {
            read_state!(world, state.speed) * (0.21600002f32 / (f*f*f))
        } else {
            read_state!(world, state.speed)
        }
    } else {
        flying_speed(state, world)
    }
}

// Entity.getOnPos
fn on_pos(state: &PlayerState, world: &impl World, offset: f32) -> BlockPos {
    if let Some(p) = read_state!(world, state.main_supporting_block) {
        if offset <= 1.0e-5f32 {
            return p;
        }
        let b = world.hook_block(p);
        if offset as f64 <= 0.5 && b.fence || b.wall_or_gate {
            return p;
        }
        return [p[0],
        math::floor(read_state!(world, state.position).y-offset as f64),
        p[2]];
    }
    block_pos(Vec3::new(read_state!(world, state.position).x, read_state!(world, state.position).y-offset as f64, read_state!(world, state.position).z))
}

// Entity.getBlockJumpFactor
fn jump_factor(state: &PlayerState, world: &impl World) -> f32 {
    let here = world.hook_block(block_pos(read_state!(world, state.position))).jump_factor;
    let below = world.hook_block(on_pos(state, world, 0.500001)).jump_factor;
    if here as f64 == 1.0 {
        below
    } else {
        here
    }
}

// Entity.getBlockSpeedFactor
fn speed_factor(state: &PlayerState, world: &impl World) -> f32 {
    let b = world.hook_block(block_pos(read_state!(world, state.position)));
    if b.water_block {
        return b.speed_factor;
    }
    if b.speed_factor as f64 == 1.0 {
        world.hook_block(on_pos(state, world, 0.500001)).speed_factor
    } else {
        b.speed_factor
    }
}

// LivingEntity.getJumpPower; LivingEntity.jumpFromGround; ServerPlayer.jumpFromGround
fn jump_from_ground(state: &mut PlayerState, world: &impl World) {
    let power = read_state!(world, state.jump_strength) as f32 * 1.0f32 * jump_factor(state, world)
        + jump_boost_power(state, world);
    if power > 1.0e-5f32 {
        modify_state!(world, state.velocity).y = (power as f64).max(read_state!(world, state.velocity).y);
        if read_state!(world, state.sprinting) {
            let yaw = (read_state!(world, state.yaw) * 0.017453292f32) as f64;
            write_state!(world, state.velocity) = read_state!(world, state.velocity).plus(Vec3::new((-math::sin(yaw)) as f64 * 0.2, 0.0, math::cos(yaw) as f64*0.2));
        }
    }
    cause_food_exhaustion(state, world, if read_state!(world, state.sprinting) {
        0.2
    } else {
        0.05
    });
}

// LivingEntity.onClimbable; LivingEntity.trapdoorUsableAsLadder
fn climbable(state: &PlayerState, world: &impl World) -> bool {
    let p = block_pos(read_state!(world, state.position));
    let b = world.hook_block(p);
    b.climbable || b.open_trapdoor_facing.is_some_and(|f| world.hook_block([p[0], p[1]-1, p[2]]).ladder_facing == Some(f))
}

// LivingEntity.handleOnClimbable
fn handle_climbable(state: &mut PlayerState, world: &impl World) {
    if climbable(state, world) {
        write_state!(world, state.fall_distance) = 0.0;
        modify_state!(world, state.velocity).x = read_state!(world, state.velocity).x.clamp(-0.15f32 as f64, 0.15f32 as f64);
        modify_state!(world, state.velocity).z = read_state!(world, state.velocity).z.clamp(-0.15f32 as f64, 0.15f32 as f64);
        modify_state!(world, state.velocity).y = read_state!(world, state.velocity).y.max(-0.15f32 as f64);
        if read_state!(world, state.velocity).y < 0.0 && !world.hook_block(block_pos(read_state!(world, state.position))).scaffolding && read_state!(world, state.shift_key_down) {
            modify_state!(world, state.velocity).y = 0.0;
        }
    }
}

// LivingEntity.getEffectiveGravity (@8709)
fn effective_gravity(state: &PlayerState, world: &impl World, pre_bump_vy: f64) -> f64 {
    let g = read_state!(world, state.gravity);
    if pre_bump_vy <= 0.0 && read_state!(world, state.slow_falling) {
        g.min(0.01)
    } else {
        g
    }
}

// Player.travel; LivingEntity.travel
fn travel(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    if read_state!(world, state.swimming) {
        let pitch = (read_state!(world, state.pitch) * 0.017453292f32) as f64;
        let look_y = -math::sin(pitch) as f64;
        let factor = if look_y < -0.2 {
            0.085
        } else {
            0.06
        };
        if look_y <= 0.0 || read_state!(world, state.input).jump || world.fluid_block(block_pos(Vec3::new(read_state!(world, state.position).x, read_state!(world, state.position).y+1.0-0.1, read_state!(world, state.position).z))).fluid.kind != FluidKind::Empty {
            modify_state!(world, state.velocity).y += (look_y - read_state!(world, state.velocity).y) * factor;
        }
    }
    let flying = read_state!(world, state.flying);
    let pre_travel_vy = if flying { read_state!(world, state.velocity).y } else { 0.0 };
    let collision = if !flying && (read_state!(world, state.in_water) || read_state!(world, state.in_lava)) {
        travel_fluid(state, world, movement)?
    } else if read_state!(world, state.fall_flying) {
        travel_fall_flying(state, world, movement)?
    } else {
        travel_air(state, world, movement)?
    };
    if flying {
        write_state!(world, state.velocity).y = pre_travel_vy * 0.6;
    }
    Ok(collision)
}

// LivingEntity.travelFallFlying (@0-73)
fn travel_fall_flying(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    if climbable(state, world) {
        let collision = travel_air(state, world, movement)?;
        write_state!(world, state.fall_flying) = false;
        return Ok(collision);
    }
    let vel = read_state!(world, state.velocity);
    let pre_h = vel.horizontal_sqr().sqrt();
    let g = effective_gravity(state, world, vel.y);
    let pitch = read_state!(world, state.pitch);
    let look = crate::math::calculate_view_vector(read_state!(world, state.yaw), pitch);
    let updated = crate::elytra::update_fall_flying_movement(vel, look, pitch, g);
    write_state!(world, state.velocity) = updated;
    let collision = move_player(state, world, updated)?;
    let post_h = read_state!(world, state.velocity).horizontal_sqr().sqrt();
    if let Some(damage) = crate::elytra::fly_into_wall_damage(read_state!(world, state.horizontal_collision), pre_h, post_h) {
        // flyIntoWall: bypasses_armor = true, is_fire = false, is_fall = false, exhaustion = 0.0.
        apply_server_damage(state, world, damage, false, true, false, false, 0.0);
    }
    Ok(collision)
}

// LivingEntity.travelInAir; LivingEntity.handleRelativeFrictionAndCalculateMovement
fn travel_air(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    let below = on_pos(state, world, 0.500001);
    let f = if read_state!(world, state.on_ground) {
        math::modified_friction(world.hook_block(below).friction, read_state!(world, state.friction_modifier))
    } else {
        1.0
    };
    let acceleration = friction_speed(state, world, f);
    write_state!(world, state.velocity) = read_state!(world, state.velocity).plus(input_vector(movement, acceleration, read_state!(world, state.yaw)));
    handle_climbable(state, world);
    let collision = move_player(state, world, read_state!(world, state.velocity))?;
    let mut velocity = read_state!(world, state.velocity);
    let pre_bump_vy = velocity.y;
    if (read_state!(world, state.horizontal_collision) || read_state!(world, state.input).jump) && (climbable(state, world) || read_state!(world, state.was_in_powder_snow) && read_state!(world, state.leather_boots)) {
        velocity.y = 0.2;
    }
    let vy = if let Some(amp) = read_state!(world, state.levitation) {
        velocity.y + (0.05 * ((amp as i32 + 1) as f64) - velocity.y) * 0.2
    } else {
        // LivingEntity.travelInAir @101-131: unloaded-chunk -0.1/0.0 fallback is client-only
        // (`isClientSide() == false` jumps directly to gravity) and dead even on 26.2 client.
        velocity.y - effective_gravity(state, world, pre_bump_vy)
    };
    let drag = math::modified_friction(0.91, read_state!(world, state.air_drag_modifier));
    let horizontal = f*drag;
    world.observe_air_travel(movement.z * acceleration as f64, horizontal);
    let vertical = math::modified_friction(0.98, read_state!(world, state.air_drag_modifier));
    write_state!(world, state.velocity) = Vec3::new(velocity.x*horizontal as f64, vy*vertical as f64, velocity.z*horizontal as f64);
    Ok(collision)
}

// LivingEntity.travelInFluid; LivingEntity.travelInWater; LivingEntity.travelInLava
fn travel_fluid(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    let pre_vel_y = read_state!(world, state.velocity).y;
    let falling = pre_vel_y <= 0.0;
    let base_gravity = effective_gravity(state, world, pre_vel_y);
    let old_y = read_state!(world, state.position).y;
    let collision = if read_state!(world, state.in_water) {
        let mut drag = if read_state!(world, state.sprinting) {
            0.9f32
        } else {
            0.8f32
        };
        let mut acceleration = 0.02f32;
        let efficiency = read_state!(world, state.water_movement_efficiency) * if read_state!(world, state.on_ground) {
            1.0
        } else {
            0.5
        };
        if efficiency > 0.0 {
            drag += (0.54600006f32-drag)*efficiency;
            acceleration += (read_state!(world, state.speed)-acceleration)*efficiency;
        }
        if read_state!(world, state.dolphins_grace) {
            drag = 0.96f32;
        }
        write_state!(world, state.velocity) = read_state!(world, state.velocity).plus(input_vector(movement, acceleration, read_state!(world, state.yaw)));
        let collision = move_player(state, world, read_state!(world, state.velocity))?;
        if read_state!(world, state.horizontal_collision) && climbable(state, world) {
            modify_state!(world, state.velocity).y = 0.2;
        }
        write_state!(world, state.velocity) = read_state!(world, state.velocity).multiply(Vec3::new(drag as f64, 0.8f32 as f64, drag as f64));
        fluid_falling(state, world, falling, base_gravity);
        collision
    } else {
        write_state!(world, state.velocity) = read_state!(world, state.velocity).plus(input_vector(movement, 0.02, read_state!(world, state.yaw)));
        let collision = move_player(state, world, read_state!(world, state.velocity))?;
        if read_state!(world, state.lava_height) <= 0.4 {
            write_state!(world, state.velocity) = read_state!(world, state.velocity).multiply(Vec3::new(0.5, 0.8f32 as f64, 0.5));
            fluid_falling(state, world, falling, base_gravity);
        } else {
            write_state!(world, state.velocity) = read_state!(world, state.velocity).scale(0.5);
        }
        if base_gravity != 0.0 {
            modify_state!(world, state.velocity).y += -base_gravity/4.0;
        }
        collision
    };
    let v = read_state!(world, state.velocity);
    let jump_box = bounding_box(state, world).moved(Vec3::new(v.x, v.y+0.6f32 as f64-read_state!(world, state.position).y+old_y, v.z));
    if read_state!(world, state.horizontal_collision) && world.no_collision(jump_box, collision_context(state, world)) && !contains_fluid(world, jump_box) {
        modify_state!(world, state.velocity).y = 0.3f32 as f64;
    }
    Ok(collision)
}

// LivingEntity.getFluidFallingAdjustedMovement
fn fluid_falling(state: &mut PlayerState, world: &impl World, falling: bool, base_gravity: f64) {
    if base_gravity != 0.0 && !read_state!(world, state.sprinting) {
        modify_state!(world, state.velocity).y = if falling && (read_state!(world, state.velocity).y-0.005).abs() >= 0.003 && (read_state!(world, state.velocity).y-base_gravity/16.0).abs() < 0.003 {
            -0.003
        } else {
            read_state!(world, state.velocity).y-base_gravity/16.0
        };
    }
}

// Player.canPlayerFitWithinBlocksAndEntitiesWhen
fn fits(state: &PlayerState, world: &impl World, pose: Pose) -> bool {
    world.no_collision(Aabb::player(read_state!(world, state.position), pose.height()).deflate(1.0e-7), collision_context(state, world))
}

// Player.updatePlayerPose; Player.getDesiredPose
fn update_pose(state: &mut PlayerState, world: &impl World) {
    if !fits(state, world, Pose::Swimming) {
        return;
    }
    let desired = if read_state!(world, state.swimming) {
        Pose::Swimming
    } else if read_state!(world, state.fall_flying) {
        Pose::FallFlying
    } else if read_state!(world, state.shift_key_down) && !read_state!(world, state.flying) {
        Pose::Crouching
    } else {
        Pose::Standing
    };
    write_state!(world, state.pose) = if read_state!(world, state.game_mode) == GameType::Spectator
        || read_state!(world, state.riding)
        || fits(state, world, desired)
    {
        desired
    } else if fits(state, world, Pose::Crouching) {
        Pose::Crouching
    } else {
        Pose::Swimming
    };
}

// Player.canFallAtLeast
pub fn can_fall(state: &PlayerState, world: &impl World, x: f64, z: f64, depth: f64) -> bool {
    let b = bounding_box(state, world);
    world.no_collision(Aabb::new(b.min.x+1.0e-7+x, b.min.y-depth-1.0e-7, b.min.z+1.0e-7+z,
    b.max.x-1.0e-7+x, b.min.y, b.max.z-1.0e-7+z), collision_context(state, world))
}

// Player.maybeBackOffFromEdge; Player.isAboveGround
fn back_off(state: &PlayerState, world: &impl World, mut v: Vec3) -> Vec3 {
    let step = read_state!(world, state.step_height);
    if v.y > 0.0
        || !read_state!(world, state.shift_key_down)
        || read_state!(world, state.flying)
        || !(read_state!(world, state.on_ground) || read_state!(world, state.fall_distance) < step as f64 && !can_fall(state, world, 0.0, 0.0, step as f64-read_state!(world, state.fall_distance)))
    {
        return v;
    }
    let delta = if cfg!(feature="mutant-edge-backoff") {
        0.06
    } else {
        0.05
    };
    let dx = v.x.signum()*delta;
    let dz = v.z.signum()*delta;
    while v.x != 0.0 && can_fall(state, world, v.x, 0.0, step as f64) {
        if v.x.abs() <= delta {
            v.x=0.0;
            break;
        }
        v.x-=dx;
    }
    while v.z != 0.0 && can_fall(state, world, 0.0, v.z, step as f64) {
        if v.z.abs() <= delta {
            v.z=0.0;
            break;
        }
        v.z-=dz;
    }
    while v.x != 0.0 && v.z != 0.0 && can_fall(state, world, v.x, v.z, step as f64) {
        if v.x.abs() <= delta {
            v.x=0.0;
        } else {
            v.x-=dx;
        }
        if v.z.abs() <= delta {
            v.z=0.0;
        } else {
            v.z-=dz;
        }
    }
    v
}

// Entity.collide; Entity.collectCandidateStepUpHeights
fn collide_in_world(world: &impl World, bounds: Aabb, context: CollisionContext, wanted: Vec3, b: Aabb) -> crate::contact::Pass {
    // Inline storage is independent of terrain complexity. If it fills, stream
    // the arbitrary-size query instead; no shape or result allocation is needed.
    let mut colliders: [Option<crate::shapes::SharedCollider>;32] = std::array::from_fn(|_|None);
    let mut len=0;
    let complete=world.visit_collisions(bounds,context,|shape| {
        if len==colliders.len() { return false; }
        colliders[len]=Some(shape.shared()); len+=1; true
    });
    if complete {
        if len==0 { return crate::contact::Pass::free(bounds,b,wanted); }
        let mut result=Vec3::default();
        let mut report=crate::contact::Pass::free(bounds,b,wanted);
        for axis in axis_step_order(wanted) {
            let mut d=wanted.get(axis);
            if d==0.0 { continue; }
            let moved=b.moved(result);
            for shape in colliders[..len].iter().flatten() {
                if d.abs()<1.0e-7 { report.axes[axis as usize].outcome=crate::contact::Outcome::Hold {plane:report.axes[axis as usize].outcome.plane(),remainder:d};d=0.0; break; }
                d=shape.view().collide_report(axis,moved,d,&mut report.axes[axis as usize].outcome);
            }
            result=result.with(axis,d);report.axes[axis as usize].body=moved;report.axes[axis as usize].moved=d;
        }
        report.moved=result;return report;
    }
    let mut result = Vec3::default();
    let mut any = false;
    let mut report=crate::contact::Pass::free(bounds,b,wanted);
    for axis in axis_step_order(wanted) {
        let mut d = wanted.get(axis);
        if d == 0.0 { continue; }
        let moved = b.moved(result);
        world.visit_collisions(bounds, context, |shape| {
            any = true;
            if d.abs() < 1.0e-7 { report.axes[axis as usize].outcome=crate::contact::Outcome::Hold {plane:report.axes[axis as usize].outcome.plane(),remainder:d};d=0.0; return false; }
            d = shape.collide_report(axis, moved, d,&mut report.axes[axis as usize].outcome);
            true
        });
        result = result.with(axis,d);report.axes[axis as usize].body=moved;report.axes[axis as usize].moved=d;
    }
    report.moved=if any {result} else {wanted};report
}

fn collide(state: &PlayerState, world: &impl World, v: Vec3) -> Vec3 {
    let b = bounding_box(state, world);
    let context = collision_context(state, world);
    let report = if v.length_sqr() == 0.0 {
        crate::contact::Pass::free(b,b,v)
    } else {
        collide_in_world(world, b.expand(v), context, v, b)
    };
    let result=report.moved;
    let landed = v.y != result.y && v.y < 0.0;
    let height = if cfg!(feature="mutant-step-up") {
        f32::from_bits(read_state!(world, state.step_height).to_bits()+1)
    } else {
        read_state!(world, state.step_height)
    };
    if height > 0.0 && (landed || read_state!(world, state.on_ground)) && (v.x != result.x || v.z != result.z) {
        let base = if landed {
            b.moved(Vec3::new(0.0, result.y, 0.0))
        } else {
            b
        };
        let mut bounds = base.expand(Vec3::new(v.x, height as f64, v.z));
        if !landed {
            bounds = bounds.expand(Vec3::new(0.0, -1.0e-5f32 as f64, 0.0));
        }
        let previous_y = result.y as f32;
        let mut last: Option<f32> = None;
        loop {
            let mut next: Option<f32> = None;
            world.visit_collisions(bounds, context, |shape| {
            for i in 0..shape.shape.coords[1].len() {
                let candidate = (shape.coord(1,i)-base.min.y) as f32;
                if candidate < 0.0 || candidate == previous_y {
                    continue;
                }
                if candidate > height {
                    break;
                }
                if last.is_some_and(|v| candidate == v || candidate.total_cmp(&v).is_le()) { continue; }
                if next.is_none_or(|v| candidate.total_cmp(&v).is_lt()) {
                    next=Some(candidate);
                }
            }
            true
            });
            let Some(candidate) = next else { break; };
            last=Some(candidate);
            let step_report = collide_in_world(world, bounds, context, Vec3::new(v.x, candidate as f64, v.z), base);
            let stepped=step_report.moved;
            if stepped.horizontal_sqr() > result.horizontal_sqr() {
                step_report.report(world);return stepped.minus(Vec3::new(0.0, b.min.y-base.min.y, 0.0));
            }
        }
    }
    report.report(world);result
}

// CollisionGetter.findSupportingBlock; Entity.checkSupportingBlock
fn supporting_block(state: &mut PlayerState, world: &impl World, moved: Vec3) {
    if !read_state!(world, state.on_ground) {
        write_state!(world, state.on_ground_no_blocks)=false;
        write_state!(world, state.main_supporting_block)=None;
        return;
    }
    let b = bounding_box(state, world);
    let foot = Aabb::new(b.min.x, b.min.y-1.0e-6, b.min.z, b.max.x, b.min.y, b.max.z);
    let find = |bounds: Aabb| {
        let lo = block_pos(bounds.min);
        let hi = block_pos(bounds.max);
        let (lo,hi)=world.collision_cell_bounds(bounds,lo.map(|v|v-1),hi.map(|v|v+1))?;
        let mut best: Option<(f64, BlockPos)> = None;
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    let p=[x, y, z];
                    if !world.collision_shape(p, collision_context(state, world)).at(Vec3::new(x as f64, y as f64, z as f64)).intersects(bounds) {
                        continue;
                    }
                    let d=Vec3::new(x as f64+0.5, y as f64+0.5, z as f64+0.5).minus(read_state!(world, state.position)).length_sqr();
                    if best.is_none_or(|(old, q)| d < old || d == old && (y, z, x) > (q[1], q[2], q[0])) {
                        best=Some((d, p));
                    }
                }
            }
        }
        best.map(|(_, p)|p)
    };
    let found = find(foot);
    if found.is_some() || read_state!(world, state.on_ground_no_blocks) {
        write_state!(world, state.main_supporting_block)=found;
    } else {
        write_state!(world, state.main_supporting_block)=find(foot.moved(Vec3::new(-moved.x, 0.0, -moved.z)));
    }
    write_state!(world, state.on_ground_no_blocks) = read_state!(world, state.main_supporting_block).is_none();
}

// Entity.move; Entity.restituteMovementAfterCollisions
fn move_player(state: &mut PlayerState, world: &impl World, mut wanted: Vec3) -> Result<Option<(Vec3, Vec3)>, Refusal> {
    if read_state!(world, state.no_physics) {
        write_state!(world, state.position) = read_state!(world, state.position).plus(wanted);
        return Ok(None);
    }
    if read_state!(world, state.stuck_speed_multiplier).length_sqr() > 1.0e-7 {
        wanted = wanted.multiply(read_state!(world, state.stuck_speed_multiplier));
        write_state!(world, state.stuck_speed_multiplier)=Vec3::default();
        write_state!(world, state.velocity)=Vec3::default();
    }
    wanted = back_off(state, world, wanted);
    let moved=collide(state, world, wanted);
    let distance=moved.length_sqr();
    if distance > 1.0e-7 || wanted.length_sqr()-distance < 1.0e-7 {
        if read_state!(world, state.fall_distance) != 0.0 && distance >= 1.0 && fall_reset_ray(state, world, moved) {
            write_state!(world, state.fall_distance)=0.0;
        }
        write_state!(world, state.movement_this_tick)=Some(wanted);
        write_state!(world, state.position)=read_state!(world, state.position).plus(moved);
    }
    let hit_x=!math::equal(wanted.x, moved.x);
    let hit_z=!math::equal(wanted.z, moved.z);
    write_state!(world, state.horizontal_collision)=hit_x||hit_z;
    write_state!(world, state.vertical_collision)=wanted.y!=moved.y;
    write_state!(world, state.vertical_collision_below)=read_state!(world, state.vertical_collision)&&wanted.y<0.0;
    write_state!(world, state.on_ground)=read_state!(world, state.vertical_collision_below);
    supporting_block(state, world, moved);
    write_state!(world, state.minor_horizontal_collision)=minor_collision(state, world, moved);
    if !read_state!(world, state.in_water) {
        update_fluid(state, world);
    }
    let block=world.hook_block(on_pos(state, world, 0.2));
    check_fall_damage(state, world, block, moved.y)?;
    if wanted.y.abs()>0.0 && read_state!(world, state.vertical_collision) || read_state!(world, state.horizontal_collision) {
        let mut bounce=if read_state!(world, state.shift_key_down) {
            0.0
        } else {
            read_state!(world, state.bounciness)
        };
        let before=read_state!(world, state.velocity);
        if hit_x {
            modify_state!(world, state.velocity).x=-before.x*bounce;
        }
        if hit_z {
            modify_state!(world, state.velocity).z=-before.z*bounce;
        }
        if read_state!(world, state.vertical_collision) {
            if read_state!(world, state.vertical_collision_below) {
                bounce=if -before.y < read_state!(world, state.gravity) || read_state!(world, state.shift_key_down) || block.suppresses_bounce {
                    0.0
                } else {
                    bounce.max(block.bounce as f64)
                };
            }
            let (gravity, drag)=if bounce>0.0 {
                let fraction=moved.y/before.y;
                (fraction*read_state!(world, state.gravity), 1.0+fraction*(math::modified_friction(0.98, read_state!(world, state.air_drag_modifier)) as f64-1.0))
            } else {
                (0.0, 1.0)
            };
            if !(cfg!(feature="mutant-ceiling") && wanted.y>0.0) {
                modify_state!(world, state.velocity).y=(gravity-before.y)*drag*bounce;
            }
        }
    }
    let factor=speed_factor(state, world);
    let factor=(factor+read_state!(world, state.movement_efficiency)*(1.0f32-factor)) as f64;
    world.observe_speed_factor(factor);
    write_state!(world, state.velocity)=read_state!(world, state.velocity).multiply(Vec3::new(factor, 1.0, factor));
    Ok(Some((wanted, moved)))
}

// LocalPlayer.isHorizontalCollisionMinor
fn minor_collision(state: &PlayerState, world: &impl World, moved: Vec3) -> bool {
    if !read_state!(world, state.horizontal_collision) {
        return false;
    }
    let input=modify_input(&read_state!(world, state.input), read_state!(world, state.crouching)||(read_state!(world, state.pose)==Pose::Swimming&&!read_state!(world, state.in_water)), read_state!(world, state.sneaking_speed), item_multiplier(state, world));
    let yaw = (read_state!(world, state.yaw) * 0.017453292f32) as f64;
    let sin = math::sin(yaw) as f64;
    let cos = math::cos(yaw) as f64;
    let intended=Vec3::new(input.x*cos-input.z*sin, 0.0, input.z*cos+input.x*sin);
    let a=intended.horizontal_sqr();
    let b=moved.horizontal_sqr();
    if a<1.0e-5f32 as f64 || b<1.0e-5f32 as f64 {
        return false;
    }
    let angle=((intended.x*moved.x+intended.z*moved.z)/(a*b).sqrt()).acos();
    angle < 0.13962633907794952
}

// LivingEntity.calculateFallPower
pub fn calculate_fall_power(state: &PlayerState, distance: f64) -> f64 {
    distance+1.0e-6-state.safe_fall_distance
}

// LivingEntity.calculateFallDamage
pub fn calculate_fall_damage(state: &PlayerState, distance: f64, multiplier: f32) -> i32 {
    math::floor(calculate_fall_power(state, distance)*multiplier as f64*state.fall_damage_multiplier)
}

// Entity.checkFallDamage; LivingEntity.checkFallDamage; Block.fallOn
fn check_fall_damage(state: &mut PlayerState, world: &impl World, block: &Block, dy: f64) -> Result<(), Refusal> {
    if !read_state!(world, state.in_water) && dy<0.0 {
        modify_state!(world, state.fall_distance) -= (dy as f32) as f64;
    }
    if read_state!(world, state.on_ground) {
        if read_state!(world, state.fall_distance)>0.0 {
            if let Some(name) = &block.unsupported_fall_on {
                refuse!("BlockState.fallOn: {name}");
            }
            if !read_state!(world, state.mayfly) {
                let (distance, multiplier) = if block.slime || block.inside == Inside::PowderSnow {
                    (read_state!(world, state.fall_distance), 0.0)
                } else if block.fall_on_multiplier == 2.0 {
                    let (dist, mult, _) = player::dripstone_fall_on_args(
                        read_state!(world, state.fall_distance),
                        true,
                        true,
                    );
                    (dist, mult)
                } else {
                    (
                        if block.bed {
                            read_state!(world, state.fall_distance) * 0.5
                        } else {
                            read_state!(world, state.fall_distance)
                        },
                        block.fall_on_multiplier,
                    )
                };
                let _ = (read_state!(world, state.safe_fall_distance), read_state!(world, state.fall_damage_multiplier));
                let damage=calculate_fall_damage(state, distance, multiplier).max(0);
                if damage>0 {
                    // fall damage type: is_fire=false, bypasses_armor=true, is_fall=true, exhaustion=0.0.
                    apply_server_damage(state, world, damage as f32, false, true, false, true, 0.0);
                }
            }
        }
        write_state!(world, state.fall_distance)=0.0;
    }
    Ok(())
}

// EntityFluidInteraction.update; Entity.updateFluidInteraction
fn update_fluid(state: &mut PlayerState, world: &impl World) {
    let b=bounding_box(state, world).deflate(0.001);
    let lo=block_pos(b.min);
    let hi=[b.max.x.ceil() as i32-1, b.max.y.ceil() as i32-1, b.max.z.ceil() as i32-1];
    write_state!(world, state.water_height)=0.0;
    write_state!(world, state.lava_height)=0.0;
    write_state!(world, state.in_water)=false;
    write_state!(world, state.in_lava)=false;
    write_state!(world, state.eye_in_water)=false;
    write_state!(world, state.underwater)=false;
    world.observe_region(crate::footprint::Query::Fluid, lo, hi);
    if !world.has_fluid() {return;}
    // Level.hasChunksAt over the chunks of these columns, asked cell by cell: a chunk is
    // loaded exactly when its cells are, and the reads stay relative to the body.
    for z in lo[2]-1..=hi[2]+1 {
        for x in lo[0]-1..=hi[0]+1 {
            if !world.loaded([x, lo[1], z]) {
                return;
            }
        }
    }
    let mut heights=[0.0f64; 2];
    let mut eyes=[false; 2];
    let mut currents=[Vec3::default(); 2];
    let mut counts=[0; 2];
    let center=block_pos(read_state!(world, state.position));
    let eye_y=read_state!(world, state.position).y+read_state!(world, state.pose).eye_height() as f64;
    // Player.isPushedByFluid; EntityFluidInteraction.update skips current while flying.
    let pushed = !read_state!(world, state.flying);
    for x in lo[0]..=hi[0] {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                let f=world.fluid_block([x, y, z]).fluid;
                let a=match f.kind {
                    FluidKind::Empty=>continue,
                    FluidKind::Water=>0,
                    FluidKind::Lava=>1
                };
                let top=y as f64+f.surface_height(world, [x, y, z]) as f64;
                if top < b.min.y {
                    continue;
                }
                if x==center[0] && z==center[2] && eye_y>=y as f64 && eye_y<=top {
                    eyes[a]=true;
                }
                heights[a]=heights[a].max(top-bounding_box(state, world).min.y);
                if pushed {
                    let flow=f.current(world, [x, y, z]);
                    let flow=if heights[a]<0.4 { flow.scale(heights[a]) } else { flow };
                    currents[a]=currents[a].plus(flow);
                    counts[a]+=1;
                }
            }
        }
    }
    write_state!(world, state.water_height)=heights[0];
    write_state!(world, state.lava_height)=heights[1];
    write_state!(world, state.in_water)=heights[0]>0.0;
    write_state!(world, state.in_lava)=heights[1]>0.0;
    write_state!(world, state.eye_in_water)=eyes[0];
    write_state!(world, state.underwater)=read_state!(world, state.was_eye_in_water) && read_state!(world, state.in_water);
    if read_state!(world, state.in_water) {
        write_state!(world, state.fall_distance)=0.0;
    }
    for a in 0..2 {
        if counts[a]==0 || currents[a].length_sqr()<1.0e-5f32 as f64 {
            continue;
        }
        let scale=if a==0 {
            0.014
        } else if world.fast_lava() {
            0.007
        } else {
            0.0023333333333333335
        };
        let mut flow=currents[a].scale(1.0/counts[a] as f64).scale(scale);
        if read_state!(world, state.velocity).x.abs()<0.003 && read_state!(world, state.velocity).z.abs()<0.003 && flow.length_sqr().sqrt()<0.0045000000000000005 {
            flow=flow.normalize().scale(0.0045000000000000005);
        }
        write_state!(world, state.velocity)=read_state!(world, state.velocity).plus(flow);
    }
}

// CollisionGetter.containsAnyLiquid
fn contains_fluid(world: &impl World, b: Aabb) -> bool {
    let lo=block_pos(b.min);
    let hi=block_pos(b.max);
    for x in lo[0]..=hi[0] {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                if world.fluid_block([x, y, z]).fluid.kind!=FluidKind::Empty {
                    return true;
                }
            }
        }
    }
    false
}

// Entity.move; BlockGetter.clip
fn fall_reset_ray(state: &PlayerState, world: &impl World, movement: Vec3) -> bool {
    let end = read_state!(world, state.position).plus(movement.normalize().scale(movement.length_sqr().sqrt().min(8.0)));
    world.fall_reset_clip(read_state!(world, state.position), end)
}

// Entity.makeStuckInBlock; Player.makeStuckInBlock (@0-17: gated by !abilities.flying)
fn stuck(state: &mut PlayerState, world: &impl World, multiplier: Vec3) {
    if read_state!(world, state.flying) {
        return;
    }
    write_state!(world, state.fall_distance)=0.0;
    write_state!(world, state.stuck_speed_multiplier)=multiplier;
}

/// Runs the grounded stepOn phase on the already-moved player, before inside effects.
/// Resolve the returned request on the server resource timeline when predicting damage.
pub fn apply_grounded_step(state: &mut PlayerState, world: &impl World) -> Option<ContactDamage> {
    if read_state!(world, state.no_physics) || !read_state!(world, state.on_ground) {
        return None;
    }
    let block = world.hook_block(on_pos(state, world, 0.2));
    if block.slime && read_state!(world, state.velocity).y.abs()<0.1 && !read_state!(world, state.shift_key_down) {
        let f=0.4+read_state!(world, state.velocity).y.abs()*0.2;
        write_state!(world, state.velocity)=read_state!(world, state.velocity).multiply(Vec3::new(f, 1.0, f));
    }
    match block.grounded_contact {
        GroundedContact::DamageUnlessCareful(request) if !write_state!(world, state.shift_key_down) => Some(request),
        _ => None,
    }
}

fn fire_grace_tail(state: &mut PlayerState, world: &impl World, fire_before: i32) {
    if fire_before < 0 {
        let after = read_state!(world, state.remaining_fire_ticks);
        let updated = player::apply_effects_fire_tail(after, fire_before, true, false, true);
        if updated != after {
            set_remaining_fire_ticks(state, world, updated);
        }
    }
}

// Entity.applyEffectsFromBlocks; Entity.checkInsideBlocks
fn apply_block_effects(state: &mut PlayerState, world: &impl World, start: Vec3) -> Result<Option<ContactDamage>, Refusal> {
    if read_state!(world, state.no_physics) {
        return Ok(None);
    }
    let contact = apply_grounded_step(state, world);
    let fire_before = read_state!(world, state.remaining_fire_ticks);
    let swept = bounding_box(state, world).expand(start.minus(read_state!(world, state.position)));
    world.observe_region(crate::footprint::Query::Hooks, block_pos(swept.min), block_pos(swept.max));
    if !world.has_inside_effects() {
        fire_grace_tail(state, world, fire_before);
        return Ok(contact);
    }
    let delta=read_state!(world, state.position).minus(start);
    let mut visited=crate::blocks::SeenBlocks::new();
    let mut collector=StepBasedCollector::new();
    let mut inside_contact=None;
    let mut budget=16;
    let mut from=start;
    if let Some(original) = read_state!(world, state.movement_this_tick).filter(|_| delta.length_sqr()>0.0) {
        for axis in axis_step_order(original) {
            let distance=delta.get(axis);
            if distance==0.0 {
                continue;
            }
            let to=from.plus(Vec3::default().with(axis, distance));
            budget-=inside_segment(state, world, from, to, &mut visited, &mut collector, budget, delta, &mut inside_contact)?;
            from=to;
        }
    } else {
        budget-=inside_segment(state, world, start, read_state!(world, state.position), &mut visited, &mut collector, 16, delta, &mut inside_contact)?;
    }
    if budget<=0 {
        inside_segment(state, world, read_state!(world, state.position), read_state!(world, state.position), &mut visited, &mut collector, 1, delta, &mut inside_contact)?;
    }
    let collector_contact = apply_collector(state, world, &mut collector);
    fire_grace_tail(state, world, fire_before);
    Ok(contact.or(inside_contact).or(collector_contact))
}

fn set_remaining_fire_ticks(state: &mut PlayerState, world: &impl World, ticks: i32) {
    // Player.setRemainingFireTicks: creative/invulnerable caps fire duration at 1 tick.
    write_state!(world, state.remaining_fire_ticks) = if read_state!(world, state.invulnerable) {
        ticks.min(1)
    } else {
        ticks
    };
}

fn apply_collector(state: &mut PlayerState, world: &impl World, collector: &mut StepBasedCollector) -> Option<ContactDamage> {
    let mut contact = None;
    for op in collector.take_final_effects() {
        if read_state!(world, state.health) <= 0.0 {
            break;
        }
        match op {
            InsideEffectOp::ApplyType(InsideBlockEffectType::Freeze) => {
                write_state!(world, state.in_powder_snow) = true;
                if read_state!(world, state.game_mode) != GameType::Spectator
                    && !read_state!(world, state.leather_boots)
                {
                    write_state!(world, state.ticks_frozen) = (read_state!(world, state.ticks_frozen) + 1).min(140);
                }
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::ClearFreeze) => {
                write_state!(world, state.ticks_frozen) = 0;
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::FireIgnite) => {
                let next = player::base_fire_ignite_ticks(
                    read_state!(world, state.remaining_fire_ticks),
                    false,
                    true,
                );
                set_remaining_fire_ticks(state, world, next);
                if next >= 0 {
                    write_state!(world, state.ticks_frozen) = 0;
                }
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::LavaIgnite) => {
                let ticks = math::floor((player::LAVA_IGNITE_SECONDS * 20.0f32) as f64);
                if read_state!(world, state.remaining_fire_ticks) < ticks {
                    set_remaining_fire_ticks(state, world, ticks);
                }
                write_state!(world, state.ticks_frozen) = 0;
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::Extinguish) => {
                let extinguished = player::clear_fire_ticks(read_state!(world, state.remaining_fire_ticks));
                set_remaining_fire_ticks(state, world, extinguished);
            }
            InsideEffectOp::LavaHurt => {
                let cooldown = read_state!(world, state.hurt_cooldown);
                let mut entity = HurtEntityState {
                    health: read_state!(world, state.health),
                    invulnerable_time: cooldown.remaining_ticks,
                    last_hurt: cooldown.last_raw_hit,
                    hurt_duration: 0,
                    hurt_time: 0,
                    remaining_fire_ticks: read_state!(world, state.remaining_fire_ticks),
                    ticks_frozen: read_state!(world, state.ticks_frozen),
                    exhaustion: read_state!(world, state.exhaustion),
                    fire_immune: false,
                    abilities_invulnerable: read_state!(world, state.invulnerable),
                    has_fire_resistance: read_state!(world, state.fire_resistance),
                    armor: read_state!(world, state.armor),
                    armor_toughness: read_state!(world, state.armor_toughness),
                    protection_epf: read_state!(world, state.protection_epf),
                    feather_falling_epf: read_state!(world, state.feather_falling_epf),
                    resistance: read_state!(world, state.resistance),
                    absorption: read_state!(world, state.absorption),
                };
                if player::lava_hurt(&mut entity, true) {
                    write_state!(world, state.health) = entity.health.max(0.0);
                    write_state!(world, state.absorption) = entity.absorption;
                    write_state!(world, state.hurt_cooldown) = HurtCooldown {
                        remaining_ticks: entity.invulnerable_time,
                        last_raw_hit: entity.last_hurt,
                    };
                    write_state!(world, state.exhaustion) = entity.exhaustion;
                }
            }
            InsideEffectOp::FireHurt { soul } => {
                if contact.is_none() {
                    contact = Some(ContactDamage::InFire { soul });
                }
            }
            InsideEffectOp::Custom(_) => {}
        }
    }
    contact
}

// Entity.checkInsideBlocks; Entity.lambda$checkInsideBlocks$0
#[allow(clippy::too_many_arguments)]
fn inside_segment(state: &mut PlayerState, world: &impl World, from: Vec3, to: Vec3,
visited: &mut impl crate::blocks::BlockSet, collector: &mut StepBasedCollector, budget: i32,
delta: Vec3, damage: &mut Option<ContactDamage>) -> Result<i32, Refusal> {
    let bounds=Aabb::player(to, read_state!(world, state.pose).height()).deflate(1.0e-5f32 as f64);
    let large=from.minus(to).length_sqr()>(1.0-1.0e-5f32 as f64)*(1.0-1.0e-5f32 as f64);
    let body=Aabb::player(from, read_state!(world, state.pose).height());
    let movement=to.minus(from);
    let mut last_step=0;
    let mut refused=None;
    crate::blocks::for_each_intersected(from, to, bounds, |p, step| {
        if read_state!(world, state.health)<=0.0 || step>=budget {
            return false;
        }
        last_step=step;
        let block=world.hook_block(p);
        if block.air {
            return true;
        }
        let offset=Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
        let inside=block.inside_shape.as_ref().is_none_or(|shape| body.collided_along_view(movement, shape.at(offset)));
        let fluid_inside = block.fluid.kind != FluidKind::Empty && body.collided_along(movement,
            &VoxelShape::boxes(&[Aabb::new(offset.x, offset.y, offset.z,
                offset.x+1.0, (p[1] as f32+block.fluid.surface_height(world, p)) as f64, offset.z+1.0)]));
        if !(inside || fluid_inside) || !visited.insert(p) {
            return true;
        }
        if inside {
            // Entity.lambda$checkInsideBlocks$0:194-214: advance on the block branch.
            collector.advance_step(step);
            let cube=Aabb {
                min: offset, max: offset.plus(Vec3::new(1.0, 1.0, 1.0))
            };
            match inside_block(state, world, p, large||bounds.intersects(cube), collector, delta) {
                Ok(d) => {
                    if let Some(d) = d {
                        damage.get_or_insert(d);
                    }
                }
                Err(rule) => {
                    refused=Some(rule);
                    return false;
                }
            }
        }
        if fluid_inside {
            // Entity.lambda$checkInsideBlocks$0:287-313: the fluid branch advances independently.
            collector.advance_step(step);
            match block.fluid.kind {
                FluidKind::Water => collector.apply(InsideBlockEffectType::Extinguish),
                FluidKind::Lava => collector.record_lava_cell(),
                FluidKind::Empty => unreachable!(),
            }
        }
        true
    });
    match refused { Some(rule) => Err(rule), None => Ok(last_step+1) }
}

// WebBlock.entityInside; PowderSnowBlock.entityInside; HoneyBlock.entityInside;
// BubbleColumnBlock.entityInside; BaseFireBlock.entityInside; CampfireBlock.entityInside;
// SweetBerryBushBlock.entityInside; CactusBlock.entityInside
fn inside_block(
    state: &mut PlayerState,
    world: &impl World,
    p: BlockPos,
    intersects: bool,
    collector: &mut StepBasedCollector,
    delta: Vec3,
) -> Result<Option<ContactDamage>, Refusal> {
    let block=world.hook_block(p);
    if let Some(multiplier)=block.stuck_multiplier {
        stuck(state, world, multiplier);
    }
    let damage = match &block.inside {
        Inside::None=>None,
        Inside::Cobweb=>{
            let multiplier = if read_state!(world, state.weaving) {
                Vec3::new(0.5, 0.25, 0.5)
            } else {
                Vec3::new(0.25, 0.05f32 as f64, 0.25)
            };
            stuck(state, world, multiplier);
            None
        },
        Inside::PowderSnow=>{
            if world.hook_block(block_pos(read_state!(world, state.position))).inside==Inside::PowderSnow {
                stuck(state, world, Vec3::new(0.9f32 as f64, 1.5, 0.9f32 as f64));
            }
            collector.apply(InsideBlockEffectType::Freeze);
            collector.apply(InsideBlockEffectType::Extinguish);
            None
        },
        Inside::Honey=>{
            let old_y=read_state!(world, state.velocity).y/0.98f32 as f64+0.08;
            let edge=0.4375+(0.6f32/2.0f32) as f64;
            if !read_state!(world, state.on_ground) && read_state!(world, state.position).y<=p[1] as f64+0.9375-1.0e-7 && old_y < -0.08 &&
            ((p[0] as f64+0.5-read_state!(world, state.position).x).abs()+1.0e-7>edge || (p[2] as f64+0.5-read_state!(world, state.position).z).abs()+1.0e-7>edge) {
                if old_y < -0.13 {
                    let f=-0.05/old_y;
                    modify_state!(world, state.velocity).x*=f;
                    modify_state!(world, state.velocity).z*=f;
                }
                modify_state!(world, state.velocity).y=(-0.05-0.08)*0.98f32 as f64;
                write_state!(world, state.fall_distance)=0.0;
            }
            None
        },
        Inside::BubbleColumn { drag } => {
            if intersects && !read_state!(world, state.flying) {
                let drag = *drag;
                let above = world.block([p[0], p[1] + 1, p[2]]);
                let open_above = above.shape.is_empty() && above.fluid.kind == FluidKind::Empty;
                let vy = read_state!(world, state.velocity).y;
                if open_above {
                    write_state!(world, state.velocity).y = if drag {
                        (-0.9f64).max(vy - 0.03)
                    } else {
                        1.8f64.min(vy + 0.1)
                    };
                } else {
                    write_state!(world, state.velocity).y = if drag {
                        (-0.3f64).max(vy - 0.03)
                    } else {
                        0.7f64.min(vy + 0.06)
                    };
                    write_state!(world, state.fall_distance) = 0.0;
                }
            }
            None
        },
        Inside::Fire { soul } => {
            collector.record_fire_cell(*soul);
            None
        },
        Inside::Campfire { soul } => Some(ContactDamage::Campfire { soul: *soul }),
        Inside::BerryBush { hurt } => {
            stuck(state, world, Vec3::new(0.8f32 as f64, 0.75, 0.8f32 as f64));
            (*hurt && (delta.x.abs() >= 0.003f32 as f64 || delta.z.abs() >= 0.003f32 as f64))
                .then_some(ContactDamage::SweetBerryBush)
        },
        Inside::Cactus => Some(ContactDamage::Cactus),
        Inside::Extinguish => {
            collector.apply(InsideBlockEffectType::Extinguish);
            None
        },
        Inside::Lava => {
            collector.record_lava_cell();
            None
        },
        Inside::EndPortal => refuse!("EndPortalBlock/EndGatewayBlock.entityInside: setAsInsidePortal leaves the dimension"),
        Inside::NetherPortal => refuse!("NetherPortalBlock.entityInside: setAsInsidePortal enters portal"),
        Inside::WitherRose => {
            if world.difficulty() != 0 && !read_state!(world, state.invulnerable) {
                refuse!("WitherRoseBlock.entityInside: adds 40-tick WITHER effect (server, non-peaceful)");
            }
            None
        },
        Inside::Unsupported(name) => refuse!("BlockState.entityInside: {name}"),
    };
    Ok(damage)
}

// FoodData.addExhaustion
fn add_exhaustion(state: &mut PlayerState, world: &impl World, amount: f32) {
    write_state!(world, state.exhaustion)=(read_state!(world, state.exhaustion)+amount).min(40.0);
}

// Player.causeFoodExhaustion
fn cause_food_exhaustion(state: &mut PlayerState, world: &impl World, amount: f32) {
    if !read_state!(world, state.invulnerable) {
        add_exhaustion(state, world, amount);
    }
}

// ServerPlayer.checkMovementStatistics
fn movement_exhaustion(state: &mut PlayerState, world: &impl World, delta: Vec3) {
    if read_state!(world, state.riding) {
        return;
    }
    if read_state!(world, state.swimming) || read_state!(world, state.eye_in_water) || read_state!(world, state.in_water) {
        let distance=if read_state!(world, state.swimming) || read_state!(world, state.eye_in_water) {
            delta.length_sqr().sqrt()
        } else {
            delta.horizontal_sqr().sqrt()
        };
        let cm=math::round(distance as f32*100.0);
        if cm>0 {
            cause_food_exhaustion(state, world, 0.01f32*cm as f32*0.01f32);
        }
    } else if !read_state!(world, state.on_climbable) && read_state!(world, state.on_ground) && read_state!(world, state.sprinting) {
        let cm=math::round(delta.horizontal_sqr().sqrt() as f32*100.0);
        if cm>0 {
            cause_food_exhaustion(state, world, 0.1f32*cm as f32*0.01f32);
        }
    }
}

/// Advances resources for a living, unmounted server player with default max health.
/// Supply the server entity's tickCount at Player.aiStep, with preceding server
/// exhaustion and damage already applied. This covers ticks with no intervening
/// resource changes between Player.aiStep's regeneration and FoodData.tick.
pub fn tick_server_resources(state: &mut PlayerState, world: &impl World, server_player_tick: i32) {
    server_regeneration(state, world, server_player_tick);
    food_tick(state, world);
}

/// Ordered inputs at the authoritative server resource boundaries.
#[derive(Clone, Debug)]
pub enum ServerResourceEvent {
    /// A resolved LivingEntity.knockback call. `x`, `z` are the direction after the
    /// server's jitter loop (so x² + z² ≥ 1e-5f): the caller resolves it, since the law
    /// draws no randomness. `resistance` is the knockback-resistance attribute.
    Knockback { strength: f64, resistance: f64, x: f64, z: f64 },
    Exhaustion(f32),
    /// Observed health debit. Any observed exhaustion is supplied separately.
    Damage(f32),
    /// ServerPlayer.tick, once before the connection's ServerPlayer.doTick.
    HurtCooldownTick,
    /// LivingEntity.onBelowWorld after Entity.checkBelowWorld found
    /// `getY() < (minY - 64)`: the void's 4.0 hit after the cooldown tick. Bypasses
    /// armor, resistance, invulnerability (creative included) and knockback; exhaustion 0.0.
    FellOutOfWorld,
    /// Predict the contact debit and its exhaustion in place of the observed events.
    /// Requires normal survival, fire damage enabled, stock damage data and a
    /// loaded, non-invulnerable player without gear, effects or absorption.
    /// Anchor the cooldown and last raw hit before switching from observed damage.
    UnmodifiedContactDamage(ContactDamage),
    Regeneration(i32),
    FoodTick,
}

pub fn apply_server_resource_event(state: &mut PlayerState, world: &impl World, event: ServerResourceEvent) {
    match event {
        ServerResourceEvent::Knockback { strength, resistance, x, z } => knockback(state, world, strength, resistance, x, z),
        ServerResourceEvent::Exhaustion(amount) => add_exhaustion(state, world, amount),
        ServerResourceEvent::Damage(amount) => write_state!(world, state.health) = (read_state!(world, state.health) - amount).max(0.0),
        ServerResourceEvent::HurtCooldownTick => {
            if read_state!(world, state.hurt_cooldown).remaining_ticks > 0 {
                modify_state!(world, state.hurt_cooldown).remaining_ticks -= 1;
            }
        }
        ServerResourceEvent::FellOutOfWorld => {
            // Player.hurtServer's isDeadOrDying gate; out_of_world is in
            // bypasses_invulnerability/armor/resistance, so the raw hit is the debit.
            if read_state!(world, state.health) > 0.0
                && let Some(amount) = accept_raw_hit(&mut modify_state!(world, state.hurt_cooldown), crate::player::death::FELL_OUT_OF_WORLD)
            {
                write_state!(world, state.health) = (read_state!(world, state.health) - amount).max(0.0);
            }
        }
        ServerResourceEvent::UnmodifiedContactDamage(request) => {
            assert!(
                world.difficulty() == 2
                    && !read_state!(world, state.unmodeled_effects)
                    && !(read_state!(world, state.effects) && !has_modeled_effect(state, world)),
                "contact damage prediction requires unmodified normal survival"
            );
            apply_server_damage(
                state,
                world,
                request.raw_amount(),
                request.is_fire(),
                request.bypasses_armor(),
                false,
                request.is_fall(),
                request.exhaustion(),
            );
        }
        ServerResourceEvent::Regeneration(tick) => server_regeneration(state, world, tick),
        ServerResourceEvent::FoodTick => food_tick(state, world),
    }
}

// LivingEntity.knockback; the caller supplies the resolved direction.
fn knockback(state: &mut PlayerState, world: &impl World, strength: f64, resistance: f64, x: f64, z: f64) {
    let strength = strength * (1.0 - resistance);
    // dload; dconst_0; dcmpg; ifgt: a NaN strength does not return, as in vanilla.
    if strength <= 0.0 { return; }
    debug_assert!(!(x * x + z * z < 1.0e-5f32 as f64), "knockback direction must include server jitter");
    let impulse = Vec3::new(x, 0.0, z).normalize().scale(strength);
    let velocity = read_state!(world, state.velocity);
    let y = if read_state!(world, state.on_ground) {
        let y = velocity.y / 2.0 + strength;
        if y.is_nan() { y } else { 0.4f64.min(y) }
    } else { velocity.y };
    write_state!(world, state.velocity) = Vec3::new(velocity.x / 2.0 - impulse.x, y, velocity.z / 2.0 - impulse.z);
}

// Regression from the 26.2 bytecode (LivingEntity.knockback), not a server oracle:
// one case per branch. A JarOracle run for numeric oracle values is a GLM lane.
#[cfg(test)]
mod knockback_regression {
    use super::*;
    use crate::BoxWorld;
    fn hit(on_ground: bool, velocity: Vec3, strength: f64, resistance: f64, x: f64, z: f64) -> Vec3 {
        let mut state = PlayerState { on_ground, velocity, ..Default::default() };
        apply_server_resource_event(&mut state, &BoxWorld::default(), ServerResourceEvent::Knockback { strength, resistance, x, z });
        state.velocity
    }
    const S: f64 = 0.4f32 as f64;
    #[test]
    fn grounded_below_the_cap() {
        let v = hit(true, Vec3::new(0.3, -0.08, 0.2), S, 0.0, 1.0, 0.0);
        assert_eq!((v.x.to_bits(), v.y.to_bits(), v.z.to_bits()),
            ((0.3f64 / 2.0 - S).to_bits(), (-0.08f64 / 2.0 + S).to_bits(), (0.2f64 / 2.0).to_bits()));
    }
    #[test]
    fn grounded_at_the_cap() {
        assert_eq!(hit(true, Vec3::new(0.0, 0.2, 0.0), S, 0.0, 1.0, 0.0).y.to_bits(), 0.4f64.to_bits());
    }
    #[test]
    fn airborne_keeps_vertical() {
        assert_eq!(hit(false, Vec3::new(0.0, 0.3, 0.0), S, 0.0, 1.0, 0.0).y.to_bits(), 0.3f64.to_bits());
    }
    #[test]
    fn full_resistance_changes_nothing() {
        let v0 = Vec3::new(0.3, -0.08, 0.2);
        let v = hit(true, v0, S, 1.0, 1.0, 0.0);
        assert_eq!((v.x.to_bits(), v.y.to_bits(), v.z.to_bits()), (v0.x.to_bits(), v0.y.to_bits(), v0.z.to_bits()));
    }
    #[test]
    fn diagonal_direction_is_normalized() {
        let v = hit(false, Vec3::new(0.0, 0.0, 0.0), S, 0.0, 1.0, 1.0);
        let unit = 1.0f64 / 2.0f64.sqrt();
        assert_eq!((v.x.to_bits(), v.z.to_bits()), ((0.0 - unit * S).to_bits(), (0.0 - unit * S).to_bits()));
    }
    #[test]
    fn nan_strength_continues_as_in_vanilla() {
        assert!(hit(true, Vec3::new(0.0, 0.0, 0.0), f64::NAN, 0.0, 1.0, 0.0).x.is_nan());
    }
}

// LivingEntity.hurtServer's cooldown branch for an eligible, non-bypassing hit.
pub(crate) fn accept_raw_hit(cooldown: &mut HurtCooldown, amount: f32) -> Option<f32> {
    let debit = if cooldown.remaining_ticks > 10 {
        if amount <= cooldown.last_raw_hit {
            return None;
        }
        amount - cooldown.last_raw_hit
    } else {
        cooldown.remaining_ticks = 20;
        amount
    };
    cooldown.last_raw_hit = amount;
    Some(debit)
}

#[allow(clippy::too_many_arguments)]
fn apply_hit(
    state: &mut PlayerState,
    world: &impl World,
    amount: f32,
    bypasses_armor: bool,
    bypasses_resistance: bool,
    is_fall: bool,
    exhaustion: f32,
) {
    let mut health = read_state!(world, state.health);
    let mut absorption = read_state!(world, state.absorption);
    let mut cur_exhaustion = read_state!(world, state.exhaustion);
    player::actually_hurt(
        &mut health,
        &mut absorption,
        &mut cur_exhaustion,
        read_state!(world, state.invulnerable),
        amount,
        read_state!(world, state.armor),
        read_state!(world, state.armor_toughness),
        read_state!(world, state.resistance),
        read_state!(world, state.protection_epf),
        read_state!(world, state.feather_falling_epf),
        bypasses_armor,
        false,
        bypasses_resistance,
        false,
        is_fall,
        exhaustion,
    );
    write_state!(world, state.absorption) = absorption;
    if cur_exhaustion != read_state!(world, state.exhaustion) {
        write_state!(world, state.exhaustion) = cur_exhaustion;
    }
    write_state!(world, state.health) = health;
}

#[allow(clippy::too_many_arguments)]
fn apply_server_damage(
    state: &mut PlayerState,
    world: &impl World,
    mut raw_amount: f32,
    is_fire: bool,
    bypasses_armor: bool,
    bypasses_resistance: bool,
    is_fall: bool,
    exhaustion: f32,
) {
    if read_state!(world, state.invulnerable)
        || read_state!(world, state.health) <= 0.0
        || (is_fire && read_state!(world, state.fire_resistance))
    {
        return;
    }
    if raw_amount < 0.0f32 {
        raw_amount = 0.0f32;
    } else if !raw_amount.is_finite() {
        raw_amount = f32::MAX;
    }
    if let Some(debit) = accept_raw_hit(&mut modify_state!(world, state.hurt_cooldown), raw_amount) {
        apply_hit(state, world, debit, bypasses_armor, bypasses_resistance, is_fall, exhaustion);
    }
}

/// Server-side freeze recurrence after travel. The caller supplies the server's
/// counter and same-tick powder-snow flag; the returned damage is a request,
/// not a resolved health or cooldown outcome.
pub fn server_freeze_step(ticks_frozen: i32, in_powder_snow: bool, can_freeze: bool,
    feet_non_air: bool, tick_count: i32) -> (i32, Option<f64>, Option<f32>) {
    // LivingEntity.aiStep:697-722; Entity.baseTick:75-85.
    let frozen = if in_powder_snow && can_freeze { ticks_frozen } else { (ticks_frozen - 2).max(0) };
    // LivingEntity.removeFrost/tryAddFrost:1045-1096; Entity.getPercentFrozen:0-17.
    let frost = (feet_non_air && frozen > 0).then(|| (-0.05f32 * (frozen.min(140) as f32 / 140.0f32)) as f64);
    // LivingEntity.aiStep:733-768; Entity.isFullyFrozen:0-16.
    let damage = (tick_count % 40 == 0 && frozen >= 140 && can_freeze).then_some(1.0f32);
    (frozen, frost, damage)
}

fn server_regeneration(state: &mut PlayerState, world: &impl World, server_player_tick: i32) {
    // ServerPlayer.tickRegeneration, called by Player.aiStep before FoodData.tick.
    if world.difficulty() == 0 && world.natural_regeneration() {
        if server_player_tick % 20 == 0 {
            let health = read_state!(world, state.health);
            if health > 0.0 && health < 20.0 {
                write_state!(world, state.health) = (health + 1.0).min(20.0);
            }
            if read_state!(world, state.saturation) < 20.0 {
                modify_state!(world, state.saturation) += 1.0;
            }
        }
        if server_player_tick % 10 == 0 && read_state!(world, state.food_level) < 20 {
            modify_state!(world, state.food_level) += 1;
        }
    }
}

// FoodData.tick
fn food_tick(state: &mut PlayerState, world: &impl World) {
    if read_state!(world, state.exhaustion)>4.0 {
        modify_state!(world, state.exhaustion)-=4.0;
        if read_state!(world, state.saturation)>0.0 {
            write_state!(world, state.saturation)=(read_state!(world, state.saturation)-1.0).max(0.0);
        } else if world.difficulty()!=0 {
            write_state!(world, state.food_level)=(read_state!(world, state.food_level)-1).max(0);
        }
    }
    if world.natural_regeneration() && read_state!(world, state.saturation)>0.0 && read_state!(world, state.health)>0.0 && read_state!(world, state.health)<20.0 && read_state!(world, state.food_level)>=20 {
        modify_state!(world, state.food_tick_timer)+=1;
        if read_state!(world, state.food_tick_timer)>=10 {
            let amount=read_state!(world, state.saturation).min(6.0);
            write_state!(world, state.health)=(read_state!(world, state.health)+amount/6.0).min(20.0);
            add_exhaustion(state, world, amount);
            write_state!(world, state.food_tick_timer)=0;
        }
    } else if world.natural_regeneration() && read_state!(world, state.food_level)>=18 && read_state!(world, state.health)>0.0 && read_state!(world, state.health)<20.0 {
        modify_state!(world, state.food_tick_timer)+=1;
        if read_state!(world, state.food_tick_timer)>=80 {
            write_state!(world, state.health)=(read_state!(world, state.health)+1.0).min(20.0);
            add_exhaustion(state, world, 6.0);
            write_state!(world, state.food_tick_timer)=0;
        }
    } else if read_state!(world, state.food_level)<=0 {
        modify_state!(world, state.food_tick_timer)+=1;
        if read_state!(world, state.food_tick_timer)>=80 {
            if read_state!(world, state.health)>10.0 || world.difficulty()==3 || read_state!(world, state.health)>1.0 && world.difficulty()==2 {
                write_state!(world, state.health)=(read_state!(world, state.health)-1.0).max(0.0);
            }
            write_state!(world, state.food_tick_timer)=0;
        }
    } else {
        write_state!(world, state.food_tick_timer)=0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_resource_cadences_precede_food_data() {
        let resources = |s: &PlayerState| {
            (s.health, s.food_level, s.saturation, s.exhaustion, s.food_tick_timer)
        };
        for (difficulty, natural_regeneration, around_ten, around_twenty) in [
            (0, true,
                [(19.0, 19, 19.5, 0.0, 9), (20.0, 20, 19.5, 6.0, 0), (20.0, 20, 18.5, 2.0, 0)],
                [(19.0, 20, 20.0, 4.0, 9), (20.0, 20, 19.0, 0.25, 0), (20.0, 20, 19.0, 0.25, 0)]),
            (2, true,
                [(19.0, 19, 19.5, 0.0, 9), (19.0, 19, 19.5, 0.0, 10), (19.0, 19, 19.5, 0.0, 11)],
                [(19.0, 20, 20.0, 4.0, 9), (20.0, 20, 19.0, 6.25, 0), (20.0, 20, 18.0, 2.25, 0)]),
            (0, false,
                [(19.0, 19, 19.5, 0.0, 0); 3],
                [(19.0, 20, 20.0, 4.0, 0), (19.0, 20, 19.0, 0.25, 0), (19.0, 20, 19.0, 0.25, 0)]),
        ] {
            let mut world = crate::BoxWorld::default();
            world.difficulty = difficulty;
            world.natural_regeneration = natural_regeneration;
            let mut state = PlayerState {
                health: 19.0, food_level: 19, saturation: 19.5, food_tick_timer: 8,
                ..Default::default()
            };
            for (tick, expected) in (9..=11).zip(around_ten) {
                tick_server_resources(&mut state, &world, tick);
                assert_eq!(resources(&state), expected, "difficulty {difficulty}, regen {natural_regeneration}, tick {tick}");
            }

            let mut state = PlayerState {
                health: 19.0, saturation: 20.0, exhaustion: 4.0, food_tick_timer: 8,
                ..Default::default()
            };
            for (tick, expected) in (19..=21).zip(around_twenty) {
                if tick == 20 {
                    modify_state!(world, state.exhaustion) += 0.25;
                }
                tick_server_resources(&mut state, &world, tick);
                assert_eq!(resources(&state), expected, "difficulty {difficulty}, regen {natural_regeneration}, tick {tick}");
            }
        }

        let mut world = crate::BoxWorld::default();
        world.difficulty = 0;
        let mut state = PlayerState { saturation: 19.5, ..Default::default() };
        tick_server_resources(&mut state, &world, 20);
        assert_eq!(read_state!(world, state.saturation), 20.5);
    }

    #[test]
    fn jar_sprint_latch_corpus() {
        for row in include_str!("../tests/data/sprint.tsv").lines() {
            let (bits, expected) = row.split_once(' ').unwrap();
            let bits: u32 = bits.parse().unwrap();
            let mut state = PlayerState {
                sprinting: bits & 1 != 0,
                food_level: if bits & 4 != 0 { 20 } else { 6 },
                in_water: bits & 8 != 0,
                underwater: bits & 16 != 0,
                swimming: bits & 32 != 0,
                on_ground: bits & 64 != 0,
                horizontal_collision: bits & 256 != 0,
                minor_horizontal_collision: bits & 512 != 0,
                ..PlayerState::default()
            };
            let input = Input {
                forward: bits & 2 != 0,
                sneak: bits & 128 != 0,
                sprint: bits & 2048 != 0,
                ..Input::default()
            };
            update_sprinting(&mut state, &crate::BoxWorld::default(), &input, bits & 1024 != 0, false, 0.0);
            assert_eq!(state.sprinting, expected == "true", "jar sprint case {bits}");
        }
    }

    #[test]
    fn push_out_direction_order_and_suffocation() {
        let mut world = crate::BoxWorld::default();
        world.set([0, 0, 0], Block::full());
        let mut state = PlayerState {
            position: Vec3::new(0.5, 0.0, 0.5),
            velocity: Vec3::new(0.7, 0.2, 0.8),
            ..PlayerState::default()
        };
        move_towards_closest_space(&mut state, &world, 0.5, 0.5);
        assert_eq!(read_state!(world, state.velocity), Vec3::new(-0.1, 0.2, 0.8));
        world.set([-1, 0, 0], Block::full());
        move_towards_closest_space(&mut state, &world, 0.5, 0.5);
        assert_eq!(read_state!(world, state.velocity), Vec3::new(0.1, 0.2, 0.8));
        world.set([0, 0, 0], Block { suffocating: false, ..Block::full() });
        write_state!(world, state.velocity) = Vec3::default();
        move_towards_closest_space(&mut state, &world, 0.5, 0.5);
        assert_eq!(read_state!(world, state.velocity), Vec3::default());
    }

    #[test]
    fn fluid_only_intersection_is_visited_and_extinguishes() {
        let mut world = crate::BoxWorld::default();
        world.set([0, 0, 0], Block {
            air: false,
            inside_shape: Some(VoxelShape::default()),
            fluid: crate::Fluid { kind: FluidKind::Water, height: 1.0, ..crate::Fluid::default() },
            ..Block::default()
        });
        let mut state = PlayerState {
            position: Vec3::new(0.5, 0.0, 0.5),
            remaining_fire_ticks: 10,
            ..PlayerState::default()
        };
        let mut visited = std::collections::BTreeSet::new();
        let mut collector = StepBasedCollector::new();
        let position = read_state!(world, state.position);
        inside_segment(&mut state, &world, position, position, &mut visited, &mut collector, 16, Vec3::default(), &mut None).unwrap();
        apply_collector(&mut state, &world, &mut collector);
        assert!(visited.contains(&[0, 0, 0]));
        assert_eq!(read_state!(world, state.remaining_fire_ticks), 0);
        write_state!(world, state.remaining_fire_ticks) = 10;
        inside_segment(&mut state, &world, position, position, &mut visited, &mut collector, 16, Vec3::default(), &mut None).unwrap();
        apply_collector(&mut state, &world, &mut collector);
        assert_eq!(read_state!(world, state.remaining_fire_ticks), 10);
        visited.clear();
        world.set([0, 0, 0], Block {
            air: false, inside_shape: Some(VoxelShape::default()), ..Block::default()
        });
        inside_segment(&mut state, &world, position, position, &mut visited, &mut collector, 16, Vec3::default(), &mut None).unwrap();
        apply_collector(&mut state, &world, &mut collector);
        assert!(visited.is_empty());
    }
}

// Witness from local-20261111-1823-1-3-port-fluid-rules (port-fluid-rules).
// Jar anchors: Player.isPushedByFluid = !abilities.flying (Player.java.txt:4122,
// offsets 0-15); Entity.updateFluidInteraction (Entity.java.txt:4183) passes it as
// EntityFluidInteraction.update's skip-flow flag (offsets 6-17, consumed at offsets
// 359-402) and gates applyCurrentTo on it (offsets 73-77).
#[cfg(test)]
mod flying_current_witness {
    use super::*;
    use crate::{BoxWorld, Fluid};

    // Feet cell water[amount=7] (own height 7/9 >= 0.4), +X neighbour water[amount=4]
    // (own 4/9). FlowingFluid.getFlow: exactly one horizontal contributor, difference
    // 7/9-4/9 toward +X, final normalize gives (1,0,0); Tracker count 1, the Player
    // branch of Tracker.applyCurrentTo averages by count and scales by 0.014 (Entity
    // offsets 88-95); height 7/9 >= 0.4 so no 0.4 flow scale, and 0.014 >= 0.0045 so
    // the minimum-current floor does not engage. Vanilla delta-x = exactly 0.014.
    fn witness_world() -> BoxWorld {
        let water = |amount: u8| Block {
            fluid: Fluid {
                kind: FluidKind::Water,
                raw_amount: Some(amount),
                raw_falling: Some(false),
                ..Fluid::default()
            },
            ..Block::default()
        };
        let mut world = BoxWorld::default();
        world.set([0, 0, 0], water(7));
        world.set([1, 0, 0], water(4));
        world
    }

    fn witness_player(flying: bool) -> PlayerState {
        PlayerState { position: Vec3::new(0.5, 0.0, 0.5), flying, ..PlayerState::default() }
    }

    #[test]
    fn control_not_flying_takes_the_current() {
        let world = witness_world();
        let mut state = witness_player(false);
        update_fluid(&mut state, &world);
        assert_eq!(read_state!(world, state.velocity).x.to_bits(), 0.014f64.to_bits());
        assert!(read_state!(world, state.in_water));
    }

    #[test]
    fn flying_player_takes_no_current() {
        let world = witness_world();
        let mut state = witness_player(true);
        update_fluid(&mut state, &world);
        // The height half of contact is ungated in the jar: the eyes/height updates
        // (EntityFluidInteraction.java.txt offsets 305-356) run before the skip-flow
        // branch at offset 359, so in_water still updates while flying.
        assert!(read_state!(world, state.in_water));
        assert_eq!(
            read_state!(world, state.velocity).x.to_bits(),
            0.0f64.to_bits(),
            "jar applies no fluid current while abilities.flying (Player.isPushedByFluid)"
        );
    }
}
