//! Pattern keys and the pattern table (map/detail/footprints.md, section 3). A cell enters
//! the key as the content of the queries that read it; a hit proposes reuse.
use std::collections::HashMap;
use crate::{Aabb, Block, BlockPos, CollisionContext, Inside, PlayerState, Vec3, VoxelShape, World};
use crate::footprint::{EnvironmentRead, Footprint, Query};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub quarter_turns: u8,
    pub mirrored: bool,
}

/// The identity first, then the other quarter turns, then the mirrored four. The key
/// keeps only the identity: the others fail bit-exact replay (world/tests/frames.rs).
pub const FRAMES: [Frame; 8] = {
    let mut frames = [Frame { quarter_turns: 0, mirrored: false }; 8];
    let mut i = 0;
    while i < 8 {
        frames[i] = Frame { quarter_turns: (i % 4) as u8, mirrored: i >= 4 };
        i += 1;
    }
    frames
};

impl Frame {
    /// Vectors: mirror x, then turn (x, z) to (-z, x).
    fn vector(self, mut x: f64, mut z: f64) -> (f64, f64) {
        if self.mirrored { x = -x; }
        for _ in 0..self.quarter_turns { (x, z) = (-z, x); }
        (x, z)
    }

    /// Points inside a cell, about its center.
    fn point(self, x: f64, z: f64) -> (f64, f64) {
        let (x, z) = if self.mirrored { (1.0 - x, z) } else { (x, z) };
        (0..self.quarter_turns).fold((x, z), |(x, z), _| (1.0 - z, x))
    }

    fn inverse_point(self, x: f64, z: f64) -> (f64, f64) {
        let (x, z) = (0..self.quarter_turns).fold((x, z), |(x, z), _| (z, 1.0 - x));
        if self.mirrored { (1.0 - x, z) } else { (x, z) }
    }

