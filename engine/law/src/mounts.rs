//! `LivingEntity.travelRidden` and the mount overrides, ported from the 26.2
//! server bytecode (class, method and bytecode offsets cited per rule). This is
//! the ridden *layer* only: input transform, rotation sync, jump charge, ridden
//! speed, fall-damage split. The `travel` recurrence underneath (friction,
//! inputVector, move, gravity, drag) is the player law in `movement.rs` — the
//! mount enters it with `setSpeed(getRiddenSpeed(player))` and the mount's
//! `STEP_HEIGHT` attribute (1.0 for horses, `AbstractHorse.createBaseHorseAttributes`).
//!
//! 26.2 execution order (LivingEntity.aiStep offset 562, travelRidden offset 0):
//! when `getControllingPassenger()` is a `Player` and the mount `isAlive()`:
//! `travelRidden(player, travelVector)`:
//!   0:3   `vec3 = getRiddenInput(player, tv)`   (mount override)
//!   7:10  `tickRidden(player, vec3)`            (mount override)
//!   13:17 if `canSimulateMovement()`:
//!         `setSpeed(getRiddenSpeed(player))`    (20:26)
//!         `travel(vec3)`                        (29:31)
//!   else  `setDeltaMovement(Vec3.ZERO)`         (37:41)

use crate::math;
use crate::shapes::Vec3;

/// `PlayerRideableJumping.getPlayerJumpPendingScale(int)` — 26.2 default method.
/// `charge >= 90 -> 1.0f`, else `0.4f + (0.4f * charge) / 90.0f`.
pub fn player_jump_pending_scale(charge: i32) -> f32 {
    if charge >= 90 {
        1.0
    } else {
        0.4f32 + 0.4f32 * charge as f32 / 90.0f32
    }
}

