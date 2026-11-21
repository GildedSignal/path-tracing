//! Position-offset collision shapes (`BambooStalkBlock`, `SpeleothemBlock`), ShulkerBox
//! shape/animation/open/break rules, and non-player/world block hooks (Mojang 26.2).
//!
//! Block identity is strictly data-driven (`BlockId` and properties, never block-name strings).

use crate::interact::{BlockId, Direction};
use crate::neighbor::update_flags;
use crate::{Aabb, BlockPos, CollisionContext, Vec3, VoxelShape, World};

#[inline]
const fn aabb(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Aabb {
    Aabb {
        min: Vec3::new(x0, y0, z0),
        max: Vec3::new(x1, y1, z1),
    }
}

// ============================================================================
// 1. Position-offset XZ collision (Mth.getSeed, Bamboo, Speleothem / SulfurSpike)
// ============================================================================

/// `BlockBehaviour.getMaxHorizontalOffset()` default (`0.25f`), used by `BambooStalkBlock`.
pub const BAMBOO_MAX_HORIZONTAL_OFFSET: f32 = 0.25;
/// `SpeleothemBlock.MAX_HORIZONTAL_OFFSET` (`0.125f`), shared by `SulfurSpikeBlock` and
/// `PointedDripstoneBlock`.
pub const SPELEOTHEM_MAX_HORIZONTAL_OFFSET: f32 = 0.125;

/// `Mth.getSeed(int x, int y, int z)` (`@0-33`):
/// `x * 3129871` is a 32-bit `imul` widened via `i2l` AFTER multiplication, while `z` uses
/// 64-bit multiplication (`i2l; ldc 116129781L; lmul`).
#[inline]
pub fn get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut l = (x.wrapping_mul(3129871) as i64) ^ (z as i64).wrapping_mul(116129781) ^ y as i64;
    l = l
        .wrapping_mul(l)
        .wrapping_mul(42317861)
        .wrapping_add(l.wrapping_mul(11));
    l >> 16
}

/// `Mth.clamp(double v, double min, double max)`: `v < min ? min : Math.min(v, max)`.
#[inline]
fn clamp_axis(k: i64, max: f64) -> f64 {
    let v = (((k as f32) / 15.0f32) as f64 - 0.5) * 0.5;
    if v < -max {
        -max
    } else {
        v.min(max)
    }
}

/// `BlockBehaviour$Properties.lambda$offsetType$1` (`OffsetType.XZ`):
/// computes `BlockStateBase.getOffset(BlockPos)` for `bamboo` (`0.25f`), `sulfur_spike` (`0.125f`),
/// and `pointed_dripstone` (`0.125f`).
#[inline]
pub fn get_xz_offset(x: i32, z: i32, max_horizontal_offset: f32) -> Vec3 {
    let l = get_seed(x, 0, z);
    let max = max_horizontal_offset as f64;
    Vec3::new(
        clamp_axis(l & 15, max),
        0.0,
        clamp_axis((l >> 8) & 15, max),
    )
}

/// `BambooLeaves` property (`BambooStalkBlock.getShape`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BambooLeaves {
    None,
    Small,
    Large,
}

/// `SpeleothemThickness` property (`SpeleothemBlock.getShape` / `getCollisionShape`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpeleothemThickness {
    TipMerge,
    Tip,
    Frustum,
    Middle,
    Base,
}

/// `VERTICAL_DIRECTION` (`TIP_DIRECTION`) property for speleothems (`Up` or `Down`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TipDirection {
    Up,
    Down,
}

/// Data-driven offset-collision block descriptor (never keyed by block name).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffsetCollisionSpec {
    None,
    Bamboo,
    Speleothem {
        thickness: SpeleothemThickness,
        tip_direction: TipDirection,
    },
}

/// `Block.column(width, minY, maxY)` = `[8 - w/2, minY, 8 - w/2, 8 + w/2, maxY, 8 + w/2] / 16`.
#[inline]
const fn block_column(width: f64, min_y: f64, max_y: f64) -> Aabb {
    let half = width / 2.0;
    aabb(
        (8.0 - half) / 16.0,
        min_y / 16.0,
        (8.0 - half) / 16.0,
        (8.0 + half) / 16.0,
        max_y / 16.0,
        (8.0 + half) / 16.0,
    )
}

