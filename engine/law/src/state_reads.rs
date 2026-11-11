//! The state fields the law reads before it writes them: the law reports each read and
//! write to its world (`World::read_state`, `World::write_state`), and a recording world
//! keeps the reads. The reports observe places and never substitute values. A partial
//! write keeps the whole field as read.
use crate::{PlayerState, Input, Vec3, Pose, HurtCooldown, SpeedModifier};

macro_rules! state_fields {
    ($($field:ident),* $(,)?) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(u8)]
        pub enum StateField { $($field,)* }

        impl StateField {
            pub const ALL: &'static [Self] = &[$(Self::$field,)*];
        }

        impl PlayerState {
            /// Appends the field's raw bits, floats included.
            pub fn push_field_bits(&self, field: StateField, out: &mut Vec<u64>) {
                // Exhaustive destructuring makes a new state field a compile error
                // until the observer and its encoding have been considered.
                let PlayerState { $($field: _,)* } = self;
                match field { $(StateField::$field => self.$field.encode(out),)* }
            }
        }
    };
}
state_fields!(
    t,
    input,
    position,
    velocity,
    yaw,
    pitch,
    on_ground,
    horizontal_collision,
    vertical_collision,
    vertical_collision_below,
    minor_horizontal_collision,
    sprinting,
    sprint_speed_modifier,
    no_physics,
    shift_key_down,
    pose,
    fall_distance,
    in_water,
    in_lava,
    on_climbable,
    health,
    hurt_cooldown,
    food_level,
    saturation,
    exhaustion,
    food_tick_timer,
    remaining_fire_ticks,
    no_jump_delay,
    stuck_speed_multiplier,
    movement_this_tick,
    main_supporting_block,
    on_ground_no_blocks,
    crouching,
    swimming,
    underwater,
    eye_in_water,
    was_eye_in_water,
    water_height,
    lava_height,
    in_powder_snow,
    was_in_powder_snow,
    ticks_frozen,
    leather_boots,
    movement_speed_base,
    speed_modifiers,
    speed,
    jump_strength,
    gravity,
    step_height,
    friction_modifier,
    air_drag_modifier,
    water_movement_efficiency,
    movement_efficiency,
    sneaking_speed,
    safe_fall_distance,
    fall_damage_multiplier,
    bounciness,
    fall_flying,
    riding,
    boat,
    effects,
    flying,
    using_item,
);
impl StateField {
    pub fn is_resource(self) -> bool {
        matches!(self, Self::health | Self::food_level | Self::saturation | Self::exhaustion | Self::food_tick_timer)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateReads(pub u128);
impl StateReads {
    pub fn contains(self, field: StateField) -> bool { self.0 & (1u128 << field as u8) != 0 }
    pub fn fields(self) -> impl Iterator<Item = StateField> { StateField::ALL.iter().copied().filter(move |&f| self.contains(f)) }
}
macro_rules! read_state {
    ($w:expr, $s:ident.$f:ident) => { (*{ $w.read_state(crate::state_reads::StateField::$f); &$s.$f }) };
}
macro_rules! write_state {
    ($w:expr, $s:ident.$f:ident) => { (*{ $w.write_state(crate::state_reads::StateField::$f); &mut $s.$f }) };
}
macro_rules! modify_state {
    ($w:expr, $s:ident.$f:ident) => { (*{ $w.read_state(crate::state_reads::StateField::$f); &mut $s.$f }) };
}
pub(crate) use {read_state, write_state, modify_state};

trait Encode { fn encode(&self, out: &mut Vec<u64>); }
macro_rules! encode_as { ($($t:ty: |$v:ident| $e:expr),* $(,)?) => { $(impl Encode for $t {
    fn encode(&self, out: &mut Vec<u64>) { let $v = self; out.extend($e); }
})* } }
encode_as!(
    bool: |v| [*v as u64], i32: |v| [*v as u64], u32: |v| [*v as u64], u64: |v| [*v],
    f32: |v| [v.to_bits() as u64], f64: |v| [v.to_bits()], Pose: |v| [*v as u64],
    Vec3: |v| [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()],
    [i32; 3]: |v| v.map(|c| c as u64),
    HurtCooldown: |v| [v.remaining_ticks as u64, v.last_raw_hit.to_bits() as u64],
    Input: |v| [v.forward as u64, v.back as u64, v.left as u64, v.right as u64, v.jump as u64,
        v.sneak as u64, v.sprint as u64, v.yaw.to_bits() as u64, v.pitch.to_bits() as u64],
);
impl<T: Encode> Encode for Option<T> {
    fn encode(&self, out: &mut Vec<u64>) {
        out.push(self.is_some() as u64);
        if let Some(value) = self { value.encode(out); }
    }
}
impl Encode for Vec<SpeedModifier> {
    fn encode(&self, out: &mut Vec<u64>) {
        out.push(self.len() as u64);
        for m in self {
            out.push(m.id.len() as u64);
            out.extend(m.id.bytes().map(u64::from));
            out.extend([m.amount.to_bits(), m.operation as u64]);
        }
    }
}
