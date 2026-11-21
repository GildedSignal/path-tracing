//! Item consumption (food & potions) and Trident Riptide / auto spin attack laws
//! (Mojang 26.2 server bytecode).
//!
//! References:
//! - `LivingEntity.tick` (`updatingUsingItem` `@5`, `aiStep` `@178`), `ServerPlayer.doTick` (`@15`, `@46`)
//! - `Consumable` (`startConsuming`, `canConsume`, `consumeTicks`, `onConsume`)
//! - `Player.canEat`, `FoodData.eat(FoodProperties)`, `FoodData.eat(int, float)`, `FoodConstants`
//! - `PotionContents.onConsume`, `MobEffectInstance.withScaledDuration`
//! - `TridentItem.use` (`@0-49`), `TridentItem.releaseUsing` (`@0-418`)
//! - `EnchantmentHelper.getTridentSpinAttackStrength`, `LevelBasedValue$Linear`, `AddValue`
//! - `Player.startAutoSpinAttack`, `Player.getDesiredPose`, `Avatar.POSES`
//! - `LivingEntity.aiStep` (`@789-813`), `LivingEntity.checkAutoSpinAttack` (`@0-152`)

use crate::{math, Aabb, Vec3};

// ============================================================================
// 1. Food & potion consumption
// ============================================================================

/// Default consume duration (`1.6f` -> `32` ticks) for food and drink items.
pub const DEFAULT_CONSUME_SECONDS: f32 = 1.6;
pub const DEFAULT_CONSUME_TICKS: i32 = 32;
/// `Items.DRIED_KELP` (`0.8f` -> `16` ticks).
pub const DRIED_KELP_CONSUME_TICKS: i32 = 16;
/// `Items.HONEY_BOTTLE` (`2.0f` -> `40` ticks).
pub const HONEY_BOTTLE_CONSUME_TICKS: i32 = 40;
/// `FoodConstants.MAX_FOOD` and `MAX_SATURATION` upper bound (`20`).
pub const MAX_FOOD_LEVEL: i32 = 20;

/// One item's registered consumption facts (`Consumable` + `FoodProperties`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Consumable {
    /// `Consumable.consumeTicks()` = `(int)(consumeSeconds * 20.0f)`.
    pub consume_ticks: i32,
    /// True iff `DataComponents.FOOD` is present; only then does `canConsume` gate on hunger.
    pub has_food: bool,
    /// `FoodProperties.nutrition`.
    pub nutrition: i32,
    /// Baked `FoodProperties.saturation` (already multiplied by `2.0f` during registration).
    /// `FoodProperties.onConsume -> FoodData.eat(FoodProperties)` adds this verbatim.
    pub saturation: f32,
    /// `FoodProperties.canAlwaysEat`.
    pub can_always_eat: bool,
    /// Whether completion happens on release rather than countdown reaching zero.
    pub use_on_release: bool,
}

/// Registered item constants from 26.2 `Items` + `Consumables` (`FoodProperties.Builder.build()`
/// computes `saturation_by_modifier` once at registration).
pub const GOLDEN_CARROT: Consumable = Consumable {
    consume_ticks: 32,
    has_food: true,
    nutrition: 6,
    saturation: f32::from_bits(0x4166_6667), // saturation_by_modifier(6, 1.2f)
    can_always_eat: false,
    use_on_release: false,
};
pub const COOKED_BEEF: Consumable = Consumable {
    consume_ticks: 32,
    has_food: true,
    nutrition: 8,
    saturation: f32::from_bits(0x414c_cccd), // saturation_by_modifier(8, 0.8f)
    can_always_eat: false,
    use_on_release: false,
};
pub const GOLDEN_APPLE: Consumable = Consumable {
    consume_ticks: 32,
    has_food: true,
    nutrition: 4,
    saturation: f32::from_bits(0x4119_999a), // saturation_by_modifier(4, 1.2f)
    can_always_eat: true,
    use_on_release: false,
};
pub const ENCHANTED_GOLDEN_APPLE: Consumable = Consumable {
    consume_ticks: 32,
    has_food: true,
    nutrition: 4,
    saturation: f32::from_bits(0x4119_999a),
    can_always_eat: true,
    use_on_release: false,
};
pub const DRIED_KELP: Consumable = Consumable {
    consume_ticks: 16,
    has_food: true,
    nutrition: 1,
    saturation: f32::from_bits(0x3f19_999a), // saturation_by_modifier(1, 0.3f)
    can_always_eat: false,
    use_on_release: false,
};
pub const HONEY_BOTTLE: Consumable = Consumable {
    consume_ticks: 40,
    has_food: true,
    nutrition: 6,
    saturation: f32::from_bits(0x3f99_999a), // saturation_by_modifier(6, 0.1f)
    can_always_eat: true,
    use_on_release: false,
};
/// Potions / milk / ominous bottle (`has_food = false`, never gated by `Player.canEat`).
pub const POTION: Consumable = Consumable {
    consume_ticks: 32,
    has_food: false,
    nutrition: 0,
    saturation: 0.0,
    can_always_eat: false,
    use_on_release: false,
};

