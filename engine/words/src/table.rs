//! The proposal table (vocabulary.md, sections 2, 3 and 11). The class proposes; the
//! replay from the concrete state certifies. Proposals live under the class alone; the
//! world part orders them and never excludes one. Nothing is filed by its exact start.
use std::collections::{HashMap, HashSet};
use law::{PlayerState, World, pattern::{Key, PatternError, PatternTable}, state_reads::StateReads};
use crate::{Arguments, Exit, Program, Segment, SegmentError, class::{self, ClassOf}, run_segment};

/// How a replay ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Outcome { pub exit: Exit, pub ticks: u32 }
impl Outcome { pub fn of(segment: &Segment) -> Self { Self { exit: segment.exit, ticks: segment.ticks } } }

#[derive(Clone, Debug)]
pub struct Proposal {
    /// An index into the table's programs.
    pub program: u32,
    pub arguments: Arguments,
    /// Each outcome its replays reached, in the order first seen, with its count.
    pub outcomes: Vec<(Outcome, u32)>,
    /// The world part: the pattern key of the cells its last filed replay read.
    pub world: Key,
    stamp: u32,
    class: u32,
    pub replays: u64,
    /// The times a consumer took it, and the ticks it carried then (Zipf, section 11).
    pub walked: u64,
    pub walked_ticks: u64,
}
impl Proposal {
    /// The most frequent outcome; the earliest seen wins a tie.
    pub fn predicted(&self) -> Outcome {
        self.outcomes.iter().fold(self.outcomes[0], |best, &seen| if seen.1 > best.1 { seen } else { best }).0
    }
    pub fn class(&self) -> u32 { self.class }
}
/// A proposal of the state's class. `here`: its world part holds at the state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found { pub id: u32, pub here: bool }

/// A replayed segment. Its footprint is the certificate; a key is computed only if asked.
#[derive(Clone, Debug)]
pub struct Certified {
    pub segment: Segment,
    /// The replay ended as predicted: the same exit on the same tick.
    pub as_proposed: bool,
}
impl Certified {
    /// The pattern key of what the replay read from its exact start, filed in `store`.
    pub fn key<V>(&self, world: &impl World, start: &PlayerState, store: &mut PatternTable<V>, value: V) -> Result<Key, PatternError> {
        store.insert(world, &self.segment.footprint, start, value)
    }
}
/// A surprise's witness: what replays it exactly, on the world it happened on.
#[derive(Clone, Debug)]
pub struct Surprise { pub start: PlayerState, pub proposal: u32, pub program: u32, pub arguments: Arguments, pub predicted: Outcome, pub actual: Outcome }
impl Surprise {
    /// One line: the start's bits in hex, the program, the arguments, both outcomes.
    pub fn line(&self, table: &ProposalTable) -> String {
        let bits = self.start.bits().iter().map(|word| format!("{word:x}")).collect::<Vec<_>>().join(",");
        format!("start={bits}\tprogram={:?}\targuments={:?}\tpredicted={:?}\tactual={:?}", table.program(self.program), self.arguments, self.predicted, self.actual)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    /// Lookups; those that found a proposal, and those whose classes had none.
    pub decisions: u64,
    pub hits: u64,
    pub misses: u64,
    /// Proposals handed out, and proposals replayed.
    pub returned: u64,
    pub replayed: u64,
    pub surprises: u64,
    pub refusals: u64,
    /// Segments run by `invent`, and the proposals ever made.
    pub invented: u64,
    pub learned: u64,
    /// Hits audited, the ticks a better piece would have saved, and the exits no hit reached.
    pub regret_samples: u64,
    pub regret_ticks: u64,
    pub regret_unseen: u64,
}
#[derive(Clone, Debug, Default)]
pub struct ClassRecord {
    pub words: Box<[u64]>,
    pub proposals: Vec<u32>,
    pub decisions: u64,
    pub replayed: u64,
    pub surprises: u64,
    pub refusals: u64,
    pub regret_samples: u64,
    pub regret_ticks: u64,
}

/// One run of the compile step.
#[derive(Clone, Debug)]
pub struct Piece { pub program: usize, pub arguments: Arguments, pub segment: Segment }
#[derive(Clone, Debug, Default)]
pub struct Pieces {
    /// The distinct pieces: one per exit state, under the first program that reached it.
    pub pieces: Vec<Piece>,
    pub runs: u32,
    pub refusals: u32,
    pub duplicates: u32,
}
/// The record's compile step (section 5): from a state, every program at every aim to
/// its event, each certified by the run itself. A run the law refuses, by an error or by
/// `unimplemented!`, is a refusal. Pieces that reach the same bits are one piece.
pub fn pieces<W: World>(world: &W, state: &PlayerState, programs: &[Program], arguments: &[Arguments]) -> Pieces {
    let mut found = Pieces::default();
    let mut seen = HashSet::new();
    for &arguments in arguments {
        for (program, rows) in programs.iter().enumerate() {
            found.runs += 1;
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_segment(world, state, rows, arguments)));
            match run {
                Ok(Ok(segment)) => if seen.insert(segment.state.bits()) { found.pieces.push(Piece { program, arguments, segment }); } else { found.duplicates += 1; },
                _ => found.refusals += 1,
            }
        }
    }
    found
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Regret { pub ticks: u32, pub unseen: u32 }

