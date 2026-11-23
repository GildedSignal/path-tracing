//! `body`: the recurrence every body runs. Per tick, velocity += input + gravity, move
//! with `sweep`, velocity × drag (the horizontal drag times the friction of the block
//! under it). A body is a row of parameters (gravity, drags, the order Java composes
//! them in), never its own code; where a class breaks the pattern (an arrow's drag
//! around the move, an item's rest cadence) a thin scar at the call site uses these
//! operations. `tick` is the `k = 1` face of free flight; `run` takes `k` ticks.

use crate::Vec3;

/// The order a class composes gravity (G), drag (D) and move (M) in one tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// `FallingBlockEntity.tick`, `ItemEntity.tick`: G, M, D.
    GravityMoveDrag,
    /// `ThrowableProjectile.tick`: G, D, M (the move uses the post-drag velocity).
    GravityDragMove,
    /// `LivingEntity.travelInAir`: M, G, D (the input is added before the move).
    MoveGravityDrag,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineBody {
    /// `getDefaultGravity`.
    pub gravity: f64,
    /// `getDefaultGravity` while in water, where the class changes it.
    pub water_gravity: f64,
    /// `getAirDrag` (horizontal, for a living body), widened to `f64` where it multiplies.
    pub air_drag: f32,
    /// The vertical air drag: `air_drag`, except 0.98f for a living body.
    pub drag_y: f32,
    /// `getWaterInertia` / `applyInertia`'s water branch.
    pub water_drag: f32,
    pub order: Order,
}

/// `FallingBlockEntity`: `getDefaultGravity 0-3` 0.04, `Entity.getAirDrag` 0.98f (`tick @893`).
pub const FALLING_BLOCK: AffineBody = AffineBody { gravity: 0.04, water_gravity: 0.04, air_drag: 0.98, drag_y: 0.98, water_drag: 0.98, order: Order::GravityMoveDrag };
/// `ThrowableProjectile` (snowball, egg, ender pearl): gravity 0.03, `getAirDrag` 0.99f,
/// `applyInertia @97-100` water 0.8f.
pub const THROWABLE: AffineBody = AffineBody { gravity: 0.03, water_gravity: 0.03, air_drag: 0.99, drag_y: 0.99, water_drag: 0.8, order: Order::GravityDragMove };
/// `AbstractArrow`: `getDefaultGravity` 0.05, `getAirDrag` 0.99f, `getWaterInertia` 0.6f
/// (its tick splits the drag around the move: the scar lives in `projectiles::arrow_free_tick`).
pub const ARROW: AffineBody = AffineBody { gravity: 0.05, water_gravity: 0.05, air_drag: 0.99, drag_y: 0.99, water_drag: 0.6, order: Order::GravityMoveDrag };
/// `ItemEntity`: `getDefaultGravity` 0.04, `Entity.getAirDrag` 0.98f, water `setUnderwaterMovement` 0.99f
/// (its fluids replace gravity and its rest cadence skips the move: scars in `items::tick`).
pub const ITEM: AffineBody = AffineBody { gravity: 0.04, water_gravity: 0.04, air_drag: 0.98, drag_y: 0.98, water_drag: 0.99, order: Order::GravityMoveDrag };
/// `AbstractMinecart`: `getDefaultGravity` 0.04, 0.005 in water; `comeOffTrack` air drag and
/// `applyNaturalSlowdown` water factor 0.95f (its rails and slowdown are scars in `minecart`).
pub const MINECART: AffineBody = AffineBody { gravity: 0.04, water_gravity: 0.005, air_drag: 0.95, drag_y: 0.95, water_drag: 0.95, order: Order::GravityMoveDrag };
/// `LivingEntity.travelInAir`: `Attributes.GRAVITY` 0.08, horizontal drag 0.91f times the ground
/// friction, vertical 0.98f, both after the move (`AIR_DRAG_MODIFIER` scales the two drags).
pub const LIVING: AffineBody = AffineBody { gravity: 0.08, water_gravity: 0.08, air_drag: 0.91, drag_y: 0.98, water_drag: 0.8, order: Order::MoveGravityDrag };

impl AffineBody {
    /// `Entity.applyGravity`: `setDeltaMovement(getDeltaMovement().add(0.0, -g, 0.0))`.
    #[inline(always)]
    pub fn gravity(&self, v: &mut Vec3, in_water: bool) {
        *v = v.plus(Vec3::new(0.0, -if in_water { self.water_gravity } else { self.gravity }, 0.0));
    }

    #[inline(always)]
    pub fn drag(&self, v: &mut Vec3, in_water: bool) {
        *v = if in_water { v.scale(self.water_drag as f64) } else { self.friction_drag(*v, v.y, 1.0) };
    }

