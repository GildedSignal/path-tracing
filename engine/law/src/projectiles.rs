//! Bow, crossbow, arrow, throwable (snowball/egg), and fishing-hook laws ported
//! bit-for-bit from Mojang 26.2 `server-26.2.jar` + JOML 1.10.8. Every launch is
//! `basis::body::launch` with the item's power, inaccuracy and roll.

use crate::{math, Vec3};
use crate::basis::body::{facing, shoot, spread, Launch};

/// `Entity.calculateViewVector @0-67` in `f32`:
/// `(Mth.sin(-rot_b * r) * Mth.cos(rot_a * r), -Mth.sin(rot_a * r), Mth.cos(-rot_b * r) * Mth.cos(rot_a * r))`,
/// where `r = 0.017453292f` and `(rot_a, rot_b)` is `(xRot, yRot)`.
#[inline]
pub fn calculate_view_vector(x_rot: f32, y_rot: f32) -> (f32, f32, f32) {
    let v = math::calculate_view_vector(y_rot, x_rot);
    (v.x as f32, v.y as f32, v.z as f32)
}

/// `Entity.calculateUpVector @0-10`: `calculateViewVector(x_rot - 90.0f, y_rot)`.
#[inline]
pub fn calculate_up_vector(x_rot: f32, y_rot: f32) -> (f32, f32, f32) {
    calculate_view_vector(x_rot - 90.0f32, y_rot)
}

// ============================================================================
// BowItem
// ============================================================================

/// `BowItem.releaseUsing @57-60`: minimum release power threshold in `f64`.
pub const BOW_MIN_RELEASE_POWER: f64 = 0.1;
/// `BowItem.releaseUsing @114`: `power * 3.0f`.
pub const BOW_VELOCITY_SCALE: f32 = 3.0;
/// `BowItem.releaseUsing @117`: inaccuracy passed to `shoot`.
pub const BOW_INACCURACY: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BowRelease {
    pub power: f32,
    pub speed: f32,
    pub crit: bool,
}

/// `BowItem.getPowerForTime @0-26`:
/// `f = (float)draw_ticks / 20.0f; f = (f*f + f*2.0f) / 3.0f; if (f > 1.0f) f = 1.0f;`
/// No lower clamp is applied here.
#[inline]
pub fn bow_power_for_time(draw_ticks: i32) -> f32 {
    let mut f = draw_ticks as f32 / 20.0f32;
    f = (f * f + f * 2.0f32) / 3.0f32;
    if f > 1.0f32 {
        f = 1.0f32;
    }
    f
}

/// `BowItem.releaseUsing @54-65`: releases iff `(double)power >= 0.1d`.
#[inline]
pub const fn bow_releases(power: f32) -> bool {
    (power as f64) >= BOW_MIN_RELEASE_POWER
}

/// `BowItem.releaseUsing @112-116`: `power * 3.0f`.
#[inline]
pub const fn bow_launch_speed(power: f32) -> f32 {
    power * BOW_VELOCITY_SCALE
}

/// `BowItem.releaseUsing @117-121`: crit arrow iff `power == 1.0f`.
#[inline]
pub const fn bow_is_crit(power: f32) -> bool {
    power == 1.0f32
}

/// Evaluates `BowItem.releaseUsing` for `charge_ticks = 72000 - time_left`.
#[inline]
pub fn bow_release(charge_ticks: i32) -> Option<BowRelease> {
    let power = bow_power_for_time(charge_ticks);
    if !bow_releases(power) {
        return None;
    }
    Some(BowRelease {
        power,
        speed: bow_launch_speed(power),
        crit: bow_is_crit(power),
    })
}


// ============================================================================
// CrossbowItem & ProjectileWeaponItem
// ============================================================================

/// `CrossbowItem.MAX_CHARGE_DURATION`.
pub const CROSSBOW_BASE_CHARGE_SECONDS: f32 = 1.25;
/// `quick_charge.json`: `crossbow_charge_time` adds `-0.25` per level (`linear`).
pub const CROSSBOW_QUICK_CHARGE_PER_LEVEL: f32 = -0.25;
/// `CrossbowItem.ARROW_POWER`.
pub const CROSSBOW_ARROW_POWER: f32 = 3.15;
/// `CrossbowItem.FIREWORK_POWER`.
pub const CROSSBOW_FIREWORK_POWER: f32 = 1.6;

