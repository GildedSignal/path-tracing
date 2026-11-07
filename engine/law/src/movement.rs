use crate::{
    math,
    shapes::{
        collide_with_shapes,
        input_vector,
        axis_step_order
    },
    world::block_pos,
    Aabb,
    Block,
    BlockPos,
    CollisionContext,
    FluidKind,
    Input,
    Inside,
    PlayerState,
    Pose,
    Vec3,
    VoxelShape,
    World
};

// Player.tick; Entity.baseTick; LocalPlayer.aiStep; LivingEntity.aiStep
// The coupled food update estimates resources from local movement without a server schedule.
pub fn tick(state: &mut PlayerState, world: &impl World, input: &Input) {
    tick_with_collision(state, world, input);
}

/// Runs the same tick and returns the request and result at Entity.collide.
pub fn tick_with_collision(
    state: &mut PlayerState, world: &impl World, input: &Input,
) -> Option<(Vec3, Vec3)> {
    if state.effects {
        unimplemented!("LivingEntity: status effects other than none");
    }
    if state.boat {
        unimplemented!("Boat movement");
    }
    if state.riding {
        unimplemented!("LivingEntity.travelRidden");
    }
    if state.fall_flying {
        unimplemented!("LivingEntity.travelFallFlying; Math.cos");
    }
    if state.flying {
        unimplemented!("Player.travel: abilities.flying");
    }
    if state.using_item {
        unimplemented!("LocalPlayer.itemUseSpeedMultiplier");
    }
    let start = state.position;
    state.movement_this_tick = None;
    state.yaw = input.yaw;
    state.pitch = input.pitch;
    state.was_in_powder_snow = state.in_powder_snow;
    state.in_powder_snow = false;
    state.was_eye_in_water = state.eye_in_water;
    update_fluid(state, world);
    if state.in_lava { state.fall_distance *= 0.5; }
    state.swimming = if state.swimming {
        state.sprinting && state.in_water
    } else {
        state.sprinting && state.underwater && world.block(block_pos(state.position)).fluid.kind == FluidKind::Water
    };
    state.crouching = !state.swimming && fits(state, world, Pose::Crouching) &&
    (state.shift_key_down || !fits(state, world, Pose::Standing));
    state.shift_key_down = input.sneak;
    state.input = *input;
    if !state.no_physics {
        let offset = 0.6f32 as f64 * 0.35;
        for (x, z) in [(-offset, offset), (-offset, -offset), (offset, -offset), (offset, offset)] {
            move_towards_closest_space(state, world, state.position.x+x, state.position.z+z);
        }
    }
    let slowly = state.crouching || (state.pose == Pose::Swimming && !state.in_water);
    update_sprinting(state, input, slowly);
    // Player.getSpeed reads the attribute after LocalPlayer updates the sprint modifier.
    state.speed = state.movement_speed_value() as f32;
    if state.in_water && input.sneak {
        state.velocity.y -= 0.04f32 as f64;
    }
    if state.no_jump_delay > 0 {
        state.no_jump_delay -= 1;
    }
    if state.velocity.horizontal_sqr() < 9.0e-6 {
        state.velocity.x = 0.0;
        state.velocity.z = 0.0;
    }
    if state.velocity.y.abs() < 0.003 {
        state.velocity.y = 0.0;
    }
    let movement = modify_input(input, slowly, state.sneaking_speed);
    state.on_climbable = climbable(state, world);
    if input.jump && state.health > 0.0 {
        let height = if state.in_lava {
            state.lava_height
        } else {
            state.water_height
        };
        let water = state.in_water && height > 0.0;
        if water && (!state.on_ground || height > 0.4) || state.in_lava && (!state.on_ground || state.lava_height > 0.4) {
            state.velocity.y += 0.04f32 as f64;
        } else if (state.on_ground || water && height <= 0.4) && state.no_jump_delay == 0 {
            jump_from_ground(state, world);
            state.no_jump_delay = 10;
        }
    } else {
        state.no_jump_delay = 0;
    }
    let movement = if state.health <= 0.0 {
        Vec3::default()
    } else {
        movement
    };
    let collision = travel(state, world, movement);
    apply_block_effects(state, world, start);
    // InsideBlockEffectType.FREEZE
    if state.in_powder_snow && !state.leather_boots {
        state.ticks_frozen = (state.ticks_frozen + 1).min(140);
    }
    state.position.x = state.position.x.clamp(-29999999.0, 29999999.0);
    state.position.z = state.position.z.clamp(-29999999.0, 29999999.0);
    update_pose(state, world);
    state.on_climbable = climbable(state, world);
    movement_exhaustion(state, state.position.minus(start));
    food_tick(state, world);
    state.t += 1;
    collision
}

