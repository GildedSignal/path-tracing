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

// ============================================================================
// Item entities and inventory (folded from GLM dossier
// local-20261122-145224-30192-1-0-port-item-entity-and-inventory)
// ============================================================================
// Item entity law 26.2: tick motion, merge, pickup delay, inventory fill, drop spawn, Unbreaking.
//
// Sources: `ItemEntity`, `Inventory`, `LivingEntity.createItemStackToDrop`,
// `ItemStack.hurtAndBreak/processDurabilityChange`, `EnchantmentHelper.processDurabilityChange`,
// `Enchantment.modifyItemFilteredCount`, `effects.RemoveBinomial`, `data/minecraft/enchantment/unbreaking.json`,
// verified against the staged 26.2 jar (see lab run local-20261122-145224-30192-1-0-port-item-entity-and-inventory).

pub const LIFETIME: i32 = 6000; // tick 544-551: despawn when age >= 6000 (server side)
pub const INFINITE_LIFETIME_AGE: i32 = -32768; // setUnlimitedLifetime 0-7
pub const INFINITE_PICKUP_DELAY: i32 = 32767; // setNeverPickUp 0-7
pub const DEFAULT_HEALTH: i32 = 5; // ItemEntity.DEFAULT_HEALTH; hurtServer 59-67: health -= (int) damage
pub const MERGE_INTERVAL_MOVING: i32 = 2; // tick 423-433: crossed a block boundary this tick
pub const MERGE_INTERVAL_RESTING: i32 = 40; // same block: merge attempt every 40 ticks
pub const REST_HORIZ_SPEED_SQR: f64 = 9.999999747378752E-6; // tick 235-238 (1.0E-5f as f64)
pub const PICKUP_MIN_FLUID_HEIGHT: f64 = 0.10000000149011612; // tick 89/117 (0.1f as f64)
pub const FLUID_RISE_VELOCITY: f64 = 0.05999999865889549; // setFluidMovement 20-23 (0.06f as f64)
pub const FLUID_RISE_ACCEL: f64 = 5.0E-4f32 as f64; // setFluidMovement 27-30
pub const LAVA_DRAG: f64 = 0.95f32 as f64; // setUnderLavaMovement 1-4
pub const REST_MOVE_MODULUS: i32 = 4; // tick 251: (tickCount + id) % 4

/// Fluid the body is submerged in with height > `PICKUP_MIN_FLUID_HEIGHT` (caller resolves
/// `isInWater`/`getFluidHeight`, ItemEntity.tick 75-135; water wins over lava).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Fluid { #[default] None, Water, Lava }

#[derive(Clone, Copy, Debug, Default)]
pub struct ItemBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: i32,
    pub pickup_delay: i32,
    pub id: i32,
    pub tick_count: i32,
    pub on_ground: bool,
}

/// Events the tick produced (ItemEntity.tick 0-558).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TickEvents {
    pub crossed_block_boundary: bool,
    pub try_merge: bool,
    pub despawn: bool,
}