/// `BambooStalkBlock.SHAPE_COLLISION`: `Block.column(3.0, 0.0, 16.0)` shifted by XZ offset.
#[inline]
pub fn bamboo_collision_aabb(pos: BlockPos) -> Aabb {
    let base = block_column(3.0, 0.0, 16.0);
    base.moved(get_xz_offset(pos[0], pos[2], BAMBOO_MAX_HORIZONTAL_OFFSET))
}

/// `BambooStalkBlock.getShape`: `SHAPE_LARGE` (`width=10`) when `Large`, else `SHAPE_SMALL` (`width=6`).
#[inline]
pub fn bamboo_outline_aabb(pos: BlockPos, leaves: BambooLeaves) -> Aabb {
    let width = if leaves == BambooLeaves::Large {
        10.0
    } else {
        6.0
    };
    block_column(width, 0.0, 16.0).moved(get_xz_offset(
        pos[0],
        pos[2],
        BAMBOO_MAX_HORIZONTAL_OFFSET,
    ))
}

/// `SpeleothemBlock.getShape` / `getCollisionShape` local AABB before `getOffset(pos)` shift.
#[inline]
pub const fn speleothem_base_aabb(
    thickness: SpeleothemThickness,
    tip_direction: TipDirection,
) -> Aabb {
    match thickness {
        SpeleothemThickness::TipMerge => block_column(6.0, 0.0, 16.0),
        SpeleothemThickness::Tip => match tip_direction {
            TipDirection::Down => block_column(6.0, 5.0, 16.0),
            TipDirection::Up => block_column(6.0, 0.0, 11.0),
        },
        SpeleothemThickness::Frustum => block_column(8.0, 0.0, 16.0),
        SpeleothemThickness::Middle => block_column(10.0, 0.0, 16.0),
        SpeleothemThickness::Base => block_column(12.0, 0.0, 16.0),
    }
}

/// `SpeleothemBlock.getShape` / `getCollisionShape` shifted by `getOffset(pos)` (`0.125f`).
#[inline]
pub fn speleothem_aabb(
    pos: BlockPos,
    thickness: SpeleothemThickness,
    tip_direction: TipDirection,
) -> Aabb {
    speleothem_base_aabb(thickness, tip_direction).moved(get_xz_offset(
        pos[0],
        pos[2],
        SPELEOTHEM_MAX_HORIZONTAL_OFFSET,
    ))
}

/// Evaluates the local shifted `VoxelShape` for an `OffsetCollisionSpec` at `pos`.
pub fn offset_collision_shape(spec: OffsetCollisionSpec, pos: BlockPos) -> Option<VoxelShape> {
    match spec {
        OffsetCollisionSpec::None => None,
        OffsetCollisionSpec::Bamboo => Some(VoxelShape::boxes(&[bamboo_collision_aabb(pos)])),
        OffsetCollisionSpec::Speleothem {
            thickness,
            tip_direction,
        } => Some(VoxelShape::boxes(&[speleothem_aabb(
            pos,
            thickness,
            tip_direction,
        )])),
    }
}

// ============================================================================
// 2. ShulkerBox shape, animation, open obstruction, placement, and break
// ============================================================================

pub const SHULKER_ANIMATION_STEP: f32 = 0.1;
pub const SHULKER_CAN_OPEN_DEFLATE: f64 = 1.0e-6;
pub const SHULKER_ENTITY_PUSH_PADDING: f64 = 0.01;
const FULL_CUBE: Aabb = aabb(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);

/// `ShulkerBoxBlockEntity$AnimationStatus` ordinals (`0..=3`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShulkerAnimationStatus {
    #[default]
    Closed = 0,
    Opening = 1,
    Opened = 2,
    Closing = 3,
}

/// `ShulkerBoxBlockEntity` animation state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShulkerBoxEntityState {
    pub status: ShulkerAnimationStatus,
    pub progress: f32,
    pub progress_old: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShulkerAnimationTickResult {
    pub do_neighbor_updates: bool,
    /// Non-`None` during `Opening`: swept `getProgressDeltaAabb(1.0, facing, progress_old, progress, pivot)`.
    pub move_collided_sweep: Option<Aabb>,
}

