//! Mojang 26.2 explosion exposure/knockback (`ServerExplosion`,
//! `ExplosionDamageCalculator`, `SimpleExplosionDamageCalculator`) and wind charge
//! projectile throw/flight/explosion (`WindChargeItem`, `WindCharge`,
//! `AbstractWindCharge`, `Projectile.shootFromRotation`).
//!
//! Verified directly against `server-26.2.jar` bytecode + `CombatExplosionWitness.java`
//! (`tests/data/combat_explosion.tsv`).

use crate::neighbor::Direction;
use crate::{math, Aabb, Vec3};

pub const MIN_EXPLOSION_RADIUS: f32 = 1.0e-5;
pub const WIND_CHARGE_SHOOT_POWER: f32 = 1.5;
pub const WIND_CHARGE_INACCURACY: f32 = 1.0;
pub const WIND_CHARGE_COOLDOWN_SECONDS: f32 = 0.5;
pub const WIND_CHARGE_COOLDOWN_TICKS: i32 = 10;
pub const WIND_CHARGE_NO_DEFLECT_TICKS: i32 = 5;
pub const WIND_CHARGE_RADIUS: f32 = 1.2;
pub const WIND_CHARGE_KB_MULTIPLIER: f32 = 1.22;
pub const WIND_CHARGE_WIDTH: f32 = 0.3125;
pub const WIND_CHARGE_HEIGHT: f32 = 0.3125;
pub const WIND_CHARGE_AABB_Y_OFFSET: f64 = 0.15f32 as f64;
pub const WIND_CHARGE_HIT_BLOCK_OFFSET: f64 = 0.25;
pub const WIND_CHARGE_DIRECT_HIT_DAMAGE: f32 = 1.0;
pub const WIND_CHARGE_MAX_Y_MARGIN: i32 = 30;
pub const PROJECTILE_SPREAD_SCALE: f64 = 0.0172275;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplosionProfile {
    pub radius: f32,
    pub damages_entities: bool,
    pub knockback_multiplier: f32,
    /// True in `SimpleExplosionDamageCalculator`: flying players get `knockback_multiplier = 0.0`.
    pub zero_kb_for_flying_player: bool,
}

impl ExplosionProfile {
    pub const fn tnt() -> Self {
        Self {
            radius: 4.0,
            damages_entities: true,
            knockback_multiplier: 1.0,
            zero_kb_for_flying_player: false,
        }
    }

    pub const fn creeper(charged: bool) -> Self {
        Self {
            radius: if charged { 6.0 } else { 3.0 },
            damages_entities: true,
            knockback_multiplier: 1.0,
            zero_kb_for_flying_player: false,
        }
    }

    pub const fn end_crystal() -> Self {
        Self {
            radius: 6.0,
            damages_entities: true,
            knockback_multiplier: 1.0,
            zero_kb_for_flying_player: false,
        }
    }

    /// `WindCharge.EXPLOSION_DAMAGE_CALCULATOR`: `SimpleExplosionDamageCalculator(true, false, Optional.of(1.22f), ...)`, `RADIUS = 1.2f`.
    pub const fn wind_charge() -> Self {
        Self {
            radius: WIND_CHARGE_RADIUS,
            damages_entities: false,
            knockback_multiplier: WIND_CHARGE_KB_MULTIPLIER,
            zero_kb_for_flying_player: true,
        }
    }
}

/// `ServerExplosion.hurtEntities @11-149`: `Mth.floor(center ± (radius * 2) ± 1)`.
#[inline]
pub fn explosion_search_aabb(center: Vec3, radius: f32) -> Option<Aabb> {
    if radius < MIN_EXPLOSION_RADIUS {
        return None;
    }
    let diameter = (radius * 2.0f32) as f64;
    Some(Aabb::new(
        math::floor(center.x - diameter - 1.0) as f64,
        math::floor(center.y - diameter - 1.0) as f64,
        math::floor(center.z - diameter - 1.0) as f64,
        math::floor(center.x + diameter + 1.0) as f64,
        math::floor(center.y + diameter + 1.0) as f64,
        math::floor(center.z + diameter + 1.0) as f64,
    ))
}

