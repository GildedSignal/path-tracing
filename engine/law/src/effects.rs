//! The periodic status-effect law of the 26.2 server jar:
//! `LivingEntity.tickEffects` (server branch), `MobEffectInstance.tickServer`,
//! and the per-effect rules (`Poison/Wither/Regeneration/Hunger/Saturation/
//! HealOrHarm/AbsorptionMobEffect`, plain `MobEffect` for HEALTH_BOOST).
//!
//! The tick state machine is copied from `MobEffectInstance.tickServer`
//! (javap offsets cited per function). Resource deltas execute the jar's own
//! `heal` / `hurtServer`+`actuallyHurt` semantics for a bare survival player
//! (no armor, no resistance, no enchantments): the hurt cooldown window is the
//! same `LivingEntity.hurtServer` offsets 184-258 law already ported as
//! `HurtCooldown`/`accept_raw_hit` in `movement.rs`; `effects.absorption`
//! covers `LivingEntity.actuallyHurt` offsets 27-144; health clamps are
//! `LivingEntity.setHealth` (`Mth.clamp(v, 0, max)`).
//!
//! Server wiring: `LivingEntity.baseTick` calls `tickEffects` at offset 604,
//! before `Player.aiStep` runs the food/exhaustion law, and decrements the hurt
//! cooldown in the same pass. A tick here assumes that decrement already
//! happened (`ServerResourceEvent::HurtCooldownTick`).

use crate::state::{HurtCooldown, PlayerState};

/// The kit's verbs for the effects the bot's plan consumes. A block is its
/// answers to the kit's verbs; an effect is its answers to these.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    Poison,
    Wither,
    Regeneration,
    Hunger,
    InstantHealth,
    InstantDamage,
    Saturation,
    Absorption,
    HealthBoost,
}

impl EffectKind {
    /// `MobEffectInstance` static init: amplifier = `Mth.clamp(amp, 0, 255)`.
    fn clamp_amplifier(amp: i32) -> i32 {
        amp.clamp(0, 255)
    }
}

/// `net.minecraft.world.effect.MobEffectInstance` in the tick-relevant fields.
/// `duration` -1 is infinite; `hidden` is the stack of weaker, longer effects
/// kept when a stronger, shorter one is merged over it (`MobEffectInstance.update`).
#[derive(Clone, Debug, PartialEq)]
pub struct MobEffectInstance {
    pub kind: EffectKind,
    pub duration: i32,
    pub amplifier: i32,
    hidden: Option<Box<MobEffectInstance>>,
}

impl MobEffectInstance {
    pub fn new(kind: EffectKind, duration: i32, amplifier: i32) -> Self {
        Self { kind, duration, amplifier: EffectKind::clamp_amplifier(amplifier), hidden: None }
    }

    pub fn hidden(&self) -> Option<&MobEffectInstance> {
        self.hidden.as_deref()
    }

    /// `MobEffectInstance.tickServer(ServerLevel, LivingEntity, Runnable)`,
    /// javap offsets 0-100. Returns the jar's boolean: false means the effect
    /// is removed (`tickEffects` iterator.remove + onEffectsRemoved).
    pub fn tick_server(&mut self, player: &mut PlayerState, tick: EffectTick<'_>, tick_count: i32) -> bool {
        // 0-8: hasRemainingDuration — infinite (-1) or duration > 0.
        if !(self.duration > 0 || self.duration == -1) {
            return false;
        }
        // 9-27: the period test reads entity.tickCount while infinite.
        let clock = if self.duration == -1 { tick_count } else { self.duration };
        // 29-78: apply, and drop the effect if applyEffectTick returns false.
        if self.should_apply_effect_tick_this_tick(clock) && !self.apply_effect_tick(player, tick) {
            return false;
        }
        // 79-82: duration -= 1, but -1 and 0 map to themselves; hidden ticks first.
        self.tick_down_duration();
        // 83-95: at 0 with a hidden stack, take its details and notify (onEffectUpdated).
        if self.duration == 0 && self.hidden.is_some() {
            let hidden = *self.hidden.take().expect("hidden presence checked");
            self.set_details_from(&hidden);
            self.hidden = hidden.hidden;
            // The runnable is onEffectUpdated(force=true): attribute modifiers
            // are removed and re-added; no resource changes at this grade.
        }
        // 96-100.
        self.duration > 0 || self.duration == -1
    }