pub struct ProposalTable {
    class_of: ClassOf,
    programs: Vec<Program>,
    /// The read masks seen: a state has one class under each.
    masks: Vec<StateReads>,
    ids: HashMap<Box<[u64]>, u32>,
    classes: Vec<ClassRecord>,
    proposals: Vec<Proposal>,
    /// Class and cells -> the proposals whose world part that is.
    worlds: PatternTable<Vec<(u32, u32)>>,
    stamps: u32,
    /// Whether lookups consult the world part to order proposals, and replays keep it
    /// current. True is the record; false is there to price the ordering.
    pub ordered: bool,
    pub counters: Counters,
    /// New proposals in each thousand decisions, in order (Heaps, section 11).
    pub growth: Vec<u32>,
    /// Every surprise, never suppressed. The consumer drains and writes them.
    pub witnesses: Vec<Surprise>,
}
impl Default for ProposalTable { fn default() -> Self { Self::new(class::class) } }
impl ProposalTable {
    pub fn new(class_of: ClassOf) -> Self {
        Self { class_of, programs: Vec::new(), masks: Vec::new(), ids: HashMap::new(), classes: Vec::new(), proposals: Vec::new(),
            worlds: PatternTable::default(), stamps: 0, ordered: true, counters: Counters::default(), growth: Vec::new(), witnesses: Vec::new() }
    }
    pub fn program(&self, index: u32) -> &Program { &self.programs[index as usize] }
    pub fn proposal(&self, id: u32) -> &Proposal { &self.proposals[id as usize] }
    pub fn all(&self) -> &[Proposal] { &self.proposals }
    pub fn classes(&self) -> &[ClassRecord] { &self.classes }
    pub fn masks(&self) -> &[StateReads] { &self.masks }
    /// The footprint forms filed, over every class: what a lookup has to match.
    pub fn forms(&self) -> usize { self.worlds.entries().count() }

    /// Files `id`'s world part: the cells `segment` read from `state`, under its class.
    fn file(&mut self, world: &impl World, state: &PlayerState, words: Box<[u64]>, segment: &Segment, id: u32) -> Result<(Key, u32), PatternError> {
        self.stamps += 1;
        let (stamp, proposals) = (self.stamps, &self.proposals);
        let filed = self.worlds.insert_at(world, &segment.footprint, class::start(state, words), |known| {
            // Whatever was filed here by a proposal that has since moved is dropped.
            let mut known = known.unwrap_or_default();
            known.retain(|&(other, at)| other != id && proposals.get(other as usize).is_some_and(|p| p.stamp == at));
            known.push((id, stamp));
            known
        });
        if filed.is_err() { self.counters.refusals += 1; }
        filed.map(|key| (key, stamp))
    }