/// `AABB.contract(double, double, double)` in 26.2 (`AABB.txt:496-576`):
/// per-side shrink (`d < 0 -> min -= d`, `d > 0 -> max -= d`), NOT symmetric.
pub fn aabb_contract(box_: Aabb, x: f64, y: f64, z: f64) -> Aabb {
    let mut min_x = box_.min.x;
    let mut min_y = box_.min.y;
    let mut min_z = box_.min.z;
    let mut max_x = box_.max.x;
    let mut max_y = box_.max.y;
    let mut max_z = box_.max.z;

    if x < 0.0 {
        min_x -= x;
    } else if x > 0.0 {
        max_x -= x;
    }
    if y < 0.0 {
        min_y -= y;
    } else if y > 0.0 {
        max_y -= y;
    }
    if z < 0.0 {
        min_z -= z;
    } else if z > 0.0 {
        max_z -= z;
    }
    Aabb::new(min_x, min_y, min_z, max_x, max_y, max_z)
}

/// `Shulker.getProgressDeltaAabb(float width, Direction dir, float from, float to, Vec3 pivot)`
/// (`Shulker.txt:551-657`).
pub fn shulker_progress_delta_aabb(
    width: f32,
    facing: Direction,
    from: f32,
    to: f32,
    pivot: Vec3,
) -> Aabb {
    let wf = width as f64;
    let half = (-width) as f64 * 0.5;
    let base = Aabb::new(half, 0.0, half, wf * 0.5, wf, wf * 0.5);
    let max_progress = from.max(to) as f64;
    let min_progress = from.min(to) as f64;
    let [sx, sy, sz] = facing.step();
    let expanded = base.expand(Vec3::new(
        sx as f64 * max_progress * wf,
        sy as f64 * max_progress * wf,
        sz as f64 * max_progress * wf,
    ));
    let contracted = aabb_contract(
        expanded,
        (-sx) as f64 * (1.0 + min_progress) * wf,
        (-sy) as f64 * (1.0 + min_progress) * wf,
        (-sz) as f64 * (1.0 + min_progress) * wf,
    );
    contracted.moved(pivot)
}

/// `Shulker.getProgressAabb(float width, Direction dir, float progress, Vec3 pivot)`
/// (`Shulker.txt:541-549`).
#[inline]
pub fn shulker_progress_aabb(width: f32, facing: Direction, progress: f32, pivot: Vec3) -> Aabb {
    shulker_progress_delta_aabb(width, facing, -1.0, progress, pivot)
}

impl ShulkerBoxEntityState {
    /// `ShulkerBoxBlockEntity.getProgress(float partialTick)`: `Mth.lerp(partialTick, progressOld, progress)`.
    #[inline]
    pub fn get_progress(&self, partial_tick: f32) -> f32 {
        self.progress_old + partial_tick * (self.progress - self.progress_old)
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.status == ShulkerAnimationStatus::Closed
    }

    /// `ShulkerBoxBlockEntity.getBoundingBox(BlockState)` (`@192-213`):
    /// `Shulker.getProgressAabb(1.0f, facing, 0.5f * getProgress(1.0f), Vec3(0.5, 0, 0.5))`.
    pub fn local_bounding_box(&self, facing: Direction) -> Aabb {
        shulker_progress_aabb(
            1.0,
            facing,
            0.5f32 * self.get_progress(1.0),
            Vec3::new(0.5, 0.0, 0.5),
        )
    }

    /// `ShulkerBoxBlockEntity.updateAnimation` (`@94-184`): exact f32 step, neighbor-update
    /// points, and `moveCollidedEntities` sweep box.
    pub fn tick_animation(
        &mut self,
        pos: BlockPos,
        facing: Direction,
    ) -> ShulkerAnimationTickResult {
        self.progress_old = self.progress;
        let mut do_neighbor_updates = false;
        let mut move_collided_sweep = None;

        match self.status {
            ShulkerAnimationStatus::Closed => {
                self.progress = 0.0;
            }
            ShulkerAnimationStatus::Opening => {
                self.progress += SHULKER_ANIMATION_STEP;
                if self.progress_old == 0.0 {
                    do_neighbor_updates = true;
                }
                if self.progress >= 1.0 {
                    self.status = ShulkerAnimationStatus::Opened;
                    self.progress = 1.0;
                    do_neighbor_updates = true;
                }
                let pivot = Vec3::new(pos[0] as f64 + 0.5, pos[1] as f64, pos[2] as f64 + 0.5);
                move_collided_sweep = Some(shulker_progress_delta_aabb(
                    1.0,
                    facing,
                    self.progress_old,
                    self.progress,
                    pivot,
                ));
            }
            ShulkerAnimationStatus::Opened => {
                self.progress = 1.0;
            }
            ShulkerAnimationStatus::Closing => {
                self.progress -= SHULKER_ANIMATION_STEP;
                if self.progress_old == 1.0 {
                    do_neighbor_updates = true;
                }
                if self.progress <= 0.0 {
                    self.status = ShulkerAnimationStatus::Closed;
                    self.progress = 0.0;
                    do_neighbor_updates = true;
                }
            }
        }

        ShulkerAnimationTickResult {
            do_neighbor_updates,
            move_collided_sweep,
        }
    }
}

