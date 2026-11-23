//! `sweep`: an AABB moved through a box set axis by axis in Java's order (`Shapes.collide`,
//! `Entity.collideWithShapes`). Step-up is the same sweep with the box lifted to each
//! candidate height (`Entity.collide`, `collectCandidateStepUpHeights`). Contact flags
//! come out per axis (free, clamped at a plane, held under `1e-7`); they observe the
//! recurrence and never change it.

use crate::shapes::{ShapeView, SharedCollider};
use crate::{Aabb, Axis, Block, BlockPos, CollisionContext, Input, PlayerState, Vec3, VoxelShape, World};

pub const CONTACT_ZERO: f64 = 1.0e-7;
pub const CONTACT_HIT: f64 = 1.0e-5f32 as f64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome { Free, Clamp { plane: f64 }, Hold { plane: Option<f64>, remainder: f64 } }
impl Outcome {
    pub fn kind(self) -> u8 { match self { Self::Free => 0, Self::Clamp { .. } => 1, Self::Hold { .. } => 2 } }
    pub fn plane(self) -> Option<f64> { match self { Self::Free => None, Self::Clamp { plane } => Some(plane), Self::Hold { plane, .. } => plane } }
}

#[derive(Clone, Copy, Debug)]
pub struct AxisContact { pub axis: Axis, pub bounds: Aabb, pub body: Aabb, pub wanted: f64, pub moved: f64, pub outcome: Outcome }

#[derive(Clone, Copy, Debug)]
pub(crate) struct Pass { pub moved: Vec3, pub axes: [AxisContact; 3] }
impl Pass {
    pub(crate) fn free(bounds: Aabb, body: Aabb, wanted: Vec3) -> Self {
        Self { moved: wanted, axes: [Axis::X, Axis::Y, Axis::Z].map(|axis| AxisContact { axis, bounds, body, wanted: wanted.get(axis), moved: wanted.get(axis), outcome: Outcome::Free }) }
    }
    fn report(&self, world: &impl World) {
        for axis in axis_order(Vec3::new(self.axes[0].wanted, self.axes[1].wanted, self.axes[2].wanted)) { world.observe_axis_contact(self.axes[axis as usize]); }
    }
}

/// `Direction.axisStepOrder`: Y first, then the larger horizontal component.
pub fn axis_order(v: Vec3) -> [Axis; 3] {
    if v.x.abs() < v.z.abs() || (cfg!(feature = "mutant-axis-tie") && v.x.abs() == v.z.abs()) { [Axis::Y, Axis::Z, Axis::X] } else { [Axis::Y, Axis::X, Axis::Z] }
}

