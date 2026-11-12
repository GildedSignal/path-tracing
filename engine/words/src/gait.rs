//! Gaits: exact recurrences (vocabulary.md, section 6). After one cycle of its program
//! every field of the state but position has the same bits, and position has advanced by
//! the cycle's displacement. This is the one place where exact starts recur, so the one
//! exact-start store, the cycle memo, lives here and keeps each cycle once.
use std::collections::{HashMap, HashSet};
use law::{CollisionContext, PlayerState, Vec3, World, footprint::RecordingWorld, pattern::{PatternTable, Start}};
use crate::{Arguments, Program, SegmentError, body_block, segment::events};

#[derive(Clone, Debug)]
pub struct Gait {
    pub program: Program,
    pub arguments: Arguments,
    /// A state on the cycle, and the ticks since its segment started.
    pub start: PlayerState,
    pub elapsed: u32,
    /// The period.
    pub ticks: u32,
    pub displacement: Vec3,
    /// The cycle's increments: the position's step on each tick.
    pub increments: Vec<Vec3>,
    /// Per tick and axis: the collision clamped the step (it differs from the step the law
    /// wanted, `movement_this_tick`). Block crossings follow from the increments by floor
    /// arithmetic once clamped ticks snap onto their plane (lab run gait-modulo).
    pub clamped: Vec<[bool; 3]>,
    /// Blocks per second, horizontal.
    pub speed: f64,
}
#[derive(Default)]
pub struct Discovery {
    pub gaits: Vec<Gait>,
    /// Programs whose cycle an earlier program already gave.
    pub repeats: Vec<usize>,
    pub refusals: Vec<(usize, SegmentError)>,
    /// Programs whose state did not recur within the budget.
    pub unresolved: Vec<usize>,
    /// The cycle memo: each cycle once, under its start's bits and the cells it read.
    memo: PatternTable<Vec<usize>>,
}
impl Discovery {
    /// The gaits whose cycle starts at this state, where its cells hold: one per program
    /// that cycles from it. Resources are guards, so the lookup is on a full stomach, as
    /// the discovery was.
    pub fn gaits_at(&mut self, world: &impl World, state: &PlayerState, elapsed: u32) -> Vec<&Gait> {
        let mut full = state.clone();
        fed(&mut full);
        let start = Start { origin: body_block(state), words: recurrence(&full, elapsed), context: CollisionContext::of(state) };
        let found: Vec<usize> = self.memo.lookup_at(world, &start).iter().flat_map(|hit| hit.entry.value.iter().copied()).collect();
        found.into_iter().map(|index| &self.gaits[index]).collect()
    }
    pub fn cycles(&self) -> usize { self.memo.entries().map(|(_, entry)| entry.value.len()).sum() }
}

/// What must recur: every field but position, with the supporting block taken from the
/// body's block and the tick counter left out, and the program's own clock.
pub fn recurrence(state: &PlayerState, elapsed: u32) -> Box<[u64]> {
    let block = body_block(state);
    let mut still = state.clone();
    still.t = 0;
    still.position = Vec3::default();
    still.main_supporting_block = still.main_supporting_block.map(|p| [0, 1, 2].map(|a| p[a] - block[a]));
    let mut words = still.bits();
    words.push(elapsed as u64);
    words.into()
}
/// The least rotation of a cycle, so that one cycle entered at two phases is one cycle.
fn orbit<T: Ord + Clone>(mut beats: Vec<T>) -> Vec<T> {
    let rotation = (0..beats.len()).min_by(|&a, &b| beats[a..].iter().chain(&beats[..a]).cmp(beats[b..].iter().chain(&beats[..b]))).unwrap();
    beats.rotate_left(rotation);
    beats
}
/// Discovery runs on a full stomach: resources are guards, not part of a gait.
fn fed(state: &mut PlayerState) {
    state.food_level = 20;
    state.saturation = 20.0;
    state.exhaustion = 0.0;
    state.food_tick_timer = 0;
}
/// One tick of the program, as a segment runs it: the clock restarts at each event.
fn step(world: &impl World, state: &mut PlayerState, program: &Program, args: Arguments, elapsed: u32) -> Option<u32> {
    let (row, input) = program.select(world, state, elapsed, args)?;
    let before = state.clone();
    law::tick(state, world, &input);
    let next = program.select(world, state, elapsed + 1, args)?.0;
    Some(if events(&before, state).0 != 0 || next != row { 0 } else { elapsed + 1 })
}

/// Runs each program from `seed` until its state recurs: the ticks between the two visits
/// are the gait's cycle. `budget` bounds the ticks tried.
pub fn discover_gaits(world: &impl World, seed: &PlayerState, programs: &[Program], args: Arguments, budget: usize) -> Discovery {
    let mut report = Discovery::default();
    let mut orbits = HashSet::new();
    'programs: for (index, program) in programs.iter().enumerate() {
        let mut seen = HashMap::<Box<[u64]>, usize>::new();
        let mut visits = Vec::<(PlayerState, u32)>::new();
        let (mut state, mut elapsed) = (seed.clone(), 0);
        // A program that never reads its clock recurs whatever the clock says.
        let clocked = |elapsed: u32| if program.timed() { elapsed } else { 0 };
        for _ in 0..budget {
            fed(&mut state);
            let words = recurrence(&state, clocked(elapsed));
            if let Some(&first) = seen.get(&words) {
                let (start, start_elapsed) = visits[first].clone();
                let recording = RecordingWorld::new(world);
                let (mut replay, mut clock) = (start.clone(), start_elapsed);
                let (mut cycle, mut increments, mut clamped) = (Vec::new(), Vec::new(), Vec::new());
                for _ in first..visits.len() {
                    fed(&mut replay);
                    cycle.push(recurrence(&replay, clocked(clock)));
                    let from = replay.position;
                    clock = step(&recording, &mut replay, program, args, clock).unwrap();
                    let moved = replay.position.minus(from);
                    clamped.push(replay.movement_this_tick.map_or([false; 3], |wanted| [wanted.x != moved.x, wanted.y != moved.y, wanted.z != moved.z]));
                    increments.push(moved);
                }
                if !orbits.insert(orbit(cycle)) { report.repeats.push(index); continue 'programs; }
                let filed = Start { origin: body_block(&start), words, context: CollisionContext::of(&start) };
                let gait = report.gaits.len();
                if let Err(error) = report.memo.insert_at(world, &recording.take(), filed, |known| known.into_iter().flatten().chain([gait]).collect()) { report.refusals.push((index, error.into())); continue 'programs; }
                let displacement = replay.position.minus(start.position);
                let ticks = increments.len() as u32;
                report.gaits.push(Gait { program: program.clone(), arguments: args, start, elapsed: start_elapsed, ticks, displacement, increments, clamped,
                    speed: displacement.horizontal_sqr().sqrt() * 20.0 / ticks as f64 });
                continue 'programs;
            }
            seen.insert(words, visits.len());
            visits.push((state.clone(), elapsed));
            match step(world, &mut state, program, args, elapsed) {
                Some(next) => elapsed = next,
                None => { report.refusals.push((index, SegmentError::NoMatchingRow)); continue 'programs; }
            }
        }
        report.unresolved.push(index);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::orbit;

    #[test]
    fn an_orbit_is_its_least_rotation() {
        assert_eq!(orbit(vec![2, 3, 1]), vec![1, 2, 3]);
        assert_ne!(orbit(vec![1, 2, 3]), orbit(vec![1, 3, 2]));
    }
}