    /// Direction.get3DDataValue; vertical values stay.
    fn facing(self, facing: u8) -> u8 {
        let (x, z) = match facing { 2 => (0.0, -1.0), 3 => (0.0, 1.0), 4 => (-1.0, 0.0), 5 => (1.0, 0.0), _ => return facing };
        match self.vector(x, z) { (x, _) if x < 0.0 => 4, (x, _) if x > 0.0 => 5, (_, z) if z < 0.0 => 2, _ => 3 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternError {
    /// Entities or the world border collided: not block contents.
    ExternalCollider,
    /// The port refuses this hook, so its physics is unknown.
    UnsupportedHook,
    NonFiniteGeometry,
    /// More than 65,535 distinct contents.
    TableFull,
    CodecMismatch,
    UnknownQuery,
}

/// A whole model's content seen in `frame`, or None when the frame cannot carry it
/// exactly. Equal content gives an equal digest whatever the block's name.
pub fn model_digest(block: &Block, frame: Frame) -> Result<Option<u128>, PatternError> {
    let mut bytes = Vec::new();
    Ok(encode(&mut bytes, block, SHAPES | HOOKS, frame)?.then(|| fnv(&bytes)))
}

/// One cell's content under the queries in `mask`: what those queries return, nothing else.
pub fn cell_digest(block: &Block, loaded: bool, mask: u8) -> Result<crate::codec::Canonical, PatternError> {
    crate::codec::cell(block, loaded, mask)
}

/// Canonical bytes for a query; equality must compare these, not the digest alone.
pub fn cell_bytes(block: &Block, loaded: bool, mask: u8) -> Result<Vec<u8>, PatternError> {
    Ok(crate::codec::cell(block, loaded, mask)?.bytes().to_vec())
}

/// Compatibility presence API: exact bytes, never a bare 128-bit equality witness.
pub fn law_digest(block: &Block) -> Result<crate::codec::Canonical, PatternError> {
    crate::codec::legs_presence(block)
}

pub(crate) fn encode_cell(block: &Block, loaded: bool, mask: u8) -> Result<Vec<u8>, PatternError> {
    let q = |query: Query| mask & query as u8 != 0;
    let known = Query::ALL.iter().fold(0, |bits, &query| bits | query as u8);
    if mask & !known != 0 { return Err(PatternError::UnknownQuery); }
    // Exhaustive: adding a read kind cannot silently yield an empty answer.
    let groups = Query::ALL.iter().filter(|&&query| q(query)).fold(0, |groups, query| groups | match query {
        Query::Block => SHAPES | HOOKS, Query::Shape => SHAPES, Query::Fluid => FLUID, Query::Hooks => HOOKS, Query::Loaded => 0,
    });
    let mut bytes = crate::codec::SCHEMA.to_vec();
    bytes.extend([b'c', mask, (q(Query::Loaded) && loaded) as u8]);
    encode(&mut bytes, block, groups, FRAMES[0])?;
    Ok(bytes)
}

/// A footprint's form, shared by every entry with it: its cells relative to the start
/// block with the queries that read each, sorted, and the world values it read.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Shape {
    pub cells: Box<[([i32; 3], u8)]>,
    /// Bits: 1 MinY, 2 FastLava, 4 Difficulty, 8 NaturalRegeneration.
    pub environment: u8,
}

/// Content handles: ids are dense and in-memory; the digests behind them persist.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Key {
    pub start: u32,
    pub shape: u32,
    /// The environment's id, then each cell's id in the shape's order.
    pub ids: Box<[u16]>,
}

/// What an entry's start is keyed on: the start block, the words that stand for the
/// state, and the context for the extra-collider probe. `exact` is the bit-equal start;
/// the word compiler keys proposals on a class of states (map/detail/vocabulary.md, section 3).
#[derive(Clone, Debug)]
pub struct Start {
    pub question: crate::codec::StateQuestion,
    pub origin: BlockPos,
    pub words: Box<[u64]>,
    pub context: CollisionContext,
}
impl Start {
    pub fn exact(state: &PlayerState) -> Self {
        let origin = block_of(state.position);
        Self { question: crate::codec::StateQuestion::Exact, origin, words: start_bits(state, origin), context: CollisionContext::of(state) }
    }
}

#[derive(Clone, Debug)]
pub struct Entry<V> {
    /// The start block where the word was recorded.
    pub origin: BlockPos,
    /// Per axis, the coordinates the law touched: from the least cell to the greatest plus one.
    pub scope: [[i32; 2]; 3],
    /// The union of the empty extra-collider queries, relative to `origin`; re-queried on a hit.
    pub probe: Option<Aabb>,
    pub value: V,
}

pub struct Hit<'a, V> {
    pub entry: &'a Entry<V>,
    /// Query start block minus `origin`.
    pub shift: BlockPos,
    /// True when the shift keeps every coordinate in its binade, so the recorded bits,
    /// shifted, are the law's. False: the word must replay before reuse.
    pub exact: bool,
}

/// Seeded in-memory maps (std's RandomState) over ids, with equality on ids.
pub struct PatternTable<V> {
    codec: crate::codec::Interner,
    shapes: Vec<Shape>,
    shape_ids: HashMap<Shape, u32>,
    starts: Vec<Box<[u64]>>,
    start_ids: HashMap<crate::codec::StateClassId, u32>,
    shapes_from: HashMap<u32, Vec<u32>>,
    entries: HashMap<Key, Entry<V>>,
}

impl<V> Default for PatternTable<V> {
    fn default() -> Self {
        Self { codec: crate::codec::Interner::default(), shapes: Vec::new(), shape_ids: HashMap::new(),
            starts: Vec::new(), start_ids: HashMap::new(), shapes_from: HashMap::new(), entries: HashMap::new() }
    }
}

impl<V> PatternTable<V> {
    /// The sole content namespace owned by this store, also used by proposals/labels.
    pub fn codec(&self) -> &crate::codec::Interner { &self.codec }
    pub fn codec_mut(&mut self) -> &mut crate::codec::Interner { &mut self.codec }
    /// The persistent part: the content behind each id.
    pub fn digests(&self) -> &[u128] { self.codec.digests() }
    pub fn shapes(&self) -> &[Shape] { &self.shapes }
    pub fn starts(&self) -> &[Box<[u64]>] { &self.starts }
    pub fn entries(&self) -> impl Iterator<Item = (&Key, &Entry<V>)> { self.entries.iter() }

