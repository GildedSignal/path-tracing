//! Mojang 26.2 melee combat (`Player.attack`, `Player.causeExtraKnockback`,
//! `Player.getAttackStrengthScale`, `Player.isSweepAttack`, `LivingEntity.knockback`,
//! `Entity.push(DDD)`).
//!
//! Verified directly against `server-26.2.jar` bytecode + `CombatExplosionWitness.java`
//! (`tests/data/combat_explosion.tsv`).

use crate::{math, Vec3};

pub const ATTACK_EXHAUSTION: f32 = 0.1;
pub const CRIT_MULTIPLIER: f32 = 1.5;
pub const SPRINT_KNOCKBACK_BONUS: f32 = 0.5;
pub const FULL_STRENGTH_THRESHOLD: f32 = 0.9;
pub const ATTACKER_SLOWDOWN_XZ: f64 = 0.6;
pub const KNOCKBACK_DEGENERATE_EPS_SQR: f64 = 9.999999747378752e-6;
pub const KNOCKBACK_GROUND_Y_CAP: f64 = 0.4;
pub const NON_LIVING_EXTRA_KB_Y: f64 = 0.1;

/// `Player.getCurrentItemAttackStrengthDelay() @0-14`:
/// `(float)(1.0D / attack_speed * 20.0D)`. Default unarmed / generic `ATTACK_SPEED` is `4.0` -> `5.0f`.
#[inline]
pub fn attack_strength_delay(attack_speed: f64) -> f32 {
    (1.0 / attack_speed * 20.0) as f32
}

/// `Player.getAttackStrengthScale(float base_time) @0-17`:
/// `Mth.clamp(((float)ticker + base_time) / delay, 0.0f, 1.0f)`.
/// `Player.attack @45` passes `base_time = 0.5f`.
#[inline]
pub fn attack_strength_scale(attack_strength_ticker: i32, base_time: f32, attack_speed: f64) -> f32 {
    let delay = attack_strength_delay(attack_speed);
    ((attack_strength_ticker as f32 + base_time) / delay).clamp(0.0, 1.0)
}

/// `Player.baseDamageScaleFactor() @0-19`: `0.2f + scale * scale * 0.8f` using `getAttackStrengthScale(0.5f)`.
#[inline]
pub fn base_damage_scale_factor(strength_scale: f32) -> f32 {
    0.2f32 + strength_scale * strength_scale * 0.8f32
}

/// `Player.canCriticalAttack(Entity) @0-63`.
#[inline]
pub const fn can_critical_attack(
    fall_distance: f64,
    on_ground: bool,
    on_climbable: bool,
    in_water: bool,
    mobility_restricted: bool,
    is_passenger: bool,
    target_is_living: bool,
    is_sprinting: bool,
) -> bool {
    fall_distance > 0.0
        && !on_ground
        && !on_climbable
        && !in_water
        && !mobility_restricted
        && !is_passenger
        && target_is_living
        && !is_sprinting
}

/// `Player.isSweepAttack(boolean full_strength, boolean critical, boolean sprint_kb) @0-65`.
#[inline]
pub fn is_sweep_attack(
    full_strength: bool,
    critical: bool,
    sprint_knockback: bool,
    on_ground: bool,
    known_movement: Vec3,
    speed: f32,
    holding_sword: bool,
) -> bool {
    if !full_strength || critical || sprint_knockback || !on_ground || !holding_sword {
        return false;
    }
    let threshold = (speed as f64) * 2.5;
    known_movement.horizontal_sqr() < threshold * threshold
}

