use crate::math;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}
impl Vec3 {
    // Vec3.<init>
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z
        }
    }

    // Vec3.lengthSqr
    pub fn length_sqr(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    // Vec3.horizontalDistanceSqr
    pub fn horizontal_sqr(self) -> f64 {
        self.x * self.x + self.z * self.z
    }

    // Vec3.add
    pub fn plus(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }

    // Vec3.subtract
    pub fn minus(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }

    // Vec3.multiply
    pub fn multiply(self, b: Self) -> Self {
        Self::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }

    // Vec3.scale
    pub fn scale(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    // Vec3.normalize
    pub fn normalize(self) -> Self {
        let len = self.length_sqr().sqrt();
        if len < 1.0e-5 {
            Self::default()
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    // Vec3.get
    pub fn get(self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z
        }
    }

    // Vec3.with
    pub fn with(mut self, axis: Axis, value: f64) -> Self {
        match axis {
            Axis::X => self.x = value,
            Axis::Y => self.y = value,
            Axis::Z => self.z = value
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2
}

// Direction.axisStepOrder
pub fn axis_step_order(v: Vec3) -> [Axis; 3] {
    let z_first = v.x.abs() < v.z.abs() || (cfg!(feature = "mutant-axis-tie") && v.x.abs() == v.z.abs());
    if z_first {
        [Axis::Y, Axis::Z, Axis::X]
    } else {
        [Axis::Y, Axis::X, Axis::Z]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3
}
impl Aabb {
    // AABB.<init>
    pub fn new(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Self {
        Self {
            min: Vec3::new(x0.min(x1), y0.min(y1), z0.min(z1)),
            max: Vec3::new(x0.max(x1), y0.max(y1), z0.max(z1))
        }
    }

    // EntityDimensions.makeBoundingBox
    pub fn player(p: Vec3, height: f32) -> Self {
        let r = (0.6f32 / 2.0f32) as f64;
        Self::new(p.x - r, p.y, p.z - r, p.x + r, p.y + height as f64, p.z + r)
    }

    // AABB.move
    pub fn moved(self, v: Vec3) -> Self {
        Self {
            min: self.min.plus(v),
            max: self.max.plus(v)
        }
    }

    // AABB.expandTowards
    pub fn expand(self, v: Vec3) -> Self {
        Self {
            min: self.min.plus(Vec3::new(v.x.min(0.0), v.y.min(0.0), v.z.min(0.0))),
            max: self.max.plus(Vec3::new(v.x.max(0.0), v.y.max(0.0), v.z.max(0.0)))
        }
    }

    // AABB.deflate
    pub fn deflate(self, d: f64) -> Self {
        Self::new(self.min.x+d, self.min.y+d, self.min.z+d, self.max.x-d, self.max.y-d, self.max.z-d)
    }

    // AABB.contains
    pub fn contains(self, p: Vec3) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y && p.z >= self.min.z && p.z < self.max.z
    }

    // AABB.clip; AABB.getDirection; AABB.clipPoint
    pub fn clip(self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let delta = to.minus(from);
        let mut nearest = 1.0;
        let mut hit = false;
        let axes = [Axis::X, Axis::Y, Axis::Z];
        for a in 0..3 {
            let axis = axes[a];
            let movement = delta.get(axis);
            if movement.abs() <= 1.0e-7 {
                continue;
            }
            let plane = if movement > 0.0 {
                self.min.get(axis)
            } else {
                self.max.get(axis)
            };
            let t = (plane - from.get(axis)) / movement;
            let u = axes[(a+1)%3];
            let v = axes[(a+2)%3];
            let pu = from.get(u) + t * delta.get(u);
            let pv = from.get(v) + t * delta.get(v);
            if 0.0 < t && t < nearest && self.min.get(u)-1.0e-7 < pu && pu < self.max.get(u)+1.0e-7 &&
            self.min.get(v)-1.0e-7 < pv && pv < self.max.get(v)+1.0e-7 {
                nearest = t;
                hit = true;
            }
        }
        hit.then(|| from.plus(delta.scale(nearest)))
    }

    // AABB.collidedAlongVector
    pub fn collided_along(self, movement: Vec3, shape: &VoxelShape) -> bool {
        let center = self.min.plus(self.max).scale(0.5);
        let end = center.plus(movement);
        let inflation = self.max.minus(self.min).scale(0.5).minus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7));
        let [nx, ny, nz] = shape.sizes();
        for x in 0..nx {
            for y in 0..ny {
                for z in 0..nz {
                    if !shape.is_full([x, y, z]) {
                        continue;
                    }
                    let bounds = Aabb {
                        min: Vec3::new(shape.coords[0][x], shape.coords[1][y], shape.coords[2][z]).minus(inflation),
                        max: Vec3::new(shape.coords[0][x+1], shape.coords[1][y+1], shape.coords[2][z+1]).plus(inflation),
                    };
                    if bounds.contains(center) || bounds.contains(end) || bounds.clip(center, end).is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    // AABB.intersects
    pub fn intersects(self, b: Self) -> bool {
        self.min.x < b.max.x && self.max.x > b.min.x && self.min.y < b.max.y && self.max.y > b.min.y && self.min.z < b.max.z && self.max.z > b.min.z
    }
}

#[derive(Clone, Debug, Default)]
pub struct VoxelShape {
    pub coords: [Vec<f64>; 3],
    full: Vec<bool>
}
impl VoxelShape {
    // Shapes.create; Shapes.or; BitSetDiscreteVoxelShape
    pub fn boxes(boxes: &[Aabb]) -> Self {
        let mut coords: [Vec<f64>; 3] = std::array::from_fn(|_| Vec::new());
        for b in boxes {
            for axis in [Axis::X, Axis::Y, Axis::Z] {
                coords[axis as usize].extend([b.min.get(axis), b.max.get(axis)]);
            }
        }
        for c in &mut coords {
            c.sort_by(f64::total_cmp);
            c.dedup();
        }
        let mut shape = Self {
            coords,
            full: Vec::new()
        };
        let [nx, ny, nz] = shape.sizes();
        for x in 0..nx {
            for y in 0..ny {
                for z in 0..nz {
                    shape.full.push(boxes.iter().any(|b|
                    shape.coords[0][x] >= b.min.x && shape.coords[0][x+1] <= b.max.x &&
                    shape.coords[1][y] >= b.min.y && shape.coords[1][y+1] <= b.max.y &&
                    shape.coords[2][z] >= b.min.z && shape.coords[2][z+1] <= b.max.z));
                }
            }
        }
        shape
    }

    // DiscreteVoxelShape.getSize
    fn sizes(&self) -> [usize; 3] {
        std::array::from_fn(|a| self.coords[a].len().saturating_sub(1))
    }

    // VoxelShape.isEmpty
    pub fn is_empty(&self) -> bool {
        !self.full.iter().any(|b| *b)
    }

    // VoxelShape.move
    pub fn moved(&self, v: Vec3) -> Self {
        Self {
            coords: std::array::from_fn(|a| self.coords[a].iter().map(|c| c + [v.x, v.y, v.z][a]).collect()),
            full: self.full.clone()
        }
    }

    // DiscreteVoxelShape.isFullWide
    fn is_full(&self, p: [usize; 3]) -> bool {
        let [nx, ny, nz] = self.sizes();
        p[0] < nx && p[1] < ny && p[2] < nz && self.full[(p[0]*ny + p[1])*nz + p[2]]
    }

    // VoxelShape.findIndex
    fn find_index(&self, axis: usize, value: f64) -> i32 {
        self.coords[axis].partition_point(|c| value >= *c) as i32 - 1
    }

    // VoxelShape.collideX
    pub fn collide(&self, axis: Axis, b: Aabb, mut distance: f64) -> f64 {
        if self.is_empty() {
            return distance;
        }
        if distance.abs() < 1.0e-7 {
            return 0.0;
        }
        let a = axis as usize;
        let u = (a + 1) % 3;
        let v = (a + 2) % 3;
        let axes = [Axis::X, Axis::Y, Axis::Z];
        let sizes = self.sizes();
        let lo_u = self.find_index(u, b.min.get(axes[u])+1.0e-7).max(0) as usize;
        let hi_u = (self.find_index(u, b.max.get(axes[u])-1.0e-7)+1).min(sizes[u] as i32).max(0) as usize;
        let lo_v = self.find_index(v, b.min.get(axes[v])+1.0e-7).max(0) as usize;
        let hi_v = (self.find_index(v, b.max.get(axes[v])-1.0e-7)+1).min(sizes[v] as i32).max(0) as usize;
        let start = if distance > 0.0 {
            self.find_index(a, b.max.get(axis)-1.0e-7)+1
        } else {
            self.find_index(a, b.min.get(axis)+1.0e-7)-1
        };
        let mut i = start;
        while i >= 0 && i < sizes[a] as i32 {
            for j in lo_u..hi_u {
                for k in lo_v..hi_v {
                    let mut p = [0; 3];
                    p[a] = i as usize;
                    p[u] = j;
                    p[v] = k;
                    if self.is_full(p) {
                        let delta = if distance > 0.0 {
                            self.coords[a][i as usize] - b.max.get(axis)
                        } else {
                            self.coords[a][i as usize+1] - b.min.get(axis)
                        };
                        if distance > 0.0 {
                            if delta >= -1.0e-7 {
                                distance = distance.min(delta);
                            }
                        } else if delta <= 1.0e-7 {
                            distance = distance.max(delta);
                        }
                        return distance;
                    }
                }
            }
            i += if distance > 0.0 {
                1
            } else {
                -1
            };
        }
        distance
    }

    // Shapes.block
    pub fn is_block(&self) -> bool {
        self.coords.iter().all(|c| c.as_slice()==[0.0, 1.0]) && self.full.as_slice()==[true]
    }

    // Shapes.joinIsNotEmpty; BooleanOp.AND
    pub fn intersects(&self, bounds: Aabb) -> bool {
        if self.is_empty() {
            return false;
        }
        let axes=[Axis::X, Axis::Y, Axis::Z];
        let indices: [Vec<usize>; 3]=std::array::from_fn(|a| overlap_indices(&self.coords[a], bounds.min.get(axes[a]), bounds.max.get(axes[a])));
        for x in &indices[0] {
            for y in &indices[1] {
                for z in &indices[2] {
                    if self.is_full([*x, *y, *z]) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

// IndirectMerger.<init>; IndirectMerger.forMergedIndexes
fn overlap_indices(first: &[f64], lower: f64, upper: f64) -> Vec<usize> {
    if upper-lower < 1.0e-7 {
        return Vec::new();
    }
    let second=[lower, upper];
    let mut i=0;
    let mut j=0;
    let mut previous=f64::NAN;
    let mut merged: Vec<(i32, i32)>=Vec::new();
    while i<first.len() || j<second.len() {
        let first_done=i>=first.len();
        let second_done=j>=second.len();
        let take_first=!first_done && (second_done || first[i]<second[j]+1.0e-7);
        let coordinate;
        if take_first {
            i+=1;
            if j==0 || second_done {
                continue;
            }
            coordinate=first[i-1];
        } else {
            j+=1;
            if i==0 || first_done {
                continue;
            }
            coordinate=second[j-1];
        }
        let pair=(i as i32-1, j as i32-1);
        if previous>=coordinate-1.0e-7 {
            *merged.last_mut().unwrap()=pair;
        } else {
            merged.push(pair);
            previous=coordinate;
        }
    }
    merged.iter().take(merged.len().saturating_sub(1))
    .filter_map(|&(a, b)| (a>=0 && b==0).then_some(a as usize)).collect()
}

// Entity.collideWithShapes; Shapes.collide
pub fn collide_with_shapes(wanted: Vec3, b: Aabb, shapes: &[VoxelShape]) -> Vec3 {
    if shapes.is_empty() {
        return wanted;
    }
    let mut result = Vec3::default();
    for axis in axis_step_order(wanted) {
        let mut d = wanted.get(axis);
        if d == 0.0 {
            continue;
        }
        let moved = b.moved(result);
        for shape in shapes {
            if d.abs() < 1.0e-7 {
                d = 0.0;
                break;
            }
            d = shape.collide(axis, moved, d);
        }
        result = result.with(axis, d);
    }
    result
}

// Entity.getInputVector
pub fn input_vector(input: Vec3, acceleration: f32, yaw: f32) -> Vec3 {
    let length = input.length_sqr();
    if length < 1.0e-7 {
        return Vec3::default();
    }
    let v = (if length > 1.0 {
        input.normalize()
    } else {
        input
    }).scale(acceleration as f64);
    let r = (yaw * 0.017453292f32) as f64;
    let s = math::sin(r) as f64;
    let c = math::cos(r) as f64;
    Vec3::new(v.x*c-v.z*s, v.y, v.z*c+v.x*s)
}
