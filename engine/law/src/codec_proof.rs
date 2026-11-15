//! Question-bound read certificates. Qualification reads once; edits recheck only
//! reverse-indexed dependencies at their original ticks. No motion is inferred.
use std::{collections::{BTreeMap, BTreeSet}, marker::PhantomData, sync::Arc};
use crate::{BlockPos, PlayerState, World, footprint::{EnvironmentRead, Footprint}, pattern::PatternError};
use super::{Canonical, ClassId, Interner, SCHEMA};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal { Coverage, Loaded, Schema, StateOrContext, Version, Question, Changed, UnknownCertificate, Law(PatternError) }

/// The adapter must answer at the requested tick and refuse incomplete coverage.
/// Implementations may read palettes/generators; this interface owns no terrain.
pub trait Source {
    fn cell(&self, tick: u64, position: BlockPos, mask: u8) -> Result<Canonical, Refusal>;
    fn environment(&self, tick: u64, query: &EnvironmentRead) -> Result<Canonical, Refusal>;
}

/// Qualified immutable-world adapter. Geometry and loaded evidence stay separate:
/// `World::loaded` is still read; coverage alone never fabricates loaded=true.
pub struct SnapshotSource<'a, W> {
    pub world: &'a W,
    pub ticks: (u64, u64),
    pub min: BlockPos,
    pub max: BlockPos,
}
impl<W: World> SnapshotSource<'_, W> {
    fn covered(&self, tick: u64, p: BlockPos) -> bool {
        self.ticks.0 <= tick && tick < self.ticks.1 && (0..3).all(|a| self.min[a] <= p[a] && p[a] <= self.max[a])
    }
}
impl<W: World> Source for SnapshotSource<'_, W> {
    fn cell(&self, tick: u64, p: BlockPos, mask: u8) -> Result<Canonical, Refusal> {
        if !self.covered(tick, p) { return Err(Refusal::Coverage); }
        super::cell(self.world.block(p), self.world.loaded(p), mask).map_err(Refusal::Law)
    }
    fn environment(&self, tick: u64, query: &EnvironmentRead) -> Result<Canonical, Refusal> {
        if tick < self.ticks.0 || tick >= self.ticks.1 { return Err(Refusal::Coverage); }
        if let EnvironmentRead::ExtraCollisions { bounds, .. } = query {
            for p in [bounds.min, bounds.max] {
                if !self.covered(tick, [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32]) { return Err(Refusal::Coverage); }
            }
        }
        super::environment(self.world, query).map_err(Refusal::Law)
    }
}

/// Exact anchor of a read claim. The word/kit/context are bytes, not fingerprints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    state: Box<[u64]>,
    word_context: Box<[u8]>,
    schema: Box<[u8]>,
    version: u64,
}
impl Binding {
    pub fn new(state: &PlayerState, word_context: &[u8], version: u64) -> Self {
        Self { state: state.bits().into(), word_context: word_context.into(), schema: SCHEMA.into(), version }
    }
}

mod sealed { pub trait Question {} }
pub trait Question: sealed::Question {}
/// Only equality of this recorded read set; it is not a path/dispatch proof.
#[derive(Clone, Debug)]
pub struct ReadsHold;
impl sealed::Question for ReadsHold {}
impl Question for ReadsHold {}
/// A different predicate, deliberately not interchangeable with ReadsHold.
#[derive(Clone, Debug)]
pub struct TubeEmpty;
impl sealed::Question for TubeEmpty {}
impl Question for TubeEmpty {}

#[derive(Clone, Debug)]
pub enum Answer<Q: Question> { Yes(Proof<Q>), No(Proof<Q>), Unknown(Refusal) }
/// Constructors are private. Even a proof of No says only No to its own question.
///
/// ```compile_fail
/// use law::codec::proof::{Answer, ReadsHold, TubeEmpty};
/// fn wrong(answer: Answer<ReadsHold>) -> Answer<TubeEmpty> { answer }
/// ```
/// ```compile_fail
/// use law::codec::proof::{Proof, ReadsHold};
/// let forged = Proof::<ReadsHold> { owner: std::sync::Arc::new(()), certificate: 0, revision: 0, question: std::marker::PhantomData };
/// ```
#[derive(Clone, Debug)]
pub struct Proof<Q: Question> { owner: Arc<()>, certificate: usize, revision: u64, question: PhantomData<Q> }

#[derive(Clone, Debug)]
pub struct CellRead { pub tick: u64, pub position: BlockPos, pub mask: u8, pub answer: ClassId }
#[derive(Clone, Debug)]
struct EnvRead { tick: u64, query: EnvironmentRead, answer: ClassId }
struct Certificate { binding: Binding, cells: Vec<CellRead>, environment: Vec<EnvRead>, invalid: Option<Refusal>, revision: u64 }

