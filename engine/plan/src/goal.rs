//! The goal is a region, and the lower bound nothing can beat.
use law::{Aabb, BlockPos, PlayerState, Vec3};

/// The terrain speed limit c, in blocks per tick: 19.902886 blocks per second, a 3-tick
/// orbit on blue ice under a 2.0 ceiling (HANDOFF anomaly 3; GLM local-20261110-1056-1-4,
/// reproduced). It is the bound's premise, not a number of the planner.
pub const C: f64 = 19.902886 / 20.0;

/// Touch B: the track detector's rule (bot/src/pathtracing/TrackDetector.java). A sample
/// stops the clock when the box enclosing the body before and after the tick meets B's cell
/// grown by 1e-3. Standing on B is inside this region.
#[derive(Clone, Copy, Debug)]
pub struct Goal { pub cell: BlockPos }
impl Goal {
    pub fn zone(&self) -> Aabb {
        let [x, y, z] = self.cell.map(|v| v as f64);
        Aabb::new(x - 1e-3, y - 1e-3, z - 1e-3, x + 1.0 + 1e-3, y + 1.0 + 1e-3, z + 1.0 + 1e-3)
    }
    pub fn touched(&self, before: &PlayerState, after: &PlayerState) -> bool {
        hull(before.bounding_box(), after.bounding_box()).intersects(self.zone())
    }
    /// The horizontal distance from the body's box to the region.
    pub fn gap(&self, state: &PlayerState) -> f64 {
        let (body, zone) = (state.bounding_box(), self.zone());
        let dx = (zone.min.x - body.max.x).max(body.min.x - zone.max.x).max(0.0);
        let dz = (zone.min.z - body.max.z).max(body.min.z - zone.max.z).max(0.0);
        (dx * dx + dz * dz).sqrt()
    }
    /// Ticks no sentence can beat. Premises: the body's box moves at most `C` blocks per tick
    /// horizontally (terrain only: no entity push, no knockback, no item); the goal is this
    /// region. The quotient is rounded down one ulp before the ceiling, so rounding cannot
    /// raise the bound. It must be re-derived when the law changes (learning.md 2.7).
    pub fn lower_bound(&self, state: &PlayerState) -> u32 {
        let quotient = self.gap(state) / C;
        if quotient <= 0.0 { 0 } else { f64::from_bits(quotient.to_bits() - 1).ceil() as u32 }
    }
    pub fn center(&self) -> Vec3 { Vec3::new(self.cell[0] as f64 + 0.5, self.cell[1] as f64 + 1.0, self.cell[2] as f64 + 0.5) }
}
pub fn hull(a: Aabb, b: Aabb) -> Aabb {
    Aabb { min: Vec3::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y), a.min.z.min(b.min.z)), max: Vec3::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y), a.max.z.max(b.max.z)) }
}
