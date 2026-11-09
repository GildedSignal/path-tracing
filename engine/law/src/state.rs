use crate::{
    Aabb,
    Vec3
};

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
    Swimming
}
impl Pose {
    // Avatar.POSES
    pub fn height(self) -> f32 {
        match self {
            Self::Standing => 1.8,
            Self::Crouching => 1.5,
            Self::Swimming => 0.6
        }
    }

    // Avatar.POSES
    pub fn eye_height(self) -> f32 {
        match self {
            Self::Standing => 1.62,
            Self::Crouching => 1.27,
            Self::Swimming => 0.4
        }
    }
}

/// Server invulnerableTime and lastHurt, independent of the hurt animation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HurtCooldown {
    pub remaining_ticks: i32,
    pub last_raw_hit: f32,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub t: u64,
    pub input: Input,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    pub minor_horizontal_collision: bool,
    pub sprinting: bool,
    /// MOVEMENT_SPEED modifier presence; entity-data packets update only `sprinting`.
    pub sprint_speed_modifier: bool,
    pub no_physics: bool,
    pub shift_key_down: bool,
    pub pose: Pose,
    pub fall_distance: f64,
    pub in_water: bool,
    pub in_lava: bool,
    pub on_climbable: bool,
    pub health: f32,
    pub hurt_cooldown: HurtCooldown,
    pub food_level: i32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub food_tick_timer: u32,
    pub remaining_fire_ticks: i32,
    pub no_jump_delay: i32,
    pub stuck_speed_multiplier: Vec3,
    pub movement_this_tick: Option<Vec3>,
    pub main_supporting_block: Option<[i32; 3]>,
    pub on_ground_no_blocks: bool,
    pub crouching: bool,
    pub swimming: bool,
    pub underwater: bool,
    pub eye_in_water: bool,
    pub was_eye_in_water: bool,
    pub water_height: f64,
    pub lava_height: f64,
    pub in_powder_snow: bool,
    pub was_in_powder_snow: bool,
    pub ticks_frozen: i32,
    pub leather_boots: bool,
    pub movement_speed_base: f64,
    pub speed_modifiers: Vec<crate::SpeedModifier>,
    /// Observed Player.getSpeed(), refreshed from the attribute before travel.
    pub speed: f32,
    pub jump_strength: f64,
    pub gravity: f64,
    pub step_height: f32,
    pub friction_modifier: f32,
    pub air_drag_modifier: f32,
    pub water_movement_efficiency: f32,
    pub movement_efficiency: f32,
    pub sneaking_speed: f32,
    pub safe_fall_distance: f64,
    pub fall_damage_multiplier: f64,
    pub bounciness: f64,
    pub fall_flying: bool,
    pub riding: bool,
    pub boat: bool,
    pub effects: bool,
    pub flying: bool,
    pub using_item: bool,
}
impl Default for PlayerState {
    // Entity.<init>; LivingEntity.createLivingAttributes; Player.createAttributes; FoodData.<init>
    fn default() -> Self {
        Self {
            t: 0,
            input: Input::default(),
            position: Vec3::default(),
            velocity: Vec3::default(),
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            minor_horizontal_collision: false,
            sprinting: false,
            sprint_speed_modifier: false,
            no_physics: false,
            shift_key_down: false,
            pose: Pose::Standing,
            fall_distance: 0.0,
            in_water: false,
            in_lava: false,
            on_climbable: false,
            health: 20.0,
            hurt_cooldown: HurtCooldown::default(),
            food_level: 20,
            saturation: 5.0,
            exhaustion: 0.0,
            food_tick_timer: 0,
            remaining_fire_ticks: 0,
            no_jump_delay: 0,
            stuck_speed_multiplier: Vec3::default(),
            movement_this_tick: None,
            main_supporting_block: None,
            on_ground_no_blocks: false,
            crouching: false,
            swimming: false,
            underwater: false,
            eye_in_water: false,
            was_eye_in_water: false,
            water_height: 0.0,
            lava_height: 0.0,
            in_powder_snow: false,
            was_in_powder_snow: false,
            ticks_frozen: 0,
            leather_boots: false,
            movement_speed_base: 0.1f32 as f64,
            speed_modifiers: Vec::new(),
            speed: 0.1,
            jump_strength: 0.42f32 as f64,
            gravity: 0.08,
            step_height: 0.6,
            friction_modifier: 1.0,
            air_drag_modifier: 1.0,
            water_movement_efficiency: 0.0,
            movement_efficiency: 0.0,
            sneaking_speed: 0.3,
            safe_fall_distance: 3.0,
            fall_damage_multiplier: 1.0,
            bounciness: 0.0,
            fall_flying: false,
            riding: false,
            boat: false,
            effects: false,
            flying: false,
            using_item: false
        }
    }
}
impl PlayerState {
    // Entity.makeBoundingBox
    pub fn bounding_box(&self) -> Aabb {
        Aabb::player(self.position, self.pose.height())
    }