/// Swiftness potion effect `(amplifier, duration_ticks)`.
pub const SWIFTNESS_POTION_EFFECTS: &[(i32, i32)] = &[(0, 3600)];

/// `Consumable.consumeTicks()`: `(int)(consumeSeconds * 20.0f)` (`fmul; f2i` truncation).
#[inline]
pub const fn consume_ticks(consume_seconds: f32) -> i32 {
    (consume_seconds * 20.0f32) as i32
}

/// `FoodConstants.saturationByModifier(int, float)`: `(float)nutrition * modifier * 2.0f`.
#[inline]
pub const fn saturation_by_modifier(nutrition: i32, modifier: f32) -> f32 {
    (nutrition as f32 * modifier) * 2.0f32
}

/// `MobEffectInstance.withScaledDuration(float)`:
/// preserves infinite (`duration == -1`) and `0`, otherwise `Math.max(Mth.floor((float)duration * scale), 1)`.
#[inline]
pub fn scale_effect_duration(duration: i32, scale: f32) -> i32 {
    if duration == -1 || duration == 0 {
        duration
    } else {
        math::floor_f32((duration as f32) * scale).max(1)
    }
}

/// `Player.canEat(boolean ignoreHunger)`: `abilities.invulnerable || ignoreHunger || foodData.needsFood()`.
/// In 26.2 this gates on `invulnerable`, not `instabuild`.
#[inline]
pub const fn can_eat(food_level: i32, can_always_eat: bool, invulnerable: bool) -> bool {
    invulnerable || can_always_eat || food_level < MAX_FOOD_LEVEL
}

/// `Consumable.canConsume`: runs `Player.canEat` iff `item.has_food`; potions/drinks are always allowed.
#[inline]
pub const fn can_consume(item: &Consumable, food_level: i32, invulnerable: bool) -> bool {
    if item.has_food {
        can_eat(food_level, item.can_always_eat, invulnerable)
    } else {
        true
    }
}

/// `LivingEntity.startUsingItem`: sets `useItemRemaining = item.consume_ticks`.
/// No decrement happens on the tick the input lands.
#[inline]
pub const fn start_using_item(item: &Consumable) -> i32 {
    item.consume_ticks
}

/// `LivingEntity.updateUsingItem`: decrements `remaining` and returns `true` when the item
/// completes on this server tick (`remaining == 0 && !is_client_side && !use_on_release`).
#[inline]
pub fn use_item_tick(remaining: &mut i32, is_client_side: bool, use_on_release: bool) -> bool {
    *remaining -= 1;
    *remaining == 0 && !is_client_side && !use_on_release
}

/// `LivingEntity.stopUsingItem` / `releaseUsingItem` abort: resets `remaining = 0` without commit.
#[inline]
pub fn stop_using_item(remaining: &mut i32) {
    *remaining = 0;
}

/// `FoodData.add(int nutrition, float saturation)` reached from `FoodData.eat(FoodProperties)`:
/// clamps `food_level` in `0..=20` FIRST, then clamps `saturation` in `0.0..=(food_level as f32)`.
#[inline]
pub fn eat_food(food_level: &mut i32, saturation: &mut f32, nutrition: i32, baked_saturation: f32) {
    *food_level = (*food_level + nutrition).clamp(0, MAX_FOOD_LEVEL);
    *saturation = (*saturation + baked_saturation).clamp(0.0, *food_level as f32);
}

/// Applies `FoodData.eat(FoodProperties)` for a `Consumable` if `item.has_food`.
#[inline]
pub fn eat(food_level: &mut i32, saturation: &mut f32, item: &Consumable) {
    if item.has_food {
        eat_food(food_level, saturation, item.nutrition, item.saturation);
    }
}

