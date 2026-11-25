//! `body`: the recurrence every body runs. Per tick, velocity += input + gravity, move
//! with `sweep`, velocity × drag (the horizontal drag times the friction of the block
//! under it). A body is a row of parameters (gravity, drags, the order Java composes
//! them in), never its own code; where a class breaks the pattern (an arrow's drag
//! around the move, an item's rest cadence) a thin scar at the call site uses these
//! operations. `tick` is the `k = 1` face of free flight; `run` takes `k` ticks.

use crate::Vec3;

/// `BubbleColumnBlock.entityInside`: open air uses the above-column cap;
/// submersion uses the lower cap and clears fall distance.
#[inline(always)]
pub fn bubble_column_contact(
    s: &mut crate::PlayerState, world: &impl crate::World, p: crate::BlockPos,
    drag: &bool, intersects: bool,
) {
    if intersects && !s.flying {
        let above = world.block([p[0], p[1] + 1, p[2]]);
        let vy = s.velocity.y;
        if above.shape.is_empty() && above.fluid.kind == crate::FluidKind::Empty {
            s.velocity.y = if *drag { (-0.9f64).max(vy - 0.03) } else { 1.8f64.min(vy + 0.1) };
        } else {
            s.velocity.y = if *drag { (-0.3f64).max(vy - 0.03) } else { 0.7f64.min(vy + 0.06) };
            s.fall_distance = 0.0;
        }
    }
}
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

pub(crate) enum Medium {
    Steady(bool),
    /// Move with the entry velocity and apply drag based on both fluid samples.
    Crossing { water_before: bool, water_after: bool },
}

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

    /// Advance a body, handing the contact rule the velocity at the move boundary.
    #[inline]
    pub(crate) fn advance<R>(
        &self,
        velocity: &mut Vec3,
        medium: Medium,
        move_body: impl FnOnce(Vec3) -> R,
    ) -> R {
        match medium {
            Medium::Steady(water) => {
                if self.order != Order::MoveGravityDrag {
                    self.gravity(velocity, water);
                }
                if self.order == Order::GravityDragMove {
                    self.drag(velocity, water);
                }
                let moved = move_body(*velocity);
                if self.order == Order::MoveGravityDrag {
                    self.gravity(velocity, water);
                }
                if self.order != Order::GravityDragMove {
                    self.drag(velocity, water);
                }
                moved
            }
            Medium::Crossing { water_before, water_after } => {
                let entry = *velocity;
                if water_before {
                    self.drag(velocity, true);
                }
                let moved = move_body(entry);
                if !water_after {
                    self.drag(velocity, false);
                }
                self.gravity(velocity, false);
                moved
            }
        }
    }

    /// One free tick (`k = 1`).
    #[inline(always)]
    pub fn tick(&self, p: &mut Vec3, v: &mut Vec3, in_water: bool) {
        *p = self.advance(v, Medium::Steady(in_water), |delta| p.plus(delta));
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

/// A body's first velocity and facing from an aim (`Projectile.shoot`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Launch { pub velocity: Vec3, pub y_rot: f32, pub x_rot: f32 }

/// `RandomSource.triangle(0.0, 0.0172275 * inaccuracy)` on x, y, z (6 `nextDouble` draws).
pub fn spread(inaccuracy: f32, draws: [f64; 6]) -> Vec3 {
    let extent = 0.0172275 * (inaccuracy as f64);
    Vec3::new(0.0 + extent * (draws[0] - draws[1]), 0.0 + extent * (draws[2] - draws[3]), 0.0 + extent * (draws[4] - draws[5]))
}

/// `Projectile.getMovementToShoot`: `(direction.normalize() + spread) * power`; the add
/// is kept at zero spread, so `-0.0` becomes `+0.0`.
#[inline]
pub fn shoot(direction: Vec3, power: f32, spread: Vec3) -> Vec3 { direction.normalize().plus(spread).scale(power as f64) }

/// `Projectile.shoot @25-88`: the facing `(yRot, xRot)` of a shot velocity, through `Mth.atan2`.
pub fn facing(v: Vec3) -> (f32, f32) {
    let deg = (180.0f32 / (std::f64::consts::PI as f32)) as f64;
    ((crate::math::atan2(v.x, v.z) * deg) as f32, (crate::math::atan2(v.y, v.horizontal_sqr().sqrt()) * deg) as f32)
}

/// `Projectile.shootFromRotation @0-113`: the shot along the aim (the roll offsets the
/// pitch of the vertical only), plus the thrower's known movement (its y only in the air).
pub fn shoot_from_rotation(pitch: f32, yaw: f32, roll: f32, power: f32, spread: Vec3, known: Vec3, on_ground: bool) -> Vec3 {
    let (y, p, r) = ((yaw * 0.017453292f32) as f64, (pitch * 0.017453292f32) as f64, ((pitch + roll) * 0.017453292f32) as f64);
    use crate::math::{cos, sin};
    let direction = Vec3::new((-sin(y) * cos(p)) as f64, (-sin(r)) as f64, (cos(y) * cos(p)) as f64);
    shoot(direction, power, spread).plus(Vec3::new(known.x, if on_ground { 0.0 } else { known.y }, known.z))
}

/// The launch of every projectile row: the shot, its facing taken before the thrower's
/// movement is added, then the velocity with it.
pub fn launch(pitch: f32, yaw: f32, roll: f32, power: f32, inaccuracy: f32, draws: [f64; 6], known: Vec3, on_ground: bool) -> Launch {
    let s = spread(inaccuracy, draws);
    let (y_rot, x_rot) = facing(shoot_from_rotation(pitch, yaw, roll, power, s, Vec3::default(), true));
    Launch { velocity: shoot_from_rotation(pitch, yaw, roll, power, s, known, on_ground), y_rot, x_rot }
}
