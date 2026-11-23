//! Interaction reach, eye ray / clip, and synchronous redstone interaction laws (26.2).

use crate::{blocks, world, Aabb, BlockPos, Vec3, VoxelShape};

/// `Player.DEFAULT_BLOCK_INTERACTION_RANGE` (`Attributes.BLOCK_INTERACTION_RANGE` base).
pub const DEFAULT_BLOCK_INTERACTION_RANGE: f64 = 4.5;
/// `Player.DEFAULT_ENTITY_INTERACTION_RANGE`.
pub const DEFAULT_ENTITY_INTERACTION_RANGE: f64 = 3.0;
/// `ServerPlayer.CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER` (`ADD_VALUE`).
pub const CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER: f64 = 0.5;
/// `ServerPlayer.CREATIVE_ENTITY_INTERACTION_RANGE_MODIFIER` (`ADD_VALUE`).
pub const CREATIVE_ENTITY_INTERACTION_RANGE_MODIFIER: f64 = 2.0;
/// Server-side tolerance passed to `isWithinBlockInteractionRange` by both
/// `ServerGamePacketListenerImpl.handleUseItemOn` and `ServerPlayerGameMode.handleBlockBreakAction`.
pub const SERVER_INTERACTION_RANGE_TOLERANCE: f64 = 1.0;

pub use crate::neighbor::Direction;

impl Direction {
    /// `Direction.BY_2D_DATA` order (`[SOUTH, WEST, NORTH, EAST]`).
    pub const BY_2D: [Self; 4] = [Self::South, Self::West, Self::North, Self::East];

    pub const fn clock_wise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::Down | Self::Up => panic!("unable to get Y-rotated facing of vertical direction"),
        }
    }

    pub const fn counter_clock_wise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            Self::Down | Self::Up => panic!("unable to get CCW facing of vertical direction"),
        }
    }

    pub const fn relative(self, pos: BlockPos) -> BlockPos {
        let [dx, dy, dz] = self.step();
        [pos[0] + dx, pos[1] + dy, pos[2] + dz]
    }

    /// `Direction.fromYRot(double)`.
    pub fn from_y_rot(yaw: f64) -> Self {
        let idx = ((yaw / 90.0 + 0.5).floor() as i32) & 3;
        Self::BY_2D[idx as usize]
    }

    /// `Direction.getApproximateNearest(double, double, double)`.
    pub fn get_approximate_nearest(x: f64, y: f64, z: f64) -> Self {
        Self::get_approximate_nearest_f32(x as f32, y as f32, z as f32)
    }

    /// `Direction.getApproximateNearest(float, float, float)` (`@0-95`).
    pub fn get_approximate_nearest_f32(x: f32, y: f32, z: f32) -> Self {
        let mut best = Self::North;
        let mut best_dot = f32::from_bits(1); // Float.MIN_VALUE = 1.4E-45f
        for dir in Self::ALL {
            let [nx, ny, nz] = dir.step();
            let dot = x * (nx as f32) + y * (ny as f32) + z * (nz as f32);
            if dot > best_dot {
                best_dot = dot;
                best = dir;
            }
        }
        best
    }

    /// `Direction.orderedByNearest(Entity)` (`@0-291`) using `Mth.sin`/`cos` table.
    pub fn ordered_by_nearest(pitch: f32, yaw: f32) -> [Self; 6] {
        let f = pitch * 0.017453292f32;
        let g = -yaw * 0.017453292f32;
        let h = crate::math::sin(f as f64);
        let i = crate::math::cos(f as f64);
        let j = crate::math::sin(g as f64);
        let k = crate::math::cos(g as f64);
        let flag = j > 0.0f32;
        let flag1 = h < 0.0f32;
        let flag2 = k > 0.0f32;
        let f2 = if flag { j } else { -j };
        let f3 = if flag1 { -h } else { h };
        let f4 = if flag2 { k } else { -k };
        let f5 = f2 * i;
        let f6 = f4 * i;
        let dx = if flag { Self::East } else { Self::West };
        let dy = if flag1 { Self::Up } else { Self::Down };
        let dz = if flag2 { Self::South } else { Self::North };
        let make = |a: Self, b: Self, c: Self| [a, b, c, c.opposite(), b.opposite(), a.opposite()];
        if f2 > f4 {
            if f3 > f5 {
                make(dy, dx, dz)
            } else if f6 > f3 {
                make(dx, dz, dy)
            } else {
                make(dx, dy, dz)
            }
        } else if f3 > f6 {
            make(dy, dz, dx)
        } else if f5 > f3 {
            make(dz, dx, dy)
        } else {
            make(dz, dy, dx)
        }
    }
}

/// All `Avatar.POSES` entries in 26.2 (`Pose.CRAWLING` does not exist; dry-land crawl is `Swimming`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityPose {
    Standing,
    Sleeping,
    FallFlying,
    Swimming,
    SpinAttack,
    Crouching,
    Dying,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

/// `Avatar.POSES` (`@21-177`) + `SLEEPING_DIMENSIONS`.
pub const fn pose_dimensions(pose: EntityPose) -> PoseDimensions {
    match pose {
        EntityPose::Standing => PoseDimensions { width: 0.6, height: 1.8, eye_height: 1.62 },
        EntityPose::Sleeping => PoseDimensions { width: 0.2, height: 0.2, eye_height: 0.2 },
        EntityPose::FallFlying | EntityPose::Swimming | EntityPose::SpinAttack => {
            PoseDimensions { width: 0.6, height: 0.6, eye_height: 0.4 }
        }
        EntityPose::Crouching => PoseDimensions { width: 0.6, height: 1.5, eye_height: 1.27 },
        EntityPose::Dying => PoseDimensions { width: 0.2, height: 0.2, eye_height: 1.62 },
    }
}

/// `Entity.getEyePosition()` (`@0-19`): uses the cached `eye_height` field.
pub fn get_eye_position(pos: Vec3, eye_height: f32) -> Vec3 {
    Vec3::new(pos.x, pos.y + eye_height as f64, pos.z)
}

/// `Entity.getEyePosition(float partial)` (`@0-62`).
pub fn get_eye_position_lerp(prev_pos: Vec3, pos: Vec3, eye_height: f32, partial: f32) -> Vec3 {
    let p = partial as f64;
    let x = prev_pos.x + p * (pos.x - prev_pos.x);
    let y = prev_pos.y + p * (pos.y - prev_pos.y) + eye_height as f64;
    let z = prev_pos.z + p * (pos.z - prev_pos.z);
    Vec3::new(x, y, z)
}