/// `ServerExplosion.getSeenPercent(Vec3 center, Entity) @0-294`:
/// Samples a regular grid over `bb` with ray origin `(lerp(x), lerp(y), lerp(z)) + (xOff, 0, zOff)`
/// to `center`; `ray_misses(sample_point, center)` is true on `ClipContext.Block.COLLIDER, Fluid.NONE` MISS.
pub fn seen_percent(center: Vec3, bb: Aabb, mut ray_misses: impl FnMut(Vec3, Vec3) -> bool) -> f32 {
    let step_x = 1.0 / ((bb.max.x - bb.min.x) * 2.0 + 1.0);
    let step_y = 1.0 / ((bb.max.y - bb.min.y) * 2.0 + 1.0);
    let step_z = 1.0 / ((bb.max.z - bb.min.z) * 2.0 + 1.0);
    let x_off = (1.0 - (1.0 / step_x).floor() * step_x) / 2.0;
    let z_off = (1.0 - (1.0 / step_z).floor() * step_z) / 2.0;
    if step_x < 0.0 || step_y < 0.0 || step_z < 0.0 {
        return 0.0;
    }
    let mut missed = 0i32;
    let mut total = 0i32;
    let mut fx = 0.0f64;
    while fx <= 1.0 {
        let mut fy = 0.0f64;
        while fy <= 1.0 {
            let mut fz = 0.0f64;
            while fz <= 1.0 {
                let sample = Vec3::new(
                    math::lerp(fx, bb.min.x, bb.max.x) + x_off,
                    math::lerp(fy, bb.min.y, bb.max.y),
                    math::lerp(fz, bb.min.z, bb.max.z) + z_off,
                );
                if ray_misses(sample, center) {
                    missed += 1;
                }
                total += 1;
                fz += step_z;
            }
            fy += step_y;
        }
        fx += step_x;
    }
    missed as f32 / total as f32
}

/// `ExplosionDamageCalculator.getEntityDamageAmount @0-65`.
#[inline]
pub fn entity_damage_amount(radius: f32, dist_to_center_sqr: f64, seen_percent: f32) -> f32 {
    let diameter = radius * 2.0f32;
    let dist_ratio = dist_to_center_sqr.sqrt() / diameter as f64;
    let impact = (1.0 - dist_ratio) * seen_percent as f64;
    ((impact * impact + impact) / 2.0 * 7.0 * diameter as f64 + 1.0) as f32
}

