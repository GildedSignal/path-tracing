//! `ray`: one DDA through cells, each cell clipped against its boxes. The segment walk
//! is `BlockGetter.traverseBlocks`; a box swept along a segment is the same walk from
//! its leading corner (`BlockGetter.forEachBlockIntersectedBetween`); the clip is
//! `AABB.clip` per box, `VoxelShape.clip` per shape, `BlockGetter.clip` per cell.

use crate::{math, neighbor::Direction, world::block_pos, Aabb, Axis, BlockPos, Vec3, VoxelShape};
use super::sweep::axis_order;

/// The walk shared by both DDAs: the cells after `block_pos(start)` that the segment
/// `start + movement·t`, `t ≤ 1`, enters, in order. False when `visit` stopped it.
#[inline(always)]
fn walk(start: Vec3, movement: Vec3, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
    let mut cell = block_pos(start);
    let d = [movement.x, movement.y, movement.z];
    let s = [start.x, start.y, start.z];
    let signs: [i32; 3] = std::array::from_fn(|a| if d[a] > 0.0 { 1 } else if d[a] < 0.0 { -1 } else { 0 });
    let step: [f64; 3] = std::array::from_fn(|a| if signs[a] == 0 { f64::MAX } else { signs[a] as f64 / d[a] });
    let mut next: [f64; 3] = std::array::from_fn(|a| {
        let fraction = s[a] - math::lfloor(s[a]) as f64;
        step[a] * if signs[a] > 0 { 1.0 - fraction } else { fraction }
    });
    while next[0] <= 1.0 || next[1] <= 1.0 || next[2] <= 1.0 {
        let a = if next[0] < next[1] { if next[0] < next[2] { 0 } else { 2 } } else if next[1] < next[2] { 1 } else { 2 };
        cell[a] += signs[a];
        next[a] += step[a];
        if !visit(cell) { return false; }
    }
    true
}

/// `BlockGetter.traverseBlocks`: true when `hit` returned true at some cell.
pub fn cells(from: Vec3, to: Vec3, mut hit: impl FnMut(BlockPos) -> bool) -> bool {
    if from == to { return false; }
    let start = from.plus(from.minus(to).scale(1.0e-7));
    let end = to.plus(to.minus(from).scale(1.0e-7));
    hit(block_pos(start)) || !walk(start, end.minus(start), |c| !hit(c))
}

/// Membership only: the order comes from the walk. Inline storage covers a body's footprint.
pub(crate) struct SeenBlocks { inline: [BlockPos; 32], len: usize, overflow: Vec<BlockPos> }
pub(crate) trait BlockSet { fn insert(&mut self, p: BlockPos) -> bool; }
impl BlockSet for SeenBlocks {
    fn insert(&mut self, p: BlockPos) -> bool {
        if self.inline[..self.len].contains(&p) || self.overflow.contains(&p) { return false; }
        if self.len < self.inline.len() { self.inline[self.len] = p; self.len += 1; } else { self.overflow.push(p); }
        true
    }
}
#[cfg(test)]
impl BlockSet for std::collections::BTreeSet<BlockPos> {
    fn insert(&mut self, p: BlockPos) -> bool { std::collections::BTreeSet::insert(self, p) }
}
impl SeenBlocks { pub fn new() -> Self { Self { inline: [[0; 3]; 32], len: 0, overflow: Vec::new() } } }

/// `BlockPos.betweenCornersInDirection`.
#[inline]
fn between_corners(a: BlockPos, b: BlockPos, direction: Vec3, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
    let axes = axis_order(direction);
    let d = [direction.x, direction.y, direction.z];
    let start: BlockPos = std::array::from_fn(|i| if d[i] >= 0.0 { a[i].min(b[i]) } else { a[i].max(b[i]) });
    let signs: [i32; 3] = std::array::from_fn(|i| if d[i] >= 0.0 { 1 } else { -1 });
    let len: [i32; 3] = std::array::from_fn(|i| (b[i] - a[i]).abs());
    for first in 0..=len[axes[0] as usize] {
        for second in 0..=len[axes[1] as usize] {
            for third in 0..=len[axes[2] as usize] {
                let mut p = start;
                for (axis, offset) in axes.into_iter().zip([first, second, third]) { p[axis as usize] += signs[axis as usize] * offset; }
                if !visit(p) { return false; }
            }
        }
    }
    true
}

