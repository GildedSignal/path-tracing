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
    pub no_physics: bool,
    pub shift_key_down: bool,
    pub pose: Pose,
    pub fall_distance: f64,
    pub in_water: bool,
    pub in_lava: bool,
    pub on_climbable: bool,
    pub health: f32,
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
    pub leather_boots: bool,
    pub movement_speed: f64,
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
            no_physics: false,
            shift_key_down: false,
            pose: Pose::Standing,
            fall_distance: 0.0,
            in_water: false,
            in_lava: false,
            on_climbable: false,
            health: 20.0,
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
            leather_boots: false,
            movement_speed: 0.1f32 as f64,
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
}