/// `FoodData.eat(int nutrition, float saturationModifier)`:
/// computes `saturation_by_modifier(nutrition, modifier)` then calls `eat_food`.
/// Not used by `FoodProperties.onConsume` (which passes the already-baked saturation).
#[inline]
pub fn eat_with_modifier(
    food_level: &mut i32,
    saturation: &mut f32,
    nutrition: i32,
    modifier: f32,
) {
    let sat = saturation_by_modifier(nutrition, modifier);
    eat_food(food_level, saturation, nutrition, sat);
}

// ============================================================================
// 2. Trident Riptide launch & Auto Spin Attack
// ============================================================================

/// `TridentItem.THROW_THRESHOLD_TIME` (`@29-37`).
pub const THROW_THRESHOLD_TICKS: i32 = 10;
/// `TridentItem.getUseDuration` (`@0-4`).
pub const USE_DURATION_TICKS: i32 = 72000;
/// `data/minecraft/enchantment/riptide.json`: `linear { base: 1.5, per_level_above_first: 0.75 }`.
pub const RIPTIDE_BASE: f32 = 1.5;
pub const RIPTIDE_PER_LEVEL_ABOVE_FIRST: f32 = 0.75;
/// `TridentItem.releaseUsing @351-358`: `startAutoSpinAttack(20, 8.0f, stack)`.
pub const AUTO_SPIN_ATTACK_TICKS: i32 = 20;
pub const AUTO_SPIN_ATTACK_DAMAGE: f32 = 8.0;
/// `TridentItem.releaseUsing @369-390`: `onGround()` hop vector (`1.1999999f` dead store ignored).
pub const GROUNDED_HOP: Vec3 = Vec3::new(0.0, 1.1999999284744263, 0.0);
/// `Avatar.POSES` (`Pose.SPIN_ATTACK`): `0.6 x 0.6`, eye height `0.4`.
pub const SPIN_ATTACK_POSE_WIDTH: f32 = 0.6;
pub const SPIN_ATTACK_POSE_HEIGHT: f32 = 0.6;
pub const SPIN_ATTACK_POSE_EYE_HEIGHT: f32 = 0.4;

/// `EnchantmentHelper.getTridentSpinAttackStrength`: `AddValue(Linear(1.5, 0.75))` over `0.0f`.
/// Levels 1..=3 give `1.5`, `2.25`, `3.0` (`0.0` when `level <= 0`).
#[inline]
pub fn trident_spin_attack_strength(level: i32) -> f32 {
    if level <= 0 {
        0.0
    } else {
        0.0f32 + (RIPTIDE_BASE + RIPTIDE_PER_LEVEL_ABOVE_FIRST * (level - 1) as f32)
    }
}

/// `TridentItem.use` (`@0-49`): fails if about to break or if riptide > 0 while dry.
#[inline]
pub fn can_start_trident_use(strength: f32, wet: bool, next_damage_will_break: bool) -> bool {
    !next_damage_will_break && !(strength > 0.0 && !wet)
}

/// `TridentItem.releaseUsing` (`@18-79`): full gate before either throw (`strength == 0`) or
/// riptide launch (`strength > 0`).
#[inline]
pub fn can_release_trident(
    held_ticks: i32,
    strength: f32,
    wet: bool,
    passenger: bool,
    next_damage_will_break: bool,
) -> bool {
    held_ticks >= THROW_THRESHOLD_TICKS
        && !(strength > 0.0 && (!wet || passenger))
        && !next_damage_will_break
}

/// True when `releaseUsing` executes a Riptide launch (`strength > 0` and all release gates pass).
#[inline]
pub fn can_riptide(
    held_ticks: i32,
    strength: f32,
    wet: bool,
    passenger: bool,
    next_damage_will_break: bool,
) -> bool {
    strength > 0.0 && can_release_trident(held_ticks, strength, wet, passenger, next_damage_will_break)
}

/// `TridentItem.releaseUsing` (`@216-347`): bit-exact Riptide impulse vector.
/// Angles in degrees are multiplied by `0.017453292f` in `f32`, widened to `f64` for `Mth.sin`/`cos`,
/// combined in `f32`, normalized by `Mth.sqrt` (`f32`), and scaled per-axis by `strength / f4`.
pub fn riptide_push(yaw: f32, pitch: f32, strength: f32) -> Vec3 {
    let yaw_rad = (yaw * 0.017453292f32) as f64;
    let pitch_rad = (pitch * 0.017453292f32) as f64;
    let mut f1 = -math::sin(yaw_rad) * math::cos(pitch_rad);
    let mut f2 = -math::sin(pitch_rad);
    let mut f3 = math::cos(yaw_rad) * math::cos(pitch_rad);
    let f4 = math::sqrt(f1 * f1 + f2 * f2 + f3 * f3);
    let q = strength / f4;
    f1 *= q;
    f2 *= q;
    f3 *= q;
    Vec3::new(f1 as f64, f2 as f64, f3 as f64)
}