#[derive(Default)]
pub struct ReadCertificates {
    owner: Arc<()>,
    codec_owner: Option<Arc<()>>,
    records: Vec<Certificate>,
    cells: BTreeMap<BlockPos, BTreeMap<u64, BTreeSet<(usize, usize)>>>,
    environment: BTreeMap<u64, BTreeSet<(usize, usize)>>,
}
impl ReadCertificates {
    /// The footprint must come from the native law recording on this source.
    /// A new scope is qualified once; callers cannot construct a proof directly.
    pub fn capture(&mut self, codec: &mut Interner, binding: Binding, reads: &[(u64, &Footprint)], source: &impl Source) -> Result<usize, Refusal> {
        if self.codec_owner.as_ref().is_some_and(|owner| !Arc::ptr_eq(owner, &codec.owner)) { return Err(Refusal::Schema); }
        let mut cells = Vec::new();
        let mut environment = Vec::new();
        for &(tick, footprint) in reads {
            for (&position, &mask) in &footprint.reads {
                let bytes = source.cell(tick, position, mask)?;
                let answer = codec.intern(bytes.bytes()).ok_or(Refusal::Law(PatternError::TableFull))?;
                cells.push(CellRead { tick, position, mask, answer });
            }
            for query in &footprint.environment {
                // Global scan hints are not law dependencies: their local candidate
                // cells were recorded even when the optimized scan was skipped.
                if matches!(query, EnvironmentRead::HasFluid(_) | EnvironmentRead::HasInsideEffects(_)) { continue; }
                let bytes = source.environment(tick, query)?;
                let answer = codec.intern(bytes.bytes()).ok_or(Refusal::Law(PatternError::TableFull))?;
                environment.push(EnvRead { tick, query: query.clone(), answer });
            }
        }
        let id = self.records.len();
        self.codec_owner = Some(codec.owner.clone());
        for (index, read) in cells.iter().enumerate() { self.cells.entry(read.position).or_default().entry(read.tick).or_default().insert((id, index)); }
        for (index, read) in environment.iter().enumerate() { self.environment.entry(read.tick).or_default().insert((id, index)); }
        self.records.push(Certificate { binding, cells, environment, invalid: None, revision: 0 });
        Ok(id)
    }
    pub fn cells(&self, id: usize) -> Option<&[CellRead]> { Some(&self.records.get(id)?.cells) }
    pub fn answer(&self, id: usize, binding: &Binding) -> Answer<ReadsHold> {
        let Some(record) = self.records.get(id) else { return Answer::Unknown(Refusal::UnknownCertificate) };
        if record.binding.schema != binding.schema { return Answer::Unknown(Refusal::Schema); }
        if record.binding.version != binding.version { return Answer::Unknown(Refusal::Version); }
        if record.binding != *binding { return Answer::Unknown(Refusal::StateOrContext); }
        let proof = Proof { owner: self.owner.clone(), certificate: id, revision: record.revision, question: PhantomData };
        match &record.invalid { None => Answer::Yes(proof), Some(Refusal::Changed) => Answer::No(proof), Some(r) => Answer::Unknown(r.clone()) }
    }
    /// This is the atomic read-proof consumption boundary; no source tick scan.
    pub fn validate(&self, proof: &Proof<ReadsHold>, expected: usize, binding: &Binding) -> Result<(), Refusal> {
        if !Arc::ptr_eq(&self.owner, &proof.owner) { return Err(Refusal::UnknownCertificate); }
        if proof.certificate != expected { return Err(Refusal::Question); }
        let record = self.records.get(proof.certificate).ok_or(Refusal::UnknownCertificate)?;
        if record.invalid.is_some() || record.revision != proof.revision { return Err(record.invalid.clone().unwrap_or(Refusal::Changed)); }
        match self.answer(proof.certificate, binding) { Answer::Yes(_) => Ok(()), Answer::Unknown(r) => Err(r), Answer::No(_) => Err(Refusal::Changed) }
    }
    /// Publish after changing source contents. Only this cell's claims at affected
    /// recorded ticks are re-read, including previously un-evented native reads.
    pub fn edit(&mut self, codec: &Interner, position: BlockPos, ticks: (u64, u64), source: &impl Source) -> Vec<usize> {
        let affected: BTreeSet<_> = self.cells.get(&position).into_iter().flat_map(|at| at.range(ticks.0..ticks.1))
            .flat_map(|(_, ids)| ids.iter().copied()).collect();
        let mut broken = Vec::new();
        for (id, index) in affected {
            let record = &mut self.records[id];
            if record.invalid.is_some() { continue; }
            if !self.codec_owner.as_ref().is_some_and(|owner| Arc::ptr_eq(owner, &codec.owner)) {
                record.invalid = Some(Refusal::Schema); record.revision += 1; broken.push(id); continue;
            }
            let read = &record.cells[index];
            let result = source.cell(read.tick, read.position, read.mask)
                .and_then(|bytes| if codec.find(bytes.bytes()) == Some(read.answer) { Ok(()) } else { Err(Refusal::Changed) });
            if let Err(reason) = result { record.invalid = Some(reason); record.revision += 1; broken.push(id); }
        }
        broken
    }
    pub fn environment_event(&mut self, codec: &Interner, ticks: (u64, u64), source: &impl Source) -> Vec<usize> {
        let mut broken = Vec::new();
        let affected: BTreeSet<_> = self.environment.range(ticks.0..ticks.1).flat_map(|(_, ids)| ids.iter().copied()).collect();
        for (id, index) in affected {
            let record = &mut self.records[id];
            if record.invalid.is_some() { continue; }
            if !self.codec_owner.as_ref().is_some_and(|owner| Arc::ptr_eq(owner, &codec.owner)) {
                record.invalid = Some(Refusal::Schema); record.revision += 1; broken.push(id); continue;
            }
            let read = &record.environment[index];
            let result = source.environment(read.tick, &read.query)
                .and_then(|bytes| if codec.find(bytes.bytes()) == Some(read.answer) { Ok(()) } else { Err(Refusal::Changed) });
            if let Err(reason) = result { record.invalid = Some(reason); record.revision += 1; broken.push(id); }
        }
        broken
    }
}