/// One ItemEntity.tick on open ground; `ground` is the y of the support plane
/// (`None` = free fall), `friction` the support block's `Block.getFriction()`.
///
/// Order (tick offsets): pickup delay 19-43, fluid float or gravity 75-135, move + collide 221-264
/// (rest skip 221-253: on ground, horizontal speed^2 <= 1e-5 and (tickCount+id)%4 != 0 skips BOTH
/// move and the drag multiply), drag multiply 271-319 (x,z by 0.98*friction on ground else 0.98,
/// y always by 0.98), bounce 322-356 (y *= -0.5 when y < 0 survived the move), merge cadence 366-461,
/// age increment 464-481, despawn 534-555.
///
/// Collision response follows `Entity.move` -> `restituteMovementAfterCollisions`: for a non-living
/// body on a block with `getBounceRestitution() == 0` (stone, ice, dirt) the collided axis is zeroed,
/// so the bounce branch is dead on plain ground; bouncy blocks multiply the axis by
/// `max(0, restitution * 0.8)` instead of zeroing it.
pub fn tick(body: &mut ItemBody, fluid: Fluid, ground: Option<f64>, friction: f32, bounce_restitution: f32) -> TickEvents {
    body.tick_count += 1; // Entity.baseTick (super.tick) increments tickCount before the rest
    if body.pickup_delay > 0 && body.pickup_delay != INFINITE_PICKUP_DELAY {
        body.pickup_delay -= 1;
    }
    let old = body.position;
    match fluid {
        Fluid::None => crate::basis::body::ITEM.gravity(&mut body.velocity, false), // applyGravity (Entity 3832-3850)
        Fluid::Water | Fluid::Lava => {
            // setFluidMovement: x,z *= drag; y += 5e-4f while y < 0.06f (gravity is NOT applied)
            let drag = if fluid == Fluid::Water { crate::basis::body::ITEM.water_drag as f64 } else { LAVA_DRAG };
            body.velocity.x *= drag;
            body.velocity.z *= drag;
            if body.velocity.y < FLUID_RISE_VELOCITY {
                body.velocity.y += FLUID_RISE_ACCEL;
            }
        }
    }
    let h2 = body.velocity.horizontal_sqr();
    let move_runs = !body.on_ground
        || h2 > REST_HORIZ_SPEED_SQR
        || (body.tick_count + body.id) % REST_MOVE_MODULUS == 0;
    if move_runs {
        if let Some(g) = ground {
            body.position = body.position.plus(body.velocity);
            if body.position.y <= g && body.velocity.y <= 0.0 {
                body.position.y = g;
                let bounciness = if bounce_restitution <= 0.0 { 0.0 } else { (bounce_restitution * 0.8) as f64 };
                body.velocity.y = -body.velocity.y * bounciness; // restituteMovementAfterCollisions
                body.on_ground = true;
            } else if body.position.y > g {
                body.on_ground = false;
            }
        } else {
            body.position = body.position.plus(body.velocity);
            body.on_ground = false;
        }
        let f = crate::basis::body::ITEM.air_drag;
        let f1 = if body.on_ground { f * friction } else { f };
        body.velocity.x *= f1 as f64;
        body.velocity.y *= f as f64;
        body.velocity.z *= f1 as f64;
        if body.on_ground && body.velocity.y < 0.0 {
            body.velocity.y *= -0.5;
        }
    }
    let crossed = old.x.floor() != body.position.x.floor()
        || old.y.floor() != body.position.y.floor()
        || old.z.floor() != body.position.z.floor();
    let interval = if crossed { MERGE_INTERVAL_MOVING } else { MERGE_INTERVAL_RESTING };
    let try_merge = body.tick_count % interval == 0; // server-side and isMergable() checked by caller
    if body.age != INFINITE_LIFETIME_AGE {
        body.age += 1;
    }
    TickEvents { crossed_block_boundary: crossed, try_merge, despawn: body.age >= LIFETIME }
}

/// `ItemEntity.isMergable` 0-58.
pub fn is_mergable(alive: bool, pickup_delay: i32, age: i32, count: i32, max_stack_size: i32) -> bool {
    alive
        && pickup_delay != INFINITE_PICKUP_DELAY
        && age != INFINITE_LIFETIME_AGE
        && age < LIFETIME
        && count < max_stack_size
}

/// `ItemEntity.areMergable` 0-23: `a + b <= b.maxStackSize && isSameItemSameComponents(a, b)`.
pub fn are_mergable(count_a: i32, count_b: i32, max_stack_size: i32, same_item_and_components: bool) -> bool {
    count_a + count_b <= max_stack_size && same_item_and_components
}

/// `ItemEntity.merge(ItemStack, ItemStack, int)` 0-43: grow `a`, shrink `b`,
/// cap `min(a.maxStackSize, cap)`; returns `(grown count, donor remaining)`.
pub fn stack_merge(count_a: i32, count_b: i32, max_stack_size: i32, cap: i32) -> (i32, i32) {
    let moved = (max_stack_size.min(cap) - count_a).min(count_b).max(0);
    (count_a + moved, count_b - moved)
}

