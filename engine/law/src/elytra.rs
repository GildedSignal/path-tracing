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

// ============================================================================
// Part 11: Elytra: when it starts, stops, and breaks
// ============================================================================

/// Default Mojang 26.2 elytra `MAX_DAMAGE`.
pub const ELYTRA_MAX_DAMAGE: i32 = 432;

/// `EquipmentSlot.values()` declaration order (`@0-80`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EquipmentSlot {
    Mainhand = 0,
    Offhand = 1,
    Feet = 2,
    Legs = 3,
    Chest = 4,
    Head = 5,
    Body = 6,
    Saddle = 7,
}

impl EquipmentSlot {
    pub const VALUES: [Self; 8] = [
        Self::Mainhand,
        Self::Offhand,
        Self::Feet,
        Self::Legs,
        Self::Chest,
        Self::Head,
        Self::Body,
        Self::Saddle,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GliderSlotStack {
    pub has_glider: bool,
    pub equippable_slot: Option<EquipmentSlot>,
    pub damageable: bool,
    pub damage: i32,
    pub max_damage: i32,
}

impl GliderSlotStack {
    pub const fn elytra(damage: i32) -> Self {
        Self {
            has_glider: true,
            equippable_slot: Some(EquipmentSlot::Chest),
            damageable: true,
            damage,
            max_damage: ELYTRA_MAX_DAMAGE,
        }
    }
}

/// `ItemStack.nextDamageWillBreak()` (`@0-25`).
pub const fn next_damage_will_break(damageable: bool, damage: i32, max_damage: i32) -> bool {
    damageable && damage >= max_damage - 1
}

/// `LivingEntity.canGlideUsing(ItemStack, EquipmentSlot)` (`@0-37`).
pub fn can_glide_using(stack: &GliderSlotStack, slot: EquipmentSlot) -> bool {
    stack.has_glider
        && stack.equippable_slot == Some(slot)
        && !next_damage_will_break(stack.damageable, stack.damage, stack.max_damage)
}

/// `LivingEntity.canGlide()` (`@0-68`).
/// Water (`is_in_water`) and `SLOW_FALLING` do NOT block `canGlide`; only `on_ground`,
/// `is_passenger`, `MobEffects.LEVITATION`, and needing >= 1 qualifying non-breaking glider do.
pub fn living_can_glide(
    on_ground: bool,
    is_passenger: bool,
    has_levitation: bool,
    slots: &[Option<GliderSlotStack>; 8],
) -> bool {
    if on_ground || is_passenger || has_levitation {
        return false;
    }
    EquipmentSlot::VALUES.into_iter().any(|slot| {
        slots[slot as usize]
            .as_ref()
            .is_some_and(|stack| can_glide_using(stack, slot))
    })
}

/// `Player.canGlide()` (`@0-16`): creative/spectator `abilities.flying` blocks gliding.
pub fn player_can_glide(
    abilities_flying: bool,
    on_ground: bool,
    is_passenger: bool,
    has_levitation: bool,
    slots: &[Option<GliderSlotStack>; 8],
) -> bool {
    !abilities_flying && living_can_glide(on_ground, is_passenger, has_levitation, slots)
}

/// `Player.tryToStartFallFlying()` (`@0-22`): client+server entry gate (sets flag 7 when true).
/// Note: water (`is_in_water`) blocks starting here, NOT `canGlide()`.
pub const fn try_to_start_fall_flying(
    is_fall_flying: bool,
    can_glide: bool,
    is_in_water: bool,
) -> bool {
    !is_fall_flying && can_glide && !is_in_water
}

/// `LocalPlayer.aiStep` jump-edge gate (`@875-929`) before `tryToStartFallFlying()`.
pub const fn ai_step_start_fall_flying(
    was_jumping: bool,
    jump_pressed: bool,
    consumed_by_mayfly_double_tap: bool,
    on_climbable: bool,
    can_start_fall_flying: bool,
) -> bool {
    !was_jumping
        && jump_pressed
        && !consumed_by_mayfly_double_tap
        && !on_climbable
        && can_start_fall_flying
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FallFlyingUpdateResult {
    pub worn_slot: Option<EquipmentSlot>,
    pub emitted_glide_game_event: bool,
    pub cleared_fall_flying: bool,
}

/// `LivingEntity.updateFallFlying()` (`@0-135`).
/// Called BEFORE `aiStep`'s `++fallFlyTicks` increment; does not mutate `fall_fly_ticks`.
/// On `!can_glide`: clears `flag7_fall_flying` and returns without wear or gameEvent.
/// When `can_glide && is_server_level`:
/// - fires `gameEvent(ELYTRA_GLIDE)` whenever `(fall_fly_ticks + 1) % 10 == 0`;
/// - wears (`+1` damage) the first `EquipmentSlot::VALUES` glider slot passing `can_glide_using`
///   whenever `((fall_fly_ticks + 1) / 10) % 2 == 0` (i.e. `fall_fly_ticks = 19, 39, 59, ...`).
pub fn update_fall_flying(
    flag7_fall_flying: &mut bool,
    fall_fly_ticks: i32,
    abilities_flying: bool,
    on_ground: bool,
    is_passenger: bool,
    has_levitation: bool,
    is_server_level: bool,
    slots: &mut [Option<GliderSlotStack>; 8],
) -> FallFlyingUpdateResult {
    let can_glide = player_can_glide(
        abilities_flying,
        on_ground,
        is_passenger,
        has_levitation,
        slots,
    );
    if !can_glide {
        let was_flying = *flag7_fall_flying;
        *flag7_fall_flying = false;
        return FallFlyingUpdateResult {
            worn_slot: None,
            emitted_glide_game_event: false,
            cleared_fall_flying: was_flying,
        };
    }

    let mut worn_slot = None;
    let mut emitted_glide_game_event = false;
    if *flag7_fall_flying && is_server_level {
        let i = fall_fly_ticks + 1;
        if i % 10 == 0 {
            if (i / 10) % 2 == 0 {
                for slot in EquipmentSlot::VALUES {
                    if let Some(stack) = slots[slot as usize].as_mut() {
                        if can_glide_using(stack, slot) {
                            if stack.damageable {
                                stack.damage += 1;
                            }
                            worn_slot = Some(slot);
                            break;
                        }
                    }
                }
            }
            emitted_glide_game_event = true;
        }
    }

    FallFlyingUpdateResult {
        worn_slot,
        emitted_glide_game_event,
        cleared_fall_flying: false,
    }
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

    #[test]
    fn part11_elytra_start_and_stop_witnesses() {
        // evidence/probe-output.txt:10-25 & check-evidence/check-output.txt:4-16
        assert!(!next_damage_will_break(true, 429, ELYTRA_MAX_DAMAGE));
        assert!(!next_damage_will_break(true, 430, ELYTRA_MAX_DAMAGE));
        assert!(next_damage_will_break(true, 431, ELYTRA_MAX_DAMAGE));
        assert!(next_damage_will_break(true, 432, ELYTRA_MAX_DAMAGE));
        // Undamageable glider does not break even at max_damage-1 (check-output.txt:13-15)
        assert!(!next_damage_will_break(false, 431, ELYTRA_MAX_DAMAGE));

        // canGlideUsing checks stack.has_glider && slot match && !next_damage_will_break
        assert!(can_glide_using(&GliderSlotStack::elytra(430), EquipmentSlot::Chest));
        assert!(!can_glide_using(&GliderSlotStack::elytra(431), EquipmentSlot::Chest));
        for wrong in [
            EquipmentSlot::Mainhand,
            EquipmentSlot::Offhand,
            EquipmentSlot::Feet,
            EquipmentSlot::Legs,
            EquipmentSlot::Head,
            EquipmentSlot::Body,
            EquipmentSlot::Saddle,
        ] {
            assert!(!can_glide_using(&GliderSlotStack::elytra(0), wrong), "slot={wrong:?}");
        }
        // Head-equippable glider works only from Head, not Chest (check-output.txt:10-12)
        let head_glider = GliderSlotStack {
            has_glider: true,
            equippable_slot: Some(EquipmentSlot::Head),
            damageable: false,
            damage: 431,
            max_damage: ELYTRA_MAX_DAMAGE,
        };
        assert!(can_glide_using(&head_glider, EquipmentSlot::Head));
        assert!(!can_glide_using(&head_glider, EquipmentSlot::Chest));

        // Slot helper
        let slots_with = |slot: EquipmentSlot, stack: GliderSlotStack| -> [Option<GliderSlotStack>; 8] {
            let mut s = [None; 8];
            s[slot as usize] = Some(stack);
            s
        };

        // G1..G8 & A4-A5 (probe-output.txt:27-48 & check-output.txt:17-20)
        let good_slots = slots_with(EquipmentSlot::Chest, GliderSlotStack::elytra(0));
        // canGlide: true in air even in water (G6) and with slow falling (A5); false on_ground / passenger / levitation / creative flying
        assert!(living_can_glide(false, false, false, &good_slots));
        assert!(!living_can_glide(true, false, false, &good_slots));
        assert!(!living_can_glide(false, true, false, &good_slots));
        assert!(!living_can_glide(false, false, true, &good_slots));
        assert!(!player_can_glide(true, false, false, false, &good_slots));
        assert!(player_can_glide(false, false, false, false, &good_slots));

        // tryToStartFallFlying blocks in water even when canGlide is true (probe-output.txt:36-40)
        let cg = player_can_glide(false, false, false, false, &good_slots);
        assert!(try_to_start_fall_flying(false, cg, false));
        assert!(!try_to_start_fall_flying(false, cg, true));
        assert!(!try_to_start_fall_flying(true, cg, false));

        // LocalPlayer jump-edge gate requires !was_jumping && jump && !double_tap && !on_climbable
        assert!(ai_step_start_fall_flying(false, true, false, false, true));
        assert!(!ai_step_start_fall_flying(true, true, false, false, true));
        assert!(!ai_step_start_fall_flying(false, true, true, false, true));
        assert!(!ai_step_start_fall_flying(false, true, false, true, true));

        // 65-tick durability & ELYTRA_GLIDE cadence (probe-output.txt:52-79):
        // gameEvent at fallFlyTicks=9,19,29,39,49,59; wear at fallFlyTicks=19,39,59
        let mut slots = slots_with(EquipmentSlot::Chest, GliderSlotStack::elytra(0));
        let mut flag7 = true;
        let mut worn_at = Vec::new();
        let mut events_at = Vec::new();
        for t in 0..65 {
            let step = update_fall_flying(&mut flag7, t, false, false, false, false, true, &mut slots);
            if step.worn_slot.is_some() {
                worn_at.push(t);
            }
            if step.emitted_glide_game_event {
                events_at.push(t);
            }
        }
        assert_eq!(worn_at, vec![19, 39, 59]);
        assert_eq!(events_at, vec![9, 19, 29, 39, 49, 59]);
        assert_eq!(slots[EquipmentSlot::Chest as usize].unwrap().damage, 3);
        assert!(flag7);

        // Near-break boundary starting at dmg=429 (check-output.txt:22-77):
        // wears to 430 at t=19, wears to 431 at t=39 (flag7 still true on that tick!),
        // then at t=40 canGlide is false -> flag7 clears with NO wear or gameEvent.
        let mut near_break = slots_with(EquipmentSlot::Chest, GliderSlotStack::elytra(429));
        let mut fly = true;
        for t in 0..=40 {
            let res = update_fall_flying(&mut fly, t, false, false, false, false, true, &mut near_break);
            if t < 19 {
                assert_eq!(near_break[EquipmentSlot::Chest as usize].unwrap().damage, 429);
                assert!(fly);
            } else if t < 39 {
                assert_eq!(near_break[EquipmentSlot::Chest as usize].unwrap().damage, 430);
                assert!(fly);
            } else if t == 39 {
                assert_eq!(near_break[EquipmentSlot::Chest as usize].unwrap().damage, 431);
                assert_eq!(res.worn_slot, Some(EquipmentSlot::Chest));
                assert!(fly);
            } else {
                // t == 40
                assert!(!fly);
                assert!(res.cleared_fall_flying);
                assert_eq!(res.worn_slot, None);
                assert!(!res.emitted_glide_game_event);
            }
        }
    }
}