    /// `MobEffect.shouldApplyEffectTickThisTick(int duration, int amplifier)`.
    pub fn should_apply_effect_tick_this_tick(&self, duration: i32) -> bool {
        let amp = self.amplifier;
        // Java int shifts mask the count to 5 bits; j <= 0 means every tick.
        let interval = |base: i32| base >> (amp & 31);
        match self.kind {
            // PoisonMobEffect: 25 >> amp, min 1 tick.
            EffectKind::Poison => { let j = interval(25); j <= 0 || duration % j == 0 }
            // WitherMobEffect: 40 >> amp, min 1 tick.
            EffectKind::Wither => { let j = interval(40); j <= 0 || duration % j == 0 }
            // RegenerationMobEffect: 50 >> amp, min 1 tick.
            EffectKind::Regeneration => { let j = interval(50); j <= 0 || duration % j == 0 }
            // HungerMobEffect / AbsorptionMobEffect: every tick.
            EffectKind::Hunger | EffectKind::Absorption => true,
            // InstantaneousMobEffect: duration >= 1.
            EffectKind::InstantHealth | EffectKind::InstantDamage | EffectKind::Saturation => duration >= 1,
            // Plain MobEffect base: never periodic.
            EffectKind::HealthBoost => false,
        }
    }

    /// `MobEffect.applyEffectTick(ServerLevel, LivingEntity, int amplifier)` for
    /// each subclass. Returns false where the jar's override returns false
    /// (only Absorption, once its shield is spent).
    fn apply_effect_tick(&mut self, player: &mut PlayerState, tick: EffectTick<'_>) -> bool {
        let amp = self.amplifier;
        match self.kind {
            // PoisonMobEffect offsets 0-24: only if health > 1.0f, then 1.0f
            // magic damage. The guard is the poison floor: from 1.5 the debit
            // lands at 0.5; from 1.0 or below, nothing.
            EffectKind::Poison => {
                if player.health > 1.0 {
                    hurt(player, tick, 1.0);
                }
                true
            }
            // WitherMobEffect offsets 0-15: unconditional 1.0f wither damage, lethal.
            EffectKind::Wither => { hurt(player, tick, 1.0); true }
            // RegenerationMobEffect offsets 0-18: heal(1.0f) only under max.
            EffectKind::Regeneration => {
                if player.health < *tick.max_health {
                    heal(player, tick, 1.0);
                }
                true
            }
            // HungerMobEffect offsets 0-25: Player.causeFoodExhaustion(0.005f * (amp+1)).
            EffectKind::Hunger => {
                add_exhaustion(player, 0.005f32 * (amp + 1) as f32);
                true
            }
            // HealOrHarmMobEffect.applyEffectTick offsets 0-45: a player is not
            // inverted, so HEAL heals max(4 << amp, 0) and HARM deals 6 << amp magic.
            EffectKind::InstantHealth => { heal(player, tick, (4i32 << (amp & 31)).max(0) as f32); true }
            EffectKind::InstantDamage => { hurt(player, tick, (6i32 << (amp & 31)) as f32); true }
            // SaturationMobEffect offsets 0-25: FoodData.eat(amp + 1, 1.0f).
            EffectKind::Saturation => {
                eat(player, amp + 1, 1.0f32);
                true
            }
            // AbsorptionMobEffect offsets 0-14: keep while any absorption remains.
            EffectKind::Absorption => *tick.absorption > 0.0,
            // Plain MobEffect base: return true.
            EffectKind::HealthBoost => true,
        }
    }

    /// `MobEffectInstance.tickDownDuration` + `mapDuration` + `lambda$tickDownDuration$0`.
    fn tick_down_duration(&mut self) {
        if let Some(hidden) = self.hidden.as_deref_mut() {
            hidden.tick_down_duration();
        }
        if self.duration > 0 {
            self.duration -= 1;
        }
    }