// LocalPlayer.aiStep; LocalPlayer.canStartSprinting; LocalPlayer.shouldStopRunSprinting; LocalPlayer.shouldStopSwimSprinting
fn update_sprinting(state: &mut PlayerState, input: &Input, slowly: bool) {
    let forward = input.forward && !input.back;
    let possible = state.food_level > 6 && (state.flying || !state.in_water || state.underwater);
    if !state.sprinting && forward && possible && (!slowly || state.underwater) && input.sprint {
        state.sprinting = true;
    }
    if state.sprinting {
        let stop = if state.swimming {
            state.food_level <= 6 || !state.in_water || (!forward && !state.on_ground && !input.sneak)
        } else {
            !possible || !forward || (state.horizontal_collision && !state.minor_horizontal_collision)
        };
        if stop {
            state.sprinting = false;
        }
    }
}

// LocalPlayer.suffocatesAt
fn suffocates_at(state: &PlayerState, world: &impl World, p: BlockPos) -> bool {
    let body = state.bounding_box();
    let bounds = Aabb::new(p[0] as f64, body.min.y, p[2] as f64,
        p[0] as f64+1.0, body.max.y, p[2] as f64+1.0).deflate(1.0e-7);
    world.collides_with_suffocating_block(bounds, CollisionContext::of(state))
}

// LocalPlayer.moveTowardsClosestSpace
fn move_towards_closest_space(state: &mut PlayerState, world: &impl World, x: f64, z: f64) {
    let p = block_pos(Vec3::new(x, state.position.y, z));
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
        state.velocity = state.velocity.with(axis, 0.1*step as f64);
    }
}

