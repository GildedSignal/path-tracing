//! Te search (job 3.2b steps 3, 4, 6 and 7): A* over exact states (Hart, Nilsson and
//! Raphael 1968, recalled, not re-read). A node is a concrete state with its tick. An edge is
//! a word: a proposal of the node's class certified by one replay, a free arc, gait cycles,
//! or a segment cut where the world changes. The cost is ticks.
//!
//! The closed set holds exact states. One dominance is used, by proof: in a world that never
//! changes, the law reads the tick counter nowhere (footprints.md section 4), so of two
//! states equal in every other bit the later one is dominated. A node is discarded otherwise
//! only when its cost plus the lower bound cannot beat a certified sentence.
//!
//! Two orders. `Admissible` is A* proper: least cost plus the lower bound first, every aim at
//! once. `Field` takes its order from the backward pass (reach.rs): first the least estimate
//! of ticks to go (greedy best-first, Doran and Michie 1966, recalled), aims nearest the
//! next set first and the others ring by ring, so every proposal is reached at some budget;
//! then, once a sentence is certified, least cost plus estimate, discarding by the bound.
//! The estimate only orders.
use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}, time::Instant};
use law::{Block, BlockPos, PlayerState, footprint::RecordingWorld};
use words::{Arguments, Found, Program, ProposalTable, SegmentError, body_block, recurrence};
use crate::{Certificate, Field, Goal, Lane, Step, Timeline, advance, arith, certify};

pub struct Problem<'a, T: Timeline> {
    pub timeline: &'a T,
    pub start: PlayerState,
    pub goal: Goal,
    /// The box the body's block must stay in, and the box of cells that may be read.
    pub body: (BlockPos, BlockPos),
    pub read: (BlockPos, BlockPos),
    /// The track's A, for its clock.
    pub a: Option<BlockPos>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order { Admissible, Field }
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub order: Order,
    pub arcs: bool,
    pub gaits: bool,
    /// Replay every arc in the world beside its arithmetic and count the states that differ.
    pub audit: bool,
    /// Keep searching after the first certified sentence.
    pub improve: bool,
    /// Nodes held at most: memory is a resource, like time.
    pub nodes: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stopped { Exhausted, Asked, First, Memory }

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub expansions: u64,
    pub nodes: u64,
    pub decisions: u64,
    pub replays: u64,
    pub replay_ticks: u64,
    pub replay_us: f64,
    /// Proposals replayed at each decision.
    pub per_decision: Vec<u32>,
    pub misses: u64,
    pub miss_aims: u64,
    pub miss_us: Vec<f32>,
    pub invented_runs: u64,
    pub invented_ticks: u64,
    pub arcs: u64,
    pub arc_ticks: u64,
    pub arc_us: f64,
    pub gaits: u64,
    pub gait_ticks: u64,
    pub cut_ticks: u64,
    pub audits: u64,
    pub audit_differs: u64,
    pub dead: u64,
    pub dominated: u64,
    pub pruned: u64,
    pub no_event: u64,
    /// Each rule the law refused by name, its count and the first place.
    pub refusals: Vec<(String, u64, [f64; 3])>,
    /// Each certified improvement: seconds of thinking, ticks.
    pub solutions: Vec<(f64, u32)>,
    pub uncertified: Vec<String>,
    /// Per lane: step ends compared with the certifying replay, and those whose bits differ.
    pub compared: [u64; 4],
    pub differs: [u64; 4],
}
struct Node { state: PlayerState, g: u32, parent: u32, step: Step, anchor: Option<BlockPos>, h: f64, lb: u32, recur: u64 }
struct Entry { key: [f64; 2], node: u32, ring: u8 }
impl PartialEq for Entry { fn eq(&self, other: &Self) -> bool { self.cmp(other) == Ordering::Equal } }
impl Eq for Entry {}
impl PartialOrd for Entry { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl Ord for Entry {
    // Least key first; among equals the node made last, then the nearer ring.
    fn cmp(&self, other: &Self) -> Ordering {
        other.key[0].total_cmp(&self.key[0]).then(other.key[1].total_cmp(&self.key[1])).then(self.node.cmp(&other.node)).then(other.ring.cmp(&self.ring))
    }
}
/// Where a free arc went: out of the world, or to its last free tick.
enum Fate { Lost, Ends(Option<f64>) }
const ALL: u8 = u8::MAX;
const ROOT: u32 = u32::MAX;

pub struct Outcome {
    pub sentence: Option<(Vec<Step>, Certificate)>,
    /// The bracket: nothing beats `lower`; the sentence is the upper bound.
    pub lower: u32,
    /// The least cost plus bound left open: what this vocabulary could still say.
    pub frontier: Option<u32>,
    pub stopped: Stopped,
    pub seconds: f64,
    /// The node nearest the goal by the order's estimate: where the search stood.
    pub nearest: Option<(PlayerState, u32, Option<BlockPos>)>,
}

pub struct Planner<'a, T: Timeline> {
    pub problem: Problem<'a, T>,
    pub programs: Vec<Program>,
    pub table: &'a mut ProposalTable,
    pub field: Option<Field>,
    pub options: Options,
    pub stats: Stats,
    free: Option<&'a Block>,
    nodes: Vec<Node>,
    open: BinaryHeap<Entry>,
    closed: HashMap<Box<[u64]>, u32>,
    best: Option<(Vec<Step>, Certificate)>,
    frozen: bool,
    improving: bool,
    index: Vec<Option<u16>>,
    clock: Instant,
}
fn fed(state: &PlayerState) -> PlayerState {
    PlayerState { food_level: 20, saturation: 20.0, exhaustion: 0.0, food_tick_timer: 0, ..state.clone() }
}
fn hash(words: &[u64]) -> u64 { words.iter().fold(0xcbf29ce484222325, |h, &w| (h ^ w).wrapping_mul(0x100000001b3)) }