    /// Records the piece `segment`, run from `state`, under its class: a new proposal, or
    /// one more outcome of the proposal the class already has for the program and aim.
    pub fn learn(&mut self, world: &impl World, state: &PlayerState, program: &Program, arguments: Arguments, segment: &Segment) -> Result<u32, PatternError> {
        let program = self.programs.iter().position(|known| known == program).unwrap_or_else(|| {
            self.programs.push(program.clone());
            self.programs.len() - 1
        }) as u32;
        let reads = segment.footprint.state_reads;
        let words = (self.class_of)(state, reads);
        let class = self.ids.get(&words).copied();
        let known = class.and_then(|class| self.classes[class as usize].proposals.iter().copied()
            .find(|&id| (self.proposals[id as usize].program, self.proposals[id as usize].arguments) == (program, arguments)));
        let id = known.unwrap_or(self.proposals.len() as u32);
        let (world, stamp) = self.file(world, state, words.clone(), segment, id)?;
        if !self.masks.contains(&reads) { self.masks.push(reads); }
        if let Some(id) = known {
            let proposal = &mut self.proposals[id as usize];
            (proposal.world, proposal.stamp) = (world, stamp);
            count(&mut proposal.outcomes, Outcome::of(segment));
            return Ok(id);
        }
        let class = class.unwrap_or_else(|| {
            self.classes.push(ClassRecord { words: words.clone(), ..ClassRecord::default() });
            self.ids.insert(words, self.classes.len() as u32 - 1);
            self.classes.len() as u32 - 1
        });
        self.classes[class as usize].proposals.push(id);
        self.proposals.push(Proposal { program, arguments, outcomes: vec![(Outcome::of(segment), 1)], world, stamp, class, replays: 0, walked: 0, walked_ticks: 0 });
        self.counters.learned += 1;
        if let Some(window) = self.growth.last_mut() { *window += 1; }
        Ok(id)
    }

    /// Every proposal of the state's classes, those whose world part holds here first.
    /// A miss is no proposal at all.
    pub fn proposals(&mut self, world: &impl World, state: &PlayerState) -> Vec<Found> {
        if self.counters.decisions % 1000 == 0 { self.growth.push(0); }
        self.counters.decisions += 1;
        let (mut found, mut rest) = (Vec::new(), Vec::new());
        for index in 0..self.masks.len() {
            let words = (self.class_of)(state, self.masks[index]);
            let Some(&class) = self.ids.get(&words) else { continue };
            let from = found.len();
            for hit in if self.ordered { self.worlds.lookup_at(world, &class::start(state, words)) } else { Vec::new() } {
                found.extend(hit.entry.value.iter().filter(|&&(id, stamp)| self.proposals[id as usize].stamp == stamp).map(|&(id, _)| Found { id, here: true }));
            }
            let record = &mut self.classes[class as usize];
            record.decisions += 1;
            rest.extend(record.proposals.iter().filter(|&&id| !found[from..].iter().any(|f| f.id == id)).map(|&id| Found { id, here: false }));
        }
        found.extend(rest);
        if found.is_empty() { self.counters.misses += 1; } else { self.counters.hits += 1; }
        self.counters.returned += found.len() as u64;
        found
    }

