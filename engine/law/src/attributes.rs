use crate::PlayerState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedOperation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpeedModifier {
    pub id: String,
    pub amount: f64,
    pub operation: SpeedOperation,
}

// String.hashCode; Identifier.hashCode; fastutil.HashCommon.mix
fn modifier_hash(id: &str) -> usize {
    fn java_hash(s: &str) -> u32 {
        s.encode_utf16().fold(0u32, |hash, c| hash.wrapping_mul(31).wrapping_add(c as u32))
    }
    let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let hash = java_hash(namespace).wrapping_mul(31).wrapping_add(java_hash(path));
    let mixed = hash.wrapping_mul(0x9e3779b9);
    (mixed ^ (mixed >> 16)) as usize
}

// AttributeInstance.getModifiers; Object2ObjectOpenHashMap.insert/rehash/MapIterator
fn operation_order<'a>(modifiers: impl Iterator<Item = &'a SpeedModifier>) -> Vec<&'a SpeedModifier> {
    let mut slots: Vec<Option<&SpeedModifier>> = vec![None; 32];
    let mut size = 0;
    for modifier in modifiers {
        let insert = |slots: &mut Vec<Option<&'a SpeedModifier>>, modifier: &'a SpeedModifier| {
            let mut slot = modifier_hash(&modifier.id) & (slots.len() - 1);
            while slots[slot].is_some() { slot = (slot + 1) & (slots.len() - 1); }
            slots[slot] = Some(modifier);
        };
        insert(&mut slots, modifier);
        size += 1;
        if size > slots.len() * 3 / 4 {
            let mut larger = vec![None; slots.len() * 2];
            for entry in slots.iter().rev().flatten() { insert(&mut larger, entry); }
            slots = larger;
        }
    }
    slots.into_iter().rev().flatten().collect()
}

impl PlayerState {
    // LivingEntity.setSprinting, bytecode 0..35: flag, remove modifier, optional add.
    pub fn set_sprinting(&mut self, sprinting: bool) {
        self.sprinting = sprinting;
        self.sprint_speed_modifier = sprinting;
    }

    pub fn set_speed_effect(&mut self, amplifier: Option<u8>) {
        self.profile.speed_effect = amplifier;
    }

    pub fn set_slowness_effect(&mut self, amplifier: Option<u8>) {
        self.profile.slowness_effect = amplifier;
    }

    pub fn set_jump_boost(&mut self, amplifier: Option<u8>) {
        let bonus = |amp: Option<u8>| amp.map_or(0.0, |a| (a as i32 + 1) as f64);
        self.profile.safe_fall_distance += bonus(amplifier) - bonus(self.profile.jump_boost);
        self.profile.jump_boost = amplifier;
    }

    pub fn set_levitation(&mut self, amplifier: Option<u8>) {
        self.profile.levitation = amplifier;
    }

    pub fn set_slow_falling(&mut self, active: bool) {
        self.profile.slow_falling = active;
    }

    pub fn set_dolphins_grace(&mut self, active: bool) {
        self.profile.dolphins_grace = active;
    }

    pub fn set_weaving(&mut self, active: bool) {
        self.profile.weaving = active;
    }

    pub fn set_movement_enchantments(&mut self, soul_speed: u8, depth_strider: u8, swift_sneak: u8) {
        let enchantments = MovementEnchantments { soul_speed, depth_strider, swift_sneak };
        self.profile.water_movement_efficiency = enchantments.depth_strider_water_efficiency();
        self.profile.sneaking_speed = enchantments.swift_sneak_sneaking_speed();
    }

