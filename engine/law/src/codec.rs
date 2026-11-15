//! Canonical answers are interned by bytes; a digest only selects a bucket.
use std::collections::HashMap;
use crate::{Block, FluidKind, Inside, footprint::Query, pattern::{self, PatternError}};
#[path = "codec_state.rs"]
mod state;
pub use state::state_answers;
#[path = "codec_proof.rs"]
pub mod proof;
#[path = "codec_planes.rs"]
pub mod planes;

pub const SCHEMA: &[u8] = b"law-role-codec/1";

/// Exact content is the equality witness. Fingerprints are only bucket addresses.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Canonical(pub(crate) Box<[u8]>);
impl Canonical {
    pub fn bytes(&self) -> &[u8] { &self.0 }
    pub fn fingerprint(&self) -> u128 { digest(&self.0) }
}

pub fn state_bytes(words: &[u64]) -> Vec<u8> {
    state_key(StateQuestion::Comparisons, words)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StateQuestion { Exact, Comparisons, Recurrence }

pub fn state_key(question: StateQuestion, words: &[u64]) -> Vec<u8> {
    let mut bytes = vec![b's', question as u8];
    bytes.extend(SCHEMA);
    bytes.extend((words.len() as u64).to_le_bytes());
    for word in words { bytes.extend(word.to_le_bytes()); }
    bytes
}

/// The same environment schema for footprints and read certificates. Scan hints
/// are represented locally by the footprint's candidate-cell answers instead.
pub fn environment(world: &impl crate::World, query: &crate::footprint::EnvironmentRead) -> Result<Canonical, PatternError> {
    use crate::footprint::EnvironmentRead as E;
    let mut bytes = SCHEMA.to_vec();
    match query {
        E::MinY(_) => return Ok(environment_mask(world, 1, [0; 3])),
        E::FastLava(_) => return Ok(environment_mask(world, 2, [0; 3])),
        E::Difficulty(_) => return Ok(environment_mask(world, 4, [0; 3])),
        E::NaturalRegeneration(_) => return Ok(environment_mask(world, 8, [0; 3])),
        E::HasFluid(_) | E::HasInsideEffects(_) => bytes.push(4),
        E::ExtraCollisions { bounds, context, .. } => {
            bytes.push(5);
            let shapes = world.extra_collisions(*bounds, *context);
            bytes.extend((shapes.len() as u64).to_le_bytes());
            for shape in shapes { pattern::encode_shape(&mut bytes, &shape, pattern::FRAMES[0])?; }
        }
    }
    Ok(Canonical(bytes.into()))
}

pub fn environment_mask(world: &impl crate::World, mask: u8, origin: crate::BlockPos) -> Canonical {
    let mut bytes = SCHEMA.to_vec();
    bytes.extend([b'e', mask]);
    if mask & 1 != 0 { bytes.extend((world.min_y() as i64 - origin[1] as i64).to_le_bytes()); }
    if mask & 2 != 0 { bytes.push(world.fast_lava() as u8); }
    if mask & 4 != 0 { bytes.push(world.difficulty()); }
    if mask & 8 != 0 { bytes.push(world.natural_regeneration() as u8); }
    Canonical(bytes.into())
}

/// The role is part of the question, even when two roles have the same mask.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum LegsRole { Support02, Support05, BodyFeet, Inside }

impl LegsRole {
    pub fn mask(self) -> u8 {
        let hooks_loaded = Query::Hooks as u8 | Query::Loaded as u8;
        match self {
            Self::Support02 | Self::Support05 => hooks_loaded,
            Self::BodyFeet => hooks_loaded | Query::Shape as u8 | Query::Fluid as u8,
            Self::Inside => hooks_loaded | Query::Fluid as u8,
        }
    }
}

/// The precise legs question and its canonical law answers.
pub fn legs_bytes(block: &Block, loaded: bool, role: LegsRole) -> Result<Vec<u8>, PatternError> {
    let mut bytes = vec![b'L', role as u8];
    bytes.extend(cell(&legs_model(block), loaded, role.mask())?.bytes());
    Ok(bytes)
}

pub fn cell(block: &Block, loaded: bool, mask: u8) -> Result<Canonical, PatternError> {
    Ok(Canonical(pattern::encode_cell(block, loaded, mask)?.into()))
}

fn legs_model(block: &Block) -> Block {
    let mut block = block.clone();
    block.world_hook = false;
    if block.inside == Inside::None && block.stuck_multiplier.is_none() { block.air = false; }
    if block.fluid.kind == FluidKind::Empty { block.fluid_flow = None; }
    block
}

pub fn legs_presence(block: &Block) -> Result<Canonical, PatternError> {
    let mut bytes = vec![b'p'];
    bytes.extend(cell(&legs_model(block), true, Query::Block as u8 | Query::Loaded as u8)?.bytes());
    Ok(Canonical(bytes.into()))
}

/// Unported verbs retain fine state and registry identity, and never certify a capability.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum UnportedVerb { Break, Place }

pub fn unknown_verb_bytes(verb: UnportedVerb, registry_hash: &[u8], fine_state: &[u8], kit: &[u8]) -> Vec<u8> {
    let mut bytes = vec![b'U', verb as u8];
    for part in [registry_hash, fine_state, kit] {
        bytes.extend((part.len() as u64).to_le_bytes());
        bytes.extend(part);
    }
    bytes
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Projection { Known(ClassId), Unknown(ClassId) }

/// Inputs to the future generator-store key. Each variable-length component is framed.
pub struct Label<'a> {
    pub law_schema: &'a [u8],
    pub body_kit: &'a [u8],
    pub support: &'a [ClassId],
    pub medium: &'a [ClassId],
}

/// Fine IDs belong to the source's pinned registry, not to the legs model table.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FineId(pub u32);

/// A kit swaps this table. The source keeps one fine terrain; only query answers
/// are cached here. Context-dependent shapes remain in the canonical model bytes.
pub struct ProjectionTable {
    owner: Option<std::sync::Arc<()>>,
    registry: Box<[u8]>,
    kit: Box<[u8]>,
    legs: HashMap<(FineId, bool, LegsRole), ClassId>,
    verbs: HashMap<(FineId, UnportedVerb), Projection>,
}
impl ProjectionTable {
    pub fn new(registry: &[u8], kit: &[u8]) -> Self {
        Self { owner: None, registry: registry.into(), kit: kit.into(), legs: HashMap::new(), verbs: HashMap::new() }
    }
    fn bind(&mut self, codec: &Interner) -> bool {
        if let Some(owner) = &self.owner { std::sync::Arc::ptr_eq(owner, &codec.owner) }
        else { self.owner = Some(codec.owner.clone()); true }
    }
    pub fn legs(&mut self, codec: &mut Interner, fine: FineId, model: &Block, loaded: bool, role: LegsRole) -> Result<ClassId, PatternError> {
        if !self.bind(codec) { return Err(PatternError::CodecMismatch); }
        if let Some(&id) = self.legs.get(&(fine, loaded, role)) { return Ok(id); }
        let id = codec.legs(model, loaded, role)?;
        self.legs.insert((fine, loaded, role), id);
        Ok(id)
    }
    pub fn verb(&mut self, codec: &mut Interner, fine: FineId, verb: UnportedVerb) -> Option<Projection> {
        if !self.bind(codec) { return None; }
        if let Some(&answer) = self.verbs.get(&(fine, verb)) { return Some(answer); }
        let answer = codec.unknown_verb(verb, &self.registry, &fine.0.to_le_bytes(), &self.kit)?;
        self.verbs.insert((fine, verb), answer);
        Some(answer)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ClassId(pub(crate) u16);
impl ClassId { pub fn index(self) -> u16 { self.0 } }

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LabelId(pub(crate) u32);
impl LabelId { pub fn index(self) -> u32 { self.0 } }

pub struct Interner {
    pub(crate) owner: std::sync::Arc<()>,
    hash: fn(&[u8]) -> u128,
    classes: Vec<Box<[u8]>>,
    digests: Vec<u128>,
    buckets: HashMap<u128, Vec<ClassId>>,
    labels: Vec<Box<[u8]>>,
    label_buckets: HashMap<u128, Vec<LabelId>>,
}
impl Default for Interner { fn default() -> Self { Self::with_digest(digest) } }

pub fn digest(bytes: &[u8]) -> u128 {
    bytes.iter().fold(0x6c62272e07bb014262b821756295c58d, |hash, &byte|
        (hash ^ byte as u128).wrapping_mul(0x0000000001000000000000000000013b))
}

impl Interner {
    /// Alternate bucket function for qualification, including forced collisions.
    pub fn with_digest(hash: fn(&[u8]) -> u128) -> Self {
        Self { owner: std::sync::Arc::new(()), hash, classes: Vec::new(), digests: Vec::new(), buckets: HashMap::new(), labels: Vec::new(), label_buckets: HashMap::new() }
    }
    pub fn find(&self, bytes: &[u8]) -> Option<ClassId> { self.find_in((self.hash)(bytes), bytes) }
    fn find_in(&self, hash: u128, bytes: &[u8]) -> Option<ClassId> {
        self.buckets.get(&hash)?.iter().copied().find(|id| &*self.classes[id.0 as usize] == bytes)
    }
    pub fn intern(&mut self, bytes: &[u8]) -> Option<ClassId> { self.intern_in((self.hash)(bytes), bytes) }
    fn intern_in(&mut self, hash: u128, bytes: &[u8]) -> Option<ClassId> {
        if let Some(id) = self.find_in(hash, bytes) { return Some(id); }
        let id = ClassId(u16::try_from(self.classes.len()).ok().filter(|&n| n != u16::MAX)?);
        self.classes.push(bytes.into());
        self.digests.push(hash);
        self.buckets.entry(hash).or_default().push(id);
        Some(id)
    }
    pub fn digests(&self) -> &[u128] { &self.digests }
    pub fn legs(&mut self, block: &Block, loaded: bool, role: LegsRole) -> Result<ClassId, PatternError> {
        self.intern(&legs_bytes(block, loaded, role)?).ok_or(PatternError::TableFull)
    }
    pub fn unknown_verb(&mut self, verb: UnportedVerb, registry_hash: &[u8], fine_state: &[u8], kit: &[u8]) -> Option<Projection> {
        self.intern(&unknown_verb_bytes(verb, registry_hash, fine_state, kit)).map(Projection::Unknown)
    }
    /// Only the already-ported progress query is Known; this does not answer the
    /// unported harvest/drop/restoration questions and cannot execute Break.
    pub fn mining_progress(&mut self, sample: crate::mining::MiningSample, kit: &[u8]) -> Option<Projection> {
        let mut bytes = SCHEMA.to_vec();
        bytes.push(b'm');
        bytes.extend((kit.len() as u64).to_le_bytes()); bytes.extend(kit);
        bytes.extend(sample.destroy_speed.to_bits().to_le_bytes());
        bytes.extend(sample.hardness.to_bits().to_le_bytes());
        bytes.push(sample.correct_tool as u8);
        bytes.extend(sample.progress().to_bits().to_le_bytes());
        self.intern(&bytes).map(Projection::Known)
    }
    pub fn label(&mut self, label: Label<'_>) -> Option<LabelId> {
        let mut bytes = Vec::new();
        for part in [label.law_schema, label.body_kit] {
            bytes.extend((part.len() as u64).to_le_bytes());
            bytes.extend(part);
        }
        for classes in [label.support, label.medium] {
            bytes.extend((classes.len() as u64).to_le_bytes());
            for id in classes { bytes.extend(id.0.to_le_bytes()); }
        }
        self.intern_label(&bytes)
    }
    fn intern_label(&mut self, bytes: &[u8]) -> Option<LabelId> {
        let hash = (self.hash)(bytes);
        if let Some(ids) = self.label_buckets.get(&hash) {
            if let Some(id) = ids.iter().copied().find(|id| &*self.labels[id.0 as usize] == bytes) { return Some(id); }
        }
        let id = LabelId(u32::try_from(self.labels.len()).ok()?);
        self.labels.push(bytes.into());
        self.label_buckets.entry(hash).or_default().push(id);
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collision_compares_bytes() {
        let mut table = Interner::default();
        let a = table.intern_in(0, b"a").unwrap();
        let b = table.intern_in(0, b"b").unwrap();
        assert_ne!(a, b);
        assert_eq!(table.intern_in(0, b"a"), Some(a));
        assert_eq!(table.find_in(0, b"b"), Some(b));
    }

    #[test]
    fn roles_and_unknown_verbs_keep_their_questions() {
        let mut table = Interner::default();
        let air = Block::default();
        let mut inert = air.clone();
        inert.air = false;
        inert.world_hook = true;
        for role in [LegsRole::Support02, LegsRole::Support05, LegsRole::BodyFeet, LegsRole::Inside] {
            assert_eq!(table.legs(&air, true, role), table.legs(&inert, true, role));
        }
        assert_ne!(table.legs(&air, true, LegsRole::Support02), table.legs(&air, true, LegsRole::Support05));
        let first = table.unknown_verb(UnportedVerb::Break, b"registry", b"stone", b"silk");
        assert!(matches!(first, Some(Projection::Unknown(_))));
        assert_ne!(first, table.unknown_verb(UnportedVerb::Place, b"registry", b"stone", b"silk"));
        assert_ne!(first, table.unknown_verb(UnportedVerb::Break, b"registry", b"grass", b"silk"));
        assert_ne!(first, table.unknown_verb(UnportedVerb::Break, b"registry", b"stone", b"bare"));
    }
}