    /// The only way to a segment: the law runs from the concrete state. The outcome is
    /// counted on the proposal; a surprise leaves its witness. A proposal whose world part
    /// did not hold here, or that surprised, takes the cells this replay read.
    pub fn replay(&mut self, world: &impl World, state: &PlayerState, found: Found) -> Result<Certified, SegmentError> {
        let proposal = &self.proposals[found.id as usize];
        let (program, arguments, predicted, class) = (proposal.program, proposal.arguments, proposal.predicted(), proposal.class as usize);
        self.counters.replayed += 1;
        self.classes[class].replayed += 1;
        let segment = match run_segment(world, state, &self.programs[program as usize], arguments) {
            Ok(segment) => segment,
            Err(error) => { self.counters.refusals += 1; self.classes[class].refusals += 1; return Err(error); }
        };
        let actual = Outcome::of(&segment);
        let as_proposed = actual == predicted;
        if !as_proposed {
            self.counters.surprises += 1;
            self.classes[class].surprises += 1;
            self.witnesses.push(Surprise { start: state.clone(), proposal: found.id, program, arguments, predicted, actual });
        }
        if self.ordered && (!found.here || !as_proposed) {
            let words = self.classes[class].words.clone();
            match self.file(world, state, words, &segment, found.id) {
                Ok((world, stamp)) => (self.proposals[found.id as usize].world, self.proposals[found.id as usize].stamp) = (world, stamp),
                Err(error) => { self.classes[class].refusals += 1; return Err(error.into()); }
            }
        }
        let proposal = &mut self.proposals[found.id as usize];
        proposal.replays += 1;
        count(&mut proposal.outcomes, actual);
        Ok(Certified { segment, as_proposed })
    }

    /// The consumer took this proposal's piece: `ticks` of its route ride on it.
    pub fn walked(&mut self, id: u32, ticks: u32) {
        self.proposals[id as usize].walked += 1;
        self.proposals[id as usize].walked_ticks += ticks as u64;
    }

    /// Inventing a piece, the price of a miss: the compile step, each distinct piece learned.
    pub fn invent<W: World>(&mut self, world: &W, state: &PlayerState, programs: &[Program], arguments: &[Arguments]) -> (Pieces, Vec<Option<u32>>) {
        let found = pieces(world, state, programs, arguments);
        self.counters.invented += found.runs as u64;
        self.counters.refusals += found.refusals as u64;
        let ids = found.pieces.iter().map(|piece| self.learn(world, state, &programs[piece.program], piece.arguments, &piece.segment).ok()).collect();
        (found, ids)
    }

    /// The audit of a hit (section 11): invent from the same state, learn nothing, and
    /// count the ticks the best invented piece would have saved over the best certified
    /// one that reached the same exit, and the exits no certified piece reached.
    pub fn regret<W: World>(&mut self, world: &W, state: &PlayerState, hit: &[(Found, Certified)], programs: &[Program], arguments: &[Arguments]) -> (Regret, Pieces) {
        let invented = pieces(world, state, programs, arguments);
        let best = |exit: Exit, outcomes: &mut dyn Iterator<Item = Outcome>| outcomes.filter(|o| o.exit == exit).map(|o| o.ticks).min();
        let mut exits = Vec::<Exit>::new();
        for piece in &invented.pieces { if !exits.contains(&piece.segment.exit) { exits.push(piece.segment.exit); } }
        let mut regret = Regret::default();
        for exit in exits {
            let new = best(exit, &mut invented.pieces.iter().map(|p| Outcome::of(&p.segment))).unwrap();
            match best(exit, &mut hit.iter().map(|(_, c)| Outcome::of(&c.segment))) {
                Some(held) => regret.ticks += held.saturating_sub(new),
                None => regret.unseen += 1,
            }
        }
        self.counters.regret_samples += 1;
        self.counters.regret_ticks += regret.ticks as u64;
        self.counters.regret_unseen += regret.unseen as u64;
        if let Some((found, _)) = hit.first() {
            let record = &mut self.classes[self.proposals[found.id as usize].class as usize];
            record.regret_samples += 1;
            record.regret_ticks += regret.ticks as u64;
        }
        (regret, invented)
    }
}
fn count(outcomes: &mut Vec<(Outcome, u32)>, outcome: Outcome) {
    match outcomes.iter_mut().find(|seen| seen.0 == outcome) {
        Some(seen) => seen.1 += 1,
        None => outcomes.push((outcome, 1)),
    }
}