/// `ItemEntity.merge(ItemEntity, ItemStack, ItemEntity, ItemStack)` 0-47:
/// survivor keeps the merged stack, `age = min(a, b)`, `pickupDelay = max(a, b)`.
pub fn merge_survivor_state(age_a: i32, delay_a: i32, age_b: i32, delay_b: i32) -> (i32, i32) {
    (age_a.min(age_b), delay_a.max(delay_b))
}

/// Spawner-constructor random motion (`ItemEntity.<init>(Level,x,y,z,ItemStack)` 23-61):
/// `(r*0.2 - 0.1, 0.2, r*0.2 - 0.1)`.
pub fn spawn_velocity(rng: &mut math::JavaRandom) -> Vec3 {
    Vec3::new(rng.next_double() * 0.2 - 0.1, 0.2, rng.next_double() * 0.2 - 0.1)
}

/// `LivingEntity.createItemStackToDrop` 9-33: spawn origin, always `pickupDelay = 40`.
pub fn drop_origin(eye_y: f64) -> (f64, f64) {
    (0.3f32 as f64, eye_y - 0.3f32 as f64) // x offset constant, y = eyeY - 0.3f
}

/// Aimed-throw velocity (the only branch `Player.drop(stack, boolean)` reaches:
/// `drop(stack, false=false-around, includeThrowerName)`), 125-296.
pub fn drop_velocity_aimed(pitch: f32, yaw: f32, rng: &mut math::JavaRandom) -> Vec3 {
    let f8 = math::sin((pitch * 0.017453292f32) as f64);
    let f9 = math::cos((pitch * 0.017453292f32) as f64);
    let f10 = math::sin((yaw * 0.017453292f32) as f64);
    let f11 = math::cos((yaw * 0.017453292f32) as f64);
    let f12 = rng.next_float() * 6.2831855f32;
    let f13 = 0.02f32 * rng.next_float();
    Vec3::new(
        (-f10 * f9 * 0.3f32) as f64 + (f12 as f64).cos() * f13 as f64,
        (-f8 * 0.3f32 + 0.1f32 + (rng.next_float() - rng.next_float()) * 0.1f32) as f64,
        (f11 * f9 * 0.3f32) as f64 + (f12 as f64).sin() * f13 as f64,
    )
}

/// Random-toss branch (`dropAround = true`; not reachable from `Player.drop`), 64-122.
pub fn drop_velocity_toss(rng: &mut math::JavaRandom) -> Vec3 {
    let f = rng.next_float() * 0.5f32;
    let t = rng.next_float() * 6.2831855f32;
    Vec3::new((-(math::sin(t as f64) * f)) as f64, 0.2, (math::cos(t as f64) * f) as f64)
}

// ---- Inventory.fill (Inventory.getSlotWithRemainingSpace / getFreeSlot / add / addResource) ----

pub const SLOT_OFFHAND: i32 = 40; // Inventory 9

/// The state one stack slot needs for the fill rules. `item` equality must stand for
/// `isSameItemSameComponents`; `max_stack_size` is `ItemStack.getMaxStackSize()`
/// (26.2: data-driven component, absent component means 1 — not 64).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SlotStack {
    pub item: u32,
    pub count: i32,
    pub max_stack_size: i32,
    pub damaged: bool,
}

/// `Inventory.getSlotWithRemainingSpace` 0-79: selected hotbar slot first, then offhand 40,
/// then 0..36 in order. `hasRemainingSpaceForItem`: non-empty, same item+components,
/// stackable and `count < maxStackSize`.
pub fn slot_with_remaining_space(items: &[SlotStack; 36], offhand: SlotStack, selected: i32, incoming: &SlotStack) -> i32 {
    if accepts(items[selected as usize], incoming) {
        return selected;
    }
    if accepts(offhand, incoming) {
        return SLOT_OFFHAND;
    }
    for (i, s) in items.iter().enumerate() {
        if accepts(*s, incoming) {
            return i as i32;
        }
    }
    -1
}

fn accepts(slot: SlotStack, incoming: &SlotStack) -> bool {
    // hasRemainingSpaceForItem 0-39: slot non-empty, isSameItemSameComponents(slot, incoming),
    // slot.isStackable() (maxStackSize > 1), slot.count < min(99, slot.maxStackSize)
    slot.count > 0
        && slot.item == incoming.item
        && slot.max_stack_size > 1
        && slot.count < slot.max_stack_size
}