/// `ItemBasedSteering.boostFactor()` — `1.0f + 1.15f * Mth.sin((t/T) * pi)` while boosting.
pub fn boost_factor(boosting: bool, boost_time: i32, boost_time_total: i32) -> f32 {
    if boosting {
        let t = boost_time as f32 / boost_time_total as f32;
        1.0f32 + 1.15f32 * math::sin((t * 3.1415927f32) as f64)
    } else {
        1.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MountKind {
    /// `AbstractHorse` family (horse, skeleton horse, zombie horse, donkey, mule).
    Horse,
    Pig,
    Strider,
    Camel,
    HappyGhast,
}

/// Attribute bases read from the 26.2 bytecode:
/// `AbstractHorse.createBaseHorseAttributes` (JUMP_STRENGTH 0.7d, MOVEMENT_SPEED
/// 0.22499999403953552d, STEP_HEIGHT 1.0d, SAFE_FALL_DISTANCE 6.0d,
/// FALL_DAMAGE_MULTIPLIER 0.5d), `Pig.createAttributes` (0.25d),
/// `Strider.createAttributes` (0.17499999701976776d).
#[derive(Clone, Debug)]
pub struct MountState {
    pub kind: MountKind,
    pub speed_attribute: f64,
    pub jump_strength_attribute: f64,
    pub safe_fall_distance: f64,
    pub fall_damage_multiplier: f64,
    /// AbstractHorse.playerJumpPendingScale
    pub player_jump_pending_scale: f32,
    /// AbstractHorse.isStanding / allowStandSliding
    pub is_standing: bool,
    pub allow_stand_sliding: bool,
    /// ItemBasedSteering (Pig, Strider)
    pub boosting: bool,
    pub boost_time: i32,
    pub boost_time_total: i32,
    /// Camel.dashCooldown
    pub dash_cooldown: i32,
    /// AbstractHorse.gallopSoundCounter (sound cadence only)
    pub gallop_sound_counter: i32,
    /// Mount velocity; y-rotation of the mount.
    pub velocity: Vec3,
    pub yaw: f32,
    pub on_ground: bool,
    pub is_jumping: bool,
}

impl MountState {
    pub fn horse() -> Self {
        MountState {
            kind: MountKind::Horse,
            speed_attribute: 0.224_999_994_039_535_52,
            jump_strength_attribute: 0.7,
            safe_fall_distance: 6.0,
            fall_damage_multiplier: 0.5,
            player_jump_pending_scale: 0.0,
            is_standing: false,
            allow_stand_sliding: false,
            boosting: false,
            boost_time: 0,
            boost_time_total: 0,
            dash_cooldown: 0,
            gallop_sound_counter: 0,
            velocity: Vec3::default(),
            yaw: 0.0,
            on_ground: true,
            is_jumping: false,
        }
    }

    pub fn pig() -> Self {
        let mut m = Self::horse();
        m.kind = MountKind::Pig;
        m.speed_attribute = 0.25;
        m.safe_fall_distance = 3.0;
        m.fall_damage_multiplier = 1.0;
        m
    }

    pub fn strider() -> Self {
        let mut m = Self::pig();
        m.kind = MountKind::Strider;
        m.speed_attribute = 0.174_999_997_019_767_76;
        m
    }

    pub fn camel() -> Self {
        let mut m = Self::horse();
        m.kind = MountKind::Camel;
        m.speed_attribute = 0.090_000_003_576_278_69;
        m.jump_strength_attribute = 0.419_999_986_886_978_15;
        m
    }

    pub fn happy_ghast() -> Self {
        let mut m = Self::pig();
        m.kind = MountKind::HappyGhast;
        m.speed_attribute = 0.05;
        m
    }
}

/// `AbstractHorse.getRiddenInput(Player, Vec3)` — offset 0.
/// Standing horses with no pending jump refuse to move; `xxa * 0.5f`,
/// `zza <= 0.0f -> zza * 0.25f` (only a strictly positive forward escapes).
pub fn ridden_input_horse(m: &MountState, xxa: f32, zza: f32) -> Vec3 {
    if m.on_ground && m.player_jump_pending_scale == 0.0 && m.is_standing && !m.allow_stand_sliding {
        return Vec3::default();
    }
    let x = xxa * 0.5f32;
    let mut z = zza;
    if !(z > 0.0f32) {
        z *= 0.25f32;
    }
    Vec3::new(x as f64, 0.0, z as f64)
}

/// `HappyGhast.getRiddenInput(Player, Vec3)` — pitch-driven flight input scaled
/// by `3.9000000953674316d * FLYING_SPEED`.
pub fn ridden_input_happy_ghast(xxa: f32, zza: f32, pitch: f32, jumping: bool, flying_speed: f64) -> Vec3 {
    let mut y = 0.0f32;
    let mut z = 0.0f32;
    if zza != 0.0f32 {
        let mut f6 = math::cos((pitch * 0.017_453_292f32) as f64);
        let mut f7 = -math::sin((pitch * 0.017_453_292f32) as f64);
        if !(zza >= 0.0f32) {
            f6 *= -0.5f32;
            f7 *= -0.5f32;
        }
        y = f7;
        z = f6;
    }
    if jumping {
        y += 0.5f32;
    }
    Vec3::new(xxa as f64, y as f64, z as f64).scale(3.900_000_095_367_431_6 * flying_speed)
}

/// `getRiddenSpeed(Player)` per mount — the speed `travel` runs with.
/// Horse: the raw `MOVEMENT_SPEED` attribute (AbstractHorse offset 0).
/// Pig: `attr * 0.225d * boostFactor` (Pig offset 0).
/// Strider: `attr * (suffocating ? 0.35f : 0.55f) * boostFactor`.
/// Camel: `attr + (player sprinting && jumpCooldown == 0 ? 0.1f : 0.0f)`.
pub fn ridden_speed(m: &MountState, player_sprinting: bool, suffocating: bool) -> f32 {
    match m.kind {
        MountKind::Horse => m.speed_attribute as f32,
        MountKind::Pig => (m.speed_attribute * 0.225 * boost_factor(m.boosting, m.boost_time, m.boost_time_total) as f64) as f32,
        MountKind::Strider => {
            let f = if suffocating { 0.35f32 } else { 0.55f32 };
            (m.speed_attribute * (f as f64) * boost_factor(m.boosting, m.boost_time, m.boost_time_total) as f64) as f32
        }
        MountKind::Camel => {
            let bonus = if player_sprinting && m.dash_cooldown == 0 { 0.1f32 } else { 0.0f32 };
            m.speed_attribute as f32 + bonus
        }
        MountKind::HappyGhast => m.speed_attribute as f32,
    }
}

/// `AbstractHorse.executeRidersJump(float, Vec3)` — offset 0.
/// `getJumpPower(scale)` = `JUMP_STRENGTH * scale * blockJumpFactor + jumpBoost`
/// (LivingEntity.getJumpPower(float): `(attr d2f * scale) * 1.0f + 0.0f` without
/// effects); then, moving forward only (`vec3.z > 0.0`), the boost
/// `(-0.4f * sin) * scale`, `(0.4f * cos) * scale` along the mount yaw.
pub fn execute_riders_jump_horse(m: &mut MountState, scale: f32, input_z: f64) {
    let jump_power = (m.jump_strength_attribute as f32 * scale) * 1.0f32 + 0.0f32;
    let v = m.velocity;
    m.velocity = Vec3::new(v.x, jump_power as f64, v.z);
    if input_z > 0.0 {
        let (s, c) = math::sin_cos_degrees(m.yaw);
        m.velocity = m.velocity.plus(Vec3::new(
            (-0.4f32 * s * scale) as f64,
            0.0,
            (0.4f32 * c * scale) as f64,
        ));
    }
}

/// `Camel.executeRidersJump` — the dash: the horizontal look vector scaled by
/// `22.2222f * scale * speed * blockSpeedFactor`, plus `1.4285f * scale * jumpPower`
/// up, then `dashCooldown = 55`.
pub fn execute_riders_jump_camel(m: &mut MountState, scale: f32) {
    let jump_power = (m.jump_strength_attribute as f32 * 1.0f32) * 1.0f32 + 0.0f32;
    // getLookAngle().multiply(1,0,1).normalize() with pitch 0 (dash is horizontal)
    let (s, c) = math::sin_cos_degrees(m.yaw);
    let look = Vec3::new((-s) as f64, 0.0, c as f64).normalize();
    let horizontal = look.scale((22.2222f32 * scale) as f64 * m.speed_attribute * 1.0f32 as f64);
    let delta = horizontal.plus(Vec3::new(0.0, (1.4285f32 * scale) as f64 * jump_power as f64, 0.0));
    m.velocity = m.velocity.plus(delta);
    m.dash_cooldown = 55;
}

/// `HappyGhast.tickRidden` — the mount's yaw chases the rider's by
/// `0.08 * wrapDegrees(target - yaw)`.
pub fn tick_ridden_happy_ghast(m: &mut MountState, player_yaw: f32, player_pitch: f32) {
    let delta = crate::math::wrap_degrees(player_yaw - m.yaw);
    m.yaw = m.yaw + delta * 0.08f32;
    let _ = player_pitch * 0.5f32;
}