    /// Records a word run from `start` on the snapshot its footprint was recorded on.
    pub fn insert(&mut self, world: &impl World, footprint: &Footprint, start: &PlayerState, value: V) -> Result<Key, PatternError> {
        self.insert_at(world, footprint, Start::exact(start), |_| value)
    }

    /// The same, from any start key. `value` receives the entry already under the whole key.
    pub fn insert_at(&mut self, world: &impl World, footprint: &Footprint, start: Start,
        value: impl FnOnce(Option<V>) -> V) -> Result<Key, PatternError> {
        let origin = start.origin;
        let mut probe: Option<Aabb> = None;
        let mut environment = 0;
        for read in &footprint.environment {
            environment |= match read {
                EnvironmentRead::MinY(_) => 1, EnvironmentRead::FastLava(_) => 2,
                EnvironmentRead::Difficulty(_) => 4, EnvironmentRead::NaturalRegeneration(_) => 8,
                // Omission certificates: forcing them true changes no tick.
                EnvironmentRead::HasFluid(_) | EnvironmentRead::HasInsideEffects(_) => 0,
                EnvironmentRead::ExtraCollisions { bounds, shapes, .. } => {
                    if !shapes.is_empty() { return Err(PatternError::ExternalCollider); }
                    let b = Aabb { min: bounds.min.minus(corner(origin)), max: bounds.max.minus(corner(origin)) };
                    probe = Some(probe.map_or(b, |p| Aabb::new(p.min.x.min(b.min.x), p.min.y.min(b.min.y), p.min.z.min(b.min.z),
                        p.max.x.max(b.max.x), p.max.y.max(b.max.y), p.max.z.max(b.max.z))));
                    0
                }
            };
        }
        let shape = Shape { cells: footprint.reads.iter().map(|(p, &mask)| ([0, 1, 2].map(|a| p[a] - origin[a]), mask)).collect(), environment };
        let mut ids = Vec::with_capacity(shape.cells.len() + 1);
        let mut memo = Memo::default();
        ids.push(self.codec.intern(&environment_bytes(world, environment, origin)).ok_or(PatternError::TableFull)?.0);
        for &(offset, mask) in shape.cells.iter() {
            let p = [0, 1, 2].map(|a| origin[a] + offset[a]);
            ids.push(self.cell_id(world, p, mask, &mut memo, true)?.expect("an added content has an id"));
        }
        let mut scope = [[i32::MAX, i32::MIN]; 3];
        for p in footprint.reads.keys().chain(&footprint.swept) {
            for a in 0..3 { scope[a] = [scope[a][0].min(p[a]), scope[a][1].max(p[a] + 1)]; }
        }
        let shape = match self.shape_ids.get(&shape) {
            Some(&id) => id,
            None => { self.shapes.push(shape.clone()); self.shape_ids.insert(shape, self.shapes.len() as u32 - 1); self.shapes.len() as u32 - 1 }
        };
        let bits = start.words;
        let answer = self.codec.intern_state(&crate::codec::state_key(start.question, &bits)).ok_or(PatternError::TableFull)?;
        let start = match self.start_ids.get(&answer) {
            Some(&id) => id,
            None => { self.starts.push(bits); self.start_ids.insert(answer, self.starts.len() as u32 - 1); self.starts.len() as u32 - 1 }
        };
        let from = self.shapes_from.entry(start).or_default();
        if !from.contains(&shape) { from.push(shape); }
        let key = Key { start, shape, ids: ids.into() };
        let previous = self.entries.remove(&key).map(|entry| entry.value);
        self.entries.insert(key.clone(), Entry { origin, scope, probe, value: value(previous) });
        Ok(key)
    }