/// `LivingEntity.knockback(double strength, double x, double z, ...) @0-184`.
/// Returns `None` if `effective_strength = strength * (1.0 - kb_resistance) <= 0.0`.
/// When `x*x + z*z < 9.999999747378752e-6`, draws jitter from `next_double` until nonzero,
/// matching the `@30-95` retry loop.
pub fn living_knockback(
    velocity: Vec3,
    on_ground: bool,
    strength: f64,
    mut x: f64,
    mut z: f64,
    knockback_resistance: f64,
    mut next_double: impl FnMut() -> f64,
) -> Option<Vec3> {
    let effective = strength * (1.0 - knockback_resistance);
    if effective <= 0.0 {
        return None;
    }
    while x * x + z * z < KNOCKBACK_DEGENERATE_EPS_SQR {
        x = (next_double() - next_double()) * 0.01;
        z = (next_double() - next_double()) * 0.01;
    }
    let delta = Vec3::new(x, 0.0, z).normalize().scale(effective);
    let vx = velocity.x / 2.0 - delta.x;
    let vy = if on_ground {
        KNOCKBACK_GROUND_Y_CAP.min(velocity.y / 2.0 + effective)
    } else {
        velocity.y
    };
    let vz = velocity.z / 2.0 - delta.z;
    Some(Vec3::new(vx, vy, vz))
}

/// `Entity.push(double x, double y, double z) @0-42`: additive finite delta.
#[inline]
pub fn push_impulse(velocity: Vec3, dx: f64, dy: f64, dz: f64) -> Option<Vec3> {
    if dx.is_finite() && dy.is_finite() && dz.is_finite() {
        Some(velocity.plus(Vec3::new(dx, dy, dz)))
    } else {
        None
    }
}

/// Outcome of `Player.causeExtraKnockback(target, extra_kb, prev_vel, ...) @0-169`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtraKnockbackResult {
    pub attacker_velocity: Vec3,
    pub attacker_sprinting: bool,
    pub target_packet_velocity: Vec3,
    pub target_server_velocity: Vec3,
    pub knockback_applied: bool,
}

/// Exact `Player.causeExtraKnockback @0-169`:
/// Only when `extra_knockback > 0.0f` does it apply knockback/push to `target`, multiply
/// attacker XZ velocity by `0.6`, and clear `attacker.sprinting = false`.
/// For `ServerPlayer` targets with `hurtMarked`, sends the knockback velocity in a packet
/// and restores `prev_target_vel` on the server (`@123-169`).
pub fn cause_extra_knockback(
    attacker_velocity: Vec3,
    attacker_sprinting: bool,
    attacker_yaw: f32,
    target_velocity: Vec3,
    target_on_ground: bool,
    target_is_living: bool,
    target_kb_resistance: f64,
    target_is_server_player: bool,
    extra_knockback: f32,
    next_double: impl FnMut() -> f64,
) -> ExtraKnockbackResult {
    let prev_target_velocity = target_velocity;
    let mut new_target_vel = target_velocity;
    let mut target_hurt_marked = false;
    let mut new_attacker_vel = attacker_velocity;
    let mut new_sprinting = attacker_sprinting;

    if extra_knockback > 0.0 {
        let rad = (attacker_yaw * 0.017453292f32) as f64;
        let sx = math::sin(rad);
        let cz = math::cos(rad);
        if target_is_living {
            if let Some(kb) = living_knockback(
                target_velocity,
                target_on_ground,
                extra_knockback as f64,
                sx as f64,
                (-cz) as f64,
                target_kb_resistance,
                next_double,
            ) {
                new_target_vel = kb;
                target_hurt_marked = true;
            }
        } else if let Some(pushed) = push_impulse(
            target_velocity,
            (-sx * extra_knockback) as f64,
            NON_LIVING_EXTRA_KB_Y,
            (cz * extra_knockback) as f64,
        ) {
            new_target_vel = pushed;
            target_hurt_marked = true;
        }

        // @100-120: always runs inside `if (extra_knockback > 0)` even if KB resistance absorbed target KB!
        new_attacker_vel = Vec3::new(
            attacker_velocity.x * ATTACKER_SLOWDOWN_XZ,
            attacker_velocity.y,
            attacker_velocity.z * ATTACKER_SLOWDOWN_XZ,
        );
        new_sprinting = false;
    }

    // @123-169: ServerPlayer restores prev_target_vel after transmitting ClientboundSetEntityMotionPacket.
    let target_server_velocity = if target_is_server_player && target_hurt_marked {
        prev_target_velocity
    } else {
        new_target_vel
    };

    ExtraKnockbackResult {
        attacker_velocity: new_attacker_vel,
        attacker_sprinting: new_sprinting,
        target_packet_velocity: new_target_vel,
        target_server_velocity,
        knockback_applied: extra_knockback > 0.0,
    }
}