/// `EnchantmentHelper.modifyCrossbowChargingTime @0-28` with `quick_charge.json`:
/// `max(0.0f, 1.25f + (-0.25f * level))` when `level > 0`, else `1.25f`.
#[inline]
pub fn crossbow_charging_time_seconds(quick_charge_level: u8) -> f32 {
    if quick_charge_level == 0 {
        CROSSBOW_BASE_CHARGE_SECONDS
    } else {
        let delta = CROSSBOW_QUICK_CHARGE_PER_LEVEL
            + CROSSBOW_QUICK_CHARGE_PER_LEVEL * ((quick_charge_level - 1) as f32);
        (CROSSBOW_BASE_CHARGE_SECONDS + delta).max(0.0f32)
    }
}

/// `CrossbowItem.getChargeDuration @0-17`: `Mth.floor(modifyCrossbowChargingTime * 20.0f)`.
/// Quick Charge `0..=3` -> `25 / 20 / 15 / 10` ticks.
#[inline]
pub fn crossbow_charge_duration(quick_charge_level: u8) -> i32 {
    math::floor_f32(crossbow_charging_time_seconds(quick_charge_level) * 20.0f32)
}

/// `CrossbowItem.getShootingPower @0-15`: `1.6f` if firework rocket is loaded, else `3.15f`.
#[inline]
pub const fn crossbow_shooting_power(is_firework: bool) -> f32 {
    if is_firework {
        CROSSBOW_FIREWORK_POWER
    } else {
        CROSSBOW_ARROW_POWER
    }
}

/// `ProjectileWeaponItem.shoot @10-125`:
/// computes the yaw/axis fan angle (degrees) for the `index`-th of `count` projectiles.
/// With `count = 3, spread_degrees = 10.0` (Multishot I), indices `0, 1, 2` yield `0.0, -10.0, +10.0`.
#[inline]
pub fn projectile_fan_angle(index: usize, count: usize, spread_degrees: f32) -> f32 {
    let step = if count == 1 {
        0.0f32
    } else {
        2.0f32 * spread_degrees / (count - 1) as f32
    };
    let base = ((count - 1) % 2) as f32 * step / 2.0f32;
    let sign = if index % 2 == 0 { 1.0f32 } else { -1.0f32 };
    base + sign * (((index + 1) / 2) as f32) * step
}

/// Multishot fan angles `[0.0, -spread_degrees, +spread_degrees]` in spawn order (`[0.0, -10.0, 10.0]` for Multishot I).
#[inline]
pub fn crossbow_multishot_angles(spread_degrees: f32) -> [f32; 3] {
    [
        projectile_fan_angle(0, 3, spread_degrees),
        projectile_fan_angle(1, 3, spread_degrees),
        projectile_fan_angle(2, 3, spread_degrees),
    ]
}

/// `org.joml.Quaternionf.setAngleAxis(DDDD) @0-54` with `org.joml.Math.cosFromSinInternal(DD)`.
fn joml_quaternion_axis_angle(angle: f64, ax: f64, ay: f64, az: f64) -> (f32, f32, f32, f32) {
    let half = angle * 0.5f64;
    let sin = half.sin();
    let mut cos = (1.0f64 - sin * sin).sqrt();
    let a = half + 1.5707963267948966f64;
    let two_pi = 6.283185307179586f64;
    let mut reduced = a - ((a / two_pi) as i32) as f64 * two_pi;
    if reduced < 0.0f64 {
        reduced += two_pi;
    }
    if reduced >= 3.141592653589793f64 {
        cos = -cos;
    }
    ((ax * sin) as f32, (ay * sin) as f32, (az * sin) as f32, cos as f32)
}

