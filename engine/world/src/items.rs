use std::{collections::BTreeMap, sync::OnceLock};

/// Registry defaults extracted from the 26.2 jar. Instance component overrides
/// must be supplied by the caller instead of this table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UseEffects { pub can_sprint: bool, pub speed_multiplier: f32 }

pub fn use_effects(item: &str) -> Option<UseEffects> {
    static ITEMS: OnceLock<BTreeMap<&'static str, UseEffects>> = OnceLock::new();
    ITEMS.get_or_init(|| include_str!("../data/use-effects.tsv").lines().map(|line| {
        let [id, sprint, bits] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated item row") };
        (id, UseEffects { can_sprint: sprint.parse().expect("generated canSprint"), speed_multiplier: f32::from_bits(u32::from_str_radix(bits, 16).expect("generated multiplier")) })
    }).collect()).get(item).copied()
}