    /// `MobEffectInstance.setDetailsFrom`: duration, amplifier, visibility flags.
    fn set_details_from(&mut self, other: &MobEffectInstance) {
        self.duration = other.duration;
        self.amplifier = other.amplifier;
    }

    /// `MobEffectInstance.update(MobEffectInstance)`, javap offsets 0-227: merge
    /// a freshly applied instance into this one. Returns whether anything changed.
    pub fn update(&mut self, other: &MobEffectInstance) -> bool {
        let mut changed = false;
        if other.amplifier > self.amplifier {
            // 39-71: a stronger (even shorter) dose hides the running one.
            let chained = self.hidden.take();
            let mut hidden = Box::new(self.clone());
            hidden.hidden = chained;
            self.hidden = Some(hidden);
            self.amplifier = other.amplifier;
            self.duration = other.duration;
            changed = true;
        } else if other.amplifier == self.amplifier && other.duration > self.duration {
            // 93-122: same level, longer: extend.
            self.duration = other.duration;
            changed = true;
        } else if other.amplifier < self.amplifier {
            // 125-155: weaker dose merges into the hidden stack.
            match self.hidden.as_deref_mut() {
                None => self.hidden = Some(Box::new(other.clone())),
                Some(hidden) => { hidden.update(other); }
            }
        }
        changed
    }
}

/// The effect book the server holds per entity: the active instances in the
/// order `LivingEntity.activeEffects` would iterate them, plus the two
/// effect-owned resources the rest of the law does not carry.
#[derive(Clone, Debug)]
pub struct Effects {
    pub instances: Vec<MobEffectInstance>,
    /// `LivingEntity.absorptionAmount`, floor 0, cap `max_absorption` = 4*(amp+1)
    /// while ABSORPTION runs (MAX_ABSORPTION attribute, else 0).
    pub absorption: f32,
    /// MAX_HEALTH while HEALTH_BOOST runs; 20.0 bare.
    pub max_health: f32,
}

impl Default for Effects {
    fn default() -> Self {
        Self { instances: Vec::new(), absorption: 0.0, max_health: 20.0 }
    }
}

impl Effects {
    /// `LivingEntity.addEffect` + `MobEffectInstance.onEffectStarted`:
    /// merge into a running instance of the same kind, else push; then the
    /// start hooks (AbsorptionMobEffect.onEffectStarted: absorption =
    /// max(absorption, 4*(amp+1)); HEALTH_BOOST's MAX_HEALTH modifier).
    pub fn add(&mut self, kind: EffectKind, duration: i32, amplifier: i32, _player: &mut PlayerState) {
        let fresh = MobEffectInstance::new(kind, duration, amplifier);
        if let Some(existing) = self.instances.iter_mut().find(|i| i.kind == kind) {
            existing.update(&fresh);
        } else {
            self.instances.push(fresh);
        }
        let amp = amplifier.clamp(0, 255);
        match kind {
            EffectKind::Absorption => {
                // setAbsorptionAmount clamps to [0, max_absorption]; the modifier
                // is already in place when onEffectStarted runs, so 4*(amp+1) fits.
                self.absorption = self.absorption.max(4.0f32 * (amp + 1) as f32);
            }
            EffectKind::HealthBoost => {
                // addAttributeModifiers: MAX_HEALTH + 4.0 * (amp+1), ADD_VALUE.
                self.max_health = 20.0 + 4.0f32 * (amp + 1) as f32;
            }
            _ => {}
        }
    }