/// `BlockGetter.getFurthestCorner`.
fn furthest_corner(v: Vec3) -> Vec3 {
    let [x, y, z] = [v.x, v.y, v.z].map(|c| if c >= 0.0 { 1.0 } else { -1.0 });
    if v.x.abs() <= v.y.abs() && v.x.abs() <= v.z.abs() { Vec3::new(-x, -z, y) }
    else if v.y.abs() <= v.z.abs() { Vec3::new(z, -y, -x) }
    else { Vec3::new(-y, x, -z) }
}

/// `BlockGetter.forEachBlockIntersectedBetween`: the cells `bounds` crosses on its way
/// from `from` to `to`, each with the walk step that reached it. False when stopped.
pub fn cells_along(from: Vec3, to: Vec3, bounds: Aabb, mut visit: impl FnMut(BlockPos, i32) -> bool) -> bool {
    let movement = to.minus(from);
    if movement.length_sqr() < (1.0e-5f32 * 1.0e-5f32) as f64 {
        let (lo, hi) = (block_pos(bounds.min), block_pos(bounds.max));
        for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] {
            if !visit([x, y, z], 0) { return false; }
        } } }
        return true;
    }
    let initial = bounds.moved(movement.scale(-1.0));
    let mut seen = SeenBlocks::new();
    if !between_corners(block_pos(initial.min), block_pos(initial.max), movement, |p| { seen.insert(p); visit(p, 0) }) {
        return false;
    }
    let size = bounds.max.minus(bounds.min);
    let corner = furthest_corner(movement);
    let end = bounds.min.plus(bounds.max).scale(0.5).plus(size.scale(0.5).multiply(corner));
    let start = end.minus(movement);
    let mut count = 0;
    walk(start, movement, |cell| {
        let position = Vec3::new(cell[0] as f64, cell[1] as f64, cell[2] as f64);
        let Some(hit) = (Aabb { min: position, max: position.plus(Vec3::new(1.0, 1.0, 1.0)) }).clip(start, end) else { return true; };
        count += 1;
        let e = 1.0e-5f32 as f64;
        let point = Vec3::new(hit.x.clamp(position.x + e, position.x + 1.0 - e),
            hit.y.clamp(position.y + e, position.y + 1.0 - e), hit.z.clamp(position.z + e, position.z + 1.0 - e));
        let opposite = block_pos(point.minus(size.multiply(corner)));
        between_corners(cell, opposite, movement, |p| !seen.insert(p) || visit(p, count))
    }) && between_corners(block_pos(bounds.min), block_pos(bounds.max), movement, |p| !seen.insert(p) || visit(p, count + 1))
}

