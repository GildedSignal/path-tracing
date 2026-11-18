//! Elytra firework boost and wall-impact damage laws (26.2).
//!
//! Glide itself (`LivingEntity.travelFallFlying`) is not in this module; the boost
//! is a foreign write from `FireworkRocketEntity.tick` that lands after the rider's
//! same-tick `travelFallFlying` and takes effect on tick N + 1.

use crate::{math, Vec3};

/// `Entity.getLookAngle` -> `Entity.calculateViewVector(xRot, yRot)` (`@0-64`).
pub fn calculate_view_vector(yaw: f32, pitch: f32) -> Vec3 {
    let f = pitch * 0.017453292f32;
    let g = -yaw * 0.017453292f32;
    let sin_g = math::sin(g as f64);
    let cos_g = math::cos(g as f64);
    let sin_f = math::sin(f as f64);
    let cos_f = math::cos(f as f64);
    Vec3::new((sin_g * cos_f) as f64, (-sin_f) as f64, (cos_g * cos_f) as f64)
}

/// `FireworkRocketEntity.tick` (`@75-174`) velocity rewrite in Mojang's exact
/// bytecode association: `v + look * 0.1 + (look * 1.5 - v) * 0.5`.
pub fn firework_boost(vel: Vec3, yaw: f32, pitch: f32) -> Vec3 {
    let look = calculate_view_vector(yaw, pitch);
    vel.plus(Vec3::new(
        look.x * 0.1 + (look.x * 1.5 - vel.x) * 0.5,
        look.y * 0.1 + (look.y * 1.5 - vel.y) * 0.5,
        look.z * 0.1 + (look.z * 1.5 - vel.z) * 0.5,
    ))
}

/// `FireworkRocketEntity.<init>(Level, double, double, double, ItemStack)` (`@47-141`):
/// `lifetime = 10 * (1 + flight_duration) + nextInt(6) + nextInt(7)`.
pub fn firework_lifetime_bounds(flight_duration: u8) -> (u32, u32) {
    let base = 10 * (1 + flight_duration as u32);
    (base, base + 11)
}