    /// `LivingEntity.tickEffects` (server branch, javap offsets 12-125) for one
    /// server tick: every instance ticks; `false` removes it and runs the
    /// removal hooks (`onEffectsRemoved` -> removeAttributeModifiers ->
    /// refreshDirtyAttributes). The %600 branch only resyncs packets.
    pub fn tick(&mut self, player: &mut PlayerState, tick_count: i32) {
        let mut i = 0;
        while i < self.instances.len() {
            let Effects { instances, absorption, max_health } = self;
            let kept = instances[i].tick_server(player, EffectTick { absorption, max_health }, tick_count);
            if !kept {
                let removed = self.instances.remove(i);
                match removed.kind {
                    // MAX_ABSORPTION modifier leaves: absorption clamps to 0.
                    EffectKind::Absorption => self.absorption = 0.0,
                    // MAX_HEALTH modifier leaves: max returns to 20.0 and health
                    // clamps to it (attribute-changed handler).
                    EffectKind::HealthBoost => {
                        self.max_health = 20.0;
                        player.health = player.health.min(self.max_health);
                    }
                    _ => {}
                }
            } else {
                i += 1;
            }
        }
    }

    /// `HealOrHarmMobEffect.applyInstantaneousEffect` (and the base default for
    /// Saturation, which ignores potency): `(int)(potency * (4|6 << amp) + 0.5)`.
    pub fn apply_instantaneous(&mut self, kind: EffectKind, amplifier: i32, potency: f64, player: &mut PlayerState) {
        let amp = amplifier.clamp(0, 255);
        match kind {
            EffectKind::InstantHealth => {
                let amount = (potency * ((4i32 << (amp & 31)) as f64) + 0.5) as i32;
                heal(player, EffectTick { absorption: &mut self.absorption, max_health: &mut self.max_health }, amount as f32);
            }
            EffectKind::InstantDamage => {
                let amount = (potency * ((6i32 << (amp & 31)) as f64) + 0.5) as i32;
                hurt(player, EffectTick { absorption: &mut self.absorption, max_health: &mut self.max_health }, amount as f32);
            }
            EffectKind::Saturation => eat(player, amp + 1, 1.0f32),
            _ => {}
        }
    }
}

// LivingEntity.heal offsets 0-18: only the living heal, and setHealth clamps.
fn heal(player: &mut PlayerState, tick: EffectTick<'_>, amount: f32) {
    if player.health > 0.0 {
        player.health = (player.health + amount).clamp(0.0, *tick.max_health);
    }
}

// LivingEntity.hurtServer offsets 0-10, 164-181, 184-262 + actuallyHurt offsets
// 10-154 for a bare player: the hurt window gates, absorption eats, health pays.
fn hurt(player: &mut PlayerState, tick: EffectTick<'_>, amount: f32) {
    // isDeadOrDying gate; NaN amounts cap to Float.MAX_VALUE in the jar.
    if player.health <= 0.0 {
        return;
    }
    let amount = if amount.is_nan() { f32::MAX } else { amount };
    if let Some(debit) = accept_effect_hit(&mut player.hurt_cooldown, amount) {
        // actuallyHurt offsets 27-48: absorption first, then health, floors 0.
        let absorbed = debit.min(*tick.absorption);
        *tick.absorption -= absorbed;
        player.health = (player.health - (debit - absorbed)).max(0.0);
    }
}

/// The two effect-owned resources a tick mutates, borrowed disjointly from
/// `Effects` so one instance can tick while the book is iterated.
pub struct EffectTick<'a> {
    pub absorption: &'a mut f32,
    pub max_health: &'a mut f32,
}

// LivingEntity.hurtServer offsets 184-262: the invulnerability window, already
// ported as movement.rs's accept_raw_hit from the same bytecode.
fn accept_effect_hit(cooldown: &mut HurtCooldown, amount: f32) -> Option<f32> {
    crate::movement::accept_raw_hit(cooldown, amount)
}
// FoodData.addExhaustion offsets 0-15: capped at 40.0f.
fn add_exhaustion(player: &mut PlayerState, amount: f32) {
    player.exhaustion = (player.exhaustion + amount).min(40.0);
}

// FoodData.eat -> FoodConstants.saturationByModifier(i, f) = i as f32 * f * 2.0f,
// then FoodData.add: food clamps to [0, 20], saturation to [0, food].
fn eat(player: &mut PlayerState, food: i32, modifier: f32) {
    player.food_level = (player.food_level + food).clamp(0, 20);
    player.saturation = (player.saturation + food as f32 * modifier * 2.0f32).clamp(0.0, player.food_level as f32);
}

