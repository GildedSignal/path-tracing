//! The event face: state + held input → the next cause and the state there.
//!
//! A held input leaves two kinds of facts. The lanes only accumulate: the position on each
//! axis the move never clips, the clock `t`, the exhaustion the walk and the jump cost, the
//! fall of a free fall. Everything else is the state the law branches on. The core steps until
//! that state repeats with a period of P ticks (the drag's transient has no exact closed form:
//! a rounded product per tick); from there the world is the only cause left. While every cell
//! the body can read answers as its layer's first cell did, the next tick is the sample's
//! phase with the lanes advanced by the core's own additions (no world read, no tick). The jump
//! stops before the first cause: a cell that answers differently (a wall, an edge, another floor,
//! a block effect), the food clock, the world border, or the horizon.
use crate::core::{self, Effect, Refusal};
use crate::{world::block_pos, Aabb, Block, BlockPos, CollisionContext, FluidKind, Input, Inside, PlayerState, Vec3, World};

/// What ended a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// The held input's end.
    Horizon,
    /// A cell the body would read answers unlike the cells its jump was certified on.
    Cell(BlockPos),
    /// The food clock: the exhaustion would pass 4.
    Food,
    /// The world border.
    Border,
}

/// Why the core stepped the ticks it stepped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stepped {
    /// The state the law branches on has not repeated yet (the drag's transient, a short hold).
    Transient,
    /// It repeats, but a lane is not plain addition: an axis the move clips, a fall that
    /// resets, a contact request each tick.
    Lanes,
    /// It repeats, but a cell the body reads answers unlike its layer.
    Cell(BlockPos),
}

/// `ticks` ticks were advanced, `jumped` of them without the core; `stepped` says why the rest
/// were stepped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Jump { pub ticks: u64, pub jumped: u64, pub cause: Cause, pub stepped: Stepped }

const PERIOD: usize = 40;
/// Cells around the body each tick that the certificate covers (the core reads within two: the
/// sweep pads its cell range by one, and a fence's shape reaches up from the cell below).
const MARGIN: i32 = 2;

/// A tick seen: the state, its effect, and its key (x, y, z and the fall, then a hash of the
/// rest of the state its lanes left out, then that rest).
struct Seen { state: PlayerState, effect: Effect, key: Vec<u64> }

/// Holds `input` for at most `horizon` ticks, stopping at the first cause: byte-identical to
/// that many core ticks.
pub fn advance(s: &mut PlayerState, world: &impl World, input: &Input, horizon: u64) -> Result<Jump, Refusal> {
    let mut seen = std::collections::VecDeque::from([sample(s, Effect::default())]);
    let (mut ticks, mut retry, mut stepped) = (0, 0, Stepped::Transient);
    while ticks < horizon {
        let effect = core::step(s, world, input)?;
        ticks += 1;
        if seen.len() > PERIOD { seen.pop_front(); }
        seen.push_back(sample(s, effect));
        if ticks < retry { continue; }
        let last = seen.len() - 1;
        let (now, key) = (&seen[last].key, |p: usize| &seen[last - p].key);
        let Some(p) = (1..=last).find(|&p| now[4] == key(p)[4] && now[4..] == key(p)[4..]) else { continue };
        // The lane axes are the ones the cycle moved along; y carries the fall with it.
        let axes = [0, 1, 2].map(|i| now[i] != key(p)[i]);
        if !axes[1] && now[3] != key(p)[3] { continue; }
        let cycle: Vec<&Seen> = seen.range(last - p..).collect();
        match jump(s, world, &cycle, axes, horizon - ticks) {
            Ok((jumped, cause)) => return Ok(Jump { ticks: ticks + jumped, jumped, cause, stepped }),
            Err(why) => (stepped, retry) = (why, ticks + p as u64),
        }
    }
    Ok(Jump { ticks, jumped: 0, cause: Cause::Horizon, stepped })
}

/// The tick's key: every fact of the state but its lanes (t, the food ledger, the support's
/// column), with the position and the fall in front so a jump can leave its lane axes out.
fn sample(s: &PlayerState, effect: Effect) -> Seen {
    use crate::state::StateField as F;
    let mut key = vec![s.position.x.to_bits(), s.position.y.to_bits(), s.position.z.to_bits(), s.fall_distance.to_bits(), 0];
    for &field in F::ALL {
        match field {
            F::t | F::exhaustion | F::saturation | F::food_level | F::position | F::fall_distance => {}
            F::main_supporting_block => key.push(s.main_supporting_block.map_or(u64::MAX, |p| p[1] as u64)),
            _ => s.push_field_bits(field, &mut key),
        }
    }
    key[4] = key[5..].iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &w| (h ^ w).wrapping_mul(0x100_0000_01b3));
    Seen { state: s.clone(), effect, key }
}