/// `AABB.distanceToSqr(Vec3)` (`@0-86`): per-axis `max(max(min - c, c - max), 0)` then sum of squares.
pub fn aabb_distance_to_sqr(aabb: Aabb, point: Vec3) -> f64 {
    let dx = (aabb.min.x - point.x).max(point.x - aabb.max.x).max(0.0);
    let dy = (aabb.min.y - point.y).max(point.y - aabb.max.y).max(0.0);
    let dz = (aabb.min.z - point.z).max(point.z - aabb.max.z).max(0.0);
    dx * dx + dy * dy + dz * dz
}

/// `Player.isWithinBlockInteractionRange(BlockPos, double)` (`@0-37`):
/// strict `<` after adding `tolerance` before squaring.
pub fn is_within_block_interaction_range(
    eye_pos: Vec3,
    block_interaction_range: f64,
    pos: BlockPos,
    tolerance: f64,
) -> bool {
    let d = block_interaction_range + tolerance;
    let box_pos = Aabb::new(
        pos[0] as f64,
        pos[1] as f64,
        pos[2] as f64,
        (pos[0] + 1) as f64,
        (pos[1] + 1) as f64,
        (pos[2] + 1) as f64,
    );
    aabb_distance_to_sqr(box_pos, eye_pos) < d * d
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipFluidMode {
    None,
    SourceOnly,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockHitResult {
    Hit {
        pos: BlockPos,
        face: Direction,
        location: Vec3,
        inside: bool,
    },
    Miss {
        pos: BlockPos,
        face: Direction,
        location: Vec3,
    },
}

impl BlockHitResult {
    pub fn location(self) -> Vec3 {
        match self {
            Self::Hit { location, .. } | Self::Miss { location, .. } => location,
        }
    }

    pub fn is_hit(self) -> bool {
        matches!(self, Self::Hit { .. })
    }
}

fn find_coord_index(coords: &[f64], value: f64) -> i32 {
    let mut lo = 0usize;
    let mut hi = coords.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if value >= coords[mid] {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo as i32 - 1
}

#[allow(clippy::too_many_arguments)]
fn clip_point(
    nearest: &mut f64,
    current: Option<Direction>,
    d_a: f64,
    d_u: f64,
    d_v: f64,
    plane: f64,
    u_min: f64,
    u_max: f64,
    v_min: f64,
    v_max: f64,
    face: Direction,
    f_a: f64,
    f_u: f64,
    f_v: f64,
) -> Option<Direction> {
    let t = (plane - f_a) / d_a;
    let pu = f_u + t * d_u;
    let pv = f_v + t * d_v;
    if 0.0 < t
        && t < *nearest
        && u_min - 1.0e-7 < pu
        && pu < u_max + 1.0e-7
        && v_min - 1.0e-7 < pv
        && pv < v_max + 1.0e-7
    {
        *nearest = t;
        Some(face)
    } else {
        current
    }
}

fn clip_aabb_with_face(
    aabb: Aabb,
    from: Vec3,
    to: Vec3,
    nearest: &mut f64,
    mut best: Option<Direction>,
) -> Option<Direction> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    if dx > 1.0e-7 {
        best = clip_point(
            nearest, best, dx, dy, dz, aabb.min.x, aabb.min.y, aabb.max.y, aabb.min.z,
            aabb.max.z, Direction::West, from.x, from.y, from.z,
        );
    } else if dx < -1.0e-7 {
        best = clip_point(
            nearest, best, dx, dy, dz, aabb.max.x, aabb.min.y, aabb.max.y, aabb.min.z,
            aabb.max.z, Direction::East, from.x, from.y, from.z,
        );
    }
    if dy > 1.0e-7 {
        best = clip_point(
            nearest, best, dy, dz, dx, aabb.min.y, aabb.min.z, aabb.max.z, aabb.min.x,
            aabb.max.x, Direction::Down, from.y, from.z, from.x,
        );
    } else if dy < -1.0e-7 {
        best = clip_point(
            nearest, best, dy, dz, dx, aabb.max.y, aabb.min.z, aabb.max.z, aabb.min.x,
            aabb.max.x, Direction::Up, from.y, from.z, from.x,
        );
    }
    if dz > 1.0e-7 {
        best = clip_point(
            nearest, best, dz, dx, dy, aabb.min.z, aabb.min.x, aabb.max.x, aabb.min.y,
            aabb.max.y, Direction::North, from.z, from.x, from.y,
        );
    } else if dz < -1.0e-7 {
        best = clip_point(
            nearest, best, dz, dx, dy, aabb.max.z, aabb.min.x, aabb.max.x, aabb.min.y,
            aabb.max.y, Direction::South, from.z, from.x, from.y,
        );
    }
    best
}

/// `VoxelShape.clip(Vec3, Vec3, BlockPos)` (`@0-151`):
/// checks `isFullWide` at `from + (to - from) * 0.001`; otherwise clips boxes with un-nudged `from`.
pub fn clip_voxel_shape(
    shape: &VoxelShape,
    pos: BlockPos,
    from: Vec3,
    to: Vec3,
) -> Option<BlockHitResult> {
    if shape.is_empty() {
        return None;
    }
    let delta = to.minus(from);
    if delta.length_sqr() < 1.0e-7 {
        return None;
    }
    let nudge = from.plus(delta.scale(0.001));
    let ix = find_coord_index(&shape.coords[0], nudge.x - pos[0] as f64);
    let iy = find_coord_index(&shape.coords[1], nudge.y - pos[1] as f64);
    let iz = find_coord_index(&shape.coords[2], nudge.z - pos[2] as f64);
    if ix >= 0 && iy >= 0 && iz >= 0 && shape.is_full([ix as usize, iy as usize, iz as usize]) {
        return Some(BlockHitResult::Hit {
            pos,
            face: Direction::get_approximate_nearest(delta.x, delta.y, delta.z).opposite(),
            location: nudge,
            inside: true,
        });
    }

    let nx = shape.coords[0].len().saturating_sub(1);
    let ny = shape.coords[1].len().saturating_sub(1);
    let nz = shape.coords[2].len().saturating_sub(1);
    let mut nearest = 1.0;
    let mut best = None;
    let ox = pos[0] as f64;
    let oy = pos[1] as f64;
    let oz = pos[2] as f64;
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                if !shape.is_full([x, y, z]) {
                    continue;
                }
                let cell = Aabb::new(
                    shape.coords[0][x] + ox,
                    shape.coords[1][y] + oy,
                    shape.coords[2][z] + oz,
                    shape.coords[0][x + 1] + ox,
                    shape.coords[1][y + 1] + oy,
                    shape.coords[2][z + 1] + oz,
                );
                best = clip_aabb_with_face(cell, from, to, &mut nearest, best);
            }
        }
    }
    best.map(|face| BlockHitResult::Hit {
        pos,
        face,
        location: Vec3::new(
            from.x + nearest * delta.x,
            from.y + nearest * delta.y,
            from.z + nearest * delta.z,
        ),
        inside: false,
    })
}

