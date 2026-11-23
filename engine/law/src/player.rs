//! Player-level tick laws (26.2): sprint trigger reset, lava inside-block collector & hurt,
//! game-mode ability switch, and flying horizontal speed.

use crate::{math, Vec3};

// ============================================================================
// Part 7: Sprint trigger reset by food and items
// ============================================================================

/// `ClientInput.hasForwardImpulse` (`@0-18`): strict `moveVector.y > 1.0E-5f`.
pub fn has_forward_impulse(move_vector_y: f32) -> bool {
    move_vector_y > 1.0e-5f32
}

/// `FoodData.hasEnoughFood` (`@0-16`): strict `(float)foodLevel > 6.0f`.
pub fn food_data_has_enough_food(food_level: i32) -> bool {
    (food_level as f32) > 6.0f32
}

/// `Player.hasEnoughFoodToDoExhaustiveManoeuvres` (`@0-25`).
pub fn has_enough_food_to_do_exhaustive_manoeuvres(food_level: i32, mayfly: bool) -> bool {
    food_data_has_enough_food(food_level) || mayfly
}

/// `LocalPlayer.isSlowDueToUsingItem` (`@0-34`).
pub fn is_slow_due_to_using_item(using_item: bool, use_effects_can_sprint: bool) -> bool {
    using_item && !use_effects_can_sprint
}

