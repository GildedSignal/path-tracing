//! Nether-portal creation bounds, destination scaling/clamping, transit geometry,
//! and cooldown handling (Mojang 26.2 `PortalForcer`, `NetherPortalBlock`, `PortalShape`,
//! `PositionMoveRotation`, `Entity.handlePortal`).

use crate::{math, Vec3};

pub type BlockPos = [i32; 3];

/// `WorldBorder.clampToBounds` horizontal epsilon (`1.0E-5f` as `double`).
pub const BORDER_CLAMP_EPS: f64 = 9.999999747378752e-6;

/// Nether/Overworld default horizontal border extent for portal clamping.
pub const DEFAULT_WORLD_BORDER_RADIUS: f64 = 29_999_984.0;

/// `Entity.getDimensionChangingDelay @19-22` (plain entity with no `ServerPlayer` passenger).
pub const PORTAL_COOLDOWN_ENTITY: i32 = 300;
/// `Player.getDimensionChangingDelay @0-2` (also inherited by `ServerPlayer`).
pub const PORTAL_COOLDOWN_PLAYER: i32 = 10;
/// `VehicleEntity.getDimensionChangingDelay @0-2`.
pub const PORTAL_COOLDOWN_VEHICLE: i32 = 10;
/// `GameRules.PLAYERS_NETHER_PORTAL_DEFAULT_DELAY` (80; post-increment triggers on 81st inside tick).
pub const NETHER_PORTAL_DEFAULT_DELAY_TICKS: i32 = 80;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimensionHeightBounds {
    pub min_y: i32,
    pub max_y: i32,
    pub logical_height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBorderBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl WorldBorderBounds {
    pub const fn default_mc() -> Self {
        Self {
            min_x: -DEFAULT_WORLD_BORDER_RADIUS,
            max_x: DEFAULT_WORLD_BORDER_RADIUS,
            min_z: -DEFAULT_WORLD_BORDER_RADIUS,
            max_z: DEFAULT_WORLD_BORDER_RADIUS,
        }
    }

    pub fn from_center_size(center_x: f64, center_z: f64, size: f64) -> Self {
        let half = size / 2.0;
        Self {
            min_x: center_x - half,
            max_x: center_x + half,
            min_z: center_z - half,
            max_z: center_z + half,
        }
    }

    /// `WorldBorder.clampToBounds(double, double, double)` -> `BlockPos.containing`.
    /// Clamps X and Z into `[min, max - BORDER_CLAMP_EPS]` and leaves Y untouched before flooring.
    pub fn clamp_vec_to_bounds(self, pos: Vec3) -> BlockPos {
        let x = pos.x.clamp(self.min_x, self.max_x - BORDER_CLAMP_EPS);
        let z = pos.z.clamp(self.min_z, self.max_z - BORDER_CLAMP_EPS);
        [math::floor(x), math::floor(pos.y), math::floor(z)]
    }

    /// `WorldBorder.clampToBounds(BlockPos)` (`BlockPos.containing` via `Mth.floor`).
    /// Note: does not alter Y; runs AFTER fallback Y-clamping in `PortalForcer.createPortal`.
    pub fn clamp_to_bounds(self, pos: BlockPos) -> BlockPos {
        self.clamp_vec_to_bounds(Vec3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64))
    }
}

/// `Entity.getDimensionChangingDelay`, `Player.getDimensionChangingDelay`, `VehicleEntity.getDimensionChangingDelay`.
pub const fn dimension_changing_delay(
    is_player_or_vehicle: bool,
    first_passenger_is_server_player: bool,
) -> i32 {
    if is_player_or_vehicle || first_passenger_is_server_player {
        PORTAL_COOLDOWN_PLAYER
    } else {
        PORTAL_COOLDOWN_ENTITY
    }
}

/// `DimensionType.getTeleportationScale @170-181`: `from.coordinateScale / to.coordinateScale`.
/// Overworld (`1.0`) -> Nether (`8.0`) is `0.125`; Nether -> Overworld is `8.0`.
pub fn teleportation_scale(from_scale: f64, to_scale: f64) -> f64 {
    from_scale / to_scale
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalAxis {
    X,
    Z,
}

impl PortalAxis {
    pub const fn other(self) -> Self {
        match self {
            Self::X => Self::Z,
            Self::Z => Self::X,
        }
    }
}

/// `BlockUtil.FoundRectangle`: portal rectangle around an entry/exit block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoundRectangle {
    pub min_corner: BlockPos,
    pub axis1_size: i32,
    pub axis2_size: i32,
}

