pub mod mining;
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
    tick_with_collision,
    tick_with_contact,
    tick_server_resources
};
pub use shapes::{
    Aabb,
    Axis,
    Vec3,
    VoxelShape
};
pub use state::{
    HurtCooldown,
    Input,
    PlayerState,
    Refusal,
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