/// `LocalPlayer.isSprintingPossible(boolean flying)` (`@0-51`).
pub fn is_sprinting_possible(
    mobility_restricted_blindness: bool,
    is_passenger: bool,
    vehicle_can_sprint: bool,
    food_level: i32,
    mayfly: bool,
    flying: bool,
    in_water: bool,
    underwater: bool,
) -> bool {
    if mobility_restricted_blindness {
        return false;
    }
    let food_ok = if is_passenger {
        vehicle_can_sprint
    } else {
        has_enough_food_to_do_exhaustive_manoeuvres(food_level, mayfly)
    };
    let in_shallow_water = in_water && !underwater;
    food_ok && (flying || !in_shallow_water)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SprintMachineState {
    pub sprint_trigger_time: i32,
    pub is_sprinting: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SprintTickInput {
    pub pre_shift_key_down: bool,
    pub pre_move_vector_y: f32,
    pub post_move_vector_y: f32,
    pub post_backward: bool,
    pub post_sprint_key: bool,
    pub sprint_window: i32,
    pub using_item: bool,
    pub use_effects_can_sprint: bool,
    pub is_passenger: bool,
    pub vehicle_can_sprint: bool,
    pub food_level: i32,
    pub mayfly: bool,
    pub flying: bool,
    pub in_water: bool,
    pub underwater: bool,
    pub fall_flying: bool,
    pub moving_slowly: bool,
    pub swimming: bool,
    pub on_ground: bool,
    pub sneak_key: bool,
    pub horizontal_collision: bool,
    pub minor_horizontal_collision: bool,
    pub blindness: bool,
}

/// `LocalPlayer.aiStep` sprint machine (`@0-484`).
pub fn step_ai_sprint(state: &mut SprintMachineState, input: &SprintTickInput) {
    // @0-14 decrement
    if state.sprint_trigger_time > 0 {
        state.sprint_trigger_time -= 1;
    }
    // @67-85 pre-input.tick captures
    let was_shift = input.pre_shift_key_down;
    let was_forward = has_forward_impulse(input.pre_move_vector_y);

    let slow_using = is_slow_due_to_using_item(input.using_item, input.use_effects_can_sprint);
    // @343-376 reset BEFORE canStartSprinting
    if was_shift || (slow_using && !input.is_passenger) || input.post_backward {
        state.sprint_trigger_time = 0;
    }

    let post_forward = has_forward_impulse(input.post_move_vector_y);
    let possible = is_sprinting_possible(
        input.blindness,
        input.is_passenger,
        input.vehicle_can_sprint,
        input.food_level,
        input.mayfly,
        input.flying,
        input.in_water,
        input.underwater,
    );
    let can_start = !state.is_sprinting
        && post_forward
        && possible
        && !slow_using
        && (!input.fall_flying || input.underwater)
        && (!input.moving_slowly || input.underwater);

    if can_start {
        if !was_forward {
            if state.sprint_trigger_time > 0 {
                state.is_sprinting = true;
            } else {
                state.sprint_trigger_time = input.sprint_window;
            }
        }
        if !state.is_sprinting && input.post_sprint_key {
            state.is_sprinting = true;
        }
    }

    // @446-484 stop cascade
    if state.is_sprinting {
        let stop = if input.swimming {
            !food_data_has_enough_food(input.food_level)
                || !input.in_water
                || (!post_forward && !input.on_ground && !input.sneak_key)
        } else {
            !possible
                || !post_forward
                || (input.horizontal_collision && !input.minor_horizontal_collision)
        };
        if stop {
            state.is_sprinting = false;
        }
    }
}

// ============================================================================
// Part 8: Lava inside the body: hurt and the collector
// ============================================================================

/// `InsideBlockEffectType` declaration order = `APPLY_ORDER`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InsideBlockEffectType {
    Freeze = 0,
    ClearFreeze = 1,
    FireIgnite = 2,
    LavaIgnite = 3,
    Extinguish = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsideEffectOp {
    ApplyType(InsideBlockEffectType),
    LavaHurt,
    FireHurt { soul: bool },
    Custom(u32),
}

/// `InsideBlockEffectApplier$StepBasedCollector`.
#[derive(Clone, Debug, Default)]
pub struct StepBasedCollector {
    effects_in_step: u8,
    before_effects: [Vec<InsideEffectOp>; 5],
    after_effects: [Vec<InsideEffectOp>; 5],
    final_effects: Vec<InsideEffectOp>,
    last_step: i32,
}

impl StepBasedCollector {
    pub fn new() -> Self {
        Self {
            effects_in_step: 0,
            before_effects: Default::default(),
            after_effects: Default::default(),
            final_effects: Vec::new(),
            last_step: -1,
        }
    }

    /// `apply(type)` (`@0-11`): `EnumSet` add, deduped per step.
    pub fn apply(&mut self, kind: InsideBlockEffectType) {
        self.effects_in_step |= 1u8 << (kind as u8);
    }

    /// `runAfter(type, consumer)` (`@0-20`): `ArrayList` add, appends every call.
    pub fn run_after(&mut self, kind: InsideBlockEffectType, op: InsideEffectOp) {
        self.after_effects[kind as usize].push(op);
    }

    /// `advanceStep(step)` (`@0-17`).
    pub fn advance_step(&mut self, step: i32) {
        if self.last_step != step {
            self.last_step = step;
            self.flush_step();
        }
    }

    /// `flushStep()` (`@0-124`): iterates `APPLY_ORDER`.
    pub fn flush_step(&mut self) {
        let all = [
            InsideBlockEffectType::Freeze,
            InsideBlockEffectType::ClearFreeze,
            InsideBlockEffectType::FireIgnite,
            InsideBlockEffectType::LavaIgnite,
            InsideBlockEffectType::Extinguish,
        ];
        for kind in all {
            let idx = kind as usize;
            self.final_effects.extend(self.before_effects[idx].drain(..));
            let bit = 1u8 << (kind as u8);
            if self.effects_in_step & bit != 0 {
                self.effects_in_step &= !bit;
                self.final_effects.push(InsideEffectOp::ApplyType(kind));
            }
            self.final_effects.extend(self.after_effects[idx].drain(..));
        }
    }

    /// `LavaFluid.entityInside` (`@0-35`).
    pub fn record_lava_cell(&mut self) {
        self.apply(InsideBlockEffectType::ClearFreeze);
        self.apply(InsideBlockEffectType::LavaIgnite);
        self.run_after(InsideBlockEffectType::LavaIgnite, InsideEffectOp::LavaHurt);
    }

    /// `BaseFireBlock.entityInside` (`@396-409`): `ClearFreeze` + `FireIgnite` (Set dedup),
    /// and `FireHurt` after-consumer once per visited cell.
    pub fn record_fire_cell(&mut self, soul: bool) {
        self.apply(InsideBlockEffectType::ClearFreeze);
        self.apply(InsideBlockEffectType::FireIgnite);
        self.run_after(InsideBlockEffectType::FireIgnite, InsideEffectOp::FireHurt { soul });
    }

    /// Drains and returns the flushed ops in execution order; caller checks `is_alive()` before each.
    pub fn take_final_effects(&mut self) -> Vec<InsideEffectOp> {
        self.flush_step();
        self.last_step = -1;
        std::mem::take(&mut self.final_effects)
    }
}

pub const LAVA_IGNITE_SECONDS: f32 = 15.0;
pub const LAVA_DAMAGE: f32 = 4.0;
pub const LAVA_EXHAUSTION: f32 = 0.1;

/// `CombatRules.getDamageAfterAbsorb(LivingEntity, float, DamageSource, float, float)` (`@0-99`),
/// environmental branch (no weapon item). `armor` is `Mth.floor(generic.armor)`.
pub fn damage_after_armor_absorb(damage: f32, armor: f32, toughness: f32) -> f32 {
    let f = 2.0f32 + toughness / 4.0f32;
    let clamped = crate::math::clamp_f32(armor - damage / f, armor * 0.2f32, 20.0f32);
    damage * (1.0f32 - clamped / 25.0f32)
}

/// `LivingEntity.getDamageAfterMagicAbsorb` Resistance branch (`@42-73`):
/// `j = 25 - (amp + 1) * 5`, `Math.max(damage * (float)j / 25.0f, 0.0f)`.
pub fn damage_after_resistance(damage: f32, amplifier: u8) -> f32 {
    let j = 25 - (amplifier as i32 + 1) * 5;
    (damage * (j as f32) / 25.0f32).max(0.0f32)
}

/// `CombatRules.getDamageAfterMagicAbsorb(float, float)`: `damage * (1.0f - Mth.clamp(epf, 0.0f, 20.0f) / 25.0f)`.
pub fn damage_after_magic_absorb(damage: f32, epf: f32) -> f32 {
    damage * (1.0f32 - crate::math::clamp_f32(epf, 0.0f32, 20.0f32) / 25.0f32)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HurtEntityState {
    pub health: f32,
    pub invulnerable_time: i32,
    pub last_hurt: f32,
    pub hurt_duration: i32,
    pub hurt_time: i32,
    pub remaining_fire_ticks: i32,
    pub ticks_frozen: i32,
    pub exhaustion: f32,
    pub fire_immune: bool,
    pub abilities_invulnerable: bool,
    pub has_fire_resistance: bool,
    pub armor: f32,
    pub armor_toughness: f32,
    pub protection_epf: f32,
    pub feather_falling_epf: f32,
    pub resistance: Option<u8>,
    pub absorption: f32,
}

impl Default for HurtEntityState {
    fn default() -> Self {
        Self {
            health: 20.0,
            invulnerable_time: 0,
            last_hurt: 0.0,
            hurt_duration: 0,
            hurt_time: 0,
            remaining_fire_ticks: 0,
            ticks_frozen: 0,
            exhaustion: 0.0,
            fire_immune: false,
            abilities_invulnerable: false,
            has_fire_resistance: false,
            armor: 0.0,
            armor_toughness: 0.0,
            protection_epf: 0.0,
            feather_falling_epf: 0.0,
            resistance: None,
            absorption: 0.0,
        }
    }
}

/// `Entity.igniteForTicks` + `Entity.clearFreeze`.
pub fn ignite_for_ticks(entity: &mut HurtEntityState, ticks: i32) {
    if entity.remaining_fire_ticks < ticks {
        entity.remaining_fire_ticks = ticks;
    }
    entity.ticks_frozen = 0;
}

/// `Entity.clearFire()` (`Entity.code.txt:1271-1279`): `min(0, current)`.
pub fn clear_fire_ticks(current: i32) -> i32 {
    current.min(0)
}

/// `BaseFireBlock.fireIgnite` (`@1-69`): skips `fireImmune`, charges `< 0` by `+1`,
/// increments `ServerPlayer` `>= 0` deterministically by `+1` (`nextInt(1, 3)` min),
/// and ignites for `8.0f * 20 = 160` ticks when `next >= 0`.
pub fn base_fire_ignite_ticks(current: i32, fire_immune: bool, is_server_player: bool) -> i32 {
    if fire_immune {
        return current;
    }
    let next = if current < 0 {
        current + 1
    } else if is_server_player {
        current + 1
    } else {
        current
    };
    if next >= 0 { next.max(160) } else { next }
}

/// `Entity.applyEffectsFromBlocks` fire grace tail (`@129-157`): server-only reset to
/// `-Player.getFireImmuneTicks() (-20)` for players (or `0` for non-player entities) when
/// `!isOnFire && !grew`.
pub fn apply_effects_fire_tail(
    after_ticks: i32,
    before_ticks: i32,
    is_server_level: bool,
    fire_immune: bool,
    is_player: bool,
) -> i32 {
    let is_on_fire = !fire_immune && after_ticks > 0;
    let grew = after_ticks > before_ticks;
    if is_server_level && !is_on_fire && !grew {
        if is_player { -20 } else { 0 }
    } else {
        after_ticks
    }
}

/// `Entity.lavaIgnite()` (`@0-15`).
pub fn lava_ignite(entity: &mut HurtEntityState) {
    if entity.fire_immune {
        return;
    }
    let ticks = math::floor((LAVA_IGNITE_SECONDS * 20.0f32) as f64);
    ignite_for_ticks(entity, ticks);
}

/// `PointedDripstoneBlock.fallOn` (`@0-60`): returns `(distance, multiplier, is_stalagmite)`.
pub fn dripstone_fall_on_args(
    fall_distance: f64,
    tip_direction_up: bool,
    thickness_is_tip: bool,
) -> (f64, f32, bool) {
    if tip_direction_up && thickness_is_tip {
        (fall_distance + 2.5, 2.0, true)
    } else {
        (fall_distance, 1.0, false)
    }
}

/// `FarmlandBlock.fallOn` trample condition (`@0-99`, `LivingEntity.getBbWidth()*width*height > 0.512f`,
/// `random.nextFloat() < fall_distance - 0.5`).
pub fn farmland_trample_threshold(width: f32, height: f32, fall_distance: f64, random_float: f32) -> bool {
    let bb_volume = width * width * height;
    bb_volume > 0.512f32 && (random_float as f64) < (fall_distance - 0.5)
}

/// `WitherMobEffect.DAMAGE_INTERVAL` (`40` ticks at amplifier 0) and `applyEffectTick` damage (`1.0f`).
pub const WITHER_DAMAGE_BASE_INTERVAL: i32 = 40;

/// `WitherMobEffect.shouldApplyEffectTickThisTick` (`@0-43`):
/// `interval = 40 >> amplifier`; true when `interval <= 0 || duration % interval == 0`.
#[inline]
pub const fn wither_should_apply_effect_tick(duration: i32, amplifier: i32) -> bool {
    let interval = WITHER_DAMAGE_BASE_INTERVAL.wrapping_shr(amplifier as u32);
    interval <= 0 || duration % interval == 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageScaling {
    Never,
    WhenCausedByLivingNonPlayer,
    Always,
}

/// `Player.actuallyHurt` (`Player.javap:1679`) + `LivingEntity.getDamageAfterArmorAbsorb`
/// + `LivingEntity.getDamageAfterMagicAbsorb`.
#[allow(clippy::too_many_arguments)]
pub fn actually_hurt(
    health: &mut f32,
    absorption: &mut f32,
    exhaustion: &mut f32,
    abilities_invulnerable: bool,
    mut amount: f32,
    armor: f32,
    armor_toughness: f32,
    resistance: Option<u8>,
    protection_epf: f32,
    feather_falling_epf: f32,
    bypasses_armor: bool,
    bypasses_effects: bool,
    bypasses_resistance: bool,
    bypasses_enchantments: bool,
    is_fall: bool,
    source_exhaustion: f32,
) {
    if !bypasses_armor {
        let floored_armor = math::floor(armor as f64) as f32;
        amount = damage_after_armor_absorb(amount, floored_armor, armor_toughness);
    }
    if !bypasses_effects {
        if !bypasses_resistance && let Some(amp) = resistance {
            amount = damage_after_resistance(amount, amp);
        }
        if amount <= 0.0f32 {
            amount = 0.0f32;
        } else if !bypasses_enchantments {
            let epf = protection_epf + if is_fall { feather_falling_epf } else { 0.0f32 };
            if epf > 0.0f32 {
                amount = damage_after_magic_absorb(amount, epf);
            }
        }
    }
    let to_health = (amount - *absorption).max(0.0f32);
    *absorption -= amount - to_health;
    if to_health != 0.0f32 {
        if !abilities_invulnerable {
            *exhaustion = (*exhaustion + source_exhaustion).min(40.0f32);
        }
        *health = (*health - to_health).max(0.0f32);
    }
}

/// Full `Player.hurtServer` + `LivingEntity.hurtServer` + `Player.actuallyHurt` pipeline.
#[allow(clippy::too_many_arguments)]
pub fn hurt_server_mitigated(
    entity: &mut HurtEntityState,
    is_server_level: bool,
    mut amount: f32,
    is_fire: bool,
    bypasses_invulnerability: bool,
    bypasses_cooldown: bool,
    bypasses_armor: bool,
    bypasses_effects: bool,
    bypasses_resistance: bool,
    bypasses_enchantments: bool,
    is_fall: bool,
    scales: DamageScaling,
    caused_by_living_non_player: bool,
    difficulty: u8,
    source_exhaustion: f32,
) -> bool {
    if !is_server_level {
        return false;
    }
    if entity.abilities_invulnerable && !bypasses_invulnerability {
        return false;
    }
    if entity.health <= 0.0f32 {
        return false;
    }
    let should_scale = match scales {
        DamageScaling::Never => false,
        DamageScaling::WhenCausedByLivingNonPlayer => caused_by_living_non_player,
        DamageScaling::Always => true,
    };
    if should_scale {
        amount = match difficulty {
            0 => 0.0f32, // PEACEFUL
            1 => (amount / 2.0f32 + 1.0f32).min(amount), // EASY
            3 => amount * 3.0f32 / 2.0f32, // HARD
            _ => amount, // NORMAL
        };
        if amount == 0.0f32 {
            return false;
        }
    }
    if is_fire && entity.has_fire_resistance {
        return false;
    }
    if amount < 0.0f32 {
        amount = 0.0f32;
    } else if !amount.is_finite() {
        amount = f32::MAX;
    }

    // LivingEntity.hurtServer @184-269: strict `(float)invulnerableTime > 10.0f`.
    let damage_to_apply = if (entity.invulnerable_time as f32) > 10.0f32 && !bypasses_cooldown {
        if amount <= entity.last_hurt {
            return false;
        }
        let partial = amount - entity.last_hurt;
        entity.last_hurt = amount;
        partial
    } else {
        entity.last_hurt = amount;
        entity.invulnerable_time = 20;
        entity.hurt_duration = 10;
        entity.hurt_time = 10;
        amount
    };

    actually_hurt(
        &mut entity.health,
        &mut entity.absorption,
        &mut entity.exhaustion,
        entity.abilities_invulnerable,
        damage_to_apply,
        entity.armor,
        entity.armor_toughness,
        entity.resistance,
        entity.protection_epf,
        entity.feather_falling_epf,
        bypasses_armor,
        bypasses_effects,
        bypasses_resistance,
        bypasses_enchantments,
        is_fall,
        source_exhaustion,
    );
    true
}

/// `Player.hurtServer` + `LivingEntity.hurtServer` + `Player.actuallyHurt` (default non-bypassing flags).
#[allow(clippy::too_many_arguments)]
pub fn hurt_server(
    entity: &mut HurtEntityState,
    is_server_level: bool,
    amount: f32,
    is_fire: bool,
    bypasses_invulnerability: bool,
    bypasses_cooldown: bool,
    scales: DamageScaling,
    caused_by_living_non_player: bool,
    difficulty: u8,
    source_exhaustion: f32,
) -> bool {
    hurt_server_mitigated(
        entity,
        is_server_level,
        amount,
        is_fire,
        bypasses_invulnerability,
        bypasses_cooldown,
        false,
        false,
        false,
        false,
        false,
        scales,
        caused_by_living_non_player,
        difficulty,
        source_exhaustion,
    )
}

/// `Entity.lavaHurt()` (`@0-99`): `4.0f` unscaled environmental lava damage (`0.1f` exhaustion).
pub fn lava_hurt(entity: &mut HurtEntityState, is_server_level: bool) -> bool {
    if entity.fire_immune || !is_server_level {
        return false;
    }
    hurt_server(
        entity,
        is_server_level,
        LAVA_DAMAGE,
        true,
        false,
        false,
        DamageScaling::WhenCausedByLivingNonPlayer,
        false,
        2,
        LAVA_EXHAUSTION,
    )
}

/// `StepBasedCollector.applyAndClear` (`@0-67`) for the standard inside-effect ops.
pub fn apply_collector_effects(
    collector: &mut StepBasedCollector,
    entity: &mut HurtEntityState,
    is_server_level: bool,
) -> (u32, u32) {
    let ops = collector.take_final_effects();
    let mut hurt_calls = 0u32;
    let mut ignite_calls = 0u32;
    for op in ops {
        if entity.health <= 0.0f32 {
            break;
        }
        match op {
            InsideEffectOp::ApplyType(InsideBlockEffectType::Freeze) => {
                entity.ticks_frozen = (entity.ticks_frozen + 1).min(140);
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::ClearFreeze) => {
                entity.ticks_frozen = 0;
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::FireIgnite) => {
                let next = base_fire_ignite_ticks(entity.remaining_fire_ticks, entity.fire_immune, is_server_level);
                entity.remaining_fire_ticks = next;
                if next >= 0 {
                    entity.ticks_frozen = 0;
                }
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::LavaIgnite) => {
                ignite_calls += 1;
                lava_ignite(entity);
            }
            InsideEffectOp::ApplyType(InsideBlockEffectType::Extinguish) => {
                entity.remaining_fire_ticks = clear_fire_ticks(entity.remaining_fire_ticks);
            }
            InsideEffectOp::LavaHurt => {
                hurt_calls += 1;
                lava_hurt(entity, is_server_level);
            }
            InsideEffectOp::FireHurt { soul } => {
                hurt_calls += 1;
                let amount = if soul { 2.0 } else { 1.0 };
                hurt_server(
                    entity,
                    is_server_level,
                    amount,
                    true,
                    false,
                    false,
                    DamageScaling::WhenCausedByLivingNonPlayer,
                    false,
                    2,
                    0.1,
                );
            }
            InsideEffectOp::Custom(_) => {}
        }
    }
    (hurt_calls, ignite_calls)
}

// ============================================================================
// Part 9: The game-mode switch's abilities
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GameType {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl GameType {
    pub const ALL: [Self; 4] = [Self::Survival, Self::Creative, Self::Adventure, Self::Spectator];

    pub const fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }

    pub const fn is_block_placing_restricted(self) -> bool {
        matches!(self, Self::Adventure | Self::Spectator)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Abilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub mayfly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            mayfly: false,
            instabuild: false,
            may_build: true,
            flying_speed: 0.05f32,
            walking_speed: 0.1f32,
        }
    }
}

/// `GameType.updatePlayerAbilities(Abilities)` (`GameType.javap:102-153`).
/// Never writes `flying_speed` or `walking_speed`.
pub fn update_player_abilities(mode: GameType, abilities: &mut Abilities) {
    match mode {
        GameType::Creative => {
            abilities.mayfly = true;
            abilities.instabuild = true;
            abilities.invulnerable = true;
        }
        GameType::Spectator => {
            abilities.mayfly = true;
            abilities.instabuild = false;
            abilities.invulnerable = true;
            abilities.flying = true;
        }
        GameType::Survival | GameType::Adventure => {
            abilities.mayfly = false;
            abilities.instabuild = false;
            abilities.invulnerable = false;
            abilities.flying = false;
        }
    }
    abilities.may_build = !mode.is_block_placing_restricted();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameModeSwitchPacket {
    PlayerAbilities,
    PlayerInfoUpdateGameMode(GameType),
    GameEventChangeGameMode(GameType),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerGameModeSwitchResult {
    pub changed: bool,
    pub previous_mode: GameType,
    pub current_mode: GameType,
    pub packets: Vec<GameModeSwitchPacket>,
    pub cleared_shoulder_entities: bool,
    pub stopped_riding: bool,
    pub stopped_using_item: bool,
    pub reset_impulse_context: bool,
}

/// `ServerPlayer.setGameMode(GameType)` + `ServerPlayerGameMode.changeGameModeForPlayer` (`@0-109`).
/// `is_in_range_of_ground` = `colliders.isEmpty() && available_space_below(1.0) < 1.0`.
pub fn server_set_game_mode(
    current_mode: &mut GameType,
    previous_mode: &mut GameType,
    abilities: &mut Abilities,
    new_mode: GameType,
    is_in_range_of_ground: bool,
) -> ServerGameModeSwitchResult {
    if *current_mode == new_mode {
        return ServerGameModeSwitchResult {
            changed: false,
            previous_mode: *previous_mode,
            current_mode: *current_mode,
            packets: Vec::new(),
            cleared_shoulder_entities: false,
            stopped_riding: false,
            stopped_using_item: false,
            reset_impulse_context: false,
        };
    }
    let was_spectator = *current_mode == GameType::Spectator;
    let old = *current_mode;
    *previous_mode = old;
    *current_mode = new_mode;
    update_player_abilities(new_mode, abilities);

    // ServerPlayerGameMode.changeGameModeForPlayer @27-50
    if abilities.flying && new_mode != GameType::Spectator && is_in_range_of_ground {
        abilities.flying = false;
    }

    let mut packets = Vec::with_capacity(4);
    packets.push(GameModeSwitchPacket::PlayerAbilities); // @53-57
    packets.push(GameModeSwitchPacket::PlayerInfoUpdateGameMode(new_mode)); // @60-84
    packets.push(GameModeSwitchPacket::GameEventChangeGameMode(new_mode)); // setGameMode @18-37
    let entering_spectator = new_mode == GameType::Spectator;
    let _exiting_spectator = !entering_spectator && was_spectator;
    packets.push(GameModeSwitchPacket::PlayerAbilities); // setGameMode @83-84

    ServerGameModeSwitchResult {
        changed: true,
        previous_mode: *previous_mode,
        current_mode: *current_mode,
        packets,
        cleared_shoulder_entities: entering_spectator,
        stopped_riding: entering_spectator,
        stopped_using_item: entering_spectator,
        reset_impulse_context: new_mode == GameType::Creative,
    }
}

// ============================================================================
// Part 10: Horizontal speed while flying
// ============================================================================

/// `Blocks.BLUE_ICE.getFriction()` in Mojang 26.2 (`0x3f7d2f1b` = `0.989f32`, not 1.21's `0.98`).
pub const BLUE_ICE_FRICTION: f32 = 0.989f32;

/// `Player.getFlyingSpeed()` (`client-Player-all.txt:4807-4836`).
pub fn player_get_flying_speed(
    abilities_flying: bool,
    is_passenger: bool,
    is_sprinting: bool,
    abilities_flying_speed: f32,
) -> f32 {
    if abilities_flying && !is_passenger {
        if is_sprinting {
            abilities_flying_speed * 2.0f32
        } else {
            abilities_flying_speed
        }
    } else if is_sprinting {
        0.025999999f32
    } else {
        0.02f32
    }
}

/// `LivingEntity.getFrictionInfluencedSpeed(float f)` (`@7338`).
pub fn get_friction_influenced_speed(
    on_ground: bool,
    friction: f32,
    movement_speed: f32,
    flying_speed: f32,
) -> f32 {
    if on_ground {
        if (friction as f64) > 0.6 {
            movement_speed * (0.21600002f32 / (friction * friction * friction))
        } else {
            movement_speed
        }
    } else {
        flying_speed
    }
}

/// One tick of `Player.travel` + `LivingEntity.travelInAir` horizontal velocity update
/// for synthetic forward input `(0, 0, 1)` at `yaw = 0` (exact acceleration + friction drag).
pub fn travel_in_air_forward_tick(
    vel: Vec3,
    on_ground: bool,
    ground_friction: f32,
    friction_modifier: f32,
    air_drag_modifier: f32,
    movement_speed: f32,
    abilities_flying: bool,
    abilities_flying_speed: f32,
    is_passenger: bool,
    is_sprinting: bool,
) -> Vec3 {
    let f3 = if on_ground {
        math::modified_friction(ground_friction, friction_modifier)
    } else {
        1.0f32
    };
    let fly_spd = player_get_flying_speed(
        abilities_flying,
        is_passenger,
        is_sprinting,
        abilities_flying_speed,
    );
    let accel = get_friction_influenced_speed(on_ground, f3, movement_speed, fly_spd);
    let vz_after_accel = vel.z + accel as f64;
    let f9 = math::modified_friction(0.91f32, air_drag_modifier);
    let f10 = f3 * f9;
    let vy = if abilities_flying {
        vel.y * 0.6
    } else {
        (vel.y - 0.08) * (math::modified_friction(0.98f32, air_drag_modifier) as f64)
    };
    Vec3::new(vel.x * f10 as f64, vy, vz_after_accel * f10 as f64)
}

#[cfg(test)]
mod part7_to_part10_tests {
    use super::*;

    #[test]
    fn part7_sprint_food_and_item_reset_witnesses() {
        // check-reproduce-out.txt:225-299 & check/checkwitness-out.txt:1-66
        assert!(!food_data_has_enough_food(0));
        assert!(!food_data_has_enough_food(6));
        assert!(food_data_has_enough_food(7));
        assert!(food_data_has_enough_food(20));
        assert!(!has_enough_food_to_do_exhaustive_manoeuvres(6, false));
        assert!(has_enough_food_to_do_exhaustive_manoeuvres(6, true));
        assert!(has_enough_food_to_do_exhaustive_manoeuvres(0, true));

        // ClientInput.hasForwardImpulse strict 1.0E-5f boundary (checkwitness-out.txt:2-6)
        assert!(!has_forward_impulse(1.0e-5f32));
        assert!(has_forward_impulse(f32::from_bits(1.0e-5f32.to_bits() + 1)));
        assert!(!has_forward_impulse(-1.0f32));

        let base = SprintTickInput {
            pre_shift_key_down: false,
            pre_move_vector_y: 0.0,
            post_move_vector_y: 1.0,
            post_backward: false,
            post_sprint_key: false,
            sprint_window: 7,
            using_item: false,
            use_effects_can_sprint: false,
            is_passenger: false,
            vehicle_can_sprint: true,
            food_level: 20,
            mayfly: false,
            flying: false,
            in_water: false,
            underwater: false,
            fall_flying: false,
            moving_slowly: false,
            swimming: false,
            on_ground: true,
            sneak_key: false,
            horizontal_collision: false,
            minor_horizontal_collision: false,
            blindness: false,
        };

        // S2 double-tap killed by bow/food use (check-reproduce-out.txt:242-248)
        let mut s2 = SprintMachineState::default();
        step_ai_sprint(&mut s2, &base);
        assert_eq!(s2, SprintMachineState { sprint_trigger_time: 7, is_sprinting: false });
        step_ai_sprint(&mut s2, &SprintTickInput { using_item: true, ..base });
        assert_eq!(s2, SprintMachineState { sprint_trigger_time: 0, is_sprinting: false });

        // S3 double-tap survives spear use (canSprint=true) (check-reproduce-out.txt:257-265)
        let mut s3 = SprintMachineState::default();
        step_ai_sprint(&mut s3, &base);
        step_ai_sprint(&mut s3, &SprintTickInput {
            pre_move_vector_y: 1.0,
            post_move_vector_y: 0.0,
            using_item: true,
            use_effects_can_sprint: true,
            ..base
        });
        assert_eq!(s3, SprintMachineState { sprint_trigger_time: 6, is_sprinting: false });
        step_ai_sprint(&mut s3, &SprintTickInput {
            pre_move_vector_y: 0.0,
            post_move_vector_y: 1.0,
            using_item: true,
            use_effects_can_sprint: true,
            ..base
        });
        assert_eq!(s3, SprintMachineState { sprint_trigger_time: 5, is_sprinting: true });

        // S5 vs S6 food=6 Survival vs Creative mayfly (check-reproduce-out.txt:283-299)
        let mut s5 = SprintMachineState::default();
        step_ai_sprint(&mut s5, &SprintTickInput { food_level: 6, post_sprint_key: true, ..base });
        assert_eq!(s5, SprintMachineState { sprint_trigger_time: 0, is_sprinting: false });
        let mut s6 = SprintMachineState::default();
        step_ai_sprint(&mut s6, &SprintTickInput { food_level: 6, mayfly: true, ..base });
        assert_eq!(s6.sprint_trigger_time, 7);
        step_ai_sprint(&mut s6, &SprintTickInput { food_level: 6, mayfly: true, pre_move_vector_y: 1.0, post_move_vector_y: 0.0, ..base });
        step_ai_sprint(&mut s6, &SprintTickInput { food_level: 6, mayfly: true, pre_move_vector_y: 0.0, post_move_vector_y: 1.0, ..base });
        assert!(s6.is_sprinting);

        // A1 mid-sprint bow start keeps sprinting while zeroing window (checkwitness-out.txt:8-18)
        let mut a1 = SprintMachineState { sprint_trigger_time: 5, is_sprinting: true };
        step_ai_sprint(&mut a1, &SprintTickInput { pre_move_vector_y: 1.0, using_item: true, ..base });
        assert_eq!(a1, SprintMachineState { sprint_trigger_time: 0, is_sprinting: true });

        // A2 passenger + bow exempts reset (checkwitness-out.txt:20-26)
        let mut a2 = SprintMachineState { sprint_trigger_time: 7, is_sprinting: false };
        step_ai_sprint(&mut a2, &SprintTickInput { is_passenger: true, using_item: true, ..base });
        assert_eq!(a2, SprintMachineState { sprint_trigger_time: 6, is_sprinting: false });

        // A3 backward tap resets window (checkwitness-out.txt:28-35)
        let mut a3 = SprintMachineState { sprint_trigger_time: 6, is_sprinting: false };
        step_ai_sprint(&mut a3, &SprintTickInput { post_move_vector_y: -1.0, post_backward: true, ..base });
        assert_eq!(a3.sprint_trigger_time, 0);
    }

    #[test]
    fn part8_lava_hurt_and_collector_witnesses() {
        // probe-output.txt:3-18 & mycheck-output.txt:1-7
        // S1 client vs server level
        let mut c = StepBasedCollector::new();
        c.advance_step(0);
        c.record_lava_cell();
        let mut e_client = HurtEntityState::default();
        let (hc, ic) = apply_collector_effects(&mut c, &mut e_client, false);
        assert_eq!((hc, ic), (1, 1));
        assert_eq!(e_client.health, 20.0);
        assert_eq!(e_client.remaining_fire_ticks, 300);

        let mut e_srv = HurtEntityState::default();
        c.advance_step(0);
        c.record_lava_cell();
        apply_collector_effects(&mut c, &mut e_srv, true);
        assert_eq!(e_srv.health.to_bits(), 0x4180_0000); // 16.0
        assert_eq!(e_srv.remaining_fire_ticks, 300);

        // S2a strict invulnerableTime > 10.0f cadence & partial hit (probe-output.txt:5-8, mycheck-output.txt:1-2)
        let mut e10 = HurtEntityState { invulnerable_time: 10, last_hurt: 100.0, ..Default::default() };
        assert!(lava_hurt(&mut e10, true));
        assert_eq!((e10.health, e10.invulnerable_time, e10.last_hurt, e10.exhaustion), (16.0, 20, 4.0, 0.1));

        let mut e11_blk = HurtEntityState { invulnerable_time: 11, last_hurt: 4.0, ..Default::default() };
        assert!(!lava_hurt(&mut e11_blk, true));
        assert_eq!((e11_blk.health, e11_blk.invulnerable_time, e11_blk.exhaustion), (20.0, 11, 0.0));

        let mut e11_part = HurtEntityState { invulnerable_time: 11, last_hurt: 2.0, ..Default::default() };
        assert!(lava_hurt(&mut e11_part, true));
        assert_eq!((e11_part.health, e11_part.invulnerable_time, e11_part.last_hurt, e11_part.exhaustion), (18.0, 11, 4.0, 0.1));

        // S2b two cells same step vs diff steps (probe-output.txt:9-10)
        c.advance_step(0);
        c.record_lava_cell();
        c.record_lava_cell();
        let mut e_same = HurtEntityState::default();
        assert_eq!(apply_collector_effects(&mut c, &mut e_same, true), (2, 1));
        assert_eq!(e_same.health, 16.0);

        c.advance_step(0);
        c.record_lava_cell();
        c.advance_step(1);
        c.record_lava_cell();
        let mut e_diff = HurtEntityState::default();
        assert_eq!(apply_collector_effects(&mut c, &mut e_diff, true), (2, 2));
        assert_eq!(e_diff.health, 16.0);

        // S3 & A5 order: FREEZE runs before CLEAR_FREEZE in one flush (probe-output.txt:11, mycheck-output.txt:5)
        c.advance_step(0);
        c.apply(InsideBlockEffectType::ClearFreeze);
        c.apply(InsideBlockEffectType::Freeze);
        let mut e_frz = HurtEntityState { ticks_frozen: 10, ..Default::default() };
        apply_collector_effects(&mut c, &mut e_frz, true);
        assert_eq!(e_frz.ticks_frozen, 0);

        // S4 difficulty scaling: lava unscaled across all 4 difficulties; ALWAYS control scales (probe-output.txt:12-15)
        for (diff, expected_ctrl) in [(0u8, 20.0f32), (1, 17.0), (2, 16.0), (3, 14.0)] {
            let mut el = HurtEntityState::default();
            assert!(hurt_server(&mut el, true, 4.0, true, false, false, DamageScaling::WhenCausedByLivingNonPlayer, false, diff, 0.1));
            assert_eq!(el.health, 16.0);
            let mut ec = HurtEntityState::default();
            let hit = hurt_server(&mut ec, true, 4.0, false, false, false, DamageScaling::Always, false, diff, 0.1);
            assert_eq!(hit, diff != 0);
            assert_eq!(ec.health, expected_ctrl);
        }

        // S5 fire resistance blocks damage/exhaustion while ignite & clearFreeze run (probe-output.txt:16)
        let mut e_res = HurtEntityState { has_fire_resistance: true, ticks_frozen: 10, ..Default::default() };
        c.advance_step(0);
        c.record_lava_cell();
        apply_collector_effects(&mut c, &mut e_res, true);
        assert_eq!((e_res.health, e_res.remaining_fire_ticks, e_res.ticks_frozen, e_res.invulnerable_time, e_res.exhaustion), (20.0, 300, 0, 0, 0.0));

        // S6 & A3 ignite max-only (probe-output.txt:17, mycheck-output.txt:3)
        let mut e_400 = HurtEntityState { remaining_fire_ticks: 400, ..Default::default() };
        lava_ignite(&mut e_400);
        assert_eq!(e_400.remaining_fire_ticks, 400);
        let mut e_299 = HurtEntityState { remaining_fire_ticks: 299, ..Default::default() };
        lava_ignite(&mut e_299);
        assert_eq!(e_299.remaining_fire_ticks, 300);
    }

    #[test]
    fn part9_gamemode_ability_switch_witnesses() {
        // out/driver-transitions.txt:4-35 (all 16 ordered transitions)
        let expected_post = |mode: GameType, pre_flying: bool| -> (bool, bool, bool, bool, bool) {
            match mode {
                GameType::Survival => (false, false, false, false, true),
                GameType::Creative => (true, pre_flying, true, true, true),
                GameType::Adventure => (false, false, false, false, false),
                GameType::Spectator => (true, true, true, false, false),
            }
        };
        for from in GameType::ALL {
            for to in GameType::ALL {
                let mut ab = Abilities {
                    flying_speed: 0.42,
                    walking_speed: 0.24,
                    ..Abilities::default()
                };
                update_player_abilities(from, &mut ab);
                if matches!(from, GameType::Creative | GameType::Spectator) {
                    ab.flying = true;
                }
                let pre_fly = ab.flying;
                update_player_abilities(to, &mut ab);
                let (inv, fly, may, insta, build) = expected_post(to, pre_fly);
                assert_eq!((ab.invulnerable, ab.flying, ab.mayfly, ab.instabuild, ab.may_build), (inv, fly, may, insta, build), "{from:?} -> {to:?}");
                assert_eq!((ab.flying_speed, ab.walking_speed), (0.42, 0.24));
            }
        }

        // ServerPlayer.setGameMode SPECTATOR->CREATIVE landing clear + double Abilities packet
        let mut cur = GameType::Spectator;
        let mut prev = GameType::Survival;
        let mut ab = Abilities::default();
        update_player_abilities(cur, &mut ab);
        let res = server_set_game_mode(&mut cur, &mut prev, &mut ab, GameType::Creative, true);
        assert!(res.changed);
        assert!(!ab.flying);
        assert_eq!(
            res.packets,
            vec![
                GameModeSwitchPacket::PlayerAbilities,
                GameModeSwitchPacket::PlayerInfoUpdateGameMode(GameType::Creative),
                GameModeSwitchPacket::GameEventChangeGameMode(GameType::Creative),
                GameModeSwitchPacket::PlayerAbilities,
            ]
        );
        assert!(!res.stopped_using_item && res.reset_impulse_context);
        // Same-mode refusal
        assert!(!server_set_game_mode(&mut cur, &mut prev, &mut ab, GameType::Creative, false).changed);
    }

    #[test]
    fn part10_flying_horizontal_speed_witnesses() {
        // evidence/harness-output.txt:6-29 & check-evidence/advfly-output.txt:2-16
        assert_eq!(BLUE_ICE_FRICTION.to_bits(), 0x3f7d_2f1b);
        assert_eq!(player_get_flying_speed(true, false, false, 0.05).to_bits(), 0x3d4c_cccd);
        assert_eq!(player_get_flying_speed(true, false, true, 0.05).to_bits(), 0x3dcc_cccd);
        assert_eq!(player_get_flying_speed(false, false, true, 0.05).to_bits(), 0x3cd4_fdf3);
        assert_eq!(player_get_flying_speed(false, false, false, 0.05).to_bits(), 0x3ca3_d70a);

        let attr_01 = f32::from_bits(0x3dcc_cccd); // 0.10000000149011612 (0.1f32)
        // S1a..S1d (air)
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), false, 0.6, 1.0, 1.0, attr_01, true, 0.05, false, false).z.to_bits(), 0x3fa7_4bc6_b906_24e0);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), false, 0.6, 1.0, 1.0, attr_01, true, 0.05, false, true).z.to_bits(), 0x3fb7_4bc6_b906_24e0);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), false, 0.6, 1.0, 1.0, attr_01, false, 0.05, false, false).z.to_bits(), 0x3f92_a305_552b_d3c0);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), false, 0.6, 1.0, 1.0, attr_01, false, 0.05, false, true).z.to_bits(), 0x3f98_3a53_afdf_e320);

        // S2a..S2d (ground: flying invisible)
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), true, 0.6, 1.0, 1.0, attr_01, true, 0.05, false, false).z.to_bits(), 0x3fab_f488_3a30_5540);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), true, 0.6, 1.0, 1.0, attr_01, false, 0.05, false, false).z.to_bits(), 0x3fab_f488_3a30_5540);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), true, BLUE_ICE_FRICTION, 1.0, 1.0, attr_01, true, 0.05, false, false).z.to_bits(), 0x3f94_93f5_ec74_7540);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), true, BLUE_ICE_FRICTION, 1.0, 1.0, attr_01, false, 0.05, false, false).z.to_bits(), 0x3f94_93f5_ec74_7540);

        // S3b descend land vy (harness-output.txt:23-24)
        let t1 = travel_in_air_forward_tick(Vec3::new(0.0, -0.5, 0.0), false, BLUE_ICE_FRICTION, 1.0, 1.0, attr_01, true, 0.05, false, false);
        assert_eq!((t1.z.to_bits(), t1.y.to_bits()), (0x3fa7_4bc6_b906_24e0, 0xbfd3_3333_3333_3333));
        let t2 = travel_in_air_forward_tick(Vec3::new(0.0, -0.3, t1.z), true, BLUE_ICE_FRICTION, 1.0, 1.0, attr_01, true, 0.05, false, false);
        assert_eq!((t2.z.to_bits(), t2.y.to_bits()), (0x3faf_4151_c766_0339, 0xbfc7_0a3d_70a3_d70a));

        // AdvFly A1..B6 (check-evidence/advfly-output.txt:2-12)
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), false, 0.6, 1.0, 1.0, 0.2, true, 0.07, false, false).z.to_bits(), 0x3fb0_4ea4_b1ce_0760);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), false, 0.6, 1.0, 1.0, 0.2, true, 0.07, false, true).z.to_bits(), 0x3fc0_4ea4_b1ce_0760);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), true, 0.6, 1.0, 1.0, 0.2, true, 0.07, false, false).z.to_bits(), 0x3fbb_f488_3a30_5540);
        assert_eq!(travel_in_air_forward_tick(Vec3::default(), true, BLUE_ICE_FRICTION, 1.0, 1.0, 0.2, true, 0.07, false, false).z.to_bits(), 0x3fa4_93f5_ec74_7540);
    }
}

