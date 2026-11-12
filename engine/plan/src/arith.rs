//! Arithmetic lanes (job 3.2b step 4): what the search may take without a replay in the world.
//!
//! **The free arc** (lab/wiki/arc-certificate.md: a collision-free arc's increments are bit for
//! bit the same wherever it flies). The arc's ticks come from the law itself, run in free
//! space; the tube of each tick is then checked against the world. The check asks only what
//! the law asks of the cells a flying body meets, each named by its line in `engine/law/src`:
//! nothing to collide with in the box the step sweeps (movement.rs `collide`), and in every
//! cell that box touches, down to the half block under the feet (`on_pos`, movement.rs:316),
//! no fluid (`update_fluid`), no inside effect (`inside_block`), nothing to climb
//! (`climbable`), no speed factor (`speed_factor`), nothing that resets a fall
//! (world.rs `fall_reset_clip`). A tick that fails the check is not taken: the arc ends on
//! the tick before, and the law decides the contact by a replay.
use law::{Block, BlockPos, CollisionContext, FluidKind, Inside, PlayerState, Pose, Vec3, World};
use words::{Arguments, Program};
use crate::{Free, goal::hull};

/// A cell that answers a flying body as free space does.
fn plain(block: &Block, free: &Block) -> bool {
    std::ptr::eq(block, free) || block.fluid.kind == FluidKind::Empty && block.inside == Inside::None && block.stuck_multiplier.is_none()
        && !block.climbable && block.open_trapdoor_facing.is_none() && !block.water_block && !block.world_hook
        && block.speed_factor.to_bits() == free.speed_factor.to_bits() && !block.resets_fall_distance && !block.end_portal_or_gateway
}
/// A body the free arc's premises hold for: in the air, upright, in no medium.
pub fn flying(state: &PlayerState) -> bool {
    !state.on_ground && !state.in_water && !state.in_lava && !state.on_climbable && !state.swimming && !state.in_powder_snow
        && state.pose == Pose::Standing && state.stuck_speed_multiplier.length_sqr() == 0.0 && !state.no_physics && state.health > 0.0
}
/// One tick of a free arc from `state`, or None where the tube meets the world.
pub fn free_tick<W: World>(real: &W, free: &Block, state: &PlayerState, program: &Program, args: Arguments, elapsed: u32,
    readable: &dyn Fn(BlockPos) -> bool) -> Option<PlayerState> {
    if !flying(state) { return None; }
    let space = Free { real, block: free };
    let (_, input) = program.select(&space, state, elapsed, args)?;
    let mut next = state.clone();
    law::try_tick(&mut next, &space, &input).ok()?;
    let wanted = next.movement_this_tick?;
    let before = state.bounding_box();
    let swept = hull(hull(before, before.moved(wanted)), next.bounding_box());
    let (lo, hi) = (law::math::floor, law::math::floor);
    let (min, max) = ([lo(swept.min.x), lo(swept.min.y - 0.500001), lo(swept.min.z)], [hi(swept.max.x), hi(swept.max.y), hi(swept.max.z)]);
    for x in min[0]..=max[0] { for y in min[1]..=max[1] { for z in min[2]..=max[2] {
        if !readable([x, y, z]) || !real.loaded([x, y, z]) || !plain(real.block([x, y, z]), free) { return None; }
    } } }
    let context = CollisionContext::of(state);
    // One cell of margin: a shape may reach out of its cell, as the law's own scan allows.
    if !readable(min.map(|v| v - 1)) || !readable(max.map(|v| v + 1)) { return None; }
    if !real.no_collision(swept, context) || !real.extra_collisions(swept, context).is_empty() { return None; }
    flying(&next).then_some(next)
}