/// What a held input only accumulates.
#[derive(Clone, Copy, PartialEq)]
struct Lanes { position: Vec3, exhaustion: f32, fall: f64, saturation: f32, food: i32 }
impl Lanes {
    fn of(s: &PlayerState) -> Self { Self { position: s.position, exhaustion: s.exhaustion, fall: s.fall_distance, saturation: s.saturation, food: s.food_level } }
    fn bits(&self) -> [u64; 7] {
        let p = self.position;
        [p.x.to_bits(), p.y.to_bits(), p.z.to_bits(), self.exhaustion.to_bits() as u64, self.fall.to_bits(), self.saturation.to_bits() as u64, self.food as u64]
    }
}

/// The lanes after one tick of the phase that ends in `after`, and whether the food clock
/// drained; None when the phase is not the lanes' own arithmetic (a lane axis the move clips,
/// a fall that shifts). An axis that is no lane, and the fall unless y is one, take the phase's.
fn lanes(after: &Seen, world: &impl World, mut l: Lanes, axes: [bool; 3]) -> Option<(Lanes, bool)> {
    let a = &after.state;
    let (wanted, moved) = after.effect.collision?;
    let from = l.position;
    for (i, axis) in [crate::Axis::X, crate::Axis::Y, crate::Axis::Z].into_iter().enumerate() {
        if !axes[i] { l.position = l.position.with(axis, a.position.get(axis)); continue; }
        if wanted.get(axis).to_bits() != moved.get(axis).to_bits() { return None; }
        if a.movement_this_tick.is_some() { l.position = l.position.with(axis, l.position.get(axis) + moved.get(axis)); }
    }
    if after.effect.jumped { l.exhaustion = core::exhausted(a, l.exhaustion, core::jump_cost(a)); }
    if let Some(cost) = core::walk_cost(a, l.position.minus(from)) { l.exhaustion = core::exhausted(a, l.exhaustion, cost); }
    let drained = l.exhaustion > 4.0;
    (l.exhaustion, l.saturation, l.food) = core::drain(world, l.exhaustion, l.saturation, l.food);
    l.fall = if axes[1] { if a.shift_key_down || a.fall_flying { return None; } core::fallen(a, l.fall, moved.y) } else { a.fall_distance };
    Some((l, drained))
}

/// Jumps along the cycle `cycle[0] → … → cycle[p]` (the last is `s`), at most `horizon` ticks.
fn jump(s: &mut PlayerState, world: &impl World, cycle: &[&Seen], axes: [bool; 3], horizon: u64) -> Result<(u64, Cause), Stepped> {
    let p = cycle.len() - 1;
    // The food lanes leave the law's branches alone only while the body is unhurt and fed above 6.
    let hungry = |a: &PlayerState, l: &Lanes| a.health > 0.0 && a.health < 20.0 || l.food <= 6;
    // The sample itself must be the lanes' arithmetic, phase by phase, bit for bit.
    for j in 1..=p {
        let (b, a) = (&cycle[j - 1].state, &cycle[j].state);
        let (l, drained) = lanes(cycle[j], world, Lanes::of(b), axes).ok_or(Stepped::Lanes)?;
        if l.bits() != Lanes::of(a).bits() || drained && hungry(a, &l) || cycle[j].effect.contact.is_some() { return Err(Stepped::Lanes); }
    }
    let mut corridor = Corridor { refs: std::collections::HashMap::new(), axes, context: CollisionContext::of(s), last: None };
    for j in 1..=p {
        corridor.admit(world, cycle[j - 1].state.bounding_box(), cycle[j].state.bounding_box()).map_err(Stepped::Cell)?;
    }
    let mut l = Lanes::of(s);
    let (mut k, mut cause) = (0, Cause::Horizon);
    while k < horizon {
        let j = (k as usize % p) + 1;
        let (b, a) = (&cycle[j - 1].state, &cycle[j].state);
        let (next, drained) = lanes(cycle[j], world, l, axes).expect("a certified phase");
        if next.position.x.abs() >= 29_999_000.0 || next.position.z.abs() >= 29_999_000.0 { cause = Cause::Border; break; }
        if drained && hungry(a, &next) { cause = Cause::Food; break; }
        let (moved, to) = (Aabb::player(next.position, b.pose.height()), Aabb::player(next.position, a.pose.height()));
        if let Err(cell) = corridor.admit(world, Aabb::player(l.position, b.pose.height()), to) { cause = Cause::Cell(cell); break; }
        world.observe_tick();
        world.observe_sweep(Aabb::player(l.position, b.pose.height()), moved);
        world.observe_sweep(moved, to);
        (l, k) = (next, k + 1);
    }
    if let (0, Cause::Cell(cell)) = (k, cause) { return Err(Stepped::Cell(cell)); }
    if k == 0 { return Ok((0, cause)); }
    let j = ((k - 1) as usize % p) + 1;
    let (b, a) = (&cycle[j - 1].state, &cycle[j]);
    let t = s.t + k;
    *s = a.state.clone();
    (s.position, s.exhaustion, s.fall_distance, s.saturation, s.food_level, s.t) = (l.position, l.exhaustion, l.fall, l.saturation, l.food, t);
    if s.main_supporting_block.is_some() {
        // The support found after the last move: the box it was found with, the flag before it.
        let pose = std::mem::replace(&mut s.pose, b.pose);
        s.on_ground_no_blocks = b.on_ground_no_blocks;
        core::support(s, world, a.effect.collision.map_or(Vec3::default(), |(_, moved)| moved));
        s.pose = pose;
    }
    Ok((k, cause))
}

