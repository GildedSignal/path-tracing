//! Four dimensions: the world the planner reads takes a tick (learning.md, "Four
//! dimensions"). A static world answers every tick alike; a scheduled one is piecewise
//! constant, and each change is a fact at a known tick.
use std::cell::Cell;
use law::{Block, BlockPos, BoxWorld, World};

pub trait Timeline {
    type World: World;
    /// The world as it stands during the tick a state whose counter is `tick` is about to
    /// take. It is one object: a later call moves its clock, so read before calling again.
    fn at(&self, tick: u64) -> &Self::World;
    /// The first tick after `tick` at which a cell differs. None: nothing changes again.
    fn next_change(&self, tick: u64) -> Option<u64>;
    fn changes(&self) -> Vec<u64> {
        let mut ticks = Vec::new();
        while let Some(next) = self.next_change(ticks.last().copied().unwrap_or(0)) { ticks.push(next); }
        ticks
    }
}

/// A world that never changes.
pub struct Still<'a, W>(pub &'a W);
impl<W: World> Timeline for Still<'_, W> {
    type World = W;
    fn at(&self, _: u64) -> &W { self.0 }
    fn next_change(&self, _: u64) -> Option<u64> { None }
}

/// A world on a schedule: each frame stands from its tick until the next frame's.
pub struct Scheduled { clock: Cell<u64>, frames: Vec<(u64, BoxWorld)> }
impl Scheduled {
    pub fn new(first: BoxWorld) -> Self { Self { clock: Cell::new(0), frames: vec![(0, first)] } }
    pub fn then(mut self, from: u64, world: BoxWorld) -> Self {
        assert!(from > self.frames.last().unwrap().0, "frames in order");
        self.frames.push((from, world));
        self
    }
    /// Each frame with the ticks it stands over: from, and until.
    pub fn frames(&self) -> impl Iterator<Item = ((u64, Option<u64>), &BoxWorld)> {
        self.frames.iter().enumerate().map(|(i, frame)| ((frame.0, self.frames.get(i + 1).map(|next| next.0)), &frame.1))
    }
    fn frame(&self) -> &BoxWorld { &self.frames.iter().rev().find(|frame| frame.0 <= self.clock.get()).unwrap().1 }
}
impl World for Scheduled {
    fn block(&self, p: BlockPos) -> &Block { self.frame().block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.frame().loaded(p) }
    fn min_y(&self) -> i32 { self.frame().min_y() }
    fn fast_lava(&self) -> bool { self.frame().fast_lava() }
    fn difficulty(&self) -> u8 { self.frame().difficulty() }
    fn natural_regeneration(&self) -> bool { self.frame().natural_regeneration() }
}
impl Timeline for Scheduled {
    type World = Scheduled;
    fn at(&self, tick: u64) -> &Scheduled { self.clock.set(tick); self }
    fn next_change(&self, tick: u64) -> Option<u64> { self.frames.iter().map(|frame| frame.0).find(|&from| from > tick) }
}

/// Free space: every cell is `block`, the real world's own free cell; the world-wide values
/// are the real world's. The law run here is the generator of a free arc.
pub struct Free<'a, W> { pub real: &'a W, pub block: &'a Block }
impl<W: World> World for Free<'_, W> {
    fn block(&self, _: BlockPos) -> &Block { self.block }
    fn loaded(&self, _: BlockPos) -> bool { true }
    fn min_y(&self) -> i32 { self.real.min_y() }
    fn fast_lava(&self) -> bool { self.real.fast_lava() }
    fn difficulty(&self) -> u8 { self.real.difficulty() }
    fn natural_regeneration(&self) -> bool { self.real.natural_regeneration() }
}
