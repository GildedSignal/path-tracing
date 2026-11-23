//! Sparse mutable world overlay and unified Mojang 26.2 world-tick simulator.
//!
//! Wires the standalone ported 26.2 subsystems together without duplicating their rules:
//! - [`WorldOverlay`] provides sparse block/fluid overrides over any immutable `&'a W: World`
//!   while recording mutated `BlockPos` footprints (`dirty_positions()`) for certificate
//!   invalidation and repair.
//! - [`SimWorld`] executes [`SERVER_TICK_PHASES`] (`ticks::LevelTicks`, `fluid`, `neighbor`,
//!   `piston`, `falling`, `entity_contact`, `boat`, `pearl`, `place`, `mining`, `interact`,
//!   `world_hooks`, `explosion`, `combat`) against data-driven cell descriptors.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::boat::{self, BoatBody, Controls as BoatControls};
use crate::entity_contact::{ContactEntity, ContactKind, SpatialCrowdIndex, TeamCollisionRule};
use crate::falling::{self, AnvilStage, LandingContext, LandingOutcome};
use crate::fluid::{
    self, FlowKind, FluidCell, LavaContact, SpreadWrite, TargetBlock, TargetCell,
};
use crate::footprint::Footprint;
use crate::interact::{
    self, BlockId, DoorHalf,
    RedstoneEvent,
};
use crate::neighbor::{
    self, CollectingNeighborUpdater, Direction, DrainStats, NeighborQueue, UpdateEvent,
    UpdateOrDestroyAction, UPDATE_SHAPE_ORDER, update_flags,
};
use crate::pearl::{
    self, PearlEntityTarget, PearlLanding, PearlState,
};
use crate::piston::{
    self, BlockKind as PistonBlockKind, MovingPiston, PistonBlock, PistonEntity, PistonWorld,
    Push, PushReaction, Resolver, TriggerAction, set_block_flags, trigger as piston_trigger,
};
use crate::place::{
    self, ScaffoldingCell,
};
use crate::basis::sweep::shapes as collide_with_shapes;
use crate::ticks::{
    ChunkPos, LevelTicks, MAX_TICKS_PER_TICK, ScheduledTick, SubTickCounter, TickPriority,
};
use crate::world::block_pos;
use crate::world_hooks::{
    self, BucketCell, BucketContent, BucketEvent, BucketWorld,
    ButtonEvent, ButtonState, PlateEvent, PressurePlateState, TriggerEntity,
    TripwireEvent,
    WATER_TICK_DELAY,
};
use crate::{
    Aabb, Block, BlockPos, CollisionContext, FluidKind, Inside, Vec3, VoxelShape, World,
};

#[inline]
fn zyx_key(p: BlockPos) -> (i32, i32, i32) {
    (p[2], p[1], p[0])
}

// ============================================================================
// 2. WorldOverlay<'a, W: World>
// ============================================================================

/// Sparse mutable block/fluid overlay over any immutable `&'a W: World`.
/// Tracks mutated `BlockPos` footprints (`dirty_positions()`) for local certificate
/// invalidation and repair while preserving exact `World` z/y/x traversal order.
pub struct WorldOverlay<'a, W: World> {
    base: &'a W,
    overrides: HashMap<BlockPos, Block>,
    dirty: BTreeSet<BlockPos>,
    bounds: Option<(BlockPos, BlockPos)>,
    unbounded: bool,
    extra_fluids: bool,
    extra_inside: bool,
}

impl<'a, W: World> WorldOverlay<'a, W> {
    pub fn new(base: &'a W) -> Self {
        Self {
            base,
            overrides: HashMap::new(),
            dirty: BTreeSet::new(),
            bounds: None,
            unbounded: false,
            extra_fluids: false,
            extra_inside: false,
        }
    }

    #[inline]
    pub fn base(&self) -> &'a W {
        self.base
    }

    /// Inserts or replaces a block override at `pos` and records `pos` as dirty.
    pub fn set_block(&mut self, pos: BlockPos, block: Block) {
        self.unbounded |= !block.collision_within_cell();
        self.extra_fluids |= block.fluid.kind != FluidKind::Empty;
        self.extra_inside |=
            block.inside != Inside::None || block.stuck_multiplier.is_some() || block.fluid.kind != FluidKind::Empty;
        let (lo, hi) = self.bounds.get_or_insert((pos, pos));
        for a in 0..3 {
            lo[a] = lo[a].min(pos[a]);
            hi[a] = hi[a].max(pos[a]);
        }
        self.overrides.insert(pos, block);
        self.dirty.insert(pos);
    }

    /// Deterministic sorted set of mutated `BlockPos` cells since creation / last clear.
    #[inline]
    pub fn dirty_positions(&self) -> &BTreeSet<BlockPos> {
        &self.dirty
    }

    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Inclusive bounding box of all currently dirty positions, if non-empty.
    pub fn dirty_bounds(&self) -> Option<(BlockPos, BlockPos)> {
        let mut iter = self.dirty.iter().copied();
        let first = iter.next()?;
        let mut lo = first;
        let mut hi = first;
        for p in iter {
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        }
        Some((lo, hi))
    }

    /// Conservative check: returns `true` iff any mutated cell touches `footprint`'s read or swept set.
    pub fn invalidates_footprint(&self, footprint: &Footprint) -> bool {
        self.dirty.iter().any(|&pos| footprint.contains(pos))
    }

    fn override_cell_bounds(
        &self,
        bounds: Aabb,
        lo: BlockPos,
        hi: BlockPos,
    ) -> Option<(BlockPos, BlockPos)> {
        let (mut min, mut max) = self.bounds?;
        if !self.unbounded {
            let tight_lo = block_pos(bounds.min);
            let tight_hi = block_pos(bounds.max);
            for a in 0..3 {
                min[a] = min[a].max(tight_lo[a]);
                max[a] = max[a].min(tight_hi[a]);
            }
        }
        for a in 0..3 {
            min[a] = min[a].max(lo[a]);
            max[a] = max[a].min(hi[a]);
        }
        if (0..3).any(|a| min[a] > max[a]) {
            None
        } else {
            Some((min, max))
        }
    }
}

impl<W: World> World for WorldOverlay<'_, W> {
    #[inline]
    fn block(&self, position: BlockPos) -> &Block {
        self.overrides
            .get(&position)
            .unwrap_or_else(|| self.base.block(position))
    }

    #[inline]
    fn model_id(&self, position: BlockPos) -> Option<u16> {
        if self.overrides.contains_key(&position) {
            None
        } else {
            self.base.model_id(position)
        }
    }

    #[inline]
    fn loaded(&self, position: BlockPos) -> bool {
        self.base.loaded(position)
    }

    #[inline]
    fn min_y(&self) -> i32 {
        self.base.min_y()
    }

    #[inline]
    fn fast_lava(&self) -> bool {
        self.base.fast_lava()
    }

    #[inline]
    fn difficulty(&self) -> u8 {
        self.base.difficulty()
    }

    #[inline]
    fn natural_regeneration(&self) -> bool {
        self.base.natural_regeneration()
    }

    #[inline]
    fn has_fluid(&self) -> bool {
        self.extra_fluids || self.base.has_fluid()
    }

    #[inline]
    fn has_inside_effects(&self) -> bool {
        self.extra_inside || self.base.has_inside_effects()
    }

    #[inline]
    fn raw_fluid_block(&self, position: BlockPos) -> Option<&Block> {
        if let Some(ov) = self.overrides.get(&position) {
            self.base.loaded(position).then_some(ov)
        } else {
            self.base.raw_fluid_block(position)
        }
    }

    #[inline]
    fn extra_collisions(&self, bounds: Aabb, context: CollisionContext) -> Vec<VoxelShape> {
        self.base.extra_collisions(bounds, context)
    }

    #[inline]
    fn visit_extra_collisions(
        &self,
        bounds: Aabb,
        context: CollisionContext,
        visit: impl FnMut(crate::shapes::ShapeView<'_>) -> bool,
    ) -> bool {
        self.base.visit_extra_collisions(bounds, context, visit)
    }

    fn collision_cell_bounds(
        &self,
        bounds: Aabb,
        lo: BlockPos,
        hi: BlockPos,
    ) -> Option<(BlockPos, BlockPos)> {
        let base_b = self.base.collision_cell_bounds(bounds, lo, hi);
        let ov_b = self.override_cell_bounds(bounds, lo, hi);
        match (base_b, ov_b) {
            (None, None) => None,
            (Some(b), None) | (None, Some(b)) => Some(b),
            (Some((mut alo, mut ahi)), Some((blo, bhi))) => {
                for a in 0..3 {
                    alo[a] = alo[a].min(blo[a]);
                    ahi[a] = ahi[a].max(bhi[a]);
                }
                Some((alo, ahi))
            }
        }
    }

    fn visit_collision_cells(
        &self,
        bounds: Aabb,
        lo: BlockPos,
        hi: BlockPos,
        mut visit: impl FnMut(BlockPos) -> bool,
    ) -> bool {
        let Some((ov_lo, ov_hi)) = self.override_cell_bounds(bounds, lo, hi) else {
            return self.base.visit_collision_cells(bounds, lo, hi, visit);
        };
        let mut extra: Vec<BlockPos> = self
            .overrides
            .iter()
            .filter_map(|(&p, blk)| {
                let in_box = (0..3).all(|a| p[a] >= ov_lo[a] && p[a] <= ov_hi[a]);
                (in_box && !blk.never_collides()).then_some(p)
            })
            .collect();
        if extra.is_empty() {
            return self.base.visit_collision_cells(bounds, lo, hi, visit);
        }
        extra.sort_unstable_by_key(|&p| zyx_key(p));

        let mut ei = 0usize;
        let mut stopped = false;
        let base_ok = self.base.visit_collision_cells(bounds, lo, hi, |bp| {
            let bkey = zyx_key(bp);
            while ei < extra.len() && zyx_key(extra[ei]) < bkey {
                if !visit(extra[ei]) {
                    stopped = true;
                    return false;
                }
                ei += 1;
            }
            if ei < extra.len() && zyx_key(extra[ei]) == bkey {
                ei += 1;
            }
            if !visit(bp) {
                stopped = true;
                return false;
            }
            true
        });
        if !base_ok || stopped {
            return false;
        }
        while ei < extra.len() {
            if !visit(extra[ei]) {
                return false;
            }
            ei += 1;
        }
        true
    }
}

// ============================================================================
// 3. The cell (`basis::cell`): a block id, the family's state, the table's rows
// ============================================================================

pub use crate::basis::cell::{CellState, FallingKind, Rules as CellProps, SimCell, SurviveRule, WorldMut};

