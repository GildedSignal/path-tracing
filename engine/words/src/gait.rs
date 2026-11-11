//! Gaits: the attractors of tiny programs, kept as cycles of classes
//! (vocabulary.md, section 6).
use std::collections::HashSet;
use law::{PlayerState, Vec3, World, pattern::{Key, PatternTable}};
use crate::{Arguments, Grid, Program, SegmentError, run_segment};

#[derive(Clone, Debug)]
pub struct Gait {
    pub program: Program,
    pub arguments: Arguments,
    /// A state on the attractor: its velocity bits recur.
    pub start: PlayerState,
    /// The cycle: each segment's class (state words and world part, as a key of the
    /// discovery's own table) and its ticks. The last segment exits into the first class.
    pub classes: Vec<(Key, u32)>,
    pub ticks: u32,
    pub displacement: Vec3,
    /// The orbit: the least period of the per-tick velocity, in ticks.
    pub period: u32,
    /// Blocks per second, horizontal.
    pub speed: f64,
}
#[derive(Debug, Default)]
pub struct Discovery {
    pub gaits: Vec<Gait>,
    /// Programs whose orbit an earlier program already gave.
    pub repeats: Vec<usize>,
    pub refusals: Vec<(usize, SegmentError)>,
    /// Programs with no class that recurs on recurring velocity bits within the budget.
    pub unresolved: Vec<usize>,
}

type Beat = ([u64; 3], bool, u8, bool);
fn beat(state: &PlayerState) -> Beat {
    let v = state.velocity;
    ([v.x.to_bits(), v.y.to_bits(), v.z.to_bits()], state.on_ground, state.pose as u8, state.sprinting)
}
/// The least period of a cyclic sequence, rotated to its least rotation.
fn orbit<T: Ord + Clone>(mut beats: Vec<T>) -> Vec<T> {
    let period = (1..=beats.len()).find(|&n| beats.len() % n == 0 && (n..beats.len()).all(|i| beats[i] == beats[i - n])).unwrap();
    beats.truncate(period);
    let rotation = (0..period).min_by(|&a, &b| beats[a..].iter().chain(&beats[..a]).cmp(beats[b..].iter().chain(&beats[..b]))).unwrap();
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

/// Runs each program from `seed` until a class recurs with the same velocity bits:
/// the cycle between the two visits is the gait. `budget` bounds the segments tried.
pub fn discover_gaits(world: &impl World, seed: &PlayerState, programs: &[Program], args: Arguments, grid: Grid, budget: usize) -> Discovery {
    let mut report = Discovery::default();
    let mut orbits = HashSet::new();
    'programs: for (index, program) in programs.iter().enumerate() {
        let mut table = PatternTable::<usize>::default();
        let mut visits = Vec::<(PlayerState, Key, u32)>::new();
        let mut state = seed.clone();
        for visit in 0..budget {
            fed(&mut state);
            let segment = match run_segment(world, &state, program, args) {
                Ok(segment) => segment,
                Err(error) => { report.refusals.push((index, error)); continue 'programs; }
            };
            let mut earlier = None;
            let key = match table.insert_at(world, &segment.footprint, crate::class::start(&state, segment.footprint.state_reads, grid),
                |previous| { earlier = previous; visit }) {
                Ok(key) => key,
                Err(error) => { report.refusals.push((index, error.into())); continue 'programs; }
            };
            if let Some(first) = earlier.filter(|&first| beat(&visits[first].0).0 == beat(&state).0) {
                let start = visits[first].0.clone();
                let ticks = visits[first..].iter().map(|visit| visit.2).sum();
                let mut beats = Vec::new();
                let mut replay = start.clone();
                for _ in first..visit {
                    fed(&mut replay);
                    let segment = run_segment(world, &replay, program, args).unwrap();
                    let mut tick = replay.clone();
                    for elapsed in 0..segment.ticks {
                        beats.push(beat(&tick));
                        let input = program.select(world, &tick, elapsed, args).unwrap().1;
                        law::tick(&mut tick, world, &input);
                    }
                    replay = segment.state;
                }
                let beats = orbit(beats);
                let period = beats.len() as u32;
                if !orbits.insert(beats) { report.repeats.push(index); continue 'programs; }
                let displacement = state.position.minus(start.position);
                report.gaits.push(Gait { program: program.clone(), arguments: args, start, ticks, displacement, period,
                    classes: visits[first..].iter().map(|visit| (visit.1.clone(), visit.2)).collect(),
                    speed: displacement.horizontal_sqr().sqrt() * 20.0 / ticks as f64 });
                continue 'programs;
            }
            visits.push((state, key, segment.ticks));
            state = segment.state;
        }
        report.unresolved.push(index);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::orbit;

    #[test]
    fn an_orbit_is_its_least_period_and_least_rotation() {
        assert_eq!(orbit(vec![2, 3, 1]), vec![1, 2, 3]);
        assert_eq!(orbit(vec![1, 2, 1, 2]), vec![1, 2]);
        assert_ne!(orbit(vec![1, 2, 3]), orbit(vec![1, 3, 2]));
    }
}