/// `ShulkerBoxBlock.getShape` / `getCollisionShape`:
/// full unit cube when the block entity is absent or `Closed`, else `be.local_bounding_box(facing)`.
#[inline]
pub fn shulker_collision_aabb(facing: Direction, be: Option<ShulkerBoxEntityState>) -> Aabb {
    match be {
        Some(entity) => entity.local_bounding_box(facing),
        None => FULL_CUBE,
    }
}

/// `ShulkerBoxBlock.getBlockSupportShape`: `SHAPES_OPEN_SUPPORT[facing.opposite()]`
/// (1/16 plate on the closed back face) when `!closed`, else the full unit cube.
#[inline]
pub const fn shulker_open_support_aabb(facing: Direction, closed: bool) -> Aabb {
    if closed {
        return FULL_CUBE;
    }
    match facing.opposite() {
        Direction::Down => aabb(0.0, 0.0, 0.0, 1.0, 0.0625, 1.0),
        Direction::Up => aabb(0.0, 0.9375, 0.0, 1.0, 1.0, 1.0),
        Direction::North => aabb(0.0, 0.0, 0.0, 1.0, 1.0, 0.0625),
        Direction::South => aabb(0.0, 0.0, 0.9375, 1.0, 1.0, 1.0),
        Direction::West => aabb(0.0, 0.0, 0.0, 0.0625, 1.0, 1.0),
        Direction::East => aabb(0.9375, 0.0, 0.0, 1.0, 1.0, 1.0),
    }
}

/// `ShulkerBoxBlock.canOpen` (`@12-39`): `getProgressDeltaAabb(1.0, facing, 0.0, 0.5, bottomCenter).deflate(1.0e-6)`.
#[inline]
pub fn shulker_can_open_box(pos: BlockPos, facing: Direction) -> Aabb {
    let pivot = Vec3::new(pos[0] as f64 + 0.5, pos[1] as f64, pos[2] as f64 + 0.5);
    shulker_progress_delta_aabb(1.0, facing, 0.0, 0.5, pivot).deflate(SHULKER_CAN_OPEN_DEFLATE)
}

/// `ShulkerBoxBlock.canOpen`: returns `true` unconditionally if `status != Closed`, otherwise
/// requires `world.no_collision(box, empty)` AND no intersecting entity `AABB` (note: border ignored
/// since `Level.noCollision(AABB)` passes `entity = null`).
pub fn shulker_can_open(
    world: &(impl World + ?Sized),
    pos: BlockPos,
    facing: Direction,
    status: ShulkerAnimationStatus,
    entity_aabbs: &[Aabb],
) -> bool {
    if status != ShulkerAnimationStatus::Closed {
        return true;
    }
    let probe = shulker_can_open_box(pos, facing);
    let ctx = CollisionContext {
        feet_y: f64::NEG_INFINITY,
        descending: false,
        fall_distance: 0.0,
        leather_boots: false,
    };
    if !world.no_collision(probe, ctx) {
        return false;
    }
    !entity_aabbs.iter().any(|e| e.intersects(probe))
}

/// `ShulkerBoxBlockEntity.moveCollidedEntities` (`@263-290`): push vector applied with
/// `MoverType.SHULKER_BOX` for entities in `sweep` whose `PushReaction != IGNORE`.
#[inline]
pub fn shulker_entity_push_vector(facing: Direction, sweep: Aabb) -> Vec3 {
    let [sx, sy, sz] = facing.step();
    let x_size = sweep.max.x - sweep.min.x;
    let y_size = sweep.max.y - sweep.min.y;
    let z_size = sweep.max.z - sweep.min.z;
    Vec3::new(
        (x_size + SHULKER_ENTITY_PUSH_PADDING) * sx as f64,
        (y_size + SHULKER_ENTITY_PUSH_PADDING) * sy as f64,
        (z_size + SHULKER_ENTITY_PUSH_PADDING) * sz as f64,
    )
}