    /// Evaluates Soul Speed's `location_changed` hysteresis and updates `soul_speed_active`,
    /// `speed_modifiers`, and `movement_efficiency`.
    pub fn apply_soul_speed_location(&mut self, soul_speed_level: u8, on_soul_speed_block: bool) {
        let holds = soul_speed_level > 0
            && MovementEnchantments::soul_speed_holds(
                self.profile.soul_speed_active,
                self.profile.riding,
                self.flying,
                self.on_ground,
                on_soul_speed_block,
            );
        self.profile.speed_modifiers.retain(|m| m.id != "minecraft:enchantment.soul_speed/feet");
        if holds {
            let enchantments = MovementEnchantments { soul_speed: soul_speed_level, ..Default::default() };
            if let Some(modifier) = enchantments.soul_speed_modifier() {
                self.profile.speed_modifiers.push(modifier);
            }
            if !self.profile.soul_speed_active {
                self.profile.movement_efficiency += enchantments.soul_speed_movement_efficiency();
            }
        } else if self.profile.soul_speed_active {
            let enchantments = MovementEnchantments { soul_speed: soul_speed_level.max(1), ..Default::default() };
            self.profile.movement_efficiency = (self.profile.movement_efficiency - enchantments.soul_speed_movement_efficiency()).max(0.0);
        }
        self.profile.soul_speed_active = holds;
    }

    pub fn soul_speed_holds(
        active: bool,
        riding: bool,
        flying: bool,
        on_ground: bool,
        on_soul_speed_block: bool,
    ) -> bool {
        MovementEnchantments::soul_speed_holds(active, riding, flying, on_ground, on_soul_speed_block)
    }

    pub fn soul_speed_modifier_for(level: u8) -> Option<SpeedModifier> {
        MovementEnchantments { soul_speed: level, ..Default::default() }.soul_speed_modifier()
    }

    /// `protection.json`: `add` linear (1.0, 1.0) EPF per piece.
    pub fn protection_epf_for(level: u8) -> f32 {
        if level == 0 { 0.0 } else { linear_level_value(1.0, 1.0, level) }
    }

    /// `feather_falling.json`: `add` linear (3.0, 3.0) EPF on `is_fall && !bypasses_invulnerability`.
    pub fn feather_falling_epf_for(level: u8) -> f32 {
        if level == 0 { 0.0 } else { linear_level_value(3.0, 3.0, level) }
    }

    /// `LivingEntity.getJumpBoostPower()`: `0.1f * ((float)amplifier + 1.0f)` when active, else `0.0f`.
    pub fn jump_boost_power(&self) -> f32 {
        self.profile.jump_boost.map_or(0.0f32, |amp| 0.1f32 * ((amp as i32 as f32) + 1.0f32))
    }

    pub fn abilities(&self) -> crate::player::Abilities {
        crate::player::Abilities {
            invulnerable: self.profile.invulnerable,
            flying: self.flying,
            mayfly: self.profile.mayfly,
            instabuild: self.profile.instabuild,
            may_build: self.profile.may_build,
            flying_speed: self.profile.flying_speed,
            walking_speed: self.profile.movement_speed_base as f32,
        }
    }

    pub fn set_abilities(&mut self, abilities: crate::player::Abilities) {
        self.profile.invulnerable = abilities.invulnerable;
        self.flying = abilities.flying;
        self.profile.mayfly = abilities.mayfly;
        self.profile.instabuild = abilities.instabuild;
        self.profile.may_build = abilities.may_build;
        self.profile.flying_speed = abilities.flying_speed;
    }

    pub fn set_game_mode(&mut self, mode: crate::player::GameType) {
        self.profile.game_mode = mode;
        let mut abilities = self.abilities();
        crate::player::update_player_abilities(mode, &mut abilities);
        self.set_abilities(abilities);
    }