    /// The entry recorded from this start state whose footprint holds the same content
    /// here. Reads `world` at each candidate shape's cells, so it must cover them.
    pub fn lookup(&mut self, world: &impl World, start: &PlayerState) -> Option<Hit<'_, V>> {
        let start = Start::exact(start);
        let key = self.matching(world, &start, true).pop()?;
        Some(self.hit(&key, start.origin))
    }

    /// Every entry under this start key whose footprint holds the same content here:
    /// one per footprint form.
    pub fn lookup_at(&mut self, world: &impl World, start: &Start) -> Vec<Hit<'_, V>> {
        let keys = self.matching(world, start, false);
        keys.iter().map(|key| self.hit(key, start.origin)).collect()
    }

    fn hit(&self, key: &Key, origin: BlockPos) -> Hit<'_, V> {
        let entry = &self.entries[key];
        let shift = [0, 1, 2].map(|a| origin[a] - entry.origin[a]);
        Hit { entry, shift, exact: in_scope(entry.scope, shift) }
    }

    fn matching(&mut self, world: &impl World, start: &Start, first: bool) -> Vec<Key> {
        let origin = start.origin;
        let mut found = Vec::new();
        let Some(answer) = self.codec.find_state(&crate::codec::state_key(start.question, &start.words)) else { return found };
        let Some(&start_id) = self.start_ids.get(&answer) else { return found };
        let mut memo = Memo::default();
        'shapes: for index in 0..self.shapes_from[&start_id].len() {
            let shape_id = self.shapes_from[&start_id][index];
            let Some(id) = self.codec.find(&environment_bytes(world, self.shapes[shape_id as usize].environment, origin)).map(|id| id.0) else { continue };
            let mut ids = Vec::with_capacity(self.shapes[shape_id as usize].cells.len() + 1);
            ids.push(id);
            for cell in 0..self.shapes[shape_id as usize].cells.len() {
                let (offset, mask) = self.shapes[shape_id as usize].cells[cell];
                let p = [0, 1, 2].map(|a| origin[a] + offset[a]);
                let Ok(Some(id)) = self.cell_id(world, p, mask, &mut memo, false) else { continue 'shapes };
                ids.push(id);
            }
            let key = Key { start: start_id, shape: shape_id, ids: ids.into() };
            let Some(entry) = self.entries.get(&key) else { continue };
            if let Some(probe) = entry.probe {
                let bounds = Aabb { min: probe.min.plus(corner(origin)), max: probe.max.plus(corner(origin)) };
                if !world.extra_collisions(bounds, start.context).is_empty() { continue; }
            }
            found.push(key);
            if first { break; }
        }
        found
    }

    /// A cell's content id, or None for content the table has never held.
    fn cell_id(&mut self, world: &impl World, p: BlockPos, mask: u8, memo: &mut Memo, add: bool) -> Result<Option<u16>, PatternError> {
        let bytes = memo.bytes(world, p, mask)?;
        if add { Ok(Some(self.codec.intern(&bytes).ok_or(PatternError::TableFull)?.0)) }
        else { Ok(self.codec.find(&bytes).map(|id| id.0)) }
    }
}

/// Encodings by model address for one call: the borrowed world keeps addresses stable.
#[derive(Default)]
struct Memo(HashMap<(*const Block, u8, bool), Vec<u8>>);
impl Memo {
    fn bytes(&mut self, world: &impl World, p: BlockPos, mask: u8) -> Result<Vec<u8>, PatternError> {
        let (block, loaded) = (world.block(p), world.loaded(p));
        if let Some(bytes) = self.0.get(&(block as *const Block, mask, loaded)) { return Ok(bytes.clone()); }
        let bytes = cell_bytes(block, loaded, mask)?;
        self.0.insert((block, mask, loaded), bytes.clone());
        Ok(bytes)
    }
}

fn block_of(p: Vec3) -> BlockPos { [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32] }
fn corner(p: BlockPos) -> Vec3 { Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64) }

/// The start state's bits with its position and supporting block taken from the start
/// block. The tick counter `t` is left out: the law only counts it.
fn start_bits(start: &PlayerState, origin: BlockPos) -> Box<[u64]> {
    let mut state = start.clone();
    state.t = 0;
    state.yaw=0.0;state.pitch=0.0;state.input=crate::Input::default();
    state.position = state.position.minus(corner(origin));
    state.main_supporting_block = state.main_supporting_block.map(|p| [0, 1, 2].map(|a| p[a] - origin[a]));
    state.bits().into()
}

/// The world values a footprint read. MinY is relative to the start block.
fn environment_bytes(world: &impl World, environment: u8, origin: BlockPos) -> Vec<u8> {
    crate::codec::environment_mask(world, environment, origin).bytes().to_vec()
}

