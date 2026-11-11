//! Opt-in dependencies for word certificates and worldtubes. The wrapped world
//! must be an immutable snapshot for the duration of a recording.
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
};
use crate::{Aabb, Block, BlockPos, CollisionContext, Input, PlayerState, VoxelShape, World, state_reads::{StateField, StateReads}};
use crate::shapes::ShapeView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Query {
    Block = 1,
    Shape = 2,
    Fluid = 4,
    Hooks = 8,
    Loaded = 16,
}

#[derive(Clone, Debug)]
pub enum EnvironmentRead {
    HasFluid(bool),
    HasInsideEffects(bool),
    MinY(i32),
    FastLava(bool),
    Difficulty(u8),
    NaturalRegeneration(bool),
    /// Entity/border dependencies are spatial queries, separate from block cells.
    ExtraCollisions {
        bounds: Aabb,
        context: CollisionContext,
        shapes: Vec<VoxelShape>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Footprint {
    /// Bitwise union of Query values. Includes known-empty candidate cells that
    /// optimized collision/fluid/hook queries certified without fetching a block.
    pub reads: BTreeMap<BlockPos, u8>,
    /// Cells of the conservative enclosing box of each body segment.
    pub swept: BTreeSet<BlockPos>,
    pub environment: Vec<EnvironmentRead>,
    /// The state fields read before they were written (docs/vocabulary.md, section 3).
    pub state_reads: StateReads,
}

impl Footprint {
    pub fn contains(&self, position: BlockPos) -> bool {
        self.reads.contains_key(&position) || self.swept.contains(&position)
    }

    pub fn extend(&mut self, other: &Self) {
        for (&position, &queries) in &other.reads {
            *self.reads.entry(position).or_default() |= queries;
        }
        self.swept.extend(&other.swept);
        self.environment.extend(other.environment.iter().cloned());
        self.state_reads.0 |= other.state_reads.0;
    }
}

#[derive(Clone, Debug)]
pub struct TickRecord {
    /// Includes all simulation counters and observed attributes at entry.
    pub initial: PlayerState,
    pub input: Input,
    pub footprint: Footprint,
}

#[derive(Clone, Debug)]
pub struct ResourceRecord {
    pub initial: PlayerState,
    pub event: crate::ServerResourceEvent,
    pub footprint: Footprint,
}

/// Cells per axis of the tape's dense window.
const SPAN: i32 = 16;
/// The tape's own bit for a swept cell, beside the five `Query` bits.
const SWEPT: u8 = 128;

/// What a recording holds while the law runs: one byte per cell in a window around the
/// first cell touched, and a map for the cells beyond it. `Footprint` is its sorted form.
struct Tape {
    /// The window's least corner, once a cell has been touched.
    corner: Option<BlockPos>,
    /// Indexed (x, y, z), so ascending indices are ascending positions.
    window: [u8; (SPAN * SPAN * SPAN) as usize],
    beyond: BTreeMap<BlockPos, u8>,
    environment: Vec<EnvironmentRead>,
}

/// Every recorded bit passes here.
#[inline(always)]
fn mark(slot: &mut u8, bits: u8) {
    *slot |= bits;
}

impl Tape {
    fn new() -> Self {
        Self { corner: None, window: [0; (SPAN * SPAN * SPAN) as usize], beyond: BTreeMap::new(), environment: Vec::new() }
    }

    #[inline]
    fn cell(&mut self, p: BlockPos, bits: u8) {
        let corner = *self.corner.get_or_insert(p.map(|v| v - SPAN / 2));
        let d = [0, 1, 2].map(|a| p[a].wrapping_sub(corner[a]));
        if d.iter().all(|v| (0..SPAN).contains(v)) {
            mark(&mut self.window[((d[0] * SPAN + d[1]) * SPAN + d[2]) as usize], bits);
        } else {
            mark(self.beyond.entry(p).or_default(), bits);
        }
    }

    fn region(&mut self, lo: BlockPos, hi: BlockPos, bits: u8) {
        if (0..3).any(|a| lo[a] > hi[a]) { return; }
        let corner = *self.corner.get_or_insert(lo.map(|v| v - SPAN / 2));
        let inside = |p: BlockPos| (0..3).all(|a| (0..SPAN).contains(&p[a].wrapping_sub(corner[a])));
        if inside(lo) && inside(hi) {
            let d = |p: BlockPos, a: usize| (p[a] - corner[a]) as usize;
            let span = SPAN as usize;
            for x in d(lo, 0)..=d(hi, 0) {
                for y in d(lo, 1)..=d(hi, 1) {
                    let row = (x * span + y) * span;
                    for slot in &mut self.window[row + d(lo, 2)..=row + d(hi, 2)] { mark(slot, bits); }
                }
            }
        } else {
            cells(lo, hi, |p| self.cell(p, bits));
        }
    }

    /// The sorted form. The window is scanned eight cells at a time.
    fn footprint(&self, state_reads: StateReads, environment: Vec<EnvironmentRead>) -> Footprint {
        let mut reads = Vec::new();
        let mut swept = Vec::new();
        let mut cell = |p: BlockPos, bits: u8| {
            if bits & !SWEPT != 0 { reads.push((p, bits & !SWEPT)); }
            if bits & SWEPT != 0 { swept.push(p); }
        };
        if let Some(corner) = self.corner {
            for (chunk, eight) in self.window.chunks_exact(8).enumerate() {
                if u64::from_ne_bytes(eight.try_into().unwrap()) == 0 { continue; }
                for (offset, &bits) in eight.iter().enumerate().filter(|(_, bits)| **bits != 0) {
                    let index = (chunk * 8 + offset) as i32;
                    cell([corner[0] + index / (SPAN * SPAN), corner[1] + index / SPAN % SPAN, corner[2] + index % SPAN], bits);
                }
            }
        }
        // The window comes out sorted, and sorted input builds each tree in one pass.
        if !self.beyond.is_empty() {
            for (&p, &bits) in &self.beyond { cell(p, bits); }
            reads.sort_unstable();
            swept.sort_unstable();
        }
        Footprint { reads: reads.into_iter().collect(), swept: swept.into_iter().collect(), environment, state_reads }
    }
}

pub struct RecordingWorld<'a, W> {
    world: &'a W,
    tape: RefCell<Tape>,
    /// The state fields read before they were written, and those written, since the last `take`.
    state_reads: Cell<u128>,
    written: Cell<u128>,
}

impl<'a, W: World> RecordingWorld<'a, W> {
    pub fn new(world: &'a W) -> Self {
        Self {
            world,
            tape: RefCell::new(Tape::new()),
            state_reads: Cell::new(0),
            written: Cell::new(0),
        }
    }

