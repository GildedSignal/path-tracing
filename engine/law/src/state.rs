use crate::{player::GameType, Aabb, SpeedModifier, SpeedOperation, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pose {
    #[default] Standing,
    Crouching,
    Swimming,
    FallFlying,
}
impl Pose {
    // Avatar.POSES
    pub fn height(self) -> f32 {
        match self {
            Self::Standing => 1.8,
            Self::Crouching => 1.5,
            Self::Swimming | Self::FallFlying => 0.6
        }
    }

    // Avatar.POSES
    pub fn eye_height(self) -> f32 {
        match self {
            Self::Standing => 1.62,
            Self::Crouching => 1.27,
            Self::Swimming | Self::FallFlying => 0.4
        }
    }
}

/// Server invulnerableTime and lastHurt, independent of the hurt animation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HurtCooldown {
    pub remaining_ticks: i32,
    pub last_raw_hit: f32,
}

/// The one declaration of the player's state: each field, its type and its value at
/// spawn (Entity.<init>; LivingEntity.createLivingAttributes; Player.createAttributes;
/// FoodData.<init>). The struct, its default, the footprint's field names and each
/// field's raw bits come from this list. The hot part is what a tick reads and writes;
/// the profile is what no tick writes (attributes, gear, game mode, observed effects),
/// shared by every copy of the state until one of them changes it.
macro_rules! player_state {
    (hot { $($(#[$meta:meta])* $field:ident: $ty:ty = $default:expr,)* }
     profile { $($(#[$pmeta:meta])* $pfield:ident: $pty:ty = $pdefault:expr,)* }) => {
        #[derive(Clone, Debug)]
        pub struct PlayerState { $($(#[$meta])* pub $field: $ty,)* pub profile: Shared<Profile> }
        #[derive(Clone, Debug)]
        pub struct Profile { $($(#[$pmeta])* pub $pfield: $pty,)* }
        impl Default for PlayerState {
            fn default() -> Self { Self { $($field: $default,)* profile: Shared::default() } }
        }
        impl Default for Profile {
            fn default() -> Self { Self { $($pfield: $pdefault,)* } }
        }
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(u8)]
        pub enum StateField { $($field,)* $($pfield,)* }
        impl StateField { pub const ALL: &'static [Self] = &[$(Self::$field,)* $(Self::$pfield,)*]; }
        impl PlayerState {
            /// Appends the field's raw bits, floats included.
            pub fn push_field_bits(&self, field: StateField, out: &mut Vec<u64>) {
                match field {
                    $(StateField::$field => Wire::put(&self.$field, out),)*
                    $(StateField::$pfield => Wire::put(&self.profile.$pfield, out),)*
                }
            }
        }
    };
}

/// A value shared by the copies of a state: reading is a pointer away, and writing
/// copies it first if another state still holds it.
#[derive(Clone, Debug, Default)]
pub struct Shared<T>(std::sync::Arc<T>);
impl<T> std::ops::Deref for Shared<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T { &self.0 }
}
impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self { Self(std::sync::Arc::new(value)) }
}
impl<T: Clone> std::ops::DerefMut for Shared<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T { std::sync::Arc::make_mut(&mut self.0) }
}
player_state! {
    hot {
        t: u64 = 0,
        input: Input = Input::default(),
        position: Vec3 = Vec3::default(),
        velocity: Vec3 = Vec3::default(),
        yaw: f32 = 0.0,
        pitch: f32 = 0.0,
        on_ground: bool = false,
        horizontal_collision: bool = false,
        vertical_collision: bool = false,
        vertical_collision_below: bool = false,
        minor_horizontal_collision: bool = false,
        sprinting: bool = false,
        /// MOVEMENT_SPEED modifier presence; entity-data packets update only `sprinting`.
        sprint_speed_modifier: bool = false,
        shift_key_down: bool = false,
        pose: Pose = Pose::Standing,
        fall_distance: f64 = 0.0,
        in_water: bool = false,
        in_lava: bool = false,
        on_climbable: bool = false,
        health: f32 = 20.0,
        hurt_cooldown: HurtCooldown = HurtCooldown::default(),
        food_level: i32 = 20,
        saturation: f32 = 5.0,
        exhaustion: f32 = 0.0,
        food_tick_timer: u32 = 0,
        remaining_fire_ticks: i32 = 0,
        no_jump_delay: i32 = 0,
        stuck_speed_multiplier: Vec3 = Vec3::default(),
        movement_this_tick: Option<Vec3> = None,
        main_supporting_block: Option<[i32; 3]> = None,
        on_ground_no_blocks: bool = false,
        crouching: bool = false,
        swimming: bool = false,
        underwater: bool = false,
        eye_in_water: bool = false,
        was_eye_in_water: bool = false,
        water_height: f64 = 0.0,
        lava_height: f64 = 0.0,
        in_powder_snow: bool = false,
        was_in_powder_snow: bool = false,
        ticks_frozen: i32 = 0,
        /// Observed Player.getSpeed(), refreshed from the attribute before travel.
        speed: f32 = 0.1,
        fall_flying: bool = false,
        effects: bool = false,
        flying: bool = false,
        jump_trigger_time: i32 = 0,
        sprint_trigger_time: i32 = 0,
        sprint_window: i32 = 0,
        absorption: f32 = 0.0,
        /// The server's effect book (effects.rs) and the MAX_HEALTH it sets.
        effect_book: Vec<crate::MobEffectInstance> = Vec::new(),
        max_health: f32 = 20.0,
    }
    profile {
        no_physics: bool = false,
        leather_boots: bool = false,
        movement_speed_base: f64 = 0.1f32 as f64,
        speed_modifiers: Vec<SpeedModifier> = Vec::new(),
        jump_strength: f64 = 0.42f32 as f64,
        gravity: f64 = 0.08,
        step_height: f32 = 0.6,
        friction_modifier: f32 = 1.0,
        air_drag_modifier: f32 = 1.0,
        water_movement_efficiency: f32 = 0.0,
        movement_efficiency: f32 = 0.0,
        sneaking_speed: f32 = 0.3,
        safe_fall_distance: f64 = 3.0,
        fall_damage_multiplier: f64 = 1.0,
        bounciness: f64 = 0.0,
        riding: bool = false,
        boat: bool = false,
        using_item: bool = false,
        use_item_speed_multiplier: Option<f32> = None,
        use_item_can_sprint: Option<bool> = None,
        game_mode: GameType = GameType::Survival,
        mayfly: bool = false,
        instabuild: bool = false,
        invulnerable: bool = false,
        may_build: bool = true,
        flying_speed: f32 = 0.05f32,
        jump_boost: Option<u8> = None,
        speed_effect: Option<u8> = None,
        slowness_effect: Option<u8> = None,
        blindness: bool = false,
        fire_resistance: bool = false,
        unmodeled_effects: bool = false,
        levitation: Option<u8> = None,
        slow_falling: bool = false,
        dolphins_grace: bool = false,
        weaving: bool = false,
        resistance: Option<u8> = None,
        armor: f32 = 0.0,
        armor_toughness: f32 = 0.0,
        protection_epf: f32 = 0.0,
        feather_falling_epf: f32 = 0.0,
        soul_speed_active: bool = false,
    }
}

/// The wire order of `bits` (the bot's mod speaks it over its channel, Recorder.java):
/// a head, then tails sent only while they differ from the spawn state, so an older
/// encoding without them still decodes.
macro_rules! wire {
    ($([$($($path:ident).+),* $(,)?])*) => {
        const SECTIONS: usize = [$(stringify!($($($path).+)*)),*].len();
        impl PlayerState {
            fn put_wire(&self, out: &mut Vec<u64>, ends: &mut [usize; SECTIONS]) {
                let mut section = 0;
                $($(Wire::put(&self.$($path).+, out);)* ends[section] = out.len(); section += 1;)*
                let _ = section;
            }
            fn take_wire(&mut self, b: &mut Bits<'_>) -> Result<(), Refusal> {
                let mut section = 0;
                $(if section == 0 || b.at < b.words.len() { $(self.$($path).+ = Wire::take(b)?;)* } section += 1;)*
                let _ = section;
                Ok(())
            }
        }
    };
}
wire! {
    [t, input.yaw, input.pitch, yaw, pitch, pose, hurt_cooldown.remaining_ticks, hurt_cooldown.last_raw_hit,
     food_level, food_tick_timer, remaining_fire_ticks, no_jump_delay, ticks_frozen,
     input.forward, input.back, input.left, input.right, input.jump, input.sneak, input.sprint, on_ground,
     horizontal_collision, vertical_collision, vertical_collision_below, minor_horizontal_collision,
     sprinting, sprint_speed_modifier, profile.no_physics, shift_key_down, in_water, in_lava, on_climbable,
     on_ground_no_blocks, crouching, swimming, underwater, eye_in_water, was_eye_in_water, in_powder_snow,
     was_in_powder_snow, profile.leather_boots, fall_flying, profile.riding, profile.boat, effects, flying, profile.using_item,
     health, saturation, exhaustion, speed, profile.step_height, profile.friction_modifier, profile.air_drag_modifier,
     profile.water_movement_efficiency, profile.movement_efficiency, profile.sneaking_speed, profile.use_item_speed_multiplier, profile.use_item_can_sprint,
     fall_distance, water_height, lava_height, profile.movement_speed_base, profile.jump_strength, profile.gravity, profile.safe_fall_distance,
     profile.fall_damage_multiplier, profile.bounciness, position, velocity, stuck_speed_multiplier, movement_this_tick,
     main_supporting_block, profile.speed_modifiers]
    [profile.game_mode, profile.mayfly, profile.instabuild, profile.invulnerable, profile.may_build, profile.flying_speed, jump_trigger_time, sprint_trigger_time,
     sprint_window, profile.jump_boost, profile.speed_effect, profile.slowness_effect, profile.blindness, profile.fire_resistance, profile.unmodeled_effects]
    [profile.levitation, profile.slow_falling, profile.dolphins_grace, profile.weaving, profile.resistance, profile.armor, profile.armor_toughness, profile.protection_epf,
     profile.feather_falling_epf, absorption, profile.soul_speed_active]
    [max_health, effect_book]
}

impl PlayerState {
    // Entity.makeBoundingBox
    pub fn bounding_box(&self) -> Aabb {
        Aabb::player(self.position, self.pose.height())
    }

    /// Every field as raw bits, floats included: two states are the same exactly
    /// when these are.
    pub fn bits(&self) -> Vec<u64> {
        static SPAWN: std::sync::OnceLock<(Vec<u64>, [usize; SECTIONS])> = std::sync::OnceLock::new();
        let (spawn, spawn_ends) = SPAWN.get_or_init(|| {
            let (mut out, mut ends) = (Vec::new(), [0; SECTIONS]);
            PlayerState::default().put_wire(&mut out, &mut ends);
            (out, ends)
        });
        let (mut out, mut ends) = (Vec::with_capacity(spawn.len() + 8), [0; SECTIONS]);
        self.put_wire(&mut out, &mut ends);
        // A tail is kept while it, or one after it, differs from the spawn state's.
        let keep = (1..SECTIONS).rev()
            .find(|&i| out[ends[i - 1]..ends[i]] != spawn[spawn_ends[i - 1]..spawn_ends[i]])
            .unwrap_or(0);
        out.truncate(ends[keep]);
        out
    }

    /// Decode an observation only when every encoded word is known.  The mask
    /// follows the variable-length `bits` schema, including option tags and
    /// modifier bytes; unknown values cannot be replaced with defaults.
    pub fn from_observed_bits(words: &[u64], known: &[bool]) -> Result<Self, Refusal> {
        if known.len() != words.len() {
            return Err(Refusal { position: known.len().min(words.len()), reason: "known mask length" });
        }
        if let Some(position) = known.iter().position(|known| !known) {
            return Err(Refusal { position, reason: "unknown word" });
        }
        Self::from_bits(words)
    }

    /// Decode exactly the representation returned by `bits`.
    pub fn from_bits(words: &[u64]) -> Result<Self, Refusal> {
        let mut b = Bits { words, at: 0 };
        let mut s = Self::default();
        s.take_wire(&mut b)?;
        if b.at != words.len() { return Err(Refusal { position: b.at, reason: "extra word" }); }
        Ok(s)
    }
}

/// Invalid state encoding and the position of the offending or missing word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal { pub position: usize, pub reason: &'static str }

pub(crate) struct Bits<'a> { pub(crate) words: &'a [u64], pub(crate) at: usize }
impl Bits<'_> {
    pub(crate) fn word(&mut self) -> Result<u64, Refusal> {
        let at = self.at;
        let value = *self.words.get(at).ok_or(Refusal { position: at, reason: "missing word" })?;
        self.at += 1; Ok(value)
    }
    /// The next word as one of `n` codes.
    pub(crate) fn code(&mut self, n: u64, reason: &'static str) -> Result<u64, Refusal> {
        let at = self.at;
        let value = self.word()?;
        if value < n { Ok(value) } else { Err(Refusal { position: at, reason }) }
    }
}

/// One value as words, both ways.
pub(crate) trait Wire: Sized {
    fn put(&self, out: &mut Vec<u64>);
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal>;
}
impl Wire for u64 {
    fn put(&self, out: &mut Vec<u64>) { out.push(*self) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { b.word() }
}
impl Wire for bool {
    fn put(&self, out: &mut Vec<u64>) { out.push(*self as u64) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(b.code(2, "boolean")? == 1) }
}
impl Wire for f32 {
    fn put(&self, out: &mut Vec<u64>) { out.push(self.to_bits() as u64) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(f32::from_bits(b.code(1 << 32, "f32")? as u32)) }
}
impl Wire for f64 {
    fn put(&self, out: &mut Vec<u64>) { out.push(self.to_bits()) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(f64::from_bits(b.word()?)) }
}
impl Wire for i32 {
    fn put(&self, out: &mut Vec<u64>) { out.push(*self as u64) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> {
        let at = b.at; let n = b.word()?; let v = n as i32;
        if v as u64 == n { Ok(v) } else { Err(Refusal { position: at, reason: "i32" }) }
    }
}
impl Wire for u32 {
    fn put(&self, out: &mut Vec<u64>) { out.push(*self as u64) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(b.code(1 << 32, "u32")? as u32) }
}
impl Wire for Pose {
    fn put(&self, out: &mut Vec<u64>) { out.push(*self as u64) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> {
        Ok([Pose::Standing, Pose::Crouching, Pose::Swimming, Pose::FallFlying][b.code(4, "pose")? as usize])
    }
}
impl Wire for GameType {
    fn put(&self, out: &mut Vec<u64>) { out.push(*self as u64) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(GameType::ALL[b.code(4, "game mode")? as usize]) }
}
impl Wire for Vec3 {
    fn put(&self, out: &mut Vec<u64>) { out.extend([self.x.to_bits(), self.y.to_bits(), self.z.to_bits()]) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(Vec3::new(f64::take(b)?, f64::take(b)?, f64::take(b)?)) }
}
impl Wire for [i32; 3] {
    fn put(&self, out: &mut Vec<u64>) { out.extend(self.map(|c| c as u64)) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok([i32::take(b)?, i32::take(b)?, i32::take(b)?]) }
}
impl Wire for HurtCooldown {
    fn put(&self, out: &mut Vec<u64>) { self.remaining_ticks.put(out); self.last_raw_hit.put(out); }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(Self { remaining_ticks: i32::take(b)?, last_raw_hit: f32::take(b)? }) }
}
impl Wire for Input {
    fn put(&self, out: &mut Vec<u64>) {
        for key in [self.forward, self.back, self.left, self.right, self.jump, self.sneak, self.sprint] { key.put(out); }
        self.yaw.put(out); self.pitch.put(out);
    }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> {
        Ok(Self { forward: Wire::take(b)?, back: Wire::take(b)?, left: Wire::take(b)?, right: Wire::take(b)?,
            jump: Wire::take(b)?, sneak: Wire::take(b)?, sprint: Wire::take(b)?, yaw: Wire::take(b)?, pitch: Wire::take(b)? })
    }
}
/// An effect amplifier: u64::MAX when the effect is absent.
impl Wire for Option<u8> {
    fn put(&self, out: &mut Vec<u64>) { out.push(self.map_or(u64::MAX, u64::from)) }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> {
        let at = b.at;
        match b.word()? { u64::MAX => Ok(None), w => u8::try_from(w).map(Some).map_err(|_| Refusal { position: at, reason: "effect amplifier" }) }
    }
}
macro_rules! tagged { ($($t:ty),*) => { $(impl Wire for Option<$t> {
    fn put(&self, out: &mut Vec<u64>) { self.is_some().put(out); if let Some(v) = self { v.put(out); } }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> { Ok(if bool::take(b)? { Some(<$t>::take(b)?) } else { None }) }
})* } }
tagged!(f32, bool, Vec3, [i32; 3]);
impl Wire for Vec<SpeedModifier> {
    fn put(&self, out: &mut Vec<u64>) {
        out.push(self.len() as u64);
        for m in self {
            out.push(m.id.len() as u64);
            out.extend(m.id.bytes().map(u64::from));
            out.extend([m.amount.to_bits(), m.operation as u64]);
        }
    }
    fn take(b: &mut Bits<'_>) -> Result<Self, Refusal> {
        let at = b.at; let count = b.word()?;
        if count > (b.words.len() - b.at) as u64 / 3 { return Err(Refusal { position: at, reason: "modifier count" }); }
        let mut modifiers = Vec::new();
        for _ in 0..count {
            let at = b.at; let len = b.word()?;
            if len > (b.words.len() - b.at) as u64 { return Err(Refusal { position: at, reason: "id length" }); }
            let start = b.at;
            let bytes = (0..len).map(|_| b.code(256, "id byte").map(|n| n as u8)).collect::<Result<Vec<_>, _>>()?;
            let id = String::from_utf8(bytes).map_err(|e| Refusal { position: start + e.utf8_error().valid_up_to(), reason: "id utf8" })?;
            let amount = f64::take(b)?;
            let operation = [SpeedOperation::AddValue, SpeedOperation::AddMultipliedBase, SpeedOperation::AddMultipliedTotal]
                [b.code(3, "operation")? as usize];
            modifiers.push(SpeedModifier { id, amount, operation });
        }
        Ok(modifiers)
    }
}