/// `org.joml.Quaternionf.transform(FFFLorg/joml/Vector3f) @0-262` via `org.joml.Math.fma`.
/// With default `org.joml.Options.USE_MATH_FMA == false`, `Runtime.HAS_Math_fma` is `false`,
/// so JOML executes unfused `a * b + c`.
fn joml_rotate_vec3f(v: (f32, f32, f32), q: (f32, f32, f32, f32)) -> (f32, f32, f32) {
    let (x, y, z) = v;
    let (qx, qy, qz, qw) = q;
    let (xx, yy, zz, ww) = (qx * qx, qy * qy, qz * qz, qw * qw);
    let (xy, xz, yz) = (qx * qy, qx * qz, qy * qz);
    let (xw, zw, yw) = (qx * qw, qz * qw, qy * qw);
    let inv = 1.0f32 / (xx + yy + zz + ww);
    (
        ((xx - yy - zz + ww) * inv) * x
            + ((2.0f32 * (xy - zw) * inv) * y + 2.0f32 * (xz + yw) * inv * z),
        (2.0f32 * (xy + zw) * inv) * x
            + (((yy - xx - zz + ww) * inv) * y + 2.0f32 * (yz - xw) * inv * z),
        (2.0f32 * (xz - yw) * inv) * x
            + ((2.0f32 * (yz + xw) * inv) * y + (zz - xx - yy + ww) * inv * z),
    )
}

/// `CrossbowItem.shootProjectile @91-173` (non-targeted player shot):
/// rotates `view = calculateViewVector(xRot, yRot)` by `angle_deg * 0.017453292f`
/// radians around `up = calculateUpVector(xRot, yRot)` via JOML `Quaternionf`,
/// then calls `Projectile.shoot(shot.x, shot.y, shot.z, speed, inaccuracy)`
/// (note: crossbow does NOT add shooter `knownMovement`).
pub fn crossbow_launch(
    x_rot: f32,
    y_rot: f32,
    angle_deg: f32,
    speed: f32,
    inaccuracy: f32,
    draws: [f64; 6],
) -> Launch {
    let up = calculate_up_vector(x_rot, y_rot);
    let view = calculate_view_vector(x_rot, y_rot);
    let q = joml_quaternion_axis_angle(
        (angle_deg * 0.017453292f32) as f64,
        up.0 as f64,
        up.1 as f64,
        up.2 as f64,
    );
    let shot = joml_rotate_vec3f(view, q);
    let velocity = shoot(Vec3::new(shot.0 as f64, shot.1 as f64, shot.2 as f64), speed, spread(inaccuracy, draws));
    let (y_rot, x_rot) = facing(velocity);
    Launch { velocity, y_rot, x_rot }
}

// ============================================================================
// AbstractArrow
// ============================================================================

/// `AbstractArrow.<init> @24`.
pub const ARROW_BASE_DAMAGE: f64 = 2.0;
/// `AbstractArrow.tickDespawn @14`.
pub const ARROW_DESPAWN_TICKS: i32 = 1200;
/// `AbstractArrow.onHitBlock @157`.
pub const ARROW_SHAKE_TIME: i32 = 7;
/// `AbstractArrow.onHitBlock @93`: `(double) 0.05f` = `0.05000000074505806`.
pub const ARROW_STICK_BACKOFF: f64 = 0.05f32 as f64;
/// `AbstractArrow.onHitEntity @104`.
pub const ARROW_MAX_DAMAGE: f64 = 2.147483647E9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrowState {
    pub pos: Vec3,
    pub vel: Vec3,
    pub in_ground: bool,
    pub in_ground_time: i32,
    pub shake_time: i32,
    pub life: i32,
    pub crit: bool,
    pub pierce_level: u8,
    pub base_damage: f64,
}

impl Default for ArrowState {
    fn default() -> Self {
        Self {
            pos: Vec3::default(),
            vel: Vec3::default(),
            in_ground: false,
            in_ground_time: 0,
            shake_time: 0,
            life: 0,
            crit: false,
            pierce_level: 0,
            base_damage: ARROW_BASE_DAMAGE,
        }
    }
}