/// `Inventory.getFreeSlot` 0-39: first empty slot in 0..36 (hotbar first).
pub fn free_slot(items: &[SlotStack; 36]) -> i32 {
    items.iter().position(|s| s.count == 0).map(|i| i as i32).unwrap_or(-1)
}

/// `Inventory.addResource(int, ItemStack)` 0-96: fill `slot` up to
/// `min(containerMax, stackMax)`, container max for Inventory is the Container default 99.
pub fn add_resource(slot: &mut SlotStack, incoming: &mut SlotStack) -> bool {
    if slot.count == 0 {
        *slot = *incoming;
        slot.count = 0;
    }
    let cap = slot.max_stack_size.min(99);
    let moved = incoming.count.min(cap - slot.count);
    if moved <= 0 {
        return false;
    }
    incoming.count -= moved;
    slot.count += moved;
    true
}

/// `Inventory.add(int, ItemStack)` 0-164 for `slot == -1` (the `playerTouch` path).
/// Damaged stacks never stack: they go straight to a free slot (102-145).
/// Returns `(added, fill order)` where fill order lists the slots touched, in order.
pub fn add(items: &mut [SlotStack; 36], offhand: &mut SlotStack, selected: i32, incoming: &mut SlotStack) -> (bool, Vec<i32>) {
    let mut order = Vec::new();
    if incoming.count == 0 {
        return (false, order);
    }
    if incoming.damaged {
        let slot = free_slot(items);
        if slot >= 0 {
            items[slot as usize] = *incoming;
            incoming.count = 0;
            order.push(slot);
            return (true, order);
        }
        return (false, order); // no free slot: player.hasInfiniteMaterials path not modeled
    }
    let initial = incoming.count;
    loop {
        let before = incoming.count;
        let target = { let t = slot_with_remaining_space(items, *offhand, selected, incoming); if t == -1 { free_slot(items) } else { t } };
        if target == -1 {
            break;
        }
        if target == SLOT_OFFHAND {
            add_resource(offhand, incoming);
            order.push(SLOT_OFFHAND);
        } else {
            add_resource(&mut items[target as usize], incoming);
            order.push(target);
        }
        // Inventory.add 48-60: retry while the count still decreases
        if incoming.count == 0 || incoming.count >= before {
            break;
        }
    }
    (incoming.count < initial, order)
}

// ---- Unbreaking (data/minecraft/enchantment/unbreaking.json + RemoveBinomial.process) ----

/// `LevelBasedValue$Linear.calculate`: `base + perLevelAboveFirst * (level - 1)`, all f32.
pub fn linear(base: f32, per_level_above_first: f32, level: i32) -> f32 {
    base + per_level_above_first * (level - 1) as f32
}

/// Unbreaking `remove_binomial` chance (`LevelBasedValue$Fraction.calculate`):
/// armor `(2+2(l-1))/(10+5(l-1))`, else `(1+(l-1))/(2+(l-1))`.
/// Measured against the jar: take-damage probability is `1 - chance`
/// (armor takes 0.8/0.7333/0.7 at levels 1/2/3, non-armor takes 1/(l+1) = 0.5/0.333/0.25).
pub fn unbreaking_remove_chance(is_armor: bool, level: i32) -> f32 {
    let (n, d) = if is_armor {
        (linear(2.0, 2.0, level), linear(10.0, 5.0, level))
    } else {
        (linear(1.0, 1.0, level), linear(2.0, 1.0, level))
    };
    if d == 0.0 {
        0.0
    } else {
        n / d
    }
}