/// `AABB.clip(..)` over the three axes (`AABB.clipPoint`, `AABB.getDirection`): the entry
/// face nearer than `nearest`, which it lowers; else `best` unchanged.
#[inline]
pub fn clip_face(b: Aabb, from: Vec3, to: Vec3, nearest: &mut f64, mut best: Option<Direction>) -> Option<Direction> {
    const FACES: [[Direction; 2]; 3] = [[Direction::West, Direction::East], [Direction::Down, Direction::Up], [Direction::North, Direction::South]];
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let delta = to.minus(from);
    for a in 0..3 {
        let d = delta.get(axes[a]);
        let (plane, face) = if d > 1.0e-7 { (b.min.get(axes[a]), FACES[a][0]) }
            else if d < -1.0e-7 { (b.max.get(axes[a]), FACES[a][1]) } else { continue };
        let (u, v) = (axes[(a + 1) % 3], axes[(a + 2) % 3]);
        let t = (plane - from.get(axes[a])) / d;
        let pu = from.get(u) + t * delta.get(u);
        let pv = from.get(v) + t * delta.get(v);
        if 0.0 < t && t < *nearest && b.min.get(u) - 1.0e-7 < pu && pu < b.max.get(u) + 1.0e-7
            && b.min.get(v) - 1.0e-7 < pv && pv < b.max.get(v) + 1.0e-7 {
            *nearest = t;
            best = Some(face);
        }
    }
    best
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipFluidMode { None, SourceOnly, Any }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockHitResult {
    Hit { pos: BlockPos, face: Direction, location: Vec3, inside: bool },
    Miss { pos: BlockPos, face: Direction, location: Vec3 },
}
impl BlockHitResult {
    pub fn location(self) -> Vec3 { match self { Self::Hit { location, .. } | Self::Miss { location, .. } => location } }
    pub fn is_hit(self) -> bool { matches!(self, Self::Hit { .. }) }
}

/// `VoxelShape.clip(Vec3, Vec3, BlockPos)`: inside when the point `0.001` along the ray
/// is in a full cell (`isFullWide`), else the nearest box face with the un-nudged `from`.
pub fn clip_shape(shape: &VoxelShape, pos: BlockPos, from: Vec3, to: Vec3) -> Option<BlockHitResult> {
    if shape.is_empty() { return None; }
    let delta = to.minus(from);
    if delta.length_sqr() < 1.0e-7 { return None; }
    let nudge = from.plus(delta.scale(0.001));
    let view = shape.view();
    let i: [i32; 3] = std::array::from_fn(|a| view.find_index(a, [nudge.x, nudge.y, nudge.z][a] - pos[a] as f64));
    if i.iter().all(|&i| i >= 0) && shape.is_full(i.map(|i| i as usize)) {
        let face = Direction::get_approximate_nearest(delta.x, delta.y, delta.z).opposite();
        return Some(BlockHitResult::Hit { pos, face, location: nudge, inside: true });
    }
    let offset = Vec3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64);
    let (mut nearest, mut best) = (1.0, None);
    for (cell, full) in shape.at(offset).grid_cells() {
        if full { best = clip_face(cell, from, to, &mut nearest, best); }
    }
    best.map(|face| BlockHitResult::Hit { pos, face, location: Vec3::new(from.x + nearest * delta.x,
        from.y + nearest * delta.y, from.z + nearest * delta.z), inside: false })
}

/// `BlockGetter.clip`: the walk, `clipWithInteractionOverride`, the fluid compared by
/// distance, and the backward miss. `cell` gives (shape, interaction shape, fluid
/// height, fluid is a source) at a position.
pub fn clip<'a>(from: Vec3, to: Vec3, fluid_mode: ClipFluidMode,
    mut cell: impl FnMut(BlockPos) -> (&'a VoxelShape, Option<&'a VoxelShape>, f32, bool)) -> BlockHitResult {
    let dist = |h: Option<BlockHitResult>| h.map_or(f64::MAX, |h| from.minus(h.location()).length_sqr());
    let mut hit = None;
    cells(from, to, |pos| {
        let (shape, interaction, fluid_height, source) = cell(pos);
        let mut block = clip_shape(shape, pos, from, to);
        if let Some(i) = interaction.and_then(|s| clip_shape(s, pos, from, to)) {
            if block.is_none() || dist(Some(i)) < dist(block) { block = Some(i); }
        }
        let fluid = match fluid_mode {
            ClipFluidMode::SourceOnly if !source => None,
            ClipFluidMode::SourceOnly | ClipFluidMode::Any if fluid_height > 0.0 =>
                clip_shape(&VoxelShape::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, fluid_height as f64, 1.0)]), pos, from, to),
            _ => None,
        };
        hit = if dist(block) <= dist(fluid) { block } else { fluid };
        hit.is_some()
    });
    hit.unwrap_or_else(|| {
        let back = from.minus(to);
        BlockHitResult::Miss { face: Direction::get_approximate_nearest(back.x, back.y, back.z), location: to, pos: block_pos(to) }
    })
}