fn dist_sqr(a: Vec3, b: Vec3) -> f64 {
    a.minus(b).length_sqr()
}

/// `BlockGetter.clip` (`traverseBlocks` + `clipWithInteractionOverride` + fluid compare + backward miss).
pub fn clip_blocks<'a>(
    from: Vec3,
    to: Vec3,
    fluid_mode: ClipFluidMode,
    mut cell_shapes: impl FnMut(BlockPos) -> (&'a VoxelShape, Option<&'a VoxelShape>, f32, bool),
) -> BlockHitResult {
    let mut hit: Option<BlockHitResult> = None;
    blocks::traverse_blocks(from, to, |pos| {
        let (block_shape, interaction_shape, fluid_height, fluid_is_source) = cell_shapes(pos);
        let mut block_hit = clip_voxel_shape(block_shape, pos, from, to);
        if let Some(inter) = interaction_shape {
            if let Some(inter_hit) = clip_voxel_shape(inter, pos, from, to) {
                let replace = block_hit.is_none_or(|bh| {
                    dist_sqr(from, inter_hit.location()) < dist_sqr(from, bh.location())
                });
                if replace {
                    block_hit = Some(inter_hit);
                }
            }
        }
        let fluid_hit = match fluid_mode {
            ClipFluidMode::None => None,
            ClipFluidMode::SourceOnly if !fluid_is_source => None,
            ClipFluidMode::SourceOnly | ClipFluidMode::Any => {
                if fluid_height > 0.0 {
                    let fh = VoxelShape::boxes(&[Aabb::new(
                        0.0,
                        0.0,
                        0.0,
                        1.0,
                        fluid_height as f64,
                        1.0,
                    )]);
                    clip_voxel_shape(&fh, pos, from, to)
                } else {
                    None
                }
            }
        };
        let d_block = block_hit.map_or(f64::MAX, |h| dist_sqr(from, h.location()));
        let d_fluid = fluid_hit.map_or(f64::MAX, |h| dist_sqr(from, h.location()));
        let chosen = if d_block <= d_fluid { block_hit } else { fluid_hit };
        if chosen.is_some() {
            hit = chosen;
            true
        } else {
            false
        }
    });
    hit.unwrap_or_else(|| {
        let backward = from.minus(to);
        BlockHitResult::Miss {
            face: Direction::get_approximate_nearest(backward.x, backward.y, backward.z),
            location: to,
            pos: world::block_pos(to),
        }
    })
}

/// `BlockPlaceContext.getNearestLookingDirections()` (`@0-69`):
/// `Direction.orderedByNearest`, rotating `clicked_face.opposite()` to index 0 when `!replace_clicked`.
pub fn nearest_looking_directions(
    pitch: f32,
    yaw: f32,
    clicked_face: Direction,
    replace_clicked: bool,
) -> [Direction; 6] {
    let mut dirs = Direction::ordered_by_nearest(pitch, yaw);
    if replace_clicked {
        return dirs;
    }
    let opp = clicked_face.opposite();
    let mut idx = 0;
    while idx < dirs.len() && dirs[idx] != opp {
        idx += 1;
    }
    if idx > 0 && idx < dirs.len() {
        dirs.copy_within(0..idx, 1);
        dirs[0] = opp;
    }
    dirs
}