#[derive(Clone, Copy, Debug)]
pub struct ExplosionTarget {
    pub feet_pos: Vec3,
    pub eye_y: f64,
    pub bb: Aabb,
    pub is_living: bool,
    pub is_player: bool,
    pub is_spectator: bool,
    pub is_creative_flying: bool,
    pub is_flying_ability: bool,
    pub is_primed_tnt: bool,
    pub ignore_explosion: bool,
    pub explosion_knockback_resistance: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplosionHit {
    pub damage: f32,
    pub knockback_impulse: Vec3,
    /// Added to `ServerExplosion.hitPlayers` (transmitted via `ClientboundExplodePacket`).
    pub record_player_impulse: bool,
}

/// Exact `ServerExplosion.hurtEntities @185-515` evaluation for one candidate entity.
/// `ray_misses` is only invoked if `damages_entities || effective_kb_mult != 0.0` (`@284-308`).
pub fn explosion_entity_effect(
    profile: ExplosionProfile,
    center: Vec3,
    target: ExplosionTarget,
    ray_misses: impl FnMut(Vec3, Vec3) -> bool,
) -> Option<ExplosionHit> {
    if profile.radius < MIN_EXPLOSION_RADIUS || target.ignore_explosion {
        return None;
    }
    let diameter = (profile.radius * 2.0f32) as f64;
    let dist_sqr = target.feet_pos.minus(center).length_sqr();
    let dist_ratio = dist_sqr.sqrt() / diameter;
    if dist_ratio > 1.0 {
        return None;
    }

    let dir_origin = if target.is_primed_tnt {
        target.feet_pos
    } else {
        Vec3::new(target.feet_pos.x, target.eye_y, target.feet_pos.z)
    };
    let dir = dir_origin.minus(center).normalize();

    let kb_mult = if profile.zero_kb_for_flying_player && target.is_player && target.is_flying_ability {
        0.0f32
    } else {
        profile.knockback_multiplier
    };

    let exposure = if profile.damages_entities || kb_mult != 0.0 {
        seen_percent(center, target.bb, ray_misses)
    } else {
        0.0f32
    };

    let damage = if profile.damages_entities {
        entity_damage_amount(profile.radius, dist_sqr, exposure)
    } else {
        0.0f32
    };

    let kb_res = if target.is_living {
        target.explosion_knockback_resistance
    } else {
        0.0
    };
    let scale = (1.0 - dist_ratio) * (exposure as f64) * (kb_mult as f64) * (1.0 - kb_res);
    let knockback_impulse = dir.scale(scale);

    let record_player_impulse =
        target.is_player && !target.is_spectator && !target.is_creative_flying;

    Some(ExplosionHit {
        damage,
        knockback_impulse,
        record_player_impulse,
    })
}

/// `WindChargeItem.lambda$use$0`: spawns at `(player.x, player.eye_y, player.z)`.
#[inline]
pub const fn wind_charge_spawn_pos(player_pos: Vec3, eye_y: f64) -> Vec3 {
    Vec3::new(player_pos.x, eye_y, player_pos.z)
}

/// `AbstractWindCharge.makeBoundingBox(Vec3) @0-82`:
/// `half_w = 0.3125f / 2`, `y0 = pos.y - 0.15f`, `y1 = y0 + 0.3125f`.
#[inline]
pub fn wind_charge_aabb(pos: Vec3) -> Aabb {
    let half_w = (WIND_CHARGE_WIDTH / 2.0f32) as f64;
    let y0 = pos.y - WIND_CHARGE_AABB_Y_OFFSET;
    let y1 = y0 + WIND_CHARGE_HEIGHT as f64;
    Aabb::new(pos.x - half_w, y0, pos.z - half_w, pos.x + half_w, y1, pos.z + half_w)
}

/// Exact `Projectile.shootFromRotation` + `Projectile.getMovementToShoot` (`@0-113`, `@0-74`).
/// Pass `spread = Vec3::ZERO` for zero-inaccuracy expectation or 3 triangular draws
/// `random.triangle(0.0, 0.0172275 * inaccuracy)` along X, Y, Z.
pub fn shoot_from_rotation(
    pitch: f32,
    yaw: f32,
    roll_offset: f32,
    power: f32,
    spread: Vec3,
    thrower_known_movement: Vec3,
    thrower_on_ground: bool,
) -> Vec3 {
    let y_rad = (yaw * 0.017453292f32) as f64;
    let p_rad = (pitch * 0.017453292f32) as f64;
    let r_rad = ((pitch + roll_offset) * 0.017453292f32) as f64;

    let fx = -math::sin(y_rad) * math::cos(p_rad);
    let fy = -math::sin(r_rad);
    let fz = math::cos(y_rad) * math::cos(p_rad);

    let shoot_vel = Vec3::new(fx as f64, fy as f64, fz as f64)
        .normalize()
        .plus(spread)
        .scale(power as f64);

    let known_y = if thrower_on_ground { 0.0 } else { thrower_known_movement.y };
    shoot_vel.plus(Vec3::new(thrower_known_movement.x, known_y, thrower_known_movement.z))
}

/// `AbstractWindCharge.onHitBlock @39-53`: `hit_location + face.unit_vec * 0.25`.
#[inline]
pub fn wind_charge_block_hit_center(hit_location: Vec3, face: Direction) -> Vec3 {
    let s = face.step();
    hit_location.plus(Vec3::new(
        s[0] as f64 * WIND_CHARGE_HIT_BLOCK_OFFSET,
        s[1] as f64 * WIND_CHARGE_HIT_BLOCK_OFFSET,
        s[2] as f64 * WIND_CHARGE_HIT_BLOCK_OFFSET,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindChargeTick {
    pub position: Vec3,
    pub velocity: Vec3,
    pub no_deflect_ticks: i32,
    pub exploded_above_world: bool,
}

/// One free-flight server tick of `WindCharge` (when no block/entity hit occurred):
/// - `WindCharge.tick @10-39`: auto-explodes & discards if `blockY > max_y + 30`.
/// - `AbstractHurtingProjectile.tick`: `applyInertia()` (`inertia = 1.0f` in BOTH air & water,
///   `accelerationPower = 0.0`), `pos += vel`, then `noDeflectTicks = (n - 1).max(0)`.
#[inline]
pub fn wind_charge_free_tick(
    position: Vec3,
    velocity: Vec3,
    no_deflect_ticks: i32,
    max_y: i32,
) -> WindChargeTick {
    if math::floor(position.y) > max_y + WIND_CHARGE_MAX_Y_MARGIN {
        return WindChargeTick {
            position,
            velocity,
            no_deflect_ticks,
            exploded_above_world: true,
        };
    }
    // `accelerationPower = 0.0`, `getInertia = 1.0f`, `getLiquidInertia = 1.0f`
    let next_vel = velocity.plus(velocity.normalize().scale(0.0)).scale(1.0);
    let next_pos = position.plus(next_vel);
    WindChargeTick {
        position: next_pos,
        velocity: next_vel,
        no_deflect_ticks: if no_deflect_ticks > 0 { no_deflect_ticks - 1 } else { 0 },
        exploded_above_world: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_charge_straight_down_standing_and_jumping() {
        // Straight down pitch = 90.0f, yaw = 0.0f, power = 1.5f.
        // Check unit direction is downward (fy = -Mth.sin(90deg)).
        let stand_vel = shoot_from_rotation(90.0, 0.0, 0.0, WIND_CHARGE_SHOOT_POWER, Vec3::default(), Vec3::default(), true);
        assert!(stand_vel.y < -1.49);

        // Jumping player (not on ground, vy = 0.42): wind charge inherits +0.42 Y velocity!
        let jump_vel = shoot_from_rotation(
            90.0,
            0.0,
            0.0,
            WIND_CHARGE_SHOOT_POWER,
            Vec3::default(),
            Vec3::new(0.0, 0.42, 0.0),
            false,
        );
        assert!((jump_vel.y - (stand_vel.y + 0.42)).abs() < 1e-12);

        // On ground throwing ignores player's vertical velocity (`onGround() ? 0.0 : known.y`).
        let ground_ignored = shoot_from_rotation(
            90.0,
            0.0,
            0.0,
            WIND_CHARGE_SHOOT_POWER,
            Vec3::default(),
            Vec3::new(0.0, -0.0784, 0.0),
            true,
        );
        assert_eq!(ground_ignored.to_bits_tuple(), stand_vel.to_bits_tuple());

        // Block hit center offset = +0.25 * face.step
        assert_eq!(
            wind_charge_block_hit_center(Vec3::new(0.5, 64.0, 0.5), Direction::Up),
            Vec3::new(0.5, 64.25, 0.5)
        );

        // Unobstructed player standing directly above wind charge burst at (0.5, 64.25, 0.5)
        let target = ExplosionTarget {
            feet_pos: Vec3::new(0.5, 64.0, 0.5),
            eye_y: 64.0 + 1.62,
            bb: Aabb::player(Vec3::new(0.5, 64.0, 0.5), 1.8),
            is_living: true,
            is_player: true,
            is_spectator: false,
            is_creative_flying: false,
            is_flying_ability: false,
            is_primed_tnt: false,
            ignore_explosion: false,
            explosion_knockback_resistance: 0.0,
        };
        let hit = explosion_entity_effect(
            ExplosionProfile::wind_charge(),
            Vec3::new(0.5, 64.25, 0.5),
            target,
            |_, _| true,
        )
        .unwrap();
        assert_eq!(hit.damage, 0.0);
        assert!(hit.knockback_impulse.y > 0.9);
        assert!(hit.record_player_impulse);
    }

    trait Vec3Bits {
        fn to_bits_tuple(self) -> (u64, u64, u64);
    }
    impl Vec3Bits for Vec3 {
        fn to_bits_tuple(self) -> (u64, u64, u64) {
            (self.x.to_bits(), self.y.to_bits(), self.z.to_bits())
        }
    }
}