/// Void damage and respawn reset (26.2 Entity.checkBelowWorld, ServerPlayer.restoreFrom).
pub mod death {
    //! The death side of the player law: the void's threshold and toll, and the reset
    //! a respawn performs. Anchors are the 26.2 server jar: `Entity.checkBelowWorld`,
    //! `LivingEntity.onBelowWorld`, `Player.hurtServer`, `PlayerList.respawn`,
    //! `ServerPlayer.restoreFrom`; the offsets sit in the run dossier.

    use crate::{PlayerState, Vec3, World};

    /// LivingEntity.onBelowWorld: hurt(damageSources().fellOutOfWorld(), 4.0f).
    /// out_of_world bypasses armor, resistance, invulnerability and knockback, and
    /// carries 0.0 exhaustion, so the raw hit is the debit for any player without
    /// protective enchantments (data/minecraft/damage_type/out_of_world.json and the
    /// tags bypasses_armor, bypasses_invulnerability, bypasses_resistance, no_knockback).
    pub const FELL_OUT_OF_WORLD: f32 = 4.0;

    /// Entity.checkBelowWorld: `getY() < (double)(level.getMinY() - 64)`, asked every
    /// Entity.baseTick. Strict: the overworld (minY -64) reads -128.0 and a player at
    /// exactly -128.0 is still in the world.
    pub fn below_world(y: f64, min_y: i32) -> bool {
        y < (min_y - 64) as f64
    }

