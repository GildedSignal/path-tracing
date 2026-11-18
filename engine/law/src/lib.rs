pub mod contact;
pub mod mining;
include!(concat!(env!("OUT_DIR"),"/law_hash.rs"));
pub mod codec;
pub mod footprint;
pub mod pattern;
mod attributes;
pub use attributes::{SpeedModifier, SpeedOperation};
mod movement;
pub mod blocks;
pub mod math;
pub mod shapes;
mod state;
mod world;
pub use movement::{
    can_fall,
    ServerResourceEvent,
    apply_grounded_step,
    apply_server_resource_event,
    calculate_fall_damage,
    calculate_fall_power,
    tick,
    try_tick,
    Refusal,
    tick_with_collision,
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
    // A state encoding refused by `PlayerState::from_bits`; `Refusal` is the law refusing an unported rule.
    Refusal as BitsRefusal,
    Pose
};
pub use world::{
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

pub mod entity_contact;
pub mod state_reads;

pub mod boat;
pub mod elytra;
pub mod pearl;

#[cfg(feature = "count-state-copies")]
pub use state::player_state_copies;