/// `AbstractArrow.tick @13-583` (free flight / grounded ageing, no block or entity hit):
/// - Decrements `shake_time` if positive.
/// - Grounded arrows increment `life` (`tickDespawn`, returns `false` at `life >= 1200`)
///   and `in_ground_time`.
/// - Airborne arrows move by the PRE-drag velocity `d` captured at `@13-17`:
///   water inertia `0.6f` applies BEFORE the move (`@279-290`), `pos += d` (`@526-546`),
///   air drag `0.99f` applies AFTER the move when `!in_water_after` (`@549-561`),
///   and gravity `0.05d` applies last (`@564-576`).
/// Returns `true` if still alive (not despawned).
pub fn arrow_free_tick(
    arrow: &mut ArrowState,
    in_water_before: bool,
    in_water_after: bool,
) -> bool {
    if arrow.shake_time > 0 {
        arrow.shake_time -= 1;
    }
    if arrow.in_ground {
        arrow.life += 1;
        arrow.in_ground_time += 1;
        return arrow.life < ARROW_DESPAWN_TICKS;
    }
    arrow.in_ground_time = 0;
    let d = arrow.vel;
    if in_water_before {
        crate::basis::body::ARROW.drag(&mut arrow.vel, true);
    }
    arrow.pos = arrow.pos.plus(d);
    if !in_water_after {
        crate::basis::body::ARROW.drag(&mut arrow.vel, false);
    }
    crate::basis::body::ARROW.gravity(&mut arrow.vel, false);
    true
}