/// The cells the jump is certified on: each answers as the first cell of its line along the lane
/// axes did (a layer on the ground, every cell in a fall), and no block effect counts its cells.
struct Corridor<'w> { refs: std::collections::HashMap<BlockPos, (&'w Block, bool, Option<u16>)>, axes: [bool; 3], context: CollisionContext, last: Option<(BlockPos, BlockPos)> }
impl<'w> Corridor<'w> {
    fn admit(&mut self, world: &'w impl World, from: Aabb, to: Aabb) -> Result<(), BlockPos> {
        let hull = Aabb::new(from.min.x.min(to.min.x), from.min.y.min(to.min.y), from.min.z.min(to.min.z),
            from.max.x.max(to.max.x), from.max.y.max(to.max.y), from.max.z.max(to.max.z));
        let (lo, hi) = (block_pos(hull.min).map(|c| c - MARGIN), block_pos(hull.max).map(|c| c + MARGIN));
        if self.last == Some((lo, hi)) || self.axes == [false; 3] { return Ok(()); }
        let old = self.last.replace((lo, hi));
        let bounds = Aabb::new(lo[0] as f64, lo[1] as f64, lo[2] as f64, hi[0] as f64 + 1.0, hi[1] as f64 + 1.0, hi[2] as f64 + 1.0);
        if !world.visit_extra_collisions(bounds, self.context, |_| false) { self.last = old; return Err(lo); }
        for x in lo[0]..=hi[0] { for y in lo[1]..=hi[1] { for z in lo[2]..=hi[2] {
            let p = [x, y, z];
            if old.is_some_and(|(a, b)| (0..3).all(|i| p[i] >= a[i] && p[i] <= b[i])) { continue; }
            if !self.same(world, p) { self.last = old; return Err(p); }
        } } }
        Ok(())
    }
    fn same(&mut self, world: &'w impl World, p: BlockPos) -> bool {
        let (block, loaded, model) = (world.block(p), world.loaded(p), world.model_id(p));
        let (hook, fluid) = (world.hook_block(p), world.fluid_block(p));
        let _ = world.collision_shape(p, self.context);
        if !std::ptr::eq(hook, block) || !std::ptr::eq(fluid, block) || !quiet(block) { return false; }
        let key = [0, 1, 2].map(|i| if self.axes[i] { 0 } else { p[i] });
        let Some(&(first, first_loaded, first_model)) = self.refs.get(&key) else {
            self.refs.insert(key, (block, loaded, model));
            return true;
        };
        (loaded, model) == (first_loaded, first_model) && (std::ptr::eq(first, block)
            || crate::pattern::encode_cell(first, loaded, 31).ok() == crate::pattern::encode_cell(block, loaded, 31).ok())
    }
}

/// A block whose effect on a body does not count the cells it crosses.
fn quiet(b: &Block) -> bool {
    matches!(b.inside, Inside::None | Inside::Cobweb | Inside::Extinguish) && matches!(b.fluid.kind, FluidKind::Empty | FluidKind::Water)
}
