//! Opt-in dependencies for word certificates and worldtubes. The wrapped world
//! must be an immutable snapshot for the duration of a recording.
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};
use crate::{Aabb, Block, BlockPos, CollisionContext, Input, PlayerState, VoxelShape, World};
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

pub struct RecordingWorld<'a, W> {
    world: &'a W,
    footprint: RefCell<Footprint>,
}

impl<'a, W: World> RecordingWorld<'a, W> {
    pub fn new(world: &'a W) -> Self {
        Self {
            world,
            footprint: RefCell::new(Footprint::default()),
        }
    }

    pub fn footprint(&self) -> Footprint {
        self.footprint.borrow().clone()
    }

    pub fn take(&self) -> Footprint {
        std::mem::take(&mut *self.footprint.borrow_mut())
    }

    /// Returns one tick's dependencies and retains their union across calls.
    pub fn tick(&self, state: &mut PlayerState, input: &Input) -> TickRecord {
        let tick_world = Self::new(self.world);
        let initial = state.clone();
        crate::tick(state, &tick_world, input);
        let footprint = tick_world.take();
        self.footprint.borrow_mut().extend(&footprint);
        TickRecord { initial, input: *input, footprint }
    }

    pub fn server_event(&self, state: &mut PlayerState, event: crate::ServerResourceEvent) -> ResourceRecord {
        let event_world = Self::new(self.world);
        let initial = state.clone();
        crate::apply_server_resource_event(state, &event_world, event.clone());
        let footprint = event_world.take();
        self.footprint.borrow_mut().extend(&footprint);
        ResourceRecord { initial, event, footprint }
    }

    fn read(&self, position: BlockPos, query: Query) {
        *self.footprint.borrow_mut().reads.entry(position).or_default() |= query as u8;
    }

    fn environment(&self, read: EnvironmentRead) {
        self.footprint.borrow_mut().environment.push(read);
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
        cells(lo, hi, |p| self.read(p, query));
    }
    fn observe_sweep(&self, from: Aabb, to: Aabb) {
        let lo = crate::world::block_pos(crate::Vec3::new(
            from.min.x.min(to.min.x), from.min.y.min(to.min.y), from.min.z.min(to.min.z)));
        let hi = crate::world::block_pos(crate::Vec3::new(
            from.max.x.max(to.max.x), from.max.y.max(to.max.y), from.max.z.max(to.max.z)));
        cells(lo, hi, |p| {
            self.footprint.borrow_mut().swept.insert(p);
        });
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