    /// PlayerList.respawn into a fresh ServerPlayer with restoreFrom(keepEverything=false):
    /// health to the maximum (20.0 for the stock player), FoodData to its constructor
    /// (foodLevel 20, saturationLevel 5.0f, exhaustion 0.0f), and a new entity's zero for
    /// the hurt cooldown, fire, freezing, fall distance, velocity and effects. The
    /// position is the respawn teleport the caller performs: death is a teleport with a
    /// toll (S189), and this reset is the toll's ledger, not the teleport.
    pub fn respawn(state: &mut PlayerState, world: &impl World) {
        use crate::state_reads::{StateField as F, StateReads};
        world.state_footprint(StateReads(0), StateReads::of(&[F::health, F::food_level, F::saturation, F::exhaustion,
            F::food_tick_timer, F::velocity, F::fall_distance, F::remaining_fire_ticks, F::ticks_frozen, F::effects, F::effect_book, F::max_health, F::hurt_cooldown]));
        state.health = 20.0;
        state.food_level = 20;
        state.saturation = 5.0;
        state.exhaustion = 0.0;
        state.food_tick_timer = 0;
        state.velocity = Vec3::default();
        state.fall_distance = 0.0;
        state.remaining_fire_ticks = 0;
        state.ticks_frozen = 0;
        state.effects = false;
        state.effect_book.clear();
        state.max_health = 20.0;
        state.hurt_cooldown.remaining_ticks = 0;
        state.hurt_cooldown.last_raw_hit = 0.0;
    }

