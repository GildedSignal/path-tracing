use crate::{
    math,
    shapes::axis_step_order,
    world::block_pos,
    Aabb,
    Axis,
    BlockPos,
    Vec3
};

// Membership only: traversal order comes from the law, never from this set.
// Inline storage covers the body footprint without allocating tree nodes.
pub(crate) struct SeenBlocks {
    inline: [BlockPos;32],
    len: usize,
    overflow: Vec<BlockPos>,
}
pub(crate) trait BlockSet { fn insert(&mut self,p:BlockPos)->bool; }
impl BlockSet for SeenBlocks {
    fn insert(&mut self,p:BlockPos)->bool { SeenBlocks::insert(self,p) }
}
#[cfg(test)]
impl BlockSet for std::collections::BTreeSet<BlockPos> {
    fn insert(&mut self,p:BlockPos)->bool { std::collections::BTreeSet::insert(self,p) }
}
impl SeenBlocks {
    pub fn new()->Self { Self { inline:[[0;3];32],len:0,overflow:Vec::new() } }
    pub fn insert(&mut self,p:BlockPos)->bool {
        if self.inline[..self.len].contains(&p) || self.overflow.contains(&p) { return false; }
        if self.len<self.inline.len() { self.inline[self.len]=p;self.len+=1; }
        else { self.overflow.push(p); }
        true
    }
}

// BlockPos.betweenCornersInDirection; BlockPos$7.computeNext
fn between_corners(a: BlockPos, b: BlockPos, direction: Vec3, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
    let axes = axis_step_order(direction);
    let start: BlockPos = std::array::from_fn(|i| if direction.get([Axis::X, Axis::Y, Axis::Z][i]) >= 0.0 {
        a[i].min(b[i])
    } else {
        a[i].max(b[i])
    });
    let signs: [i32; 3] = std::array::from_fn(|i| if direction.get([Axis::X, Axis::Y, Axis::Z][i]) >= 0.0 {
        1
    } else {
        -1
    });
    let lengths: [i32; 3] = std::array::from_fn(|i| (b[i]-a[i]).abs());
    for first in 0..=lengths[axes[0] as usize] {
        for second in 0..=lengths[axes[1] as usize] {
            for third in 0..=lengths[axes[2] as usize] {
                let mut p = start;
                for (axis, offset) in axes.into_iter().zip([first, second, third]) {
                    p[axis as usize] += signs[axis as usize]*offset;
                }
                if !visit(p) {
                    return false;
                }
            }
        }
    }
    true
}

// BlockGetter.getFurthestCorner
fn furthest_corner(v: Vec3) -> Vec3 {
    let x = if v.x >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let y = if v.y >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let z = if v.z >= 0.0 {
        1.0
    } else {
        -1.0
    };
    if v.x.abs() <= v.y.abs() && v.x.abs() <= v.z.abs() {
        Vec3::new(-x, -z, y)
    } else if v.y.abs() <= v.z.abs() {
        Vec3::new(z, -y, -x)
    } else {
        Vec3::new(-y, x, -z)
    }
}

