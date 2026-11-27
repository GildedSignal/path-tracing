pub mod basis;
pub mod mining;
include!(concat!(env!("OUT_DIR"),"/law_hash.rs"));
pub mod codec;
pub mod footprint;
pub mod pattern;
mod attributes;
pub use attributes::{SpeedModifier, SpeedOperation};
mod effects;
pub use effects::{MobEffectInstance, EffectKind};
pub mod core;
pub mod face;

pub mod math;
pub mod shapes;
pub mod breath;
pub mod mounts;
mod state;
mod world;
pub use core::{
    can_fall,
    ServerResourceEvent,
    apply_grounded_step,
    apply_server_resource_event,
    server_freeze_step,
    calculate_fall_damage,
    calculate_fall_power,
    tick,
    try_tick,
    Refusal,
    tick_with_collision,
    try_tick_with_collision,
    tick_with_contact,
    tick_server_resources
};
pub use shapes::{
    face_comparisons,
    Aabb,
    Axis,
    Vec3,
    VoxelShape
};
pub use state::{
    HurtCooldown,
    Input,
    PlayerState,
    Profile,
    Shared,
    // A state encoding refused by `PlayerState::from_bits`; `Refusal` is the law refusing an unported rule.
    Refusal as BitsRefusal,
    Pose
};
pub use world::{
    block_collision_shape,
    block_pos,
    Block,
    BlockPos,
    BoxWorld,
    CollisionContext,
    ContactDamage,
    Fluid,
    FluidFlowBlock,
    FluidKind,
    GroundedContact,
    Inside,
    World
};

pub mod combat;
pub mod entity_contact;
pub mod explosion;

pub mod falling;
pub mod fluid;
pub mod neighbor;
pub mod piston;
pub mod state_reads;
pub mod ticks;

pub mod boat;
mod dismount;
pub mod elytra;
pub mod interact;
pub mod pearl;
pub mod place;
pub mod player;
pub use player::death::{below_world, respawn, FELL_OUT_OF_WORLD};
pub mod portal;
pub mod world_hooks;

pub mod block_hooks;
pub mod items;
pub mod minecart;
pub mod projectiles;
pub mod redstone;

pub mod solve_meter;
pub mod sim;