    /// Every field as raw bits, floats included: two states are the same exactly
    /// when these are. Exhaustive, so a new field must be added here.
    pub fn bits(&self) -> Vec<u64> {
        let PlayerState {
            t, input, position, velocity, yaw, pitch, on_ground, horizontal_collision,
            vertical_collision, vertical_collision_below, minor_horizontal_collision,
            sprinting, sprint_speed_modifier, no_physics, shift_key_down, pose, fall_distance, in_water, in_lava,
            on_climbable, health, hurt_cooldown, food_level, saturation, exhaustion,
            food_tick_timer, remaining_fire_ticks, no_jump_delay, stuck_speed_multiplier,
            movement_this_tick, main_supporting_block, on_ground_no_blocks, crouching,
            swimming, underwater, eye_in_water, was_eye_in_water, water_height, lava_height,
            in_powder_snow, was_in_powder_snow, ticks_frozen, leather_boots,
            movement_speed_base, speed_modifiers, speed, jump_strength, gravity,
            step_height, friction_modifier, air_drag_modifier, water_movement_efficiency,
            movement_efficiency, sneaking_speed, safe_fall_distance, fall_damage_multiplier,
            bounciness, fall_flying, riding, boat, effects, flying, using_item,
        } = self;
        let mut bits = vec![*t, input.yaw.to_bits() as u64, input.pitch.to_bits() as u64,
            yaw.to_bits() as u64, pitch.to_bits() as u64, *pose as u64,
            hurt_cooldown.remaining_ticks as u64, hurt_cooldown.last_raw_hit.to_bits() as u64,
            *food_level as u64, *food_tick_timer as u64, *remaining_fire_ticks as u64,
            *no_jump_delay as u64, *ticks_frozen as u64];
        for value in [input.forward, input.back, input.left, input.right, input.jump, input.sneak,
            input.sprint, *on_ground, *horizontal_collision, *vertical_collision, *vertical_collision_below,
            *minor_horizontal_collision, *sprinting, *sprint_speed_modifier, *no_physics, *shift_key_down, *in_water, *in_lava,
            *on_climbable, *on_ground_no_blocks, *crouching, *swimming, *underwater, *eye_in_water,
            *was_eye_in_water, *in_powder_snow, *was_in_powder_snow, *leather_boots, *fall_flying,
            *riding, *boat, *effects, *flying, *using_item] { bits.push(value as u64); }
        for value in [health, saturation, exhaustion, speed, step_height, friction_modifier,
            air_drag_modifier, water_movement_efficiency, movement_efficiency, sneaking_speed] {
            bits.push(value.to_bits() as u64);
        }
        for value in [fall_distance, water_height, lava_height, movement_speed_base, jump_strength,
            gravity, safe_fall_distance, fall_damage_multiplier, bounciness] { bits.push(value.to_bits()); }
        for v in [position, velocity, stuck_speed_multiplier] { bits.extend([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]); }
        bits.push(movement_this_tick.is_some() as u64);
        if let Some(v) = movement_this_tick { bits.extend([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]); }
        bits.push(main_supporting_block.is_some() as u64);
        if let Some(p) = main_supporting_block { bits.extend(p.map(|v| v as u64)); }
        bits.push(speed_modifiers.len() as u64);
        for modifier in speed_modifiers {
            bits.push(modifier.id.len() as u64); bits.extend(modifier.id.bytes().map(u64::from));
            bits.extend([modifier.amount.to_bits(), modifier.operation as u64]);
        }
        bits
    }
}