/// `RemoveBinomial.process(int, RandomSource, float)` 16-138: each damage point is removed with
/// probability `chance`; for `damage > 128` with both binomial tails `>= 20` a Gaussian
/// approximation (`floor(n*p) + nextGaussian() * sqrt(n*p*(1-p))`, clamped) is used instead.
/// Returns the damage that survives (is dealt).
pub fn remove_binomial(chance: f32, damage: i32, rng: &mut math::JavaRandom) -> i32 {
    let n = damage as f32;
    if n > 128.0 && (n * chance) >= 20.0 && (n * (1.0 - chance)) >= 20.0 {
        let mean = (n * chance) as f64; // Math.floor of the product
        let sd = ((n * chance * (1.0 - chance)) as f64).sqrt();
        let drawn = (mean + rng.next_gaussian() * sd).round() as i64;
        damage - drawn.clamp(0, damage as i64) as i32
    } else {
        let mut removed = 0i32;
        let mut i = 0f32;
        while i < n {
            if rng.next_float() < chance {
                removed += 1;
            }
            i += 1.0;
        }
        damage - removed
    }
}


/// `ItemStack.processDurabilityChange` 0-35 + `hurtAndBreak` 0-28: 0 for undamageable or
/// infinite-materials players; `amount <= 0` or unenchanted passes through unchanged; else
/// Unbreaking runs. The returned damage is added to `damage_value` in `applyDamage`
/// (item breaks when the new value reaches `maxDamage`).
pub fn durability_change(is_damageable: bool, amount: i32, unbreaking_level: i32, is_armor: bool, rng: &mut math::JavaRandom) -> i32 {
    if !is_damageable || amount <= 0 || unbreaking_level <= 0 {
        return amount;
    }
    let chance = unbreaking_remove_chance(is_armor, unbreaking_level);
    remove_binomial(chance, amount, rng)
}

#[cfg(test)]
mod item_entity_tests {
    use super::*;

    fn bits_f(x: f32) -> u32 { x.to_bits() }
    fn bits_d(x: f64) -> u64 { x.to_bits() }

    // Harness run local-20261122-145224-30192-1-0: harness-output.txt.
    #[test]
    fn constants_match_jar() {
        assert_eq!(bits_d(crate::basis::body::ITEM.gravity), 0x3FA47AE147AE147B);
        assert_eq!(bits_f(crate::basis::body::ITEM.air_drag), 0x3F7AE148);
        assert_eq!(bits_f(0.6f32), 0x3F19999A); // stone friction
        assert_eq!(bits_f(0.98f32 * 0.6f32), 0x3F16872C); // ground f1 = 0.588
        assert_eq!(bits_f(0.98f32 * 0.98f32), 0x3F75DCC7); // ice f1 = 0.9604
    }

    #[test]
    fn stone_motion_replays_jar() {
        let mut rng = math::JavaRandom::new(0x262);
        let v0 = spawn_velocity(&mut rng);
        assert_eq!(bits_d(v0.x), 0x3FA217997C8B0898);
        assert_eq!(bits_d(v0.y), 0x3FC999999999999A);
        assert_eq!(bits_d(v0.z), 0x3FB8CDA8C0E44448);
        let mut body = ItemBody { position: Vec3::new(0.0, 70.0, 0.0), velocity: v0, id: 0, ..Default::default() };
        // ticks 1..6 of the harness trace (harness collided every tick; the airborne phase is
        // order-identical to the port until the rest-skip can fire)
        let expect = [
            (0x3FA1BAF7B829679E, 0x3FC41205C28F5C29, 0x3FB84EAA8787E98F),
            (0x3FA160303A26F760, 0x3FBD4D8356FB8F5C, 0x3FB7D236825BCEA8),
            (0x3FA10739863ABA2E, 0x3FB2AE78BFE65F7B, 0x3FB7583FB0558F2B),
            (0x3FA0B00A50AC812F, 0x3FA08B9F46A3D539, 0x3FB6E0B952FF729B),
            (0x3FA05A997D5C4476, 0xBF7ED8E6CA8BE097, 0x3FB66B96ED2387C7),
            (0x3FA006DE1ECE7435, 0xBFA7D96498C40D11, 0x3FB5F8CC417D9185),
        ];
        for (t, e) in expect.iter().enumerate() {
            tick(&mut body, Fluid::None, Some(64.0), 0.6, 0.0);
            assert_eq!((bits_d(body.velocity.x), bits_d(body.velocity.y), bits_d(body.velocity.z)), *e, "tick {}", t + 1);
        }
        // run to t24: the item has landed and rests at y = 64 exactly (restitution clamps to ground)
        for _ in 7..=24 {
            tick(&mut body, Fluid::None, Some(64.0), 0.6, 0.0);
        }
        assert_eq!(bits_d(body.position.y), 0x4050000000000000);
    }