    /// `Blocks.lambda$static$39` (26.2): bed `bounceRestitution = 0.75f`.
    pub const BED_BOUNCE_RESTITUTION: f32 = 0.75;
    /// `BedBlock.fallOn`: `fallDistance * 0.5d`.
    pub const BED_FALL_DISTANCE_MULTIPLIER: f64 = 0.5;
    /// `BedBlock.useWithoutItem` / `RespawnAnchorBlock.explode`: explosion power `5.0f` with fire.
    pub const BAD_RESPAWN_EXPLOSION_RADIUS: f32 = 5.0;
    pub const RESPAWN_ANCHOR_MIN_CHARGES: u8 = 0;
    pub const RESPAWN_ANCHOR_MAX_CHARGES: u8 = 4;

    /// `BedRule.Rule` (`net.minecraft.world.attribute.BedRule`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BedRuleGate {
        Always,
        WhenDark,
        Never,
    }

    impl BedRuleGate {
        pub const fn test(self, is_dark_outside: bool) -> bool {
            match self {
                Self::Always => true,
                Self::WhenDark => is_dark_outside,
                Self::Never => false,
            }
        }
    }

    /// `BedRule` environment attribute (`CAN_SLEEP_WHEN_DARK` vs `EXPLODES`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BedRule {
        pub can_sleep: BedRuleGate,
        pub can_set_spawn: BedRuleGate,
        pub explodes: bool,
    }

    impl BedRule {
        pub const CAN_SLEEP_WHEN_DARK: Self = Self {
            can_sleep: BedRuleGate::WhenDark,
            can_set_spawn: BedRuleGate::Always,
            explodes: false,
        };
        pub const EXPLODES: Self = Self {
            can_sleep: BedRuleGate::Never,
            can_set_spawn: BedRuleGate::Never,
            explodes: true,
        };
    }

    /// `RespawnAnchorBlock.useWithoutItem` outcome.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum RespawnAnchorUse {
        Pass,
        SetSpawn,
        Explode { radius: f32, fire: bool },
    }

    /// `RespawnAnchorBlock.useItemOn` + `charge`: glowstone charges up to `MAX_CHARGES = 4`.
    pub const fn respawn_anchor_charge(charges: u8) -> Option<u8> {
        if charges < RESPAWN_ANCHOR_MAX_CHARGES {
            Some(charges + 1)
        } else {
            None
        }
    }

    /// `RespawnAnchorBlock.useWithoutItem`: `charge == 0` passes; else `works` sets spawn or explodes (`5.0f`, fire).
    pub const fn respawn_anchor_use(charges: u8, works: bool) -> RespawnAnchorUse {
        if charges == RESPAWN_ANCHOR_MIN_CHARGES {
            RespawnAnchorUse::Pass
        } else if works {
            RespawnAnchorUse::SetSpawn
        } else {
            RespawnAnchorUse::Explode { radius: BAD_RESPAWN_EXPLOSION_RADIUS, fire: true }
        }
    }

    /// `ServerPlayer.findRespawnAndUseSpawnBlock` for a respawn anchor:
    /// accepted when `(forced || charges > 0) && works`; consumes 1 charge when `!keep_everything && charges > 0`.
    pub const fn respawn_anchor_find_and_use(
        charges: u8,
        forced: bool,
        works: bool,
        keep_everything: bool,
    ) -> Option<u8> {
        if !(forced || charges > 0) || !works {
            return None;
        }
        if !keep_everything && charges > 0 {
            Some(charges - 1)
        } else {
            Some(charges)
        }
    }
}
