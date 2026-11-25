//! Floor height and pose clearance for dismounting.
use crate::{Aabb, BlockPos, CollisionContext, Pose, Vec3, World};

/// `BlockGetter.getBlockFloorHeight` (`@0-44`).
pub fn block_floor_height(world: &impl World, pos: BlockPos) -> f64 {
    let ctx = CollisionContext::empty(f64::NEG_INFINITY);
    let shape = world.collision_shape(pos, ctx);
    if !shape.is_empty() {
        return shape.view().max_y();
    }
    let below_shape = world.collision_shape([pos[0], pos[1] - 1, pos[2]], ctx);
    let below_max = below_shape.view().max_y();
    if below_max >= 1.0 {
        below_max - 1.0
    } else {
        f64::NEG_INFINITY
    }
}

/// `DismountHelper.isBlockFloorValid` (`@0-18`).
pub fn is_block_floor_valid(floor_height: f64) -> bool {
    !floor_height.is_infinite() && floor_height < 1.0
}

pub(crate) fn clear_pose(
    world: &impl World,
    width: f32,
    horizontal: [f64; 2],
    floors: [BlockPos; 2],
) -> Option<(Vec3, Pose)> {
    let candidates = floors.map(|pos| {
        let h = block_floor_height(world, pos);
        is_block_floor_valid(h).then(|| Vec3::new(horizontal[0], pos[1] as f64 + h, horizontal[1]))
    });
    let half = (width / 2.0f32) as f64;
    for pose in [Pose::Standing, Pose::Crouching, Pose::Swimming] {
        let local = Aabb::new(-half, 0.0, -half, half, pose.height() as f64, half);
        for &pos in candidates.iter().flatten() {
            if world.no_collision(local.moved(pos), CollisionContext::empty(pos.y)) {
                return Some((pos, pose));
            }
        }
    }
    None
}