    // AttributeInstance.calculateValue; LivingEntity.setSprinting; RangedAttribute.sanitizeValue
    pub fn movement_speed_value(&self) -> f64 {
        let speed = self.profile.speed_effect
            .filter(|_| !self.profile.speed_modifiers.iter().any(|m| m.id == "minecraft:effect.speed"))
            .map(|amp| SpeedModifier {
                id: "minecraft:effect.speed".to_owned(),
                amount: 0.20000000298023224f64 * ((amp as i32 + 1) as f64),
                operation: SpeedOperation::AddMultipliedTotal,
            });
        let slowness = self.profile.slowness_effect
            .filter(|_| !self.profile.speed_modifiers.iter().any(|m| m.id == "minecraft:effect.slowness"))
            .map(|amp| SpeedModifier {
                id: "minecraft:effect.slowness".to_owned(),
                amount: -0.15000000596046448f64 * ((amp as i32 + 1) as f64),
                operation: SpeedOperation::AddMultipliedTotal,
            });
        if self.profile.speed_modifiers.is_empty() && speed.is_none() && slowness.is_none() {
            let value = if self.sprint_speed_modifier {
                self.profile.movement_speed_base * (1.0 + 0.3f32 as f64)
            } else {
                self.profile.movement_speed_base
            };
            return if value.is_nan() { 0.0 } else { value.clamp(0.0, 1024.0) };
        }
        let sprint = SpeedModifier {
            id: "minecraft:sprinting".to_owned(),
            amount: 0.3f32 as f64,
            operation: SpeedOperation::AddMultipliedTotal,
        };
        let ordered = |operation| operation_order(
            self.profile.speed_modifiers.iter()
                .chain(self.sprint_speed_modifier.then_some(&sprint))
                .chain(speed.as_ref())
                .chain(slowness.as_ref())
                .filter(move |modifier| modifier.operation == operation));
        let mut base = self.profile.movement_speed_base;
        for modifier in ordered(SpeedOperation::AddValue) { base += modifier.amount; }
        let mut value = base;
        for modifier in ordered(SpeedOperation::AddMultipliedBase) { value += base * modifier.amount; }
        for modifier in ordered(SpeedOperation::AddMultipliedTotal) { value *= 1.0 + modifier.amount; }
        if value.is_nan() { 0.0 } else { value.clamp(0.0, 1024.0) }
    }
}

// Equipment-driven movement enchantments, from the 26.2 jar:
// `data/minecraft/enchantment/{soul_speed,depth_strider,swift_sneak}.json` (mirrored by the
// `Enchantments` bootstrap constants).
// `LevelBasedValue.Linear.calculate`: `base + per_level_above_first * (level - 1)`, f32, product first.
fn linear_level_value(base: f32, per_level_above_first: f32, level: u8) -> f32 {
    base + per_level_above_first * (level as i32 - 1) as f32
}

/// Boots and leggings movement enchantments; level 0 = not enchanted. Max level 3 for all three.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MovementEnchantments {
    /// Boots, `slots: ["feet"]`.
    pub soul_speed: u8,
    /// Boots, `slots: ["feet"]`.
    pub depth_strider: u8,
    /// Leggings, `slots: ["legs"]`.
    pub swift_sneak: u8,
}

impl MovementEnchantments {
    /// Soul Speed's `minecraft:location_changed` attribute effect on movement_speed:
    /// `add_value`, linear (0.0405, 0.0105), modifier id `minecraft:enchantment.soul_speed/feet`.
    pub fn soul_speed_modifier(&self) -> Option<SpeedModifier> {
        (self.soul_speed > 0).then(|| SpeedModifier {
            id: "minecraft:enchantment.soul_speed/feet".to_owned(),
            amount: linear_level_value(0.0405, 0.0105, self.soul_speed) as f64,
            operation: SpeedOperation::AddValue,
        })
    }

    /// Soul Speed's second effect: `movement_efficiency` +1.0, a constant `add_value`.
    pub fn soul_speed_movement_efficiency(&self) -> f32 {
        (self.soul_speed > 0).then_some(1.0).unwrap_or(0.0)
    }

    /// Depth Strider on `water_movement_efficiency` (base 0.0): `add_value`, linear (0.33333334, 0.33333334).
    pub fn depth_strider_water_efficiency(&self) -> f32 {
        if self.depth_strider == 0 {
            0.0
        } else {
            linear_level_value(0.33333334, 0.33333334, self.depth_strider)
        }
    }

    /// Swift Sneak on `sneaking_speed` (base 0.3): `add_value`, linear (0.15, 0.15).
    pub fn swift_sneak_sneaking_speed(&self) -> f32 {
        if self.swift_sneak == 0 {
            0.3
        } else {
            (0.3f64 + linear_level_value(0.15, 0.15, self.swift_sneak) as f64) as f32
        }
    }

    /// Soul-speed condition from `soul_speed.json`: holds while `!riding && !flying` and
    /// `(on_soul_speed_block || (active && !on_ground))`.
    pub fn soul_speed_holds(
        active: bool,
        riding: bool,
        flying: bool,
        on_ground: bool,
        on_soul_speed_block: bool,
    ) -> bool {
        !riding && !flying && (on_soul_speed_block || active && !on_ground)
    }

}
