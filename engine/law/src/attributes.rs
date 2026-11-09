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

    // AttributeInstance.calculateValue; LivingEntity.setSprinting; RangedAttribute.sanitizeValue
    pub fn movement_speed_value(&self) -> f64 {
        if self.speed_modifiers.is_empty() {
            let value = if self.sprint_speed_modifier {
                self.movement_speed_base * (1.0 + 0.3f32 as f64)
            } else {
                self.movement_speed_base
            };
            return if value.is_nan() { 0.0 } else { value.clamp(0.0, 1024.0) };
        }
        let sprint = SpeedModifier {
            id: "minecraft:sprinting".to_owned(),
            amount: 0.3f32 as f64,
            operation: SpeedOperation::AddMultipliedTotal,
        };
        let ordered = |operation| operation_order(
            self.speed_modifiers.iter().chain(self.sprint_speed_modifier.then_some(&sprint))
                .filter(move |modifier| modifier.operation == operation));
        let mut base = self.movement_speed_base;
        for modifier in ordered(SpeedOperation::AddValue) { base += modifier.amount; }
        let mut value = base;
        for modifier in ordered(SpeedOperation::AddMultipliedBase) { value += base * modifier.amount; }
        for modifier in ordered(SpeedOperation::AddMultipliedTotal) { value *= 1.0 + modifier.amount; }
        if value.is_nan() { 0.0 } else { value.clamp(0.0, 1024.0) }
    }
}