#[cfg(test)]
mod jar_regression {
    use super::*;

    fn player(health: f32) -> PlayerState {
        PlayerState { health, ..Default::default() }
    }

    /// Jar: shouldApplyEffectTickThisTick first-applies at these durations.
    #[test]
    fn periods_match_the_jar_tables() {
        let first = |kind, amp| {
            let inst = MobEffectInstance::new(kind, 0, amp);
            (1..=1000).find(|d| inst.should_apply_effect_tick_this_tick(*d))
        };
        for (kind, amp, want) in [
            (EffectKind::Poison, 0, Some(25)), (EffectKind::Poison, 1, Some(12)),
            (EffectKind::Poison, 2, Some(6)), (EffectKind::Poison, 3, Some(3)),
            (EffectKind::Wither, 0, Some(40)), (EffectKind::Wither, 1, Some(20)),
            (EffectKind::Wither, 2, Some(10)), (EffectKind::Wither, 3, Some(5)),
            (EffectKind::Regeneration, 0, Some(50)), (EffectKind::Regeneration, 1, Some(25)),
            (EffectKind::Regeneration, 2, Some(12)), (EffectKind::Regeneration, 3, Some(6)),
            (EffectKind::Hunger, 3, Some(1)), (EffectKind::Saturation, 0, Some(1)),
            (EffectKind::InstantHealth, 2, Some(1)), (EffectKind::InstantDamage, 3, Some(1)),
            (EffectKind::Absorption, 1, Some(1)),
            (EffectKind::HealthBoost, 0, None),
        ] {
            assert_eq!(first(kind, amp), want, "{kind:?} amp {amp}");
        }
    }

    /// Jar: hunger exhaustion f32 bits after ticks 1..5, levels 0..=3.
    #[test]
    fn hunger_exhaustion_bits_match_the_jar() {
        let want: [[u32; 5]; 4] = [
            [0x3BA3D70A, 0x3C23D70A, 0x3C75C28F, 0x3CA3D70A, 0x3CCCCCCC],
            [0x3C23D70A, 0x3CA3D70A, 0x3CF5C28F, 0x3D23D70A, 0x3D4CCCCC],
            [0x3C75C28F, 0x3CF5C28F, 0x3D3851EB, 0x3D75C28F, 0x3D999999],
            [0x3CA3D70A, 0x3D23D70A, 0x3D75C28F, 0x3DA3D70A, 0x3DCCCCCC],
        ];
        for amp in 0..=3 {
            let mut p = player(20.0);
            let mut effects = Effects::default();
            effects.add(EffectKind::Hunger, 10, amp, &mut p);
            for t in 0..5 {
                effects.tick(&mut p, 0);
                assert_eq!(p.exhaustion.to_bits(), want[amp as usize][t as usize], "amp {amp} tick {}", t + 1);
            }
        }
    }

    /// Jar: saturation eats (amp+1) food and 2*(amp+1) saturation per tick from empty.
    #[test]
    fn saturation_feeds_like_the_jar() {
        let mut p = player(20.0);
        p.food_level = 0;
        p.saturation = 0.0;
        let mut effects = Effects::default();
        effects.add(EffectKind::Saturation, 3, 3, &mut p);
        for (food, sat, kept) in [(4i32, 4.0f32, true), (8, 8.0, true), (12, 12.0, false)] {
            effects.tick(&mut p, 0);
            assert_eq!((p.food_level, p.saturation.to_bits(), !effects.instances.is_empty()), (food, sat.to_bits(), kept));
        }
    }

    /// Jar: regeneration heals 1.0f each 50>>amp ticks and clamps at max health.
    #[test]
    fn regeneration_cadence_matches_the_jar() {
        let mut p = player(0.5);
        let mut effects = Effects::default();
        effects.add(EffectKind::Regeneration, 260, 2, &mut p);
        let mut changes = Vec::new();
        for t in 1..=260 {
            let before = p.health;
            effects.tick(&mut p, 0);
            if p.health != before {
                changes.push((t, p.health));
            }
        }
        assert_eq!(&changes[..6], &[(9, 1.5), (21, 2.5), (33, 3.5), (45, 4.5), (57, 5.5), (69, 6.5)][..]);
        // The jar's last change is @237:20.0 (setHealth clamp from 20.5).
        assert_eq!(*changes.last().expect("regen healed"), (237, 20.0));
    }

