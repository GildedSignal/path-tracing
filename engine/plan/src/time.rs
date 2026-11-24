//! Four dimensions: the world the planner reads takes a tick (learning.md, "Four
//! dimensions"). A static world answers every tick alike; a scheduled one is piecewise
//! constant, and each change is a fact at a known tick.
use law::World;

pub trait Timeline {
    type World: World;
    /// The world as it stands during the tick a state whose counter is `tick` is about to
    /// take. It is one object: a later call moves its clock, so read before calling again.
    fn at(&self, tick: u64) -> &Self::World;
}

/// A world that never changes.
pub struct Still<'a, W>(pub &'a W);
impl<W: World> Timeline for Still<'_, W> {
    type World = W;
    fn at(&self, _: u64) -> &W { self.0 }
}