/// Every real in [lo, hi] has one sign and one exponent k, with k <= 23 so that an
/// integer is a multiple of the f32 ulp too (the law casts some coordinates to f32).
fn binade([lo, hi]: [i32; 2]) -> Option<(bool, u32)> {
    let (negative, small, large) = if lo >= 1 { (false, lo as i64, hi as i64) } else if hi <= -1 { (true, -(hi as i64), -(lo as i64)) } else { return None };
    let k = 63 - small.leading_zeros();
    (k <= 23 && large < 1 << (k + 1)).then_some((negative, k))
}

/// An integer shift is bit-exact while every coordinate keeps its binade.
pub fn in_scope(scope: [[i32; 2]; 3], shift: BlockPos) -> bool {
    (0..3).all(|a| shift[a] == 0 || binade(scope[a]).is_some_and(|b| binade([scope[a][0] + shift[a], scope[a][1] + shift[a]]) == Some(b)))
}

/// FNV-1a 128 with explicit constants, independent of std's Hash.
fn fnv(bytes: &[u8]) -> u128 {
    crate::codec::digest(bytes)
}

fn f64s(bytes: &mut Vec<u8>, values: impl IntoIterator<Item = f64>) { values.into_iter().for_each(|v| bytes.extend(v.to_bits().to_le_bytes())); }

/// Every grid cell, empty subdivisions included, as the law's planes see them.
pub(crate) fn encode_shape(bytes: &mut Vec<u8>, shape: &VoxelShape, frame: Frame) -> Result<bool, PatternError> {
    let mut rows = Vec::new();
    let mut exact = true;
    for (bounds, full) in shape.view().grid_cells() {
        let mut row = vec![full as u8];
        let mut corners = [(0.0, 0.0); 2];
        for (corner, p) in corners.iter_mut().zip([bounds.min, bounds.max]) {
            if !(p.x.is_finite() && p.y.is_finite() && p.z.is_finite()) { return Err(PatternError::NonFiniteGeometry); }
            *corner = frame.point(p.x, p.z);
            let back = frame.inverse_point(corner.0, corner.1);
            exact &= back.0.to_bits() == p.x.to_bits() && back.1.to_bits() == p.z.to_bits();
        }
        let [(ax, az), (bx, bz)] = corners;
        f64s(&mut row, [ax.min(bx), bounds.min.y, az.min(bz), ax.max(bx), bounds.max.y, az.max(bz)]);
        rows.push(row);
    }
    rows.sort();
    bytes.extend((rows.len() as u64).to_le_bytes());
    rows.iter().for_each(|row| bytes.extend(row));
    Ok(exact)
}

const SHAPES: u8 = 1;
const FLUID: u8 = 2;
/// Everything but the four collision shapes, the fluid included.
const HOOKS: u8 = 4;