    pub fn footprint(&self) -> Footprint {
        let tape = self.tape.borrow();
        tape.footprint(StateReads(self.state_reads.get()), tape.environment.clone())
    }

    pub fn take(&self) -> Footprint {
        let mut tape = self.tape.borrow_mut();
        let environment = std::mem::take(&mut tape.environment);
        let footprint = tape.footprint(StateReads(self.state_reads.replace(0)), environment);
        self.written.set(0);
        tape.corner = None;
        tape.window.fill(0);
        tape.beyond.clear();
        footprint
    }

    /// Adds another recording's footprint to this one.
    fn absorb(&self, footprint: &Footprint) {
        let mut tape = self.tape.borrow_mut();
        for (&p, &queries) in &footprint.reads { tape.cell(p, queries); }
        for &p in &footprint.swept { tape.cell(p, SWEPT); }
        tape.environment.extend(footprint.environment.iter().cloned());
        self.state_reads.set(self.state_reads.get() | footprint.state_reads.0);
    }

    /// Returns one tick's dependencies and retains their union across calls.
    pub fn tick(&self, state: &mut PlayerState, input: &Input) -> TickRecord {
        let tick_world = Self::new(self.world);
        let initial = state.clone();
        crate::tick(state, &tick_world, input);
        let footprint = tick_world.take();
        self.absorb(&footprint);
        TickRecord { initial, input: *input, footprint }
    }

    pub fn server_event(&self, state: &mut PlayerState, event: crate::ServerResourceEvent) -> ResourceRecord {
        let event_world = Self::new(self.world);
        let initial = state.clone();
        crate::apply_server_resource_event(state, &event_world, event.clone());
        let footprint = event_world.take();
        self.absorb(&footprint);
        ResourceRecord { initial, event, footprint }
    }

    #[inline]
    fn read(&self, position: BlockPos, query: Query) {
        self.tape.borrow_mut().cell(position, query as u8);
    }

