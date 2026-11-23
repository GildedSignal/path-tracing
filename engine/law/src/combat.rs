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

// ============================================================================
// Shield blocking, crit and sweep gates, mace smash (folded from GLM dossier
// local-20261122-145222-30104-1-0-port-shield-crit-sweep-and-mace)
// ============================================================================
// Melee combat laws from the 26.2 server jar: shield blocking
// (`LivingEntity.applyItemBlocking`, `BlocksAttacks`), the crit and sweep
// gates of `Player.attack`, and the mace smash (`MaceItem`). Every constant
// and comparison is cited by class and bytecode offset; boundary values are
// the jar's own, run by `lab/results/local-20261122-145222-30104-1-0-*
// /evidence/harness-run.txt`. Knockback itself, attack-cooldown scaling and
// `causeExtraKnockback` live in `movement.rs` and are not repeated here.


// ---------------------------------------------------------------- shield ----

// BlocksAttacks record, shield values read from the live registry
// (Items.SHIELD, delayed BLOCKS_ATTACKS component): blockDelaySeconds=0.25,
// disableCooldownScale=1.0, one DamageReduction{horizontalBlockingAngle=90.0,
// base=0.0, factor=1.0}, itemDamage{threshold=3.0, base=1.0, factor=1.0},
// bypassedBy = damage_type tag minecraft:bypasses_shield.
pub const SHIELD_BLOCK_DELAY_SECONDS: f32 = 0.25;
pub const SHIELD_DISABLE_COOLDOWN_SCALE: f32 = 1.0;
pub const SHIELD_HORIZONTAL_BLOCKING_ANGLE: f32 = 90.0;
pub const SHIELD_REDUCTION_BASE: f32 = 0.0;
pub const SHIELD_REDUCTION_FACTOR: f32 = 1.0;
// Weapon.AXE_DISABLES_BLOCKING_FOR_SECONDS; every axe item carries
// Weapon{itemDamagePerAttack=2, disableBlockingForSeconds=5.0}, the mace and
// swords carry 0.0 (they never disable a shield).
pub const AXE_DISABLES_BLOCKING_FOR_SECONDS: f32 = 5.0;

/// `BlocksAttacks.blockDelayTicks`: `Math.round(blockDelaySeconds * 20.0f)`.
/// `Math.round(F)I` is floor(x + 0.5f); the codec pins the input non-negative.
pub fn block_delay_ticks(block_delay_seconds: f32) -> i32 {
    (block_delay_seconds * 20.0f32 + 0.5f32).floor() as i32
}

/// `LivingEntity.getItemBlockingWith` offsets 48-53: blocking is active once
/// `getUseDuration(stack, self) - useItemRemaining >= blockDelayTicks()`.
/// `ticks_used` is that elapsed use time.
pub fn item_blocking_ready(ticks_used: i32, block_delay_seconds: f32) -> bool {
    ticks_used >= block_delay_ticks(block_delay_seconds)
}

/// The incoming-angle of `LivingEntity.applyItemBlocking` offsets 108-170:
/// `angle = acos(flat_dir.dot(view))` where `flat_dir` is the horizontal
/// source-to-blocker direction normalized and `view` is the blocker's
/// `calculateViewVector(0.0f, yHeadRot)`. A damage source with no position
/// yields the float pi widened: 3.1415927410125732.
pub fn incoming_angle(view: Vec3, source_pos: Option<Vec3>, blocker_pos: Vec3) -> f64 {
    match source_pos {
        Some(src) => {
            let rel = src.minus(blocker_pos);
            let flat = Vec3::new(rel.x, 0.0, rel.z).normalize();
            (flat.x * view.x + flat.y * view.y + flat.z * view.z).acos()
        }
        None => 3.1415927410125732f64, // ldc2_w #1851
    }
}

/// `BlocksAttacks$DamageReduction.resolve` offsets 0-14: the gate is
/// `angle > (0.017453292f * horizontalBlockingAngle)` widened to double, so
/// exactly-at-the-limit blocks. Then `Mth.clamp(base + factor * amount, 0, amount)`
/// over the summed reductions (`BlocksAttacks.resolveBlockedDamage` offsets 0-59).
pub fn blocked_amount(angle_rad: f64, amount: f32, base: f32, factor: f32,
                      horizontal_blocking_angle: f32) -> f32 {
    let limit = (0.017453292f32 * horizontal_blocking_angle) as f64;
    if angle_rad > limit {
        return 0.0;
    }
    (base + factor * amount).clamp(0.0, amount)
}

