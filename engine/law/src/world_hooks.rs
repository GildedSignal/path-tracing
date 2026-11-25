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
//!
//! Block identity is strictly data-driven (`BlockId` and properties, never block-name strings).

use crate::interact::{AttachFace, BlockHitResult, BlockId, ClipFluidMode, Direction};
use crate::basis::cell::{CellState, StateId, WaterContainer, WorldMut, DIRECTION};
use crate::neighbor::update_flags;
use crate::{Aabb, BlockPos, Vec3};

/// `pos.relative(dir, n)`.
#[inline]
pub const fn step_n(dir: Direction, [x, y, z]: BlockPos, n: i32) -> BlockPos {
    let [dx, dy, dz] = dir.step();
    [x + dx * n, y + dy * n, z + dz * n]
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
pub enum CellEvent {
    SetBlock { pos: BlockPos, state: StateId, flags: i32 },
    DestroyBlock { pos: BlockPos, drop: bool },
    UpdateNeighborsAt { pos: BlockPos, source_id: BlockId },
    ScheduleTick { pos: BlockPos, block_id: BlockId, delay: u32 },
    ScheduleFluidTick { pos: BlockPos, fluid: BucketContent, delay: u32 },
    SensorFeedback { pos: BlockPos, activated: bool },
    HookFeedback { pos: BlockPos, feedback: TripwireHookFeedback },
    GameEventShear { pos: BlockPos },
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
    events: &mut Vec<CellEvent>,
) -> Option<(BucketStack, BucketSound)> {
    let cell = world.cell(pos);
    let (content, sound) = match cell.block().name() {
        "water" if cell.get("level") == Some("0") => (BucketContent::Water, BucketSound::PickupWater),
        "lava" if cell.get("level") == Some("0") => (BucketContent::Lava, BucketSound::PickupLava),
        _ if cell.is_waterlogged_container() && cell.get("waterlogged") == Some("true") => {
            let dry = cell.with_bool("waterlogged", false);
            events.push(CellEvent::SetBlock { pos, state: dry, flags: UPDATE_ALL });
            if !world.can_survive(pos, dry) {
                events.push(CellEvent::DestroyBlock { pos, drop: true });
            }
            return Some((BucketStack::new(BucketContent::Water, 1), BucketSound::PickupWater));
        }
        _ => return None,
    };
    events.push(CellEvent::SetBlock { pos, state: StateId::AIR, flags: UPDATE_ALL_IMMEDIATE });
    Some((BucketStack::new(content, 1), sound))
}

fn emit_empty_sound_and_event(pos: BlockPos, content: BucketContent, events: &mut Vec<CellEvent>) {
    let sound = match content {
        BucketContent::Lava => BucketSound::EmptyLava,
        BucketContent::Water | BucketContent::Empty => BucketSound::EmptyWater,
    };
    events.push(CellEvent::PlaySound { pos, sound });
    events.push(CellEvent::GameEvent {
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
    events: &mut Vec<CellEvent>,
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
        events.push(CellEvent::PlaySound { pos, sound: BucketSound::FireExtinguish });
        events.push(CellEvent::EvaporateSmokeParticles { pos, count: 8 });
        return true;
    }

    if water && rules.water_container != WaterContainer::None {
        // BucketItem ignores placeLiquid's return value, including for an already wet block.
        if accepts_water && state.get("waterlogged") == Some("false") && !is_client_side {
            events.push(CellEvent::SetBlock { pos, state: state.with_bool("waterlogged", true), flags: UPDATE_ALL });
            events.push(CellEvent::ScheduleFluidTick { pos, fluid: content, delay: WATER_TICK_DELAY });
        }
    } else {
        if !is_client_side && can_replace && !rules.is_air && !matches!(state.block().name(), "water" | "lava") {
            events.push(CellEvent::DestroyBlock { pos, drop: true });
        }
        let source = bucket_source(content);
        if state != source {
            events.push(CellEvent::SetBlock { pos, state: source, flags: UPDATE_ALL_IMMEDIATE });
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
    events: &mut Vec<CellEvent>,
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
            events.push(CellEvent::PlaySound { pos, sound });
            events.push(CellEvent::GameEvent {
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
pub const PRESSURE_PLATE_TOUCH_AABB: Aabb = Aabb::block_column(14.0, 0.0, 4.0);

/// `BasePressurePlateBlock.SHAPE` (unpressed).
pub const PRESSURE_PLATE_SHAPE: Aabb = Aabb::block_column(14.0, 0.0, 1.0);

/// `BasePressurePlateBlock.SHAPE_PRESSED` (signal > 0).
pub const PRESSURE_PLATE_SHAPE_PRESSED: Aabb = Aabb::block_column(14.0, 0.0, 0.5);

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


fn notify_attached(pos: BlockPos, support: Direction, source_id: BlockId, events: &mut Vec<CellEvent>) {
    events.extend([pos, support.relative(pos)].map(|pos| CellEvent::UpdateNeighborsAt { pos, source_id }));
}

struct SensorPulse {
    pos: BlockPos,
    support: Direction,
    state: StateId,
    signal: u8,
    delay: u32,
    flags: i32,
}

impl SensorPulse {
    fn emit(self, previous: u8, hand_press: bool, events: &mut Vec<CellEvent>) {
        let Self { pos, support, state, signal, delay, flags } = self;
        let block_id = state.block();
        if signal != previous || hand_press {
            let next = if state.get("power").is_some() {
                state.with("power", &signal.to_string())
            } else {
                state.with_bool("powered", signal != 0)
            };
            events.push(CellEvent::SetBlock { pos, flags, state: next });
            notify_attached(pos, support, block_id, events);
        }
        let activated = signal > 0;
        let feedback = CellEvent::SensorFeedback { pos, activated };
        let edge = activated != (previous > 0);
        if edge && !hand_press {
            events.push(feedback);
        }
        if activated {
            events.push(CellEvent::ScheduleTick { pos, block_id, delay });
        }
        if hand_press {
            events.push(feedback);
        }
    }
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
        crate::redstone::signal_on_face(self.signal, Direction::Up, dir)
    }

    /// `BasePressurePlateBlock.getEntityCount` (`@0-24`) + `PressurePlateBlock`/`WeightedPressurePlateBlock.getSignalStrength`.
    pub fn compute_signal_strength(self, pos: BlockPos, entities: &[TriggerEntity]) -> u8 {
        let touch = PRESSURE_PLATE_TOUCH_AABB.moved(Vec3::block_corner(pos));
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

/// `BasePressurePlateBlock.checkPressed`.
fn pressure_plate_check_pressed(
    state: StateId, plate: PressurePlateState, pos: BlockPos, entities: &[TriggerEntity],
    events: &mut Vec<CellEvent>,
) {
    let pulse = SensorPulse {
        pos,
        support: Direction::Down,
        state,
        signal: plate.compute_signal_strength(pos, entities),
        delay: plate.pressed_time(),
        flags: UPDATE_CLIENTS,
    };
    pulse.emit(plate.signal, false, events);
}



/// `BasePressurePlateBlock.affectNeighborsAfterRemoval` (`@0-19`).
pub fn pressure_plate_after_removal(
    plate: PressurePlateState,
    pos: BlockPos,
    moved_by_piston: bool,
    events: &mut Vec<CellEvent>,
) {
    if !moved_by_piston && plate.own_signal() > 0 {
        notify_attached(pos, Direction::Down, plate.block_id, events);
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



impl ButtonState {
    /// `FaceAttachedHorizontalDirectionalBlock.getConnectedDirection` (`CEILING->DOWN, FLOOR->UP, WALL->FACING`).
    #[inline]
    pub const fn connected_direction(self) -> Direction {
        self.face.connected_direction(self.facing)
    }

    /// `ButtonBlock.updateNeighbours` secondary cell (`pos.relative(getConnectedDirection(state).opposite())`).
    #[inline]
    pub const fn support_pos(self, pos: BlockPos) -> BlockPos {
        self.connected_direction().opposite().relative(pos)
    }

    /// `ButtonBlock.ownSignal` (`@0-22`).
    #[inline]
    pub const fn own_signal(self) -> u8 {
        crate::redstone::binary_signal(self.powered)
    }

    /// `ButtonBlock.getDirectSignal` (`@0-29`): 15 toward `getConnectedDirection(state)`.
    #[inline]
    pub const fn direct_signal(self, dir: Direction) -> u8 {
        crate::redstone::signal_on_face(self.own_signal(), self.connected_direction(), dir)
    }

    /// `ButtonBlock.getShape` (`@0-13`, `makeShapes` `@0-45`).
    pub const fn shape(self) -> Aabb {
        let depth = if self.powered { 1.0 / 16.0 } else { 2.0 / 16.0 };
        let w_lo = 5.0 / 16.0;
        let w_hi = 11.0 / 16.0;
        let h_lo = 6.0 / 16.0;
        let h_hi = 10.0 / 16.0;
        if matches!(self.face, AttachFace::Wall) {
            return wall_sensor_shape(self.facing, h_lo, h_hi, depth);
        }
        let (x0, x1, z0, z1) = if matches!(self.facing, Direction::North | Direction::South) {
            (w_lo, w_hi, h_lo, h_hi)
        } else {
            (h_lo, h_hi, w_lo, w_hi)
        };
        Aabb::ordered(x0, 0.0, z0, x1, 1.0, z1)
            .on_unit_face(self.connected_direction().opposite(), depth)
    }
}

const fn wall_sensor_shape(facing: Direction, bottom: f64, top: f64, depth: f64) -> Aabb {
    let support = if facing.is_horizontal() { facing.opposite() } else { Direction::South };
    Aabb::ordered(5.0 / 16.0, bottom, 5.0 / 16.0, 11.0 / 16.0, top, 11.0 / 16.0)
        .on_unit_face(support, depth)
}

fn update_button(
    state: StateId, button: ButtonState, pos: BlockPos, pressed: bool, hand_press: bool,
    events: &mut Vec<CellEvent>,
) {
    let pulse = SensorPulse {
        pos,
        support: button.connected_direction().opposite(),
        state,
        signal: crate::redstone::binary_signal(pressed),
        delay: button.ticks_to_stay_pressed,
        flags: UPDATE_ALL,
    };
    pulse.emit(button.own_signal(), hand_press, events);
}

pub enum SensorInput<'a> {
    Use,
    Contact { bodies: &'a [TriggerEntity], arrows: &'a [Aabb], client: bool },
    Scheduled { bodies: &'a [TriggerEntity], arrows: &'a [Aabb] },
}

pub(crate) fn sensor_events(
    state: StateId, pos: BlockPos, input: SensorInput<'_>, events: &mut Vec<CellEvent>,
) -> Option<ButtonUseResult> {
    let sensor = state.kind();
    let active = match sensor {
        CellState::Button(button) => button.powered,
        CellState::PressurePlate(plate) => plate.signal > 0,
        _ => return None,
    };
    let (bodies, arrows) = match input {
        SensorInput::Use => {
            let CellState::Button(button) = sensor else { return None };
            if active {
                return Some(ButtonUseResult::Consume);
            }
            update_button(state, button, pos, true, true, events);
            return Some(ButtonUseResult::Success);
        }
        SensorInput::Contact { bodies, arrows, client: false } if !active => (bodies, arrows),
        SensorInput::Scheduled { bodies, arrows } if active => (bodies, arrows),
        _ => return None,
    };
    match sensor {
        CellState::PressurePlate(plate) => pressure_plate_check_pressed(state, plate, pos, bodies, events),
        CellState::Button(button) => {
            let bounds = button.shape().moved(Vec3::block_corner(pos));
            let pressed = button.arrow_activatable && arrows.iter().any(|arrow| arrow.intersects(bounds));
            update_button(state, button, pos, pressed, false, events);
        }
        _ => unreachable!(),
    }
    None
}

/// `ButtonBlock.affectNeighborsAfterRemoval` (`@0-28`).
pub fn button_after_removal(
    btn: ButtonState,
    pos: BlockPos,
    moved_by_piston: bool,
    events: &mut Vec<CellEvent>,
) {
    if !moved_by_piston && btn.own_signal() > 0 {
        notify_attached(pos, btn.connected_direction().opposite(), btn.block_id, events);
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
pub const TRIPWIRE_SHAPE_ATTACHED: Aabb = Aabb::block_column(16.0, 1.0, 2.5);

/// `TripWireBlock.SHAPE_NOT_ATTACHED` (`Block.column(16, 0, 8)`).
pub const TRIPWIRE_SHAPE_UNATTACHED: Aabb = Aabb::block_column(16.0, 0.0, 8.0);

/// `TripWireHookBlock.getShape` (`boxZ(6, 0, 10, 10, 16)` rotated horizontally).
pub const fn tripwire_hook_shape(facing: Direction) -> Aabb {
    wall_sensor_shape(facing, 0.0, 10.0 / 16.0, 6.0 / 16.0)
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
        crate::redstone::binary_signal(self.powered)
    }

    /// `TripWireHookBlock.getDirectSignal` (`@0-34`): 15 only toward `FACING` when `POWERED`.
    #[inline]
    pub const fn direct_signal(self, dir: Direction) -> u8 {
        crate::redstone::signal_on_face(self.own_signal(), self.facing, dir)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TripwireHookFeedback {
    ClickOn,
    ClickOff,
    Attach,
    Detach,
}


fn wire_state(state: StateId, wire: TripwireWireState) -> StateId {
    let base = if state.block() == wire.block_id { state } else { wire.block_id.default_state() };
    base.with_bool("powered", wire.powered).with_bool("attached", wire.attached).with_bool("disarmed", wire.disarmed)
}

fn hook_state(state: StateId, hook: TripwireHookState) -> StateId {
    let base = if state.block() == hook.block_id { state } else { hook.block_id.default_state() };
    base.with("facing", DIRECTION[hook.facing as usize]).with_bool("powered", hook.powered).with_bool("attached", hook.attached)
}




fn emit_hook_state(
    pos: BlockPos,
    attached_new: bool,
    powered_new: bool,
    attached_old: bool,
    powered_old: bool,
    events: &mut Vec<CellEvent>,
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
        events.push(CellEvent::HookFeedback { pos, feedback: fb });
    }
}

fn set_tripwire_state(
    world: &mut impl WorldMut, pos: BlockPos, next: StateId, flags: i32, events: &mut Vec<CellEvent>,
) {
    if world.cell(pos) != next {
        world.set(pos, next);
        events.push(CellEvent::SetBlock { pos, flags, state: next });
    }
}

fn set_wire(
    world: &mut impl WorldMut, pos: BlockPos, wire: TripwireWireState, flags: i32,
    events: &mut Vec<CellEvent>,
) {
    set_tripwire_state(world, pos, wire_state(world.cell(pos), wire), flags, events);
}

fn set_hook(
    world: &mut impl WorldMut, pos: BlockPos, hook: TripwireHookState, events: &mut Vec<CellEvent>,
) {
    set_tripwire_state(world, pos, hook_state(world.cell(pos), hook), UPDATE_ALL, events);
}

fn hook_on_removed(
    hook_state: TripwireHookState,
    world: &mut impl WorldMut,
    pos: BlockPos,
    events: &mut Vec<CellEvent>,
) {
    if hook_state.attached || hook_state.powered {
        tripwire_hook_calculate_state(world, pos, hook_state, true, false, -1, None, events);
    }
    if hook_state.powered {
        notify_attached(pos, hook_state.facing.opposite(), hook_state.block_id, events);
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
    events: &mut Vec<CellEvent>,
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
        let cell = world.cell(step_n(dir, pos, i)).kind();
        if let CellState::TripwireHook(hook) = cell {
            if hook.facing == dir.opposite() {
                far_dist = i;
            }
            break;
        }
        let wire = match cell {
            CellState::TripwireWire(wire) => Some(wire),
            _ => None,
        };
        let wire = if i == changed_dist { changed_wire_state.or(wire) } else { wire };
        if let Some(wire) = wire {
            let armed = !wire.disarmed;
            powered_new |= armed && wire.powered;
            astate[i as usize] = Some(wire);
            if i == changed_dist {
                events.push(CellEvent::ScheduleTick { pos, block_id: state.block_id, delay: TRIPWIRE_RECHECK_PERIOD });
                attached_new &= armed;
            }
        } else {
            attached_new = false;
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
        set_hook(world, far_pos, far_new, events);
        notify_attached(far_pos, opp.opposite(), state.block_id, events);
        if !matches!(world.cell(pos).kind(), CellState::TripwireHook(_)) {
            hook_on_removed(hook_new, world, pos, events);
            return;
        }
        emit_hook_state(far_pos, attached_new, powered_new, attached_old, powered_old, events);
    }

    emit_hook_state(pos, attached_new, powered_new, attached_old, powered_old, events);
    if !is_being_destroyed {
        set_hook(world, pos, hook_new, events);
        if notify_neighbors {
            notify_attached(pos, dir.opposite(), state.block_id, events);
        }
    }

    if attached_old != attached_new {
        for j in 1..far_dist {
            if let Some(mut wire_st) = astate[j as usize] {
                let wire_pos = step_n(dir, pos, j);
                if matches!(world.cell(wire_pos).kind(), CellState::TripwireWire(_) | CellState::TripwireHook(_)) {
                    wire_st.attached = attached_new;
                    set_wire(world, wire_pos, wire_st, UPDATE_ALL, events);
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
    events: &mut Vec<CellEvent>,
) {
    for dir in [Direction::South, Direction::West] {
        for i in 1..TRIPWIRE_DIST_MAX {
            let p = step_n(dir, pos, i);
            match world.cell(p).kind() {
                CellState::TripwireHook(hook_st) => {
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
                CellState::TripwireWire(_) => {}
                _ => break,
            }
        }
    }
}

/// `TripWireBlock.checkPressed`: release leaves a delay-0 echo pending.
fn update_tripwire(
    world: &mut impl WorldMut, pos: BlockPos, mut wire: TripwireWireState, occupied: bool,
    events: &mut Vec<CellEvent>,
) {
    if occupied != wire.powered {
        wire.powered = occupied;
        set_wire(world, pos, wire, UPDATE_ALL, events);
        tripwire_update_source(world, pos, wire, events);
    } else if !occupied {
        return;
    }
    let delay = if occupied { TRIPWIRE_RECHECK_PERIOD } else { 0 };
    events.push(CellEvent::ScheduleTick { pos, block_id: wire.block_id, delay });
}

/// `TripWireBlock.entityInside` (`@0-50`).
pub fn tripwire_entity_inside(
    world: &mut impl WorldMut,
    pos: BlockPos,
    is_client_side: bool,
    entity: TriggerEntity,
    events: &mut Vec<CellEvent>,
) {
    if is_client_side {
        return;
    }
    let CellState::TripwireWire(wire_st) = world.cell(pos).kind() else {
        return;
    };
    if wire_st.powered || world.has_scheduled_tick(pos, wire_st.block_id) {
        return;
    }
    update_tripwire(world, pos, wire_st, !entity.is_ignoring_block_triggers, events);
}

/// `TripWireBlock.tick` (`@0-27`) + `TripWireBlock.checkPressed(Level, BlockPos)` (`@0-34`).
pub fn tripwire_tick(
    world: &mut impl WorldMut,
    pos: BlockPos,
    entities: &[TriggerEntity],
    events: &mut Vec<CellEvent>,
) {
    let CellState::TripwireWire(wire_st) = world.cell(pos).kind() else {
        return;
    };
    if !wire_st.powered {
        return;
    }
    let shape = wire_st
        .shape()
        .moved(Vec3::block_corner(pos));
    // Spectators count unless they ignore block triggers.
    let occupied = entities.iter().any(|entity| !entity.is_ignoring_block_triggers && entity.bbox.intersects(shape));
    update_tripwire(world, pos, wire_st, occupied, events);
}

/// `TripWireHookBlock.tick` (`@0-10`).
pub fn tripwire_hook_tick(
    world: &mut impl WorldMut,
    pos: BlockPos,
    events: &mut Vec<CellEvent>,
) {
    let CellState::TripwireHook(hook_st) = world.cell(pos).kind() else {
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
    events: &mut Vec<CellEvent>,
) {
    if is_client_side || !main_hand_is_shears {
        return;
    }
    let CellState::TripwireWire(mut wire_st) = world.cell(pos).kind() else {
        return;
    };
    wire_st.disarmed = true;
    set_wire(world, pos, wire_st, UPDATE_NONE, events);
    events.push(CellEvent::GameEventShear { pos });
}

/// `TripWireBlock.affectNeighborsAfterRemoval` (`@0-25`): phantom `POWERED = true` copy.
pub fn tripwire_after_removal(
    world: &mut impl WorldMut,
    pos: BlockPos,
    old_wire: TripwireWireState,
    moved_by_piston: bool,
    events: &mut Vec<CellEvent>,
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
    events: &mut Vec<CellEvent>,
) {
    if !moved_by_piston {
        hook_on_removed(old_hook, world, pos, events);
    }
}
