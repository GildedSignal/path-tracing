//! Boat laws: placement, damage/break, water-entry snap, IN_WATER/land/air physics, mount/clamp/dismount.
use crate::{
    interact::{self, BlockHitResult, ClipFluidMode},
    math,

    world::block_pos,
    Aabb,
    BlockPos,
    CollisionContext,
    FluidKind,
    Pose,
    Vec3,
    VoxelShape,
    World,
};

pub const BOAT_WIDTH: f32 = 1.375;
pub const BOAT_HEIGHT: f32 = 0.5625;
/// `BoatItem.use @39-44`: eye-ray entity obstruction search reach (`5.0d`).
pub const BOAT_EYE_OBSTRUCTION_RANGE: f64 = 5.0;
/// `VehicleEntity.hurtServer @110`: survival break requires strict `damage > 40.0f`.
pub const BOAT_BREAK_THRESHOLD: f32 = 40.0;
/// `VehicleEntity.hurtServer @44`: per-hit accumulation `amount * 10.0f`.
pub const BOAT_HURT_MULTIPLIER: f32 = 10.0;
/// `VehicleEntity.hurtServer @29`: `setHurtTime(10)`.
pub const BOAT_HURT_TIME: i32 = 10;
/// `AbstractBoat.tick @110`: `damage` decays by `1.0f` per tick while `> 0.0f`.
pub const BOAT_DAMAGE_DECAY: f32 = 1.0;

// EntityDimensions.makeBoundingBox for EntityTypes.OAK_BOAT (1.375 x 0.5625).
pub fn boat_aabb(position: Vec3) -> Aabb {
    let half = (BOAT_WIDTH / 2.0f32) as f64;
    Aabb::new(
        position.x - half,
        position.y,
        position.z - half,
        position.x + half,
        position.y + BOAT_HEIGHT as f64,
        position.z + half,
    )
}