impl<'a, T: Timeline> Planner<'a, T> {
    pub fn new(problem: Problem<'a, T>, programs: Vec<Program>, table: &'a mut ProposalTable, field: Option<Field>, options: Options) -> Self {
        let frozen = problem.timeline.next_change(0).is_none();
        // Free space is the content of the cell the body starts in, if that is air.
        let world = problem.timeline.at(problem.start.t);
        let free = Some(law::World::block(world, body_block(&problem.start))).filter(|block| block.air && options.arcs);
        let mut planner = Planner { problem, programs, table, field, options, stats: Stats::default(), free, nodes: Vec::new(), open: BinaryHeap::new(),
            closed: HashMap::new(), best: None, frozen, improving: false, index: Vec::new(), clock: Instant::now() };
        let start = planner.problem.start.clone();
        let root = Step { program: 0, args: Arguments::default(), ticks: 0, lane: Lane::Replay };
        planner.push(ROOT, root, start);
        planner
    }
    fn seconds(&self) -> f64 { self.clock.elapsed().as_secs_f64() }
    fn readable(&self, p: BlockPos) -> bool { (0..3).all(|a| p[a] >= self.problem.read.0[a] && p[a] <= self.problem.read.1[a]) }
    fn dead(&self, state: &PlayerState) -> bool {
        let block = body_block(state);
        state.health <= 0.0 || (0..3).any(|a| block[a] < self.problem.body.0[a] || block[a] > self.problem.body.1[a])
    }
    fn key(&self, node: &Node) -> [f64; 2] {
        match (self.options.order, self.improving) {
            (Order::Admissible, _) => [(node.g + node.lb) as f64, node.lb as f64],
            // Among equal estimates the deeper node first: where nothing moves, time does.
            (Order::Field, false) => [node.h, -(node.g as f64)],
            (Order::Field, true) => [node.g as f64 + node.h, node.h],
        }
    }
    /// What makes two states one under the law. The fields every tick writes before it reads
    /// them (the previous input, yaw and pitch, `movement_this_tick`, crouching, the `was_*`
    /// flags: movement.rs:128-140, vocabulary.md section 3) cannot reach the future, so states
    /// that differ only there have the same futures. So has the tick counter, in a world that
    /// never changes.
    fn identity(&self, state: &PlayerState) -> Box<[u64]> {
        let mut same = PlayerState { input: law::Input::default(), yaw: 0.0, pitch: 0.0, movement_this_tick: None, crouching: false,
            was_eye_in_water: false, was_in_powder_snow: false, ..state.clone() };
        if self.frozen { same.t = 0; }
        same.bits().into()
    }
    /// Adds a node unless an exact state proves it dominated or the bound proves it useless.
    fn push(&mut self, parent: u32, step: Step, state: PlayerState) -> Option<u32> {
        if self.dead(&state) { self.stats.dead += 1; return None; }
        let g = if parent == ROOT { 0 } else { self.nodes[parent as usize].g + step.ticks };
        let lb = self.problem.goal.lower_bound(&state);
        if self.best.as_ref().is_some_and(|best| g + lb >= best.1.ticks) { self.stats.pruned += 1; return None; }
        let identity = self.identity(&state);
        if self.closed.get(&identity).is_some_and(|&known| known <= g) { self.stats.dominated += 1; return None; }
        self.closed.insert(identity, g);
        // The body's set: the cell it stands on if the backward pass reached it, else its parent's.
        let inherited = if parent == ROOT { None } else { self.nodes[parent as usize].anchor };
        let (anchor, h) = match &mut self.field {
            Some(field) => {
                let stood = state.main_supporting_block.filter(|support| state.on_ground && field.cells.contains_key(support));
                if let (Some(now), Some(before)) = (stood, inherited) {
                    if field.cells[&now].t < field.cells[&before].t { field.cells.get_mut(&before).unwrap().arrivals += 1; }
                }
                let anchor = stood.or(inherited);
                (anchor, anchor.map_or(f64::INFINITY, |anchor| field.estimate(anchor, state.position, !state.on_ground && state.velocity.y < 0.0, &self.problem.goal).0 + lb as f64))
            }
            None => (None, lb as f64),
        };
        let recur = hash(&recurrence(&fed(&state), 0));
        let id = self.nodes.len() as u32;
        self.nodes.push(Node { state, g, parent, step, anchor, h, lb, recur });
        self.stats.nodes += 1;
        let ring = if self.options.order == Order::Field { 0 } else { ALL };
        self.open.push(Entry { key: self.key(&self.nodes[id as usize]), node: id, ring });
        Some(id)
    }
    fn steps(&self, mut node: u32) -> (Vec<Step>, Vec<u32>) {
        let (mut steps, mut ids) = (Vec::new(), Vec::new());
        while self.nodes[node as usize].parent != ROOT {
            steps.push(self.nodes[node as usize].step);
            ids.push(node);
            node = self.nodes[node as usize].parent;
        }
        steps.reverse();
        ids.reverse();
        (steps, ids)
    }
    /// A chain touched the goal: it becomes a sentence only if one replay from the exact
    /// start certifies it.
    fn solution(&mut self, parent: u32, last: Step) {
        let (mut steps, ids) = self.steps(parent);
        steps.push(last);
        let p = &self.problem;
        match certify(p.timeline, &self.programs, &p.start, &steps, &p.goal, p.a, false) {
            Ok(certificate) => {
                for (place, &id) in ids.iter().enumerate().take(certificate.ends.len()) {
                    let lane = self.nodes[id as usize].step.lane as usize;
                    self.stats.compared[lane] += 1;
                    self.stats.differs[lane] += (self.nodes[id as usize].state.bits() != certificate.ends[place]) as u64;
                }
                if self.best.as_ref().is_none_or(|best| certificate.ticks < best.1.ticks) {
                    self.stats.solutions.push((self.seconds(), certificate.ticks));
                    self.best = Some((certificate.steps.clone(), certificate));
                    if !self.improving && self.options.order == Order::Field {
                        // From here the order is cost plus estimate, and the bound discards.
                        self.improving = true;
                        let entries = std::mem::take(&mut self.open).into_vec();
                        let bound = self.best.as_ref().unwrap().1.ticks;
                        self.open = entries.into_iter().filter(|e| { let n = &self.nodes[e.node as usize]; n.g + n.lb < bound })
                            .map(|e| Entry { key: self.key(&self.nodes[e.node as usize]), ..e }).collect();
                    }
                }
            }
            Err(why) => if self.stats.uncertified.len() < 64 { self.stats.uncertified.push(why) },
        }
    }
    /// The zone lies within one cell of where a segment between these states can be.
    fn near(&self, a: &PlayerState, b: &PlayerState) -> bool {
        let reach = crate::goal::hull(a.bounding_box(), b.bounding_box());
        let grown = law::Aabb { min: reach.min.minus(law::Vec3::new(1.0, 1.0, 1.0)), max: reach.max.plus(law::Vec3::new(1.0, 1.0, 1.0)) };
        grown.intersects(self.problem.goal.zone())
    }
    /// A word run from `parent` reached `state`: check the goal tick by tick where it could
    /// have been touched, then file the node and try the arithmetic lanes from it.
    fn child(&mut self, parent: u32, step: Step, state: PlayerState) {
        let from = self.nodes[parent as usize].state.clone();
        if self.near(&from, &state) {
            let (timeline, goal) = (self.problem.timeline, self.problem.goal);
            let mut again = from.clone();
            let mut touched = false;
            let ran = advance(timeline, &mut again, &self.programs[step.program as usize], step.args, step.ticks, &mut |before, after| { touched = goal.touched(before, after); touched });
            if let (Ok(ticks), true) = (ran, touched) { return self.solution(parent, Step { ticks, ..step }); }
        }
        let Some(id) = self.push(parent, step, state) else { return };
        if step.lane != Lane::Replay { return; }
        if self.options.arcs && self.free.is_some() && arith::flying(&self.nodes[id as usize].state) {
            // A body in the air is ordered by where its free arc ends (the arc's ticks plus the
            // estimate there): last if the arc leaves the
            // world, and never discarded for it.
            let fate = self.arc(id);
            match fate {
                Fate::Lost => self.nodes[id as usize].h = f64::INFINITY,
                Fate::Ends(Some(through)) => self.nodes[id as usize].h = through,
                Fate::Ends(None) => {}
            }
        }
        if self.options.gaits && self.frozen { self.gait(id); }
    }
    /// The free arc from a node in the air: the same program, tick by tick in free space,
    /// while its tube meets nothing. One edge, to the last free tick.
    fn arc(&mut self, from: u32) -> Fate {
        let step = self.nodes[from as usize].step;
        let program = self.programs[step.program as usize].clone();
        if program.timed() { return Fate::Ends(None); }
        let (free, timeline, goal) = (self.free.unwrap(), self.problem.timeline, self.problem.goal);
        let mut lost = false;
        let clock = Instant::now();
        let mut state = self.nodes[from as usize].state.clone();
        let change = timeline.next_change(state.t);
        let mut ticks = 0;
        loop {
            if change.is_some_and(|change| state.t >= change) { break; }
            let real = timeline.at(state.t);
            let next = match arith::free_tick(real, free, &state, &program, step.args, ticks, &|p| self.readable(p)) {
                Ok(next) => next,
                Err(ends) => { lost = ends == arith::Ends::Leaves; break }
            };
            if self.options.audit {
                let mut replayed = state.clone();
                let same = advance(timeline, &mut replayed, &program, step.args, 1, &mut |_, _| false).is_ok() && replayed.bits() == next.bits();
                self.stats.audits += 1;
                self.stats.audit_differs += !same as u64;
            }
            ticks += 1;
            let touched = goal.touched(&state, &next);
            state = next;
            if touched {
                self.stats.arc_ticks += ticks as u64;
                self.stats.arc_us += clock.elapsed().as_secs_f64() * 1e6;
                self.solution(from, Step { ticks, lane: Lane::Arc, ..step });
                return Fate::Ends(None);
            }
            if self.dead(&state) { lost = true; break; }
        }
        self.stats.arc_us += clock.elapsed().as_secs_f64() * 1e6;
        if lost { return Fate::Lost; }
        if ticks == 0 { return Fate::Ends(None); }
        self.stats.arcs += 1;
        self.stats.arc_ticks += ticks as u64;
        // The estimate through the arc stands whether or not its end is a new node.
        let anchor = self.nodes[from as usize].anchor;
        let through = match (&self.field, anchor) {
            (Some(field), Some(anchor)) => Some(ticks as f64 + field.estimate(anchor, state.position, state.velocity.y < 0.0, &goal).0 + goal.lower_bound(&state) as f64),
            _ => None,
        };
        let end = self.push(from, Step { ticks, lane: Lane::Arc, ..step }, state);
        Fate::Ends(end.map(|end| ticks as f64 + self.nodes[end as usize].h).or(through).filter(|_| self.field.is_some()))
    }
    /// A gait, found and not given: the node's state, position and resources aside, has the
    /// bits of an ancestor reached by the same program at the same aim. One cycle is replayed
    /// and recorded; further cycles advance by its increments while the ground stays uniform.
    fn gait(&mut self, id: u32) {
        let step = self.nodes[id as usize].step;
        let program = self.programs[step.program as usize].clone();
        if program.timed() { return; }
        let words = recurrence(&fed(&self.nodes[id as usize].state), 0);
        let (mut ancestor, mut period) = (self.nodes[id as usize].parent, step.ticks);
        let found = loop {
            if ancestor == ROOT { return; }
            let node = &self.nodes[ancestor as usize];
            if node.recur == self.nodes[id as usize].recur && recurrence(&fed(&node.state), 0) == words { break period; }
            if node.parent == ROOT || node.step.program != step.program || node.step.args != step.args || node.step.lane != Lane::Replay { return; }
            period += node.step.ticks;
            ancestor = node.parent;
        };
        let (timeline, goal) = (self.problem.timeline, self.problem.goal);
        let world = timeline.at(0);
        let recording = RecordingWorld::new(world);
        let mut states = vec![self.nodes[id as usize].state.clone()];
        for elapsed in 0..found {
            let mut next = states[elapsed as usize].clone();
            let Some((_, input)) = program.select(&recording, &next, elapsed, step.args) else { return };
            if law::try_tick(&mut next, &recording, &input).is_err() || goal.touched(&states[elapsed as usize], &next) { return; }
            states.push(next);
        }
        if recurrence(&fed(states.last().unwrap()), 0) != words { return; }
        let Some(cycle) = arith::Cycle::observed(&states, recording.take().reads.keys().copied()) else { return };
        self.stats.replay_ticks += found as u64;
        let Some(first) = self.push(id, Step { ticks: found, ..step }, states.last().unwrap().clone()) else { return };
        let (mut state, mut cycles) = (states.last().unwrap().clone(), 0u32);
        while cycle.layered(world, &state, &|p| self.readable(p)) {
            let mut next = state.clone();
            if let Some(ticks) = cycle.advance(&mut next, &states, &mut |before, after| goal.touched(before, after)) {
                self.stats.gait_ticks += (cycles * found + ticks) as u64;
                return self.solution(first, Step { ticks: cycles * found + ticks, lane: Lane::Gait, ..step });
            }
            if self.dead(&next) { break; }
            (state, cycles) = (next, cycles + 1);
        }
        if cycles == 0 { return; }
        self.stats.gaits += 1;
        self.stats.gait_ticks += (cycles * found) as u64;
        self.push(first, Step { ticks: cycles * found, lane: Lane::Gait, ..step }, state);
    }
    fn refused(&mut self, error: &SegmentError, count: u64, at: &PlayerState) {
        let name = match error { SegmentError::Refused(refusal) => refusal.0.clone(), SegmentError::NoEvent { .. } => { self.stats.no_event += count; return } other => format!("{other:?}") };
        match self.stats.refusals.iter_mut().find(|known| known.0 == name) {
            Some(known) => known.1 += count,
            None => self.stats.refusals.push((name, count, [at.position.x, at.position.y, at.position.z])),
        }
    }
    fn program_index(&mut self, table: u32) -> u16 {
        if self.index.len() <= table as usize { self.index.resize(table as usize + 1, None); }
        if self.index[table as usize].is_none() {
            let program = self.table.program(table);
            self.index[table as usize] = Some(self.programs.iter().position(|known| known == program).expect("a program of the planner's") as u16);
        }
        self.index[table as usize].unwrap()
    }
    /// The aim nearest the bearing from `node` to its next set, or to the goal.
    fn bearing(&self, node: &Node) -> u8 {
        let (x, z) = match (&self.field, node.anchor) {
            (Some(field), Some(anchor)) => field.estimate(anchor, node.state.position, false, &self.problem.goal).1,
            _ => (self.problem.goal.cell[0] as f64 + 0.5, self.problem.goal.cell[2] as f64 + 0.5),
        };
        let degrees = (node.state.position.x - x).atan2(z - node.state.position.z).to_degrees();
        (degrees / 5.625).round().rem_euclid(64.0) as u8
    }
    /// A segment that met no event before the world changes ends at the change: a fact at a
    /// known tick cuts a word as an event does.
    fn cut(&mut self, id: u32, program: u16, args: Arguments, ticks: u32) {
        let (timeline, goal) = (self.problem.timeline, self.problem.goal);
        let mut state = self.nodes[id as usize].state.clone();
        let mut touched = false;
        let ran = advance(timeline, &mut state, &self.programs[program as usize], args, ticks, &mut |before, after| { touched = goal.touched(before, after); touched });
        let Ok(ran) = ran else { return };
        self.stats.cut_ticks += ran as u64;
        let step = Step { program, args, ticks: ran, lane: Lane::Cut };
        if touched { self.solution(id, step) } else { self.push(id, step, state); }
    }
    /// One decision: the proposals of the node's class at this ring's aims, each certified by
    /// a replay; where the class has none at an aim, invention there (a miss).
    fn expand(&mut self, id: u32, ring: u8) {
        let state = self.nodes[id as usize].state.clone();
        let timeline = self.problem.timeline;
        let horizon = timeline.next_change(state.t).map(|change| (change - state.t) as u32);
        let aims: Vec<u8> = if ring == ALL { (0..64).collect() } else {
            let base = self.bearing(&self.nodes[id as usize]);
            match ring { 0 => vec![base], 32 => vec![(base + 32) % 64], r => vec![(base + r) % 64, (base + 64 - r) % 64] }
        };
        if let (Some(field), Some(anchor)) = (&mut self.field, self.nodes[id as usize].anchor) { field.cells.get_mut(&anchor).unwrap().tries += 1; }
        self.stats.expansions += 1;
        self.stats.decisions += 1;
        let found: Vec<Found> = self.table.proposals(&state).into_iter().filter(|found| {
            let args = self.table.proposal(found.id).arguments;
            args.refined_yaw.is_none() && args.max_ticks == Arguments::default().max_ticks && aims.contains(&args.yaw_index)
        }).collect();
        self.stats.per_decision.push(found.len() as u32);
        let world = timeline.at(state.t);
        for &proposal in &found {
            let (program, args) = (self.table.proposal(proposal.id).program, self.table.proposal(proposal.id).arguments);
            let program = self.program_index(program);
            let clock = Instant::now();
            let replayed = self.table.replay(world, &state, proposal);
            self.stats.replay_us += clock.elapsed().as_secs_f64() * 1e6;
            self.stats.replays += 1;
            match replayed {
                // A word that runs past a change of the world is cut at the change.
                Ok(certified) if horizon.is_some_and(|h| certified.segment.ticks > h) => self.cut(id, program, args, horizon.unwrap()),
                Ok(certified) => {
                    self.stats.replay_ticks += certified.segment.ticks as u64;
                    self.child(id, Step { program, args, ticks: certified.segment.ticks, lane: Lane::Replay }, certified.segment.state);
                }
                Err(SegmentError::NoEvent { .. }) if horizon.is_some() => self.cut(id, program, args, horizon.unwrap()),
                Err(error) => self.refused(&error, 1, &state),
            }
        }
        let missing: Vec<Arguments> = aims.iter().filter(|&&aim| !found.iter().any(|f| self.table.proposal(f.id).arguments.yaw_index == aim))
            .map(|&yaw_index| Arguments { yaw_index, ..Arguments::default() }).collect();
        if missing.is_empty() { return; }
        self.stats.misses += 1;
        self.stats.miss_aims += missing.len() as u64;
        let clock = Instant::now();
        let programs = self.programs.clone();
        let (invented, _) = self.table.invent(world, &state, &programs, &missing);
        self.stats.miss_us.push((clock.elapsed().as_secs_f64() * 1e6) as f32);
        self.stats.invented_runs += invented.runs as u64;
        for (error, count) in &invented.refused { self.refused(error, *count as u64, &state); }
        for piece in &invented.pieces {
            if horizon.is_some_and(|h| piece.segment.ticks > h) { self.cut(id, piece.program as u16, piece.arguments, horizon.unwrap()); continue; }
            self.stats.invented_ticks += piece.segment.ticks as u64;
            self.stats.replay_ticks += piece.segment.ticks as u64;
            self.child(id, Step { program: piece.program as u16, args: piece.arguments, ticks: piece.segment.ticks, lane: Lane::Replay }, piece.segment.state.clone());
        }
        // The inputs whose run met no event before the world changes.
        if let Some(horizon) = horizon.filter(|&h| h <= Arguments::default().max_ticks) {
            for &args in &missing {
                let mut inputs = Vec::new();
                for (program, rows) in programs.iter().enumerate() {
                    let Some((_, input)) = rows.select(world, &state, 0, args) else { continue };
                    if inputs.contains(&input) || invented.pieces.iter().any(|piece| piece.arguments == args && programs[piece.program].select(world, &state, 0, args).map(|s| s.1) == Some(input)) { continue; }
                    inputs.push(input);
                    self.cut(id, program as u16, args, horizon);
                }
            }
        }
    }
    /// Runs until `stop` says so (seconds of thinking, the best ticks so far, the lower
    /// bound), the open list empties, or memory runs out. Anytime: whatever stops it, the
    /// best certified sentence so far is returned with its bracket.
    pub fn run(&mut self, stop: &mut dyn FnMut(f64, Option<u32>, u32) -> bool) -> Outcome {
        let lower = self.problem.goal.lower_bound(&self.problem.start);
        let stopped = loop {
            if stop(self.seconds(), self.best.as_ref().map(|best| best.1.ticks), lower) { break Stopped::Asked; }
            if self.best.is_some() && !self.options.improve { break Stopped::First; }
            if self.nodes.len() >= self.options.nodes { break Stopped::Memory; }
            let Some(entry) = self.open.pop() else { break Stopped::Exhausted };
            let node = &self.nodes[entry.node as usize];
            if self.best.as_ref().is_some_and(|best| node.g + node.lb >= best.1.ticks) { self.stats.pruned += 1; continue; }
            if self.closed.get(&self.identity(&node.state)).is_some_and(|&known| known < node.g) { self.stats.dominated += 1; continue; }
            // An estimate that moved since the entry was made: the entry takes its place again.
            let key = self.key(node);
            if key != entry.key { self.open.push(Entry { key, ..entry }); continue; }
            if entry.ring != ALL && entry.ring < 32 {
                self.open.push(Entry { key: entry.key, node: entry.node, ring: entry.ring + 1 });
            }
            self.expand(entry.node, entry.ring);
        };
        let frontier = self.open.iter().map(|entry| { let node = &self.nodes[entry.node as usize]; node.g + node.lb }).min();
        let nearest = self.nodes.iter().min_by(|a, b| a.h.total_cmp(&b.h).then(a.lb.cmp(&b.lb))).map(|node| (node.state.clone(), node.g, node.anchor));
        // The returned sentence carries its tube: one more replay, recorded.
        let p = &self.problem;
        let sentence = self.best.clone().and_then(|(steps, _)| certify(p.timeline, &self.programs, &p.start, &steps, &p.goal, p.a, true).ok().map(|certificate| (steps, certificate)));
        Outcome { sentence, lower, frontier, stopped, seconds: self.seconds(), nearest }
    }
}
