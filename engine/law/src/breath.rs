// Mojang 26.2 air supply, ported from the jar: LivingEntity.baseTick offsets 206..419,
// LivingEntity.decreaseAirSupply / increaseAirSupply / shouldTakeDrowningDamage,
// MobEffectUtil.hasWaterBreathing / shouldEffectsRefillAirsupply, Entity.getMaxAirSupply.
// Executed against the 26.2 server jar in
// lab/results/local-20261122-145218-28368-1-0-port-air-drowning-and-breath.

/// Entity.getMaxAirSupply — sipush 300; Entity's constructor defines air to this.
pub const MAX_AIR_SUPPLY: i32 = 300;
/// LivingEntity.baseTick offset 327 — hurtServer(damageSources().drown(), 2.0F).
pub const DROWN_DAMAGE: f32 = 2.0;
/// LivingEntity.increaseAirSupply — Math.min(air + 4, getMaxAirSupply()).
pub const REFILL_PER_TICK: i32 = 4;
/// LivingEntity.shouldTakeDrowningDamage — getAirSupply() <= -20.
pub const DROWN_THRESHOLD: i32 = -20;

/// The three effects the breath gates read (MobEffectUtil).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BreathEffects {
    /// MobEffects.WATER_BREATHING
    pub water_breathing: bool,
    /// MobEffects.CONDUIT_POWER
    pub conduit_power: bool,
    /// MobEffects.BREATH_OF_THE_NAUTILUS (26.2)
    pub breath_of_the_nautilus: bool,
}

/// MobEffectUtil.hasWaterBreathing: any of water breathing, conduit power,
/// breath of the nautilus. All three stop air loss.
pub fn has_water_breathing(effects: &BreathEffects) -> bool {
    effects.water_breathing || effects.conduit_power || effects.breath_of_the_nautilus
}

/// MobEffectUtil.shouldEffectsRefillAirsupply: false only under a pure Breath of the
/// Nautilus — nautilus present without water breathing and without conduit power.
pub fn effects_refill_air_supply(effects: &BreathEffects) -> bool {
    !(effects.breath_of_the_nautilus && !effects.water_breathing && !effects.conduit_power)
}

/// LivingEntity.decreaseAirSupply. `oxygen_bonus` is the OXYGEN_BONUS attribute value;
/// Respiration level L contributes L.0 (enchantment/respiration.json: linear, base 1.0,
/// per_level_above_first 1.0, add_value, head slot). The draw happens only when the
/// bonus is positive, exactly as the bytecode orders it.
pub fn decrease_air_supply(air: i32, oxygen_bonus: f64, random: &mut crate::math::JavaRandom) -> i32 {
    if oxygen_bonus > 0.0 && random.next_double() >= 1.0 / (oxygen_bonus + 1.0) {
        air
    } else {
        air - 1
    }
}

/// LivingEntity.increaseAirSupply: Math.min(air + 4, maxAir). Works from negative air.
pub fn increase_air_supply(air: i32) -> i32 {
    (air + REFILL_PER_TICK).min(MAX_AIR_SUPPLY)
}

/// One drown hit: the jar resets air to 0, broadcasts entity event 67, then applies
/// hurtServer(drown, 2.0F) — baseTick offsets 306..331.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrownHit {
    pub damage: f32,
}

/// One server tick of the breath section of LivingEntity.baseTick, after Entity.baseTick
/// (fire, fluid trackers) and before aiStep/travel — breath judges the pre-move position.
///
/// `eye_in_water`: Entity.isEyeInFluid(FluidTags.WATER) — the fluid trackers hold it when
/// the eye point sits inside the fluid column at its own block X/Z, waterlogged blocks
/// included (EntityFluidInteraction.update, blockY <= eyeY <= blockY + fluidHeight).
/// `eye_in_bubble_column`: the block at the eye position is a bubble column; its fluid
/// still counts as water, but baseTick skips the loss branch for it (offset 241).
/// `can_breathe_underwater`: EntityTypeTags.CAN_BREATHE_UNDER_WATER; false for players.
/// `player_invulnerable`: Player.getAbilities().invulnerable (creative); it also leaves
/// the refill gate open, so creative air regenerates underwater.
pub fn tick_air(
    air: i32,
    eye_in_water: bool,
    eye_in_bubble_column: bool,
    effects: &BreathEffects,
    oxygen_bonus: f64,
    player_invulnerable: bool,
    can_breathe_underwater: bool,
    random: &mut crate::math::JavaRandom,
) -> (i32, Option<DrownHit>) {
    if eye_in_water && !eye_in_bubble_column {
        if !can_breathe_underwater && !has_water_breathing(effects) && !player_invulnerable {
            let air = decrease_air_supply(air, oxygen_bonus, random);
            if air <= DROWN_THRESHOLD {
                return (0, Some(DrownHit { damage: DROWN_DAMAGE }));
            }
            (air, None)
        } else if air < MAX_AIR_SUPPLY && effects_refill_air_supply(effects) {
            (increase_air_supply(air), None)
        } else {
            (air, None)
        }
    } else if air < MAX_AIR_SUPPLY {
        (increase_air_supply(air), None)
    } else {
        (air, None)
    }
}