// KeyboardInput.tick; LocalPlayer.modifyInput; LocalPlayer.modifyInputSpeedForSquareMovement
fn modify_input(i: &Input, slowly: bool, sneaking_speed: f32) -> Vec3 {
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
fn flying_speed(state: &PlayerState) -> f32 {
    if state.sprinting {
        0.025999999
    } else {
        0.02
    }
}

// LivingEntity.getFrictionInfluencedSpeed
fn friction_speed(state: &PlayerState, f: f32) -> f32 {
    if state.on_ground {
        if f as f64 > 0.6 {
            state.speed * (0.21600002f32 / (f*f*f))
        } else {
            state.speed
        }
    } else {
        flying_speed(state)
    }
}

// Entity.getOnPos
fn on_pos(state: &PlayerState, world: &impl World, offset: f32) -> BlockPos {
    if let Some(p) = state.main_supporting_block {
        if offset <= 1.0e-5f32 {
            return p;
        }
        let b = world.block(p);
        if offset as f64 <= 0.5 && b.fence || b.wall_or_gate {
            return p;
        }
        return [p[0],
        math::floor(state.position.y-offset as f64),
        p[2]];
    }
    block_pos(Vec3::new(state.position.x, state.position.y-offset as f64, state.position.z))
}

// Entity.getBlockJumpFactor
fn jump_factor(state: &PlayerState, world: &impl World) -> f32 {
    let here = world.block(block_pos(state.position)).jump_factor;
    let below = world.block(on_pos(state, world, 0.500001)).jump_factor;
    if here as f64 == 1.0 {
        below
    } else {
        here
    }
}

// Entity.getBlockSpeedFactor
fn speed_factor(state: &PlayerState, world: &impl World) -> f32 {
    let b = world.block(block_pos(state.position));
    if b.water_block {
        return b.speed_factor;
    }
    if b.speed_factor as f64 == 1.0 {
        world.block(on_pos(state, world, 0.500001)).speed_factor
    } else {
        b.speed_factor
    }
}

// LivingEntity.getJumpPower; LivingEntity.jumpFromGround; ServerPlayer.jumpFromGround
fn jump_from_ground(state: &mut PlayerState, world: &impl World) {
    let power = state.jump_strength as f32 * 1.0f32 * jump_factor(state, world) + 0.0f32;
    if power > 1.0e-5f32 {
        state.velocity.y = (power as f64).max(state.velocity.y);
        if state.sprinting {
            let yaw = (state.yaw * 0.017453292f32) as f64;
            state.velocity = state.velocity.plus(Vec3::new((-math::sin(yaw)) as f64 * 0.2, 0.0, math::cos(yaw) as f64*0.2));
        }
    }
    add_exhaustion(state, if state.sprinting {
        0.2
    } else {
        0.05
    });
}

// LivingEntity.onClimbable; LivingEntity.trapdoorUsableAsLadder
fn climbable(state: &PlayerState, world: &impl World) -> bool {
    let p = block_pos(state.position);
    let b = world.block(p);
    b.climbable || b.open_trapdoor_facing.is_some_and(|f| world.block([p[0], p[1]-1, p[2]]).ladder_facing == Some(f))
}

// LivingEntity.handleOnClimbable
fn handle_climbable(state: &mut PlayerState, world: &impl World) {
    if climbable(state, world) {
        state.fall_distance = 0.0;
        state.velocity.x = state.velocity.x.clamp(-0.15f32 as f64, 0.15f32 as f64);
        state.velocity.z = state.velocity.z.clamp(-0.15f32 as f64, 0.15f32 as f64);
        state.velocity.y = state.velocity.y.max(-0.15f32 as f64);
        if state.velocity.y < 0.0 && !world.block(block_pos(state.position)).scaffolding && state.shift_key_down {
            state.velocity.y = 0.0;
        }
    }
}

// Player.travel; LivingEntity.travel
fn travel(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Option<(Vec3, Vec3)> {
    if state.swimming {
        let pitch = (state.pitch * 0.017453292f32) as f64;
        let look_y = -math::sin(pitch) as f64;
        let factor = if look_y < -0.2 {
            0.085
        } else {
            0.06
        };
        if look_y <= 0.0 || state.input.jump || world.block(block_pos(Vec3::new(state.position.x, state.position.y+1.0-0.1, state.position.z))).fluid.kind != FluidKind::Empty {
            state.velocity.y += (look_y - state.velocity.y) * factor;
        }
    }
    if state.in_water || state.in_lava {
        travel_fluid(state, world, movement)
    } else {
        travel_air(state, world, movement)
    }
}

// LivingEntity.travelInAir; LivingEntity.handleRelativeFrictionAndCalculateMovement
fn travel_air(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Option<(Vec3, Vec3)> {
    let below = on_pos(state, world, 0.500001);
    let f = if state.on_ground {
        math::modified_friction(world.block(below).friction, state.friction_modifier)
    } else {
        1.0
    };
    state.velocity = state.velocity.plus(input_vector(movement, friction_speed(state, f), state.yaw));
    handle_climbable(state, world);
    let collision = move_player(state, world, state.velocity);
    let mut velocity = state.velocity;
    if (state.horizontal_collision || state.input.jump) && (climbable(state, world) || state.was_in_powder_snow && state.leather_boots) {
        velocity.y = 0.2;
    }
    let vy = if world.loaded(below) {
        velocity.y - state.gravity
    } else if state.position.y > world.min_y() as f64 {
        -0.1
    } else {
        0.0
    };
    let drag = math::modified_friction(0.91, state.air_drag_modifier);
    let horizontal = f*drag;
    let vertical = math::modified_friction(0.98, state.air_drag_modifier);
    state.velocity = Vec3::new(velocity.x*horizontal as f64, vy*vertical as f64, velocity.z*horizontal as f64);
    collision
}

// LivingEntity.travelInFluid; LivingEntity.travelInWater; LivingEntity.travelInLava
fn travel_fluid(state: &mut PlayerState, world: &impl World, movement: Vec3) -> Option<(Vec3, Vec3)> {
    let falling = state.velocity.y <= 0.0;
    let old_y = state.position.y;
    let collision = if state.in_water {
        let mut drag = if state.sprinting {
            0.9f32
        } else {
            0.8f32
        };
        let mut acceleration = 0.02f32;
        let efficiency = state.water_movement_efficiency * if state.on_ground {
            1.0
        } else {
            0.5
        };
        if efficiency > 0.0 {
            drag += (0.54600006f32-drag)*efficiency;
            acceleration += (state.speed-acceleration)*efficiency;
        }
        state.velocity = state.velocity.plus(input_vector(movement, acceleration, state.yaw));
        let collision = move_player(state, world, state.velocity);
        if state.horizontal_collision && climbable(state, world) {
            state.velocity.y = 0.2;
        }
        state.velocity = state.velocity.multiply(Vec3::new(drag as f64, 0.8f32 as f64, drag as f64));
        fluid_falling(state, falling);
        collision
    } else {
        state.velocity = state.velocity.plus(input_vector(movement, 0.02, state.yaw));
        let collision = move_player(state, world, state.velocity);
        if state.lava_height <= 0.4 {
            state.velocity = state.velocity.multiply(Vec3::new(0.5, 0.8f32 as f64, 0.5));
            fluid_falling(state, falling);
        } else {
            state.velocity = state.velocity.scale(0.5);
        }
        if state.gravity != 0.0 {
            state.velocity.y += -state.gravity/4.0;
        }
        collision
    };
    let v = state.velocity;
    let jump_box = state.bounding_box().moved(Vec3::new(v.x, v.y+0.6f32 as f64-state.position.y+old_y, v.z));
    if state.horizontal_collision && world.no_collision(jump_box, CollisionContext::of(state)) && !contains_fluid(world, jump_box) {
        state.velocity.y = 0.3f32 as f64;
    }
    collision
}

// LivingEntity.getFluidFallingAdjustedMovement
fn fluid_falling(state: &mut PlayerState, falling: bool) {
    if state.gravity != 0.0 && !state.sprinting {
        state.velocity.y = if falling && (state.velocity.y-0.005).abs() >= 0.003 && (state.velocity.y-state.gravity/16.0).abs() < 0.003 {
            -0.003
        } else {
            state.velocity.y-state.gravity/16.0
        };
    }
}

// Player.canPlayerFitWithinBlocksAndEntitiesWhen
fn fits(state: &PlayerState, world: &impl World, pose: Pose) -> bool {
    world.no_collision(Aabb::player(state.position, pose.height()).deflate(1.0e-7), CollisionContext::of(state))
}

// Player.updatePlayerPose; Player.getDesiredPose
fn update_pose(state: &mut PlayerState, world: &impl World) {
    if !fits(state, world, Pose::Swimming) {
        return;
    }
    let desired = if state.swimming {
        Pose::Swimming
    } else if state.shift_key_down {
        Pose::Crouching
    } else {
        Pose::Standing
    };
    state.pose = if fits(state, world, desired) {
        desired
    } else if fits(state, world, Pose::Crouching) {
        Pose::Crouching
    } else {
        Pose::Swimming
    };
}

// Player.canFallAtLeast
fn can_fall(state: &PlayerState, world: &impl World, x: f64, z: f64, depth: f64) -> bool {
    let b = state.bounding_box();
    world.no_collision(Aabb::new(b.min.x+1.0e-7+x, b.min.y-depth-1.0e-7, b.min.z+1.0e-7+z,
    b.max.x-1.0e-7+x, b.min.y, b.max.z-1.0e-7+z), CollisionContext::of(state))
}

// Player.maybeBackOffFromEdge; Player.isAboveGround
fn back_off(state: &PlayerState, world: &impl World, mut v: Vec3) -> Vec3 {
    let step = state.step_height;
    if v.y > 0.0 || !state.shift_key_down || !(state.on_ground || state.fall_distance < step as f64 && !can_fall(state, world, 0.0, 0.0, step as f64-state.fall_distance)) {
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
fn collide(state: &PlayerState, world: &impl World, v: Vec3) -> Vec3 {
    let b = state.bounding_box();
    let context = CollisionContext::of(state);
    let shapes = world.collisions(b.expand(v), context);
    let result = if v.length_sqr() == 0.0 {
        v
    } else {
        collide_with_shapes(v, b, &shapes)
    };
    let landed = v.y != result.y && v.y < 0.0;
    let height = if cfg!(feature="mutant-step-up") {
        f32::from_bits(state.step_height.to_bits()+1)
    } else {
        state.step_height
    };
    if height > 0.0 && (landed || state.on_ground) && (v.x != result.x || v.z != result.z) {
        let base = if landed {
            b.moved(Vec3::new(0.0, result.y, 0.0))
        } else {
            b
        };
        let mut bounds = base.expand(Vec3::new(v.x, height as f64, v.z));
        if !landed {
            bounds = bounds.expand(Vec3::new(0.0, -1.0e-5f32 as f64, 0.0));
        }
        let shapes = world.collisions(bounds, context);
        let previous_y = result.y as f32;
        let mut candidates = Vec::new();
        for shape in &shapes {
            for y in &shape.coords[1] {
                let candidate = (y-base.min.y) as f32;
                if candidate < 0.0 || candidate == previous_y {
                    continue;
                }
                if candidate > height {
                    break;
                }
                if !candidates.contains(&candidate) {
                    candidates.push(candidate);
                }
            }
        }
        candidates.sort_by(f32::total_cmp);
        for candidate in candidates {
            let stepped = collide_with_shapes(Vec3::new(v.x, candidate as f64, v.z), base, &shapes);
            if stepped.horizontal_sqr() > result.horizontal_sqr() {
                return stepped.minus(Vec3::new(0.0, b.min.y-base.min.y, 0.0));
            }
        }
    }
    result
}

// CollisionGetter.findSupportingBlock; Entity.checkSupportingBlock
fn supporting_block(state: &mut PlayerState, world: &impl World, moved: Vec3) {
    if !state.on_ground {
        state.on_ground_no_blocks=false;
        state.main_supporting_block=None;
        return;
    }
    let b = state.bounding_box();
    let foot = Aabb::new(b.min.x, b.min.y-1.0e-6, b.min.z, b.max.x, b.min.y, b.max.z);
    let find = |bounds: Aabb| {
        let lo = block_pos(bounds.min);
        let hi = block_pos(bounds.max);
        let mut best: Option<(f64, BlockPos)> = None;
        for x in lo[0]-1..=hi[0]+1 {
            for y in lo[1]-1..=hi[1]+1 {
                for z in lo[2]-1..=hi[2]+1 {
                    let p=[x, y, z];
                    if !world.collision_shape(p, CollisionContext::of(state)).moved(Vec3::new(x as f64, y as f64, z as f64)).intersects(bounds) {
                        continue;
                    }
                    let d=Vec3::new(x as f64+0.5, y as f64+0.5, z as f64+0.5).minus(state.position).length_sqr();
                    if best.is_none_or(|(old, q)| d < old || d == old && (y, z, x) > (q[1], q[2], q[0])) {
                        best=Some((d, p));
                    }
                }
            }
        }
        best.map(|(_, p)|p)
    };
    let found = find(foot);
    if found.is_some() || state.on_ground_no_blocks {
        state.main_supporting_block=found;
    } else {
        state.main_supporting_block=find(foot.moved(Vec3::new(-moved.x, 0.0, -moved.z)));
    }
    state.on_ground_no_blocks = state.main_supporting_block.is_none();
}

// Entity.move; Entity.restituteMovementAfterCollisions
fn move_player(state: &mut PlayerState, world: &impl World, mut wanted: Vec3) -> Option<(Vec3, Vec3)> {
    if state.no_physics {
        state.position = state.position.plus(wanted);
        return None;
    }
    if state.stuck_speed_multiplier.length_sqr() > 1.0e-7 {
        wanted = wanted.multiply(state.stuck_speed_multiplier);
        state.stuck_speed_multiplier=Vec3::default();
        state.velocity=Vec3::default();
    }
    wanted = back_off(state, world, wanted);
    let moved=collide(state, world, wanted);
    let distance=moved.length_sqr();
    if distance > 1.0e-7 || wanted.length_sqr()-distance < 1.0e-7 {
        if state.fall_distance != 0.0 && distance >= 1.0 && fall_reset_ray(state, world, moved) {
            state.fall_distance=0.0;
        }
        state.movement_this_tick=Some(wanted);
        state.position=state.position.plus(moved);
    }
    let hit_x=!math::equal(wanted.x, moved.x);
    let hit_z=!math::equal(wanted.z, moved.z);
    state.horizontal_collision=hit_x||hit_z;
    state.vertical_collision=wanted.y!=moved.y;
    state.vertical_collision_below=state.vertical_collision&&wanted.y<0.0;
    state.on_ground=state.vertical_collision_below;
    supporting_block(state, world, moved);
    state.minor_horizontal_collision=minor_collision(state, moved);
    if !state.in_water {
        update_fluid(state, world);
    }
    let block=world.block(on_pos(state, world, 0.2));
    check_fall_damage(state, block, moved.y);
    if wanted.y.abs()>0.0 && state.vertical_collision || state.horizontal_collision {
        let mut bounce=if state.shift_key_down {
            0.0
        } else {
            state.bounciness
        };
        let before=state.velocity;
        if hit_x {
            state.velocity.x=-before.x*bounce;
        }
        if hit_z {
            state.velocity.z=-before.z*bounce;
        }
        if state.vertical_collision {
            if state.vertical_collision_below {
                bounce=if -before.y < state.gravity || state.shift_key_down || block.suppresses_bounce {
                    0.0
                } else {
                    bounce.max(block.bounce as f64)
                };
            }
            let (gravity, drag)=if bounce>0.0 {
                let fraction=moved.y/before.y;
                (fraction*state.gravity, 1.0+fraction*(math::modified_friction(0.98, state.air_drag_modifier) as f64-1.0))
            } else {
                (0.0, 1.0)
            };
            if !(cfg!(feature="mutant-ceiling") && wanted.y>0.0) {
                state.velocity.y=(gravity-before.y)*drag*bounce;
            }
        }
    }
    let factor=speed_factor(state, world);
    let factor=(factor+state.movement_efficiency*(1.0f32-factor)) as f64;
    state.velocity=state.velocity.multiply(Vec3::new(factor, 1.0, factor));
    Some((wanted, moved))
}

// LocalPlayer.isHorizontalCollisionMinor
fn minor_collision(state: &PlayerState, moved: Vec3) -> bool {
    if !state.horizontal_collision {
        return false;
    }
    let input=modify_input(&state.input, state.crouching||(state.pose==Pose::Swimming&&!state.in_water), state.sneaking_speed);
    let yaw = (state.yaw * 0.017453292f32) as f64;
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
fn check_fall_damage(state: &mut PlayerState, block: &Block, dy: f64) {
    if !state.in_water && dy<0.0 {
        state.fall_distance -= (dy as f32) as f64;
    }
    if state.on_ground {
        if state.fall_distance>0.0 {
            if let Some(name) = &block.unsupported_fall_on {
                unimplemented!("BlockState.fallOn: {name}");
            }
            let multiplier=if block.slime || block.inside==Inside::PowderSnow {
                0.0
            } else {
                block.fall_on_multiplier
            };
            let distance = if block.bed { state.fall_distance*0.5 } else { state.fall_distance };
            let damage=calculate_fall_damage(state, distance, multiplier).max(0);
            if damage>0 {
                state.health=(state.health-damage as f32).max(0.0);
            }
        }
        state.fall_distance=0.0;
    }
}

// EntityFluidInteraction.update; Entity.updateFluidInteraction
fn update_fluid(state: &mut PlayerState, world: &impl World) {
    let b=state.bounding_box().deflate(0.001);
    let lo=block_pos(b.min);
    let hi=[b.max.x.ceil() as i32-1, b.max.y.ceil() as i32-1, b.max.z.ceil() as i32-1];
    state.water_height=0.0;
    state.lava_height=0.0;
    state.in_water=false;
    state.in_lava=false;
    state.eye_in_water=false;
    state.underwater=false;
    for z in ((lo[2]-1)>>4)..=((hi[2]+1)>>4) {
        for x in ((lo[0]-1)>>4)..=((hi[0]+1)>>4) {
            if !world.loaded([x*16, lo[1], z*16]) {
                return;
            }
        }
    }
    let mut heights=[0.0f64; 2];
    let mut eyes=[false; 2];
    let mut currents=[Vec3::default(); 2];
    let mut counts=[0; 2];
    let center=block_pos(state.position);
    let eye_y=state.position.y+state.pose.eye_height() as f64;
    for x in lo[0]..=hi[0] {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                let f=world.block([x, y, z]).fluid;
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
                heights[a]=heights[a].max(top-state.bounding_box().min.y);
                let flow=if heights[a]<0.4 {
                    f.flow.scale(heights[a])
                } else {
                    f.flow
                };
                currents[a]=currents[a].plus(flow);
                counts[a]+=1;
            }
        }
    }
    state.water_height=heights[0];
    state.lava_height=heights[1];
    state.in_water=heights[0]>0.0;
    state.in_lava=heights[1]>0.0;
    state.eye_in_water=eyes[0];
    state.underwater=state.was_eye_in_water && state.in_water;
    if state.in_water {
        state.fall_distance=0.0;
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
        if state.velocity.x.abs()<0.003 && state.velocity.z.abs()<0.003 && flow.length_sqr().sqrt()<0.0045000000000000005 {
            flow=flow.normalize().scale(0.0045000000000000005);
        }
        state.velocity=state.velocity.plus(flow);
    }
}

// CollisionGetter.containsAnyLiquid
fn contains_fluid(world: &impl World, b: Aabb) -> bool {
    let lo=block_pos(b.min);
    let hi=block_pos(b.max);
    for x in lo[0]..=hi[0] {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                if world.block([x, y, z]).fluid.kind!=FluidKind::Empty {
                    return true;
                }
            }
        }
    }
    false
}

// Entity.move; BlockGetter.clip
fn fall_reset_ray(state: &PlayerState, world: &impl World, movement: Vec3) -> bool {
    let end = state.position.plus(movement.normalize().scale(movement.length_sqr().sqrt().min(8.0)));
    world.fall_reset_clip(state.position, end)
}

// Entity.makeStuckInBlock; Player.makeStuckInBlock
fn stuck(state: &mut PlayerState, multiplier: Vec3) {
    state.fall_distance=0.0;
    state.stuck_speed_multiplier=multiplier;
}

// Entity.applyEffectsFromBlocks; Entity.checkInsideBlocks
fn apply_block_effects(state: &mut PlayerState, world: &impl World, start: Vec3) {
    if state.no_physics {
        return;
    }
    if state.on_ground {
        let block=world.block(on_pos(state, world, 0.2));
        if block.slime && state.velocity.y.abs()<0.1 && !state.shift_key_down {
            let f=0.4+state.velocity.y.abs()*0.2;
            state.velocity=state.velocity.multiply(Vec3::new(f, 1.0, f));
        }
    }
    let delta=state.position.minus(start);
    let mut visited=std::collections::BTreeSet::new();
    let mut budget=16;
    let mut from=start;
    if let Some(original) = state.movement_this_tick.filter(|_| delta.length_sqr()>0.0) {
        for axis in axis_step_order(original) {
            let distance=delta.get(axis);
            if distance==0.0 {
                continue;
            }
            let to=from.plus(Vec3::default().with(axis, distance));
            budget-=inside_segment(state, world, from, to, &mut visited, budget);
            from=to;
        }
    } else {
        budget-=inside_segment(state, world, start, state.position, &mut visited, 16);
    }
    if budget<=0 {
        inside_segment(state, world, state.position, state.position, &mut visited, 1);
    }
}

// Entity.checkInsideBlocks; Entity.lambda$checkInsideBlocks$0
fn inside_segment(state: &mut PlayerState, world: &impl World, from: Vec3, to: Vec3,
visited: &mut std::collections::BTreeSet<BlockPos>, budget: i32) -> i32 {
    let bounds=Aabb::player(to, state.pose.height()).deflate(1.0e-5f32 as f64);
    let large=from.minus(to).length_sqr()>(1.0-1.0e-5f32 as f64)*(1.0-1.0e-5f32 as f64);
    let body=Aabb::player(from, state.pose.height());
    let movement=to.minus(from);
    let mut last_step=0;
    crate::blocks::for_each_intersected(from, to, bounds, |p, step| {
        if state.health<=0.0 || step>=budget {
            return false;
        }
        last_step=step;
        let block=world.block(p);
        if block.air {
            return true;
        }
        let offset=Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
        let inside=block.inside_shape.as_ref().is_none_or(|shape| body.collided_along(movement, &shape.moved(offset)));
        let fluid_inside = block.fluid.kind != FluidKind::Empty && body.collided_along(movement,
            &VoxelShape::boxes(&[Aabb::new(offset.x, offset.y, offset.z,
                offset.x+1.0, (p[1] as f32+block.fluid.surface_height(world, p)) as f64, offset.z+1.0)]));
        if !(inside || fluid_inside) || !visited.insert(p) {
            return true;
        }
        if inside {
            let cube=Aabb {
                min: offset, max: offset.plus(Vec3::new(1.0, 1.0, 1.0))
            };
            inside_block(state, world, p, large||bounds.intersects(cube));
        }
        if fluid_inside {
            match block.fluid.kind {
                FluidKind::Water => state.remaining_fire_ticks = state.remaining_fire_ticks.min(0),
                FluidKind::Lava => unimplemented!("FluidState.entityInside: LavaFluid CLEAR_FREEZE, LAVA_IGNITE and deferred lava damage"),
                FluidKind::Empty => unreachable!(),
            }
        }
        true
    });
    last_step+1
}

// WebBlock.entityInside; PowderSnowBlock.entityInside; HoneyBlock.entityInside
fn inside_block(state: &mut PlayerState, world: &impl World, p: BlockPos, _intersects: bool) {
    let block=world.block(p);
    if let Some(multiplier)=block.stuck_multiplier {
        stuck(state, multiplier);
    }
    match &block.inside {
        Inside::None=>{
        },
        Inside::Cobweb=>stuck(state, Vec3::new(0.25, 0.05f32 as f64, 0.25)),
        Inside::PowderSnow=>{
            if world.block(block_pos(state.position)).inside==Inside::PowderSnow {
                stuck(state, Vec3::new(0.9f32 as f64, 1.5, 0.9f32 as f64));
            }
            state.in_powder_snow=true;
        },
        Inside::Honey=>{
            let old_y=state.velocity.y/0.98f32 as f64+0.08;
            let edge=0.4375+(0.6f32/2.0f32) as f64;
            if !state.on_ground && state.position.y<=p[1] as f64+0.9375-1.0e-7 && old_y < -0.08 &&
            ((p[0] as f64+0.5-state.position.x).abs()+1.0e-7>edge || (p[2] as f64+0.5-state.position.z).abs()+1.0e-7>edge) {
                if old_y < -0.13 {
                    let f=-0.05/old_y;
                    state.velocity.x*=f;
                    state.velocity.z*=f;
                }
                state.velocity.y=(-0.05-0.08)*0.98f32 as f64;
                state.fall_distance=0.0;
            }
        },
        Inside::Unsupported(name) => unimplemented!("BlockState.entityInside: {name}"),
    }
}

// FoodData.addExhaustion
fn add_exhaustion(state: &mut PlayerState, amount: f32) {
    state.exhaustion=(state.exhaustion+amount).min(40.0);
}

// ServerPlayer.checkMovementStatistics
fn movement_exhaustion(state: &mut PlayerState, delta: Vec3) {
    if state.swimming || state.eye_in_water || state.in_water {
        let distance=if state.swimming || state.eye_in_water {
            delta.length_sqr().sqrt()
        } else {
            delta.horizontal_sqr().sqrt()
        };
        let cm=math::round(distance as f32*100.0);
        if cm>0 {
            add_exhaustion(state, 0.01f32*cm as f32*0.01f32);
        }
    } else if !state.on_climbable && state.on_ground && state.sprinting {
        let cm=math::round(delta.horizontal_sqr().sqrt() as f32*100.0);
        if cm>0 {
            add_exhaustion(state, 0.1f32*cm as f32*0.01f32);
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
pub enum ServerResourceEvent {
    Exhaustion(f32),
    Damage(f32),
    Regeneration(i32),
    FoodTick,
}

pub fn apply_server_resource_event(state: &mut PlayerState, world: &impl World, event: ServerResourceEvent) {
    match event {
        ServerResourceEvent::Exhaustion(amount) => add_exhaustion(state, amount),
        ServerResourceEvent::Damage(amount) => state.health = (state.health - amount).max(0.0),
        ServerResourceEvent::Regeneration(tick) => server_regeneration(state, world, tick),
        ServerResourceEvent::FoodTick => food_tick(state, world),
    }
}

fn server_regeneration(state: &mut PlayerState, world: &impl World, server_player_tick: i32) {
    // ServerPlayer.tickRegeneration, called by Player.aiStep before FoodData.tick.
    if world.difficulty() == 0 && world.natural_regeneration() {
        if server_player_tick % 20 == 0 {
            if state.health < 20.0 {
                state.health = (state.health + 1.0).min(20.0);
            }
            if state.saturation < 20.0 {
                state.saturation += 1.0;
            }
        }
        if server_player_tick % 10 == 0 && state.food_level < 20 {
            state.food_level += 1;
        }
    }
}

// FoodData.tick
fn food_tick(state: &mut PlayerState, world: &impl World) {
    if state.exhaustion>4.0 {
        state.exhaustion-=4.0;
        if state.saturation>0.0 {
            state.saturation=(state.saturation-1.0).max(0.0);
        } else if world.difficulty()!=0 {
            state.food_level=(state.food_level-1).max(0);
        }
    }
    if world.natural_regeneration() && state.saturation>0.0 && state.health>0.0 && state.health<20.0 && state.food_level>=20 {
        state.food_tick_timer+=1;
        if state.food_tick_timer>=10 {
            let amount=state.saturation.min(6.0);
            state.health=(state.health+amount/6.0).min(20.0);
            add_exhaustion(state, amount);
            state.food_tick_timer=0;
        }
    } else if world.natural_regeneration() && state.food_level>=18 && state.health>0.0 && state.health<20.0 {
        state.food_tick_timer+=1;
        if state.food_tick_timer>=80 {
            state.health=(state.health+1.0).min(20.0);
            add_exhaustion(state, 6.0);
            state.food_tick_timer=0;
        }
    } else if state.food_level<=0 {
        state.food_tick_timer+=1;
        if state.food_tick_timer>=80 {
            if state.health>10.0 || world.difficulty()==3 || state.health>1.0 && world.difficulty()==2 {
                state.health=(state.health-1.0).max(0.0);
            }
            state.food_tick_timer=0;
        }
    } else {
        state.food_tick_timer=0;
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
                    state.exhaustion += 0.25;
                }
                tick_server_resources(&mut state, &world, tick);
                assert_eq!(resources(&state), expected, "difficulty {difficulty}, regen {natural_regeneration}, tick {tick}");
            }
        }

        let mut world = crate::BoxWorld::default();
        world.difficulty = 0;
        let mut state = PlayerState { saturation: 19.5, ..Default::default() };
        tick_server_resources(&mut state, &world, 20);
        assert_eq!(state.saturation, 20.5);
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
            update_sprinting(&mut state, &input, bits & 1024 != 0);
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
        assert_eq!(state.velocity, Vec3::new(-0.1, 0.2, 0.8));
        world.set([-1, 0, 0], Block::full());
        move_towards_closest_space(&mut state, &world, 0.5, 0.5);
        assert_eq!(state.velocity, Vec3::new(0.1, 0.2, 0.8));
        world.set([0, 0, 0], Block { suffocating: false, ..Block::full() });
        state.velocity = Vec3::default();
        move_towards_closest_space(&mut state, &world, 0.5, 0.5);
        assert_eq!(state.velocity, Vec3::default());
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
        let position = state.position;
        inside_segment(&mut state, &world, position, position, &mut visited, 16);
        assert!(visited.contains(&[0, 0, 0]));
        assert_eq!(state.remaining_fire_ticks, 0);
        state.remaining_fire_ticks = 10;
        inside_segment(&mut state, &world, position, position, &mut visited, 16);
        assert_eq!(state.remaining_fire_ticks, 10);
        visited.clear();
        world.set([0, 0, 0], Block {
            air: false, inside_shape: Some(VoxelShape::default()), ..Block::default()
        });
        inside_segment(&mut state, &world, position, position, &mut visited, 16);
        assert!(visited.is_empty());
    }
}