/// Inputs for `Player.attack(Entity) @0-355` (post-`cannotAttack`, post-`deflectProjectile`).
#[derive(Clone, Copy, Debug)]
pub struct AttackInput {
    pub base_attack_damage: f32,
    pub enchanted_full_damage: f32,
    pub item_damage_bonus: f32,
    pub attack_strength_ticker: i32,
    pub attack_speed: f64,
    pub attacker_velocity: Vec3,
    pub attacker_known_movement: Vec3,
    pub attacker_speed: f32,
    pub attacker_yaw: f32,
    pub attacker_on_ground: bool,
    pub attacker_on_climbable: bool,
    pub attacker_in_water: bool,
    pub attacker_mobility_restricted: bool,
    pub attacker_is_passenger: bool,
    pub attacker_fall_distance: f64,
    pub attacker_sprinting: bool,
    pub holding_sword: bool,
    /// `LivingEntity.getKnockback`: `(ATTACK_KNOCKBACK + enchant) / 2.0f` (`0.0f` unenchanted).
    pub weapon_knockback: f32,
    pub target_is_living: bool,
    pub target_is_server_player: bool,
    pub target_on_ground: bool,
    pub target_velocity: Vec3,
    pub target_kb_resistance: f64,
    /// Result of `target.hurtOrSimulate(source, total_damage)`.
    pub hurt_succeeded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackResult {
    pub strength_scale: f32,
    pub base_damage: f32,
    pub enchanted_bonus: f32,
    pub total_damage: f32,
    pub critical: bool,
    pub sprint_knockback: bool,
    pub sweep: bool,
    pub exhaustion_added: f32,
    pub reset_attack_ticker: bool,
    pub knockback: ExtraKnockbackResult,
}

/// Evaluates `Player.attack(Entity)` damage scaling, crit/sprint/sweep gates,
/// `causeExtraKnockback` slowdown/sprint clear, ticker reset, and exhaustion.
pub fn resolve_player_attack(input: AttackInput, next_double: impl FnMut() -> f64) -> AttackResult {
    let scale = attack_strength_scale(input.attack_strength_ticker, 0.5, input.attack_speed);
    let enchanted_bonus = scale * (input.enchanted_full_damage - input.base_attack_damage);
    let mut base = input.base_attack_damage * base_damage_scale_factor(scale);

    // @76: `onAttack()` resets ticker before deflection / hit checks!
    let reset_attack_ticker = true;

    let mut critical = false;
    let mut sprint_knockback = false;
    let mut sweep = false;
    let mut exhaustion_added = 0.0f32;

    if base > 0.0 || enchanted_bonus > 0.0 {
        let full_strength = scale > FULL_STRENGTH_THRESHOLD;
        sprint_knockback = input.attacker_sprinting && full_strength;
        base += input.item_damage_bonus;
        critical = full_strength
            && can_critical_attack(
                input.attacker_fall_distance,
                input.attacker_on_ground,
                input.attacker_on_climbable,
                input.attacker_in_water,
                input.attacker_mobility_restricted,
                input.attacker_is_passenger,
                input.target_is_living,
                input.attacker_sprinting,
            );
        if critical {
            base *= CRIT_MULTIPLIER;
        }
        sweep = is_sweep_attack(
            full_strength,
            critical,
            sprint_knockback,
            input.attacker_on_ground,
            input.attacker_known_movement,
            input.attacker_speed,
            input.holding_sword,
        );
    }

    let total_damage = base + enchanted_bonus;
    let extra_kb = input.weapon_knockback + if sprint_knockback { SPRINT_KNOCKBACK_BONUS } else { 0.0 };

    let knockback = if (base > 0.0 || enchanted_bonus > 0.0) && input.hurt_succeeded {
        exhaustion_added = ATTACK_EXHAUSTION;
        cause_extra_knockback(
            input.attacker_velocity,
            input.attacker_sprinting,
            input.attacker_yaw,
            input.target_velocity,
            input.target_on_ground,
            input.target_is_living,
            input.target_kb_resistance,
            input.target_is_server_player,
            extra_kb,
            next_double,
        )
    } else {
        ExtraKnockbackResult {
            attacker_velocity: input.attacker_velocity,
            attacker_sprinting: input.attacker_sprinting,
            target_packet_velocity: input.target_velocity,
            target_server_velocity: input.target_velocity,
            knockback_applied: false,
        }
    };

    AttackResult {
        strength_scale: scale,
        base_damage: base,
        enchanted_bonus,
        total_damage,
        critical,
        sprint_knockback,
        sweep,
        exhaustion_added,
        reset_attack_ticker,
        knockback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_scale_sprint_slowdown_and_kb_gates() {
        // Default player attack speed 4.0 -> delay 5.0f.
        assert_eq!(attack_strength_delay(4.0), 5.0);
        assert_eq!(attack_strength_scale(5, 0.5, 4.0), 1.0);
        // Scale 0.5 (ticker = 2 -> (2 + 0.5) / 5 = 0.5): below 0.9f sprint-KB gate!
        assert_eq!(attack_strength_scale(2, 0.5, 4.0), 0.5);
        assert_eq!(base_damage_scale_factor(0.5), 0.2f32 + 0.25f32 * 0.8f32);

        let base_in = AttackInput {
            base_attack_damage: 1.0,
            enchanted_full_damage: 1.0,
            item_damage_bonus: 0.0,
            attack_strength_ticker: 5,
            attack_speed: 4.0,
            attacker_velocity: Vec3::new(0.2, 0.0, 0.4),
            attacker_known_movement: Vec3::default(),
            attacker_speed: 0.13,
            attacker_yaw: 0.0,
            attacker_on_ground: true,
            attacker_on_climbable: false,
            attacker_in_water: false,
            attacker_mobility_restricted: false,
            attacker_is_passenger: false,
            attacker_fall_distance: 0.0,
            attacker_sprinting: true,
            holding_sword: false,
            weapon_knockback: 0.0,
            target_is_living: true,
            target_is_server_player: false,
            target_on_ground: true,
            target_velocity: Vec3::default(),
            target_kb_resistance: 0.0,
            hurt_succeeded: true,
        };

        // Sprint + scale=1.0 -> extra_kb = 0.5f -> attacker slowed *(0.6, 1, 0.6) & sprint cleared!
        let r_sprint = resolve_player_attack(base_in, || 0.5);
        assert!(r_sprint.sprint_knockback);
        assert_eq!(r_sprint.knockback.attacker_velocity, Vec3::new(0.12, 0.0, 0.24));
        assert!(!r_sprint.knockback.attacker_sprinting);
        assert_eq!(r_sprint.exhaustion_added, 0.1f32);

        // Sprint + scale=0.5 -> extra_kb = 0.0 -> NO attacker slowdown and sprint stays true!
        let r_half = resolve_player_attack(AttackInput { attack_strength_ticker: 2, ..base_in }, || 0.5);
        assert!(!r_half.sprint_knockback);
        assert_eq!(r_half.knockback.attacker_velocity, Vec3::new(0.2, 0.0, 0.4));
        assert!(r_half.knockback.attacker_sprinting);
    }
}
