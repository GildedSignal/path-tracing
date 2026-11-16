//! The goal is a region, and the lower bound nothing can beat.
use law::{Aabb, BlockPos, PlayerState, Vec3};

/// The terrain speed limit c, in blocks per tick: 19.902886 blocks per second, a 3-tick
/// orbit on blue ice under a 2.0 ceiling (HANDOFF anomaly 3; GLM local-20261110-1056-1-4,
/// reproduced). It is the bound's premise, not a number of the planner.
pub const C: f64 = 19.902886 / 20.0;

/// Touch B using the same closed plane and open tangents as the client detector.
#[derive(Clone, Copy, Debug)]
pub struct Goal { pub cell: BlockPos, pub face: Option<Face> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face { West, East, Down, Up, North, South }
impl Face {
    pub fn parse(s: &str) -> Result<Self, String> { match s {
        "west" => Ok(Self::West), "east" => Ok(Self::East), "down" => Ok(Self::Down), "up" => Ok(Self::Up), "north" => Ok(Self::North), "south" => Ok(Self::South), _ => Err(format!("invalid B face: {s}")),
    }}
    fn index(self) -> usize { match self { Self::West => 0, Self::East => 1, Self::Down => 2, Self::Up => 3, Self::North => 4, Self::South => 5 } }
}
impl Goal {
    pub fn zone(&self) -> Aabb {
        let [x, y, z] = self.cell.map(|v| v as f64);
        Aabb::new(x, y, z, x + 1.0, y + 1.0, z + 1.0)
    }
    pub fn touched(&self, before: &PlayerState, after: &PlayerState) -> bool {
        self.touches_box(hull(before.bounding_box(), after.bounding_box()))
    }
    pub fn touches_box(&self, b: Aabb) -> bool {
        let lo = [b.min.x, b.min.y, b.min.z];
        let hi = [b.max.x, b.max.y, b.max.z];
        (0..6).any(|i| {
            if self.face.is_some_and(|f| f.index() != i) { return false; }
            let axis = i / 2;
            let plane = self.cell[axis] as f64 + (i % 2) as f64;
            lo[axis] <= plane && hi[axis] >= plane && (0..3).filter(|&t| t != axis)
                .all(|t| lo[t] < self.cell[t] as f64 + 1.0 && hi[t] > self.cell[t] as f64)
        })
    }
    fn gaps(&self, state: &PlayerState) -> (f64, f64, f64) {
        let (body, mut zone) = (state.bounding_box(), self.zone());
        if let Some(face) = self.face { match face {
            Face::West => zone.max.x = zone.min.x, Face::East => zone.min.x = zone.max.x,
            Face::Down => zone.max.y = zone.min.y, Face::Up => zone.min.y = zone.max.y,
            Face::North => zone.max.z = zone.min.z, Face::South => zone.min.z = zone.max.z,
        }}
        let dx = (zone.min.x - body.max.x).max(body.min.x - zone.max.x).max(0.0);
        let dy = (zone.min.y - body.max.y).max(body.min.y - zone.max.y).max(0.0);
        let dz = (zone.min.z - body.max.z).max(body.min.z - zone.max.z).max(0.0);
        (dx, dy, dz)
    }
    /// Distance from the body to the selected face square (or cell for legacy tracks).
    pub fn gap(&self, state: &PlayerState) -> f64 {
        let (dx, dy, dz) = self.gaps(state);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    /// Ticks no sentence can beat. Premises: the body's box moves at most `C` blocks per tick
    /// horizontally (terrain only: no entity push, no knockback, no item); the goal is this
    /// region. The quotient is rounded down one ulp before the ceiling, so rounding cannot
    /// raise the bound. It must be re-derived when the law changes (learning.md 2.7).
    pub fn lower_bound(&self, state: &PlayerState) -> u32 {
        let (dx, _, dz) = self.gaps(state);
        let quotient = (dx * dx + dz * dz).sqrt() / C;
        if quotient <= 0.0 { 0 } else { f64::from_bits(quotient.to_bits() - 1).ceil() as u32 }
    }
    pub fn center(&self) -> Vec3 {
        let [x, y, z] = self.cell.map(|v| v as f64);
        match self.face {
            Some(Face::West) => Vec3::new(x, y + 0.5, z + 0.5),
            Some(Face::East) => Vec3::new(x + 1.0, y + 0.5, z + 0.5),
            Some(Face::Down) => Vec3::new(x + 0.5, y, z + 0.5),
            Some(Face::Up) | None => Vec3::new(x + 0.5, y + 1.0, z + 0.5),
            Some(Face::North) => Vec3::new(x + 0.5, y + 0.5, z),
            Some(Face::South) => Vec3::new(x + 0.5, y + 0.5, z + 1.0),
        }
    }
}
pub fn hull(a: Aabb, b: Aabb) -> Aabb {
    Aabb { min: Vec3::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y), a.min.z.min(b.min.z)), max: Vec3::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y), a.max.z.max(b.max.z)) }
}
