//! Mojang 26.2 world-hook and world-edit laws:
//! 1. Water/lava bucket place, waterlog, sneak redirect, evaporation, and pickup
//!    (`BucketItem`, `LiquidBlock`, `SimpleWaterloggedBlock`, `ItemUtils.createFilledResult`).
//! 2. Pressure plate press, entity/mob filtering, weighted ceiling, and scheduled release
//!    (`BasePressurePlateBlock`, `PressurePlateBlock`, `WeightedPressurePlateBlock`).
//! 3. Button press, support-cell neighbor updates, arrow hold re-arm, and exact shapes
//!    (`ButtonBlock`, `FaceAttachedHorizontalDirectionalBlock`).
//! 4. Tripwire hook & string line scan (`SOUTH`/`WEST`, `i < 42`), `farDist > 1`, gap `astate[i] = null`,
//!    shears disarm (`flags = 260`), delay-0 echo, and removal phantom-powered trigger
//!    (`TripWireBlock`, `TripWireHookBlock`).
//! 5. Bubble column `entityInside` law (`BubbleColumnBlock.entityInside`,
//!    `Entity.onAboveBubbleColumn`, `Entity.onInsideBubbleColumn`).
//!
//! Block identity is strictly data-driven (`BlockId` and properties, never block-name strings).

use crate::interact::{AttachFace, BlockHitResult, BlockId, ClipFluidMode, Direction};
use crate::basis::cell::{CellState, StateId, WaterContainer, WorldMut, DIRECTION};
use crate::neighbor::update_flags;
use crate::{Aabb, BlockPos, FluidKind, Vec3, World};

/// `pos.relative(dir, n)`.
#[inline]
pub const fn step_n(dir: Direction, [x, y, z]: BlockPos, n: i32) -> BlockPos {
    let [dx, dy, dz] = dir.step();
    [x + dx * n, y + dy * n, z + dz * n]
}