/// `ShulkerBoxBlock.getStateForPlacement` (`@121-130`): sets `FACING = clicked_face` (default `Up`).
#[inline]
pub const fn shulker_placement_facing(clicked_face: Direction) -> Direction {
    clicked_face
}

/// Outcome of `ShulkerBoxBlock.playerWillDestroy` (`@145-213`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShulkerBreakOutcome {
    /// `!client && preventsBlockDrops && !isEmpty`: spawns `ItemEntity` at `pos + 0.5` carrying
    /// container components even in Creative mode.
    CreativeDropWithComponents,
    /// `!client && preventsBlockDrops && isEmpty`: drops nothing.
    CreativeEmptyNoDrop,
    /// Survival or client break: unpacks loot table and delegates to loot drops (`copy_components`).
    NormalDrop,
}

/// `ShulkerBoxBlock.playerWillDestroy` (`@22-129`).
#[inline]
pub const fn shulker_break_outcome(
    is_client_side: bool,
    prevents_block_drops: bool,
    container_is_empty: bool,
) -> ShulkerBreakOutcome {
    if !is_client_side && prevents_block_drops {
        if !container_is_empty {
            ShulkerBreakOutcome::CreativeDropWithComponents
        } else {
            ShulkerBreakOutcome::CreativeEmptyNoDrop
        }
    } else {
        ShulkerBreakOutcome::NormalDrop
    }
}

// ============================================================================
// 3. Non-player and world block hooks (13 refused census classes -> none / world)
// ============================================================================

/// How a block hook affects a walking/falling `Player` in 26.2 (`ModelTable.HOOKS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerHookClass {
    /// Never mutates any player state (`CropBlock`, `PitcherCropBlock`, `EyeblossomBlock`,
    /// `LilyPadBlock`, `FrogspawnBlock`, `HopperBlock`).
    None,
    /// Mutates only world / block-entity / non-player state (`DetectorRailBlock`,
    /// `SculkSensorBlock`, `CalibratedSculkSensorBlock`, `SculkShriekerBlock`,
    /// `RedStoneOreBlock`, `TurtleEggBlock`), plus standard block-agnostic `Block.fallOn`.
    WorldOnly,
}

/// Entity types discriminated by the 12 hook classes (players never trigger any non-world hook).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HookEntityKind {
    Player,
    Ravager,
    Bee { has_poison: bool },
    Boat,
    FallingBlock,
    Item { non_empty: bool },
    Minecart,
    Warden,
    Zombie,
    Turtle,
    Bat,
    OtherLiving,
    OtherNonLiving,
}

/// Data-driven block hook family (never keyed by block-name string; keyed by `BlockId` + state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonPlayerHookBlock {
    Crop {
        block_id: BlockId,
    },
    PitcherCrop {
        block_id: BlockId,
    },
    Eyeblossom {
        block_id: BlockId,
        attracts_bees: bool,
    },
    LilyPad {
        block_id: BlockId,
    },
    Frogspawn {
        block_id: BlockId,
    },
    Hopper {
        block_id: BlockId,
        has_block_entity: bool,
    },
    DetectorRail {
        block_id: BlockId,
        powered: bool,
        can_survive: bool,
    },
    SculkSensor {
        block_id: BlockId,
        phase_inactive: bool,
        has_block_entity: bool,
        can_receive_step: bool,
    },
    SculkShrieker {
        block_id: BlockId,
        shrieking: bool,
        can_respond: bool,
        has_block_entity: bool,
    },
    RedStoneOre {
        block_id: BlockId,
        lit: bool,
    },
    TurtleEgg {
        block_id: BlockId,
        eggs: u8,
    },
}

impl NonPlayerHookBlock {
    #[inline]
    pub const fn block_id(self) -> BlockId {
        match self {
            Self::Crop { block_id }
            | Self::PitcherCrop { block_id }
            | Self::Eyeblossom { block_id, .. }
            | Self::LilyPad { block_id }
            | Self::Frogspawn { block_id }
            | Self::Hopper { block_id, .. }
            | Self::DetectorRail { block_id, .. }
            | Self::SculkSensor { block_id, .. }
            | Self::SculkShrieker { block_id, .. }
            | Self::RedStoneOre { block_id, .. }
            | Self::TurtleEgg { block_id, .. } => block_id,
        }
    }

