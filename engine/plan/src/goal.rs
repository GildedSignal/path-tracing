//! The goal is a region, and the lower bound nothing can beat.
use law::{Aabb, BlockPos, Vec3};

/// A face of B: the law's `Direction`, in Mojang's 3D-data order (down, up, north, south,
/// west, east), which is also its wire value.
pub use law::neighbor::Direction as Face;

/// The terrain speed limit c, in blocks per tick: 19.902886 blocks per second, a 3-tick
/// orbit on blue ice under a 2.0 ceiling (HANDOFF anomaly 3; GLM local-20261110-1056-1-4,
/// reproduced). It is the bound's premise, not a number of the planner.
pub const C: f64 = 19.902886 / 20.0;

/// The axis (0 x, 1 y, 2 z) of each face in 3D-data order; an even face is the cell's low
/// plane on it, an odd face its high plane.
const AXIS: [usize; 6] = [1, 1, 2, 2, 0, 0];

/// The face named as in block states and track definitions (`"west"`).
pub fn face_named(name: &str) -> Result<Face, String> {
    law::basis::cell::DIRECTION.iter().position(|&d| d == name).map(|i| Face::from_3d_data(i as u8)).ok_or_else(|| format!("invalid B face: {name}"))
}

/// Touch B: the track detector's rule (bot/src/pathtracing/TrackDetector.java). A sample
/// stops the clock when the box enclosing the body before and after the tick meets a face of
/// B: closed on the face's plane, open on its two tangent axes, without epsilon. Standing
/// exactly on B touches its top; an edge or a corner alone touches nothing. A selected face
/// is the only face that counts; `None` is any face.
#[derive(Clone, Copy, Debug)]
pub struct Goal { pub cell: BlockPos, pub face: Option<Face> }
impl Goal {
    /// B, any face.
    pub const fn of(cell: BlockPos) -> Self { Self { cell, face: None } }
    /// The region to reach: B's cell, or the selected face's square.
    pub fn zone(&self) -> Aabb {
        let [x, y, z] = self.cell.map(|v| v as f64);
        let (mut lo, mut hi) = ([x, y, z], [x + 1.0, y + 1.0, z + 1.0]);
        if let Some(f) = self.face {
            let (i, a) = (f as usize, AXIS[f as usize]);
            if i & 1 == 0 { hi[a] = lo[a] } else { lo[a] = hi[a] }
        }
        Aabb::new(lo[0], lo[1], lo[2], hi[0], hi[1], hi[2])
    }
    pub fn touched(&self, before: &impl crate::state_view::StateSource, after: &impl crate::state_view::StateSource) -> bool {
        self.touched_boxes(before.view().bounding_box(),after.view().bounding_box())
    }
    /// The same predicate for exact generator readouts, without copying states.
    pub fn touched_boxes(&self,before:Aabb,after:Aabb)->bool {
        let h = hull(before, after);
        let (lo, hi, c) = ([h.min.x, h.min.y, h.min.z], [h.max.x, h.max.y, h.max.z], self.cell.map(|v| v as f64));
        let open = |t: usize| lo[t] < c[t] + 1.0 && hi[t] > c[t];
        let face = |i: usize| {
            let a = AXIS[i];
            let plane = c[a] + (i & 1) as f64;
            lo[a] <= plane && hi[a] >= plane && open((a + 1) % 3) && open((a + 2) % 3)
        };
        match self.face { Some(f) => face(f as usize), None => (0..6).any(face) }
    }
    /// The two closed half-space predicates of the zone on one axis. A touch implies both on
    /// every axis, and they only grow with the hull, so exact preimages bisect them.
    pub fn axis_sides(&self,before:Aabb,after:Aabb,axis:usize)->(bool,bool) {
        let a=hull(before,after);let b=self.zone();let axis=[law::Axis::X,law::Axis::Y,law::Axis::Z][axis];
        (a.max.get(axis)>=b.min.get(axis),a.min.get(axis)<=b.max.get(axis))
    }
    /// The horizontal distance from the body's box to the region.
    pub fn gap(&self, state: &impl crate::state_view::StateSource) -> f64 {
        let (body, zone) = (state.view().bounding_box(), self.zone());
        let dx = (zone.min.x - body.max.x).max(body.min.x - zone.max.x).max(0.0);
        let dz = (zone.min.z - body.max.z).max(body.min.z - zone.max.z).max(0.0);
        (dx * dx + dz * dz).sqrt()
    }
    /// Ticks no sentence can beat. Premises: the body's box moves at most `C` blocks per tick
    /// horizontally (terrain only: no entity push, no knockback, no item); the goal is this
    /// region. The quotient is rounded down one ulp before the ceiling, so rounding cannot
    /// raise the bound. It must be re-derived when the law changes (learning.md 2.7).
    pub fn lower_bound(&self, state: &impl crate::state_view::StateSource) -> u32 {
        let quotient = self.gap(state) / C;
        if quotient <= 0.0 { 0 } else { f64::from_bits(quotient.to_bits() - 1).ceil() as u32 }
    }
    /// Where to aim: the top's centre for any face, else the selected square's centre.
    pub fn center(&self) -> Vec3 {
        if self.face.is_none() { return Vec3::new(self.cell[0] as f64 + 0.5, self.cell[1] as f64 + 1.0, self.cell[2] as f64 + 0.5); }
        let z = self.zone();
        Vec3::new((z.min.x + z.max.x) / 2.0, (z.min.y + z.max.y) / 2.0, (z.min.z + z.max.z) / 2.0)
    }
}
pub fn hull(a: Aabb, b: Aabb) -> Aabb {
    Aabb { min: Vec3::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y), a.min.z.min(b.min.z)), max: Vec3::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y), a.max.z.max(b.max.z)) }
}