    /// Jar: poison never fires at or below 1.0f; from 1.5 the debit lands at
    /// 0.5; absorption eats the debit first.
    #[test]
    fn poison_floor_and_absorption_eat_like_the_jar() {
        let mut p = player(1.0);
        let mut effects = Effects::default();
        effects.add(EffectKind::Poison, 25, 0, &mut p);
        effects.tick(&mut p, 0);
        assert_eq!(p.health, 1.0, "the guard blocks at exactly 1.0f");

        let mut p = player(1.5);
        let mut effects = Effects::default();
        effects.add(EffectKind::Poison, 25, 0, &mut p);
        effects.tick(&mut p, 0);
        assert_eq!(p.health.to_bits(), 0.5f32.to_bits());

        let mut p = player(1.5);
        let mut effects = Effects::default();
        effects.add(EffectKind::Absorption, 100, 0, &mut p);
        effects.add(EffectKind::Poison, 25, 0, &mut p);
        effects.tick(&mut p, 0);
        assert_eq!((p.health.to_bits(), effects.absorption.to_bits()), (1.5f32.to_bits(), 3.0f32.to_bits()));
    }

    /// Jar: wither is lethal; repeated 1.0f debits take a bare player to 0.
    #[test]
    fn wither_kills() {
        let mut p = player(20.0);
        let mut effects = Effects::default();
        effects.add(EffectKind::Wither, 1000, 0, &mut p);
        let mut ticks = 0;
        while p.health > 0.0 && ticks < 1000 {
            // baseTick decrements the hurt window each server tick.
            if p.hurt_cooldown.remaining_ticks > 0 {
                p.hurt_cooldown.remaining_ticks -= 1;
            }
            effects.tick(&mut p, 0);
            ticks += 1;
        }
        assert_eq!(p.health, 0.0);
    }

    /// Jar: absorption is 4*(amp+1) on add, max() on re-add, removed when
    /// drained, and its shield is clamped away on expiry.
    #[test]
    fn absorption_lifecycle_matches_the_jar() {
        for amp in 0..=3 {
            let mut p = player(20.0);
            let mut effects = Effects::default();
            effects.add(EffectKind::Absorption, 10, amp, &mut p);
            assert_eq!(effects.absorption, 4.0 * (amp + 1) as f32);
            effects.add(EffectKind::Absorption, 10, amp, &mut p);
            assert_eq!(effects.absorption, 4.0 * (amp + 1) as f32, "re-add is max()");
            // One raw 1.0f magic debit eats one absorption point.
            super::hurt(&mut p, EffectTick { absorption: &mut effects.absorption, max_health: &mut effects.max_health }, 1.0);
            assert_eq!((effects.absorption, p.health), (4.0 * (amp + 1) as f32 - 1.0, 20.0));
            // Drain the shield the way the harness drained it with the jar's
            // actuallyHurt: repeated equal debits, outside the hurt window
            // (inside the window, 4.0 <= lastHurt 4.0 is refused).
            while effects.absorption > 0.0 {
                let debit = 4.0f32.min(effects.absorption);
                effects.absorption -= debit;
            }
            effects.tick(&mut p, 0);
            assert!(effects.instances.is_empty(), "drained absorption effect removes itself");
            // Duration expiry on a fresh book zeroes the shield via MAX_ABSORPTION.
            let mut p2 = player(20.0);
            let mut effects2 = Effects::default();
            effects2.add(EffectKind::Absorption, 3, amp, &mut p2);
            for _ in 0..5 {
                effects2.tick(&mut p2, 0);
            }
            assert!(effects2.instances.is_empty());
            assert_eq!(effects2.absorption, 0.0, "onEffectsRemoved clamps the shield to the new max");
        }
    }