/// Opaque block-type identity from the block table (never matched by block name).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachFace {
    Floor,
    Wall,
    Ceiling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorHalf {
    Lower,
    Upper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeverState {
    pub block_id: BlockId,
    pub face: AttachFace,
    pub facing: Direction,
    pub powered: bool,
}

impl LeverState {
    /// `FaceAttachedHorizontalDirectionalBlock.getConnectedDirection` (`@132-154`).
    pub const fn connected_direction(self) -> Direction {
        match self.face {
            AttachFace::Ceiling => Direction::Down,
            AttachFace::Floor => Direction::Up,
            AttachFace::Wall => self.facing,
        }
    }

    /// `LeverBlock.updateNeighbours`: support cell is `pos.relative(getConnectedDirection(state).opposite())`.
    pub const fn support_pos(self, pos: BlockPos) -> BlockPos {
        self.connected_direction().opposite().relative(pos)
    }

    /// `LeverBlock.getSignal` (`@0-22`): `15` to all sides when powered.
    pub const fn signal(self) -> u8 {
        if self.powered { 15 } else { 0 }
    }

    /// `LeverBlock.getDirectSignal` (`@0-29`): `15` only toward `getConnectedDirection`.
    pub const fn direct_signal(self, dir: Direction) -> u8 {
        if self.powered && self.connected_direction() as u8 == dir as u8 {
            15
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedstoneDoorState {
    pub block_id: BlockId,
    pub half: DoorHalf,
    pub open: bool,
    pub powered: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrapDoorState {
    pub block_id: BlockId,
    pub open: bool,
    pub powered: bool,
    pub waterlogged: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FenceGateState {
    pub block_id: BlockId,
    pub open: bool,
    pub powered: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedstoneCellKind {
    Empty,
    Lever(LeverState),
    Door(RedstoneDoorState),
    TrapDoor(TrapDoorState),
    FenceGate(FenceGateState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedstoneCell {
    pub kind: RedstoneCellKind,
    pub is_redstone_conductor: bool,
}

impl Default for RedstoneCell {
    fn default() -> Self {
        Self {
            kind: RedstoneCellKind::Empty,
            is_redstone_conductor: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedstoneEvent {
    SetBlock {
        pos: BlockPos,
        flags: u8,
        open: Option<bool>,
        powered: bool,
        waterlogged: Option<bool>,
    },
    UpdateNeighborsAt {
        pos: BlockPos,
        source_id: BlockId,
    },
    DoorSoundAndEvent {
        pos: BlockPos,
        open: bool,
    },
    TrapDoorSoundAndEvent {
        pos: BlockPos,
        open: bool,
    },
    FenceGateSoundAndEvent {
        pos: BlockPos,
        open: bool,
    },
    LeverSoundAndEvent {
        pos: BlockPos,
        activated: bool,
    },
    ScheduleWaterTick {
        pos: BlockPos,
        delay: u32,
    },
}

/// `NeighborUpdater.UPDATE_ORDER` (`@0-44`): `{WEST, EAST, DOWN, UP, NORTH, SOUTH}`.
pub const NEIGHBOR_UPDATE_ORDER: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
];

/// `Fluids.WATER.getTickDelay(level)` scheduled by `TrapDoorBlock.neighborChanged` when waterlogged.
pub const WATERLOGGED_TRAPDOOR_TICK_DELAY: u32 = 5;

pub trait RedstoneWorld {
    fn cell(&self, pos: BlockPos) -> RedstoneCell;
    fn set_cell(&mut self, pos: BlockPos, cell: RedstoneCell);
}

/// `SignalGetter.getDirectSignalTo(BlockPos)`.
pub fn get_direct_signal_to(world: &impl RedstoneWorld, pos: BlockPos) -> u8 {
    let mut best = 0u8;
    for dir in Direction::ALL {
        let neighbor = world.cell(dir.relative(pos));
        let sig = match neighbor.kind {
            RedstoneCellKind::Lever(l) => l.direct_signal(dir),
            _ => 0,
        };
        if sig >= 15 {
            return 15;
        }
        if sig > best {
            best = sig;
        }
    }
    best
}

/// `SignalGetter.getSignal(BlockPos, Direction)` (`@0-41`).
pub fn get_signal(world: &impl RedstoneWorld, pos: BlockPos, _dir: Direction) -> u8 {
    let cell = world.cell(pos);
    let own = match cell.kind {
        RedstoneCellKind::Lever(l) => l.signal(),
        _ => 0,
    };
    if cell.is_redstone_conductor {
        own.max(get_direct_signal_to(world, pos))
    } else {
        own
    }
}

/// `SignalGetter.hasNeighborSignal(BlockPos)` (`@0-111`): scans `Direction.VALUES`.
pub fn has_neighbor_signal(world: &impl RedstoneWorld, pos: BlockPos) -> bool {
    for dir in Direction::ALL {
        if get_signal(world, dir.relative(pos), dir) > 0 {
            return true;
        }
    }
    false
}

/// `DoorBlock.neighborChanged` (`@0-161`).
pub fn door_neighbor_changed(
    world: &mut impl RedstoneWorld,
    pos: BlockPos,
    source_id: BlockId,
    events: &mut Vec<RedstoneEvent>,
) {
    let cell = world.cell(pos);
    let RedstoneCellKind::Door(mut door) = cell.kind else {
        return;
    };
    let other_half_pos = match door.half {
        DoorHalf::Lower => Direction::Up.relative(pos),
        DoorHalf::Upper => Direction::Down.relative(pos),
    };
    let has_signal = has_neighbor_signal(world, pos) || has_neighbor_signal(world, other_half_pos);
    if source_id == door.block_id {
        return;
    }
    if has_signal == door.powered {
        return;
    }
    if has_signal != door.open {
        events.push(RedstoneEvent::DoorSoundAndEvent { pos, open: has_signal });
    }
    door.powered = has_signal;
    door.open = has_signal;
    world.set_cell(pos, RedstoneCell { kind: RedstoneCellKind::Door(door), ..cell });
    events.push(RedstoneEvent::SetBlock {
        pos,
        flags: 2,
        open: Some(door.open),
        powered: has_signal,
        waterlogged: None,
    });
}

/// `TrapDoorBlock.neighborChanged` (`@0-129`).
pub fn trapdoor_neighbor_changed(
    world: &mut impl RedstoneWorld,
    pos: BlockPos,
    is_client_side: bool,
    events: &mut Vec<RedstoneEvent>,
) {
    if is_client_side {
        return;
    }
    let cell = world.cell(pos);
    let RedstoneCellKind::TrapDoor(mut trap) = cell.kind else {
        return;
    };
    let has_signal = has_neighbor_signal(world, pos);
    if has_signal == trap.powered {
        return;
    }
    if trap.open != has_signal {
        trap.open = has_signal;
        events.push(RedstoneEvent::TrapDoorSoundAndEvent { pos, open: has_signal });
    }
    trap.powered = has_signal;
    world.set_cell(pos, RedstoneCell { kind: RedstoneCellKind::TrapDoor(trap), ..cell });
    events.push(RedstoneEvent::SetBlock {
        pos,
        flags: 2,
        open: Some(trap.open),
        powered: has_signal,
        waterlogged: Some(trap.waterlogged),
    });
    if trap.waterlogged {
        events.push(RedstoneEvent::ScheduleWaterTick {
            pos,
            delay: WATERLOGGED_TRAPDOOR_TICK_DELAY,
        });
    }
}

/// `FenceGateBlock.neighborChanged` (`@0-154`): writes BEFORE sound/event.
pub fn fence_gate_neighbor_changed(
    world: &mut impl RedstoneWorld,
    pos: BlockPos,
    is_client_side: bool,
    events: &mut Vec<RedstoneEvent>,
) {
    if is_client_side {
        return;
    }
    let cell = world.cell(pos);
    let RedstoneCellKind::FenceGate(mut gate) = cell.kind else {
        return;
    };
    let has_signal = has_neighbor_signal(world, pos);
    if has_signal == gate.powered {
        return;
    }
    let old_open = gate.open;
    gate.powered = has_signal;
    gate.open = has_signal;
    world.set_cell(pos, RedstoneCell { kind: RedstoneCellKind::FenceGate(gate), ..cell });
    events.push(RedstoneEvent::SetBlock {
        pos,
        flags: 2,
        open: Some(gate.open),
        powered: has_signal,
        waterlogged: None,
    });
    if old_open != has_signal {
        events.push(RedstoneEvent::FenceGateSoundAndEvent { pos, open: has_signal });
    }
}

fn dispatch_neighbor_updates(
    world: &mut impl RedstoneWorld,
    center: BlockPos,
    source_id: BlockId,
    is_client_side: bool,
    events: &mut Vec<RedstoneEvent>,
) {
    events.push(RedstoneEvent::UpdateNeighborsAt { pos: center, source_id });
    for dir in NEIGHBOR_UPDATE_ORDER {
        let neighbor_pos = dir.relative(center);
        match world.cell(neighbor_pos).kind {
            RedstoneCellKind::Door(_) => door_neighbor_changed(world, neighbor_pos, source_id, events),
            RedstoneCellKind::TrapDoor(_) => {
                trapdoor_neighbor_changed(world, neighbor_pos, is_client_side, events);
            }
            RedstoneCellKind::FenceGate(_) => {
                fence_gate_neighbor_changed(world, neighbor_pos, is_client_side, events);
            }
            RedstoneCellKind::Empty | RedstoneCellKind::Lever(_) => {}
        }
    }
}

/// `LeverBlock.pull` (`@0-66`) + `LeverBlock.updateNeighbours` (`@0-65`):
/// flips `powered` (`flags = 3`), synchronously notifies `pos` and `support_pos`, then emits sound/event.
pub fn lever_pull(
    world: &mut impl RedstoneWorld,
    pos: BlockPos,
    is_client_side: bool,
    events: &mut Vec<RedstoneEvent>,
) -> bool {
    let cell = world.cell(pos);
    let RedstoneCellKind::Lever(mut lever) = cell.kind else {
        return false;
    };
    lever.powered = !lever.powered;
    world.set_cell(pos, RedstoneCell { kind: RedstoneCellKind::Lever(lever), ..cell });
    events.push(RedstoneEvent::SetBlock {
        pos,
        flags: 3,
        open: None,
        powered: lever.powered,
        waterlogged: None,
    });
    let support = lever.support_pos(pos);
    dispatch_neighbor_updates(world, pos, lever.block_id, is_client_side, events);
    dispatch_neighbor_updates(world, support, lever.block_id, is_client_side, events);
    events.push(RedstoneEvent::LeverSoundAndEvent {
        pos,
        activated: lever.powered,
    });
    lever.powered
}

#[cfg(test)]
mod part1_tests {
    use super::*;
    use crate::FluidKind;
    use crate::world::{Block, BoxWorld, Fluid, World};

    #[test]
    fn reach_boundaries_and_pose_eye_basis() {
        // witness-run.txt:2-8 (Avatar.POSES)
        assert_eq!(pose_dimensions(EntityPose::Standing), PoseDimensions { width: 0.6, height: 1.8, eye_height: 1.62 });
        assert_eq!(pose_dimensions(EntityPose::Sleeping), PoseDimensions { width: 0.2, height: 0.2, eye_height: 0.2 });
        assert_eq!(pose_dimensions(EntityPose::FallFlying), PoseDimensions { width: 0.6, height: 0.6, eye_height: 0.4 });
        assert_eq!(pose_dimensions(EntityPose::Swimming), PoseDimensions { width: 0.6, height: 0.6, eye_height: 0.4 });
        assert_eq!(pose_dimensions(EntityPose::SpinAttack), PoseDimensions { width: 0.6, height: 0.6, eye_height: 0.4 });
        assert_eq!(pose_dimensions(EntityPose::Crouching), PoseDimensions { width: 0.6, height: 1.5, eye_height: 1.27 });
        assert_eq!(pose_dimensions(EntityPose::Dying), PoseDimensions { width: 0.2, height: 0.2, eye_height: 1.62 });

        // witness-run.txt:10-17 (Runner.java:109-147)
        let target = [0, 65, 1];
        let eye_exact = get_eye_position(Vec3::new(0.5, 64.0, -4.5), 1.62f32);
        assert!(!is_within_block_interaction_range(eye_exact, 4.5, target, 1.0));
        let eye_inside = get_eye_position(Vec3::new(0.5, 64.0, f64::from_bits(0xc011_ffff_ffff_fffc)), 1.62f32);
        assert!(is_within_block_interaction_range(eye_inside, 4.5, target, 1.0));

        // Bisection boundaries (witness-run.txt:13-15)
        for (eh, first_true, last_false) in [
            (1.62f32, 0xc011_ffff_ffff_ffffu64, 0xc012_0000_0000_0000u64),
            (1.27f32, 0xc011_ffff_ffff_ffffu64, 0xc012_0000_0000_0000u64),
            (0.4f32, 0xc011_de63_105b_c19bu64, 0xc011_de63_105b_c19cu64),
        ] {
            assert!(is_within_block_interaction_range(
                get_eye_position(Vec3::new(0.5, 64.0, f64::from_bits(first_true)), eh),
                4.5,
                target,
                1.0,
            ));
            assert!(!is_within_block_interaction_range(
                get_eye_position(Vec3::new(0.5, 64.0, f64::from_bits(last_false)), eh),
                4.5,
                target,
                1.0,
            ));
        }

        // Creative +0.5 -> 5.0 + 1.0 = 6.0 exact tie false (witness-run.txt:16-17)
        let creative_eye = get_eye_position(Vec3::new(0.5, 64.0, -5.0), 1.62f32);
        assert!(!is_within_block_interaction_range(
            creative_eye,
            DEFAULT_BLOCK_INTERACTION_RANGE + CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER,
            target,
            SERVER_INTERACTION_RANGE_TOLERANCE,
        ));

        // Vertical exact tie with tol=0 (check2-out.txt:2-3)
        let block_y = [0, 64, 0];
        let eye_y_tie = Vec3::new(0.5, 69.5, 0.5);
        assert!(!is_within_block_interaction_range(eye_y_tie, 4.5, block_y, 0.0));
        let eye_y_in = Vec3::new(0.5, f64::from_bits(0x4051_5fff_ffff_ffff), 0.5);
        assert!(is_within_block_interaction_range(eye_y_in, 4.5, block_y, 0.0));

        // Eye-basis divergence on below-band block (check2-out.txt:4-5)
        let below = [0, 61, 1];
        assert!(!is_within_block_interaction_range(
            get_eye_position(Vec3::new(0.5, 64.0, -3.5), 1.62f32),
            4.5,
            below,
            1.0,
        ));
        assert!(is_within_block_interaction_range(
            get_eye_position(Vec3::new(0.5, 64.0, -3.5), 0.4f32),
            4.5,
            below,
            1.0,
        ));
    }

    #[test]
    fn view_math_directions_and_clip_witnesses() {
        // witness-run.txt:21-27
        let v00 = crate::math::calculate_view_vector(0.0, 0.0);
        assert_eq!(v00.x.to_bits(), 0);
        assert_eq!(v00.y.to_bits(), 0x8000_0000_0000_0000); // -0.0
        assert_eq!(v00.z.to_bits(), 0x3ff0_0000_0000_0000);
        let ep = get_eye_position_lerp(Vec3::new(0.5, 64.0, 0.5), Vec3::new(0.5, 64.0, 0.5), 1.62f32, 1.0);
        assert_eq!(ep.y.to_bits(), 0x4050_67ae_1480_0000);

        // Table trig truncation at 45 deg (witness-run.txt:26, check2-out.txt:6-13)
        let d45 = -(45.0f32 * 0.017453292f32) as f64;
        assert_eq!(crate::math::sin(d45).to_bits(), 0xbf35_04f3);
        assert_eq!(crate::math::cos(d45).to_bits(), 0x3f35_0082);

        // orderedByNearest (witness-run.txt:24,27; check-adversarial.txt:25)
        // Note: LivingEntity.getViewYRot(1.0f) reads yHeadRot (`ev/bc-LivingEntity.txt:9114-9129`);
        // Runner.java:182 and Check.java set only Entity.yRot on Unsafe-allocated ServerPlayer,
        // so yHeadRot remained 0.0f in those non-zero-yaw witness rows.
        assert_eq!(
            Direction::ordered_by_nearest(0.0, 0.0),
            [Direction::South, Direction::Down, Direction::West, Direction::East, Direction::Up, Direction::North]
        );
        assert_eq!(
            Direction::ordered_by_nearest(-90.0, 0.0),
            [Direction::Up, Direction::South, Direction::West, Direction::East, Direction::North, Direction::Down]
        );
        assert_eq!(
            Direction::ordered_by_nearest(90.0, 0.0),
            [Direction::Down, Direction::South, Direction::West, Direction::East, Direction::North, Direction::Up]
        );
        // When view_yaw = 45.0, |sin(-45°)| = 0.70710677 > |cos(-45°)| = 0.707039 (`witness-run.txt:26`),
        // so orderedByNearest puts West first:
        assert_eq!(
            Direction::ordered_by_nearest(0.0, 45.0),
            [Direction::West, Direction::South, Direction::Down, Direction::Up, Direction::North, Direction::East]
        );

        // getDirection(yaw) (witness-run.txt:28-35)
        assert_eq!(Direction::from_y_rot(0.0), Direction::South);
        assert_eq!(Direction::from_y_rot(44.9f32 as f64), Direction::South);
        assert_eq!(Direction::from_y_rot(45.0), Direction::West);
        assert_eq!(Direction::from_y_rot(45.1f32 as f64), Direction::West);
        assert_eq!(Direction::from_y_rot(90.0), Direction::West);
        assert_eq!(Direction::from_y_rot(179.0), Direction::North);
        assert_eq!(Direction::from_y_rot(180.0), Direction::North);
        assert_eq!(Direction::from_y_rot(-90.0), Direction::East);

        // Clip witnesses (witness-run.txt:39-45, check-adversarial.txt:14-20)
        let mut w = BoxWorld::default();
        w.blocks.insert([0, 0, 0], Block::full());
        w.blocks.insert([1, 0, 0], Block::full());

        let clip_w = |world: &BoxWorld, from: Vec3, to: Vec3, fluid: ClipFluidMode| {
            clip_blocks(from, to, fluid, |pos| {
                let b = world.block(pos);
                let fh = if b.fluid.kind != FluidKind::Empty {
                    b.fluid.surface_height(world, pos)
                } else {
                    0.0
                };
                (&b.shape, None, fh, b.fluid.raw_amount == Some(8))
            })
        };

        assert_eq!(
            clip_w(&w, Vec3::new(0.5, 0.5, -5.0), Vec3::new(0.5, 0.5, 5.0), ClipFluidMode::None),
            BlockHitResult::Hit { pos: [0, 0, 0], face: Direction::North, location: Vec3::new(0.5, 0.5, 0.0), inside: false }
        );
        // exact shared edge x=1.0 -> (1, 0, 0) (witness-run.txt:40)
        let r2 = clip_w(&w, Vec3::new(1.0, 0.5, -5.0), Vec3::new(1.0, 0.5, 5.0), ClipFluidMode::None);
        assert_eq!(
            r2,
            BlockHitResult::Hit {
                pos: [1, 0, 0],
                face: Direction::North,
                location: Vec3::new(f64::from_bits(0x3ff0_0000_0000_0000), 0.5, 0.0),
                inside: false,
            }
        );
        // corner diagonal (witness-run.txt:42)
        assert_eq!(
            clip_w(&w, Vec3::new(-0.5, -0.5, -0.5), Vec3::new(3.5, 3.5, 3.5), ClipFluidMode::None),
            BlockHitResult::Hit { pos: [0, 0, 0], face: Direction::West, location: Vec3::new(0.0, 0.0, 0.0), inside: false }
        );
        // true miss -> backward direction (witness-run.txt:43)
        let empty_w = BoxWorld::default();
        assert_eq!(
            clip_w(&empty_w, Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0), ClipFluidMode::None),
            BlockHitResult::Miss { pos: [10, 0, 0], face: Direction::West, location: Vec3::new(10.0, 0.0, 0.0) }
        );
        // on_corner +X -> 0.001 nudge (witness-run.txt:44)
        let r6 = clip_w(&w, Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0), ClipFluidMode::None);
        assert_eq!(
            r6,
            BlockHitResult::Hit {
                pos: [0, 0, 0],
                face: Direction::West,
                location: Vec3::new(f64::from_bits(0x3f84_7ae1_47ae_147b), 0.0, 0.0),
                inside: true,
            }
        );
        // check-adversarial.txt:17-20
        let diag = Vec3::new(1.0, 0.0, 1.0).normalize();
        let miss_from = Vec3::new(0.5, 0.5, 0.5).plus(diag.scale(10.0)).scale(-1.0).plus(Vec3::new(0.5, 0.0, 0.5));
        let miss_to = Vec3::new(0.5, 0.5, 0.5).plus(diag.scale(10.0));
        assert!(matches!(
            clip_w(&empty_w, miss_from, miss_to, ClipFluidMode::None),
            BlockHitResult::Miss { face: Direction::North, .. }
        ));
        let mut w_corner = BoxWorld::default();
        w_corner.blocks.insert([0, 0, 0], Block::full());
        let len5 = clip_w(&w_corner, Vec3::new(0.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 0.0), ClipFluidMode::None);
        assert_eq!(len5.location().x.to_bits(), 0x3f74_7ae1_47ae_147b);
        assert_eq!(
            clip_w(&w_corner, Vec3::new(0.0, 0.0, 0.0), Vec3::new(-5.0, 0.0, 0.0), ClipFluidMode::None),
            BlockHitResult::Miss { pos: [-5, 0, 0], face: Direction::East, location: Vec3::new(-5.0, 0.0, 0.0) }
        );
        w_corner.blocks.insert([-1, 0, 0], Block::full());
        assert!(matches!(
            clip_w(&w_corner, Vec3::new(-1.0, 0.5, -5.0), Vec3::new(-1.0, 0.5, 5.0), ClipFluidMode::None),
            BlockHitResult::Hit { pos: [-1, 0, 0], face: Direction::North, .. }
        ));

        // water_cell NONE vs ANY (witness-run.txt:45)
        let mut w_water = BoxWorld::default();
        w_water.blocks.insert([0, 0, 0], Block {
            fluid: Fluid { kind: FluidKind::Water, raw_amount: Some(8), raw_falling: Some(false), ..Fluid::default() },
            ..Block::default()
        });
        assert!(matches!(
            clip_w(&w_water, Vec3::new(0.5, 2.0, 0.5), Vec3::new(0.5, -1.0, 0.5), ClipFluidMode::None),
            BlockHitResult::Miss { face: Direction::Up, .. }
        ));
        let f_any = clip_w(&w_water, Vec3::new(0.5, 2.0, 0.5), Vec3::new(0.5, -1.0, 0.5), ClipFluidMode::Any);
        assert!(matches!(
            f_any,
            BlockHitResult::Hit { pos: [0, 0, 0], face: Direction::Up, .. }
        ));
        assert_eq!(f_any.location().y.to_bits(), 0x3fec_71c7_2000_0000);
    }
}

#[cfg(test)]
mod part6_tests {
    use super::*;
    use std::collections::HashMap;

    const STONE_ID: BlockId = BlockId(1);
    const LEVER_ID: BlockId = BlockId(2);
    const IRON_DOOR_ID: BlockId = BlockId(3);
    const OAK_DOOR_ID: BlockId = BlockId(4);
    const COPPER_DOOR_ID: BlockId = BlockId(5);
    const IRON_TRAPDOOR_ID: BlockId = BlockId(6);
    const OAK_FENCE_GATE_ID: BlockId = BlockId(7);

    #[derive(Default)]
    struct MapRedstoneWorld {
        cells: HashMap<BlockPos, RedstoneCell>,
    }

    impl RedstoneWorld for MapRedstoneWorld {
        fn cell(&self, pos: BlockPos) -> RedstoneCell {
            self.cells.get(&pos).copied().unwrap_or_default()
        }

        fn set_cell(&mut self, pos: BlockPos, cell: RedstoneCell) {
            self.cells.insert(pos, cell);
        }
    }

    fn conductor() -> RedstoneCell {
        RedstoneCell { kind: RedstoneCellKind::Empty, is_redstone_conductor: true }
    }

    fn lever(face: AttachFace, facing: Direction, powered: bool) -> RedstoneCell {
        RedstoneCell {
            kind: RedstoneCellKind::Lever(LeverState {
                block_id: LEVER_ID,
                face,
                facing,
                powered,
            }),
            is_redstone_conductor: false,
        }
    }

    fn door(block_id: BlockId, half: DoorHalf, open: bool, powered: bool) -> RedstoneCell {
        RedstoneCell {
            kind: RedstoneCellKind::Door(RedstoneDoorState {
                block_id,
                half,
                open,
                powered,
            }),
            is_redstone_conductor: false,
        }
    }

    fn trapdoor(open: bool, powered: bool, waterlogged: bool) -> RedstoneCell {
        RedstoneCell {
            kind: RedstoneCellKind::TrapDoor(TrapDoorState {
                block_id: IRON_TRAPDOOR_ID,
                open,
                powered,
                waterlogged,
            }),
            is_redstone_conductor: false,
        }
    }

    fn gate(open: bool, powered: bool) -> RedstoneCell {
        RedstoneCell {
            kind: RedstoneCellKind::FenceGate(FenceGateState {
                block_id: OAK_FENCE_GATE_ID,
                open,
                powered,
            }),
            is_redstone_conductor: false,
        }
    }

    #[test]
    fn lever_pull_support_cells_and_signal_table() {
        // expected-harness-output.txt:10-27 (E1 wall, E1b floor) & checker-output.txt:12-25 (X3 ceiling, X7 signals)
        let mut w = MapRedstoneWorld::default();
        w.set_cell([0, 64, 0], lever(AttachFace::Wall, Direction::East, false));
        let mut ev = Vec::new();
        assert!(lever_pull(&mut w, [0, 64, 0], false, &mut ev));
        assert_eq!(
            ev,
            vec![
                RedstoneEvent::SetBlock { pos: [0, 64, 0], flags: 3, open: None, powered: true, waterlogged: None },
                RedstoneEvent::UpdateNeighborsAt { pos: [0, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::UpdateNeighborsAt { pos: [-1, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::LeverSoundAndEvent { pos: [0, 64, 0], activated: true },
            ]
        );
        // X7 signal table (checker-output.txt:18-25)
        let RedstoneCellKind::Lever(l_on) = w.cell([0, 64, 0]).kind else { unreachable!() };
        assert_eq!(l_on.signal(), 15);
        for d in Direction::ALL {
            assert_eq!(l_on.direct_signal(d), if d == Direction::East { 15 } else { 0 });
        }
        ev.clear();
        assert!(!lever_pull(&mut w, [0, 64, 0], false, &mut ev));
        assert_eq!(ev[0], RedstoneEvent::SetBlock { pos: [0, 64, 0], flags: 3, open: None, powered: false, waterlogged: None });

        // Floor lever (expected-harness-output.txt:23-27)
        w.set_cell([5, 64, 0], lever(AttachFace::Floor, Direction::North, false));
        ev.clear();
        lever_pull(&mut w, [5, 64, 0], false, &mut ev);
        assert_eq!(ev[1], RedstoneEvent::UpdateNeighborsAt { pos: [5, 64, 0], source_id: LEVER_ID });
        assert_eq!(ev[2], RedstoneEvent::UpdateNeighborsAt { pos: [5, 63, 0], source_id: LEVER_ID });

        // Ceiling lever (checker-output.txt:13-17)
        w.set_cell([7, 64, 0], lever(AttachFace::Ceiling, Direction::North, false));
        ev.clear();
        lever_pull(&mut w, [7, 64, 0], false, &mut ev);
        assert_eq!(ev[2], RedstoneEvent::UpdateNeighborsAt { pos: [7, 65, 0], source_id: LEVER_ID });
    }

    #[test]
    fn synchronous_lever_to_door_and_trapdoor_interleaving() {
        // expected-harness-output.txt:28-49 (E2)
        let mut w = MapRedstoneWorld::default();
        w.set_cell([0, 64, 0], lever(AttachFace::Wall, Direction::East, false));
        w.set_cell([-1, 64, 0], conductor());
        w.set_cell([-2, 64, 0], door(IRON_DOOR_ID, DoorHalf::Lower, false, false));
        w.set_cell([-2, 65, 0], door(IRON_DOOR_ID, DoorHalf::Upper, false, false));

        let mut ev = Vec::new();
        lever_pull(&mut w, [0, 64, 0], false, &mut ev);
        assert_eq!(
            ev,
            vec![
                RedstoneEvent::SetBlock { pos: [0, 64, 0], flags: 3, open: None, powered: true, waterlogged: None },
                RedstoneEvent::UpdateNeighborsAt { pos: [0, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::UpdateNeighborsAt { pos: [-1, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::DoorSoundAndEvent { pos: [-2, 64, 0], open: true },
                RedstoneEvent::SetBlock { pos: [-2, 64, 0], flags: 2, open: Some(true), powered: true, waterlogged: None },
                RedstoneEvent::LeverSoundAndEvent { pos: [0, 64, 0], activated: true },
            ]
        );
        // Only lower half written by neighborChanged; upper stays unchanged until shape wave (expected-harness-output.txt:38-39)
        assert_eq!(w.cell([-2, 65, 0]), door(IRON_DOOR_ID, DoorHalf::Upper, false, false));

        ev.clear();
        lever_pull(&mut w, [0, 64, 0], false, &mut ev);
        assert_eq!(
            ev,
            vec![
                RedstoneEvent::SetBlock { pos: [0, 64, 0], flags: 3, open: None, powered: false, waterlogged: None },
                RedstoneEvent::UpdateNeighborsAt { pos: [0, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::UpdateNeighborsAt { pos: [-1, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::DoorSoundAndEvent { pos: [-2, 64, 0], open: false },
                RedstoneEvent::SetBlock { pos: [-2, 64, 0], flags: 2, open: Some(false), powered: false, waterlogged: None },
                RedstoneEvent::LeverSoundAndEvent { pos: [0, 64, 0], activated: false },
            ]
        );

        // X10 pull -> stone -> iron trapdoor (checker-output.txt:37-54) & X11 door far at -3 (checker-output.txt:56-62)
        let mut wt = MapRedstoneWorld::default();
        wt.set_cell([0, 64, 0], lever(AttachFace::Wall, Direction::East, false));
        wt.set_cell([-1, 64, 0], conductor());
        wt.set_cell([-2, 64, 0], trapdoor(false, false, false));
        wt.set_cell([-3, 64, 0], door(IRON_DOOR_ID, DoorHalf::Lower, false, false));
        ev.clear();
        lever_pull(&mut wt, [0, 64, 0], false, &mut ev);
        assert_eq!(
            ev,
            vec![
                RedstoneEvent::SetBlock { pos: [0, 64, 0], flags: 3, open: None, powered: true, waterlogged: None },
                RedstoneEvent::UpdateNeighborsAt { pos: [0, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::UpdateNeighborsAt { pos: [-1, 64, 0], source_id: LEVER_ID },
                RedstoneEvent::TrapDoorSoundAndEvent { pos: [-2, 64, 0], open: true },
                RedstoneEvent::SetBlock { pos: [-2, 64, 0], flags: 2, open: Some(true), powered: true, waterlogged: Some(false) },
                RedstoneEvent::LeverSoundAndEvent { pos: [0, 64, 0], activated: true },
            ]
        );
        // X11: door at -3 untouched
        assert_eq!(wt.cell([-3, 64, 0]), door(IRON_DOOR_ID, DoorHalf::Lower, false, false));
    }

    #[test]
    fn door_trapdoor_fence_gate_neighbor_changed_matrix() {
        // expected-harness-output.txt:50-92 (E3a..E3h2) and checker-output.txt:1-36 (X1, X6, X4, X5, X5b)
        let mut ev = Vec::new();

        // 3b: upper updates from lower signal; X6: lower updates from upper signal
        let mut w = MapRedstoneWorld::default();
        w.set_cell([2, 64, 0], door(IRON_DOOR_ID, DoorHalf::Lower, false, false));
        w.set_cell([2, 65, 0], door(IRON_DOOR_ID, DoorHalf::Upper, false, false));
        w.set_cell([3, 64, 0], lever(AttachFace::Floor, Direction::North, true));
        door_neighbor_changed(&mut w, [2, 65, 0], STONE_ID, &mut ev);
        assert_eq!(ev, vec![
            RedstoneEvent::DoorSoundAndEvent { pos: [2, 65, 0], open: true },
            RedstoneEvent::SetBlock { pos: [2, 65, 0], flags: 2, open: Some(true), powered: true, waterlogged: None },
        ]);

        // 3d: same-door skip; X1: cross-material (oak_door) does NOT skip
        ev.clear();
        door_neighbor_changed(&mut w, [2, 64, 0], IRON_DOOR_ID, &mut ev);
        assert!(ev.is_empty());
        door_neighbor_changed(&mut w, [2, 64, 0], OAK_DOOR_ID, &mut ev);
        assert_eq!(ev.len(), 2);

        // 3e: pre-opened copper door + signal ON -> no sound, write open=true powered=true
        let mut wc = MapRedstoneWorld::default();
        wc.set_cell([2, 64, 0], door(COPPER_DOOR_ID, DoorHalf::Lower, true, false));
        wc.set_cell([3, 64, 0], lever(AttachFace::Floor, Direction::North, true));
        ev.clear();
        door_neighbor_changed(&mut wc, [2, 64, 0], STONE_ID, &mut ev);
        assert_eq!(ev, vec![
            RedstoneEvent::SetBlock { pos: [2, 64, 0], flags: 2, open: Some(true), powered: true, waterlogged: None },
        ]);

        // 3g3 & X5/X5b: waterlogged trapdoor ON, held (noop), OFF
        let mut wt = MapRedstoneWorld::default();
        wt.set_cell([4, 64, 0], trapdoor(false, false, true));
        wt.set_cell([5, 64, 0], lever(AttachFace::Floor, Direction::North, true));
        ev.clear();
        trapdoor_neighbor_changed(&mut wt, [4, 64, 0], false, &mut ev);
        assert_eq!(ev, vec![
            RedstoneEvent::TrapDoorSoundAndEvent { pos: [4, 64, 0], open: true },
            RedstoneEvent::SetBlock { pos: [4, 64, 0], flags: 2, open: Some(true), powered: true, waterlogged: Some(true) },
            RedstoneEvent::ScheduleWaterTick { pos: [4, 64, 0], delay: 5 },
        ]);
        // X5: held signal -> empty (no scheduleTick)
        ev.clear();
        trapdoor_neighbor_changed(&mut wt, [4, 64, 0], false, &mut ev);
        assert!(ev.is_empty());
        // X5b: signal OFF -> close sound, write, scheduleTick delay=5
        wt.set_cell([5, 64, 0], lever(AttachFace::Floor, Direction::North, false));
        trapdoor_neighbor_changed(&mut wt, [4, 64, 0], false, &mut ev);
        assert_eq!(ev, vec![
            RedstoneEvent::TrapDoorSoundAndEvent { pos: [4, 64, 0], open: false },
            RedstoneEvent::SetBlock { pos: [4, 64, 0], flags: 2, open: Some(false), powered: false, waterlogged: Some(true) },
            RedstoneEvent::ScheduleWaterTick { pos: [4, 64, 0], delay: 5 },
        ]);

        // 3h & X4: fence gate writes BEFORE sound, and skips sound when pre-opened
        let mut wg = MapRedstoneWorld::default();
        wg.set_cell([6, 64, 0], gate(false, false));
        wg.set_cell([7, 64, 0], lever(AttachFace::Floor, Direction::North, true));
        ev.clear();
        fence_gate_neighbor_changed(&mut wg, [6, 64, 0], false, &mut ev);
        assert_eq!(ev, vec![
            RedstoneEvent::SetBlock { pos: [6, 64, 0], flags: 2, open: Some(true), powered: true, waterlogged: None },
            RedstoneEvent::FenceGateSoundAndEvent { pos: [6, 64, 0], open: true },
        ]);
        // X4: pre-opened fence gate + signal ON -> write only, no sound
        wg.set_cell([6, 64, 0], gate(true, false));
        ev.clear();
        fence_gate_neighbor_changed(&mut wg, [6, 64, 0], false, &mut ev);
        assert_eq!(ev, vec![
            RedstoneEvent::SetBlock { pos: [6, 64, 0], flags: 2, open: Some(true), powered: true, waterlogged: None },
        ]);
    }
}

