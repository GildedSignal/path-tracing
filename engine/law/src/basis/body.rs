//! `affine_body`: the recurrence every free body runs between contacts. Three affine
//! operations, `Entity.applyGravity` (`v += (0, -g, 0)`), drag (`v *= d`) and move
//! (`p += v`), composed in the order the body's class ticks them. A body is a row of
//! parameters (gravity, air and water drag, order), never its own code; the irregular
//! parts (an arrow's split drag, an item's ground friction and rest cadence, a hit that
//! stops the move) are thin scars at the call site that still use these operations.
//! `tick` is the `k = 1` face; `run` takes `k` ticks of free flight.

use crate::Vec3;

/// The order a class composes gravity (G), drag (D) and move (M) in one tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// `FallingBlockEntity.tick`, `ItemEntity.tick`: G, M, D.
    GravityMoveDrag,
    /// `ThrowableProjectile.tick`: G, D, M (the move uses the post-drag velocity).
    GravityDragMove,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineBody {
    /// `getDefaultGravity`.
    pub gravity: f64,
    /// `getDefaultGravity` while in water, where the class changes it.
    pub water_gravity: f64,
    /// `getAirDrag`, widened to `f64` where it multiplies.
    pub air_drag: f32,
    /// `getWaterInertia` / `applyInertia`'s water branch.
    pub water_drag: f32,
    pub order: Order,
}

/// `FallingBlockEntity`: `getDefaultGravity 0-3` 0.04, `Entity.getAirDrag` 0.98f (`tick @893`).
pub const FALLING_BLOCK: AffineBody = AffineBody { gravity: 0.04, water_gravity: 0.04, air_drag: 0.98, water_drag: 0.98, order: Order::GravityMoveDrag };
/// `ThrowableProjectile` (snowball, egg, ender pearl): gravity 0.03, `getAirDrag` 0.99f,
/// `applyInertia @97-100` water 0.8f.
pub const THROWABLE: AffineBody = AffineBody { gravity: 0.03, water_gravity: 0.03, air_drag: 0.99, water_drag: 0.8, order: Order::GravityDragMove };
/// `AbstractArrow`: `getDefaultGravity` 0.05, `getAirDrag` 0.99f, `getWaterInertia` 0.6f
/// (its tick splits the drag around the move: the scar lives in `projectiles::arrow_free_tick`).
pub const ARROW: AffineBody = AffineBody { gravity: 0.05, water_gravity: 0.05, air_drag: 0.99, water_drag: 0.6, order: Order::GravityMoveDrag };
/// `ItemEntity`: `getDefaultGravity` 0.04, `Entity.getAirDrag` 0.98f, water `setUnderwaterMovement` 0.99f
/// (its fluids replace gravity and its ground friction scales x/z: scars in `items::tick`).
pub const ITEM: AffineBody = AffineBody { gravity: 0.04, water_gravity: 0.04, air_drag: 0.98, water_drag: 0.99, order: Order::GravityMoveDrag };
/// `AbstractMinecart`: `getDefaultGravity` 0.04, 0.005 in water; `comeOffTrack` air drag and
/// `applyNaturalSlowdown` water factor 0.95f (its rails and slowdown are scars in `minecart`).
pub const MINECART: AffineBody = AffineBody { gravity: 0.04, water_gravity: 0.005, air_drag: 0.95, water_drag: 0.95, order: Order::GravityMoveDrag };

impl AffineBody {
    /// `Entity.applyGravity`: `setDeltaMovement(getDeltaMovement().add(0.0, -g, 0.0))`.
    #[inline(always)]
    pub fn gravity(&self, v: &mut Vec3, in_water: bool) {
        *v = v.plus(Vec3::new(0.0, -if in_water { self.water_gravity } else { self.gravity }, 0.0));
    }

    #[inline(always)]
    pub fn drag(&self, v: &mut Vec3, in_water: bool) {
        *v = v.scale(if in_water { self.water_drag } else { self.air_drag } as f64);
    }

    /// One free tick (`k = 1`).
    #[inline(always)]
    pub fn tick(&self, p: &mut Vec3, v: &mut Vec3, in_water: bool) {
        self.gravity(v, in_water);
        match self.order {
            Order::GravityMoveDrag => {
                *p = p.plus(*v);
                self.drag(v, in_water);
            }
            Order::GravityDragMove => {
                self.drag(v, in_water);
                *p = p.plus(*v);
            }
        }
    }

    /// `k` free ticks; the caller certifies no contact inside them.
    pub fn run(&self, p: &mut Vec3, v: &mut Vec3, in_water: bool, k: u32) {
        for _ in 0..k {
            self.tick(p, v, in_water);
        }
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