// BlockGetter.forEachBlockIntersectedBetween; BlockGetter.addCollisionsAlongTravel
pub fn for_each_intersected(from: Vec3, to: Vec3, bounds: Aabb, mut visit: impl FnMut(BlockPos, i32) -> bool) -> bool {
    let movement = to.minus(from);
    if movement.length_sqr() < (1.0e-5f32 * 1.0e-5f32) as f64 {
        let lo = block_pos(bounds.min);
        let hi = block_pos(bounds.max);
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                for x in lo[0]..=hi[0] {
                    if !visit([x, y, z], 0) {
                        return false;
                    }
                }
            }
        }
        return true;
    }
    let initial = bounds.moved(movement.scale(-1.0));
    let mut seen = SeenBlocks::new();
    if !between_corners(block_pos(initial.min), block_pos(initial.max), movement, |p| {
        if !visit(p, 0) {
            return false;
        }
        seen.insert(p);
        true
    }) {
        return false;
    }
    let size = bounds.max.minus(bounds.min);
    let corner = furthest_corner(movement);
    let center = bounds.min.plus(bounds.max).scale(0.5);
    let end = center.plus(size.scale(0.5).multiply(corner));
    let start = end.minus(movement);
    let mut cell = block_pos(start);
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let signs: [i32; 3] = std::array::from_fn(|a| {
        let d = movement.get(axes[a]);
        if d > 0.0 {
            1
        } else if d < 0.0 {
            -1
        } else {
            0
        }
    });
    let step: [f64; 3] = std::array::from_fn(|a| if signs[a] == 0 {
        f64::MAX
    } else {
        signs[a] as f64 / movement.get(axes[a])
    });
    let mut next: [f64; 3] = std::array::from_fn(|a| {
        let fraction = start.get(axes[a])-math::lfloor(start.get(axes[a])) as f64;
        step[a] * if signs[a]>0 {
            1.0-fraction
        } else {
            fraction
        }
    });
    let mut count = 0;
    while next[0] <= 1.0 || next[1] <= 1.0 || next[2] <= 1.0 {
        let axis = if next[0] < next[1] {
            if next[0] < next[2] {
                0
            } else {
                2
            }
        } else if next[1] < next[2] {
            1
        } else {
            2
        };
        cell[axis] += signs[axis];
        next[axis] += step[axis];
        let position = Vec3::new(cell[0] as f64, cell[1] as f64, cell[2] as f64);
        let cube = Aabb {
            min: position,
            max: position.plus(Vec3::new(1.0, 1.0, 1.0))
        };
        let Some(hit) = cube.clip(start, end) else {
            continue;
        };
        count += 1;
        let point = Vec3::new(
        hit.x.clamp(position.x+1.0e-5f32 as f64, position.x+1.0-1.0e-5f32 as f64),
        hit.y.clamp(position.y+1.0e-5f32 as f64, position.y+1.0-1.0e-5f32 as f64),
        hit.z.clamp(position.z+1.0e-5f32 as f64, position.z+1.0-1.0e-5f32 as f64));
        let opposite = block_pos(point.minus(size.multiply(corner)));
        if !between_corners(cell, opposite, movement, |p| !seen.insert(p) || visit(p, count)) {
            return false;
        }
    }
    between_corners(block_pos(bounds.min), block_pos(bounds.max), movement, |p| !seen.insert(p) || visit(p, count+1))
}

// BlockGetter.traverseBlocks
pub fn traverse_blocks(from: Vec3, to: Vec3, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
    if from == to {
        return false;
    }
    let start = from.plus(from.minus(to).scale(1.0e-7));
    let end = to.plus(to.minus(from).scale(1.0e-7));
    let movement = end.minus(start);
    let mut cell = block_pos(start);
    if visit(cell) {
        return true;
    }
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let signs: [i32; 3] = std::array::from_fn(|a| {
        let d = movement.get(axes[a]);
        if d > 0.0 {
            1
        } else if d < 0.0 {
            -1
        } else {
            0
        }
    });
    let step: [f64; 3] = std::array::from_fn(|a| {
        if signs[a] == 0 {
            f64::MAX
        } else {
            signs[a] as f64/movement.get(axes[a])
        }
    });
    let mut next: [f64; 3] = std::array::from_fn(|a| {
        let coordinate = start.get(axes[a]);
        let fraction = coordinate-math::lfloor(coordinate) as f64;
        step[a] * if signs[a] > 0 { 1.0-fraction } else { fraction }
    });
    while next[0] <= 1.0 || next[1] <= 1.0 || next[2] <= 1.0 {
        let axis = if next[0] < next[1] {
            if next[0] < next[2] {
                0
            } else {
                2
            }
        } else if next[1] < next[2] {
            1
        } else {
            2
        };
        cell[axis] += signs[axis];
        next[axis] += step[axis];
        if visit(cell) {
            return true;
        }
    }
    false
}