/// Inclusive range of boost ticks while the attached rider remains fall-flying:
/// `FireworkRocketEntity.tick` boosts before `life++` and explodes when `life > lifetime`.
pub fn firework_boost_ticks_bounds(flight_duration: u8) -> (u32, u32) {
    let (lo, hi) = firework_lifetime_bounds(flight_duration);
    (lo + 1, hi + 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireworkStep {
    pub boosted: bool,
    pub exploded: bool,
}

/// Attached `FireworkRocketEntity` tick state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttachedFirework {
    pub life: u32,
    pub lifetime: u32,
}

impl AttachedFirework {
    pub fn new(lifetime: u32) -> Self {
        Self { life: 0, lifetime }
    }

    /// `FireworkRocketItem.use` (`@0-4,49-57`) only spawns when `isFallFlying()`.
    pub fn try_use(fall_flying: bool, lifetime: u32) -> Option<Self> {
        fall_flying.then_some(Self::new(lifetime))
    }

    /// `FireworkRocketEntity.tick` (`@40-174, 420-427, 510-540`).
    /// Runs in `EntityTickList` after the rider's same-tick `travelFallFlying`,
    /// so any boost written on tick `N` takes effect on the rider's travel at tick `N + 1`.
    pub fn tick(
        &mut self,
        attached_fall_flying: bool,
        vel: &mut Vec3,
        yaw: f32,
        pitch: f32,
    ) -> FireworkStep {
        if attached_fall_flying {
            *vel = firework_boost(*vel, yaw, pitch);
        }
        self.life += 1;
        let exploded = self.life > self.lifetime;
        FireworkStep {
            boosted: attached_fall_flying,
            exploded,
        }
    }
}

/// `LivingEntity.handleFallFlyingCollisions(double, double)` (`@0-58`):
/// `f7 = (float)((pre_h_speed - post_h_speed) * 10.0 - 3.0)`, firing `hurt(flyIntoWall, f7)`
/// on `fcmpl; ifle` (so `f7 > 0.0f` OR `f7.is_nan()` fires; `0.3` drop -> `0.0f` silent).
pub fn fly_into_wall_damage(
    horizontal_collision: bool,
    pre_h_speed: f64,
    post_h_speed: f64,
) -> Option<f32> {
    if !horizontal_collision {
        return None;
    }
    let drop = pre_h_speed - post_h_speed;
    let f7 = (drop * 10.0 - 3.0) as f32;
    (f7 > 0.0f32 || f7.is_nan()).then_some(f7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firework_look_and_3_tick_boost_witnesses() {
        let yaw = -30.0f32;
        let pitch = 15.0f32;
        let look = calculate_view_vector(yaw, pitch);
        // witness.txt:3
        assert_eq!(look.x.to_bits(), 0x3fde_e88f_e000_0000);
        assert_eq!(look.y.to_bits(), 0xbfd0_8f7a_c000_0000);
        assert_eq!(look.z.to_bits(), 0x3fea_c4fc_4000_0000);

        // check/check-adversarial.txt:4-9
        let look_cases: &[(f32, f32, [u64; 3])] = &[
            (0.0, 0.0, [0x0, 0x8000_0000_0000_0000, 0x3ff0_0000_0000_0000]),
            (90.0, 0.0, [0xbff0_0000_0000_0000, 0x8000_0000_0000_0000, 0x0]),
            (-90.0, 0.0, [0x3ff0_0000_0000_0000, 0x8000_0000_0000_0000, 0x3ca1_a626_4000_0000]),
            (45.0, 180.0, [0x3fe6_a09e_6000_0000, 0xbca1_a626_4000_0000, 0xbfe6_a010_4000_0000]),
            (-89.5, 179.9, [0xbfefffaa_e000_0000, 0xbf5d_d859_6000_0000, 0xbf82_105b_4000_0000]),
            (33.3, -127.7, [0x3fd5_7caf_a000_0000, 0x3fe9_51bd_4000_0000, 0xbfe0_5acd_a000_0000]),
        ];
        for &(y, p, exp) in look_cases {
            let l = calculate_view_vector(y, p);
            assert_eq!([l.x.to_bits(), l.y.to_bits(), l.z.to_bits()], exp, "yaw={y} pitch={p}");
        }

        // witness.txt:2,4-6 (ticks 1-3; tick 2 y=0xbfd6b6efb4cccccd differs from folded ...b4cccccc)
        let mut v = Vec3::new(
            f64::from_bits(0x3fd9_9999_9999_999a),
            f64::from_bits(0xbfb9_9999_9999_999a),
            f64::from_bits(0x3fa9_9999_9999_999a),
        );
        let expected_ticks = [
            [0x3fe3_893d_2599_999a, 0xbfd1_46c1_f000_0000, 0x3fe7_8dd6_6999_999a], // witness.txt:4
            [0x3fe6_e775_5200_0000, 0xbfd6_b6ef_b4cc_cccd, 0x3ff1_43fa_68cc_cccd], // witness.txt:5
            [0x3fe8_9691_6833_3333, 0xbfd9_6f06_9733_3333, 0x3ff4_0282_02cc_cccd], // witness.txt:6
        ];
        for (i, exp) in expected_ticks.iter().enumerate() {
            v = firework_boost(v, yaw, pitch);
            assert_eq!([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()], *exp, "boost tick {}", i + 1);
        }
    }

    #[test]
    fn firework_lifetime_boost_count_and_lag_witnesses() {
        // witness.txt:21-24
        assert_eq!(firework_lifetime_bounds(0), (10, 21));
        assert_eq!(firework_boost_ticks_bounds(0), (11, 22));
        assert_eq!(firework_lifetime_bounds(1), (20, 31));
        assert_eq!(firework_boost_ticks_bounds(1), (21, 32));
        assert_eq!(firework_lifetime_bounds(2), (30, 41));
        assert_eq!(firework_boost_ticks_bounds(2), (31, 42));
        assert_eq!(firework_lifetime_bounds(3), (40, 51));
        assert_eq!(firework_boost_ticks_bounds(3), (41, 52));

        // FireworkRocketItem.use requires fall_flying; rocket runs after rider travel and boosts lifetime+1 ticks.
        assert!(AttachedFirework::try_use(false, 20).is_none());
        let mut fw = AttachedFirework::try_use(true, 20).unwrap();
        let mut vel = Vec3::new(0.4, -0.1, 0.05);
        // Simulate tick N: rider travelFallFlying sees pre-boost vel, then rocket ticks after rider.
        let mut boosts = 0;
        loop {
            let rider_seen_vel_before_rocket = vel;
            let step = fw.tick(true, &mut vel, -30.0, 15.0);
            if step.boosted {
                boosts += 1;
                assert_ne!(vel, rider_seen_vel_before_rocket);
            }
            if step.exploded {
                break;
            }
        }
        assert_eq!(boosts, 21); // lifetime 20 -> 21 boost ticks
    }

    #[test]
    fn fly_into_wall_damage_witnesses() {
        // witness.txt:9-18 & check/check-adversarial.txt:18-27
        assert_eq!(fly_into_wall_damage(false, 1.0, 0.0), None);
        assert_eq!(fly_into_wall_damage(true, 0.25, 0.0), None); // witness.txt:9
        assert_eq!(fly_into_wall_damage(true, 0.29, 0.0), None); // witness.txt:10
        assert_eq!(
            fly_into_wall_damage(true, f64::from_bits(0x3fd3_3333_3333_3332), 0.0),
            None
        );
        assert_eq!(
            fly_into_wall_damage(true, f64::from_bits(0x3fd3_3333_3333_3333), 0.0), // d5 = 0.3, witness.txt:12,18
            None
        );
        assert_eq!(
            fly_into_wall_damage(true, f64::from_bits(0x3fd3_3333_3333_3334), 0.0)
                .map(f32::to_bits), // witness.txt:13,18
            Some(0x2600_0000)
        );
        assert_eq!(
            fly_into_wall_damage(true, 0.31, 0.0).map(f32::to_bits), // witness.txt:14
            Some(0x3dcc_cccd)
        );
        assert_eq!(
            fly_into_wall_damage(true, 0.35, 0.0).map(f32::to_bits), // witness.txt:15
            Some(0x3f00_0000)
        );
        assert_eq!(
            fly_into_wall_damage(true, 0.5, 0.0).map(f32::to_bits), // witness.txt:16
            Some(0x4000_0000)
        );
        assert_eq!(
            fly_into_wall_damage(true, 1.0, 0.0).map(f32::to_bits), // witness.txt:17
            Some(0x40e0_0000)
        );
        assert_eq!(fly_into_wall_damage(true, -0.0, 0.0), None); // check-adversarial.txt:26
        assert!(fly_into_wall_damage(true, f64::NAN, 0.0).is_some_and(f32::is_nan)); // check-adversarial.txt:27
    }
}