    /// Jar: instant heal is max(4<<amp,0) per tick and (int)(potency*(4<<amp)+0.5)
    /// per potion; instant damage is 6<<amp; the harm side floors at 0 health.
    #[test]
    fn instant_effects_scale_like_the_jar() {
        let heal_from_1 = |amp, potency| {
            let mut p = player(1.0);
            let mut effects = Effects::default();
            effects.apply_instantaneous(EffectKind::InstantHealth, amp, potency, &mut p);
            p.health
        };
        assert_eq!(heal_from_1(0, 1.0), 5.0);
        assert_eq!(heal_from_1(2, 0.5), 9.0, "(int)(0.5*16+0.5) = 8");
        assert_eq!(heal_from_1(3, 0.5), 17.0, "(int)(0.5*32+0.5) = 16");
        // 6<<amp magic through a fresh hurt window from full health.
        let harm_from_full = |amp| {
            let mut p = player(20.0);
            let mut effects = Effects::default();
            effects.apply_instantaneous(EffectKind::InstantDamage, amp, 1.0, &mut p);
            p.health
        };
        assert_eq!(harm_from_full(0), 14.0);
        assert_eq!(harm_from_full(1), 8.0);
        assert_eq!(harm_from_full(2), 0.0, "24 damage floors a bare player");
    }
    /// Jar: health boost adds 4*(amp+1) max health and clamps health back on removal.
    #[test]
    fn health_boost_bounds_match_the_jar() {
        for amp in 0..=3 {
            let mut p = player(20.0);
            let mut effects = Effects::default();
            effects.add(EffectKind::HealthBoost, 100, amp, &mut p);
            assert_eq!(effects.max_health, 20.0 + 4.0 * (amp + 1) as f32);
            p.health = effects.max_health;
            // Expire it: the MAX_HEALTH modifier leaves and health clamps to 20.
            effects.instances[0].duration = 1;
            effects.tick(&mut p, 0);
            assert_eq!((effects.max_health, p.health), (20.0, 20.0));
        }
    }

    /// Jar: a stronger shorter dose hides the running one; at 0 the stack
    /// downgrades and runs out at 10 + 90 ticks.
    #[test]
    fn hidden_stack_downgrades_and_expires_like_the_jar() {
        let mut p = player(20.0);
        let mut effects = Effects::default();
        effects.add(EffectKind::Hunger, 100, 0, &mut p);
        effects.add(EffectKind::Hunger, 10, 1, &mut p);
        let stored = effects.instances.last().unwrap();
        assert_eq!((stored.amplifier, stored.duration), (1, 10));
        assert!(stored.hidden().is_some(), "the weak dose is hidden, not dropped");
        let mut downgraded_at = None;
        let mut expired_at = None;
        for t in 1..=120 {
            // The harness samples the amplifier before each tick, so its
            // downgradedAtTick is the tick after the in-tick downgrade.
            if let Some(stored) = effects.instances.last() {
                if downgraded_at.is_none() && stored.amplifier == 0 {
                    downgraded_at = Some(t);
                }
            }
            effects.tick(&mut p, 0);
            if effects.instances.is_empty() {
                expired_at = Some(t);
                break;
            }
        }
        assert_eq!((downgraded_at, expired_at), (Some(11), Some(100)));
    }

    /// Jar: infinite (-1) duration never decrements; a 0-duration instance is dropped.
    #[test]
    fn infinite_and_zero_durations_match_the_jar() {
        let mut p = player(20.0);
        let mut effects = Effects::default();
        effects.add(EffectKind::Hunger, -1, 0, &mut p);
        for _ in 0..50 {
            effects.tick(&mut p, 0);
        }
        assert_eq!(effects.instances[0].duration, -1);
        let mut zero = MobEffectInstance::new(EffectKind::Hunger, 0, 0);
        let mut p = player(20.0);
        let mut effects = Effects::default();
        assert!(!zero.tick_server(&mut p, EffectTick { absorption: &mut effects.absorption, max_health: &mut effects.max_health }, 0));
    }
}
