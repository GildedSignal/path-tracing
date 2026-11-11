//! A segment: a program run through the law from one event to the next
//! (vocabulary.md, section 4).
use law::{Input, PlayerState, World, footprint::{Footprint, RecordingWorld},
    pattern::{self, PatternError}, state_reads::StateField as F};
use crate::{Arguments, Grid, Program, body_block, class};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExitKind { State(F), SupportContents, BodyBlock, ActiveRow }
impl ExitKind {
    fn bit(self) -> u128 {
        1 << match self { Self::State(field) => field as u8, Self::SupportContents => 125, Self::BodyBlock => 126, Self::ActiveRow => 127 }
    }
}
/// The set of kinds that fired on the segment's last tick.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Exit(pub u128);
impl Exit {
    pub fn contains(self, kind: ExitKind) -> bool { self.0 & kind.bit() != 0 }
    pub fn kinds(self) -> impl Iterator<Item = ExitKind> {
        F::ALL.iter().map(|&field| ExitKind::State(field))
            .chain([ExitKind::SupportContents, ExitKind::BodyBlock, ExitKind::ActiveRow])
            .filter(move |&kind| self.contains(kind))
    }
}
impl std::fmt::Debug for Exit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_list().entries(self.kinds()).finish() }
}

/// Resources consumed and produced, as deltas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Uses { pub food: i32, pub saturation: f32, pub exhaustion: f32, pub health: f32 }
/// The least each guarded resource reached during the segment, its start included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guards { pub food: i32, pub health: f32 }
impl Guards {
    /// Food above 6 and health above 0 throughout.
    pub fn hold(self) -> bool { self.food > 6 && self.health > 0.0 }
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub ticks: u32,
    pub exit: Exit,
    pub state: PlayerState,
    /// The cells, environment values and state fields read, and the swept cells.
    pub footprint: Footprint,
    pub uses: Uses,
    pub guards: Guards,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentError {
    NoMatchingRow,
    /// The tick budget ran out with no event: nothing is invented.
    NoEvent { ticks: u32 },
    Pattern(PatternError),
}
impl From<PatternError> for SegmentError { fn from(e: PatternError) -> Self { Self::Pattern(e) } }

/// The one rule: any discrete fact of the state changed, or the body's block did.
/// A counter's discrete fact is whether it is running: it changes every tick while it
/// counts, and a program that waits for it reads it in a predicate, which cuts by its
/// row. Resources are guards. Fields the law overwrites before it reads them (the
/// previous input, `movement_this_tick`, the `was_*` flags) echo other fields a tick late.
fn events(a: &PlayerState, b: &PlayerState) -> Exit {
    let mut exit = 0;
    macro_rules! cut {
        ($($field:ident),*) => { $(if a.$field != b.$field { exit |= ExitKind::State(F::$field).bit(); })* };
    }
    macro_rules! running {
        ($($field:ident),*) => { $(if (a.$field == 0) != (b.$field == 0) { exit |= ExitKind::State(F::$field).bit(); })* };
    }
    cut!(on_ground, horizontal_collision, vertical_collision, vertical_collision_below, minor_horizontal_collision,
        sprinting, sprint_speed_modifier, no_physics, shift_key_down, pose, in_water, in_lava, on_climbable,
        main_supporting_block, on_ground_no_blocks, crouching, swimming, underwater, eye_in_water, in_powder_snow,
        leather_boots, fall_flying, riding, boat, effects, flying, using_item, stuck_speed_multiplier);
    running!(no_jump_delay, remaining_fire_ticks, ticks_frozen);
    if a.health.to_bits() != b.health.to_bits() { exit |= ExitKind::State(F::health).bit(); }
    if (a.hurt_cooldown.remaining_ticks == 0) != (b.hurt_cooldown.remaining_ticks == 0) { exit |= ExitKind::State(F::hurt_cooldown).bit(); }
    if body_block(a) != body_block(b) { exit |= ExitKind::BodyBlock.bit(); }
    Exit(exit)
}
fn support(world: &impl World, state: &PlayerState) -> Result<Option<u128>, PatternError> {
    state.main_supporting_block.map(|p| pattern::cell_digest(world.block(p), world.loaded(p), 1 | 16)).transpose()
}

pub fn run_segment(world: &impl World, start: &PlayerState, program: &Program, args: Arguments) -> Result<Segment, SegmentError> {
    let recording = RecordingWorld::new(world);
    let mut state = start.clone();
    let mut guards = Guards { food: start.food_level, health: start.health };
    // One recording for the whole segment, predicates included: a field counts as read
    // only if the segment had not written it yet.
    let (mut row, mut input) = program.select(&recording, &state, 0, args).ok_or(SegmentError::NoMatchingRow)?;
    let mut ticks = 0;
    let exit = loop {
        if ticks == args.max_ticks { return Err(SegmentError::NoEvent { ticks }); }
        let before = state.clone();
        law::tick(&mut state, &recording, &input);
        ticks += 1;
        guards = Guards { food: guards.food.min(state.food_level), health: guards.health.min(state.health) };
        let mut exit = events(&before, &state);
        if exit.contains(ExitKind::State(F::main_supporting_block)) && support(&recording, &before)? != support(&recording, &state)? {
            exit.0 |= ExitKind::SupportContents.bit();
        }
        let next = program.select(&recording, &state, ticks, args).ok_or(SegmentError::NoMatchingRow)?;
        if next.0 != row { exit.0 |= ExitKind::ActiveRow.bit(); }
        if exit.0 != 0 { break exit; }
        (row, input) = next;
    };
    let uses = Uses { food: state.food_level - start.food_level, saturation: state.saturation - start.saturation,
        exhaustion: state.exhaustion - start.exhaustion, health: state.health - start.health };
    Ok(Segment { ticks, exit, state, footprint: recording.take(), uses, guards })
}

/// The price of risk (section 7): does the segment end in the same block and class
/// when its inputs arrive one tick early or one tick late. Clearance is not built:
/// it needs a definition first (see the job report). Width prices reuse, never gates it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Width { pub early: bool, pub late: bool }

pub fn width(world: &impl World, start: &PlayerState, program: &Program, args: Arguments, segment: &Segment, grid: Grid) -> Width {
    let mut tape = Vec::<Input>::new();
    let mut state = start.clone();
    for elapsed in 0..=segment.ticks {
        let Some((_, input)) = program.select(world, &state, elapsed, args) else { break };
        tape.push(input);
        if elapsed < segment.ticks { law::tick(&mut state, world, &input); }
    }
    let reads = segment.footprint.state_reads;
    let same = |shifted: &dyn Fn(usize) -> Input| {
        let mut state = start.clone();
        for tick in 0..segment.ticks as usize { law::tick(&mut state, world, &shifted(tick)); }
        body_block(&state) == body_block(&segment.state) && class(&state, reads, grid) == class(&segment.state, reads, grid)
    };
    Width {
        early: same(&|tick| tape[(tick + 1).min(tape.len() - 1)]),
        late: same(&|tick| if tick == 0 { start.input } else { tape[tick - 1] }),
    }
}