    #[inline]
    pub const fn entity_inside_player_class(self) -> PlayerHookClass {
        match self {
            Self::DetectorRail { .. } => PlayerHookClass::WorldOnly,
            _ => PlayerHookClass::None,
        }
    }

    #[inline]
    pub const fn step_on_player_class(self) -> PlayerHookClass {
        match self {
            Self::SculkSensor { .. }
            | Self::SculkShrieker { .. }
            | Self::RedStoneOre { .. }
            | Self::TurtleEgg { .. } => PlayerHookClass::WorldOnly,
            _ => PlayerHookClass::None,
        }
    }

    #[inline]
    pub const fn fall_on_player_class(self) -> PlayerHookClass {
        match self {
            Self::TurtleEgg { .. } => PlayerHookClass::WorldOnly,
            _ => PlayerHookClass::None,
        }
    }
}

/// `DetectorRailBlock.PRESSED_CHECK_PERIOD` (`20` ticks).
pub const DETECTOR_RAIL_CHECK_PERIOD: u32 = 20;
/// `SculkShriekerBlockEntity.shriek` tick delay (`90` ticks).
pub const SCULK_SHRIEKER_DELAY: u32 = 90;
/// `TurtleEggBlock.stepOn` trample denominator (`nextInt(100) == 0`).
pub const TURTLE_EGG_STEP_CHANCE: u32 = 100;
/// `TurtleEggBlock.fallOn` trample denominator (`nextInt(3) == 0`).
pub const TURTLE_EGG_FALL_CHANCE: u32 = 3;

/// `Hopper.SUCK_AABB` (`Block.column(16, 11, 32)`) moved to block `pos`:
/// `[x, y + 0.6875, z, x + 1.0, y + 2.0, z + 1.0]`.
#[inline]
pub fn hopper_suck_aabb(pos: BlockPos) -> Aabb {
    aabb(0.0, 11.0 / 16.0, 0.0, 1.0, 2.0, 1.0).moved(Vec3::new(
        pos[0] as f64,
        pos[1] as f64,
        pos[2] as f64,
    ))
}