#[inline]
const fn aabb(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Aabb {
    Aabb {
        min: Vec3 { x: x0, y: y0, z: z0 },
        max: Vec3 { x: x1, y: y1, z: z1 },
    }
}

// ============================================================================
// 1. Bucket place / waterlog / evaporation / pickup
// ============================================================================

/// `Block.UPDATE_ALL` (`3`): used by `SimpleWaterloggedBlock.{placeLiquid,pickupBlock}`,
/// `ButtonBlock`, `TripWireBlock` (normal writes), and `TripWireHookBlock`.
pub const UPDATE_ALL: i32 = update_flags::ALL;
/// `Block.UPDATE_CLIENTS` (`2`): used by `BasePressurePlateBlock.checkPressed`.
pub const UPDATE_CLIENTS: i32 = update_flags::CLIENTS;
/// `Block.UPDATE_ALL_IMMEDIATE` (`11`): used by `BucketItem.emptyContents` and `LiquidBlock.pickupBlock`.
pub const UPDATE_ALL_IMMEDIATE: i32 = update_flags::ALL_IMMEDIATE;
/// `Block.UPDATE_NONE` (`260 = 4 | 256`): used by `TripWireBlock.playerWillDestroy` (shears disarm).
pub const UPDATE_NONE: i32 = update_flags::INVISIBLE | update_flags::SKIP_BLOCK_ENTITY_SIDEEFFECTS;

/// `Fluids.WATER.getTickDelay(level)`.
pub const WATER_TICK_DELAY: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BucketContent {
    Empty,
    Water,
    Lava,
}

impl BucketContent {
    #[inline]
    pub const fn is_flowing_fluid(self) -> bool {
        !matches!(self, Self::Empty)
    }

    /// `BucketItem.getFluidContext` (`@0-19`).
    #[inline]
    pub const fn fluid_context(self) -> ClipFluidMode {
        match self {
            Self::Empty => ClipFluidMode::SourceOnly,
            Self::Water | Self::Lava => ClipFluidMode::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BucketStack {
    pub content: BucketContent,
    pub count: u32,
}

impl BucketStack {

    #[inline]
    pub const fn new(content: BucketContent, count: u32) -> Self {
        Self { content, count }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.count == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilledResult {
    pub hand_stack: BucketStack,
    pub added_to_inventory: Option<BucketStack>,
    pub dropped: Option<BucketStack>,
}

/// `BucketItem.getEmptySuccessItem(ItemStack, Player)` (`@0-19`).
#[inline]
pub const fn bucket_empty_success_item(held: BucketStack, has_infinite_materials: bool) -> BucketStack {
    if !has_infinite_materials {
        BucketStack::new(BucketContent::Empty, 1)
    } else {
        held
    }
}

/// `ItemUtils.createFilledResult(held, player, filled, limitCreative=true)` (`@0-71`).
pub const fn create_filled_result(
    mut held: BucketStack,
    filled: BucketStack,
    has_infinite_materials: bool,
    inventory_contains_filled: bool,
    inventory_can_add_filled: bool,
) -> FilledResult {
    if has_infinite_materials {
        let added = if !inventory_contains_filled {
            Some(filled)
        } else {
            None
        };
        return FilledResult {
            hand_stack: held,
            added_to_inventory: added,
            dropped: None,
        };
    }
    held.count = held.count.saturating_sub(1);
    if held.is_empty() {
        return FilledResult {
            hand_stack: filled,
            added_to_inventory: None,
            dropped: None,
        };
    }
    if inventory_can_add_filled {
        FilledResult {
            hand_stack: held,
            added_to_inventory: Some(filled),
            dropped: None,
        }
    } else {
        FilledResult {
            hand_stack: held,
            added_to_inventory: None,
            dropped: Some(filled),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BucketSound {
    EmptyWater,
    EmptyLava,
    FireExtinguish,
    PickupWater,
    PickupLava,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BucketGameEvent {
    FluidPlace,
    FluidPickup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BucketEvent {
    DestroyBlock { pos: BlockPos, drop: bool },
    SetBlock { pos: BlockPos, cell: StateId, flags: i32 },
    ScheduleFluidTick { pos: BlockPos, fluid: BucketContent, delay: u32 },
    PlaySound { pos: BlockPos, sound: BucketSound },
    GameEvent { pos: BlockPos, event: BucketGameEvent },
    EvaporateSmokeParticles { pos: BlockPos, count: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BucketUseResult {
    Pass,
    Fail,
    Success(FilledResult),
}

fn bucket_source(content: BucketContent) -> StateId {
    match content {
        BucketContent::Water => BlockId::of("water").default_state(),
        BucketContent::Lava => BlockId::of("lava").default_state(),
        BucketContent::Empty => StateId::AIR,
    }
}

/// `BucketPickup.pickupBlock` for liquid blocks and waterlogged blocks.
pub(crate) fn bucket_pickup_block(
    world: &impl WorldMut,
    pos: BlockPos,
    events: &mut Vec<BucketEvent>,
) -> Option<(BucketStack, BucketSound)> {
    let cell = world.cell(pos);
    let (content, sound) = match cell.block().name() {
        "water" if cell.get("level") == Some("0") => (BucketContent::Water, BucketSound::PickupWater),
        "lava" if cell.get("level") == Some("0") => (BucketContent::Lava, BucketSound::PickupLava),
        _ if cell.is_waterlogged_container() && cell.get("waterlogged") == Some("true") => {
            let dry = cell.with_bool("waterlogged", false);
            events.push(BucketEvent::SetBlock { pos, cell: dry, flags: UPDATE_ALL });
            if !world.can_survive(pos, dry) {
                events.push(BucketEvent::DestroyBlock { pos, drop: true });
            }
            return Some((BucketStack::new(BucketContent::Water, 1), BucketSound::PickupWater));
        }
        _ => return None,
    };
    events.push(BucketEvent::SetBlock { pos, cell: StateId::AIR, flags: UPDATE_ALL_IMMEDIATE });
    Some((BucketStack::new(content, 1), sound))
}

fn emit_empty_sound_and_event(pos: BlockPos, content: BucketContent, events: &mut Vec<BucketEvent>) {
    let sound = match content {
        BucketContent::Lava => BucketSound::EmptyLava,
        BucketContent::Water | BucketContent::Empty => BucketSound::EmptyWater,
    };
    events.push(BucketEvent::PlaySound { pos, sound });
    events.push(BucketEvent::GameEvent {
        pos,
        event: BucketGameEvent::FluidPlace,
    });
}

/// `BucketItem.emptyContents(LivingEntity, Level, BlockPos, BlockHitResult)` (`@0-463`).
pub(crate) fn bucket_empty_contents(
    world: &impl WorldMut,
    water_evaporates: bool,
    content: BucketContent,
    pos: BlockPos,
    hit: Option<(BlockPos, Direction)>,
    is_sneaking: bool,
    is_client_side: bool,
    events: &mut Vec<BucketEvent>,
) -> bool {
    if !content.is_flowing_fluid() {
        return false;
    }
    let state = world.cell(pos);
    let rules = state.rules();
    let water = content == BucketContent::Water;
    let can_replace = if water { rules.replaceable_by_water } else { rules.replaceable_by_lava };
    let accepts_water = water && rules.water_container == WaterContainer::Accepts;
    let proceed = rules.is_air || ((can_replace || accepts_water) && (!is_sneaking || hit.is_none()));
    if !proceed {
        return hit.is_some_and(|(hit_pos, face)| bucket_empty_contents(
            world, water_evaporates, content, face.relative(hit_pos), None, is_sneaking, is_client_side, events,
        ));
    }

    if water && water_evaporates {
        events.push(BucketEvent::PlaySound { pos, sound: BucketSound::FireExtinguish });
        events.push(BucketEvent::EvaporateSmokeParticles { pos, count: 8 });
        return true;
    }

    if water && rules.water_container != WaterContainer::None {
        // BucketItem ignores placeLiquid's return value, including for an already wet block.
        if accepts_water && state.get("waterlogged") == Some("false") && !is_client_side {
            events.push(BucketEvent::SetBlock { pos, cell: state.with_bool("waterlogged", true), flags: UPDATE_ALL });
            events.push(BucketEvent::ScheduleFluidTick { pos, fluid: content, delay: WATER_TICK_DELAY });
        }
    } else {
        if !is_client_side && can_replace && !rules.is_air && !matches!(state.block().name(), "water" | "lava") {
            events.push(BucketEvent::DestroyBlock { pos, drop: true });
        }
        let source = bucket_source(content);
        if state != source {
            events.push(BucketEvent::SetBlock { pos, cell: source, flags: UPDATE_ALL_IMMEDIATE });
        }
    }
    emit_empty_sound_and_event(pos, content, events);
    true
}

/// `BucketItem.use(Level, Player, InteractionHand)` (`@0-369`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn bucket_use(
    world: &impl WorldMut,
    water_evaporates: bool,
    held: BucketStack,
    hit: BlockHitResult,
    may_interact: bool,
    may_use_item_at_offset: bool,
    is_sneaking: bool,
    has_infinite_materials: bool,
    inventory_contains_result: bool,
    inventory_can_add_result: bool,
    is_client_side: bool,
    events: &mut Vec<BucketEvent>,
) -> BucketUseResult {
    let BlockHitResult::Hit { pos, face, .. } = hit else {
        return BucketUseResult::Pass;
    };
    let offset_pos = face.relative(pos);
    if !may_interact || !may_use_item_at_offset {
        return BucketUseResult::Fail;
    }

    let hit_state = world.cell(pos);
    let target = if hit_state.rules().water_container != WaterContainer::None && matches!(held.content, BucketContent::Water) {
        pos
    } else {
        offset_pos
    };

    if bucket_empty_contents(
        world,
        water_evaporates,
        held.content,
        target,
        Some((pos, face)),
        is_sneaking,
        is_client_side,
        events,
    ) {
        let empty_item = bucket_empty_success_item(held, has_infinite_materials);
        let swap = create_filled_result(
            held,
            empty_item,
            has_infinite_materials,
            inventory_contains_result,
            inventory_can_add_result,
        );
        return BucketUseResult::Success(swap);
    }

    if matches!(held.content, BucketContent::Empty) {
        if let Some((picked, sound)) = bucket_pickup_block(world, pos, events) {
            events.push(BucketEvent::PlaySound { pos, sound });
            events.push(BucketEvent::GameEvent {
                pos,
                event: BucketGameEvent::FluidPickup,
            });
            let swap = create_filled_result(
                held,
                picked,
                has_infinite_materials,
                inventory_contains_result,
                inventory_can_add_result,
            );
            return BucketUseResult::Success(swap);
        }
    }

    BucketUseResult::Fail
}

// ============================================================================
// 2. Pressure plates (`BasePressurePlateBlock`, `PressurePlateBlock`, `WeightedPressurePlateBlock`)
// ============================================================================

/// `BasePressurePlateBlock.getPressedTime`: 20 ticks (wood, stone, polished blackstone).
pub const PRESSURE_PLATE_PRESSED_TIME_STANDARD: u32 = 20;
/// `WeightedPressurePlateBlock.getPressedTime`: 10 ticks (light/gold, heavy/iron).
pub const PRESSURE_PLATE_PRESSED_TIME_WEIGHTED: u32 = 10;

/// `BasePressurePlateBlock.TOUCH_AABB`: `Block.column(14.0, 0.0, 4.0)`.
pub const PRESSURE_PLATE_TOUCH_AABB: Aabb = Aabb {
    min: Vec3 {
        x: 1.0 / 16.0,
        y: 0.0,
        z: 1.0 / 16.0,
    },
    max: Vec3 {
        x: 15.0 / 16.0,
        y: 4.0 / 16.0,
        z: 15.0 / 16.0,
    },
};

/// `BasePressurePlateBlock.SHAPE` (unpressed).
pub const PRESSURE_PLATE_SHAPE: Aabb = Aabb {
    min: Vec3 {
        x: 1.0 / 16.0,
        y: 0.0,
        z: 1.0 / 16.0,
    },
    max: Vec3 {
        x: 15.0 / 16.0,
        y: 1.0 / 16.0,
        z: 15.0 / 16.0,
    },
};

/// `BasePressurePlateBlock.SHAPE_PRESSED` (signal > 0).
pub const PRESSURE_PLATE_SHAPE_PRESSED: Aabb = Aabb {
    min: Vec3 {
        x: 1.0 / 16.0,
        y: 0.0,
        z: 1.0 / 16.0,
    },
    max: Vec3 {
        x: 15.0 / 16.0,
        y: 0.5 / 16.0,
        z: 15.0 / 16.0,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressurePlateKind {
    /// Wooden (`mobs_only: false`, `Sensitivity.EVERYTHING`) and Stone / Polished Blackstone
    /// (`mobs_only: true`, `Sensitivity.MOBS`).
    Binary { mobs_only: bool },
    /// Light weighted (`max_weight: 15`) and Heavy weighted (`max_weight: 150`).
    Weighted { max_weight: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PressurePlateState {
    pub block_id: BlockId,
    pub kind: PressurePlateKind,
    pub signal: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerEntity {
    pub bbox: Aabb,
    pub is_living: bool,
    pub is_spectator: bool,
    pub is_ignoring_block_triggers: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlateEvent {
    SetSignal { pos: BlockPos, flags: i32, signal: u8 },
    UpdateNeighborsAt { pos: BlockPos, source_id: BlockId },
    PlaySound { pos: BlockPos, activated: bool },
    GameEvent { pos: BlockPos, activated: bool },
    ScheduleTick { pos: BlockPos, block_id: BlockId, delay: u32 },
}

impl PressurePlateState {
    #[inline]
    pub const fn pressed_time(self) -> u32 {
        match self.kind {
            PressurePlateKind::Binary { .. } => PRESSURE_PLATE_PRESSED_TIME_STANDARD,
            PressurePlateKind::Weighted { .. } => PRESSURE_PLATE_PRESSED_TIME_WEIGHTED,
        }
    }


    /// `BasePressurePlateBlock.ownSignal`.
    #[inline]
    pub const fn own_signal(self) -> u8 {
        self.signal
    }

    /// `BasePressurePlateBlock.getDirectSignal` (`@0-15`): `signal` only for `Direction::Up`.
    #[inline]
    pub const fn direct_signal(self, dir: Direction) -> u8 {
        if matches!(dir, Direction::Up) {
            self.signal
        } else {
            0
        }
    }

    /// `BasePressurePlateBlock.getEntityCount` (`@0-24`) + `PressurePlateBlock`/`WeightedPressurePlateBlock.getSignalStrength`.
    pub fn compute_signal_strength(self, pos: BlockPos, entities: &[TriggerEntity]) -> u8 {
        let touch = PRESSURE_PLATE_TOUCH_AABB.moved(Vec3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64));
        let mobs_only = matches!(self.kind, PressurePlateKind::Binary { mobs_only: true });
        let count = entities
            .iter()
            .filter(|e| {
                !e.is_spectator
                    && !e.is_ignoring_block_triggers
                    && (!mobs_only || e.is_living)
                    && e.bbox.intersects(touch)
            })
            .count() as i32;
        weighted_or_binary_signal(self.kind, count)
    }
}

/// Exact `PressurePlateBlock` (`> 0 ? 15 : 0`) and `WeightedPressurePlateBlock`
/// (`Mth.ceil((float) min(count, maxWeight) / (float) maxWeight * 15.0F)`) signal formula.
#[inline]
pub fn weighted_or_binary_signal(kind: PressurePlateKind, count: i32) -> u8 {
    match kind {
        PressurePlateKind::Binary { .. } => {
            if count > 0 { 15 } else { 0 }
        }
        PressurePlateKind::Weighted { max_weight } => {
            let i = count.min(max_weight);
            if i > 0 {
                let f = (max_weight.min(i) as f32) / (max_weight as f32);
                (f * 15.0f32).ceil() as u8
            } else {
                0
            }
        }
    }
}

fn push_plate_neighbor_updates(block_id: BlockId, pos: BlockPos, events: &mut Vec<PlateEvent>) {
    events.push(PlateEvent::UpdateNeighborsAt {
        pos,
        source_id: block_id,
    });
    events.push(PlateEvent::UpdateNeighborsAt {
        pos: Direction::Down.relative(pos),
        source_id: block_id,
    });
}

/// `BasePressurePlateBlock.checkPressed` (`@0-168`).
pub fn pressure_plate_check_pressed(
    plate: &mut PressurePlateState,
    pos: BlockPos,
    entities: &[TriggerEntity],
    events: &mut Vec<PlateEvent>,
) {
    let old_signal = plate.signal;
    let new_signal = plate.compute_signal_strength(pos, entities);
    let was_pressed = old_signal > 0;
    let is_pressed = new_signal > 0;

    if old_signal != new_signal {
        plate.signal = new_signal;
        events.push(PlateEvent::SetSignal {
            pos,
            flags: UPDATE_CLIENTS,
            signal: new_signal,
        });
        push_plate_neighbor_updates(plate.block_id, pos, events);
    }

    if !is_pressed && was_pressed {
        events.push(PlateEvent::PlaySound {
            pos,
            activated: false,
        });
        events.push(PlateEvent::GameEvent {
            pos,
            activated: false,
        });
    } else if is_pressed && !was_pressed {
        events.push(PlateEvent::PlaySound {
            pos,
            activated: true,
        });
        events.push(PlateEvent::GameEvent {
            pos,
            activated: true,
        });
    }

    if is_pressed {
        events.push(PlateEvent::ScheduleTick {
            pos,
            block_id: plate.block_id,
            delay: plate.pressed_time(),
        });
    }
}

/// `BasePressurePlateBlock.entityInside` (`@0-31`).
pub fn pressure_plate_entity_inside(
    plate: &mut PressurePlateState,
    pos: BlockPos,
    is_client_side: bool,
    entities: &[TriggerEntity],
    events: &mut Vec<PlateEvent>,
) {
    if is_client_side {
        return;
    }
    if plate.signal == 0 {
        pressure_plate_check_pressed(plate, pos, entities, events);
    }
}

/// `BasePressurePlateBlock.tick` (`@0-22`).
pub fn pressure_plate_tick(
    plate: &mut PressurePlateState,
    pos: BlockPos,
    entities: &[TriggerEntity],
    events: &mut Vec<PlateEvent>,
) {
    if plate.signal > 0 {
        pressure_plate_check_pressed(plate, pos, entities, events);
    }
}

/// `BasePressurePlateBlock.affectNeighborsAfterRemoval` (`@0-19`).
pub fn pressure_plate_after_removal(
    plate: PressurePlateState,
    pos: BlockPos,
    moved_by_piston: bool,
    events: &mut Vec<PlateEvent>,
) {
    if !moved_by_piston && plate.signal > 0 {
        push_plate_neighbor_updates(plate.block_id, pos, events);
    }
}

// ============================================================================
// 3. Buttons (`ButtonBlock`, `FaceAttachedHorizontalDirectionalBlock`)
// ============================================================================

/// Stone and polished blackstone buttons (`ticksToStayPressed = 20`).
pub const BUTTON_STAY_STONE: u32 = 20;
/// All 12 wooden buttons (`ticksToStayPressed = 30`).
pub const BUTTON_STAY_WOOD: u32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonState {
    pub block_id: BlockId,
    pub face: AttachFace,
    pub facing: Direction,
    pub powered: bool,
    pub ticks_to_stay_pressed: u32,
    pub arrow_activatable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonUseResult {
    Consume,
    Success,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonEvent {
    SetPowered { pos: BlockPos, flags: i32, powered: bool },
    UpdateNeighborsAt { pos: BlockPos, source_id: BlockId },
    ScheduleTick { pos: BlockPos, block_id: BlockId, delay: u32 },
    PlaySound { pos: BlockPos, activated: bool },
    GameEvent { pos: BlockPos, activated: bool },
}

impl ButtonState {
    /// `FaceAttachedHorizontalDirectionalBlock.getConnectedDirection` (`CEILING->DOWN, FLOOR->UP, WALL->FACING`).
    #[inline]
    pub const fn connected_direction(self) -> Direction {
        match self.face {
            AttachFace::Ceiling => Direction::Down,
            AttachFace::Floor => Direction::Up,
            AttachFace::Wall => self.facing,
        }
    }

    /// `ButtonBlock.updateNeighbours` secondary cell (`pos.relative(getConnectedDirection(state).opposite())`).
    #[inline]
    pub const fn support_pos(self, pos: BlockPos) -> BlockPos {
        self.connected_direction().opposite().relative(pos)
    }

    /// `ButtonBlock.ownSignal` (`@0-22`).
    #[inline]
    pub const fn own_signal(self) -> u8 {
        if self.powered { 15 } else { 0 }
    }

    /// `ButtonBlock.getDirectSignal` (`@0-29`): 15 toward `getConnectedDirection(state)`.
    #[inline]
    pub const fn direct_signal(self, dir: Direction) -> u8 {
        if self.powered && (self.connected_direction() as u8 == dir as u8) {
            15
        } else {
            0
        }
    }

    /// `ButtonBlock.getShape` (`@0-13`, `makeShapes` `@0-45`).
    pub const fn shape(self) -> Aabb {
        let depth = if self.powered { 1.0 / 16.0 } else { 2.0 / 16.0 };
        let w_lo = 5.0 / 16.0;
        let w_hi = 11.0 / 16.0;
        let h_lo = 6.0 / 16.0;
        let h_hi = 10.0 / 16.0;
        let ns_axis = matches!(self.facing, Direction::North | Direction::South);
        match self.face {
            AttachFace::Floor => {
                if ns_axis {
                    aabb(w_lo, 0.0, h_lo, w_hi, depth, h_hi)
                } else {
                    aabb(h_lo, 0.0, w_lo, h_hi, depth, w_hi)
                }
            }
            AttachFace::Ceiling => {
                if ns_axis {
                    aabb(w_lo, 1.0 - depth, h_lo, w_hi, 1.0, h_hi)
                } else {
                    aabb(h_lo, 1.0 - depth, w_lo, h_hi, 1.0, w_hi)
                }
            }
            AttachFace::Wall => match self.facing {
                Direction::North => aabb(w_lo, h_lo, 1.0 - depth, w_hi, h_hi, 1.0),
                Direction::South => aabb(w_lo, h_lo, 0.0, w_hi, h_hi, depth),
                Direction::West => aabb(1.0 - depth, h_lo, w_lo, 1.0, h_hi, w_hi),
                Direction::East => aabb(0.0, h_lo, w_lo, depth, h_hi, w_hi),
                Direction::Down | Direction::Up => aabb(w_lo, h_lo, 1.0 - depth, w_hi, h_hi, 1.0),
            },
        }
    }
}

fn push_button_neighbor_updates(btn: ButtonState, pos: BlockPos, events: &mut Vec<ButtonEvent>) {
    events.push(ButtonEvent::UpdateNeighborsAt {
        pos,
        source_id: btn.block_id,
    });
    events.push(ButtonEvent::UpdateNeighborsAt {
        pos: btn.support_pos(pos),
        source_id: btn.block_id,
    });
}

/// `ButtonBlock.press` (`@0-57`).
pub fn button_press(btn: &mut ButtonState, pos: BlockPos, events: &mut Vec<ButtonEvent>) {
    btn.powered = true;
    events.push(ButtonEvent::SetPowered {
        pos,
        flags: UPDATE_ALL,
        powered: true,
    });
    push_button_neighbor_updates(*btn, pos, events);
    events.push(ButtonEvent::ScheduleTick {
        pos,
        block_id: btn.block_id,
        delay: btn.ticks_to_stay_pressed,
    });
    events.push(ButtonEvent::PlaySound {
        pos,
        activated: true,
    });
    events.push(ButtonEvent::GameEvent {
        pos,
        activated: true,
    });
}

/// `ButtonBlock.useWithoutItem` (`@0-32`).
pub fn button_use_without_item(
    btn: &mut ButtonState,
    pos: BlockPos,
    events: &mut Vec<ButtonEvent>,
) -> ButtonUseResult {
    if btn.powered {
        ButtonUseResult::Consume
    } else {
        button_press(btn, pos, events);
        ButtonUseResult::Success
    }
}

/// `ButtonBlock.checkPressed` (`@0-167`).
pub fn button_check_pressed(
    btn: &mut ButtonState,
    pos: BlockPos,
    arrow_boxes: &[Aabb],
    events: &mut Vec<ButtonEvent>,
) {
    let arrow_present = if btn.arrow_activatable {
        let world_shape = btn
            .shape()
            .moved(Vec3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64));
        arrow_boxes.iter().any(|b| b.intersects(world_shape))
    } else {
        false
    };
    let was_powered = btn.powered;
    if arrow_present != was_powered {
        btn.powered = arrow_present;
        events.push(ButtonEvent::SetPowered {
            pos,
            flags: UPDATE_ALL,
            powered: arrow_present,
        });
        push_button_neighbor_updates(*btn, pos, events);
        events.push(ButtonEvent::PlaySound {
            pos,
            activated: arrow_present,
        });
        events.push(ButtonEvent::GameEvent {
            pos,
            activated: arrow_present,
        });
    }
    if arrow_present {
        events.push(ButtonEvent::ScheduleTick {
            pos,
            block_id: btn.block_id,
            delay: btn.ticks_to_stay_pressed,
        });
    }
}

/// `ButtonBlock.tick` (`@0-24`).
pub fn button_tick(
    btn: &mut ButtonState,
    pos: BlockPos,
    arrow_boxes: &[Aabb],
    events: &mut Vec<ButtonEvent>,
) {
    if btn.powered {
        button_check_pressed(btn, pos, arrow_boxes, events);
    }
}

/// `ButtonBlock.entityInside` (`@0-41`).
pub fn button_entity_inside(
    btn: &mut ButtonState,
    pos: BlockPos,
    is_client_side: bool,
    arrow_boxes: &[Aabb],
    events: &mut Vec<ButtonEvent>,
) {
    if !is_client_side && btn.arrow_activatable && !btn.powered {
        button_check_pressed(btn, pos, arrow_boxes, events);
    }
}

/// `ButtonBlock.affectNeighborsAfterRemoval` (`@0-28`).
pub fn button_after_removal(
    btn: ButtonState,
    pos: BlockPos,
    moved_by_piston: bool,
    events: &mut Vec<ButtonEvent>,
) {
    if !moved_by_piston && btn.powered {
        push_button_neighbor_updates(btn, pos, events);
    }
}

// ============================================================================
// 4. Tripwire Hook & String (`TripWireBlock`, `TripWireHookBlock`)
// ============================================================================

/// `TripWireHookBlock.WIRE_DIST_MIN`.
pub const TRIPWIRE_DIST_MIN: i32 = 1;
/// `TripWireHookBlock.WIRE_DIST_MAX` (`i < 42` in `updateSource` & `calculateState`, max 40 wires).
pub const TRIPWIRE_DIST_MAX: i32 = 42;
/// `TripWireBlock.RECHECK_PERIOD` / `TripWireHookBlock.RECHECK_PERIOD`.
pub const TRIPWIRE_RECHECK_PERIOD: u32 = 10;

/// `TripWireBlock.SHAPE_ATTACHED` (`Block.column(16, 1, 2.5)`).
pub const TRIPWIRE_SHAPE_ATTACHED: Aabb = Aabb {
    min: Vec3 { x: 0.0, y: 1.0 / 16.0, z: 0.0 },
    max: Vec3 { x: 1.0, y: 2.5 / 16.0, z: 1.0 },
};

/// `TripWireBlock.SHAPE_NOT_ATTACHED` (`Block.column(16, 0, 8)`).
pub const TRIPWIRE_SHAPE_UNATTACHED: Aabb = Aabb {
    min: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    max: Vec3 { x: 1.0, y: 8.0 / 16.0, z: 1.0 },
};

/// `TripWireHookBlock.getShape` (`boxZ(6, 0, 10, 10, 16)` rotated horizontally).
pub const fn tripwire_hook_shape(facing: Direction) -> Aabb {
    let w_lo = 5.0 / 16.0;
    let w_hi = 11.0 / 16.0;
    let h_hi = 10.0 / 16.0;
    let depth = 6.0 / 16.0;
    match facing {
        Direction::North => aabb(w_lo, 0.0, 1.0 - depth, w_hi, h_hi, 1.0),
        Direction::South => aabb(w_lo, 0.0, 0.0, w_hi, h_hi, depth),
        Direction::West => aabb(1.0 - depth, 0.0, w_lo, 1.0, h_hi, w_hi),
        Direction::East => aabb(0.0, 0.0, w_lo, depth, h_hi, w_hi),
        Direction::Down | Direction::Up => aabb(w_lo, 0.0, 1.0 - depth, w_hi, h_hi, 1.0),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TripwireWireState {
    pub block_id: BlockId,
    pub powered: bool,
    pub attached: bool,
    pub disarmed: bool,
}

impl TripwireWireState {
    #[inline]
    pub const fn shape(self) -> Aabb {
        if self.attached {
            TRIPWIRE_SHAPE_ATTACHED
        } else {
            TRIPWIRE_SHAPE_UNATTACHED
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TripwireHookState {
    pub block_id: BlockId,
    pub facing: Direction,
    pub powered: bool,
    pub attached: bool,
}

impl TripwireHookState {
    /// `TripWireHookBlock.ownSignal` (`@0-22`).
    #[inline]
    pub const fn own_signal(self) -> u8 {
        if self.powered { 15 } else { 0 }
    }

    /// `TripWireHookBlock.getDirectSignal` (`@0-34`): 15 only toward `FACING` when `POWERED`.
    #[inline]
    pub const fn direct_signal(self, dir: Direction) -> u8 {
        if self.powered && (self.facing as u8 == dir as u8) {
            15
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TripwireCell {
    Empty,
    Wire(TripwireWireState),
    Hook(TripwireHookState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TripwireHookFeedback {
    ClickOn,
    ClickOff,
    Attach,
    Detach,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TripwireEvent {
    SetWire { pos: BlockPos, flags: i32, wire: TripwireWireState },
    SetHook { pos: BlockPos, flags: i32, hook: TripwireHookState },
    ScheduleTick { pos: BlockPos, block_id: BlockId, delay: u32 },
    UpdateNeighborsAt { pos: BlockPos, source_id: BlockId },
    HookFeedback { pos: BlockPos, feedback: TripwireHookFeedback },
    GameEventShear { pos: BlockPos },
}

/// The tripwire rules' view of a cell: a wire, a hook, or neither.
pub fn tripwire_cell(world: &impl WorldMut, pos: BlockPos) -> TripwireCell {
    match world.cell(pos).kind() {
        CellState::TripwireWire(w) => TripwireCell::Wire(w),
        CellState::TripwireHook(h) => TripwireCell::Hook(h),
        _ => TripwireCell::Empty,
    }
}

pub(crate) fn wire_state(state: StateId, wire: TripwireWireState) -> StateId {
    let base = if state.block() == wire.block_id { state } else { wire.block_id.default_state() };
    base.with_bool("powered", wire.powered).with_bool("attached", wire.attached).with_bool("disarmed", wire.disarmed)
}

pub(crate) fn hook_state(state: StateId, hook: TripwireHookState) -> StateId {
    let base = if state.block() == hook.block_id { state } else { hook.block_id.default_state() };
    base.with("facing", DIRECTION[hook.facing as usize]).with_bool("powered", hook.powered).with_bool("attached", hook.attached)
}

fn set_tripwire(world: &mut impl WorldMut, pos: BlockPos, cell: TripwireCell) {
    let state = match cell {
        TripwireCell::Empty => StateId::AIR,
        TripwireCell::Wire(wire) => wire_state(world.cell(pos), wire),
        TripwireCell::Hook(hook) => hook_state(world.cell(pos), hook),
    };
    world.set(pos, state);
}

fn push_hook_neighbor_updates(
    hook_id: BlockId,
    pos: BlockPos,
    facing: Direction,
    events: &mut Vec<TripwireEvent>,
) {
    let opp = facing.opposite();
    events.push(TripwireEvent::UpdateNeighborsAt {
        pos,
        source_id: hook_id,
    });
    events.push(TripwireEvent::UpdateNeighborsAt {
        pos: opp.relative(pos),
        source_id: hook_id,
    });
}

fn emit_hook_state(
    pos: BlockPos,
    attached_new: bool,
    powered_new: bool,
    attached_old: bool,
    powered_old: bool,
    events: &mut Vec<TripwireEvent>,
) {
    let feedback = if powered_new && !powered_old {
        Some(TripwireHookFeedback::ClickOn)
    } else if !powered_new && powered_old {
        Some(TripwireHookFeedback::ClickOff)
    } else if attached_new && !attached_old {
        Some(TripwireHookFeedback::Attach)
    } else if !attached_new && attached_old {
        Some(TripwireHookFeedback::Detach)
    } else {
        None
    };
    if let Some(fb) = feedback {
        events.push(TripwireEvent::HookFeedback { pos, feedback: fb });
    }
}

fn set_hook_if_changed(
    world: &mut impl WorldMut,
    pos: BlockPos,
    next: TripwireHookState,
    events: &mut Vec<TripwireEvent>,
) {
    if tripwire_cell(world, pos) != TripwireCell::Hook(next) {
        set_tripwire(world, pos, TripwireCell::Hook(next));
        events.push(TripwireEvent::SetHook {
            pos,
            flags: UPDATE_ALL,
            hook: next,
        });
    }
}

fn set_wire_if_changed(
    world: &mut impl WorldMut,
    pos: BlockPos,
    next: TripwireWireState,
    flags: i32,
    events: &mut Vec<TripwireEvent>,
) {
    if tripwire_cell(world, pos) != TripwireCell::Wire(next) {
        set_tripwire(world, pos, TripwireCell::Wire(next));
        events.push(TripwireEvent::SetWire {
            pos,
            flags,
            wire: next,
        });
    }
}

fn hook_on_removed(
    hook_state: TripwireHookState,
    world: &mut impl WorldMut,
    pos: BlockPos,
    events: &mut Vec<TripwireEvent>,
) {
    if hook_state.attached || hook_state.powered {
        tripwire_hook_calculate_state(world, pos, hook_state, true, false, -1, None, events);
    }
    if hook_state.powered {
        push_hook_neighbor_updates(hook_state.block_id, pos, hook_state.facing, events);
    }
}

/// `TripWireHookBlock.calculateState` (`@0-609`).
#[allow(clippy::too_many_arguments)]
pub fn tripwire_hook_calculate_state(
    world: &mut impl WorldMut,
    pos: BlockPos,
    state: TripwireHookState,
    is_being_destroyed: bool,
    notify_neighbors: bool,
    changed_dist: i32,
    changed_wire_state: Option<TripwireWireState>,
    events: &mut Vec<TripwireEvent>,
) {
    let dir = state.facing;
    if !dir.is_horizontal() {
        return;
    }
    let attached_old = state.attached;
    let powered_old = state.powered;
    let mut attached_new = !is_being_destroyed;
    let mut powered_new = false;
    let mut far_dist: i32 = 0;
    let mut astate: [Option<TripwireWireState>; 42] = [None; 42];

    for i in 1..TRIPWIRE_DIST_MAX {
        let p = step_n(dir, pos, i);
        let cell = tripwire_cell(world, p);
        match cell {
            TripwireCell::Hook(far_hook) => {
                if far_hook.facing == dir.opposite() {
                    far_dist = i;
                }
                break;
            }
            TripwireCell::Wire(mut wire_st) => {
                if i == changed_dist {
                    if let Some(override_w) = changed_wire_state {
                        wire_st = override_w;
                    }
                }
                let not_disarmed = !wire_st.disarmed;
                powered_new |= not_disarmed && wire_st.powered;
                astate[i as usize] = Some(wire_st);
                if i == changed_dist {
                    events.push(TripwireEvent::ScheduleTick {
                        pos,
                        block_id: state.block_id,
                        delay: TRIPWIRE_RECHECK_PERIOD,
                    });
                    attached_new &= not_disarmed;
                }
            }
            TripwireCell::Empty => {
                if i == changed_dist {
                    if let Some(wire_st) = changed_wire_state {
                        let not_disarmed = !wire_st.disarmed;
                        powered_new |= not_disarmed && wire_st.powered;
                        astate[i as usize] = Some(wire_st);
                        events.push(TripwireEvent::ScheduleTick {
                            pos,
                            block_id: state.block_id,
                            delay: TRIPWIRE_RECHECK_PERIOD,
                        });
                        attached_new &= not_disarmed;
                        continue;
                    }
                }
                astate[i as usize] = None;
                attached_new = false;
            }
        }
    }

    attached_new &= far_dist > TRIPWIRE_DIST_MIN;
    powered_new &= attached_new;

    let hook_new = TripwireHookState {
        block_id: state.block_id,
        facing: dir,
        powered: powered_new,
        attached: attached_new,
    };

    if far_dist > 0 {
        let far_pos = step_n(dir, pos, far_dist);
        let opp = dir.opposite();
        let far_new = TripwireHookState {
            facing: opp,
            ..hook_new
        };
        set_hook_if_changed(world, far_pos, far_new, events);
        push_hook_neighbor_updates(state.block_id, far_pos, opp, events);
        if !matches!(tripwire_cell(world, pos), TripwireCell::Hook(_)) {
            hook_on_removed(hook_new, world, pos, events);
            return;
        }
        emit_hook_state(far_pos, attached_new, powered_new, attached_old, powered_old, events);
    }

    emit_hook_state(pos, attached_new, powered_new, attached_old, powered_old, events);
    if !is_being_destroyed {
        set_hook_if_changed(world, pos, hook_new, events);
        if notify_neighbors {
            push_hook_neighbor_updates(state.block_id, pos, dir, events);
        }
    }

    if attached_old != attached_new {
        for j in 1..far_dist {
            if let Some(mut wire_st) = astate[j as usize] {
                let wire_pos = step_n(dir, pos, j);
                if matches!(tripwire_cell(world, wire_pos), TripwireCell::Wire(_) | TripwireCell::Hook(_)) {
                    wire_st.attached = attached_new;
                    set_wire_if_changed(world, wire_pos, wire_st, UPDATE_ALL, events);
                }
            }
        }
    }
}

/// `TripWireBlock.updateSource` (`@0-136`): walks `SOUTH` and `WEST` rays only, `i < 42`.
pub fn tripwire_update_source(
    world: &mut impl WorldMut,
    pos: BlockPos,
    changed_wire_state: TripwireWireState,
    events: &mut Vec<TripwireEvent>,
) {
    for dir in [Direction::South, Direction::West] {
        for i in 1..TRIPWIRE_DIST_MAX {
            let p = step_n(dir, pos, i);
            match tripwire_cell(world, p) {
                TripwireCell::Hook(hook_st) => {
                    if hook_st.facing == dir.opposite() {
                        tripwire_hook_calculate_state(
                            world,
                            p,
                            hook_st,
                            false,
                            true,
                            i,
                            Some(changed_wire_state),
                            events,
                        );
                    }
                    break;
                }
                TripwireCell::Wire(_) => {}
                TripwireCell::Empty => break,
            }
        }
    }
}

/// `TripWireBlock.checkPressed(Level, BlockPos, List)` (`@0-166`).
pub fn tripwire_check_pressed_with_entities(
    world: &mut impl WorldMut,
    pos: BlockPos,
    entities: &[TriggerEntity],
    events: &mut Vec<TripwireEvent>,
) {
    let TripwireCell::Wire(mut wire_st) = tripwire_cell(world, pos) else {
        return;
    };
    let flag = wire_st.powered;
    // Notice TripWireBlock does NOT filter out spectators (`isIgnoringBlockTriggers` only).
    let occupied = entities.iter().any(|e| !e.is_ignoring_block_triggers);
    if occupied != flag {
        wire_st.powered = occupied;
        set_wire_if_changed(world, pos, wire_st, UPDATE_ALL, events);
        tripwire_update_source(world, pos, wire_st, events);
    }
    if occupied {
        events.push(TripwireEvent::ScheduleTick {
            pos,
            block_id: wire_st.block_id,
            delay: TRIPWIRE_RECHECK_PERIOD,
        });
    } else if flag {
        // Delay-0 echo holding `hasScheduledTick` gate for one tick.
        events.push(TripwireEvent::ScheduleTick {
            pos,
            block_id: wire_st.block_id,
            delay: 0,
        });
    }
}

/// `TripWireBlock.entityInside` (`@0-50`).
pub fn tripwire_entity_inside(
    world: &mut impl WorldMut,
    pos: BlockPos,
    is_client_side: bool,
    entity: TriggerEntity,
    events: &mut Vec<TripwireEvent>,
) {
    if is_client_side {
        return;
    }
    let TripwireCell::Wire(wire_st) = tripwire_cell(world, pos) else {
        return;
    };
    if wire_st.powered || world.has_scheduled_tick(pos, wire_st.block_id) {
        return;
    }
    tripwire_check_pressed_with_entities(world, pos, &[entity], events);
}

/// `TripWireBlock.tick` (`@0-27`) + `TripWireBlock.checkPressed(Level, BlockPos)` (`@0-34`).
pub fn tripwire_tick(
    world: &mut impl WorldMut,
    pos: BlockPos,
    entities: &[TriggerEntity],
    events: &mut Vec<TripwireEvent>,
) {
    let TripwireCell::Wire(wire_st) = tripwire_cell(world, pos) else {
        return;
    };
    if !wire_st.powered {
        return;
    }
    let shape = wire_st
        .shape()
        .moved(Vec3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64));
    let overlapping: Vec<TriggerEntity> = entities
        .iter()
        .copied()
        .filter(|e| e.bbox.intersects(shape))
        .collect();
    tripwire_check_pressed_with_entities(world, pos, &overlapping, events);
}

/// `TripWireHookBlock.tick` (`@0-10`).
pub fn tripwire_hook_tick(
    world: &mut impl WorldMut,
    pos: BlockPos,
    events: &mut Vec<TripwireEvent>,
) {
    let TripwireCell::Hook(hook_st) = tripwire_cell(world, pos) else {
        return;
    };
    tripwire_hook_calculate_state(world, pos, hook_st, false, true, -1, None, events);
}

/// `TripWireBlock.playerWillDestroy` (`@0-74`): shears disarm sets `DISARMED=true` with `flags = 260`.
pub fn tripwire_player_will_destroy(
    world: &mut impl WorldMut,
    pos: BlockPos,
    is_client_side: bool,
    main_hand_is_shears: bool,
    events: &mut Vec<TripwireEvent>,
) {
    if is_client_side || !main_hand_is_shears {
        return;
    }
    let TripwireCell::Wire(mut wire_st) = tripwire_cell(world, pos) else {
        return;
    };
    wire_st.disarmed = true;
    set_wire_if_changed(world, pos, wire_st, UPDATE_NONE, events);
    events.push(TripwireEvent::GameEventShear { pos });
}

/// `TripWireBlock.affectNeighborsAfterRemoval` (`@0-25`): phantom `POWERED = true` copy.
pub fn tripwire_after_removal(
    world: &mut impl WorldMut,
    pos: BlockPos,
    old_wire: TripwireWireState,
    moved_by_piston: bool,
    events: &mut Vec<TripwireEvent>,
) {
    if !moved_by_piston {
        let phantom = TripwireWireState {
            powered: true,
            ..old_wire
        };
        tripwire_update_source(world, pos, phantom, events);
    }
}

/// `TripWireHookBlock.affectNeighborsAfterRemoval` (`@0-12`).
pub fn tripwire_hook_after_removal(
    world: &mut impl WorldMut,
    pos: BlockPos,
    old_hook: TripwireHookState,
    moved_by_piston: bool,
    events: &mut Vec<TripwireEvent>,
) {
    if !moved_by_piston {
        hook_on_removed(old_hook, world, pos, events);
    }
}

// ============================================================================
// 5. Bubble column `entityInside` law
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BubbleColumnStep {
    pub vy: f64,
    pub reset_fall_distance: bool,
    pub applied: bool,
}

/// `BubbleColumnBlock.entityInside` above-cell probe (`@15-44`):
/// `aboveState.getCollisionShape(level, pos).isEmpty() && aboveState.getFluidState().isEmpty()`.
#[inline]
pub const fn bubble_column_open_top(above_collision_empty: bool, above_fluid_empty: bool) -> bool {
    above_collision_empty && above_fluid_empty
}

/// `Entity.handleOnAboveBubbleColumn` (`@0-66`): no fall-distance reset.
#[inline]
pub fn on_above_bubble_column(vy: f64, drag_down: bool) -> f64 {
    if drag_down {
        (-0.9f64).max(vy - 0.03f64)
    } else {
        (1.8f64).min(vy + 0.1f64)
    }
}

/// `Entity.handleOnInsideBubbleColumn` (`@0-59`): resets `fallDistance = 0.0`.
#[inline]
pub fn on_inside_bubble_column(vy: f64, drag_down: bool) -> (f64, bool) {
    let next_vy = if drag_down {
        (-0.3f64).max(vy - 0.03f64)
    } else {
        (0.7f64).min(vy + 0.06f64)
    };
    (next_vy, true)
}

/// `BubbleColumnBlock.entityInside` (`@0-91`): pure velocity + fall-reset transition.
#[inline]
pub fn bubble_column_entity_inside(
    vy: f64,
    drag_down: bool,
    open_top: bool,
    intersects: bool,
) -> BubbleColumnStep {
    if !intersects {
        return BubbleColumnStep {
            vy,
            reset_fall_distance: false,
            applied: false,
        };
    }
    if open_top {
        BubbleColumnStep {
            vy: on_above_bubble_column(vy, drag_down),
            reset_fall_distance: false,
            applied: true,
        }
    } else {
        let (next_vy, reset) = on_inside_bubble_column(vy, drag_down);
        BubbleColumnStep {
            vy: next_vy,
            reset_fall_distance: reset,
            applied: true,
        }
    }
}

/// Convenience probe reading `world.hook_block([pos[0], pos[1] + 1, pos[2]])`.
pub fn bubble_column_world_step(
    world: &impl World,
    pos: BlockPos,
    vy: f64,
    drag_down: bool,
    intersects: bool,
) -> BubbleColumnStep {
    if !intersects {
        return BubbleColumnStep {
            vy,
            reset_fall_distance: false,
            applied: false,
        };
    }
    let above = world.hook_block([pos[0], pos[1] + 1, pos[2]]);
    let open_top = bubble_column_open_top(above.shape.is_empty(), above.fluid.kind == FluidKind::Empty);
    bubble_column_entity_inside(vy, drag_down, open_top, true)
}