    #[test]
    fn water_float_replays_jar() {
        // fluid.WATER.t1/t2 from v = (0.05, -0.12, -0.05): setFluidMovement, then the
        // post-move drag multiply (both executed by the jar-backed harness)
        let mut b = ItemBody { velocity: Vec3::new(0.05, -0.12, -0.05), ..Default::default() };
        for (ex, ey, ez) in [
            (0x3FA8D64D8B2E9CCDu64, 0xBFBDFAEBCDB95AAF, 0xBFA8D64D8B2E9CCD),
            (0x3FA818D35E513B87, 0xBFBD414F689B74C6, 0xBFA818D35E513B87),
        ] {
            tick(&mut b, Fluid::Water, None, 0.6, 0.0);
            assert_eq!((bits_d(b.velocity.x), bits_d(b.velocity.y), bits_d(b.velocity.z)), (ex, ey, ez));
        }
    }

    #[test]
    fn merge_rules_match_jar() {
        assert!(are_mergable(30, 10, 64, true));
        assert!(!are_mergable(60, 30, 64, true)); // 90 > 64
        assert!(!are_mergable(10, 10, 16, true)); // pearls 20 > 16
        assert!(!are_mergable(30, 10, 64, false));
        assert_eq!(stack_merge(30, 10, 64, 64), (40, 0));
        assert_eq!(stack_merge(30, 40, 64, 64), (64, 6));
        assert_eq!(merge_survivor_state(1234, 40, 77, 0), (77, 40)); // age min, delay max
        assert!(is_mergable(true, 0, 100, 30, 64));
        assert!(!is_mergable(true, INFINITE_PICKUP_DELAY, 100, 30, 64));
        assert!(!is_mergable(true, 0, 6000, 30, 64));
    }

    #[test]
    fn inventory_fill_matches_jar() {
        let d = |count: i32| SlotStack { item: 1, count, max_stack_size: 64, damaged: false };
        // case1: empty inventory, add 40 -> slot 0
        let mut items = [SlotStack::default(); 36];
        let mut offhand = SlotStack::default();
        let mut add_stack = d(40);
        let (ok, order) = add(&mut items, &mut offhand, 0, &mut add_stack);
        assert_eq!((ok, order.clone(), add_stack.count, items[0].count), (true, vec![0], 0, 40));
        // case2: selected=5 holds 10, add 60 -> slot5=64 then slot0=6
        let mut items = [SlotStack::default(); 36];
        items[5] = d(10);
        let mut add_stack = d(60);
        let (ok, order) = add(&mut items, &mut offhand, 5, &mut add_stack);
        assert!(ok && order == vec![5, 0] && items[5].count == 64 && items[0].count == 6 && add_stack.count == 0);
        // case3: offhand holds 32, add 40 -> offhand=64 then slot0=8 (harness: freeSlot=1)
        let mut items = [SlotStack::default(); 36];
        let mut offhand = d(32);
        let mut add_stack = d(40);
        let (ok, order) = add(&mut items, &mut offhand, 0, &mut add_stack);
        assert!(ok && order == vec![SLOT_OFFHAND, 0] && offhand.count == 64 && items[0].count == 8, "{:?}", order);
        // case5: pearls (max 16), add 20 -> slot0=16, slot1=4
        let mut items = [SlotStack::default(); 36];
        let mut add_stack = SlotStack { item: 2, count: 20, max_stack_size: 16, damaged: false };
        let (ok, order) = add(&mut items, &mut offhand, 0, &mut add_stack);
        assert!(ok && order == vec![0, 1] && items[0].count == 16 && items[1].count == 4);
        // case4: damaged sword never stacks -> free slot 9 with slots 0..9 full except the twin
        let mut items = [SlotStack::default(); 36];
        items[1] = SlotStack { item: 3, count: 1, max_stack_size: 1, damaged: true };
        for s in items.iter_mut().take(9) {
            if s.count == 0 { *s = SlotStack { item: 9, count: 64, max_stack_size: 64, damaged: false }; }
        }
        let mut sword = SlotStack { item: 3, count: 1, max_stack_size: 1, damaged: true };
        let (ok, order) = add(&mut items, &mut SlotStack::default(), 0, &mut sword);
        assert!(ok && order == vec![9] && items[1].count == 1 && items[9].item == 3);
    }