/// The full shield gate with the shipped shield's numbers: full block of any
/// amount whose incoming angle is at most 90 degrees.
pub fn shield_blocks(angle_rad: f64, amount: f32) -> f32 {
    if amount <= 0.0 { return 0.0; } // applyItemBlocking offsets 0-7
    blocked_amount(angle_rad, amount, SHIELD_REDUCTION_BASE, SHIELD_REDUCTION_FACTOR,
                   SHIELD_HORIZONTAL_BLOCKING_ANGLE)
}

/// `BlocksAttacks.disableBlockingForTicks`: `s = seconds * disableCooldownScale`;
/// `s > 0f ? Math.round(s * 20.0f) : 0`. An axe hit (5.0 s, scale 1.0) is 100
/// ticks; 0.02 s rounds to 0 and disables nothing, 0.025 s is 1 tick.
pub fn disable_ticks(disable_seconds: f32, disable_cooldown_scale: f32) -> i32 {
    let s = disable_seconds * disable_cooldown_scale;
    if s > 0.0f32 {
        (s * 20.0f32 + 0.5f32).floor() as i32
    } else {
        0
    }
}

/// The disable side of `Player.blockUsingItem` offsets 37-63: the ATTACKER's
/// weapon carries the seconds (`LivingEntity.getSecondsToDisableBlocking`:
/// `weapon.get(WEAPON).disableBlockingForSeconds()` when the weapon is the
/// active item, else 0); the victim's shield scales it. `disable` then puts the
/// victim's shield on its cooldown group for the ticks above (players only) and
/// `stopUsingItem` — while the group is on cooldown the shield cannot be raised.
pub fn axe_disables_shield(attacker_disable_seconds: f32,
                           victim_disable_cooldown_scale: f32) -> i32 {
    if attacker_disable_seconds <= 0.0f32 {
        return 0; // offsets 43-47: seconds > 0f required
    }
    disable_ticks(attacker_disable_seconds, victim_disable_cooldown_scale)
}

// ------------------------------------------------------------- crit gate ----

/// `Player.canCriticalAttack` (offsets 0-63) plus the strength gate of
/// `Player.attack` offsets 101-115 (`getAttackStrengthScale(0.5f) > 0.9f`).
/// `fallDistance > 0.0` is dcmpl/ifle: NaN compares as less, so NaN does not
/// crit. `isMobilityRestricted` is exactly `hasEffect(MobEffects.BLINDNESS)`
/// (`LivingEntity.isMobilityRestricted` offsets 0-7).
pub struct CritGate {
    pub fall_distance: f64,
    pub on_ground: bool,
    pub on_climbable: bool,
    pub in_water: bool,
    pub blind: bool,
    pub passenger: bool,
    pub target_is_living: bool,
    pub sprinting: bool,
}

pub fn is_critical(g: &CritGate) -> bool {
    g.fall_distance > 0.0
        && !g.on_ground
        && !g.on_climbable
        && !g.in_water
        && !g.blind
        && !g.passenger
        && g.target_is_living
        && !g.sprinting
}

// ------------------------------------------------------------ sweep gate ----

pub fn is_sweep(strong: bool, crit: bool, sprint_knockback: bool, on_ground: bool,
                known_horizontal_sqr: f64, speed: f32, main_hand_is_sword: bool) -> bool {
    strong && !crit && !sprint_knockback && on_ground && {
        let limit = (speed as f64) * 2.5;
        known_horizontal_sqr < limit * limit
    } && main_hand_is_sword
}

// ------------------------------------------------------------ mace smash ----

pub const SMASH_FALL_THRESHOLD: f64 = 1.5;   // MaceItem.SMASH_ATTACK_FALL_THRESHOLD

/// `MaceItem.canSmashAttack`: `fallDistance > 1.5 && !isFallFlying()`
/// (dcmpl/ifle, so NaN does not smash).
pub fn can_smash(fall_distance: f64, fall_flying: bool) -> bool {
    fall_distance > SMASH_FALL_THRESHOLD && !fall_flying
}