/// `DetectorRailBlock.getSearchBB(BlockPos)` (`@356-398`):
/// `[x + 0.2, y, z + 0.2, x + 0.8, y + 0.8, z + 0.8]`.
#[inline]
pub fn detector_rail_search_bb(pos: BlockPos) -> Aabb {
    const D: f64 = 0.2;
    aabb(
        pos[0] as f64 + D,
        pos[1] as f64,
        pos[2] as f64 + D,
        (pos[0] + 1) as f64 - D,
        (pos[1] + 1) as f64 - D,
        (pos[2] + 1) as f64 - D,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DetectorRailResult {
    pub new_powered: bool,
    pub set_block_flags: Option<i32>,
    pub update_power_to_connected: bool,
    pub update_neighbors_here_and_below: bool,
    pub schedule_tick_delay: Option<u32>,
    pub update_comparator_output: bool,
}

/// `DetectorRailBlock.checkPressed(Level, BlockPos, BlockState)` (`@116-231`):
/// ignores passing entity, checks `canSurvive`, polls `AbstractMinecart` in `getSearchBB`,
/// updates `POWERED` (`flags = 3`), neighbors (`pos` and `pos.below()`), schedules tick `20`,
/// and always calls `updateNeighbourForOutputSignal`.
pub fn detector_rail_check_pressed(
    powered: bool,
    can_survive: bool,
    has_minecart: bool,
) -> DetectorRailResult {
    if !can_survive {
        return DetectorRailResult {
            new_powered: powered,
            set_block_flags: None,
            update_power_to_connected: false,
            update_neighbors_here_and_below: false,
            schedule_tick_delay: None,
            update_comparator_output: false,
        };
    }
    let changed = has_minecart != powered;
    DetectorRailResult {
        new_powered: has_minecart,
        set_block_flags: changed.then_some(update_flags::ALL),
        update_power_to_connected: changed,
        update_neighbors_here_and_below: changed,
        schedule_tick_delay: has_minecart.then_some(DETECTOR_RAIL_CHECK_PERIOD),
        update_comparator_output: true,
    }
}

/// `DetectorRailBlock.entityInside` (`@0-29`): returns `None` immediately when already `powered`;
/// otherwise delegates to `checkPressed` (never inspects the passing entity).
#[inline]
pub fn detector_rail_entity_inside(
    powered: bool,
    can_survive: bool,
    has_minecart: bool,
) -> Option<DetectorRailResult> {
    if powered {
        None
    } else {
        Some(detector_rail_check_pressed(false, can_survive, has_minecart))
    }
}

/// `TurtleEggBlock.canDestroyEgg` (`@0-56`):
/// `Turtle | Bat -> false`; `Player -> true`; other `LivingEntity -> mob_griefing`; non-living -> `false`.
#[inline]
pub const fn turtle_egg_can_destroy(entity: HookEntityKind, mob_griefing: bool) -> bool {
    match entity {
        HookEntityKind::Turtle | HookEntityKind::Bat => false,
        HookEntityKind::Player => true,
        HookEntityKind::Ravager
        | HookEntityKind::Bee { .. }
        | HookEntityKind::Zombie
        | HookEntityKind::OtherLiving => mob_griefing,
        HookEntityKind::Boat
        | HookEntityKind::FallingBlock
        | HookEntityKind::Item { .. }
        | HookEntityKind::Minecart
        | HookEntityKind::Warden
        | HookEntityKind::OtherNonLiving => false,
    }
}

/// World-side effect produced when evaluating one of the 12 non-player / world hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldHookAction {
    NoOp,
    DestroyBlock { drop: bool },
    PoisonBee,
    HopperTryMoveItems,
    DetectorRailCheck(DetectorRailResult),
    SculkScheduleVibration,
    SculkShriek { schedule_delay: u32, try_warn: bool },
    SetRedstoneOreLit { flags: i32 },
    TurtleEggDecrease { remaining_eggs: u8, flags: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookEvaluation {
    pub player_mutated: bool,
    pub action: WorldHookAction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HookContext {
    pub is_client_side: bool,
    pub is_server_level: bool,
    pub peaceful_difficulty: bool,
    pub mob_griefing: bool,
    pub stepping_carefully: bool,
    pub entity_intersects_suck_aabb: bool,
    pub detector_has_minecart: bool,
    pub shrieker_has_player_source: bool,
    pub random_hit_chance: bool,
}

impl Default for HookContext {
    fn default() -> Self {
        Self {
            is_client_side: false,
            is_server_level: true,
            peaceful_difficulty: false,
            mob_griefing: true,
            stepping_carefully: false,
            entity_intersects_suck_aabb: false,
            detector_has_minecart: false,
            shrieker_has_player_source: false,
            random_hit_chance: false,
        }
    }
}

/// Evaluates `entityInside` for the 7 `entityInside` classes (`CropBlock`, `PitcherCropBlock`,
/// `EyeblossomBlock`, `LilyPadBlock`, `FrogspawnBlock`, `HopperBlock`, `DetectorRailBlock`).
/// `player_mutated` is unconditionally `false`.
pub fn evaluate_entity_inside(
    block: NonPlayerHookBlock,
    entity: HookEntityKind,
    ctx: HookContext,
) -> HookEvaluation {
    let action = if ctx.is_client_side {
        WorldHookAction::NoOp
    } else {
        match block {
            NonPlayerHookBlock::Crop { .. } | NonPlayerHookBlock::PitcherCrop { .. } => {
                if ctx.is_server_level && entity == HookEntityKind::Ravager && ctx.mob_griefing {
                    WorldHookAction::DestroyBlock { drop: true }
                } else {
                    WorldHookAction::NoOp
                }
            }
            NonPlayerHookBlock::Eyeblossom { attracts_bees, .. } => {
                if !ctx.peaceful_difficulty
                    && attracts_bees
                    && matches!(entity, HookEntityKind::Bee { has_poison: false })
                {
                    WorldHookAction::PoisonBee
                } else {
                    WorldHookAction::NoOp
                }
            }
            NonPlayerHookBlock::LilyPad { .. } => {
                if ctx.is_server_level && entity == HookEntityKind::Boat {
                    WorldHookAction::DestroyBlock { drop: true }
                } else {
                    WorldHookAction::NoOp
                }
            }
            NonPlayerHookBlock::Frogspawn { .. } => {
                if entity == HookEntityKind::FallingBlock {
                    WorldHookAction::DestroyBlock { drop: false }
                } else {
                    WorldHookAction::NoOp
                }
            }
            NonPlayerHookBlock::Hopper {
                has_block_entity, ..
            } => {
                if has_block_entity
                    && matches!(entity, HookEntityKind::Item { non_empty: true })
                    && ctx.entity_intersects_suck_aabb
                {
                    WorldHookAction::HopperTryMoveItems
                } else {
                    WorldHookAction::NoOp
                }
            }
            NonPlayerHookBlock::DetectorRail {
                powered,
                can_survive,
                ..
            } => match detector_rail_entity_inside(powered, can_survive, ctx.detector_has_minecart)
            {
                Some(res) => WorldHookAction::DetectorRailCheck(res),
                None => WorldHookAction::NoOp,
            },
            _ => WorldHookAction::NoOp,
        }
    };
    HookEvaluation {
        player_mutated: false,
        action,
    }
}

/// Evaluates `stepOn` for `SculkSensorBlock` / `CalibratedSculkSensorBlock`, `SculkShriekerBlock`,
/// `RedStoneOreBlock`, and `TurtleEggBlock`. `player_mutated` is unconditionally `false`.
pub fn evaluate_step_on(
    block: NonPlayerHookBlock,
    entity: HookEntityKind,
    ctx: HookContext,
) -> HookEvaluation {
    let action = match block {
        NonPlayerHookBlock::SculkSensor {
            phase_inactive,
            has_block_entity,
            can_receive_step,
            ..
        } => {
            if !ctx.is_client_side
                && ctx.is_server_level
                && phase_inactive
                && entity != HookEntityKind::Warden
                && has_block_entity
                && can_receive_step
            {
                WorldHookAction::SculkScheduleVibration
            } else {
                WorldHookAction::NoOp
            }
        }
        NonPlayerHookBlock::SculkShrieker {
            shrieking,
            can_respond,
            has_block_entity,
            ..
        } => {
            if ctx.is_server_level
                && !ctx.is_client_side
                && has_block_entity
                && !shrieking
                && (entity == HookEntityKind::Player || ctx.shrieker_has_player_source)
            {
                WorldHookAction::SculkShriek {
                    schedule_delay: SCULK_SHRIEKER_DELAY,
                    try_warn: can_respond,
                }
            } else {
                WorldHookAction::NoOp
            }
        }
        NonPlayerHookBlock::RedStoneOre { lit, .. } => {
            if !ctx.stepping_carefully && !lit {
                WorldHookAction::SetRedstoneOreLit {
                    flags: update_flags::ALL,
                }
            } else {
                WorldHookAction::NoOp
            }
        }
        NonPlayerHookBlock::TurtleEgg { eggs, .. } => {
            if !ctx.stepping_carefully
                && ctx.is_server_level
                && turtle_egg_can_destroy(entity, ctx.mob_griefing)
                && ctx.random_hit_chance
            {
                if eggs <= 1 {
                    WorldHookAction::DestroyBlock { drop: false }
                } else {
                    WorldHookAction::TurtleEggDecrease {
                        remaining_eggs: eggs - 1,
                        flags: update_flags::CLIENTS,
                    }
                }
            } else {
                WorldHookAction::NoOp
            }
        }
        _ => WorldHookAction::NoOp,
    };
    HookEvaluation {
        player_mutated: false,
        action,
    }
}

/// Evaluates `TurtleEggBlock.fallOn`: ignores `stepping_carefully`, guards against `Zombie`,
/// tramples with `1/3` chance, and leaves fall damage to standard `Block.fallOn`.
pub fn evaluate_fall_on(
    block: NonPlayerHookBlock,
    entity: HookEntityKind,
    ctx: HookContext,
) -> HookEvaluation {
    let action = match block {
        NonPlayerHookBlock::TurtleEgg { eggs, .. } => {
            if entity != HookEntityKind::Zombie
                && ctx.is_server_level
                && turtle_egg_can_destroy(entity, ctx.mob_griefing)
                && ctx.random_hit_chance
            {
                if eggs <= 1 {
                    WorldHookAction::DestroyBlock { drop: false }
                } else {
                    WorldHookAction::TurtleEggDecrease {
                        remaining_eggs: eggs - 1,
                        flags: update_flags::CLIENTS,
                    }
                }
            } else {
                WorldHookAction::NoOp
            }
        }
        _ => WorldHookAction::NoOp,
    };
    HookEvaluation {
        player_mutated: false,
        action,
    }
}