/// `Entity.push(DDD)` (`@0-42`): adds `impulse` to `velocity` iff all three components are finite.
#[inline]
pub fn push(velocity: Vec3, impulse: Vec3) -> Vec3 {
    if impulse.x.is_finite() && impulse.y.is_finite() && impulse.z.is_finite() {
        Vec3::new(
            velocity.x + impulse.x,
            velocity.y + impulse.y,
            velocity.z + impulse.z,
        )
    } else {
        velocity
    }
}

/// `AABB.minmax(AABB)` (`LivingEntity.checkAutoSpinAttack @0-5`): sweep hull from pre-travel
/// `old_box` to post-travel `new_box`.
#[inline]
pub fn spin_attack_hull(old_box: Aabb, new_box: Aabb) -> Aabb {
    Aabb {
        min: Vec3::new(
            old_box.min.x.min(new_box.min.x),
            old_box.min.y.min(new_box.min.y),
            old_box.min.z.min(new_box.min.z),
        ),
        max: Vec3::new(
            old_box.max.x.max(new_box.max.x),
            old_box.max.y.max(new_box.max.y),
            old_box.max.z.max(new_box.max.z),
        ),
    }
}

/// Entity list outcome passed to `LivingEntity.checkAutoSpinAttack`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinTouch {
    /// `level.getEntities(this, hull)` was empty (`@24 -> @107` horizontal collision check).
    None,
    /// `getEntities` was non-empty, but contained no `LivingEntity` (skips horizontal collision check).
    NonLivingOnly,
    /// At least one `LivingEntity` was in `hull` (`doAutoAttackOnTouch` + bounce + stop).
    Living,
}

/// `LivingEntity.autoSpinAttackTicks`, `autoSpinAttackDmg`, and `LIVING_ENTITY_FLAG_AUTO_SPIN_ATTACK` (`4`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpinAttack {
    pub remaining_ticks: i32,
    pub damage: f32,
    /// Mirrors `DATA_LIVING_ENTITY_FLAGS & 4`, which `Player.getDesiredPose` reads.
    pub flag_active: bool,
}

impl SpinAttack {
    /// `Player.startAutoSpinAttack(20, 8.0f, stack)` (`@0-35`).
    pub fn start(&mut self) {
        self.remaining_ticks = AUTO_SPIN_ATTACK_TICKS;
        self.damage = AUTO_SPIN_ATTACK_DAMAGE;
        self.flag_active = true;
    }

    /// `LivingEntity.aiStep @789-813` + `checkAutoSpinAttack @0-152`:
    /// decrements positive tick count, checks `touch` / `horizontal_collision`, applies bounce on
    /// `SpinTouch::Living`, and clears `flag_active` / `damage` once `remaining_ticks <= 0`.
    /// Returns `true` iff a living entity was struck and velocity bounced.
    pub fn check_auto_spin_attack(
        &mut self,
        velocity: &mut Vec3,
        horizontal_collision: bool,
        touch: SpinTouch,
    ) -> bool {
        if self.remaining_ticks <= 0 {
            return false;
        }
        self.remaining_ticks -= 1;
        let mut bounced = false;
        match touch {
            SpinTouch::Living => {
                self.remaining_ticks = 0;
                *velocity = Self::bounce(*velocity);
                bounced = true;
            }
            SpinTouch::NonLivingOnly => {}
            SpinTouch::None => {
                if horizontal_collision {
                    self.remaining_ticks = 0;
                }
            }
        }
        if self.remaining_ticks <= 0 {
            self.flag_active = false;
            self.damage = 0.0;
        }
        bounced
    }

    /// Simplified tick step for callers tracking only `remaining_ticks`.
    #[inline]
    pub fn step(&mut self, horizontal_collision: bool, touched_living_entity: bool) {
        let mut dummy = Vec3::default();
        let touch = if touched_living_entity {
            SpinTouch::Living
        } else {
            SpinTouch::None
        };
        let _ = self.check_auto_spin_attack(&mut dummy, horizontal_collision, touch);
    }

    /// `LivingEntity.checkAutoSpinAttack @84-95`: `velocity.scale(-0.2d)`.
    #[inline]
    pub fn bounce(velocity: Vec3) -> Vec3 {
        velocity.scale(-0.2)
    }

    #[inline]
    pub const fn finished(&self) -> bool {
        self.remaining_ticks <= 0
    }
}