/// `MaceItem.getAttackDamageBonus` offsets 36-111: 4*fall up to 3.0,
/// 12 + 2*(fall-3) up to 8.0, then 22 + fall - 8.
pub fn smash_bonus(fall_distance: f64) -> f64 {
    if fall_distance <= 3.0 {
        4.0 * fall_distance
    } else if fall_distance <= 8.0 {
        12.0 + 2.0 * (fall_distance - 3.0)
    } else {
        22.0 + fall_distance - 8.0
    }
}

/// Density from `data/minecraft/enchantment/density.json`: effect
/// `minecraft:smash_damage_per_fallen_block`, add, linear
/// base=0.5 per_level_above_first=0.5. `MaceItem.getAttackDamageBonus` offsets
/// 135-156 add `modifyFallBasedDamage(...) * fallDistance`, i.e. the per-block
/// value times the fall distance.
pub fn density_per_fallen_block(density_level: i32) -> f32 {
    if density_level <= 0 {
        0.0
    } else {
        0.5 + 0.5 * (density_level as f32 - 1.0)
    }
}

/// The mace's `Item.getAttackDamageBonus` total. Offsets 6-35 return 0 unless
/// the direct entity is a `LivingEntity` passing `canSmashAttack` (NaN or
/// gliding gives 0); the client branch (offsets 157-160) is `(float)bonus`,
/// the Density term computed in float then widened.
pub fn mace_attack_bonus(fall_distance: f64, fall_flying: bool, density_level: i32) -> f32 {
    if !can_smash(fall_distance, fall_flying) {
        return 0.0;
    }
    let bonus = smash_bonus(fall_distance)
        + (density_per_fallen_block(density_level) as f64) * fall_distance;
    bonus as f32
}

/// Wind Burst from `data/minecraft/enchantment/wind_burst.json`: a
/// `post_attack` `minecraft:explode` effect on the attacker, radius 3.5,
/// knockback multiplier [1.2, 1.75, 2.2] for levels 1-3 and
/// 1.5 + 0.35*(level-1) beyond, gated on `direct_attacker` not flying and
/// `fall_distance >= 1.5`. The impulse itself is the explosion law.
pub fn wind_burst_knockback_multiplier(level: i32) -> f32 {
    match level {
        1 => 1.2,
        2 => 1.75,
        3 => 2.2,
        _ if level > 3 => 1.5 + 0.35 * (level as f32 - 1.0),
        _ => 0.0,
    }
}

#[cfg(test)]
mod regression {
    use super::*;

    // Every expected value below was printed by the 26.2 jar itself
    // (evidence/harness-run.txt); bit-exact, like knockback_regression.

    #[test]
    fn shield_warm_up_is_five_ticks() {
        assert_eq!(block_delay_ticks(SHIELD_BLOCK_DELAY_SECONDS), 5);
        assert!(item_blocking_ready(5, SHIELD_BLOCK_DELAY_SECONDS));
        assert!(!item_blocking_ready(4, SHIELD_BLOCK_DELAY_SECONDS));
    }