/// `java.lang.Math.signum(double)`: preserves `0.0` / `-0.0` / `NaN` (unlike Rust `f64::signum`).
#[inline]
fn java_signum(d: f64) -> f64 {
    if d.is_nan() || d == 0.0 {
        d
    } else if d < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// `AbstractArrow.onHitBlock @56-183`:
/// backs off `(float)0.05f` (`0.05000000074505806`) along each `Math.signum(vel)` axis,
/// zeroes velocity, sets `in_ground = true`, `shake_time = 7`, clears `crit` and `pierce_level`.
pub fn arrow_hit_block(arrow: &mut ArrowState) {
    let back = Vec3::new(
        java_signum(arrow.vel.x),
        java_signum(arrow.vel.y),
        java_signum(arrow.vel.z),
    )
    .scale(ARROW_STICK_BACKOFF);
    arrow.pos = arrow.pos.minus(back);
    arrow.vel = Vec3::default();
    arrow.in_ground = true;
    arrow.shake_time = ARROW_SHAKE_TIME;
    arrow.crit = false;
    arrow.pierce_level = 0;
}

/// `AbstractArrow.onHitEntity @89-96` with `power.json` (`add`, `linear { base: 1.0, per_level_above_first: 0.5 }`):
/// widens/narrows `(float) base_damage`, adds enchantment bonus in `f32`, widens back to `f64`.
#[inline]
pub fn arrow_power_base_damage(base_damage: f64, power_level: u8) -> f64 {
    if power_level == 0 {
        base_damage
    } else {
        let bonus = 1.0f32 + 0.5f32 * ((power_level - 1) as f32);
        ((base_damage as f32) + bonus) as f64
    }
}

/// `AbstractArrow.onHitEntity @98-113`:
/// `Mth.ceil(Mth.clamp((double) speed_f32 * base_damage, 0.0, 2.147483647E9))`.
#[inline]
pub fn arrow_damage_from_speed(speed_f32: f32, base_damage: f64) -> i32 {
    let scaled = (speed_f32 as f64 * base_damage).clamp(0.0f64, ARROW_MAX_DAMAGE);
    math::ceil(scaled)
}

/// `AbstractArrow.onHitEntity @10-113`:
/// truncates `vel.length()` to `f32` before multiplying by `base_damage`.
#[inline]
pub fn arrow_damage(vel: Vec3, base_damage: f64) -> i32 {
    let speed_f32 = vel.length_sqr().sqrt() as f32;
    arrow_damage_from_speed(speed_f32, base_damage)
}

/// `AbstractArrow.onHitEntity @203-208`: exclusive upper bound for `random.nextInt(damage / 2 + 2)`.
#[inline]
pub const fn arrow_crit_bonus_bound(damage: i32) -> i32 {
    damage / 2 + 2
}

/// `AbstractArrow.onHitEntity @199-230`: `Math.min((long)crit_roll + (long)damage, 2147483647L)`.
#[inline]
pub fn arrow_crit_damage(damage: i32, crit_roll: i32) -> i32 {
    ((crit_roll as i64) + (damage as i64)).min(2_147_483_647) as i32
}

/// `AbstractArrow.doKnockback @28-44` with `punch.json` (`add`, `linear { base: 1.0, per_level_above_first: 1.0 }`).
#[inline]
pub fn arrow_punch_knockback(punch_level: u8) -> f32 {
    if punch_level == 0 {
        0.0f32
    } else {
        1.0f32 + 1.0f32 * ((punch_level - 1) as f32)
    }
}

/// `AbstractArrow.doKnockback @46-120`:
/// `imp = normalize(v * (1, 0, 1)) * (knockback * 0.6 * max(0, 1 - knockback_resistance))`;
/// when `knockback > 0 && imp.lengthSqr() > 0`, applies `target.push(imp.x, 0.1, imp.z)`.
pub fn arrow_punch_push(
    velocity: Vec3,
    knockback: f32,
    knockback_resistance: f64,
) -> Option<Vec3> {
    let kb = knockback as f64;
    if kb <= 0.0f64 {
        return None;
    }
    let resist = (1.0f64 - knockback_resistance).max(0.0f64);
    let imp = velocity
        .multiply(Vec3::new(1.0, 0.0, 1.0))
        .normalize()
        .scale(kb * 0.6f64 * resist);
    if imp.length_sqr() > 0.0f64 {
        Some(Vec3::new(imp.x, 0.1f64, imp.z))
    } else {
        None
    }
}

// ============================================================================
// ThrowableProjectile (Snowball, ThrownEgg, shared with EnderPearl)
// ============================================================================

/// `SnowballItem.PROJECTILE_SHOOT_POWER` & `EggItem.use @74`.
pub const SNOWBALL_SHOOT_POWER: f32 = 1.5;


// ============================================================================
// FishingHook
// ============================================================================

/// `FishingHook.<init> @171`.
pub const FISHING_CAST_SPEED_BASE: f64 = 0.6;
/// `FishingHook.<init> @90, 110`.
pub const FISHING_CAST_SPAWN_BACK_OFFSET: f64 = 0.3;
/// `FishingHook.pullEntity @44`.
pub const FISHING_PULL_SCALE: f64 = 0.1;

/// `FishingHook.<init>(Player, Level, int, int) @16-245`:
/// returns `(spawn_pos, initial_velocity)`.
/// `tri_draws` are the three per-axis `random.triangle(0.5d, 0.0103365d)` draws
/// (pass `[0.5; 3]` for deterministic mode).
pub fn fishing_cast(
    x_rot: f32,
    y_rot: f32,
    owner_pos: Vec3,
    owner_eye_y: f64,
    tri_draws: [f64; 3],
) -> (Vec3, Vec3) {
    let yaw_rad = (-y_rot * 0.017453292f32 - 3.1415927f32) as f64;
    let f7 = math::cos(yaw_rad);
    let f8 = math::sin(yaw_rad);
    let pitch_rad = (-x_rot * 0.017453292f32) as f64;
    let f9 = -math::cos(pitch_rad);
    let f10 = math::sin(pitch_rad);

    let spawn = Vec3::new(
        owner_pos.x - (f8 as f64) * FISHING_CAST_SPAWN_BACK_OFFSET,
        owner_eye_y,
        owner_pos.z - (f7 as f64) * FISHING_CAST_SPAWN_BACK_OFFSET,
    );
    let dy = crate::math::clamp_f32(-(f10 / f9), -5.0f32, 5.0f32);
    let dir = Vec3::new((-f8) as f64, dy as f64, (-f7) as f64);
    let len = dir.length_sqr().sqrt();
    let base = FISHING_CAST_SPEED_BASE / len;
    let vel = dir.multiply(Vec3::new(
        base + tri_draws[0],
        base + tri_draws[1],
        base + tri_draws[2],
    ));
    (spawn, vel)
}

/// `FishingHook.pullEntity @10-47`:
/// `(owner.position - hook.position) * 0.1` — taken from the HOOK's position, NOT the target's!
#[inline]
pub fn fishing_pull_impulse(owner_pos: Vec3, hook_pos: Vec3) -> Vec3 {
    owner_pos.minus(hook_pos).scale(FISHING_PULL_SCALE)
}

/// `FishingHook.pullEntity @51-60`: `target_vel + (owner_pos - hook_pos) * 0.1`.
#[inline]
pub fn fishing_pull_velocity(owner_pos: Vec3, hook_pos: Vec3, target_vel: Vec3) -> Vec3 {
    target_vel.plus(fishing_pull_impulse(owner_pos, hook_pos))
}