/// A box set, visited in its fixed order; false when the visitor stopped.
trait Boxes { fn each(&self, f: impl FnMut(ShapeView<'_>) -> bool) -> bool; }
impl Boxes for [VoxelShape] { fn each(&self, mut f: impl FnMut(ShapeView<'_>) -> bool) -> bool { self.iter().all(|s| f(s.view())) } }
impl Boxes for [Option<SharedCollider>] { fn each(&self, mut f: impl FnMut(ShapeView<'_>) -> bool) -> bool { self.iter().flatten().all(|s| f(s.view())) } }
struct InWorld<'a, W>(&'a W, Aabb, CollisionContext);
impl<W: World> Boxes for InWorld<'_, W> { fn each(&self, f: impl FnMut(ShapeView<'_>) -> bool) -> bool { self.0.visit_collisions(self.1, self.2, f) } }

/// The kernel: `b` moved by `wanted` through a non-empty box set, Y then the larger
/// horizontal axis; each axis stops at the first box that holds it under `1e-7`.
#[inline(always)]
fn axes(boxes: &(impl Boxes + ?Sized), bounds: Aabb, b: Aabb, wanted: Vec3) -> Pass {
    let mut report = Pass::free(bounds, b, wanted);
    let mut result = Vec3::default();
    for axis in axis_order(wanted) {
        let mut d = wanted.get(axis);
        if d == 0.0 { continue; }
        let moved = b.moved(result);
        let r = &mut report.axes[axis as usize];
        boxes.each(|shape| {
            if d.abs() < 1.0e-7 { r.outcome = Outcome::Hold { plane: r.outcome.plane(), remainder: d }; d = 0.0; return false; }
            d = shape.collide_report(axis, moved, d, &mut r.outcome);
            true
        });
        result = result.with(axis, d);
        r.body = moved;
        r.moved = d;
    }
    report.moved = result;
    report
}

/// `Entity.collideWithShapes` over an owned box set.
pub fn shapes(wanted: Vec3, b: Aabb, shapes: &[VoxelShape]) -> Vec3 {
    if shapes.is_empty() { wanted } else { axes(shapes, b, b, wanted).moved }
}

/// `Entity.collideWithShapes` over the world's colliders in `bounds`: up to 32 are held
/// inline; past that the world is streamed again per axis, in the same order.
#[inline]
pub(crate) fn world(world: &impl World, bounds: Aabb, context: CollisionContext, wanted: Vec3, b: Aabb) -> Pass {
    let mut inline: [Option<SharedCollider>; 32] = std::array::from_fn(|_| None);
    let mut len = 0;
    let complete = world.visit_collisions(bounds, context, |shape| {
        if len == inline.len() { return false; }
        inline[len] = Some(shape.shared());
        len += 1;
        true
    });
    if !complete { axes(&InWorld(world, bounds, context), bounds, b, wanted) }
    else if len == 0 { Pass::free(bounds, b, wanted) }
    else { axes(&inline[..len], bounds, b, wanted) }
}

/// `Entity.collide`: the sweep, then, when it stopped a horizontal axis on or onto the
/// ground and `step > 0`, the same sweep from each candidate height (the box set's
/// Y planes within `step`, ascending), kept when it goes further horizontally.
#[inline]
pub fn collide(world: &impl World, b: Aabb, context: CollisionContext, v: Vec3, on_ground: impl FnOnce() -> bool, step: f32) -> Vec3 {
    let report = if v.length_sqr() == 0.0 { Pass::free(b, b, v) } else { self::world(world, b.expand(v), context, v, b) };
    let result = report.moved;
    let landed = v.y != result.y && v.y < 0.0;
    if step > 0.0 && (landed || on_ground()) && (v.x != result.x || v.z != result.z) {
        let base = if landed { b.moved(Vec3::new(0.0, result.y, 0.0)) } else { b };
        let mut bounds = base.expand(Vec3::new(v.x, step as f64, v.z));
        if !landed { bounds = bounds.expand(Vec3::new(0.0, -1.0e-5f32 as f64, 0.0)); }
        let previous_y = result.y as f32;
        let mut last: Option<f32> = None;
        loop {
            let mut next: Option<f32> = None;
            world.visit_collisions(bounds, context, |shape| {
                for i in 0..shape.shape.coords[1].len() {
                    let candidate = (shape.coord(1, i) - base.min.y) as f32;
                    if candidate < 0.0 || candidate == previous_y { continue; }
                    if candidate > step { break; }
                    if last.is_some_and(|v| candidate == v || candidate.total_cmp(&v).is_le()) { continue; }
                    if next.is_none_or(|v| candidate.total_cmp(&v).is_lt()) { next = Some(candidate); }
                }
                true
            });
            let Some(candidate) = next else { break };
            last = Some(candidate);
            let stepped = self::world(world, bounds, context, Vec3::new(v.x, candidate as f64, v.z), base);
            if stepped.moved.horizontal_sqr() > result.horizontal_sqr() {
                stepped.report(world);
                return stepped.moved.minus(Vec3::new(0.0, b.min.y - base.min.y, 0.0));
            }
        }
    }
    report.report(world);
    result
}

/// The player's tick with its contact flags recorded, in axis-pass order.
pub fn tick(state: &mut PlayerState, world: &impl World, input: &Input) -> Option<(Vec3, Vec3, Vec<AxisContact>)> {
    let recorder = ContactRecorder { world, passes: std::cell::RefCell::new(Vec::new()) };
    let (wanted, moved) = crate::tick_with_collision(state, &recorder, input)?;
    Some((wanted, moved, recorder.passes.into_inner()))
}
struct ContactRecorder<'a, W> { world: &'a W, passes: std::cell::RefCell<Vec<AxisContact>> }
impl<W: World> World for ContactRecorder<'_, W> {
    fn observe_axis_contact(&self, c: AxisContact) { self.passes.borrow_mut().push(c); self.world.observe_axis_contact(c); }
    fn observe_tick(&self) { self.world.observe_tick(); }
    fn block(&self, p: BlockPos) -> &Block { self.world.block(p) }
    fn hook_block(&self, p: BlockPos) -> &Block { self.world.hook_block(p) }
    fn fluid_block(&self, p: BlockPos) -> &Block { self.world.fluid_block(p) }
    fn raw_fluid_block(&self, p: BlockPos) -> Option<&Block> { self.world.raw_fluid_block(p) }
    fn model_id(&self, p: BlockPos) -> Option<u16> { self.world.model_id(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.world.loaded(p) }
    fn difficulty(&self) -> u8 { self.world.difficulty() }
    fn natural_regeneration(&self) -> bool { self.world.natural_regeneration() }
    fn min_y(&self) -> i32 { self.world.min_y() }
    fn fast_lava(&self) -> bool { self.world.fast_lava() }
    fn has_fluid(&self) -> bool { self.world.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.world.has_inside_effects() }
    fn read_state(&self, f: crate::state_reads::StateField) { self.world.read_state(f); }
    fn write_state(&self, f: crate::state_reads::StateField) { self.world.write_state(f) }
    fn observe_region(&self, q: crate::footprint::Query, lo: BlockPos, hi: BlockPos) { self.world.observe_region(q, lo, hi) }
    fn observe_sweep(&self, a: Aabb, b: Aabb) { self.world.observe_sweep(a, b) }
    fn collision_cell_bounds(&self, b: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> { self.world.collision_cell_bounds(b, lo, hi) }
    fn visit_collision_cells(&self, b: Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool { self.world.visit_collision_cells(b, lo, hi, visit) }
    fn collision_shape(&self, p: BlockPos, c: CollisionContext) -> &VoxelShape { self.world.collision_shape(p, c) }
    fn extra_collisions(&self, b: Aabb, c: CollisionContext) -> Vec<VoxelShape> { self.world.extra_collisions(b, c) }
    fn visit_extra_collisions(&self, b: Aabb, c: CollisionContext, v: impl FnMut(ShapeView<'_>) -> bool) -> bool { self.world.visit_extra_collisions(b, c, v) }
}

/// `Entity.collide` for a body without step-up, unobserved: what `v` moves `b` by.
#[inline]
pub fn moved(world: &impl World, b: Aabb, context: CollisionContext, v: Vec3) -> Vec3 {
    if v.length_sqr() == 0.0 { v } else { self::world(world, b.expand(v), context, v, b).moved }
}