    #[test]
    fn unbreaking_matches_jar() {
        // chance values from the jar Fraction(Linear, Linear)
        assert_eq!(bits_f(unbreaking_remove_chance(false, 1)), 0x3F000000); // 0.5
        assert_eq!(bits_f(unbreaking_remove_chance(false, 2)), 0x3F2AAAAB); // 0.6666667
        assert_eq!(bits_f(unbreaking_remove_chance(false, 3)), 0x3F400000); // 0.75
        assert_eq!(bits_f(unbreaking_remove_chance(true, 1)), 0x3E4CCCCD); // 0.2
        assert_eq!(bits_f(unbreaking_remove_chance(true, 2)), 0x3E888889); // 0.26666668
        assert_eq!(bits_f(unbreaking_remove_chance(true, 3)), 0x3E99999A); // 0.3
        // loop-branch bits replay (harness seeds 990001 + lvl + 1000*kind)
        for (lvl, expect) in [
            (1, [0x00000000u32, 0x00000000, 0x3F800000, 0x3F800000]),
            (2, [0x00000000, 0x00000000, 0x3F800000, 0x00000000]),
            (3, [0x00000000, 0x3F800000, 0x00000000, 0x00000000]),
        ] {
            let mut rng = math::JavaRandom::new(990001 + lvl as i64);
            for e in expect {
                assert_eq!(bits_f(remove_binomial(unbreaking_remove_chance(false, lvl), 1, &mut rng) as f32), e, "l{}", lvl);
            }
        }
        // take probabilities (harness: 0.49991/0.33379/0.24882 generic, 0.79883/0.73183/0.70079 armor)
        for (lvl, generic, armor) in [(1i32, 0.49991, 0.79883), (2, 0.33379, 0.73183), (3, 0.24882, 0.70079)] {
            for (is_armor, want) in [(false, generic), (true, armor)] {
                let mut rng = math::JavaRandom::new(770001 + lvl as i64);
                let mut taken = 0i64;
                for _ in 0..100_000 {
                    taken += durability_change(true, 1, lvl, is_armor, &mut rng) as i64;
                }
                let p = taken as f64 / 100_000.0;
                assert!((p - want).abs() < 0.01, "lvl {} armor {} p {}", lvl, is_armor, p);
            }
        }
        // gaussian branch: damage 200, generic l3 -> mean ~50
        let mut rng = math::JavaRandom::new(550001);
        let mut sum = 0.0;
        for _ in 0..2000 {
            sum += durability_change(true, 200, 3, false, &mut rng) as f64;
        }
        assert!((sum / 2000.0 - 50.0).abs() < 2.0, "mean {}", sum / 2000.0);
    }

    #[test]
    fn drop_velocity_replays_jar() {
        let mut rng = math::JavaRandom::new(0xD10);
        let v = drop_velocity_aimed(30.0, 45.0, &mut rng);
        // y is pure table-trig float math: bit-exact. x/z mix in java.lang.Math.cos/sin on the
        // jitter angle, which may differ from Rust's libm by <= 1 ulp.
        assert_eq!(bits_d(v.y), 0xBFA3494CE0000000);
        for (got, want) in [(v.x, 0xBFC745C882FC427Du64), (v.z, 0x3FC72AF7E2D97CE7)] {
            assert!((bits_d(got) as i64).wrapping_sub(want as i64).abs() <= 2, "got {:x}", bits_d(got));
        }
        let t = drop_velocity_toss(&mut rng);
        assert_eq!((bits_d(t.x), bits_d(t.y), bits_d(t.z)), (0x3FC7B09D60000000, 0x3FC999999999999A, 0x3FC87396E0000000));
        assert_eq!(bits_d(drop_origin(1.62).1 - 1.62), 0xBFD3333340000000); // -0.3f as f64
    }
}