/// The law's view of one block, for the field groups in `groups`. Collision shapes
/// enter as the law resolves them (World::collision_shape), so equal physics encodes equally.
fn encode(bytes: &mut Vec<u8>, block: &Block, groups: u8, frame: Frame) -> Result<bool, PatternError> {
    // Exhaustive, so a new law field forces a review of the key.
    let Block {
        shape, inside_shape, water_block, above_shape, below_shape, sneak_shape, inside_cell_shape, friction,
        speed_factor, jump_factor, climbable, scaffolding, ladder_facing, open_trapdoor_facing,
        fluid, fluid_flow, inside, grounded_contact, stuck_multiplier, unsupported_fall_on,
        fall_on_multiplier, bed, suffocating, bounce, suppresses_bounce, slime, fence,
        wall_or_gate, resets_fall_distance, end_portal_or_gateway, world_hook, air,
    } = block;
    let mut exact = true;
    bytes.push(groups);
    if groups & SHAPES != 0 {
        let above = above_shape.as_ref().unwrap_or(shape);
        let below = below_shape.as_ref().unwrap_or(shape);
        let sneak = sneak_shape.as_ref().unwrap_or(above);
        let inside_cell = inside_cell_shape.as_ref().unwrap_or(below);
        for shape in [shape, above, below, sneak, inside_cell] { exact &= encode_shape(bytes, shape, frame)?; }
        // Powder snow replaces its shapes by context.
        bytes.push((*inside == Inside::PowderSnow) as u8);
    }
    if groups & (FLUID | HOOKS) != 0 {
        let crate::Fluid { kind, height, flow, raw_amount, raw_falling } = *fluid;
        bytes.push(kind as u8);
        bytes.extend(height.to_bits().to_le_bytes());
        let (x, z) = frame.vector(flow.x, flow.z);
        // Signed zeros from negation are one direction.
        f64s(bytes, [x + 0.0, flow.y + 0.0, z + 0.0]);
        bytes.extend([raw_amount.unwrap_or(255), raw_falling.map_or(255, |v| v as u8), fluid_flow.is_some() as u8]);
        if let Some(flow) = fluid_flow {
            // solid_faces order: north, east, south, west.
            const ORDER: [u8; 4] = [2, 5, 3, 4];
            let mut faces = [false; 4];
            for (facing, solid) in ORDER.into_iter().zip(flow.solid_faces) {
                faces[ORDER.iter().position(|&f| f == frame.facing(facing)).unwrap()] = solid;
            }
            bytes.push(flow.blocks_motion as u8);
            bytes.extend(faces.map(u8::from));
        }
    }
    if groups & HOOKS != 0 {
        if unsupported_fall_on.is_some() || matches!(inside, Inside::Unsupported(_)) { return Err(PatternError::UnsupportedHook); }
        bytes.push(inside_shape.is_some() as u8);
        if let Some(shape) = inside_shape { exact &= encode_shape(bytes, shape, frame)?; }
        for value in [*friction, *speed_factor, *jump_factor, *fall_on_multiplier, *bounce] { bytes.extend(value.to_bits().to_le_bytes()); }
        for value in [*water_block, *climbable, *scaffolding, *bed, *suffocating, *suppresses_bounce, *slime,
            *fence, *wall_or_gate, *resets_fall_distance, *end_portal_or_gateway, *world_hook, *air] { bytes.push(value as u8); }
        bytes.extend([*ladder_facing, *open_trapdoor_facing].map(|f| f.map_or(255, |f| frame.facing(f))));
        bytes.push(match inside {
            Inside::None => 0,
            Inside::Cobweb => 1,
            Inside::PowderSnow => 2,
            Inside::Honey => 3,
            Inside::BubbleColumn { drag: false } => 4,
            Inside::BubbleColumn { drag: true } => 5,
            Inside::Fire { soul: false } => 6,
            Inside::Fire { soul: true } => 7,
            Inside::Campfire { soul: false } => 8,
            Inside::Campfire { soul: true } => 9,
            Inside::BerryBush { hurt: false } => 10,
            Inside::BerryBush { hurt: true } => 11,
            Inside::Cactus => 12,
            Inside::Extinguish => 13,
            Inside::Lava => 14,
            Inside::EndPortal => 15,
            Inside::NetherPortal => 16,
            Inside::WitherRose => 17,
            Inside::Unsupported(_) => unreachable!()
        });
        bytes.push(match grounded_contact {
            crate::GroundedContact::None => 0,
            crate::GroundedContact::DamageUnlessCareful(c) => match c {
                crate::ContactDamage::HotFloor => 1,
                crate::ContactDamage::OnFire => 2,
                crate::ContactDamage::InFire { soul: false } => 3,
                crate::ContactDamage::InFire { soul: true } => 4,
                crate::ContactDamage::Campfire { soul: false } => 5,
                crate::ContactDamage::Campfire { soul: true } => 6,
                crate::ContactDamage::SweetBerryBush => 7,
                crate::ContactDamage::Cactus => 8,
            },
        });
        bytes.push(stuck_multiplier.is_some() as u8);
        if let Some(v) = stuck_multiplier {
            // A per-axis scale: turns swap x and z, mirrors leave it.
            let v = if frame.quarter_turns % 2 == 0 { *v } else { Vec3::new(v.z, v.y, v.x) };
            f64s(bytes, [v.x, v.y, v.z]);
        }
    }
    Ok(exact)
}