    /// The drag with the ground under the body: horizontal × (`friction` × the air
    /// drag), the vertical `vy` × the vertical drag; `friction` is 1 off the ground.
    #[inline(always)]
    pub fn friction_drag(&self, v: Vec3, vy: f64, friction: f32) -> Vec3 {
        let h = (friction * self.air_drag) as f64;
        Vec3::new(v.x * h, vy * self.drag_y as f64, v.z * h)
    }

    /// One free tick (`k = 1`).
    #[inline(always)]
    pub fn tick(&self, p: &mut Vec3, v: &mut Vec3, in_water: bool) {
        match self.order {
            Order::GravityMoveDrag => { self.gravity(v, in_water); *p = p.plus(*v); self.drag(v, in_water); }
            Order::GravityDragMove => { self.gravity(v, in_water); self.drag(v, in_water); *p = p.plus(*v); }
            Order::MoveGravityDrag => { *p = p.plus(*v); self.gravity(v, in_water); self.drag(v, in_water); }
        }
    }

    /// `k` free ticks; the caller certifies no contact inside them.
    pub fn run(&self, p: &mut Vec3, v: &mut Vec3, in_water: bool, k: u32) {
        for _ in 0..k { self.tick(p, v, in_water); }
    }
}

/// `Entity.getInputVector`: the input, normalised past length 1, scaled by the
/// acceleration and turned by the yaw through the sine table.
pub fn input_vector(input: Vec3, acceleration: f32, yaw: f32) -> Vec3 {
    let length = input.length_sqr();
    if length < 1.0e-7 { return Vec3::default(); }
    let v = (if length > 1.0 { input.normalize() } else { input }).scale(acceleration as f64);
    let r = (yaw * 0.017453292f32) as f64;
    let (s, c) = (crate::math::sin(r) as f64, crate::math::cos(r) as f64);
    Vec3::new(v.x * c - v.z * s, v.y, v.z * c + v.x * s)
}

/// `Entity.move` after the sweep, for every body: what it wanted and what the sweep
/// gave, the position rule and the collision flags (`Mth.equal` on x and z, exact on y).
#[derive(Clone, Copy, Debug)]
pub struct Moved { pub wanted: Vec3, pub moved: Vec3, pub x: bool, pub z: bool, pub y: bool }
impl Moved {
    #[inline(always)]
    pub fn new(wanted: Vec3, moved: Vec3) -> Self {
        Self { wanted, moved, x: !crate::math::equal(wanted.x, moved.x), z: !crate::math::equal(wanted.z, moved.z), y: wanted.y != moved.y }
    }
    /// `setPos` runs when the body moved more than `1e-7`² or nearly all it wanted.
    #[inline(always)]
    pub fn applies(&self) -> bool {
        let d = self.moved.length_sqr();
        d > 1.0e-7 || self.wanted.length_sqr() - d < 1.0e-7
    }
    #[inline(always)]
    pub fn horizontal(&self) -> bool { self.x || self.z }
    /// `verticalCollisionBelow`, the body's `onGround`.
    #[inline(always)]
    pub fn below(&self) -> bool { self.y && self.wanted.y < 0.0 }
    /// `restituteMovementAfterCollisions`: a hit axis is reflected by `bounce`; the
    /// vertical one keeps the fraction of gravity and drag the move used when it bounces.
    #[inline(always)]
    pub fn restitute(&self, v: &mut Vec3, bounce: f64, vertical_bounce: f64, factors: impl FnOnce() -> (f64, f32)) {
        if !(self.wanted.y.abs() > 0.0 && self.y || self.horizontal()) { return; }
        let before = *v;
        if self.x { v.x = -before.x * bounce; }
        if self.z { v.z = -before.z * bounce; }
        if self.y {
            let (g, d) = if vertical_bounce > 0.0 {
                let fraction = self.moved.y / before.y;
                let (gravity, drag_y) = factors();
                (fraction * gravity, 1.0 + fraction * (drag_y as f64 - 1.0))
            } else { (0.0, 1.0) };
            if !(cfg!(feature = "mutant-ceiling") && self.wanted.y > 0.0) { v.y = (g - before.y) * d * vertical_bounce; }
        }
    }
}

/// `Entity.move` for a body that is not living (boat, falling block, mob without AI):
/// `moved` is the sweep of `*v`; the position follows the rule and the hit axes stop
/// (bounciness 0). Returns the flags.
pub fn settle(p: &mut Vec3, v: &mut Vec3, moved: Vec3) -> Moved {
    let m = Moved::new(*v, moved);
    if m.applies() { *p = p.plus(moved); }
    m.restitute(v, 0.0, 0.0, || (0.0, 1.0));
    m
}