/// The gait lane (lab/wiki/gait-modulo.md): on uniform ground the state minus position
/// recurs bit for bit, and positions follow from one cycle's increments.
///
/// `cycle` is one observed cycle: the states from one visit of the recurrence to the next.
/// The ground is uniform where every layer of cells the cycle read, widened by one cell for
/// the phase inside a block, holds one content over the corridor: `layered` asks that.
/// Positions on axes the collision did not clamp advance by the recorded increment, the same
/// addition the law makes; clamped axes keep the recorded coordinate. Resources are not
/// advanced: the food clock is a third lane nobody has proved, so the state this returns is
/// a prediction for ordering, and the sentence's replay is what certifies it.
pub struct Cycle { pub increments: Vec<Vec3>, pub clamped: Vec<[bool; 3]>, pub level: Vec<f64>, pub reach: [[i32; 2]; 3] }
impl Cycle {
    /// From the states of one cycle (first and last recur) and the cells its replay read,
    /// relative to the first state's block.
    pub fn observed(states: &[PlayerState], reads: impl Iterator<Item = BlockPos>) -> Option<Cycle> {
        let origin = words::body_block(&states[0]);
        let mut reach = [[0, 0]; 3];
        for cell in reads { for a in 0..3 { reach[a] = [reach[a][0].min(cell[a] - origin[a]), reach[a][1].max(cell[a] - origin[a])]; } }
        let (mut increments, mut clamped, mut level) = (Vec::new(), Vec::new(), Vec::new());
        for pair in states.windows(2) {
            let moved = pair[1].position.minus(pair[0].position);
            let wanted = pair[1].movement_this_tick?;
            // Only the vertical clamp of flat ground is taken: a wall would need its plane.
            if wanted.x != moved.x || wanted.z != moved.z { return None; }
            clamped.push([false, wanted.y != moved.y, false]);
            increments.push(moved);
            level.push(pair[1].position.y);
        }
        Some(Cycle { increments, clamped, level, reach })
    }
    /// Whether the corridor of one more cycle from `state` is uniform ground like the
    /// observed one: each layer one content, the content of the observed column.
    pub fn layered<W: World>(&self, world: &W, state: &PlayerState, readable: &dyn Fn(BlockPos) -> bool) -> bool {
        let origin = words::body_block(state);
        let run = self.increments.iter().fold(Vec3::default(), |sum, step| sum.plus(*step));
        let far = |d: f64| if d < 0.0 { [d.floor() as i32 - 1, 1] } else { [-1, d.ceil() as i32 + 1] };
        let (wx, wz) = (far(run.x), far(run.z));
        if run.y != 0.0 { return false; }
        for dy in self.reach[1][0]..=self.reach[1][1] {
            let y = origin[1] + dy;
            if !readable([origin[0], y, origin[2]]) { return false; }
            let model = world.block([origin[0], y, origin[2]]);
            for x in origin[0] + self.reach[0][0] + wx[0]..=origin[0] + self.reach[0][1] + wx[1] {
                for z in origin[2] + self.reach[2][0] + wz[0]..=origin[2] + self.reach[2][1] + wz[1] {
                    if !readable([x, y, z]) || !world.loaded([x, y, z]) || !std::ptr::eq(world.block([x, y, z]), model) { return false; }
                }
            }
        }
        true
    }
    /// One cycle by arithmetic. `each` sees every tick's state pair.
    pub fn advance(&self, state: &mut PlayerState, template: &[PlayerState], each: &mut dyn FnMut(&PlayerState, &PlayerState) -> bool) -> Option<u32> {
        for tick in 0..self.increments.len() {
            let before = state.clone();
            let position = state.position.plus(self.increments[tick]);
            // Every field but position, the tick counter and the resources is the cycle's.
            let (t, food, saturation, exhaustion, timer, health) = (state.t, state.food_level, state.saturation, state.exhaustion, state.food_tick_timer, state.health);
            *state = template[tick + 1].clone();
            let moved = words::body_block(&template[tick + 1]);
            state.position = Vec3::new(position.x, if self.clamped[tick][1] { self.level[tick] } else { position.y }, position.z);
            let shift = words::body_block(state);
            state.main_supporting_block = state.main_supporting_block.map(|p| [p[0] - moved[0] + shift[0], p[1] - moved[1] + shift[1], p[2] - moved[2] + shift[2]]);
            (state.t, state.food_level, state.saturation, state.exhaustion, state.food_tick_timer, state.health) = (t + 1, food, saturation, exhaustion, timer, health);
            if each(&before, state) { return Some(tick as u32 + 1); }
        }
        None
    }
}
