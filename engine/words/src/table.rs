//! The proposal table: class -> proposals (vocabulary.md, section 2). A hit names
//! what to run; the replay from the concrete state gives the bits and the certificate.
use law::{PlayerState, World, pattern::{Key, PatternError, PatternTable}, state_reads::StateReads};
use crate::{Arguments, Exit, Grid, Program, Segment, SegmentError, run_segment};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Proposal {
    /// An index into the table's programs.
    pub program: u32,
    pub arguments: Arguments,
    pub exit: Exit,
    /// The least and the greatest ticks seen.
    pub ticks: [u32; 2],
}
/// A replayed segment and its certificate: the `law::pattern` key of what the replay
/// read, from the exact start, in the table's certificate store.
#[derive(Clone, Debug)]
pub struct Certified {
    pub segment: Segment,
    pub key: Key,
    /// The replay ended as proposed: the same exit, within the ticks seen.
    pub as_proposed: bool,
}

#[derive(Default)]
pub struct ProposalTable {
    pub grid: Grid,
    programs: Vec<Program>,
    /// The read masks seen: a class is taken under each.
    masks: Vec<StateReads>,
    proposals: PatternTable<Vec<Proposal>>,
    certificates: PatternTable<()>,
    /// Lookups that found a proposal, and that found none.
    pub hits: u64,
    pub misses: u64,
    /// Replays that ended as proposed, and that did not.
    pub confirmed: u64,
    pub surprised: u64,
    pub refusals: u64,
}
impl ProposalTable {
    pub fn new(grid: Grid) -> Self { Self { grid, ..Self::default() } }
    pub fn program(&self, index: u32) -> &Program { &self.programs[index as usize] }
    pub fn classes(&self) -> usize { self.proposals.starts().len() }
    pub fn masks(&self) -> &[StateReads] { &self.masks }

    /// Records what `segment`, run from `state`, proposes for its class.
    pub fn learn(&mut self, world: &impl World, state: &PlayerState, program: &Program, arguments: Arguments, segment: &Segment) -> Result<(), PatternError> {
        let index = self.programs.iter().position(|known| known == program).unwrap_or_else(|| {
            self.programs.push(program.clone());
            self.programs.len() - 1
        }) as u32;
        self.learn_proposal(world, state, Proposal { program: index, arguments, exit: segment.exit, ticks: [segment.ticks; 2] }, segment)
    }
    fn learn_proposal(&mut self, world: &impl World, state: &PlayerState, new: Proposal, segment: &Segment) -> Result<(), PatternError> {
        let reads = segment.footprint.state_reads;
        if !self.masks.contains(&reads) { self.masks.push(reads); }
        let result = self.proposals.insert_at(world, &segment.footprint, crate::class::start(state, reads, self.grid), |known| {
            let mut known = known.unwrap_or_default();
            match known.iter_mut().find(|p| (p.program, p.arguments, p.exit) == (new.program, new.arguments, new.exit)) {
                Some(p) => p.ticks = [p.ticks[0].min(new.ticks[0]), p.ticks[1].max(new.ticks[1])],
                None => known.push(new),
            }
            known
        });
        if result.is_err() { self.refusals += 1; }
        result.map(|_| ())
    }

    /// The proposals for the state's class where the world part matches here.
    pub fn proposals(&mut self, world: &impl World, state: &PlayerState) -> Vec<Proposal> {
        let mut found = Vec::<Proposal>::new();
        for &reads in &self.masks {
            for hit in self.proposals.lookup_at(world, &crate::class::start(state, reads, self.grid)) {
                for p in &hit.entry.value { if !found.contains(p) { found.push(*p); } }
            }
        }
        if found.is_empty() { self.misses += 1; } else { self.hits += 1; }
        found
    }

    /// The only way to a segment: the law runs from the concrete state. A replay that
    /// ends otherwise than proposed is learned, so the class proposes it next time.
    pub fn replay(&mut self, world: &impl World, state: &PlayerState, proposal: &Proposal) -> Result<Certified, SegmentError> {
        let run = run_segment(world, state, &self.programs[proposal.program as usize], proposal.arguments)
            .and_then(|segment| Ok((self.certificates.insert(world, &segment.footprint, state, ())?, segment)));
        let (key, segment) = match run {
            Ok(run) => run,
            Err(error) => { self.refusals += 1; return Err(error); }
        };
        let as_proposed = segment.exit == proposal.exit && (proposal.ticks[0]..=proposal.ticks[1]).contains(&segment.ticks);
        if as_proposed { self.confirmed += 1; } else {
            self.surprised += 1;
            self.learn_proposal(world, state, Proposal { exit: segment.exit, ticks: [segment.ticks; 2], ..*proposal }, &segment)?;
        }
        Ok(Certified { segment, key, as_proposed })
    }
}
