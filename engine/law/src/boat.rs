//! Isolated occupied-boat law in established IN_WATER, without collision.
use crate::{math, Vec3};

// AbstractBoat.getPassengerAttachmentPoint 31-85; Entity.positionRider.
pub fn seat(position: Vec3, yaw: f32, ride_height: f64, offset: f32) -> Vec3 {
    let angle = (-yaw * 0.017453292f32) as f64;
    let sin = math::sin(angle) as f64;
    let cos = math::cos(angle) as f64;
    position.plus(Vec3::new(0.0 * cos + (offset as f64) * sin, ride_height, (offset as f64) * cos - 0.0 * sin))
        .minus(Vec3::new(0.0, 0.6, 0.0)) // Avatar.DEFAULT_VEHICLE_ATTACHMENT (0, 0.6, 0)
}

// Boat.rideHeight 0-8, Raft.rideHeight; AbstractBoat passenger slot 31-59.
pub fn boat_ride_height(height: f32) -> f64 { (height / 3.0f32) as f64 }
pub fn raft_ride_height(height: f32) -> f64 { (height * 0.8888889f32) as f64 }
pub fn slot_offset(first: bool, animal: bool) -> f32 {
    let mut offset = if first { 0.2f32 } else { -0.6f32 };
    if animal { offset += 0.2f32; }
    offset
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Boat { pub position: Vec3, pub velocity: Vec3, pub yaw: f32, pub delta_rotation: f32 }
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls { pub left: bool, pub right: bool, pub forward: bool, pub backward: bool }

// AbstractBoat.tick 150,164,191-199; floatBoat 138-167,270-359;
// getDefaultGravity 0-3; controlBoat 0-152. Water level is supplied by the caller.
pub fn tick(boat: &mut Boat, water_level: f64, input: Controls) {
    let draft = (water_level - boat.position.y) / (0.5625f32 as f64);
    let drag = 0.9f32;
    boat.velocity.x *= drag as f64;
    boat.velocity.y += -0.04f64;
    boat.velocity.z *= drag as f64;
    boat.delta_rotation *= drag;
    if draft > 0.0 { boat.velocity.y = (boat.velocity.y + draft * (0.04f64 / 0.65f64)) * 0.75f64; }
    if input.left { boat.delta_rotation -= 1.0f32; }
    if input.right { boat.delta_rotation += 1.0f32; }
    let mut thrust = 0.0f32;
    if input.left != input.right && !input.forward && !input.backward { thrust += 0.005f32; }
    boat.yaw += boat.delta_rotation;
    if input.forward { thrust += 0.04f32; }
    if input.backward { thrust -= 0.005f32; }
    boat.velocity.x += (math::sin((-boat.yaw * 0.017453292f32) as f64) * thrust) as f64;
    boat.velocity.z += (math::cos((boat.yaw * 0.017453292f32) as f64) * thrust) as f64;
    boat.position = boat.position.plus(boat.velocity);
}

