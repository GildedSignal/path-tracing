//! The state fields the law reads before it writes them: each step of the law reports the
//! groups it reads and those it overwrites whole to its world (`World::state_footprint`),
//! and a recording world keeps the reads. The reports observe places and never substitute
//! values. The fields themselves are declared once, in `state.rs`.
pub use crate::state::StateField;

/// Predicates whose rounded evaluation is not monotone in the free spatial
/// coordinate. The set compiler carries both outcomes at a pin instead of
/// cutting its input interval on these fields. Covered by LAW_HASH.
pub const NON_MONOTONE_FIELDS: &[(StateField, &str)] = &[
    (StateField::minor_horizontal_collision, "movement.rs:818, minor_collision (LocalPlayer.isHorizontalCollisionMinor; acos of rounded cosine)"),
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateReads(pub u128);
impl StateReads {
    pub const ALL: Self = Self((1u128 << StateField::ALL.len()) - 1);
    pub const fn of(fields: &[StateField]) -> Self {
        let (mut mask, mut i) = (0u128, 0);
        while i < fields.len() { mask |= 1u128 << fields[i] as u8; i += 1; }
        Self(mask)
    }
    pub const fn minus(self, other: Self) -> Self { Self(self.0 & !other.0) }
    pub fn contains(self, field: StateField) -> bool { self.0 & (1u128 << field as u8) != 0 }
    pub fn fields(self) -> impl Iterator<Item = StateField> { StateField::ALL.iter().copied().filter(move |&f| self.contains(f)) }
}
/// What the player's tick overwrites whole before it reads it: the look and the
/// per-tick memories (movement.rs, tick_movement's prologue) and the speed (Player.getSpeed
/// after the sprint modifier). The previous input is read only in the creative-flight and
/// sprint-window branches, which report it there.
pub const TICK_WRITES_FIRST: StateReads = StateReads::of(&[StateField::yaw, StateField::pitch,
    StateField::movement_this_tick, StateField::was_in_powder_snow, StateField::was_eye_in_water,
    StateField::speed, StateField::input]);