    #[test]
    fn angle_gate_blocks_at_exactly_90_degrees() {
        // dots -0.01, 0.0, +0.01, 1.0 against the blocker's view vector
        let view = Vec3::new(0.0, 0.0, 1.0); // yHeadRot = 0
        let blocker = Vec3::new(0.0, 64.0, 0.0);
        let angle = |dot: f64| {
            let src = Vec3::new((1.0 - dot * dot).sqrt(), 64.0, dot).scale(10.0);
            incoming_angle(view, Some(src), blocker)
        };
        assert_eq!(shield_blocks(angle(-0.01), 10.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(shield_blocks(angle(0.0), 10.0).to_bits(), 10.0f32.to_bits());
        assert_eq!(shield_blocks(angle(0.01), 10.0).to_bits(), 10.0f32.to_bits());
        assert_eq!(shield_blocks(angle(1.0), 10.0).to_bits(), 10.0f32.to_bits());
        assert_eq!(incoming_angle(view, None, blocker).to_bits(),
                   3.1415927410125732f64.to_bits());
        assert_eq!(shield_blocks(3.1415927410125732, 10.0).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn axe_disable_is_one_hundred_ticks() {
        assert_eq!(axe_disables_shield(AXE_DISABLES_BLOCKING_FOR_SECONDS,
                                       SHIELD_DISABLE_COOLDOWN_SCALE), 100);
        assert_eq!(disable_ticks(0.02, 1.0), 0);
        assert_eq!(disable_ticks(0.025, 1.0), 1);
        assert_eq!(disable_ticks(0.0, 1.0), 0);
        assert_eq!(axe_disables_shield(0.0, 1.0), 0); // mace and swords never disable
    }

    #[test]
    fn crit_gate_flips_on_every_flag() {
        let base = CritGate {
            fall_distance: 1.0, on_ground: false, on_climbable: false, in_water: false,
            blind: false, passenger: false, target_is_living: true, sprinting: false,
        };
        assert!(is_critical(&base));
        assert!(!is_critical(&CritGate { fall_distance: 0.0, ..base }));
        assert!(!is_critical(&CritGate { fall_distance: f64::NAN, ..base }));
        assert!(!is_critical(&CritGate { on_ground: true, ..base }));
        assert!(!is_critical(&CritGate { on_climbable: true, ..base }));
        assert!(!is_critical(&CritGate { in_water: true, ..base }));
        assert!(!is_critical(&CritGate { blind: true, ..base }));
        assert!(!is_critical(&CritGate { passenger: true, ..base }));
        assert!(!is_critical(&CritGate { target_is_living: false, ..base }));
        assert!(!is_critical(&CritGate { sprinting: true, ..base }));
    }

    #[test]
    fn sweep_gate_boundary_is_strict() {
        let limit = (0.1f32 as f64) * 2.5;
        assert!(is_sweep(true, false, false, true, 0.0625, 0.1, true));
        assert!(!is_sweep(true, false, false, true, limit * limit, 0.1, true));
        assert!(!is_sweep(false, false, false, true, 0.0, 0.1, true));
        assert!(!is_sweep(true, true, false, true, 0.0, 0.1, true));
        assert!(!is_sweep(true, false, true, true, 0.0, 0.1, true));
        assert!(!is_sweep(true, false, false, false, 0.0, 0.1, true));
        assert!(!is_sweep(true, false, false, true, 0.0, 0.1, false));
    }

    #[test]
    fn mace_tiers_and_boundaries() {
        assert!(!can_smash(1.5, false));
        assert!(can_smash(1.51, false));
        assert!(!can_smash(4.0, true)); // gliding
        assert_eq!(mace_attack_bonus(1.5, false, 0).to_bits(), 0.0f32.to_bits());
        assert_eq!(mace_attack_bonus(4.0, true, 0).to_bits(), 0.0f32.to_bits()); // gliding
        assert_eq!(mace_attack_bonus(1.51, false, 0).to_bits(), 6.04f32.to_bits());
        assert_eq!(mace_attack_bonus(3.0, false, 0).to_bits(), 12.0f32.to_bits());
        assert_eq!(mace_attack_bonus(8.0, false, 0).to_bits(), 22.0f32.to_bits());
        assert_eq!(mace_attack_bonus(25.0, false, 0).to_bits(), 39.0f32.to_bits());
        assert_eq!(mace_attack_bonus(3.0000000001, false, 0).to_bits(), 12.0f32.to_bits());
        assert_eq!(mace_attack_bonus(8.0000000001, false, 0).to_bits(), 22.0f32.to_bits());
        // Density I on a 6-block fall: 0.5 * 6 = 3 more.
        assert_eq!(mace_attack_bonus(6.0, false, 1).to_bits(), 21.0f32.to_bits());
    }

    #[test]
    fn wind_burst_multiplier_table() {
        assert_eq!(wind_burst_knockback_multiplier(1), 1.2);
        assert_eq!(wind_burst_knockback_multiplier(2), 1.75);
        assert_eq!(wind_burst_knockback_multiplier(3), 2.2);
        assert_eq!(wind_burst_knockback_multiplier(4), 2.55);
        assert_eq!(wind_burst_knockback_multiplier(0), 0.0);
    }
}