// ============================================================================
// 4. Tick keys, BlockEvents, MovingPistons, and Entity states
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockTickKind {
    FallingBlock,
    Button,
    PressurePlate,
    TripwireWire,
    TripwireHook,
    Scaffolding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FluidTickKind {
    Water,
    Lava,
}

impl From<FlowKind> for FluidTickKind {
    fn from(k: FlowKind) -> Self {
        match k {
            FlowKind::Water => Self::Water,
            FlowKind::Lava { .. } => Self::Lava,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockEvent {
    pub pos: BlockPos,
    pub block_id: BlockId,
    pub event_id: i32,
    pub param: i32,
}

#[derive(Clone, Debug)]
pub struct MovingPistonEntry {
    pub pos: BlockPos,
    pub moving: MovingPiston,
    pub carried: SimCell,
    pub last_ticked: i64,
}

#[derive(Clone, Debug)]
pub struct FallingEntity {
    pub pos: Vec3,
    pub vel: Vec3,
    pub time: i32,
    pub fall_distance: f64,
    pub on_ground: bool,
    pub cell: SimCell,
    pub drop_item: bool,
    pub cancel_drop: bool,
    pub hurt_entities: bool,
    pub fall_damage_per_distance: f32,
    pub fall_damage_max: i32,
    pub removed: bool,
}

impl FallingEntity {
    pub fn aabb(&self) -> Aabb {
        let half = (0.98f32 * 0.5f32) as f64;
        let h = 0.98f32 as f64;
        Aabb::new(
            self.pos.x - half,
            self.pos.y,
            self.pos.z - half,
            self.pos.x + half,
            self.pos.y + h,
            self.pos.z + half,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SimBoat {
    pub body: BoatBody,
    pub controls: BoatControls,
    pub player_controlling: bool,
    pub apply_controls: bool,
    pub server_side: bool,
    pub removed: bool,
}

impl Default for SimBoat {
    fn default() -> Self {
        Self {
            body: BoatBody::default(),
            controls: BoatControls::default(),
            player_controlling: false,
            apply_controls: false,
            server_side: true,
            removed: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SimPearl {
    pub state: PearlState,
    pub owner_idx: Option<usize>,
    pub removed: bool,
}

#[derive(Clone, Debug)]
pub struct SimLivingEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub alive: bool,
    pub spectator: bool,
    pub creative_flying: bool,
    pub flying_ability: bool,
    pub no_physics: bool,
    pub is_player: bool,
    pub is_server_player: bool,
    pub pushable: bool,
    pub on_climbable: bool,
    pub sleeping: bool,
    pub is_passenger: bool,
    pub push_reaction: PushReaction,
    pub team_id: Option<u32>,
    pub team_rule: TeamCollisionRule,
    pub health: f32,
    pub fall_distance: f64,
    pub knockback_resistance: f64,
    pub explosion_knockback_resistance: f64,
    pub supported_by_block: Option<BlockPos>,
    pub apply_gravity: bool,
}

impl Default for SimLivingEntity {
    fn default() -> Self {
        Self {
            position: Vec3::default(),
            velocity: Vec3::default(),
            width: 0.6,
            height: 1.8,
            eye_height: 1.62,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
            alive: true,
            spectator: false,
            creative_flying: false,
            flying_ability: false,
            no_physics: false,
            is_player: false,
            is_server_player: false,
            pushable: true,
            on_climbable: false,
            sleeping: false,
            is_passenger: false,
            push_reaction: PushReaction::Normal,
            team_id: None,
            team_rule: TeamCollisionRule::Always,
            health: 20.0,
            fall_distance: 0.0,
            knockback_resistance: 0.0,
            explosion_knockback_resistance: 0.0,
            supported_by_block: None,
            apply_gravity: true,
        }
    }
}

impl SimLivingEntity {
    pub fn aabb(&self) -> Aabb {
        let half = (self.width as f64) * 0.5;
        Aabb::new(
            self.position.x - half,
            self.position.y,
            self.position.z - half,
            self.position.x + half,
            self.position.y + self.height as f64,
            self.position.z + half,
        )
    }

    pub fn as_contact_entity(&self) -> ContactEntity {
        ContactEntity {
            position: self.position,
            velocity: self.velocity,
            aabb: self.aabb(),
            no_physics: self.no_physics,
            vehicle: false,
            pushable: self.pushable,
            kind: ContactKind::Living,
            is_passenger: self.is_passenger,
            alive: self.alive,
            spectator: self.spectator,
            on_climbable: self.on_climbable,
            sleeping: self.sleeping,
            team_id: self.team_id,
            team_rule: self.team_rule,
        }
    }

    pub fn as_trigger_entity(&self) -> TriggerEntity {
        TriggerEntity {
            bbox: self.aabb(),
            is_living: true,
            is_spectator: self.spectator,
            is_ignoring_block_triggers: false,
        }
    }

}

/// Summary returned by [`SimWorld::step_tick`] for one full Mojang 26.2 server tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldTick {
    pub game_time: i64,
    pub block_ticks_ran: usize,
    pub fluid_ticks_ran: usize,
    pub block_events_ran: usize,
    pub block_entities_ticked: usize,
    pub falling_blocks_ticked: usize,
    pub boats_ticked: usize,
    pub pearls_ticked: usize,
    pub crowd_cram_damage_events: usize,
    pub neighbor_stats: DrainStats,
}

// ============================================================================
// 5. SimWorld<'a, W: World>
// ============================================================================

pub struct SimWorld<'a, W: World> {
    overlay: WorldOverlay<'a, W>,
    cells: HashMap<BlockPos, SimCell>,
    pub game_time: i64,
    pub min_y: i32,
    pub max_y: i32,
    pub fast_lava: bool,
    pub lava_source_conversion: bool,
    pub entity_drops: bool,
    pub max_entity_cramming: i32,
    pub water_evaporates: bool,
    handling_tick: bool,
    sub_tick_counter: SubTickCounter,
    pub block_ticks: LevelTicks<BlockTickKind>,
    pub fluid_ticks: LevelTicks<FluidTickKind>,
    neighbor_updater: CollectingNeighborUpdater<BlockId>,
    block_events: VecDeque<BlockEvent>,
    block_events_set: HashSet<BlockEvent>,
    moving_pistons: HashMap<BlockPos, MovingPistonEntry>,
    moving_piston_order: Vec<BlockPos>,
    pub falling_blocks: Vec<FallingEntity>,
    pub boats: Vec<SimBoat>,
    pub pearls: Vec<SimPearl>,
    pub living_entities: Vec<SimLivingEntity>,
    pub pressure_plate_positions: BTreeSet<BlockPos>,
    pub tripwire_positions: BTreeSet<BlockPos>,
    crowd_index: SpatialCrowdIndex,
    piston_push_scratch: Vec<Push>,
    block_tick_scratch: Vec<ScheduledTick<BlockTickKind>>,
    fluid_tick_scratch: Vec<ScheduledTick<FluidTickKind>>,
    rng_state: u64,
}

impl<'a, W: World> SimWorld<'a, W> {
    pub fn new(base: &'a W) -> Self {
        Self {
            min_y: base.min_y(),
            fast_lava: base.fast_lava(),
            overlay: WorldOverlay::new(base),
            cells: HashMap::new(),
            game_time: 0,
            max_y: 319,
            lava_source_conversion: false,
            entity_drops: true,
            max_entity_cramming: crate::entity_contact::CRAMMING_DEFAULT_MAX,
            water_evaporates: false,
            handling_tick: false,
            sub_tick_counter: SubTickCounter::default(),
            block_ticks: LevelTicks::new(),
            fluid_ticks: LevelTicks::new(),
            neighbor_updater: CollectingNeighborUpdater::new(1_000_000),
            block_events: VecDeque::new(),
            block_events_set: HashSet::new(),
            moving_pistons: HashMap::new(),
            moving_piston_order: Vec::new(),
            falling_blocks: Vec::new(),
            boats: Vec::new(),
            pearls: Vec::new(),
            living_entities: Vec::new(),
            pressure_plate_positions: BTreeSet::new(),
            tripwire_positions: BTreeSet::new(),
            crowd_index: SpatialCrowdIndex::new(),
            piston_push_scratch: Vec::new(),
            block_tick_scratch: Vec::new(),
            fluid_tick_scratch: Vec::new(),
            rng_state: 0x517c_c1b7_2722_0a95,
        }
    }


    #[inline]
    pub fn world(&self) -> &WorldOverlay<'a, W> {
        &self.overlay
    }

    #[inline]
    pub fn dirty_positions(&self) -> &BTreeSet<BlockPos> {
        self.overlay.dirty_positions()
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 { 1 } else { seed };
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    fn next_int_4(&mut self) -> i32 {
        (self.next_u64() >> 62) as i32 & 3
    }

    fn next_float(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u32 << 24) as f32)
    }

    /// Reads the rich simulation cell at `pos`, inferring from the base `World` when unedited.
    pub fn cell(&self, pos: BlockPos) -> SimCell {
        if let Some(c) = self.cells.get(&pos) {
            return c.clone();
        }
        let b = self.overlay.base().block(pos);
        if b.air {
            return SimCell::air(BlockId::of("air"));
        }
        if b.fluid.kind == FluidKind::Water {
            let amt = b.fluid.raw_amount.unwrap_or(8);
            let falling = b.fluid.raw_falling.unwrap_or(false);
            return SimCell::water(
                BlockId::of("water"),
                FluidCell {
                    amount: amt,
                    falling,
                    source: amt == 8 && !falling,
                },
            );
        }
        if b.fluid.kind == FluidKind::Lava {
            let amt = b.fluid.raw_amount.unwrap_or(8);
            let falling = b.fluid.raw_falling.unwrap_or(false);
            return SimCell::lava(
                BlockId::of("lava"),
                self.fast_lava,
                FluidCell {
                    amount: amt,
                    falling,
                    source: amt == 8 && !falling,
                },
            );
        }
        if b.slime {
            return SimCell::plain(BlockId::of("slime_block"));
        }
        if b.inside == Inside::Honey {
            return SimCell::plain(BlockId::of("honey_block"));
        }
        let blocks_motion = b
            .fluid_flow
            .map_or(!b.shape.is_empty(), |ff| ff.blocks_motion);
        let full = b.shape.is_block();
        let sturdy = b
            .fluid_flow
            .map_or([full; 6], |ff| {
                [
                    full,
                    full,
                    ff.solid_faces[0],
                    ff.solid_faces[2],
                    ff.solid_faces[3],
                    ff.solid_faces[1],
                ]
            });
        SimCell {
            id: BlockId::of("stone"),
            state: CellState::Plain,
            props: CellProps {
                destroy_speed: 1.5,
                requires_correct_tool: true,
                is_redstone_conductor: full,
                sturdy_faces: sturdy,
                push_reaction: PushReaction::Normal,
                piston_kind: PistonBlockKind::Other,
                blocks_motion,
                replaceable_by_fluid: true,
                can_be_replaced: false,
                is_signal_source: false,
                has_block_entity: false,
                survive_rule: SurviveRule::Always,
            },
            block: b.clone(),
        }
    }

    pub fn moving_piston(&self, pos: BlockPos) -> Option<&MovingPistonEntry> {
        self.moving_pistons.get(&pos)
    }

    /// Direct setup write without triggering neighbor/shape updates (for building fixtures).
    pub fn set_cell_raw(&mut self, pos: BlockPos, mut cell: SimCell) {
        cell.sync_block();
        self.update_hook_tracking(pos, &cell);
        self.overlay.set_block(pos, cell.block.clone());
        self.cells.insert(pos, cell);
    }

    fn update_hook_tracking(&mut self, pos: BlockPos, cell: &SimCell) {
        match cell.state {
            CellState::PressurePlate(_) => {
                self.pressure_plate_positions.insert(pos);
                self.tripwire_positions.remove(&pos);
            }
            CellState::TripwireWire(_) => {
                self.tripwire_positions.insert(pos);
                self.pressure_plate_positions.remove(&pos);
            }
            _ => {
                self.pressure_plate_positions.remove(&pos);
                self.tripwire_positions.remove(&pos);
            }
        }
    }

    #[inline]
    pub fn in_world_bounds(&self, pos: BlockPos) -> bool {
        pos[1] >= self.min_y && pos[1] <= self.max_y
    }

    pub fn schedule_block_tick(&mut self, pos: BlockPos, kind: BlockTickKind, delay: u32) -> bool {
        self.schedule_block_tick_prio(pos, kind, delay, TickPriority::Normal)
    }

    pub fn schedule_block_tick_prio(
        &mut self,
        pos: BlockPos,
        kind: BlockTickKind,
        delay: u32,
        priority: TickPriority,
    ) -> bool {
        let order = self.sub_tick_counter.next();
        self.block_ticks.schedule(ScheduledTick {
            kind,
            pos,
            trigger_tick: self.game_time + delay as i64,
            priority,
            sub_tick_order: order,
        })
    }

    pub fn schedule_fluid_tick(&mut self, pos: BlockPos, kind: FluidTickKind, delay: u32) -> bool {
        let order = self.sub_tick_counter.next();
        self.fluid_ticks.schedule(ScheduledTick {
            kind,
            pos,
            trigger_tick: self.game_time + delay as i64,
            priority: TickPriority::Normal,
            sub_tick_order: order,
        })
    }

    pub fn queue_block_event(&mut self, pos: BlockPos, block_id: BlockId, event_id: i32, param: i32) {
        let ev = BlockEvent {
            pos,
            block_id,
            event_id,
            param,
        };
        if self.block_events_set.insert(ev) {
            self.block_events.push_back(ev);
        }
    }

    // ------------------------------------------------------------------------
    // Block mutation with Mojang `Level.setBlock` flags & neighbor cascades
    // ------------------------------------------------------------------------

    pub fn set_cell_with_flags(&mut self, pos: BlockPos, cell: SimCell, flags: i32) -> DrainStats {
        self.set_cell_with_flags_limit(pos, cell, flags, update_flags::DEFAULT_LIMIT)
    }

    pub fn set_cell_with_flags_limit(
        &mut self,
        pos: BlockPos,
        cell: SimCell,
        flags: i32,
        recursion_limit: i32,
    ) -> DrainStats {
        let mut updater = std::mem::replace(
            &mut self.neighbor_updater,
            CollectingNeighborUpdater::new(0),
        );
        let did_enqueue =
            self.apply_set_cell(Some(updater.queue_mut()), pos, cell, flags, recursion_limit);
        let stats = updater.drain_with(
            |_| {},
            |q, event| {
                self.handle_neighbor_event(q, event);
            },
        );
        self.neighbor_updater = updater;
        if !did_enqueue {
            DrainStats::default()
        } else {
            stats
        }
    }

    fn apply_set_cell(
        &mut self,
        mut queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
        mut new_cell: SimCell,
        flags: i32,
        recursion_limit: i32,
    ) -> bool {
        if !self.in_world_bounds(pos) {
            return false;
        }
        new_cell.sync_block();
        let old_cell = self.cell(pos);
        if old_cell.id == new_cell.id && old_cell.state == new_cell.state {
            return false;
        }

        self.update_hook_tracking(pos, &new_cell);
        self.overlay.set_block(pos, new_cell.block.clone());
        self.cells.insert(pos, new_cell.clone());

        let moved_by_piston = (flags & update_flags::MOVE_BY_PISTON) != 0;
        if old_cell.id != new_cell.id {
            self.on_remove(queue.as_deref_mut(), pos, &old_cell, moved_by_piston);
        }
        if update_flags::triggers_on_place(flags) {
            self.on_place(queue.as_deref_mut(), pos, &new_cell, &old_cell, moved_by_piston);
        }

        let mut enqueued = false;
        if update_flags::triggers_neighbor_updates(flags) {
            if let Some(q) = queue.as_deref_mut() {
                q.update_neighbors_except(pos, old_cell.id, None);
                enqueued = true;
            }
        }
        if update_flags::triggers_shape_updates(flags, recursion_limit) {
            let prop_flags = update_flags::propagated_shape_flags(flags);
            if let Some(q) = queue {
                for dir in UPDATE_SHAPE_ORDER {
                    let neighbor_pos = dir.relative_by(pos, 1);
                    let neighbor = self.cell(neighbor_pos);
                    if update_flags::skip_shape_update_on_wire(
                        prop_flags,
                        matches!(neighbor.state, CellState::RedstoneWire { .. }),
                    ) {
                        continue;
                    }
                    q.shape_update(
                        dir.opposite(),
                        new_cell.id,
                        neighbor_pos,
                        pos,
                        prop_flags,
                        recursion_limit - 1,
                    );
                    enqueued = true;
                }
            }
        }
        enqueued
    }

    pub fn destroy_block(&mut self, pos: BlockPos, _drop_resources: bool) -> bool {
        let old = self.cell(pos);
        if old.is_air() {
            return false;
        }
        let replacement = match old.fluid_info() {
            Some((FlowKind::Water, _)) => {
                SimCell::water(BlockId::of("water"), FluidCell::WATER_SOURCE)
            }
            _ => SimCell::air(BlockId::of("air")),
        };
        self.set_cell_with_flags(pos, replacement, update_flags::ALL);
        true
    }

    fn on_remove(
        &mut self,
        mut queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
        old_cell: &SimCell,
        moved_by_piston: bool,
    ) {
        match old_cell.state {
            CellState::Button(btn) => {
                let mut evs = Vec::new();
                world_hooks::button_after_removal(btn, pos, moved_by_piston, &mut evs);
                for ev in evs {
                    if let ButtonEvent::UpdateNeighborsAt { pos: p, source_id } = ev {
                        self.enqueue_update_neighbors(queue.as_deref_mut(), p, source_id, None);
                    }
                }
            }
            CellState::PressurePlate(plate) => {
                let mut evs = Vec::new();
                world_hooks::pressure_plate_after_removal(plate, pos, moved_by_piston, &mut evs);
                for ev in evs {
                    if let PlateEvent::UpdateNeighborsAt { pos: p, source_id } = ev {
                        self.enqueue_update_neighbors(queue.as_deref_mut(), p, source_id, None);
                    }
                }
            }
            CellState::TripwireWire(wire) => {
                let mut tw = Pending::new(self);
                let mut evs = Vec::new();
                world_hooks::tripwire_after_removal(&mut tw, pos, wire, moved_by_piston, &mut evs);
                self.apply_tripwire_events(queue.as_deref_mut(), evs);
            }
            CellState::TripwireHook(hook) => {
                let mut tw = Pending::new(self);
                let mut evs = Vec::new();
                world_hooks::tripwire_hook_after_removal(
                    &mut tw,
                    pos,
                    hook,
                    moved_by_piston,
                    &mut evs,
                );
                self.apply_tripwire_events(queue.as_deref_mut(), evs);
            }
            CellState::Lever(lever) if !moved_by_piston && lever.powered => {
                self.enqueue_update_neighbors(queue.as_deref_mut(), pos, lever.block_id, None);
                self.enqueue_update_neighbors(
                    queue,
                    lever.support_pos(pos),
                    lever.block_id,
                    None,
                );
            }
            CellState::RedstoneWire { .. } if !moved_by_piston => {
                for dir in Direction::ALL {
                    self.enqueue_update_neighbors(
                        queue.as_deref_mut(),
                        dir.relative_by(pos, 1),
                        old_cell.id,
                        None,
                    );
                }
                self.update_wire_neighbors(queue, pos, old_cell.id);
            }
            CellState::MovingPiston { .. } => {
                self.moving_pistons.remove(&pos);
            }
            _ => {}
        }
    }

    fn on_place(
        &mut self,
        mut queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
        new_cell: &SimCell,
        old_cell: &SimCell,
        _moved_by_piston: bool,
    ) {
        match new_cell.state {
            CellState::Fluid { kind, cell } => {
                self.handle_liquid_block_place_or_neighbor(queue, pos, kind, cell);
            }
            CellState::Falling(_) => {
                self.schedule_block_tick(pos, BlockTickKind::FallingBlock, 2);
            }
            CellState::RedstoneWire { .. } if old_cell.id != new_cell.id => {
                self.update_redstone_wire(queue.as_deref_mut(), pos);
                for dir in [Direction::Up, Direction::Down] {
                    self.enqueue_update_neighbors(
                        queue.as_deref_mut(),
                        dir.relative_by(pos, 1),
                        new_cell.id,
                        None,
                    );
                }
                self.update_wire_neighbors(queue, pos, new_cell.id);
            }
            CellState::Piston {
                sticky,
                facing,
                extended,
            } if old_cell.id != new_cell.id => {
                self.piston_check_if_extend(pos, new_cell.id, sticky, facing, extended);
            }
            _ => {}
        }
    }

    fn enqueue_update_neighbors(
        &mut self,
        queue: Option<&mut NeighborQueue<BlockId>>,
        center: BlockPos,
        source_id: BlockId,
        skip: Option<Direction>,
    ) {
        if let Some(q) = queue {
            q.update_neighbors_except(center, source_id, skip);
        } else {
            let mut updater = std::mem::replace(
                &mut self.neighbor_updater,
                CollectingNeighborUpdater::new(0),
            );
            updater.drain_with(
                |q| {
                    q.update_neighbors_except(center, source_id, skip);
                },
                |q, ev| {
                    self.handle_neighbor_event(q, ev);
                },
            );
            self.neighbor_updater = updater;
        }
    }

    fn update_wire_neighbors(
        &mut self,
        mut queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
        wire_id: BlockId,
    ) {
        for dir in Direction::HORIZONTAL {
            let side = dir.relative_by(pos, 1);
            self.enqueue_update_neighbors(queue.as_deref_mut(), side, wire_id, None);
            let target = if self.cell(side).props.is_redstone_conductor {
                Direction::Up.relative_by(side, 1)
            } else {
                Direction::Down.relative_by(side, 1)
            };
            self.enqueue_update_neighbors(queue.as_deref_mut(), target, wire_id, None);
        }
    }

    fn handle_neighbor_event(
        &mut self,
        queue: &mut NeighborQueue<BlockId>,
        event: UpdateEvent<BlockId>,
    ) {
        match event {
            UpdateEvent::Simple {
                pos,
                source_block,
                moved_by_piston: _,
            } => {
                let cell = self.cell(pos);
                match cell.state {
                    CellState::RedstoneWire { .. } => {
                        if !self.can_survive(pos, &cell) {
                            let air = SimCell::air(BlockId::of("air"));
                            self.apply_set_cell(
                                Some(queue),
                                pos,
                                air,
                                update_flags::ALL,
                                update_flags::DEFAULT_LIMIT,
                            );
                        } else {
                            self.update_redstone_wire(Some(queue), pos);
                        }
                    }
                    CellState::Door(mut door) => {
                        if source_block == door.block_id {
                            return;
                        }
                        let other_pos = match door.half {
                            DoorHalf::Lower => Direction::Up.relative_by(pos, 1),
                            DoorHalf::Upper => Direction::Down.relative_by(pos, 1),
                        };
                        let sig = self.has_neighbor_signal(pos) || self.has_neighbor_signal(other_pos);
                        if let Some((new_powered, flags)) =
                            neighbor::powered_toggle_on_signal(door.powered, sig)
                        {
                            door.powered = new_powered;
                            door.open = new_powered;
                            let updated = SimCell::new(door.block_id, CellState::Door(door));
                            self.apply_set_cell(
                                Some(queue),
                                pos,
                                updated,
                                flags,
                                update_flags::DEFAULT_LIMIT,
                            );
                        }
                    }
                    CellState::TrapDoor {
                        facing,
                        half,
                        open: _,
                        powered,
                        waterlogged,
                    } => {
                        let sig = self.has_neighbor_signal(pos);
                        if let Some((new_powered, flags)) =
                            neighbor::powered_toggle_on_signal(powered, sig)
                        {
                            let updated = SimCell::new(cell.id, CellState::TrapDoor { facing, half, open: new_powered, powered: new_powered, waterlogged });
                            self.apply_set_cell(
                                Some(queue),
                                pos,
                                updated,
                                flags,
                                update_flags::DEFAULT_LIMIT,
                            );
                            if waterlogged {
                                self.schedule_fluid_tick(
                                    pos,
                                    FluidTickKind::Water,
                                    interact::WATERLOGGED_TRAPDOOR_TICK_DELAY,
                                );
                            }
                        }
                    }
                    CellState::FenceGate {
                        facing,
                        open: _,
                        powered,
                    } => {
                        let sig = self.has_neighbor_signal(pos);
                        if let Some((new_powered, flags)) =
                            neighbor::powered_toggle_on_signal(powered, sig)
                        {
                            let updated =
                                SimCell::new(cell.id, CellState::FenceGate { facing, open: new_powered, powered: new_powered });
                            self.apply_set_cell(
                                Some(queue),
                                pos,
                                updated,
                                flags,
                                update_flags::DEFAULT_LIMIT,
                            );
                        }
                    }
                    CellState::Piston {
                        sticky,
                        facing,
                        extended,
                    } => {
                        self.piston_check_if_extend(pos, cell.id, sticky, facing, extended);
                    }
                    CellState::Fluid { kind, cell: fcell } => {
                        self.handle_liquid_block_place_or_neighbor(
                            Some(queue),
                            pos,
                            kind,
                            fcell,
                        );
                    }
                    _ => {}
                }
            }
            UpdateEvent::Shape {
                pos,
                direction,
                neighbor_pos,
                neighbor_state: _,
                flags,
                recursion_left,
            } => {
                if recursion_left < 0 {
                    return;
                }
                let cell = self.cell(pos);
                match cell.state {
                    CellState::Door(mut door) => {
                        let other_dir = match door.half {
                            DoorHalf::Lower => Direction::Up,
                            DoorHalf::Upper => Direction::Down,
                        };
                        if direction == Direction::Down && door.half == DoorHalf::Lower {
                            if !self.cell(neighbor_pos).props.sturdy_faces[Direction::Up as usize] {
                                let action = neighbor::decide_update_or_destroy(
                                    true,
                                    true,
                                    flags,
                                    recursion_left,
                                );
                                self.execute_update_or_destroy(Some(queue), pos, action, None);
                            }
                        } else if direction == other_dir {
                            let neighbor = self.cell(neighbor_pos);
                            if let CellState::Door(other) = neighbor.state {
                                if other.half != door.half {
                                    door.facing = other.facing;
                                    door.hinge = other.hinge;
                                    door.open = other.open;
                                    door.powered = other.powered;
                                    let next = SimCell::new(door.block_id, CellState::Door(door));
                                    let changed = next.state != cell.state;
                                    let action = neighbor::decide_update_or_destroy(
                                        changed,
                                        false,
                                        flags,
                                        recursion_left,
                                    );
                                    self.execute_update_or_destroy(
                                        Some(queue),
                                        pos,
                                        action,
                                        Some(next),
                                    );
                                    return;
                                }
                            }
                            let action = neighbor::decide_update_or_destroy(
                                true,
                                true,
                                flags,
                                recursion_left,
                            );
                            self.execute_update_or_destroy(Some(queue), pos, action, None);
                        }
                    }
                    CellState::Fluid { kind, cell: fcell } => {
                        let nf_source = self
                            .cell(neighbor_pos)
                            .fluid_info()
                            .is_some_and(|(_, nf)| nf.source);
                        if fcell.source || nf_source {
                            self.schedule_fluid_tick(
                                pos,
                                FluidTickKind::from(kind),
                                kind.tick_delay() as u32,
                            );
                        }
                    }
                    CellState::Falling(_) => {
                        self.schedule_block_tick(pos, BlockTickKind::FallingBlock, 2);
                    }
                    CellState::Scaffolding(_) => {
                        self.schedule_block_tick(
                            pos,
                            BlockTickKind::Scaffolding,
                            place::SCAFFOLDING_TICK_DELAY,
                        );
                    }
                    _ => {
                        if cell.props.survive_rule != SurviveRule::Always && !self.can_survive(pos, &cell)
                        {
                            let action = neighbor::decide_update_or_destroy(
                                true,
                                true,
                                flags,
                                recursion_left,
                            );
                            self.execute_update_or_destroy(Some(queue), pos, action, None);
                        } else if matches!(
                            cell.fluid_info(),
                            Some((FlowKind::Water, FluidCell { source: true, .. }))
                        ) {
                            self.schedule_fluid_tick(pos, FluidTickKind::Water, WATER_TICK_DELAY);
                        }
                    }
                }
            }
        }
    }

    fn execute_update_or_destroy(
        &mut self,
        queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
        action: UpdateOrDestroyAction,
        replacement: Option<SimCell>,
    ) {
        match action {
            UpdateOrDestroyAction::NoOp => {}
            UpdateOrDestroyAction::Destroy {
                drop_resources: _,
                recursion_left,
            } => {
                let air = SimCell::air(BlockId::of("air"));
                self.apply_set_cell(queue, pos, air, update_flags::ALL, recursion_left);
            }
            UpdateOrDestroyAction::SetBlock {
                flags,
                recursion_left,
            } => {
                if let Some(cell) = replacement {
                    self.apply_set_cell(queue, pos, cell, flags, recursion_left);
                }
            }
        }
    }

    fn can_survive(&self, pos: BlockPos, cell: &SimCell) -> bool {
        match cell.props.survive_rule {
            SurviveRule::Always => true,
            SurviveRule::FloorSturdy => {
                let below = self.cell(Direction::Down.relative_by(pos, 1));
                below.props.sturdy_faces[Direction::Up as usize]
            }
            SurviveRule::Attached => match cell.state {
                CellState::Lever(l) => {
                    let attach = l.connected_direction().opposite();
                    let support = attach.relative_by(pos, 1);
                    self.cell(support).props.sturdy_faces[attach.opposite() as usize]
                }
                CellState::Button(b) => {
                    let attach = b.connected_direction().opposite();
                    let support = attach.relative_by(pos, 1);
                    self.cell(support).props.sturdy_faces[attach.opposite() as usize]
                }
                CellState::TripwireHook(h) => {
                    let attach = h.facing.opposite();
                    let support = attach.relative_by(pos, 1);
                    self.cell(support).props.sturdy_faces[(h.facing) as usize]
                }
                _ => true,
            },
        }
    }

    // ------------------------------------------------------------------------
    // Redstone signals & wire evaluation (reusing interact.rs & neighbor.rs)
    // ------------------------------------------------------------------------

    fn wire_has_connections(&self, pos: BlockPos) -> [bool; 4] {
        let above_conductor = self
            .cell(Direction::Up.relative_by(pos, 1))
            .props
            .is_redstone_conductor;
        let mut conn = [false; 4];
        for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
            let side = dir.relative_by(pos, 1);
            let sc = self.cell(side);
            let flat = matches!(sc.state, CellState::RedstoneWire { .. }) || sc.props.is_signal_source;
            if flat {
                conn[i] = true;
            } else if sc.props.is_redstone_conductor {
                if !above_conductor {
                    let up = Direction::Up.relative_by(side, 1);
                    conn[i] = matches!(self.cell(up).state, CellState::RedstoneWire { .. });
                }
            } else {
                let down = Direction::Down.relative_by(side, 1);
                conn[i] = matches!(self.cell(down).state, CellState::RedstoneWire { .. });
            }
        }
        // Straight-line and cross connection rules (`RedStoneWireBlock.getConnectionState`):
        let [n, e, s, w] = conn;
        let ns_empty = !n && !s;
        let ew_empty = !e && !w;
        [
            n || ew_empty,
            e || ns_empty,
            s || ew_empty,
            w || ns_empty,
        ]
    }

    pub fn get_signal(&self, pos: BlockPos, dir: Direction, wire_should_signal: bool) -> u8 {
        let cell = self.cell(pos);
        let own = match cell.state {
            CellState::RedstoneWire { power } => {
                if !wire_should_signal || dir == Direction::Down {
                    0
                } else if dir == Direction::Up {
                    power
                } else {
                    let conn = self.wire_has_connections(pos);
                    let idx = match dir.opposite() {
                        Direction::North => 0,
                        Direction::East => 1,
                        Direction::South => 2,
                        Direction::West => 3,
                        _ => unreachable!(),
                    };
                    if conn[idx] { power } else { 0 }
                }
            }
            CellState::Lever(l) => l.signal(),
            CellState::Button(b) => b.own_signal(),
            CellState::PressurePlate(p) => p.own_signal(),
            CellState::TripwireHook(h) => h.own_signal(),
            _ => 0,
        };
        if cell.props.is_redstone_conductor {
            own.max(self.get_direct_signal_to(pos, wire_should_signal))
        } else {
            own
        }
    }

    pub fn get_direct_signal(&self, pos: BlockPos, dir: Direction, wire_should_signal: bool) -> u8 {
        let cell = self.cell(pos);
        match cell.state {
            CellState::RedstoneWire { .. } => {
                if wire_should_signal {
                    self.get_signal(pos, dir, true)
                } else {
                    0
                }
            }
            CellState::Lever(l) => l.direct_signal(dir),
            CellState::Button(b) => b.direct_signal(dir),
            CellState::PressurePlate(p) => p.direct_signal(dir),
            CellState::TripwireHook(h) => h.direct_signal(dir),
            _ => 0,
        }
    }

    pub fn get_direct_signal_to(&self, pos: BlockPos, wire_should_signal: bool) -> u8 {
        let mut best = 0u8;
        for dir in Direction::ALL {
            let sig = self.get_direct_signal(dir.relative_by(pos, 1), dir, wire_should_signal);
            if sig >= 15 {
                return 15;
            }
            if sig > best {
                best = sig;
            }
        }
        best
    }

    pub fn get_best_neighbor_signal(&self, pos: BlockPos, wire_should_signal: bool) -> u8 {
        let mut best = 0u8;
        for dir in Direction::ALL {
            let sig = self.get_signal(dir.relative_by(pos, 1), dir, wire_should_signal);
            if sig >= 15 {
                return 15;
            }
            if sig > best {
                best = sig;
            }
        }
        best
    }

    pub fn has_neighbor_signal(&self, pos: BlockPos) -> bool {
        for dir in Direction::ALL {
            if self.get_signal(dir.relative_by(pos, 1), dir, true) > 0 {
                return true;
            }
        }
        false
    }

    fn piston_has_neighbor_signal(&self, pos: BlockPos, facing: Direction) -> bool {
        for dir in Direction::ALL {
            if dir != facing && self.get_signal(dir.relative_by(pos, 1), dir, true) > 0 {
                return true;
            }
        }
        if self.get_signal(pos, Direction::Down, true) > 0 {
            return true;
        }
        let above = Direction::Up.relative_by(pos, 1);
        for dir in Direction::ALL {
            if dir != Direction::Down && self.get_signal(dir.relative_by(above, 1), dir, true) > 0 {
                return true;
            }
        }
        false
    }

    fn update_redstone_wire(
        &mut self,
        mut queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
    ) {
        let CellState::RedstoneWire { power } = self.cell(pos).state else {
            return;
        };
        let block_sig = self.get_best_neighbor_signal(pos, false);
        let above_conductor = self
            .cell(Direction::Up.relative_by(pos, 1))
            .props
            .is_redstone_conductor;
        let mut incoming = [0u8; 4];
        for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
            let side = dir.relative_by(pos, 1);
            let side_cell = self.cell(side);
            let flat = match side_cell.state {
                CellState::RedstoneWire { power: p } => Some(p),
                _ => None,
            };
            let up_wire = match self.cell(Direction::Up.relative_by(side, 1)).state {
                CellState::RedstoneWire { power: p } => Some(p),
                _ => None,
            };
            let down_wire = match self.cell(Direction::Down.relative_by(side, 1)).state {
                CellState::RedstoneWire { power: p } => Some(p),
                _ => None,
            };
            incoming[i] = neighbor::redstone_wire_side_signal(
                flat,
                side_cell.props.is_redstone_conductor,
                above_conductor,
                up_wire,
                down_wire,
            );
        }
        let target = neighbor::redstone_wire_target_strength(block_sig, incoming);
        if target != power {
            let wire_id = self.cell(pos).id;
            let next = SimCell::new(wire_id, CellState::RedstoneWire { power: target });
            self.apply_set_cell(
                queue.as_deref_mut(),
                pos,
                next,
                update_flags::CLIENTS,
                update_flags::DEFAULT_LIMIT,
            );
            let mut fan = Vec::with_capacity(7);
            fan.push(pos);
            for dir in Direction::ALL {
                let np = dir.relative_by(pos, 1);
                if !fan.contains(&np) {
                    fan.push(np);
                }
            }
            for target_pos in fan {
                self.enqueue_update_neighbors(queue.as_deref_mut(), target_pos, wire_id, None);
            }
        }
    }

    // ------------------------------------------------------------------------
    // Liquid block & FlowingFluid spread (reusing fluid.rs)
    // ------------------------------------------------------------------------

    fn handle_liquid_block_place_or_neighbor(
        &mut self,
        queue: Option<&mut NeighborQueue<BlockId>>,
        pos: BlockPos,
        kind: FlowKind,
        cell: FluidCell,
    ) {
        let is_lava = matches!(kind, FlowKind::Lava { .. });
        let mut water_above_or_beside = false;
        let mut blue_ice_above_or_beside = false;
        if is_lava {
            for dir in [
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ] {
                let nc = self.cell(dir.relative_by(pos, 1));
                if nc.fluid_info().is_some_and(|(k, _)| matches!(k, FlowKind::Water)) {
                    water_above_or_beside = true;
                }
                if nc.id == BlockId::of("blue_ice") {
                    blue_ice_above_or_beside = true;
                }
            }
        }
        let soul_soil_below = self.cell(Direction::Down.relative_by(pos, 1)).id == BlockId::of("soul_soil");
        let (contact, should_sched) = fluid::should_spread_liquid(
            is_lava,
            cell.source,
            water_above_or_beside,
            soul_soil_below,
            blue_ice_above_or_beside,
        );
        match contact {
            LavaContact::Obsidian => {
                let obs = SimCell::plain(BlockId::of("obsidian"));
                self.apply_set_cell(
                    queue,
                    pos,
                    obs,
                    update_flags::ALL,
                    update_flags::DEFAULT_LIMIT,
                );
            }
            LavaContact::Cobblestone => {
                let cobble = SimCell::plain(BlockId::of("cobblestone"));
                self.apply_set_cell(
                    queue,
                    pos,
                    cobble,
                    update_flags::ALL,
                    update_flags::DEFAULT_LIMIT,
                );
            }
            LavaContact::Basalt => {
                let basalt = SimCell::plain(BlockId::of("basalt"));
                self.apply_set_cell(
                    queue,
                    pos,
                    basalt,
                    update_flags::ALL,
                    update_flags::DEFAULT_LIMIT,
                );
            }
            LavaContact::None => {}
        }
        if should_sched {
            self.schedule_fluid_tick(pos, FluidTickKind::from(kind), kind.tick_delay() as u32);
        }
    }

    fn fluid_can_convert_to_source(&self, kind: FlowKind) -> bool {
        match kind {
            FlowKind::Water => true,
            FlowKind::Lava { .. } => self.lava_source_conversion,
        }
    }

    fn same_fluid_kind(a: FlowKind, b: FlowKind) -> bool {
        matches!(
            (a, b),
            (FlowKind::Water, FlowKind::Water) | (FlowKind::Lava { .. }, FlowKind::Lava { .. })
        )
    }

    fn can_pass_through_wall(&self, _dir: Direction, from: BlockPos, to: BlockPos) -> bool {
        let a = self.cell(from);
        let b = self.cell(to);
        !a.props.blocks_motion && !b.props.blocks_motion
            || (a.is_waterlogged_container() || b.is_waterlogged_container())
                && !b.block.shape.is_block()
    }

    fn can_hold_any_fluid(cell: &SimCell) -> bool {
        if cell.is_air() || cell.props.replaceable_by_fluid || cell.is_waterlogged_container() {
            return true;
        }
        !cell.props.blocks_motion
    }

    fn can_hold_specific_fluid(cell: &SimCell, kind: FlowKind) -> bool {
        if cell.is_waterlogged_container() {
            matches!(kind, FlowKind::Water)
        } else {
            Self::can_hold_any_fluid(cell)
        }
    }

    fn fluid_can_be_replaced_with(
        existing: Option<(FlowKind, FluidCell)>,
        incoming: FlowKind,
        dir: Direction,
    ) -> bool {
        let Some((ex_kind, ex_cell)) = existing else {
            return true;
        };
        match ex_kind {
            FlowKind::Water => fluid::water_replaced_by(dir, matches!(incoming, FlowKind::Water)),
            FlowKind::Lava { .. } => {
                if matches!(incoming, FlowKind::Water) {
                    fluid::water_replaces_lava(ex_cell.own_height())
                } else {
                    false
                }
            }
        }
    }

    fn compute_new_liquid(&self, pos: BlockPos, kind: FlowKind) -> FluidCell {
        let mut max_neighbor = 0u8;
        let mut source_neighbors = 0u8;
        for dir in Direction::HORIZONTAL {
            let np = dir.relative_by(pos, 1);
            let nc = self.cell(np);
            if let Some((nk, nfluid)) = nc.fluid_info() {
                if Self::same_fluid_kind(kind, nk) && self.can_pass_through_wall(dir, pos, np) {
                    if nfluid.source {
                        source_neighbors += 1;
                    }
                    max_neighbor = max_neighbor.max(nfluid.amount);
                }
            }
        }
        let above = Direction::Up.relative_by(pos, 1);
        let fed_from_above = self
            .cell(above)
            .fluid_info()
            .is_some_and(|(ak, _)| Self::same_fluid_kind(kind, ak))
            && self.can_pass_through_wall(Direction::Up, pos, above);

        let below_pos = Direction::Down.relative_by(pos, 1);
        let below_cell = self.cell(below_pos);
        let below_solid_or_same_source = !below_cell.is_air() && below_cell.props.blocks_motion
            || below_cell
                .fluid_info()
                .is_some_and(|(bk, bf)| Self::same_fluid_kind(kind, bk) && bf.source);

        fluid::get_new_liquid(
            kind,
            max_neighbor,
            source_neighbors,
            fed_from_above,
            self.fluid_can_convert_to_source(kind),
            below_solid_or_same_source,
        )
    }

    fn source_neighbor_count(&self, pos: BlockPos, kind: FlowKind) -> u8 {
        let mut count = 0u8;
        for dir in Direction::HORIZONTAL {
            let np = dir.relative_by(pos, 1);
            if let Some((nk, nf)) = self.cell(np).fluid_info() {
                if Self::same_fluid_kind(kind, nk) && nf.source {
                    count += 1;
                }
            }
        }
        count
    }

    fn fluid_cell_from_kind(&self, kind: FlowKind, fcell: FluidCell) -> SimCell {
        match kind {
            FlowKind::Water => SimCell::water(BlockId::of("water"), fcell),
            FlowKind::Lava { .. } => SimCell::lava(BlockId::of("lava"), self.fast_lava, fcell),
        }
    }

    fn spread_fluid_to(&mut self, pos: BlockPos, dir: Direction, kind: FlowKind, new_fluid: FluidCell) {
        let target_cell = self.cell(pos);
        let existing_fluid = target_cell.fluid_info();
        let target_block = if target_cell.is_air() {
            TargetBlock::Air
        } else if target_cell.is_waterlogged_container() {
            TargetBlock::Container
        } else if target_cell.props.replaceable_by_fluid {
            TargetBlock::Destroyable
        } else {
            TargetBlock::Solid
        };
        let tcell = TargetCell {
            block: target_block,
            fluid_is_water: existing_fluid.is_some_and(|(k, _)| matches!(k, FlowKind::Water)),
            block_is_liquid: matches!(target_cell.state, CellState::Fluid { .. }),
            new_legacy_level: new_fluid.legacy_level(),
        };
        let is_lava = matches!(kind, FlowKind::Lava { .. });
        match fluid::spread_write(is_lava, dir, tcell) {
            SpreadWrite::Stone => {
                let stone = SimCell::plain(BlockId::of("stone"));
                self.set_cell_with_flags(pos, stone, update_flags::ALL);
            }
            SpreadWrite::FizzOnly => {}
            SpreadWrite::PlaceLiquid => {
                if fluid::can_waterlog_container(matches!(kind, FlowKind::Water) && new_fluid.source) {
                    let mut next = target_cell;
                    if next.set_waterlogged(true) {
                        self.set_cell_with_flags(pos, next, update_flags::ALL);
                        self.schedule_fluid_tick(pos, FluidTickKind::Water, WATER_TICK_DELAY);
                    }
                }
            }
            SpreadWrite::ReplaceFluid { .. } => {
                let next = self.fluid_cell_from_kind(kind, new_fluid);
                self.set_cell_with_flags(pos, next, update_flags::ALL);
            }
        }
    }

    fn run_fluid_tick(&mut self, pos: BlockPos, tick_kind: FluidTickKind) {
        let current = self.cell(pos);
        let Some((kind, mut fcell)) = current.fluid_info() else {
            return;
        };
        if FluidTickKind::from(kind) != tick_kind {
            return;
        }
        if !fcell.source {
            let new_liquid = self.compute_new_liquid(pos, kind);
            let r4 = self.next_int_4();
            let delay = kind.spread_delay(fcell, new_liquid, r4) as u32;
            if new_liquid.amount == 0 {
                fcell = FluidCell::EMPTY;
                let air = SimCell::air(BlockId::of("air"));
                self.set_cell_with_flags(pos, air, update_flags::ALL);
            } else if new_liquid != fcell {
                fcell = new_liquid;
                let next = self.fluid_cell_from_kind(kind, fcell);
                self.set_cell_with_flags(pos, next, update_flags::ALL);
                self.schedule_fluid_tick(pos, FluidTickKind::from(kind), delay);
            }
        }
        if fcell.amount == 0 {
            return;
        }

        // `FlowingFluid.spread`: check downward first
        let below_pos = Direction::Down.relative_by(pos, 1);
        let below_cell = self.cell(below_pos);
        let below_fluid = below_cell.fluid_info();
        let can_maybe_down = !below_fluid
            .is_some_and(|(bk, bf)| Self::same_fluid_kind(kind, bk) && bf.source)
            && Self::can_hold_any_fluid(&below_cell)
            && self.can_pass_through_wall(Direction::Down, pos, below_pos);

        let mut down_spread = false;
        if can_maybe_down {
            let below_new = self.compute_new_liquid(below_pos, kind);
            if Self::fluid_can_be_replaced_with(below_fluid, kind, Direction::Down)
                && Self::can_hold_specific_fluid(&below_cell, kind)
            {
                self.spread_fluid_to(below_pos, Direction::Down, kind, below_new);
                down_spread = true;
            }
        }

        let down_is_hole = self.can_pass_through_wall(Direction::Down, pos, below_pos)
            && (below_fluid.is_some_and(|(bk, _)| Self::same_fluid_kind(kind, bk))
                || Self::can_hold_specific_fluid(&below_cell, kind));
        let src_neighbors = self.source_neighbor_count(pos, kind);

        if fluid::spreads_to_sides(fcell.source, down_spread, down_is_hole, src_neighbors) {
            if fluid::side_amount(fcell, kind).is_none() {
                return;
            }
            let mut side_targets = [FluidCell::EMPTY; 4];
            let mut side_eligible = [false; 4];
            let mut side_replaceable = [false; 4];
            for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
                let np = dir.relative_by(pos, 1);
                let nc = self.cell(np);
                let nf = nc.fluid_info();
                let maybe = !nf
                    .is_some_and(|(nk, nfl)| Self::same_fluid_kind(kind, nk) && nfl.source)
                    && Self::can_hold_any_fluid(&nc)
                    && self.can_pass_through_wall(dir, pos, np);
                if maybe {
                    let cand = self.compute_new_liquid(np, kind);
                    if cand.amount > 0 && Self::can_hold_specific_fluid(&nc, kind) {
                        side_eligible[i] = true;
                        side_targets[i] = cand;
                        side_replaceable[i] = Self::fluid_can_be_replaced_with(nf, kind, dir);
                    }
                }
            }

            let y = pos[1];
            let (mask, _min_dist) = fluid::compute_spread_directions(
                kind,
                (pos[0], pos[2]),
                |x, z| {
                    if x == pos[0] && z == pos[2] {
                        return true;
                    }
                    let c = self.cell([x, y, z]);
                    let f = c.fluid_info();
                    !f.is_some_and(|(fk, fl)| Self::same_fluid_kind(kind, fk) && fl.source)
                        && Self::can_hold_specific_fluid(&c, kind)
                },
                |x, z| {
                    let here = [x, y, z];
                    let below = [x, y - 1, z];
                    let bc = self.cell(below);
                    let bf = bc.fluid_info();
                    self.can_pass_through_wall(Direction::Down, here, below)
                        && (bf.is_some_and(|(fk, _)| Self::same_fluid_kind(kind, fk))
                            || Self::can_hold_specific_fluid(&bc, kind))
                },
                |dir| {
                    let idx = match dir {
                        Direction::North => 0,
                        Direction::East => 1,
                        Direction::South => 2,
                        Direction::West => 3,
                        _ => return false,
                    };
                    side_eligible[idx] && side_replaceable[idx]
                },
            );

            for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
                if (mask & (1 << i)) != 0 {
                    let np = dir.relative_by(pos, 1);
                    self.spread_fluid_to(np, dir, kind, side_targets[i]);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Pistons & MovingPistonBlockEntity (reusing piston.rs)
    // ------------------------------------------------------------------------

    fn piston_check_if_extend(
        &mut self,
        pos: BlockPos,
        block_id: BlockId,
        sticky: bool,
        facing: Direction,
        extended: bool,
    ) {
        let has_signal = self.piston_has_neighbor_signal(pos, facing);
        if has_signal && !extended {
            let pworld = SimPistonView { sim: self };
            let mut resolver = Resolver::new(pos, facing, true);
            if resolver.resolve(&pworld) {
                self.queue_block_event(pos, block_id, piston_trigger::EXTEND, facing as i32);
            }
        } else if !has_signal && extended {
            let front2 = facing.relative_by(pos, 2);
            let mut event = piston_trigger::CONTRACT;
            if let Some(entry) = self.moving_pistons.get(&front2) {
                if entry.moving.direction == facing
                    && entry.moving.extending
                    && (entry.moving.progress < 0.5
                        || self.game_time == entry.last_ticked
                        || self.handling_tick)
                {
                    event = piston_trigger::DROP;
                }
            }
            let _ = sticky;
            self.queue_block_event(pos, block_id, event, facing as i32);
        }
    }

    fn run_block_events(&mut self) -> usize {
        let mut ran = 0usize;
        while let Some(ev) = self.block_events.pop_front() {
            self.block_events_set.remove(&ev);
            let cell = self.cell(ev.pos);
            if cell.id != ev.block_id {
                continue;
            }
            if let CellState::Piston {
                sticky,
                facing: _,
                extended: _,
            } = cell.state
            {
                let dir = Direction::from_3d_data(ev.param as u8);
                let head_pos = dir.relative_by(ev.pos, 1);
                let front2_pos = dir.relative_by(ev.pos, 2);
                let front_moving_piston =
                    matches!(self.cell(head_pos).state, CellState::MovingPiston { .. });
                let front2_moving_piston_extending_same_direction =
                    self.moving_pistons.get(&front2_pos).is_some_and(|e| {
                        e.moving.extending && e.moving.direction == dir
                    });
                let front2_block = self.cell(front2_pos).as_piston_block();
                let view = piston::TriggerView {
                    neighbor_signal: self.piston_has_neighbor_signal(ev.pos, dir),
                    front_moving_piston,
                    front2_moving_piston_extending_same_direction,
                    front2_block,
                };
                if let Some(actions) =
                    piston::trigger_event(sticky, dir, ev.pos, ev.event_id, view)
                {
                    let mut ok = true;
                    for &action in actions.as_slice() {
                        match action {
                            TriggerAction::MoveBlocks { extending } => {
                                if !self.execute_piston_move(ev.pos, cell.id, sticky, dir, extending) {
                                    ok = false;
                                    break;
                                }
                            }
                            TriggerAction::SetBlock { flags } => {
                                let new_extended = ev.event_id == piston_trigger::EXTEND;
                                if flags == set_block_flags::RETRACT_BODY_TO_MOVING {
                                    let carried =
                                        SimCell::new(cell.id, CellState::Piston { sticky, facing: dir, extended: false });
                                    let mp = SimCell::new(BlockId::of("moving_piston"), CellState::MovingPiston { sticky, facing: dir });
                                    self.set_cell_with_flags(ev.pos, mp, flags);
                                    self.insert_moving_piston(
                                        ev.pos,
                                        MovingPiston::new(dir, false, true),
                                        carried,
                                    );
                                } else {
                                    let next =
                                        SimCell::new(cell.id, CellState::Piston { sticky, facing: dir, extended: new_extended });
                                    self.set_cell_with_flags(ev.pos, next, flags);
                                }
                            }
                            TriggerAction::StickyPull => {
                                self.execute_piston_move(ev.pos, cell.id, sticky, dir, false);
                            }
                            TriggerAction::RemoveHead => {
                                let air = SimCell::air(BlockId::of("air"));
                                self.set_cell_with_flags(
                                    head_pos,
                                    air,
                                    set_block_flags::RETRACT_HEAD_TO_AIR,
                                );
                            }
                            TriggerAction::FinalTick { position } => {
                                self.final_tick_moving_piston(position);
                            }
                        }
                    }
                    if ok {
                        ran += 1;
                    }
                }
            }
        }
        ran
    }

    fn insert_moving_piston(&mut self, pos: BlockPos, moving: MovingPiston, carried: SimCell) {
        if !self.moving_pistons.contains_key(&pos) {
            self.moving_piston_order.push(pos);
        }
        self.moving_pistons.insert(
            pos,
            MovingPistonEntry {
                pos,
                moving,
                carried,
                last_ticked: self.game_time,
            },
        );
    }

    fn execute_piston_move(
        &mut self,
        piston_pos: BlockPos,
        _piston_id: BlockId,
        sticky: bool,
        facing: Direction,
        extending: bool,
    ) -> bool {
        let head_pos = facing.relative_by(piston_pos, 1);
        if !extending && matches!(self.cell(head_pos).state, CellState::PistonHead { .. }) {
            let air = SimCell::air(BlockId::of("air"));
            self.set_cell_with_flags(head_pos, air, set_block_flags::RETRACT_HEAD_TO_AIR);
        }

        let mut resolver = Resolver::new(piston_pos, facing, extending);
        {
            let pworld = SimPistonView { sim: self };
            if !resolver.resolve(&pworld) {
                return false;
            }
        }
        let to_push: Vec<BlockPos> = resolver.to_push().to_vec();
        let to_destroy: Vec<BlockPos> = resolver.to_destroy().to_vec();
        let push_dir = resolver.push_direction();

        let mut map: HashMap<BlockPos, SimCell> = HashMap::with_capacity(to_push.len());
        let mut orig_push_cells = Vec::with_capacity(to_push.len());
        for &p in &to_push {
            let c = self.cell(p);
            orig_push_cells.push(c.clone());
            map.insert(p, c);
        }

        let mut update_positions = Vec::with_capacity(to_push.len() + to_destroy.len());
        let mut update_old_cells = Vec::with_capacity(to_push.len() + to_destroy.len());

        // Destroy frontier blocks first (reverse order)
        for &dpos in to_destroy.iter().rev() {
            let old = self.cell(dpos);
            let air = SimCell::air(BlockId::of("air"));
            self.set_cell_with_flags(dpos, air, set_block_flags::DESTROYED_TO_AIR);
            update_positions.push(dpos);
            update_old_cells.push(old);
        }

        // Move pushed blocks in reverse order into MovingPiston destinations
        for (idx, &src_pos) in to_push.iter().enumerate().rev() {
            let old_state = orig_push_cells[idx].clone();
            let dst_pos = push_dir.relative_by(src_pos, 1);
            map.remove(&dst_pos);
            let mp = SimCell::new(BlockId::of("moving_piston"), CellState::MovingPiston { sticky, facing });
            self.set_cell_with_flags(dst_pos, mp, set_block_flags::PUSHED_TO_MOVING);
            self.insert_moving_piston(
                dst_pos,
                MovingPiston::new(facing, extending, false),
                old_state.clone(),
            );
            update_positions.push(dst_pos);
            update_old_cells.push(old_state);
        }

        if extending {
            map.remove(&head_pos);
            let head_carried = SimCell::new(BlockId::of("piston_head"), CellState::PistonHead { sticky, facing });
            let mp = SimCell::new(BlockId::of("moving_piston"), CellState::MovingPiston { sticky, facing });
            self.set_cell_with_flags(head_pos, mp, set_block_flags::PUSHED_TO_MOVING);
            self.insert_moving_piston(
                head_pos,
                MovingPiston::new(facing, true, true),
                head_carried,
            );
        }

        // Leftover source positions become Air
        let air = SimCell::air(BlockId::of("air"));
        for (&leftover, _) in &map {
            self.set_cell_with_flags(leftover, air.clone(), set_block_flags::LEFTOVER_TO_AIR);
        }

        for (p, old) in update_positions.iter().zip(update_old_cells.iter()) {
            self.enqueue_update_neighbors(None, *p, old.id, None);
        }
        if extending {
            self.enqueue_update_neighbors(
                None,
                head_pos,
                BlockId::of("piston_head"),
                None,
            );
        }
        true
    }

    fn final_tick_moving_piston(&mut self, pos: BlockPos) {
        let Some(entry) = self.moving_pistons.remove(&pos) else {
            return;
        };
        if matches!(self.cell(pos).state, CellState::MovingPiston { .. }) {
            let restored = entry.carried;
            self.set_cell_with_flags(pos, restored, set_block_flags::FINAL_RESTORE);
        }
    }

    fn tick_block_entities(&mut self) -> usize {
        let order = std::mem::take(&mut self.moving_piston_order);
        let mut next_order = Vec::with_capacity(order.len());
        let mut ticked = 0usize;

        for pos in order {
            let Some(mut entry) = self.moving_pistons.remove(&pos) else {
                continue;
            };
            ticked += 1;
            entry.last_ticked = self.game_time;
            if entry.moving.progress_o < 1.0 {
                let new_progress = entry.moving.next_progress();
                self.apply_moving_piston_entity_motion(&entry, new_progress);
            }
            let finished = entry.moving.advance(false);
            if finished {
                if matches!(self.cell(pos).state, CellState::MovingPiston { .. }) {
                    let mut restored = entry.carried;
                    if restored.is_waterlogged_container() {
                        restored.set_waterlogged(false);
                    }
                    let id = restored.id;
                    self.set_cell_with_flags(pos, restored, set_block_flags::FINAL_RESTORE);
                    self.enqueue_update_neighbors(None, pos, id, None);
                }
            } else {
                next_order.push(pos);
                self.moving_pistons.insert(pos, entry);
            }
        }
        self.moving_piston_order = next_order;
        ticked
    }

    fn extract_shape_boxes(shape: &VoxelShape) -> ([Aabb; 2], usize) {
        let mut boxes = [Aabb::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0); 2];
        let mut nboxes = 0usize;
        let nx = shape.coords[0].len().saturating_sub(1);
        let ny = shape.coords[1].len().saturating_sub(1);
        let nz = shape.coords[2].len().saturating_sub(1);
        for x in 0..nx {
            for y in 0..ny {
                for z in 0..nz {
                    if shape.is_full([x, y, z]) && nboxes < boxes.len() {
                        boxes[nboxes] = Aabb::new(
                            shape.coords[0][x],
                            shape.coords[1][y],
                            shape.coords[2][z],
                            shape.coords[0][x + 1],
                            shape.coords[1][y + 1],
                            shape.coords[2][z + 1],
                        );
                        nboxes += 1;
                    }
                }
            }
        }
        (boxes, nboxes)
    }

    fn moving_piston_boxes(entry: &MovingPistonEntry) -> ([Aabb; 2], usize) {
        if entry.moving.is_source_piston {
            (piston_head_boxes(entry.moving.direction), 2)
        } else {
            Self::extract_shape_boxes(&entry.carried.block.shape)
        }
    }

    fn collect_world_collisions(&self, bounds: Aabb, ctx: CollisionContext) -> Vec<VoxelShape> {
        let mut shapes = self.overlay.collisions(bounds, ctx);
        for entry in self.moving_pistons.values() {
            if !entry.moving.extending
                && entry.moving.is_source_piston
                && matches!(
                    entry.carried.props.piston_kind,
                    PistonBlockKind::Piston { .. } | PistonBlockKind::StickyPiston { .. }
                )
            {
                let base = crate::basis::cell::model(BlockId::of("piston"), &[("extended", "true"), ("facing", crate::basis::cell::DIRECTION[entry.moving.direction as usize])]);
                let c = &base.shape.coords;
                let base_box = Aabb::new(c[0][0], c[1][0], c[2][0], c[0][1], c[1][1], c[2][1]).moved(Vec3::new(
                    entry.pos[0] as f64,
                    entry.pos[1] as f64,
                    entry.pos[2] as f64,
                ));
                if base_box.intersects(bounds) {
                    shapes.push(VoxelShape::boxes(&[base_box]));
                }
            }
            let (local_boxes, nboxes) = Self::moving_piston_boxes(entry);
            for &lb in &local_boxes[..nboxes] {
                let shifted = piston::box_at(
                    entry.pos,
                    lb,
                    entry.moving.extending,
                    entry.moving.direction,
                    entry.moving.progress,
                );
                if shifted.intersects(bounds) {
                    shapes.push(VoxelShape::boxes(&[shifted]));
                }
            }
        }
        shapes
    }

    fn apply_moving_piston_entity_motion(
        &mut self,
        entry: &MovingPistonEntry,
        new_progress: f32,
    ) {
        let (boxes, nboxes) = Self::moving_piston_boxes(entry);
        if nboxes == 0 {
            return;
        }
        let slime_moved = entry.carried.props.piston_kind == PistonBlockKind::SlimeBlock;
        let honey_moved = entry.carried.props.piston_kind == PistonBlockKind::HoneyBlock;
        let move_dir = piston::movement_direction(entry.moving.extending, entry.moving.direction);
        let delta = (new_progress - entry.moving.progress) as f64;

        let mut query_opt: Option<Aabb> = None;
        for &b in &boxes[..nboxes] {
            let moved_bounds = piston::box_at(
                entry.pos,
                b,
                entry.moving.extending,
                entry.moving.direction,
                entry.moving.progress,
            );
            let area = piston::movement_area(moved_bounds, move_dir, delta);
            let combined = Aabb::new(
                moved_bounds.min.x.min(area.min.x),
                moved_bounds.min.y.min(area.min.y),
                moved_bounds.min.z.min(area.min.z),
                moved_bounds.max.x.max(area.max.x),
                moved_bounds.max.y.max(area.max.y),
                moved_bounds.max.z.max(area.max.z),
            );
            query_opt = Some(match query_opt {
                None => combined,
                Some(q) => Aabb::new(
                    q.min.x.min(combined.min.x),
                    q.min.y.min(combined.min.y),
                    q.min.z.min(combined.min.z),
                    q.max.x.max(combined.max.x),
                    q.max.y.max(combined.max.y),
                    q.max.z.max(combined.max.z),
                ),
            });
        }
        let Some(query_box) = query_opt else {
            return;
        };

        let mut filtered_indices = Vec::new();
        let mut p_ents = Vec::new();
        for (idx, e) in self.living_entities.iter().enumerate() {
            let bb = e.aabb();
            if bb.intersects(query_box) {
                filtered_indices.push(idx);
                p_ents.push(PistonEntity {
                    aabb: bb,
                    position: e.position,
                    delta_movement: e.velocity,
                    push_reaction: e.push_reaction,
                    on_ground: e.on_ground,
                    server_player: e.is_server_player,
                    supported_by_block: e.supported_by_block == Some(entry.pos),
                });
            }
        }
        if !p_ents.is_empty() {
            let mut pushes = std::mem::take(&mut self.piston_push_scratch);
            piston::move_collided_entities_into(
                entry.pos,
                &boxes[..nboxes],
                &mut p_ents,
                entry.moving.extending,
                entry.moving.direction,
                entry.moving.is_source_piston,
                entry.moving.progress,
                new_progress,
                slime_moved,
                &mut pushes,
            );
            for push in &pushes {
                let real_idx = filtered_indices[push.entity];
                let e = &mut self.living_entities[real_idx];
                if let Some(vel) = push.velocity_snap {
                    e.velocity = vel;
                }
                if push.distance > 0.0 {
                    let s = push.direction.step();
                    e.position = e.position.plus(Vec3::new(
                        s[0] as f64 * push.distance,
                        s[1] as f64 * push.distance,
                        s[2] as f64 * push.distance,
                    ));
                }
            }
            self.piston_push_scratch = pushes;
        }

        // Honey block sticky drag (`moveStuckEntities`)
        if honey_moved && move_dir.is_horizontal() {
            let max_y = boxes[..nboxes]
                .iter()
                .map(|b| b.max.y)
                .fold(0.0f64, f64::max);
            let stuck_box = piston::box_at(
                entry.pos,
                Aabb::new(0.0, max_y, 0.0, 1.0, piston::STUCK_BOX_HEIGHT, 1.0),
                entry.moving.extending,
                entry.moving.direction,
                entry.moving.progress,
            );
            let s = move_dir.step();
            for e in &mut self.living_entities {
                let pe = PistonEntity {
                    aabb: e.aabb(),
                    position: e.position,
                    delta_movement: e.velocity,
                    push_reaction: e.push_reaction,
                    on_ground: e.on_ground,
                    server_player: e.is_server_player,
                    supported_by_block: e.supported_by_block == Some(entry.pos),
                };
                if piston::stuck_entity(true, true, stuck_box, &pe) {
                    e.position = e.position.plus(Vec3::new(
                        s[0] as f64 * delta,
                        s[1] as f64 * delta,
                        s[2] as f64 * delta,
                    ));
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Falling blocks (reusing falling.rs)
    // ------------------------------------------------------------------------

    pub fn spawn_falling_block(&mut self, pos: BlockPos, cell: SimCell) -> usize {
        let (x, y, z) = falling::spawn(pos[0], pos[1], pos[2]);
        let (hurt, per_dist, max_dmg) = match cell.state {
            CellState::Falling(FallingKind::Anvil { .. }) => (
                true,
                falling::ANVIL_PER_DISTANCE,
                falling::ANVIL_MAX_DAMAGE,
            ),
            _ => (false, 0.0, 0),
        };
        let idx = self.falling_blocks.len();
        self.falling_blocks.push(FallingEntity {
            pos: Vec3::new(x, y, z),
            vel: Vec3::default(),
            time: 0,
            fall_distance: 0.0,
            on_ground: false,
            cell,
            drop_item: true,
            cancel_drop: false,
            hurt_entities: hurt,
            fall_damage_per_distance: per_dist,
            fall_damage_max: max_dmg,
            removed: false,
        });
        idx
    }

    fn run_block_tick(&mut self, pos: BlockPos, kind: BlockTickKind) {
        let cell = self.cell(pos);
        match kind {
            BlockTickKind::FallingBlock => {
                if !matches!(cell.state, CellState::Falling(_)) {
                    return;
                }
                let below = self.cell(Direction::Down.relative_by(pos, 1));
                let below_is_free = falling::is_free(
                    below.is_air(),
                    false,
                    matches!(below.state, CellState::Fluid { .. }),
                    below.props.can_be_replaced,
                );
                if below_is_free && pos[1] >= self.min_y {
                    self.spawn_falling_block(pos, cell);
                    let air = SimCell::air(BlockId::of("air"));
                    self.set_cell_with_flags(pos, air, update_flags::ALL);
                }
            }
            BlockTickKind::Button => {
                if let CellState::Button(mut btn) = cell.state {
                    let mut evs = Vec::new();
                    world_hooks::button_tick(&mut btn, pos, &[], &mut evs);
                    self.apply_button_events(btn, &evs);
                }
            }
            BlockTickKind::PressurePlate => {
                if let CellState::PressurePlate(mut plate) = cell.state {
                    let triggers: Vec<TriggerEntity> = self
                        .living_entities
                        .iter()
                        .filter(|e| e.alive)
                        .map(SimLivingEntity::as_trigger_entity)
                        .collect();
                    let mut evs = Vec::new();
                    world_hooks::pressure_plate_tick(&mut plate, pos, &triggers, &mut evs);
                    self.apply_plate_events(plate, &evs);
                }
            }
            BlockTickKind::TripwireWire => {
                let triggers: Vec<TriggerEntity> = self
                    .living_entities
                    .iter()
                    .filter(|e| e.alive)
                    .map(SimLivingEntity::as_trigger_entity)
                    .collect();
                let mut tw =
                    Pending::new(self);
                let mut evs = Vec::new();
                world_hooks::tripwire_tick(&mut tw, pos, &triggers, &mut evs);
                self.apply_tripwire_events(None, evs);
            }
            BlockTickKind::TripwireHook => {
                let mut tw =
                    Pending::new(self);
                let mut evs = Vec::new();
                world_hooks::tripwire_hook_tick(&mut tw, pos, &mut evs);
                self.apply_tripwire_events(None, evs);
            }
            BlockTickKind::Scaffolding => {
                if let CellState::Scaffolding(mut sc) = cell.state {
                    let dist = place::scaffolding_get_distance(pos, |p| self.scaffolding_cell(p));
                    let below_scaf = matches!(
                        self.cell(Direction::Down.relative_by(pos, 1)).state,
                        CellState::Scaffolding(_)
                    );
                    let bottom = place::scaffolding_is_bottom(dist, below_scaf);
                    if !place::scaffolding_can_survive(dist) {
                        self.spawn_falling_block(pos, cell);
                        let air = SimCell::air(BlockId::of("air"));
                        self.set_cell_with_flags(pos, air, update_flags::ALL);
                    } else if sc.distance != dist || sc.bottom != bottom {
                        sc.distance = dist;
                        sc.bottom = bottom;
                        let next = SimCell::new(cell.id, CellState::Scaffolding(sc));
                        self.set_cell_with_flags(pos, next, update_flags::ALL);
                    }
                }
            }
        }
    }

    fn tick_falling_blocks(&mut self) -> usize {
        let mut ticked = 0usize;
        let len = self.falling_blocks.len();
        for i in 0..len {
            if self.falling_blocks[i].removed {
                continue;
            }
            ticked += 1;
            let mut fb = self.falling_blocks[i].clone();
            if fb.cell.is_air() {
                fb.removed = true;
                self.falling_blocks[i] = fb;
                continue;
            }
            fb.time += 1;
            crate::basis::body::FALLING_BLOCK.gravity(&mut fb.vel, false);

            let bounds = fb.aabb();
            let wanted = fb.vel;
            let moved = if wanted.length_sqr() == 0.0 {
                wanted
            } else {
                let ctx = CollisionContext {
                    feet_y: bounds.min.y,
                    descending: false,
                    fall_distance: fb.fall_distance,
                    leather_boots: false,
                };
                let shapes = self.collect_world_collisions(bounds.expand(wanted), ctx);
                collide_with_shapes(wanted, bounds, &shapes)
            };
            fb.on_ground = crate::basis::body::settle(&mut fb.pos, &mut fb.vel, moved).below();

            let bpos = block_pos(fb.pos);
            let is_concrete_powder =
                matches!(fb.cell.state, CellState::Falling(FallingKind::ConcretePowder));
            let in_water = self
                .cell(bpos)
                .fluid_info()
                .is_some_and(|(k, _)| matches!(k, FlowKind::Water));

            // Entity.checkFallDamage (`(float) moved.y` cast before subtraction!)
            if !in_water && moved.y < 0.0 {
                fb.fall_distance -= (moved.y as f32) as f64;
            }
            if fb.on_ground {
                if fb.fall_distance > 0.0 && fb.hurt_entities {
                    let cells = falling::fall_distance_cells(fb.fall_distance);
                    let dmg = falling::fall_damage(cells, fb.fall_damage_per_distance, fb.fall_damage_max);
                    if dmg > 0.0 {
                        let fbb = fb.aabb();
                        for ent in &mut self.living_entities {
                            if ent.alive && !ent.spectator && ent.aabb().intersects(fbb) {
                                ent.health = (ent.health - dmg).max(0.0);
                                if ent.health == 0.0 {
                                    ent.alive = false;
                                }
                            }
                        }
                        if let CellState::Falling(FallingKind::Anvil {
                            facing_x_axis,
                            stage,
                        }) = fb.cell.state
                        {
                            let r = self.next_float();
                            if falling::anvil_degrades(cells, r) {
                                let next_stage = stage.damage();
                                if next_stage == AnvilStage::Broken {
                                    fb.cancel_drop = true;
                                } else {
                                    let next_id = match next_stage {
                                        AnvilStage::Intact => BlockId::of("anvil"),
                                        AnvilStage::Chipped => BlockId::of("chipped_anvil"),
                                        AnvilStage::Damaged => BlockId::of("damaged_anvil"),
                                        AnvilStage::Broken => BlockId::of("air"),
                                    };
                                    fb.cell = SimCell::new(next_id, CellState::Falling(FallingKind::Anvil { facing_x_axis, stage: next_stage }));
                                }
                            }
                        }
                    }
                }
                fb.fall_distance = 0.0;
            }

            let solidify_powder = if is_concrete_powder {
                let mut neigh_water = [false; 6];
                let dirs = [
                    Direction::Down,
                    Direction::Up,
                    Direction::North,
                    Direction::South,
                    Direction::West,
                    Direction::East,
                ];
                for (di, d) in dirs.into_iter().enumerate() {
                    let nc = self.cell(d.relative_by(bpos, 1));
                    let has_w = nc
                        .fluid_info()
                        .is_some_and(|(k, _)| matches!(k, FlowKind::Water));
                    neigh_water[di] = has_w && !nc.props.sturdy_faces[d.opposite() as usize];
                }
                falling::concrete_powder_should_solidify(in_water, neigh_water)
            } else {
                false
            };

            let landing_cell = self.cell(bpos);
            let below_cell = self.cell(Direction::Down.relative_by(bpos, 1));
            let below_is_free = falling::is_free(
                below_cell.is_air(),
                false,
                matches!(below_cell.state, CellState::Fluid { .. }),
                below_cell.props.can_be_replaced,
            );
            let ctx = LandingContext {
                on_ground: fb.on_ground,
                in_water,
                landing_is_moving_piston: matches!(
                    landing_cell.state,
                    CellState::MovingPiston { .. }
                ),
                cancel_drop: fb.cancel_drop,
                landing_can_be_replaced: landing_cell.is_air() || landing_cell.props.can_be_replaced,
                carried_can_survive: self.can_survive(bpos, &fb.cell),
                below_is_free,
                is_concrete_powder,
                carried_has_waterlogged: fb.cell.is_waterlogged_container(),
                landing_fluid_is_water: landing_cell
                    .fluid_info()
                    .is_some_and(|(k, _)| matches!(k, FlowKind::Water)),
                drop_item: fb.drop_item,
                entity_drops_gamerule: self.entity_drops,
                set_block_succeeded: self.in_world_bounds(bpos),
            };
            if fb.on_ground || in_water {
                fb.vel = falling::damp_landing_velocity(fb.vel);
            }
            match falling::resolve_landing(ctx) {
                LandingOutcome::Placed { flags, waterlogged } => {
                    let mut placed = if solidify_powder {
                        SimCell::plain(BlockId::of("stone"))
                    } else {
                        fb.cell.clone()
                    };
                    if waterlogged && !solidify_powder {
                        placed.set_waterlogged(true);
                    }
                    self.set_cell_with_flags(bpos, placed, flags);
                    fb.removed = true;
                }
                LandingOutcome::DiscardCancelledNoDrop
                | LandingOutcome::PreconditionRefuse { .. }
                | LandingOutcome::SetBlockRejectedDiscard { .. } => {
                    fb.removed = true;
                }
                LandingOutcome::Airborne
                | LandingOutcome::SurviveMovingPiston
                | LandingOutcome::SetBlockRejectedRetry => {
                    if falling::discards(fb.time, bpos[1], self.min_y, self.max_y) {
                        fb.removed = true;
                    } else {
                        crate::basis::body::FALLING_BLOCK.drag(&mut fb.vel, false);
                    }
                }
            }
            self.falling_blocks[i] = fb;
        }
        ticked
    }

    // ------------------------------------------------------------------------
    // Boats, Pearls, Living Entities, Crowd Push, Explosion, Combat, Player
    // ------------------------------------------------------------------------

    pub fn spawn_boat(&mut self, position: Vec3, yaw: f32) -> usize {
        let mut b = SimBoat::default();
        b.body.boat.position = position;
        b.body.boat.yaw = yaw;
        let idx = self.boats.len();
        self.boats.push(b);
        idx
    }

    pub fn spawn_living(&mut self, entity: SimLivingEntity) -> usize {
        let idx = self.living_entities.len();
        self.living_entities.push(entity);
        idx
    }

    fn tick_boats(&mut self) -> usize {
        let mut ticked = 0usize;
        let query = &self.overlay;
        for boat in &mut self.boats {
            if boat.removed {
                continue;
            }
            ticked += 1;
            boat::tick_body(
                &mut boat.body,
                query,
                boat.controls,
                boat.server_side,
                boat.player_controlling,
                boat.apply_controls,
            );
            // `AbstractBoat.controlBoat` calls `delta.add(dx, 0.0, dz)`, turning `-0.0 + 0.0` -> `+0.0`.
            if boat.apply_controls && boat.body.boat.velocity.y == 0.0 {
                boat.body.boat.velocity.y += 0.0;
            }
        }
        ticked
    }

    fn tick_pearls(&mut self) -> usize {
        let mut ticked = 0usize;
        let targets: Vec<PearlEntityTarget> = self
            .living_entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.alive && !e.spectator)
            .map(|(i, e)| PearlEntityTarget {
                id: i as u64,
                aabb: e.aabb(),
                can_hit: true,
            })
            .collect();
        for p in &mut self.pearls {
            if p.removed {
                continue;
            }
            ticked += 1;
            let owner_ok = p
                .owner_idx
                .is_some_and(|idx| self.living_entities.get(idx).is_some_and(|e| e.alive));
            if let Some(PearlLanding {
                teleport_pos,
                reset_fall_distance,
                damage,
                ..
            }) = pearl::pearl_tick(&self.overlay, &mut p.state, &targets, owner_ok)
            {
                if let Some(idx) = p.owner_idx {
                    if let Some(owner) = self.living_entities.get_mut(idx) {
                        owner.position = teleport_pos;
                        if reset_fall_distance {
                            owner.fall_distance = 0.0;
                        }
                        owner.health = (owner.health - damage).max(0.0);
                        if owner.health == 0.0 {
                            owner.alive = false;
                        }
                    }
                }
                p.removed = true;
            }
        }
        ticked
    }

    fn tick_living_and_crowd_push(&mut self) -> usize {
        let living_len = self.living_entities.len();
        let active_boats: Vec<usize> = self
            .boats
            .iter()
            .enumerate()
            .filter_map(|(bi, b)| (!b.removed).then_some(bi))
            .collect();
        if living_len == 0 {
            return 0;
        }

        let mut cram_events = 0usize;
        let max_cram = self.max_entity_cramming;
        let mut crowd_idx = std::mem::take(&mut self.crowd_index);
        let mut contacts = Vec::with_capacity(living_len + active_boats.len());
        let mut neighbor_idx_scratch = Vec::new();
        let mut neighbor_pairs = Vec::new();

        for i in 0..living_len {
            {
                let e = &self.living_entities[i];
                if !e.alive || e.spectator {
                    continue;
                }
                if !e.no_physics && e.apply_gravity {
                    let was_on_ground = e.on_ground;
                    let bounds = e.aabb();
                    let wanted = e.velocity;
                    let moved = if wanted.length_sqr() == 0.0 {
                        wanted
                    } else {
                        let ctx = CollisionContext {
                            feet_y: bounds.min.y,
                            descending: false,
                            fall_distance: e.fall_distance,
                            leather_boots: false,
                        };
                        let shapes = self.collect_world_collisions(bounds.expand(wanted), ctx);
                        collide_with_shapes(wanted, bounds, &shapes)
                    };
                    let em = &mut self.living_entities[i];
                    em.on_ground = crate::basis::body::settle(&mut em.position, &mut em.velocity, moved).below();
                    // The default block friction 0.6 under it.
                    let row = crate::basis::body::LIVING;
                    em.velocity = row.friction_drag(em.velocity, em.velocity.y - row.gravity, if was_on_ground { 0.6 } else { 1.0 });
                }
            }

            // Per-entity `pushEntities` in spawn order (matching `ServerLevel` sequential tick).
            contacts.clear();
            for e in &self.living_entities {
                contacts.push(e.as_contact_entity());
            }
            for &bi in &active_boats {
                let b = &self.boats[bi].body.boat;
                contacts.push(ContactEntity::boat(b.position, b.velocity));
            }
            crowd_idx.rebuild(&contacts);
            crowd_idx.query_pushable(&contacts, i, &mut neighbor_idx_scratch);
            if !neighbor_idx_scratch.is_empty() {
                neighbor_pairs.clear();
                for &j in &neighbor_idx_scratch {
                    neighbor_pairs.push((contacts[j], false));
                }
                let rand4 = self.next_int_4();
                let mut me = contacts[i];
                if let Some(dmg) = crate::entity_contact::push_entities(
                    &mut me,
                    &mut neighbor_pairs,
                    max_cram,
                    rand4,
                ) {
                    let e = &mut self.living_entities[i];
                    e.health = (e.health - dmg).max(0.0);
                    if e.health == 0.0 {
                        e.alive = false;
                    }
                    cram_events += 1;
                }
                self.living_entities[i].velocity = me.velocity;
                for (k, &j) in neighbor_idx_scratch.iter().enumerate() {
                    if j < living_len {
                        self.living_entities[j].velocity = neighbor_pairs[k].0.velocity;
                    } else {
                        let bi = active_boats[j - living_len];
                        self.boats[bi].body.boat.velocity = neighbor_pairs[k].0.velocity;
                    }
                }
            }
        }

        self.crowd_index = crowd_idx;
        cram_events
    }

    // ------------------------------------------------------------------------
    // Interactive & placement entry points (reusing place, interact, world_hooks, mining)
    // ------------------------------------------------------------------------

    pub fn pull_lever(&mut self, pos: BlockPos) -> (bool, DrainStats) {
        let mut rw = Pending::new(self);
        let mut events = Vec::new();
        let powered = interact::lever_pull(&mut rw, pos, false, &mut events);
        if events.is_empty() {
            return (powered, DrainStats::default());
        }
        let mut updater = std::mem::replace(
            &mut self.neighbor_updater,
            CollectingNeighborUpdater::new(0),
        );
        for ev in events {
            match ev {
                RedstoneEvent::SetBlock { pos: p, flags, powered: pwr, .. } => {
                    if let CellState::Lever(mut l) = self.cell(p).state {
                        l.powered = pwr;
                        self.apply_set_cell(
                            Some(updater.queue_mut()),
                            p,
                            SimCell::new(l.block_id, CellState::Lever(l)),
                            flags as i32,
                            update_flags::DEFAULT_LIMIT,
                        );
                    }
                }
                RedstoneEvent::UpdateNeighborsAt { pos: p, source_id } => {
                    updater
                        .queue_mut()
                        .update_neighbors_except(p, source_id, None);
                }
                RedstoneEvent::ScheduleWaterTick { pos, delay } => {
                    self.schedule_fluid_tick(pos, FluidTickKind::Water, delay);
                }
                RedstoneEvent::DoorSoundAndEvent { .. }
                | RedstoneEvent::TrapDoorSoundAndEvent { .. }
                | RedstoneEvent::FenceGateSoundAndEvent { .. }
                | RedstoneEvent::LeverSoundAndEvent { .. } => {}
            }
        }
        let stats = updater.drain_with(
            |_| {},
            |q, ev| {
                self.handle_neighbor_event(q, ev);
            },
        );
        self.neighbor_updater = updater;
        (powered, stats)
    }

    fn apply_button_events(&mut self, btn: ButtonState, evs: &[ButtonEvent]) {
        for &ev in evs {
            match ev {
                ButtonEvent::SetPowered { pos, flags, .. } => {
                    let next = SimCell::new(btn.block_id, CellState::Button(btn));
                    self.set_cell_with_flags(pos, next, flags);
                }
                ButtonEvent::UpdateNeighborsAt { pos, source_id } => {
                    self.enqueue_update_neighbors(None, pos, source_id, None);
                }
                ButtonEvent::ScheduleTick { pos, delay, .. } => {
                    self.schedule_block_tick(pos, BlockTickKind::Button, delay);
                }
                ButtonEvent::PlaySound { .. } | ButtonEvent::GameEvent { .. } => {}
            }
        }
    }

    fn apply_plate_events(&mut self, plate: PressurePlateState, evs: &[PlateEvent]) {
        for &ev in evs {
            match ev {
                PlateEvent::SetSignal { pos, flags, .. } => {
                    let next = SimCell::new(plate.block_id, CellState::PressurePlate(plate));
                    self.set_cell_with_flags(pos, next, flags);
                }
                PlateEvent::UpdateNeighborsAt { pos, source_id } => {
                    self.enqueue_update_neighbors(None, pos, source_id, None);
                }
                PlateEvent::ScheduleTick { pos, delay, .. } => {
                    self.schedule_block_tick(pos, BlockTickKind::PressurePlate, delay);
                }
                PlateEvent::PlaySound { .. } | PlateEvent::GameEvent { .. } => {}
            }
        }
    }

    fn apply_tripwire_events(
        &mut self,
        mut queue: Option<&mut NeighborQueue<BlockId>>,
        evs: Vec<TripwireEvent>,
    ) {
        for ev in evs {
            match ev {
                TripwireEvent::SetWire { pos, flags, wire } => {
                    let next = SimCell::new(wire.block_id, CellState::TripwireWire(wire));
                    if queue.is_some() {
                        self.apply_set_cell(
                            queue.as_deref_mut(),
                            pos,
                            next,
                            flags,
                            update_flags::DEFAULT_LIMIT,
                        );
                    } else {
                        self.set_cell_with_flags(pos, next, flags);
                    }
                }
                TripwireEvent::SetHook { pos, flags, hook } => {
                    let next = SimCell::new(hook.block_id, CellState::TripwireHook(hook));
                    if queue.is_some() {
                        self.apply_set_cell(
                            queue.as_deref_mut(),
                            pos,
                            next,
                            flags,
                            update_flags::DEFAULT_LIMIT,
                        );
                    } else {
                        self.set_cell_with_flags(pos, next, flags);
                    }
                }
                TripwireEvent::ScheduleTick {
                    pos,
                    block_id,
                    delay,
                } => {
                    let kind = if block_id == BlockId::of("tripwire_hook") {
                        BlockTickKind::TripwireHook
                    } else {
                        BlockTickKind::TripwireWire
                    };
                    self.schedule_block_tick(pos, kind, delay);
                }
                TripwireEvent::UpdateNeighborsAt { pos, source_id } => {
                    self.enqueue_update_neighbors(queue.as_deref_mut(), pos, source_id, None);
                }
                TripwireEvent::HookFeedback { .. } | TripwireEvent::GameEventShear { .. } => {}
            }
        }
    }

    pub fn empty_bucket_at(
        &mut self,
        content: BucketContent,
        pos: BlockPos,
        hit: Option<(BlockPos, Direction)>,
        is_sneaking: bool,
    ) -> bool {
        let mut events = Vec::new();
        let ok = {
            let mut bw = SimBucketView::new(self);
            world_hooks::bucket_empty_contents(
                &mut bw,
                content,
                pos,
                hit,
                is_sneaking,
                false,
                &mut events,
            )
        };
        self.apply_bucket_events(&events);
        ok
    }

    fn apply_bucket_events(&mut self, events: &[BucketEvent]) {
        for &ev in events {
            match ev {
                BucketEvent::DestroyBlock { pos, drop } => {
                    self.destroy_block(pos, drop);
                }
                BucketEvent::SetBlock { pos, cell, flags } => {
                    let sim_cell = self.bucket_cell_to_sim(pos, cell);
                    self.set_cell_with_flags(pos, sim_cell, flags);
                }
                BucketEvent::ScheduleFluidTick { pos, fluid, delay } => {
                    let fk = match fluid {
                        BucketContent::Lava => FluidTickKind::Lava,
                        BucketContent::Water | BucketContent::Empty => FluidTickKind::Water,
                    };
                    self.schedule_fluid_tick(pos, fk, delay);
                }
                BucketEvent::PlaySound { .. }
                | BucketEvent::GameEvent { .. }
                | BucketEvent::EvaporateSmokeParticles { .. } => {}
            }
        }
    }

    fn bucket_cell_to_sim(&self, pos: BlockPos, bc: BucketCell) -> SimCell {
        if bc.is_air {
            return SimCell::air(BlockId::of("air"));
        }
        if let Some((fluid, level)) = bc.liquid_block {
            let amount = if level == 0 { 8 } else { 8u8.saturating_sub(level) };
            let fcell = FluidCell {
                amount,
                falling: level >= 8,
                source: level == 0,
            };
            return match fluid {
                BucketContent::Water => SimCell::water(BlockId::of("water"), fcell),
                BucketContent::Lava => SimCell::lava(BlockId::of("lava"), self.fast_lava, fcell),
                BucketContent::Empty => SimCell::air(BlockId::of("air")),
            };
        }
        let mut existing = self.cell(pos);
        if let Some(wl) = bc.waterlogged {
            existing.set_waterlogged(wl);
        }
        existing
    }

    fn scaffolding_cell(&self, pos: BlockPos) -> ScaffoldingCell {
        let c = self.cell(pos);
        match c.state {
            CellState::Scaffolding(s) => ScaffoldingCell {
                is_scaffolding: true,
                distance: s.distance,
                sturdy_up: false,
                replaceable: false,
            },
            _ => ScaffoldingCell {
                is_scaffolding: false,
                distance: 0,
                sturdy_up: c.props.sturdy_faces[Direction::Up as usize],
                replaceable: c.is_air() || c.props.can_be_replaced,
            },
        }
    }

    fn check_world_hooks(&mut self) {
        if self.pressure_plate_positions.is_empty() && self.tripwire_positions.is_empty() {
            return;
        }
        let triggers: Vec<TriggerEntity> = self
            .living_entities
            .iter()
            .filter(|e| e.alive)
            .map(SimLivingEntity::as_trigger_entity)
            .collect();

        if !triggers.is_empty() {
            let plates: Vec<BlockPos> = self.pressure_plate_positions.iter().copied().collect();
            for p in plates {
                if let CellState::PressurePlate(mut plate) = self.cell(p).state {
                    let touch = world_hooks::PRESSURE_PLATE_TOUCH_AABB
                        .moved(Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64));
                    if triggers.iter().any(|t| t.bbox.intersects(touch)) {
                        let mut evs = Vec::new();
                        world_hooks::pressure_plate_entity_inside(
                            &mut plate,
                            p,
                            false,
                            &triggers,
                            &mut evs,
                        );
                        self.apply_plate_events(plate, &evs);
                    }
                }
            }

            let wires: Vec<BlockPos> = self.tripwire_positions.iter().copied().collect();
            for w in wires {
                if let CellState::TripwireWire(wire_st) = self.cell(w).state {
                    let box_w = wire_st
                        .shape()
                        .moved(Vec3::new(w[0] as f64, w[1] as f64, w[2] as f64));
                    for &tr in &triggers {
                        if tr.bbox.intersects(box_w) {
                            let mut tw = Pending::new(self);
                            let mut evs = Vec::new();
                            world_hooks::tripwire_entity_inside(&mut tw, w, false, tr, &mut evs);
                            self.apply_tripwire_events(None, evs);
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Unified `ServerLevel.tick` phase loop
    // ------------------------------------------------------------------------

    /// Executes one full Mojang 26.2 server tick in exact `SERVER_TICK_PHASES` order:
    /// 1. `game_time += 1` (`tickTime`)
    /// 2. `BlockTicks` (`MAX_TICKS_PER_TICK` cap)
    /// 3. `FluidTicks` (`MAX_TICKS_PER_TICK` cap)
    /// 4. `BlockEvents` (pistons)
    /// 5. `Entities` (falling blocks, boats, pearls, living/crowd push, plate/tripwire hooks)
    /// 6. `BlockEntities` (moving pistons)
    pub fn step_tick(&mut self) -> WorldTick {
        self.game_time += 1;
        let gt = self.game_time;

        self.handling_tick = true;
        // Phase 1: BlockTicks
        let mut b_scratch = std::mem::take(&mut self.block_tick_scratch);
        self.block_ticks
            .collect_due(gt, MAX_TICKS_PER_TICK, |_| true, &mut b_scratch);
        let block_ticks_ran = b_scratch.len();
        for t in &b_scratch {
            self.run_block_tick(t.pos, t.kind);
        }
        b_scratch.clear();
        self.block_tick_scratch = b_scratch;

        // Phase 2: FluidTicks
        let mut f_scratch = std::mem::take(&mut self.fluid_tick_scratch);
        self.fluid_ticks
            .collect_due(gt, MAX_TICKS_PER_TICK, |_| true, &mut f_scratch);
        let fluid_ticks_ran = f_scratch.len();
        for t in &f_scratch {
            self.run_fluid_tick(t.pos, t.kind);
        }
        f_scratch.clear();
        self.fluid_tick_scratch = f_scratch;
        self.handling_tick = false;

        // Phase 3: BlockEvents (pistons)
        let block_events_ran = self.run_block_events();

        // Phase 4: Entities
        let falling_blocks_ticked = self.tick_falling_blocks();
        let boats_ticked = self.tick_boats();
        let pearls_ticked = self.tick_pearls();
        let crowd_cram_damage_events = self.tick_living_and_crowd_push();
        self.check_world_hooks();

        // Phase 5: BlockEntities (moving pistons)
        let block_entities_ticked = self.tick_block_entities();

        WorldTick {
            game_time: gt,
            block_ticks_ran,
            fluid_ticks_ran,
            block_events_ran,
            block_entities_ticked,
            falling_blocks_ticked,
            boats_ticked,
            pearls_ticked,
            crowd_cram_damage_events,
            neighbor_stats: DrainStats::default(),
        }
    }
}

// ============================================================================
// 6. Subsystem Trait Views over SimWorld (zero-duplication bridges)
// ============================================================================

struct SimPistonView<'s, 'a, W: World> {
    sim: &'s SimWorld<'a, W>,
}

impl<W: World> PistonWorld for SimPistonView<'_, '_, W> {
    fn block(&self, position: BlockPos) -> PistonBlock {
        self.sim.cell(position).as_piston_block()
    }
    fn min_y(&self) -> i32 {
        self.sim.min_y
    }
    fn max_y(&self) -> i32 {
        self.sim.max_y
    }
}

/// The world a rule edits while it runs: its own writes, over the sim's cells. The sim then
/// applies the rule's events with their update flags.
struct Pending<'s, 'a, W: World> {
    sim: &'s mut SimWorld<'a, W>,
    pending: HashMap<BlockPos, SimCell>,
}

impl<'s, 'a, W: World> Pending<'s, 'a, W> {
    fn new(sim: &'s mut SimWorld<'a, W>) -> Self {
        Self { sim, pending: HashMap::new() }
    }
}

impl<W: World> WorldMut for Pending<'_, '_, W> {
    fn cell(&self, pos: BlockPos) -> SimCell {
        self.pending.get(&pos).cloned().unwrap_or_else(|| self.sim.cell(pos))
    }
    fn set(&mut self, pos: BlockPos, cell: SimCell) {
        self.pending.insert(pos, cell);
    }
    fn has_scheduled_tick(&self, pos: BlockPos, block: BlockId) -> bool {
        let kind = if block == BlockId::of("tripwire_hook") { BlockTickKind::TripwireHook } else { BlockTickKind::TripwireWire };
        self.sim.block_ticks.container(ChunkPos::from_block_pos(pos)).is_some_and(|c| c.has_scheduled_tick(kind, pos))
    }
}

struct SimBucketView<'s, 'a, W: World> {
    sim: &'s mut SimWorld<'a, W>,
    pending: HashMap<BlockPos, SimCell>,
}

impl<'s, 'a, W: World> SimBucketView<'s, 'a, W> {
    fn new(sim: &'s mut SimWorld<'a, W>) -> Self {
        Self {
            sim,
            pending: HashMap::new(),
        }
    }
}

impl<W: World> BucketWorld for SimBucketView<'_, '_, W> {
    fn air_id(&self) -> BlockId {
        BlockId::of("air")
    }
    fn water_block_id(&self) -> BlockId {
        BlockId::of("water")
    }
    fn lava_block_id(&self) -> BlockId {
        BlockId::of("lava")
    }
    fn cell(&self, pos: BlockPos) -> BucketCell {
        let c = self
            .pending
            .get(&pos)
            .cloned()
            .unwrap_or_else(|| self.sim.cell(pos));
        if c.is_air() {
            return BucketCell::air(c.id);
        }
        if let CellState::Fluid { kind, cell } = c.state {
            let content = match kind {
                FlowKind::Water => BucketContent::Water,
                FlowKind::Lava { .. } => BucketContent::Lava,
            };
            return if cell.source {
                BucketCell::liquid_source(c.id, content)
            } else {
                BucketCell::liquid_flowing(c.id, content, cell.legacy_level())
            };
        }
        if c.is_waterlogged_container() {
            let wl = c.fluid_info().is_some();
            return BucketCell::waterloggable(c.id, wl, self.sim.can_survive(pos, &c));
        }
        BucketCell {
            block_id: c.id,
            is_air: false,
            is_liquid: false,
            is_fluid_source: false,
            can_be_replaced_by_fluid: c.props.replaceable_by_fluid,
            is_liquid_block_container: false,
            accepts_water_container: false,
            waterlogged: None,
            liquid_block: None,
            can_survive: self.sim.can_survive(pos, &c),
        }
    }

    fn set_cell(&mut self, pos: BlockPos, cell: BucketCell) {
        let next = self.sim.bucket_cell_to_sim(pos, cell);
        self.pending.insert(pos, next);
    }

    fn water_evaporates(&self, _pos: BlockPos) -> bool {
        self.sim.water_evaporates
    }
}

fn piston_head_boxes(facing: Direction) -> [Aabb; 2] {
    let t = 0.25;
    let plate = match facing {
        Direction::Down => Aabb::new(0.0, 0.0, 0.0, 1.0, t, 1.0),
        Direction::Up => Aabb::new(0.0, 1.0 - t, 0.0, 1.0, 1.0, 1.0),
        Direction::North => Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, t),
        Direction::South => Aabb::new(0.0, 0.0, 1.0 - t, 1.0, 1.0, 1.0),
        Direction::West => Aabb::new(0.0, 0.0, 0.0, t, 1.0, 1.0),
        Direction::East => Aabb::new(1.0 - t, 0.0, 0.0, 1.0, 1.0, 1.0),
    };
    let arm = match facing {
        Direction::Down => Aabb::new(0.375, 0.25, 0.375, 0.625, 1.25, 0.625),
        Direction::Up => Aabb::new(0.375, -0.25, 0.375, 0.625, 0.75, 0.625),
        Direction::North => Aabb::new(0.375, 0.375, 0.25, 0.625, 0.625, 1.25),
        Direction::South => Aabb::new(0.375, 0.375, -0.25, 0.625, 0.625, 0.75),
        Direction::West => Aabb::new(0.25, 0.375, 0.375, 1.25, 0.625, 0.625),
        Direction::East => Aabb::new(-0.25, 0.375, 0.375, 0.75, 0.625, 0.625),
    };
    [plate, arm]
}
