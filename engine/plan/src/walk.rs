//! A sentence and its certificate. A sentence is a chain of steps, each a program at an aim
//! run for a number of ticks. Its certificate is one replay through the law from the exact
//! start state, tick by tick, with the tick of every cell its tube sweeps.
use law::{BlockPos, Input, PlayerState, footprint::RecordingWorld};
use words::{Arguments, Program};
use crate::{Goal, Timeline};

/// How the search produced a step's end state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// One segment, replayed through the law in the world.
    Replay,
}
#[derive(Clone, Copy, Debug)]
pub struct Step { pub program: u16, pub args: Arguments, pub ticks: u32, pub lane: Lane }

/// The body at rest on `column`: dropped with no input until every field but the tick
/// counter repeats. The counter then restarts at `tick`.
pub fn settle<T: Timeline>(timeline: &T, column: [f64; 3], tick: u64) -> Result<PlayerState, String> {
    let mut state = PlayerState { position: law::Vec3::new(column[0], column[1], column[2]), t: tick, ..PlayerState::default() };
    let still = |state: &PlayerState| { let mut bits = state.bits(); bits[0] = 0; bits };
    // A body that never comes to rest (a void, a bouncing floor) is refused, not waited for.
    for _ in 0..4096 {
        let before = still(&state);
        state.t = tick;
        law::try_tick(&mut state, timeline.at(tick), &Input::default()).map_err(|refusal| refusal.0)?;
        if still(&state) == before { state.t = tick; return Ok(state); }
    }
    Err(format!("no rest on {column:?}"))
}

#[derive(Clone, Debug)]
pub struct Certificate {
    /// Ticks from the start state to the first touch of the goal.
    pub ticks: u32,
    /// The same on the track's clock: from the last sample on A (Tracks.java, `onA`).
    pub clock: Option<u32>,
    /// The steps as replayed, cut at the touch.
    pub steps: Vec<Step>,
    /// The state's bits after each step.
    pub ends: Vec<Vec<u64>>,
    /// The worldtube: each cell the body's box swept, with its tick.
    pub tube: Vec<(u64, BlockPos)>,
    /// The cells read, with their queries.
    pub reads: usize,
    pub end: PlayerState,
}

/// One replay from the exact start. It is the only thing that makes a sentence certified:
/// the steps reach the goal, or the sentence is not returned.
pub fn certify<T: Timeline>(timeline: &T, programs: &[Program], start: &PlayerState, steps: &[Step], goal: &Goal, a: Option<BlockPos>, tube: bool) -> Result<Certificate, String> {
    let on_a = |state: &PlayerState| a.is_some() && state.on_ground && state.main_supporting_block == a;
    let mut state = start.clone();
    let (mut origin, mut total) = (0u32, 0u32);
    let mut out = Certificate { ticks: 0, clock: None, steps: Vec::new(), ends: Vec::new(), tube: Vec::new(), reads: 0, end: start.clone() };
    let mut reads = std::collections::BTreeSet::new();
    for step in steps {
        let program = &programs[step.program as usize];
        for elapsed in 0..step.ticks {
            let world = timeline.at(state.t);
            let before = state.clone();
            if tube {
                let recording = RecordingWorld::new(world);
                let (_, input) = program.select(&recording, &state, elapsed, step.args).ok_or("no row matches")?;
                law::try_tick(&mut state, &recording, &input).map_err(|refusal| format!("the law refused: {}", refusal.0))?;
                let footprint = recording.take();
                out.tube.extend(footprint.swept.iter().map(|&cell| (before.t, cell)));
                reads.extend(footprint.reads.keys().copied());
            } else {
                let (_, input) = program.select(world, &state, elapsed, step.args).ok_or("no row matches")?;
                law::try_tick(&mut state, world, &input).map_err(|refusal| format!("the law refused: {}", refusal.0))?;
            }
            total += 1;
            if state.health <= 0.0 { return Err(format!("dead at tick {total}")); }
            if on_a(&state) { origin = total; }
            if goal.touched(&before, &state) {
                out.steps.push(Step { ticks: elapsed + 1, ..*step });
                out.ends.push(state.bits());
                (out.ticks, out.clock, out.reads, out.end) = (total, (!on_a(&state)).then_some(total - origin), reads.len(), state);
                return Ok(out);
            }
        }
        out.steps.push(*step);
        out.ends.push(state.bits());
    }
    Err(format!("the replay ends at {:?} after {total} ticks without touching B", [state.position.x, state.position.y, state.position.z]))
}