    fn environment(&self, read: EnvironmentRead) {
        self.tape.borrow_mut().environment.push(read);
    }
}

fn cells(lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos)) {
    for z in lo[2]..=hi[2] {
        for y in lo[1]..=hi[1] {
            for x in lo[0]..=hi[0] {
                visit([x, y, z]);
            }
        }
    }
}

impl<W: World> World for RecordingWorld<'_, W> {
    #[inline]
    fn read_state(&self, field: StateField) {
        self.state_reads.set(self.state_reads.get() | (1 << field as u8) & !self.written.get());
    }
    #[inline]
    fn write_state(&self, field: StateField) {
        self.written.set(self.written.get() | 1 << field as u8);
    }

    fn block(&self, p: BlockPos) -> &Block {
        self.read(p, Query::Block);
        self.world.block(p)
    }

    fn hook_block(&self, p: BlockPos) -> &Block {
        self.read(p, Query::Hooks);
        self.world.hook_block(p)
    }

    fn fluid_block(&self, p: BlockPos) -> &Block {
        self.read(p, Query::Fluid);
        self.world.fluid_block(p)
    }
    fn raw_fluid_block(&self, p: BlockPos) -> Option<&Block> {
        self.read(p, Query::Fluid);
        self.read(p, Query::Loaded);
        self.world.raw_fluid_block(p)
    }
    fn collision_shape(&self, p: BlockPos, context: CollisionContext) -> &VoxelShape {
        self.read(p, Query::Shape);
        self.world.collision_shape(p, context)
    }
    fn loaded(&self, p: BlockPos) -> bool {
        self.read(p, Query::Loaded);
        self.world.loaded(p)
    }
    fn observe_region(&self, query: Query, lo: BlockPos, hi: BlockPos) {
        self.tape.borrow_mut().region(lo, hi, query as u8);
    }
    fn observe_sweep(&self, from: Aabb, to: Aabb) {
        let lo = crate::world::block_pos(crate::Vec3::new(
            from.min.x.min(to.min.x), from.min.y.min(to.min.y), from.min.z.min(to.min.z)));
        let hi = crate::world::block_pos(crate::Vec3::new(
            from.max.x.max(to.max.x), from.max.y.max(to.max.y), from.max.z.max(to.max.z)));
        self.tape.borrow_mut().region(lo, hi, SWEPT);
    }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        self.observe_region(Query::Shape, lo, hi);
        self.world.collision_cell_bounds(bounds, lo, hi)
    }
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos,
        visit: impl FnMut(BlockPos) -> bool) -> bool {
        self.observe_region(Query::Shape, lo, hi);
        self.world.visit_collision_cells(bounds, lo, hi, visit)
    }
    fn has_fluid(&self) -> bool {
        let value = self.world.has_fluid();
        self.environment(EnvironmentRead::HasFluid(value));
        value
    }
    fn has_inside_effects(&self) -> bool {
        let value = self.world.has_inside_effects();
        self.environment(EnvironmentRead::HasInsideEffects(value));
        value
    }
    fn min_y(&self) -> i32 {
        let value = self.world.min_y();
        self.environment(EnvironmentRead::MinY(value));
        value
    }
    fn fast_lava(&self) -> bool {
        let value = self.world.fast_lava();
        self.environment(EnvironmentRead::FastLava(value));
        value
    }
    fn difficulty(&self) -> u8 {
        let value = self.world.difficulty();
        self.environment(EnvironmentRead::Difficulty(value));
        value
    }
    fn natural_regeneration(&self) -> bool {
        let value = self.world.natural_regeneration();
        self.environment(EnvironmentRead::NaturalRegeneration(value));
        value
    }
    fn extra_collisions(&self, bounds: Aabb, context: CollisionContext) -> Vec<VoxelShape> {
        let shapes = self.world.extra_collisions(bounds, context);
        self.environment(EnvironmentRead::ExtraCollisions { bounds, context, shapes: shapes.clone() });
        shapes
    }
    fn visit_extra_collisions(&self, bounds: Aabb, context: CollisionContext,
        mut visit: impl FnMut(ShapeView<'_>) -> bool) -> bool {
        // Retain the complete query result even if the law stops at its first hit.
        let mut shapes = Vec::new();
        self.world.visit_extra_collisions(bounds, context, |shape| {
            shapes.push(shape.owned());
            true
        });
        let complete = shapes.iter().all(|shape| visit(shape.view()));
        self.environment(EnvironmentRead::ExtraCollisions { bounds, context, shapes });
        complete
    }
}