// AbstractBoat.getPassengerAttachmentPoint 31-85; Entity.positionRider.
pub fn seat(position: Vec3, yaw: f32, ride_height: f64, offset: f32) -> Vec3 {
    let angle = (-yaw * 0.017453292f32) as f64;
    let sin = math::sin(angle) as f64;
    let cos = math::cos(angle) as f64;
    position.plus(Vec3::new(0.0 * cos + (offset as f64) * sin, ride_height, (offset as f64) * cos - 0.0 * sin))
        .minus(Vec3::new(0.0, 0.6, 0.0)) // Avatar.DEFAULT_VEHICLE_ATTACHMENT (0, 0.6, 0)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Boat { pub position: Vec3, pub velocity: Vec3, pub yaw: f32, pub delta_rotation: f32 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls { pub left: bool, pub right: bool, pub forward: bool, pub backward: bool }

/// `AbstractBoat.controlBoat @8-211`: updates `delta_rotation`, `yaw`, horizontal thrust,
/// and returns `(paddle_left, paddle_right)`.
pub fn control_boat(boat: &mut Boat, input: Controls) -> (bool, bool) {
    if input.left { boat.delta_rotation -= 1.0f32; }
    if input.right { boat.delta_rotation += 1.0f32; }
    let mut thrust = 0.0f32;
    if input.left != input.right && !input.forward && !input.backward { thrust += 0.005f32; }
    boat.yaw += boat.delta_rotation;
    if input.forward { thrust += 0.04f32; }
    if input.backward { thrust -= 0.005f32; }
    boat.velocity.x += (math::sin((-boat.yaw * 0.017453292f32) as f64) * thrust) as f64;
    boat.velocity.z += (math::cos((boat.yaw * 0.017453292f32) as f64) * thrust) as f64;
    let paddle_left = (input.right && !input.left) || input.forward;
    let paddle_right = (input.left && !input.right) || input.forward;
    (paddle_left, paddle_right)
}

// AbstractBoat.tick 150,164,191-199; floatBoat 138-167,270-359;
// getDefaultGravity 0-3; controlBoat 0-152. Water level is supplied by the caller.
pub fn tick(boat: &mut Boat, water_level: f64, input: Controls) {
    let draft = (water_level - boat.position.y) / (0.5625f32 as f64);
    let drag = 0.9f32;
    boat.velocity.x *= drag as f64;
    boat.velocity.y += -0.04f64;
    boat.velocity.z *= drag as f64;
    boat.delta_rotation *= drag;
    if draft > 0.0 { boat.velocity.y = (boat.velocity.y + draft * (0.04f64 / 0.65f64)) * 0.75f64; }
    let _ = control_boat(boat, input);
    boat.position = boat.position.plus(boat.velocity);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoatStatus {
    InWater,
    UnderWater,
    UnderFlowingWater,
    OnLand,
    InAir,
}

/// `FluidState.isSource` for water, read by `AbstractBoat.isUnderwater`.
fn is_water_source(world: &impl World, pos: BlockPos) -> bool {
    let fluid = world.fluid_block(pos).fluid;
        fluid.kind == FluidKind::Water
            && fluid.raw_amount == Some(8)
            && fluid.raw_falling != Some(true)
}

/// `VehicleEntity` hit/break state (`DATA_ID_HURT`, `DATA_ID_HURTDIR`, `DATA_ID_DAMAGE`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoatHurtState {
    pub hurt_time: i32,
    pub hurt_dir: i32,
    pub damage: f32,
    pub removed: bool,
}

impl Default for BoatHurtState {
    fn default() -> Self {
        Self {
            hurt_time: 0,
            hurt_dir: 1,
            damage: 0.0,
            removed: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoatHurtOutcome {
    AlreadyRemoved,
    Invulnerable,
    Damaged,
    Destroyed { drop_item: bool },
    Discarded,
}

impl BoatHurtState {
    /// `VehicleEntity.hurtServer @0-143` + `VehicleEntity.destroy @122-151`.
    pub fn hurt_server(
        &mut self,
        amount: f32,
        attacker_instabuild: bool,
        should_source_destroy: bool,
        invulnerable_to_base: bool,
        do_entity_drops: bool,
    ) -> (bool, BoatHurtOutcome) {
        if self.removed {
            return (true, BoatHurtOutcome::AlreadyRemoved);
        }
        if invulnerable_to_base {
            return (false, BoatHurtOutcome::Invulnerable);
        }
        self.hurt_dir = -self.hurt_dir;
        self.hurt_time = BOAT_HURT_TIME;
        self.damage += amount * BOAT_HURT_MULTIPLIER;

        let destroy = (!attacker_instabuild && self.damage > BOAT_BREAK_THRESHOLD)
            || should_source_destroy;
        if destroy {
            self.removed = true;
            (true, BoatHurtOutcome::Destroyed { drop_item: do_entity_drops })
        } else if attacker_instabuild {
            self.removed = true;
            (true, BoatHurtOutcome::Discarded)
        } else {
            (true, BoatHurtOutcome::Damaged)
        }
    }

    /// `AbstractBoat.tick @79-112`: decays `hurtTime` and `damage` per tick.
    pub fn tick_decay(&mut self) {
        crate::basis::clock::COOLDOWN.cool(&mut self.hurt_time);
        if self.damage > 0.0f32 {
            self.damage -= BOAT_DAMAGE_DECAY;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoatPickTarget {
    pub aabb: Aabb,
    pub pick_radius: f32,
    pub is_pickable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoatPlaceResult {
    Pass,
    Fail,
    Success {
        pos: Vec3,
        yaw: f32,
        aabb: Aabb,
        spawned_server: bool,
        consumed: bool,
    },
}

/// `Item.getPlayerPOVHitResult` with `ClipContext.Block.OUTLINE` and caller fluid mode.
pub fn boat_pov_hit(
    world: &impl World,
    eye_pos: Vec3,
    yaw: f32,
    pitch: f32,
    block_interaction_range: f64,
    fluid_mode: ClipFluidMode,
) -> BlockHitResult {
    let view = math::calculate_view_vector(yaw, pitch);
    let end = Vec3::new(
        eye_pos.x + view.x * block_interaction_range,
        eye_pos.y + view.y * block_interaction_range,
        eye_pos.z + view.z * block_interaction_range,
    );
    interact::clip_blocks(eye_pos, end, fluid_mode, |cell| {
        let block = world.block(cell);
        let fh = if block.fluid.kind != FluidKind::Empty {
            block.fluid.surface_height(world, cell)
        } else {
            0.0
        };
        let is_source = block.fluid.raw_amount == Some(8);
        (&block.shape, None, fh, is_source)
    })
}

/// `BoatItem.use @0-264` + `BoatItem.getBoat @0-77`:
/// raycasts with `ClipContext.Fluid.ANY`, rejects if eyes lie inside an inflated pickable entity
/// box, spawns at the raw hit location with `yaw`, and checks `noCollision`.
#[allow(clippy::too_many_arguments)]
pub fn place_boat(
    world: &impl World,
    player_pos: Vec3,
    player_bb: Aabb,
    eye_height: f32,
    yaw: f32,
    pitch: f32,
    block_interaction_range: f64,
    has_infinite_materials: bool,
    is_client_side: bool,
    pick_targets: &[BoatPickTarget],
) -> BoatPlaceResult {
    let eye = interact::get_eye_position(player_pos, eye_height);
    let hit = boat_pov_hit(
        world,
        eye,
        yaw,
        pitch,
        block_interaction_range,
        ClipFluidMode::Any,
    );
    if !hit.is_hit() {
        return BoatPlaceResult::Pass;
    }

    let view = math::calculate_view_vector(yaw, pitch);
    let search_box = player_bb
        .expand(view.scale(BOAT_EYE_OBSTRUCTION_RANGE))
        .deflate(-1.0);
    for target in pick_targets {
        if target.is_pickable
            && search_box.intersects(target.aabb)
            && target
                .aabb
                .deflate(-(target.pick_radius as f64))
                .contains(eye)
        {
            return BoatPlaceResult::Pass;
        }
    }

    let pos = hit.location();
    let aabb = boat_aabb(pos);
    if !world.no_collision(aabb, empty_context(pos.y)) {
        return BoatPlaceResult::Fail;
    }

    BoatPlaceResult::Success {
        pos,
        yaw,
        aabb,
        spawned_server: !is_client_side,
        consumed: !is_client_side && !has_infinite_materials,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BoatBody {
    pub boat: Boat,
    pub status: Option<BoatStatus>,
    pub old_status: Option<BoatStatus>,
    pub land_friction: f32,
    pub out_of_control_ticks: f32,
    pub water_level: f64,
    pub last_yd: f64,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub no_gravity: bool,
    pub has_passenger: bool,
    pub paddle_left: bool,
    pub paddle_right: bool,
    pub hurt: BoatHurtState,
}

impl Default for BoatBody {
    fn default() -> Self {
        Self {
            boat: Boat::default(),
            status: None,
            old_status: None,
            land_friction: 0.0,
            out_of_control_ticks: 0.0,
            water_level: 0.0,
            last_yd: 0.0,
            on_ground: false,
            horizontal_collision: false,
            no_gravity: false,
            has_passenger: false,
            paddle_left: false,
            paddle_right: false,
            hurt: BoatHurtState::default(),
        }
    }
}

fn empty_context(feet_y: f64) -> CollisionContext {
    CollisionContext {
        feet_y,
        descending: false,
        fall_distance: 0.0,
        leather_boots: false,
    }
}

// AbstractBoat.getGroundFriction @5-340
pub fn get_ground_friction(world: &impl World, position: Vec3) -> f32 {
    let bb = boat_aabb(position);
    let probe = Aabb::new(bb.min.x, bb.min.y - 0.001, bb.min.z, bb.max.x, bb.min.y, bb.max.z);
    let x0 = math::floor(probe.min.x) - 1;
    let x1 = probe.max.x.ceil() as i32 + 1;
    let y0 = math::floor(probe.min.y) - 1;
    let y1 = probe.max.y.ceil() as i32 + 1;
    let z0 = math::floor(probe.min.z) - 1;
    let z1 = probe.max.z.ceil() as i32 + 1;
    let context = empty_context(position.y);
    let mut sum = 0.0f32;
    let mut count = 0i32;
    for x in x0..x1 {
        for z in z0..z1 {
            let edges = i32::from(x == x0 || x == x1 - 1) + i32::from(z == z0 || z == z1 - 1);
            if edges == 2 {
                continue;
            }
            for y in y0..y1 {
                if edges > 0 && (y == y0 || y == y1 - 1) {
                    continue;
                }
                let pos = [x, y, z];
                if world.block(pos).lily_pad {
                    continue;
                }
                let shape = world.collision_shape(pos, context);
                let offset = Vec3::new(x as f64, y as f64, z as f64);
                if shape.at(offset).intersects(probe) {
                    sum += world.block(pos).friction;
                    count += 1;
                }
            }
        }
    }
    sum / (count as f32)
}

// AbstractBoat.isUnderwater @0-216
fn is_underwater(world: &impl World, bb: Aabb) -> Option<BoatStatus> {
    let top = bb.max.y + 0.001;
    let x0 = math::floor(bb.min.x);
    let x1 = bb.max.x.ceil() as i32;
    let y0 = math::floor(bb.max.y);
    let y1 = top.ceil() as i32;
    let z0 = math::floor(bb.min.z);
    let z1 = bb.max.z.ceil() as i32;
    let mut under_source = false;
    for x in x0..x1 {
        for y in y0..y1 {
            for z in z0..z1 {
                let pos = [x, y, z];
                let fluid = world.fluid_block(pos).fluid;
                if fluid.kind == FluidKind::Water {
                    let surface = (y as f32 + fluid.surface_height(world, pos)) as f64;
                    if top < surface {
                        if is_water_source(world, pos) {
                            under_source = true;
                        } else {
                            return Some(BoatStatus::UnderFlowingWater);
                        }
                    }
                }
            }
        }
    }
    under_source.then_some(BoatStatus::UnderWater)
}

// AbstractBoat.checkInWater @0-220
fn check_in_water(world: &impl World, bb: Aabb, water_level: &mut f64) -> bool {
    let x0 = math::floor(bb.min.x);
    let x1 = bb.max.x.ceil() as i32;
    let y0 = math::floor(bb.min.y);
    let y1 = (bb.min.y + 0.001).ceil() as i32;
    let z0 = math::floor(bb.min.z);
    let z1 = bb.max.z.ceil() as i32;
    let mut in_water = false;
    *water_level = -f64::MAX;
    for x in x0..x1 {
        for y in y0..y1 {
            for z in z0..z1 {
                let pos = [x, y, z];
                let fluid = world.fluid_block(pos).fluid;
                if fluid.kind != FluidKind::Water {
                    continue;
                }
                let surface = y as f32 + fluid.surface_height(world, pos);
                *water_level = (*water_level).max(surface as f64);
                in_water |= bb.min.y < surface as f64;
            }
        }
    }
    in_water
}

// AbstractBoat.getWaterLevelAbove @0-207
fn water_level_above(world: &impl World, bb: Aabb, last_yd: f64) -> f32 {
    let x0 = math::floor(bb.min.x);
    let x1 = bb.max.x.ceil() as i32;
    let y0 = math::floor(bb.max.y);
    let y1 = (bb.max.y - last_yd).ceil() as i32;
    let z0 = math::floor(bb.min.z);
    let z1 = bb.max.z.ceil() as i32;
    for y in y0..y1 {
        let mut h = 0.0f32;
        for x in x0..x1 {
            for z in z0..z1 {
                let pos = [x, y, z];
                let fluid = world.fluid_block(pos).fluid;
                if fluid.kind == FluidKind::Water {
                    h = h.max(fluid.surface_height(world, pos));
                }
                if h >= 1.0f32 {
                    break;
                }
            }
            if h >= 1.0f32 {
                break;
            }
        }
        if h < 1.0f32 {
            return y as f32 + h;
        }
    }
    (y1 + 1) as f32
}

// AbstractBoat.getStatus @0-56
pub fn get_status(body: &mut BoatBody, world: &impl World) -> BoatStatus {
    let bb = boat_aabb(body.boat.position);
    if let Some(status) = is_underwater(world, bb) {
        body.water_level = bb.max.y;
        return status;
    }
    if check_in_water(world, bb, &mut body.water_level) {
        return BoatStatus::InWater;
    }
    let friction = get_ground_friction(world, body.boat.position);
    if friction > 0.0f32 {
        body.land_friction = friction;
        BoatStatus::OnLand
    } else {
        BoatStatus::InAir
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaterEntry {
    Snap(f64),
    Blocked,
}

/// `AbstractBoat.floatBoat @52-66`: `(float)(getWaterLevelAbove() - getBbHeight()) + 0.101`.
pub fn water_entry_target(water_level_above: f32, bb_height: f32) -> f64 {
    (water_level_above - bb_height) as f64 + 0.101
}

/// `AbstractBoat.floatBoat @13-135`: air -> water entry transition.
pub fn water_entry(
    old_in_air: bool,
    status_in_air: bool,
    status_on_land: bool,
    water_level_above: f32,
    bb_height: f32,
    ceiling_clear: bool,
) -> Option<WaterEntry> {
    if !old_in_air || status_in_air || status_on_land {
        return None;
    }
    Some(if ceiling_clear {
        WaterEntry::Snap(water_entry_target(water_level_above, bb_height))
    } else {
        WaterEntry::Blocked
    })
}

// AbstractBoat.floatBoat @0-362
fn float_boat(body: &mut BoatBody, world: &impl World, player_controlling: bool) {
    let mut d1 = if body.no_gravity { 0.0 } else { -0.04 };
    let mut d3 = 0.0;
    let f;
    let Some(status) = body.status else {
        return;
    };
    if body.old_status == Some(BoatStatus::InAir)
        && !matches!(status, BoatStatus::InAir | BoatStatus::OnLand)
    {
        body.water_level = body.boat.position.y + BOAT_HEIGHT as f64;
        let bb = boat_aabb(body.boat.position);
        let wla = water_level_above(world, bb, body.last_yd);
        let target_y = water_entry_target(wla, BOAT_HEIGHT);
        let moved = bb.moved(Vec3::new(0.0, target_y - body.boat.position.y, 0.0));
        if world.no_collision(moved, empty_context(body.boat.position.y)) {
            body.boat.position.y = target_y;
            // v.multiply(1, 0, 1): preserves negative zero when falling.
            body.boat.velocity = body.boat.velocity.multiply(Vec3::new(1.0, 0.0, 1.0));
            body.last_yd = 0.0;
        }
        body.status = Some(BoatStatus::InWater);
        return;
    }
    match status {
        BoatStatus::InWater => {
            d3 = (body.water_level - body.boat.position.y) / (BOAT_HEIGHT as f64);
            f = 0.9f32;
        }
        BoatStatus::UnderFlowingWater => {
            d1 = -7.0e-4;
            f = 0.9f32;
        }
        BoatStatus::UnderWater => {
            d3 = 0.01f32 as f64;
            f = 0.45f32;
        }
        BoatStatus::InAir => {
            f = 0.9f32;
        }
        BoatStatus::OnLand => {
            f = body.land_friction;
            if player_controlling {
                body.land_friction /= 2.0f32;
            }
        }
    }
    body.boat.velocity.x *= f as f64;
    body.boat.velocity.y += d1;
    body.boat.velocity.z *= f as f64;
    body.boat.delta_rotation *= f;
    if d3 > 0.0 {
        body.boat.velocity.y = (body.boat.velocity.y + d3 * (0.04 / 0.65)) * 0.75;
    }
}

/// `AbstractBoat.tick` (`@0-209`) + `Entity.move` (`maxUpStep = 0.0f`) +
/// `Entity.restituteMovementAfterCollisions` (`b = 0.0`).
/// Returns `true` if a passenger was ejected on this tick (`outOfControlTicks >= 60.0f` server-side).
pub fn tick_body(
    body: &mut BoatBody,
    world: &impl World,
    input: Controls,
    server_side: bool,
    player_controlling: bool,
    apply_controls: bool,
) -> bool {
    body.old_status = body.status;
    let status = get_status(body, world);
    body.status = Some(status);
    if matches!(status, BoatStatus::UnderWater | BoatStatus::UnderFlowingWater) {
        body.out_of_control_ticks += 1.0f32;
    } else {
        body.out_of_control_ticks = 0.0f32;
    }
    let ejected = server_side && body.has_passenger && body.out_of_control_ticks >= 60.0f32;
    if ejected {
        body.has_passenger = false;
    }
    body.hurt.tick_decay();

    // Occupied-player boat is client-authoritative; server zeroes velocity instead of stepping physics.
    if server_side && player_controlling {
        body.boat.velocity = Vec3::default();
        return ejected;
    }

    if !player_controlling {
        body.paddle_left = false;
        body.paddle_right = false;
    }
    float_boat(body, world, player_controlling);
    if apply_controls {
        let (pl, pr) = control_boat(&mut body.boat, input);
        body.paddle_left = pl;
        body.paddle_right = pr;
    }

    // AbstractBoat.maxUpStep() == 0.0f: the sweep without step-up; Entity.move @546 ->
    // AbstractBoat.checkFallDamage @0-8 reads the velocity before restitution.
    let moved = crate::basis::sweep::moved(world, boat_aabb(body.boat.position), empty_context(body.boat.position.y), body.boat.velocity);
    body.last_yd = body.boat.velocity.y;
    let m = crate::basis::body::settle(&mut body.boat.position, &mut body.boat.velocity, moved);
    body.horizontal_collision = m.horizontal();
    body.on_ground = m.below();
    ejected
}

/// `AbstractBoat.interact` (`@20-35`) + `AbstractBoat.canAddPassenger` (`@0-31`).
pub fn can_mount(
    out_of_control_ticks: f32,
    passengers_len: usize,
    eye_in_water: bool,
    sneaking: bool,
) -> bool {
    !sneaking && out_of_control_ticks < 60.0f32 && passengers_len < 2 && !eye_in_water
}

/// `Entity.startRiding` (`@106-110`): forces `Pose::Standing` without altering position, yaw, or sprint.
pub fn start_riding_pose() -> Pose {
    Pose::Standing
}

/// `Entity.addPassenger` (`@40-94`): a server-side player takes seat 0 when seat 0 is non-player.
pub fn add_passenger_seat<T: Copy>(
    seats: &mut Vec<T>,
    passenger: T,
    is_player: bool,
    is_seat_player: impl Fn(T) -> bool,
) -> bool {
    if seats.len() >= 2 {
        return false;
    }
    if is_player && !seats.is_empty() && !is_seat_player(seats[0]) {
        seats.insert(0, passenger);
    } else {
        seats.push(passenger);
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiderRotation {
    pub y_rot: f32,
    pub y_rot_o: f32,
    pub y_head_rot: f32,
    pub y_body_rot: f32,
}

/// `AbstractBoat.clampRotation` (`@0-64`).
pub fn clamp_rotation(hull_yaw: f32, mut rot: RiderRotation) -> RiderRotation {
    rot.y_body_rot = hull_yaw;
    let f = math::wrap_degrees(rot.y_rot - hull_yaw);
    let f1 = f.clamp(-105.0f32, 105.0f32);
    rot.y_rot_o += f1 - f;
    rot.y_rot = (rot.y_rot + f1) - f;
    rot.y_head_rot = rot.y_rot;
    rot
}

/// `AbstractBoat.positionRider` (`@6-45`): applies `delta_rotation` and `clamp_rotation`
/// unless the rider is in `CAN_TURN_IN_BOATS` (`minecraft:breeze` only in 26.2).
pub fn position_rider_rotation(
    hull_yaw: f32,
    delta_rotation: f32,
    mut rot: RiderRotation,
    can_turn_in_boats: bool,
) -> RiderRotation {
    if !can_turn_in_boats {
        rot.y_rot += delta_rotation;
        rot.y_head_rot += delta_rotation;
        rot = clamp_rotation(hull_yaw, rot);
    }
    rot
}

/// `Entity.getCollisionHorizontalEscapeVector` (`@0-81`).
pub fn collision_horizontal_escape_vector(a: f64, b: f64, yaw: f32) -> Vec3 {
    let d5 = (a + b + (1.0e-5f32 as f64)) / 2.0;
    let rad = (yaw * 0.017453292f32) as f64;
    let f = -math::sin(rad);
    let f1 = math::cos(rad);
    let f2 = f.abs().max(f1.abs());
    Vec3::new((f as f64 * d5) / (f2 as f64), 0.0, (f1 as f64 * d5) / (f2 as f64))
}

fn shape_max_y(shape: &VoxelShape) -> f64 {
    if shape.is_empty() {
        return f64::NEG_INFINITY;
    }
    let nx = shape.coords[0].len().saturating_sub(1);
    let ny = shape.coords[1].len().saturating_sub(1);
    let nz = shape.coords[2].len().saturating_sub(1);
    let mut max_y = f64::NEG_INFINITY;
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                if shape.is_full([x, y, z]) {
                    max_y = max_y.max(shape.coords[1][y + 1]);
                }
            }
        }
    }
    max_y
}

/// `BlockGetter.getBlockFloorHeight` (`@0-44`).
pub fn block_floor_height(world: &impl World, pos: BlockPos) -> f64 {
    let ctx = empty_context(f64::NEG_INFINITY);
    let shape = world.collision_shape(pos, ctx);
    if !shape.is_empty() {
        return shape_max_y(shape);
    }
    let below_shape = world.collision_shape([pos[0], pos[1] - 1, pos[2]], ctx);
    let below_max = shape_max_y(below_shape);
    if below_max >= 1.0 {
        below_max - 1.0
    } else {
        f64::NEG_INFINITY
    }
}

/// `DismountHelper.isBlockFloorValid` (`@0-18`).
pub fn is_block_floor_valid(floor_height: f64) -> bool {
    !floor_height.is_infinite() && floor_height < 1.0
}

/// `AbstractBoat.getDismountLocationForPassenger` (`@0-271`) with player poses
/// `[Standing, Crouching, Swimming]`. Returns `(exit_pos, Some(new_pose))` on candidate
/// success, or `(boat_center_max_y, None)` on fallback / water below.
pub fn dismount_location(
    world: &impl World,
    boat_pos: Vec3,
    rider_yaw: f32,
    rider_width: f32,
) -> (Vec3, Option<Pose>) {
    let sqrt2 = 2.0f32.sqrt();
    let escape = collision_horizontal_escape_vector(
        (BOAT_WIDTH * sqrt2) as f64,
        rider_width as f64,
        rider_yaw,
    );
    let exit_x = boat_pos.x + escape.x;
    let exit_z = boat_pos.z + escape.z;
    let boat_bb = boat_aabb(boat_pos);
    let candidate_pos = block_pos(Vec3::new(exit_x, boat_bb.max.y, exit_z));
    let below = [candidate_pos[0], candidate_pos[1] - 1, candidate_pos[2]];

    if world.fluid_block(below).fluid.kind != FluidKind::Water {
        let mut candidates = Vec::with_capacity(2);
        let h1 = block_floor_height(world, candidate_pos);
        if is_block_floor_valid(h1) {
            candidates.push(Vec3::new(exit_x, candidate_pos[1] as f64 + h1, exit_z));
        }
        let h2 = block_floor_height(world, below);
        if is_block_floor_valid(h2) {
            candidates.push(Vec3::new(exit_x, below[1] as f64 + h2, exit_z));
        }
        let half = (rider_width / 2.0f32) as f64;
        for pose in [Pose::Standing, Pose::Crouching, Pose::Swimming] {
            let local = Aabb::new(
                -half,
                0.0,
                -half,
                half,
                pose.height() as f64,
                half,
            );
            for &vec in &candidates {
                let test_box = local.moved(vec);
                if world.no_collision(test_box, empty_context(vec.y)) {
                    return (vec, Some(pose));
                }
            }
        }
    }

    (Vec3::new(boat_pos.x, boat_bb.max.y, boat_pos.z), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoxWorld;
    use crate::{Block, Fluid, FluidFlowBlock};

    struct BoatTestWorld {
        inner: BoxWorld,
        all_water: bool,
        water_source: bool,
    }

    impl BoatTestWorld {
        fn new() -> Self {
            Self {
                inner: BoxWorld::default(),
                all_water: false,
                water_source: true,
            }
        }
    }

    impl World for BoatTestWorld {
        fn block(&self, position: BlockPos) -> &Block {
            if self.all_water {
                static SOURCE: std::sync::OnceLock<Block> = std::sync::OnceLock::new();
                static FLOWING: std::sync::OnceLock<Block> = std::sync::OnceLock::new();
                if self.water_source {
                    return SOURCE.get_or_init(|| Block {
                        air: false,
                        fluid: Fluid {
                            kind: FluidKind::Water,
                            raw_amount: Some(8),
                            raw_falling: Some(false),
                            ..Fluid::default()
                        },
                        fluid_flow: Some(FluidFlowBlock::default()),
                        ..Block::default()
                    });
                } else {
                    return FLOWING.get_or_init(|| Block {
                        air: false,
                        fluid: Fluid {
                            kind: FluidKind::Water,
                            raw_amount: Some(1),
                            raw_falling: Some(false),
                            ..Fluid::default()
                        },
                        fluid_flow: Some(FluidFlowBlock::default()),
                        ..Block::default()
                    });
                }
            }
            self.inner.block(position)
        }

        fn visit_collision_cells(
            &self,
            bounds: Aabb,
            lo: BlockPos,
            hi: BlockPos,
            visit: impl FnMut(BlockPos) -> bool,
        ) -> bool {
            if self.all_water {
                return true;
            }
            self.inner.visit_collision_cells(bounds, lo, hi, visit)
        }
    }

    fn block_with_friction(friction: f32) -> Block {
        Block {
            friction,
            ..Block::full()
        }
    }

    fn stone_block() -> Block {
        block_with_friction(f32::from_bits(0x3f19_999a)) // evidence.txt:1 (0.6f)
    }

    fn blue_ice_block() -> Block {
        block_with_friction(f32::from_bits(0x3f7d_2f1b)) // evidence.txt:1 (0.989f)
    }

    fn packed_ice_block() -> Block {
        block_with_friction(f32::from_bits(0x3f7a_e148)) // evidence.txt:1 (0.98f)
    }

    fn bottom_slab_block() -> Block {
        Block {
            friction: f32::from_bits(0x3f19_999a),
            ..Block::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)])
        }
    }

    #[test]
    fn boat_land_air_water_and_collision_witnesses() {
        let zero = Controls::default();

        // 1. ON_LAND stone (evidence.txt:2-13)
        let mut stone_world = BoatTestWorld::new();
        stone_world.inner.fill([90, 60, 90], [110, 64, 110], stone_block());
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 65.0, 100.5),
                velocity: Vec3::new(f64::from_bits(0x3fd3_3333_3333_3333), 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        assert_eq!(get_ground_friction(&stone_world, b.boat.position).to_bits(), 0x3f19_9999); // evidence.txt:2
        tick_body(&mut b, &stone_world, zero, true, false, false);
        assert_eq!(b.status, Some(BoatStatus::OnLand));
        assert_eq!(b.land_friction.to_bits(), 0x3f19_9999); // evidence.txt:4
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fc7_0a3d_5999_9999); // evidence.txt:4
        assert_eq!(b.boat.velocity.y.to_bits(), 0x0000_0000_0000_0000); // evidence.txt:4 (+0.0)
        assert!(b.on_ground && !b.horizontal_collision);

        // 2. ON_LAND blue ice (evidence.txt:14-16)
        let mut blue_world = BoatTestWorld::new();
        blue_world.inner.fill([90, 60, 90], [110, 64, 110], blue_ice_block());
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 65.0, 100.5),
                velocity: Vec3::new(f64::from_bits(0x3fd3_3333_3333_3333), 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b, &blue_world, zero, true, false, false);
        assert_eq!(b.land_friction.to_bits(), 0x3f7d_2f1b); // evidence.txt:16
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fd2_fd22_0666_6666); // evidence.txt:16

        // 3. ON_LAND ice 1-ulp grid drift (evidence.txt:26-30)
        let mut ice_world = BoatTestWorld::new();
        ice_world.inner.fill([90, 60, 90], [110, 64, 110], packed_ice_block());
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 65.0, 100.5),
                velocity: Vec3::new(f64::from_bits(0x3fd3_3333_3333_3333), 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b, &ice_world, zero, true, false, false);
        assert_eq!(b.land_friction.to_bits(), 0x3f7a_e147); // evidence.txt:28
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fd2_d0e5_5333_3333); // evidence.txt:28
        tick_body(&mut b, &ice_world, zero, true, false, false);
        assert_eq!(b.land_friction.to_bits(), 0x3f7a_e148); // evidence.txt:29
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fd2_708e_d78e_9f66); // evidence.txt:29

        // 4. ON_LAND mixed stone / blue ice boundary (evidence.txt:38-40)
        let mut mixed_world = BoatTestWorld::new();
        mixed_world.inner.fill([90, 60, 90], [99, 64, 110], stone_block());
        mixed_world.inner.fill([100, 60, 90], [110, 64, 110], blue_ice_block());
        assert_eq!(
            get_ground_friction(&mixed_world, Vec3::new(100.0, 65.0, 100.5)).to_bits(),
            0x3f4b_645b // evidence.txt:38
        );

        // 5. IN_AIR (evidence.txt:50-53)
        let air_world = BoatTestWorld::new();
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 80.0, 100.5),
                velocity: Vec3::new(
                    f64::from_bits(0x3fd3_3333_3333_3333),
                    f64::from_bits(0x3fa9_9999_9999_999a),
                    0.0,
                ),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        assert!(get_ground_friction(&air_world, b.boat.position).is_nan());
        tick_body(&mut b, &air_world, zero, true, false, false);
        assert_eq!(b.status, Some(BoatStatus::InAir));
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fd1_47ae_0ccc_cccd); // evidence.txt:52
        assert_eq!(b.boat.velocity.y.to_bits(), 0x3f84_7ae1_47ae_147c); // evidence.txt:52

        // 6. UNDER_WATER + tick 60 ejection (evidence.txt:63-83)
        let mut water_world = BoatTestWorld::new();
        water_world.all_water = true;
        water_world.water_source = true;
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 60.0, 100.5),
                velocity: Vec3::new(f64::from_bits(0x3fd3_3333_3333_3333), 0.0, 0.0),
                ..Boat::default()
            },
            has_passenger: true,
            ..BoatBody::default()
        };
        tick_body(&mut b, &water_world, zero, true, false, false);
        assert_eq!(b.status, Some(BoatStatus::UnderWater));
        assert_eq!(b.out_of_control_ticks.to_bits(), 0x3f80_0000); // evidence.txt:65
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fc1_47ae_0ccc_cccd); // evidence.txt:65
        assert_eq!(b.boat.velocity.y.to_bits(), 0xbf9e_3f54_994d_fb47); // evidence.txt:65
        for t in 2..=60 {
            let ejected = tick_body(&mut b, &water_world, zero, true, false, false);
            assert_eq!(ejected, t == 60, "tick {t}");
        }
        assert_eq!(b.out_of_control_ticks.to_bits(), 0x4270_0000); // evidence.txt:82
        assert!(!b.has_passenger);

        // 7. UNDER_FLOWING_WATER (check-extra.out:3-12)
        let mut flow_world = BoatTestWorld::new();
        flow_world.all_water = true;
        flow_world.water_source = false;
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 60.0, 100.5),
                velocity: Vec3::new(
                    f64::from_bits(0x3fd3_3333_3333_3333),
                    f64::from_bits(0x3fa9_9999_9999_999a),
                    0.0,
                ),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b, &flow_world, zero, true, false, false);
        assert_eq!(b.status, Some(BoatStatus::UnderFlowingWater));
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fd1_47ae_0ccc_cccd); // check-extra.out:5
        assert_eq!(b.boat.velocity.y.to_bits(), 0x3fa9_3dd9_7f62_b6af); // check-extra.out:5

        let mut b0 = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 60.0, 100.5),
                velocity: Vec3::new(f64::from_bits(0x3fd3_3333_3333_3333), 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b0, &flow_world, zero, true, false, false);
        assert_eq!(b0.boat.velocity.y.to_bits(), 0xbf46_f006_8db8_bac7); // check-extra.out:11

        // 8. WALL +x collision -> -0.0 raw (evidence.txt:84-88)
        let mut wall_world = BoatTestWorld::new();
        wall_world.inner.fill([90, 60, 90], [110, 64, 110], stone_block());
        wall_world.inner.fill([104, 65, 98], [104, 70, 103], stone_block());
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(103.0, 65.0, 100.5),
                velocity: Vec3::new(0.5, 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b, &wall_world, zero, true, false, false);
        assert_eq!(b.boat.velocity.x.to_bits(), 0x3fd3_3333_2000_0000); // evidence.txt:86
        tick_body(&mut b, &wall_world, zero, true, false, false);
        assert!(b.horizontal_collision);
        assert_eq!(b.boat.velocity.x.to_bits(), 0x8000_0000_0000_0000); // evidence.txt:87 (-0.0)
        assert_eq!(b.boat.position.x, 103.3125); // evidence.txt:87

        // 9. WALL -x collision -> +0.0 raw (check-extra2.out:19-22)
        let mut neg_wall_world = BoatTestWorld::new();
        neg_wall_world.inner.fill([90, 60, 90], [110, 64, 110], stone_block());
        neg_wall_world.inner.fill([101, 65, 98], [101, 70, 103], stone_block());
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(103.0, 65.0, 100.5),
                velocity: Vec3::new(-0.5, 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b, &neg_wall_world, zero, true, false, false);
        assert_eq!(b.boat.velocity.x.to_bits(), 0xbfd3_3333_2000_0000); // check-extra2.out:21
        tick_body(&mut b, &neg_wall_world, zero, true, false, false);
        assert!(b.horizontal_collision);
        assert_eq!(b.boat.velocity.x.to_bits(), 0x0000_0000_0000_0000); // check-extra2.out:22 (+0.0)
        assert_eq!(b.boat.position.x, 102.6875); // check-extra2.out:22

        // 10. SLAB maxUpStep = 0.0 stop (evidence.txt:96-98)
        let mut slab_world = BoatTestWorld::new();
        slab_world.inner.fill([90, 60, 90], [110, 64, 110], stone_block());
        slab_world.inner.set([102, 65, 100], bottom_slab_block());
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(101.3, 65.0, 100.5),
                velocity: Vec3::new(0.5, 0.0, 0.0),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        tick_body(&mut b, &slab_world, zero, true, false, false);
        assert!(b.horizontal_collision);
        assert_eq!(b.boat.position.x, 101.3125); // evidence.txt:98
        assert_eq!(b.boat.velocity.x.to_bits(), 0x8000_0000_0000_0000); // evidence.txt:98

        // 11. LILY pad skipped -> NaN -> IN_AIR even while grounded (check-extra.out:27-40)
        let mut lily_world = BoatTestWorld::new();
        let lily_box = Block { lily_pad: true, ..Block::boxes(&[Aabb::new(0.0625, 0.0, 0.0625, 0.9375, 0.09375, 0.9375)]) };
        for x in 99..=101 {
            for z in 99..=101 {
                lily_world.inner.set([x, 65, z], lily_box.clone());
            }
        }
        let mut b = BoatBody {
            boat: Boat {
                position: Vec3::new(100.5, 65.09375, 100.5),
                ..Boat::default()
            },
            ..BoatBody::default()
        };
        assert!(get_ground_friction(&lily_world, b.boat.position).is_nan());
        tick_body(&mut b, &lily_world, zero, true, false, false);
        assert_eq!(b.status, Some(BoatStatus::InAir)); // check-extra.out:40
        assert!(b.on_ground);
        assert_eq!(b.boat.velocity.y.to_bits(), 0x0000_0000_0000_0000); // check-extra.out:40 (+0.0)
    }

    #[test]
    fn boat_mount_clamp_and_dismount_witnesses() {
        // interact / mount gates (adversarial-final.txt:3-6)
        assert!(can_mount(0.0, 0, false, false)); // A1
        assert!(!can_mount(0.0, 0, false, true)); // A2 sneaking
        assert!(!can_mount(60.0f32, 0, false, false)); // A3 outOfControlTicks == 60.0f
        assert!(can_mount(59.9999f32, 0, false, false)); // A4 outOfControlTicks == 59.9999f
        assert!(!can_mount(0.0, 2, false, false)); // run1.txt:22 capacity 2
        assert!(!can_mount(0.0, 0, true, false)); // eye in water

        // [player, cow] seat-0 priority (run1.txt:18, adversarial-final.txt:13)
        let mut seats = Vec::new();
        assert!(add_passenger_seat(&mut seats, "cow", false, |s| s == "player"));
        assert!(add_passenger_seat(&mut seats, "player", true, |s| s == "player"));
        assert_eq!(seats, vec!["player", "cow"]);
        assert!(!add_passenger_seat(&mut seats, "pig", false, |s| s == "player"));

        let mut seats2 = Vec::new();
        assert!(add_passenger_seat(&mut seats2, "player", true, |s| s == "player"));
        assert!(add_passenger_seat(&mut seats2, "cow", false, |s| s == "player"));
        assert_eq!(seats2, vec!["player", "cow"]);
        assert_eq!(start_riding_pose(), Pose::Standing);

        // clampRotation hull yaw 0 (run1.txt:3-12, Harness.java:186-191)
        // Note: yRot computes `(ry + f1) - f` so ry=105.1 yields (105.1 + 105.0) - 105.1 = 105.00001f32,
        // while yRotO computes `ry + (f1 - f)` = 105.0f32 (rounded to 105.0000 in run1.txt:7 %.4f).
        let cases_hull0 = [
            (0.0f32, 0.0f32, 0.0f32),
            (90.0, 90.0, 90.0),
            (104.9, 104.9, 104.9),
            (105.0, 105.0, 105.0),
            (105.1, (105.1f32 + 105.0f32) - math::wrap_degrees(105.1f32), 105.0),
            (120.0, 105.0, 105.0),
            (-120.0, -105.0, -105.0),
            (180.0, 255.0, 255.0),
            (-180.0, -105.0, -105.0),
            (480.0, 465.0, 465.0),
        ];
        for (ry, expected_y_rot, expected_y_rot_o) in cases_hull0 {
            let out = clamp_rotation(
                0.0,
                RiderRotation {
                    y_rot: ry,
                    y_rot_o: ry,
                    y_head_rot: ry,
                    y_body_rot: 0.0,
                },
            );
            assert_eq!(out.y_rot, expected_y_rot, "hull 0 ry={ry}");
            assert_eq!(out.y_rot_o, expected_y_rot_o, "hull 0 yRotO ry={ry}");
            assert_eq!(out.y_head_rot, expected_y_rot);
            assert_eq!(out.y_body_rot, 0.0);
        }

        // clampRotation hull yaw 200 (adversarial-final.txt:8-10)
        for (ry, expected) in [(0.0f32, -55.0f32), (480.0, 480.0), (95.0, 95.0)] {
            let out = clamp_rotation(
                200.0,
                RiderRotation {
                    y_rot: ry,
                    y_rot_o: ry,
                    y_head_rot: ry,
                    y_body_rot: 0.0,
                },
            );
            assert_eq!(out.y_rot, expected, "hull 200 ry={ry}");
            assert_eq!(out.y_body_rot, 200.0);
        }

        // positionRider deltaRotation +30 / -30 (run1.txt:15, adversarial-final.txt:12)
        let pos_p = position_rider_rotation(
            0.0,
            30.0,
            RiderRotation { y_rot: 90.0, y_rot_o: 90.0, y_head_rot: 90.0, y_body_rot: 0.0 },
            false,
        );
        assert_eq!(pos_p.y_rot, 105.0);
        let neg_p = position_rider_rotation(
            0.0,
            -30.0,
            RiderRotation { y_rot: -90.0, y_rot_o: -90.0, y_head_rot: -90.0, y_body_rot: 0.0 },
            false,
        );
        assert_eq!(neg_p.y_rot, -105.0);

        // Dismount scenarios (run1.txt:25-28, adversarial-final.txt:15-20)
        let mut open = BoxWorld::default();
        open.fill([-4, 63, -4], [4, 63, 4], Block::full());
        let (p, pose) = dismount_location(&open, Vec3::new(0.0, 64.0, 0.0), 0.0, 0.6);
        assert!((p.z - 1.272277).abs() < 1e-6);
        assert_eq!(p.y, 64.0);
        assert_eq!(pose, Some(Pose::Standing)); // run1.txt:25

        let (p90, pose90) = dismount_location(&open, Vec3::new(0.0, 64.0, 0.0), 90.0, 0.6);
        assert!((p90.x + 1.272277).abs() < 1e-6);
        assert_eq!(pose90, Some(Pose::Standing)); // adversarial-final.txt:15 (A8)

        let mut walled = open.clone();
        walled.set([0, 64, 1], Block::full());
        let (pw, posew) = dismount_location(&walled, Vec3::new(0.0, 64.0, 0.0), 0.0, 0.6);
        assert_eq!(pw, Vec3::new(0.0, 64.5625, 0.0));
        assert_eq!(posew, None); // run1.txt:26

        let mut ceil = open.clone();
        ceil.fill([-1, 65, 0], [1, 65, 2], Block::full());
        let (pc, posec) = dismount_location(&ceil, Vec3::new(0.0, 64.0, 0.0), 0.0, 0.6);
        assert_eq!(pc.y, 64.0);
        assert!((pc.z - 1.272277).abs() < 1e-6);
        assert_eq!(posec, Some(Pose::Swimming)); // run1.txt:27

        let mut water_below = BoxWorld::default();
        water_below.fill(
            [-4, 61, -4],
            [4, 63, 4],
            Block {
                air: false,
                fluid: Fluid {
                    kind: FluidKind::Water,
                    raw_amount: Some(8),
                    raw_falling: Some(false),
                    ..Fluid::default()
                },
                fluid_flow: Some(FluidFlowBlock::default()),
                ..Block::default()
            },
        );
        let (pwb, posewb) = dismount_location(&water_below, Vec3::new(0.0, 63.6875, 0.0), 0.0, 0.6);
        assert_eq!(pwb, Vec3::new(0.0, 64.25, 0.0)); // run1.txt:28
        assert_eq!(posewb, None);

        let mut slab_at = open.clone();
        slab_at.set([0, 64, 1], bottom_slab_block());
        let (ps, poses) = dismount_location(&slab_at, Vec3::new(0.0, 64.0, 0.0), 0.0, 0.6);
        assert_eq!(ps.y, 64.5);
        assert_eq!(poses, Some(Pose::Standing)); // adversarial-final.txt:16 (A9)

        let mut floor2_overhang = BoxWorld::default();
        floor2_overhang.fill([-4, 61, -4], [4, 61, 4], Block::full());
        floor2_overhang.set([0, 63, 1], Block::full());
        let (po, poseo) = dismount_location(&floor2_overhang, Vec3::new(0.0, 63.0, 0.0), 0.0, 0.6);
        assert_eq!(po.y, 62.0);
        assert_eq!(poseo, Some(Pose::Swimming)); // adversarial-final.txt:17 (A10)

        let mut slab_below = open;
        for c in [[-1, 0], [-1, 1], [0, 0], [0, 1]] {
            slab_below.set([c[0], 63, c[1]], bottom_slab_block());
        }
        let (psb, posesb) = dismount_location(&slab_below, Vec3::new(0.0, 64.0, 0.0), 0.0, 0.6);
        assert_eq!(psb.y, 63.5);
        assert_eq!(posesb, Some(Pose::Standing)); // adversarial-final.txt:20 (A13)
    }

    #[test]
    fn boat_trig_index_witnesses() {
        // Key divergent k/32° yaws from jar_bits.txt:1-9,440-447
        // (k, trig_bits [sin_neg, cos_neg, sin_pos, cos_pos], seat_bits, tick1_bits [vx,vy,vz,px,py,pz,yaw])
        let cases: &[(i32, [u32; 4], [u64; 3], [u64; 6], u32)] = &[
            (
                45, // jar_bits.txt:1-3 (negative-wrap index 65280)
                [3167292080, 1065348123, 1019808432, 1065348163],
                [4636772130339626777, 4634198738908427059, 13864614689191441235],
                [
                    13816154192835898234,
                    4564737724465753264,
                    4606541637762100491,
                    4636764548250828677,
                    4634204124823846811,
                    13864589042608645331,
                ],
                1074648842,
            ),
            (
                90, // jar_bits.txt:4-6
                [3175676720, 1065332928, 1028193072, 1065333007],
                [4636771785160565915, 4634198738908427059, 13864614695564686258],
                [
                    13816224841784748922,
                    4564737724465753264,
                    4606541187763612939,
                    4636764479257714565,
                    4634204124823846811,
                    13864589044366451923,
                ],
                1080547082,
            ),
            (
                11430, // jar_bits.txt:445-447
                [1028193072, 1065333007, 3175676720, 1065333007],
                [4636773166292884325, 4634198738908427059, 13864614695531551256],
                [
                    13815942014002391930,
                    4564737724465753264,
                    4606541686080482571,
                    4636764755456720773,
                    4634204124823846811,
                    13864589042419901651,
                ],
                1135804334,
            ),
        ];

        for &(k, trig_exp, seat_exp, tick_exp, yaw_exp) in cases {
            let yaw = k as f32 / 32.0f32;
            let rad_neg = (-yaw * 0.017453292f32) as f64;
            let rad_pos = (yaw * 0.017453292f32) as f64;
            assert_eq!(
                [
                    math::sin(rad_neg).to_bits(),
                    math::cos(rad_neg).to_bits(),
                    math::sin(rad_pos).to_bits(),
                    math::cos(rad_pos).to_bits(),
                ],
                trig_exp,
                "trig k={k}"
            );
            let s = seat(Vec3::new(100.5, 64.0, -200.25), yaw, 0.5625, 0.2f32);
            assert_eq!([s.x.to_bits(), s.y.to_bits(), s.z.to_bits()], seat_exp, "seat k={k}");
            let mut b = Boat {
                position: Vec3::new(100.5, 64.0, -200.25),
                velocity: Vec3::new(-0.123456789012345, -0.04, 0.987654321098765),
                yaw,
                delta_rotation: 0.9f32,
            };
            tick(&mut b, 64.75, Controls { forward: true, ..Default::default() });
            assert_eq!(
                [
                    b.velocity.x.to_bits(),
                    b.velocity.y.to_bits(),
                    b.velocity.z.to_bits(),
                    b.position.x.to_bits(),
                    b.position.y.to_bits(),
                    b.position.z.to_bits(),
                ],
                tick_exp,
                "tick1 k={k}"
            );
            assert_eq!(b.yaw.to_bits(), yaw_exp, "yaw k={k}");
        }

        // 800-tick W+A turn trace (jar_bits.txt:448)
        let mut b = Boat {
            position: Vec3::new(0.0, 64.0, 0.0),
            ..Default::default()
        };
        for _ in 0..800 {
            tick(&mut b, 64.75, Controls { left: true, forward: true, ..Default::default() });
        }
        let hs = (b.velocity.x * b.velocity.x + b.velocity.z * b.velocity.z).sqrt();
        assert_eq!(b.velocity.x.to_bits(), 13819241136836653065);
        assert_eq!(b.velocity.y.to_bits(), 4389087690360881152);
        assert_eq!(b.velocity.z.to_bits(), 4591209170509693186);
        assert_eq!(b.position.x.to_bits(), 4613697184607451226);
        assert_eq!(b.position.z.to_bits(), 4607290419979698702);
        assert_eq!(b.yaw.to_bits(), 3321311232);
        assert_eq!(hs.to_bits(), 4596625426452742353);
    }
}

