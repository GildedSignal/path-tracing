use law::{Input, PlayerState, World, state_reads::StateField as F};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Predicate {
    Always,
    OnGround,
    HorizontalCollision,
    VerticalCollision,
    InWater,
    InLava,
    WaterDeep,
    Climbable,
    JumpReady,
    Sprinting,
    SupportEnds,
    Since(u32),
    All(Vec<Predicate>),
    Not(Box<Predicate>),
}
impl Predicate {
    fn constant(&self)->Option<bool> {match self {
        Self::Always=>Some(true),Self::Not(p)=>p.constant().map(|v|!v),
        Self::All(ps)=>{let mut known=true;for p in ps {match p.constant() {Some(false)=>return Some(false),None=>known=false,_=>()}}known.then_some(true)},
        _=>None,
    }}
    fn matches(&self, world: &impl World, state: &PlayerState, elapsed: u32) -> bool {
        let read = |field| world.read_state(field);
        match self {
            Self::Always => true,
            Self::OnGround => {
                read(F::on_ground);
                state.on_ground
            }
            Self::HorizontalCollision => {
                read(F::horizontal_collision);
                state.horizontal_collision
            }
            Self::VerticalCollision => {
                read(F::vertical_collision);
                state.vertical_collision
            }
            Self::InWater => {
                read(F::in_water);
                state.in_water
            }
            Self::InLava => {
                read(F::in_lava);
                state.in_lava
            }
            // The law's own band (movement.rs:173).
            Self::WaterDeep => { read(F::water_height); state.water_height > 0.4 }
            Self::Climbable => {
                read(F::on_climbable);
                state.on_climbable
            }
            Self::JumpReady => {
                read(F::no_jump_delay);
                state.no_jump_delay == 0
            }
            Self::Sprinting => {
                read(F::sprinting);
                state.sprinting
            }
            Self::SupportEnds => {
                read(F::velocity);
                read(F::step_height);
                law::can_fall(state, world, state.velocity.x, state.velocity.z, state.step_height as f64)
            }
            Self::Since(ticks) => elapsed >= *ticks,
            Self::All(predicates) => predicates.iter().all(|p| p.matches(world, state, elapsed)),
            Self::Not(predicate) => !predicate.matches(world, state, elapsed),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub when: Predicate,
    pub input: Input,
}

/// First matching row wins. Absence of a matching row is an explicit refusal.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub rows: Vec<Row>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arguments {
    /// Repetitions of the word, including the single repetition of a transition.
    pub count: u32,
    pub yaw_index: u8,
    pub refined_yaw: Option<f32>,
    pub pitch: f32,
    /// A work budget. Exhaustion refuses the compilation without inventing an event.
    pub max_ticks: u32,
}
impl Default for Arguments {
    fn default() -> Self { Self { count: 1, yaw_index: 0, refined_yaw: None, pitch: 0.0, max_ticks: 256 } }
}
impl Arguments {
    /// Representation limits, not a physics horizon or a learning budget.
    pub const MAX_TICKS:u64=u32::MAX as u64;
    pub const MAX_COUNT:u32=u32::MAX;
    pub fn yaw(self) -> f32 {
        self.refined_yaw.unwrap_or_else(|| {
            let index = (self.yaw_index % 64) as u32 * 1024;
            // Use the middle of the selected sine-table bin. Inverting the
            // law's f32 degree conversion keeps rounding inside that bin.
            let radians = (index as f64 + 0.5) * std::f64::consts::TAU / 65536.0;
            (radians / 0.017453292f32 as f64) as f32
        })
    }
}
impl Predicate {
    /// Reads the segment's clock.
    fn timed(&self) -> bool {
        match self { Self::Since(_) => true, Self::All(all) => all.iter().any(Self::timed), Self::Not(inner) => inner.timed(), _ => false }
    }
    /// Its answer can change only on a tick the event rule cuts: every field it reads is
    /// one whose change ends a segment (segment.rs, `events`).
    fn event_bound(&self) -> bool {
        match self {
            Self::Always | Self::OnGround | Self::HorizontalCollision | Self::VerticalCollision | Self::InWater | Self::InLava
            | Self::Climbable | Self::JumpReady | Self::Sprinting => true,
            Self::WaterDeep | Self::SupportEnds | Self::Since(_) => false,
            Self::All(all) => all.iter().all(Self::event_bound),
            Self::Not(inner) => inner.event_bound(),
        }
    }
}
impl Program {
    pub fn held_input(&self,args:Arguments)->Option<Input> {
        for row in &self.rows {match row.when.constant()? {false=>continue,true=>return Some(Input {yaw:args.yaw(),pitch:args.pitch,..row.input})}}None
    }
    /// Its active row is constant inside a segment, so its segment from a state is decided
    /// by the input it selects there.
    pub fn event_bound(&self) -> bool { self.rows.iter().all(|row| row.when.event_bound()) }
    /// Some row waits on the ticks since the segment started.
    pub fn timed(&self) -> bool { self.rows.iter().any(|row| row.when.timed()) }
    pub fn select(&self, world: &impl World, state: &PlayerState, elapsed: u32, args: Arguments) -> Option<(usize, Input)> {
        let (index, row) = self.rows.iter().enumerate().find(|(_, row)| row.when.matches(world, state, elapsed))?;
        Some((index, Input { yaw: args.yaw(), pitch: args.pitch, ..row.input }))
    }
}

/// 3 movement directions x 2 sprint choices x 3 jump predicates x 3 sneak
/// predicates. No row names a technique or a block type.
#[derive(Clone,Copy)]
#[repr(u16)]
pub enum HeldPredicate {Never,Always,OnGround}
/// The product order of the toolkit, defined beside its generator.
pub const fn tiny_program_id(strafe:i8,sprint:bool,jump:HeldPredicate,sneak:HeldPredicate)->u16 {
    ((strafe+1) as u16*2+sprint as u16)*9+jump as u16*3+sneak as u16
}
pub fn tiny_programs() -> Vec<Program> {
    let predicates = [Predicate::Not(Box::new(Predicate::Always)), Predicate::Always, Predicate::OnGround];
    let mut programs = Vec::new();
    for strafe in [-1, 0, 1] {
        for sprint in [false, true] {
            for jump in &predicates {
                for sneak in &predicates {
                    let keys = Input { forward: true, left: strafe < 0, right: strafe > 0, sprint, ..Input::default() };
                    programs.push(Program { rows: vec![
                        Row { when: Predicate::All(vec![jump.clone(), sneak.clone()]), input: Input { jump: true, sneak: true, ..keys } },
                        Row { when: jump.clone(), input: Input { jump: true, ..keys } },
                        Row { when: sneak.clone(), input: Input { sneak: true, ..keys } },
                        Row { when: Predicate::Always, input: keys },
                    ] });
                }
            }
        }
    }
    programs
}
