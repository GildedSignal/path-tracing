//! Discrete state changes used by the joint wave.
use law::{PlayerState, state_reads::StateField as F};
#[derive(Clone, Copy)]
pub struct Exit(pub u128);
enum ExitKind { State(F) }
impl ExitKind { fn bit(self)->u128 { match self { Self::State(field)=>1 << field as u8 } } }
pub(crate) fn events(a: &PlayerState, b: &PlayerState) -> Exit {
    let mut exit = 0;
    macro_rules! cut {
        ($($field:ident),*) => { $(if a.$field != b.$field { exit |= ExitKind::State(F::$field).bit(); })* };
    }
    macro_rules! running {
        ($($field:ident),*) => { $(if (a.$field == 0) != (b.$field == 0) { exit |= ExitKind::State(F::$field).bit(); })* };
    }
    cut!(on_ground, horizontal_collision, vertical_collision, vertical_collision_below, minor_horizontal_collision,
        sprinting, sprint_speed_modifier, no_physics, shift_key_down, pose, in_water, in_lava, on_climbable,
        main_supporting_block, on_ground_no_blocks, crouching, swimming, underwater, eye_in_water, in_powder_snow,
        leather_boots, fall_flying, riding, boat, effects, flying, using_item, stuck_speed_multiplier);
    running!(no_jump_delay, remaining_fire_ticks, ticks_frozen);
    if a.health.to_bits() != b.health.to_bits() { exit |= ExitKind::State(F::health).bit(); }
    if (a.hurt_cooldown.remaining_ticks == 0) != (b.hurt_cooldown.remaining_ticks == 0) { exit |= ExitKind::State(F::hurt_cooldown).bit(); }
    Exit(exit)
}
/// The joint signature omits only the address-only body-block crossing.
pub fn joint_events(a:&PlayerState,b:&PlayerState)->Exit {
    events(a,b)
}