impl FoundRectangle {
    const fn axis_coord(self, axis: PortalAxis) -> i32 {
        match axis {
            PortalAxis::X => self.min_corner[0],
            PortalAxis::Z => self.min_corner[2],
        }
    }
}

fn vec_axis_coord(pos: Vec3, axis: PortalAxis) -> f64 {
    match axis {
        PortalAxis::X => pos.x,
        PortalAxis::Z => pos.z,
    }
}

/// `PortalShape.getRelativePosition @0-185`: relative `(across, up, deep)` in the entry portal.
pub fn relative_position(
    rect: FoundRectangle,
    axis: PortalAxis,
    pos: Vec3,
    width: f32,
    height: f32,
) -> Vec3 {
    let across_room = rect.axis1_size as f64 - width as f64;
    let up_room = rect.axis2_size as f64 - height as f64;
    let across = if across_room > 0.0 {
        let origin = rect.axis_coord(axis) as f64 + width as f64 / 2.0;
        ((vec_axis_coord(pos, axis) - origin - 0.0) / (across_room - 0.0)).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let up = if up_room > 0.0 {
        ((pos.y - rect.min_corner[1] as f64 - 0.0) / (up_room - 0.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let other = axis.other();
    let deep = vec_axis_coord(pos, other) - (rect.axis_coord(other) as f64 + 0.5);
    Vec3::new(across, up, deep)
}

/// `NetherPortalBlock.createDimensionTransition @73-195`: candidate exit feet position
/// inside the target `FoundRectangle` (identical to final on a collision-free portal).
pub fn exit_position(
    rect: FoundRectangle,
    exit_axis: PortalAxis,
    rel: Vec3,
    width: f32,
    height: f32,
) -> Vec3 {
    let across = width as f64 / 2.0 + (rect.axis1_size as f64 - width as f64) * rel.x;
    let up = (rect.axis2_size as f64 - height as f64) * rel.y;
    let deep = 0.5 + rel.z;
    let [mx, my, mz] = rect.min_corner;
    if exit_axis == PortalAxis::X {
        Vec3::new(mx as f64 + across, my as f64 + up, mz as f64 + deep)
    } else {
        Vec3::new(mx as f64 + deep, my as f64 + up, mz as f64 + across)
    }
}

/// `NetherPortalBlock.createDimensionTransition @59-71`: `0.0f` when axes match, `90.0f` on `X <-> Z`.
pub const fn yaw_offset(entry_axis: PortalAxis, exit_axis: PortalAxis) -> f32 {
    match (entry_axis, exit_axis) {
        (PortalAxis::X, PortalAxis::X) | (PortalAxis::Z, PortalAxis::Z) => 0.0,
        (PortalAxis::X, PortalAxis::Z) | (PortalAxis::Z, PortalAxis::X) => 90.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortalTransit {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

/// Java `Math.toRadians(double)`: `deg / 180.0 * PI`.
fn to_radians(deg: f64) -> f64 {
    deg / 180.0 * std::f64::consts::PI
}

/// `Vec3.xRot(float)` using `Mth.cos/sin`.
fn rotate_x(v: Vec3, angle: f32) -> Vec3 {
    let cos = math::cos(angle as f64) as f64;
    let sin = math::sin(angle as f64) as f64;
    Vec3::new(v.x, v.y * cos + v.z * sin, v.z * cos - v.y * sin)
}

/// `Vec3.yRot(float)` using `Mth.cos/sin`.
fn rotate_y(v: Vec3, angle: f32) -> Vec3 {
    let cos = math::cos(angle as f64) as f64;
    let sin = math::sin(angle as f64) as f64;
    Vec3::new(v.x * cos + v.z * sin, v.y, v.z * cos - v.x * sin)
}

/// `PositionMoveRotation.calculateAbsolute @92-240` for `Relative.union(DELTA, ROTATION)`:
/// unwrapped `yaw + yaw_offset`, clamped `pitch.clamp(-90, 90)`, velocity rotated via
/// `xRot(toRadians(oldPitch - newPitch)).yRot(toRadians(oldYaw - newYaw))`.
pub fn transit(
    entry: PortalTransit,
    entry_axis: PortalAxis,
    exit_axis: PortalAxis,
    exit_pos: Vec3,
) -> PortalTransit {
    let yaw = entry.yaw + yaw_offset(entry_axis, exit_axis);
    let pitch = entry.pitch.clamp(-90.0, 90.0);
    let x_rot_angle = to_radians((entry.pitch - pitch) as f64) as f32;
    let y_rot_angle = to_radians((entry.yaw - yaw) as f64) as f32;
    let velocity = rotate_y(rotate_x(entry.velocity, x_rot_angle), y_rot_angle);
    PortalTransit {
        position: exit_pos,
        velocity,
        yaw,
        pitch,
    }
}

/// `PortalProcessor` + `Entity.portalCooldown` state (`Entity.setAsInsidePortal @0-78`,
/// `Entity.handlePortal @0-148`, `PortalProcessor.processPortalTeleportation @0-56`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PortalProcessState {
    pub portal_cooldown: i32,
    pub has_processor: bool,
    pub entry_pos: BlockPos,
    pub portal_time: i32,
    pub inside_this_tick: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalHandleOutcome {
    Idle,
    Counting { portal_time: i32 },
    Decayed { portal_time: i32, expired: bool },
    Teleported,
    NullDestinationLockedOut,
}

impl PortalProcessState {
    /// `Entity.setAsInsidePortal`: if `portal_cooldown > 0`, resets cooldown and skips processor update.
    pub fn set_inside_portal(&mut self, pos: BlockPos, same_portal: bool, cooldown_delay: i32) {
        if self.portal_cooldown > 0 {
            self.portal_cooldown = cooldown_delay;
            return;
        }
        if !self.has_processor || !same_portal {
            self.has_processor = true;
            self.entry_pos = pos;
            self.portal_time = 0;
            self.inside_this_tick = true;
        } else if !self.inside_this_tick {
            self.entry_pos = pos;
            self.inside_this_tick = true;
        }
    }

    /// `Entity.handlePortal`: decrements cooldown, steps `PortalProcessor`, and on threshold
    /// sets `portal_cooldown = cooldown_delay` EVEN if `destination_exists` is `false` (leaving
    /// `has_processor = true` so the entity retries after cooldown expires).
    pub fn handle_portal(
        &mut self,
        can_use_portal: bool,
        transition_time: i32,
        cooldown_delay: i32,
        destination_exists: bool,
    ) -> PortalHandleOutcome {
        if self.portal_cooldown > 0 {
            self.portal_cooldown -= 1;
        }
        if !self.has_processor {
            return PortalHandleOutcome::Idle;
        }
        if self.inside_this_tick {
            self.inside_this_tick = false;
            if can_use_portal {
                let prev = self.portal_time;
                self.portal_time += 1;
                if prev >= transition_time {
                    self.portal_cooldown = cooldown_delay;
                    return if destination_exists {
                        PortalHandleOutcome::Teleported
                    } else {
                        PortalHandleOutcome::NullDestinationLockedOut
                    };
                }
            }
            return PortalHandleOutcome::Counting {
                portal_time: self.portal_time,
            };
        }
        self.portal_time = (self.portal_time - 4).max(0);
        let expired = self.portal_time <= 0;
        if expired {
            self.has_processor = false;
        }
        PortalHandleOutcome::Decayed {
            portal_time: self.portal_time,
            expired,
        }
    }
}

/// Top inclusive usable Y in `PortalForcer.createPortal` (`@33-57`):
/// `Math.min(level.getMaxY(), level.getMinY() + level.getLogicalHeight() - 1)`.
pub const fn top_usable(dims: DimensionHeightBounds) -> i32 {
    let logical_top = dims.min_y + dims.logical_height - 1;
    if dims.max_y < logical_top {
        dims.max_y
    } else {
        logical_top
    }
}

/// Fallback minimum Y (`@452-459`): `Math.max(70, level.getMinY() + 1)`.
pub const fn fallback_floor(min_y: i32) -> i32 {
    let min_plus_one = min_y + 1;
    if 70 > min_plus_one {
        70
    } else {
        min_plus_one
    }
}

/// Fallback ceiling Y (`@464-469`): `top_usable - 9`.
pub const fn fallback_ceiling(dims: DimensionHeightBounds) -> i32 {
    top_usable(dims) - 9
}

/// `PortalForcer.createPortal` fallback loop base (`@443-497`).
/// Assumes the primary search found no valid slot (`bestPos == null`).
/// Returns `None` without touching `border` if `ceiling < floor`.
pub fn create_portal_fallback_base(
    entry_pos: BlockPos,
    axis_step: [i32; 2],
    dims: DimensionHeightBounds,
    border: WorldBorderBounds,
) -> Option<BlockPos> {
    let floor_y = fallback_floor(dims.min_y);
    let ceil_y = fallback_ceiling(dims);
    if ceil_y < floor_y {
        return None;
    }
    let clamped_y = entry_pos[1].clamp(floor_y, ceil_y);
    let offset = [
        entry_pos[0] - axis_step[0],
        clamped_y,
        entry_pos[2] - axis_step[1],
    ];
    Some(border.clamp_to_bounds(offset))
}


// ---------------------------------------------------------------------------
// PortalForcer.findClosestPortalPosition / createPortal (26.2 spiral rewrite).
//
// `findClosestPortalPosition(pos, isNether, border)`: radius `isNether ? 16 : 128`;
// `PoiManager.getInSquare` keeps records with `|dx| <= r && |dz| <= r`; then
// `.filter(border::isWithinBounds).filter(hasProperty(HORIZONTAL_AXIS))` and
// `.min(comparingDouble(distSqr).thenComparingInt(getY))` (no getLargestRectangleAround).
// `createPortal(pos, axis)`: `BlockPos.spiralAround(pos, 16, EAST, SOUTH)` (1089 columns),
// `canHostFrame` at offsets 0 (candidate) and -1/+1 (full candidate), `distSqr` decides;
// no candidate -> emergency fallback box; then the 4x5 obsidian ring (setBlock flag 3) and
// the 2x3 portal (flag 18). The fallback base reuses `create_portal_fallback_base`.
// ---------------------------------------------------------------------------

pub const NETHER_PORTAL_SEARCH_RADIUS: i32 = 16;
pub const OVERWORLD_PORTAL_SEARCH_RADIUS: i32 = 128;
pub const CREATE_PORTAL_SPIRAL_RADIUS: i32 = 16;

/// `Vec3i.distSqr` in double arithmetic, as the jar computes it.
pub fn dist_sqr(a: BlockPos, b: BlockPos) -> f64 {
    let dx = (a[0] - b[0]) as f64;
    let dy = (a[1] - b[1]) as f64;
    let dz = (a[2] - b[2]) as f64;
    dx * dx + dy * dy + dz * dz
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalBlock {
    Obsidian,
    Air,
    /// NetherPortalBlock with the createPortal axis.
    Portal(PortalAxis),
}

/// What `PortalForcer` asks of the world; answers are the kit-class answers.
pub trait PortalWorld {
    fn dims(&self) -> DimensionHeightBounds;
    fn border(&self) -> WorldBorderBounds;
    /// MOTION_BLOCKING heightmap value at the column.
    fn height(&self, x: i32, z: i32) -> i32;
    fn within_border(&self, pos: BlockPos) -> bool;
    /// canPortalReplaceBlock: canBeReplaced && fluid empty.
    fn replaceable(&self, pos: BlockPos) -> bool;
    /// BlockState.isSolid.
    fn solid(&self, pos: BlockPos) -> bool;
    /// BlockState.hasProperty(HORIZONTAL_AXIS), the POI filter.
    fn has_horizontal_axis(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, block: PortalBlock, flag: i32);
}

fn axis_step(axis: PortalAxis) -> (i32, i32) {
    match axis {
        PortalAxis::X => (1, 0),
        PortalAxis::Z => (0, 1),
    }
}

/// `Direction.get(POSITIVE, axis).getClockWise()`: EAST->SOUTH `(0,1)`, SOUTH->WEST `(-1,0)`.
fn clockwise_step(axis: PortalAxis) -> (i32, i32) {
    match axis {
        PortalAxis::X => (0, 1),
        PortalAxis::Z => (-1, 0),
    }
}

/// `candidates` are the NETHER_PORTAL POI records of the engine's store.
pub fn find_closest_portal_position(
    world: &impl PortalWorld,
    pos: BlockPos,
    is_nether: bool,
    candidates: &[BlockPos],
) -> Option<BlockPos> {
    let radius = if is_nether { NETHER_PORTAL_SEARCH_RADIUS } else { OVERWORLD_PORTAL_SEARCH_RADIUS };
    let mut best: Option<BlockPos> = None;
    for &p in candidates {
        if (p[0] - pos[0]).abs() > radius || (p[2] - pos[2]).abs() > radius {
            continue;
        }
        if !world.within_border(p) || !world.has_horizontal_axis(p) {
            continue;
        }
        // Stream.min keeps the first on comparator ties: replace only on a strict win.
        let take = match best {
            None => true,
            Some(b) => {
                let (d_new, d_old) = (dist_sqr(p, pos), dist_sqr(b, pos));
                d_new < d_old || (d_new == d_old && p[1] < b[1])
            }
        };
        if take {
            best = Some(p);
        }
    }
    best
}

/// `BlockPos.spiralAround(center, radius, EAST, SOUTH)`: start one step SOUTH, legs
/// [EAST, SOUTH, WEST, NORTH] with sizes 1,1,2,2,3,3,...
pub fn spiral_around(center: BlockPos, radius: i32) -> Vec<BlockPos> {
    let (mut x, mut z) = (center[0], center[2] + 1);
    let mut leg: i32 = -1;
    let mut leg_size: i32 = 0;
    let mut leg_index: i32 = 0;
    let legs = 4 * radius;
    let mut out = Vec::new();
    loop {
        match ((leg + 4) % 4 + 4) % 4 {
            3 => z -= 1,
            0 => x += 1,
            1 => z += 1,
            _ => x -= 1,
        }
        if leg_index >= leg_size {
            if leg >= legs {
                break;
            }
            leg += 1;
            leg_index = 0;
            leg_size = leg / 2 + 1;
        }
        leg_index += 1;
        out.push([x, center[1], z]);
    }
    out
}

fn can_host_frame(world: &impl PortalWorld, pos: BlockPos, dir: PortalAxis, offset: i32) -> bool {
    let (dx, dz) = axis_step(dir);
    let (cx, cz) = clockwise_step(dir);
    for a in -1..3i32 {
        for b in -1..4i32 {
            let p = [pos[0] + a * dx + offset * cx, pos[1] + b, pos[2] + a * dz + offset * cz];
            if b < 0 && !world.solid(p) {
                return false;
            }
            if b >= 0 && !world.replaceable(p) {
                return false;
            }
        }
    }
    true
}

/// `PortalForcer.createPortal(BlockPos, Axis)`: every block write in the jar's order.
pub fn create_portal(world: &mut impl PortalWorld, pos: BlockPos, axis: PortalAxis) -> Option<FoundRectangle> {
    let (dx, dz) = axis_step(axis);
    let dims = world.dims();
    let mut d0 = -1.0f64;
    let mut best: Option<BlockPos> = None;
    let mut d1 = -1.0f64;
    let mut fallback_best: Option<BlockPos> = None;
    let top = top_usable(dims);
    for m in spiral_around(pos, CREATE_PORTAL_SPIRAL_RADIUS) {
        let j = top.min(world.height(m[0], m[2]));
        if !world.within_border(m) || !world.within_border([m[0] + dx, m[1], m[2] + dz]) {
            continue;
        }
        let mut m = m;
        let mut k = j;
        while k >= dims.min_y {
            m[1] = k;
            if !world.replaceable(m) {
                k -= 1;
                continue;
            }
            let orig_y = k;
            while k > dims.min_y && world.replaceable([m[0], k - 1, m[2]]) {
                k -= 1;
            }
            if k + 4 > top {
                k -= 1;
                continue;
            }
            let d = orig_y - k;
            if d > 0 && d < 3 {
                k -= 1;
                continue;
            }
            m[1] = k;
            if !can_host_frame(world, m, axis, 0) {
                k -= 1;
                continue;
            }
            let dist = dist_sqr(pos, m);
            if can_host_frame(world, m, axis, -1) && can_host_frame(world, m, axis, 1) && (d0 == -1.0 || d0 > dist) {
                d0 = dist;
                best = Some(m);
            }
            if d0 == -1.0 && (d1 == -1.0 || d1 > dist) {
                d1 = dist;
                fallback_best = Some(m);
            }
            k -= 1;
        }
    }
    if d0 == -1.0 && d1 != -1.0 {
        best = fallback_best;
    }
    let base = match best {
        Some(b) => b,
        None => {
            let p = create_portal_fallback_base(pos, [dx, dz], dims, world.border())?;
            let (cx, cz) = clockwise_step(axis);
            for b in -1..2i32 {
                for a in 0..2i32 {
                    for y in -1..3i32 {
                        let block = if y < 0 { PortalBlock::Obsidian } else { PortalBlock::Air };
                        world.set_block([p[0] + a * dx + b * cx, p[1] + y, p[2] + a * dz + b * cz], block, 3);
                    }
                }
            }
            p
        }
    };
    for a in -1..3i32 {
        for y in -1..4i32 {
            if a == -1 || a == 2 || y == -1 || y == 3 {
                world.set_block([base[0] + a * dx, base[1] + y, base[2] + a * dz], PortalBlock::Obsidian, 3);
            }
        }
    }
    for a in 0..2i32 {
        for y in 0..3i32 {
            world.set_block([base[0] + a * dx, base[1] + y, base[2] + a * dz], PortalBlock::Portal(axis), 18);
        }
    }
    Some(FoundRectangle { min_corner: base, axis1_size: 2, axis2_size: 3 })
}
#[cfg(test)]
mod part12_tests {
    use super::*;

    #[test]
    fn part12_portal_create_ceiling_and_fallback_witnesses() {
        let default_border = WorldBorderBounds::default_mc();
        let axis_x_step = [1, 0];

        // harness-output.txt:1-7 & check-output.txt:1-5
        let cases: &[(&str, DimensionHeightBounds, WorldBorderBounds, BlockPos, Option<BlockPos>)] = &[
            (
                "case1-nether",
                DimensionHeightBounds { min_y: 0, max_y: 255, logical_height: 128 },
                default_border,
                [0, 200, 0],
                Some([-1, 118, 0]),
            ),
            (
                "case2a-overworld-low",
                DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                default_border,
                [0, 40, 0],
                Some([-1, 70, 0]),
            ),
            (
                "case2b-overworld-high",
                DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                default_border,
                [0, 999, 0],
                Some([-1, 310, 0]),
            ),
            (
                "case3-maxY-leg",
                DimensionHeightBounds { min_y: 0, max_y: 256, logical_height: 256 },
                default_border,
                [0, 999, 0],
                Some([-1, 246, 0]),
            ),
            (
                "case4-empty",
                DimensionHeightBounds { min_y: 0, max_y: 78, logical_height: 128 },
                default_border,
                [0, 70, 0],
                None,
            ),
            (
                "case6-empty-gate",
                DimensionHeightBounds { min_y: 0, max_y: 319, logical_height: 78 },
                default_border,
                [0, 999, 0],
                None,
            ),
            (
                "A-ceiling-eq-floor",
                DimensionHeightBounds { min_y: 0, max_y: 79, logical_height: 128 },
                default_border,
                [0, 999, 0],
                Some([-1, 70, 0]),
            ),
            (
                "B-floor-leg-minY100",
                DimensionHeightBounds { min_y: 100, max_y: 319, logical_height: 384 },
                default_border,
                [0, 50, 0],
                Some([-1, 101, 0]),
            ),
            (
                "C-mid-passthrough",
                DimensionHeightBounds { min_y: 0, max_y: 319, logical_height: 384 },
                default_border,
                [0, 150, 0],
                Some([-1, 150, 0]),
            ),
            (
                "D-small-border-far",
                DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                WorldBorderBounds::from_center_size(0.0, 0.0, 10.0),
                [1000, 999, 1000],
                Some([4, 310, 4]),
            ),
            (
                "E-floor-gt-ceiling",
                DimensionHeightBounds { min_y: 90, max_y: 99, logical_height: 128 },
                default_border,
                [0, 999, 0],
                None,
            ),
        ];

        for &(name, dims, border, entry, expected) in cases {
            assert_eq!(
                create_portal_fallback_base(entry, axis_x_step, dims, border),
                expected,
                "{name}"
            );
        }
    }
}
