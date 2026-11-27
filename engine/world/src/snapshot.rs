use crate::{ BlockState, LoadError, WorldReader, nbt, region::metadata};
use law::{
    basis::cell::{self, models, ModelId, StateId, BEYOND, OWN_CELL},
    shapes::ShapeView,
    block_collision_shape, block_pos,
    Aabb, Block, BlockPos, CollisionContext, FluidKind, Inside, Vec3, World,
};
use std::{collections::BTreeMap, path::Path, sync::OnceLock};

const SNAPSHOT_BYTES: usize = 128 * 1024 * 1024;
const UNKNOWN_STATE: StateId = StateId(u16::MAX);
const UNKNOWN_MODEL: ModelId = ModelId(u16::MAX);
/// The state's canonical id, or the reason the law refuses it.
pub fn state_id(state: &BlockState) -> Result<StateId, String> {
    cell::lookup(&state.name, |p| state.properties.iter().find(|(k, _)| k == p).map(|(_, v)| v.as_str()))
}
/// The shared model table, indexed by model id. Every admitted state maps to one row.
pub fn model_table() -> &'static [Block] { &models().blocks }
/// The law's model of a block state, or the reason the generated rule refuses it.
pub fn model(state: &BlockState) -> Result<&'static Block, String> {
    state_id(state).map(StateId::model_block)
}

const FLAG_FLUID: u8 = 1;
const FLAG_INSIDE: u8 = 2;
const FLAG_OFFSET: u8 = 4;
const FLAG_UNKNOWN: u8 = 8;

#[derive(Clone, Copy)]
struct PaletteSlot {
    block: &'static Block,
    state: StateId,
    model: ModelId,
    reach: u8,
}

/// One 16³ section, its cells at `(y << 8) | (z << 4) | x`: a palette of states with their
/// models, each cell's palette index packed at a power-of-two width (none when the section is
/// uniform), and a summary of its 4³ subcells (bit `(y/4 << 4) | (z/4 << 2) | x/4`): `mixed`
/// marks those holding more than one state, `other` those holding a state unlike the first.
struct Section {
    slots: Box<[PaletteSlot]>,
    width: u32,
    shift: u8,
    data: Box<[u64]>,
    mixed: u64,
    other: u64,
    collide: u64,
    collide_beyond: u64,
    flags: u8,
}
impl Section {
    fn new(cells: &[StateId; 4096]) -> Self {
        let mut states = vec![cells[0]];
        let index: Vec<u16> = cells.iter().map(|s| match states.iter().position(|p| p == s) {
            Some(k) => k as u16,
            None => { states.push(*s); (states.len() - 1) as u16 }
        }).collect();
        // The width in bits: none for one state, else the power of two that holds every index.
        let width = if states.len() == 1 { 0 } else { (u16::BITS - (states.len() as u16 - 1).leading_zeros()).next_power_of_two() };
        let shift = if width == 0 { 0 } else { width.trailing_zeros() as u8 };
        let mut data = vec![0u64; width as usize * 64];
        let (mut mixed, mut other) = (0, 0);
        for (i, &k) in index.iter().enumerate() {
            if width > 0 { data[i * width as usize / 64] |= (k as u64) << (i * width as usize % 64); }
            let bit = 1u64 << (((i >> 10) << 4) | (((i >> 6) & 3) << 2) | ((i >> 2) & 3));
            if k != 0 { other |= bit; }
            if k != index[i & 0xccc] { mixed |= bit; }
        }
        let mut flags = 0u8;
        let reach = &cell::models().reach;
        static FULL_FALLBACK: OnceLock<Block> = OnceLock::new();
        let slots = states.iter().map(|&s| {
            if s == UNKNOWN_STATE {
                flags |= FLAG_FLUID | FLAG_INSIDE | FLAG_UNKNOWN;
                PaletteSlot {
                    block: FULL_FALLBACK.get_or_init(Block::full),
                    state: s,
                    model: UNKNOWN_MODEL,
                    reach: BEYOND,
                }
            } else {
                let model = s.model().expect("a modeled state");
                let block = if model.fixed() {
                    model.block_at([0, 0, 0])
                } else {
                    flags |= FLAG_OFFSET;
                    s.model_block()
                };
                if block.fluid.kind != FluidKind::Empty { flags |= FLAG_FLUID; }
                if block.has_inside_effects() { flags |= FLAG_INSIDE; }
                PaletteSlot { block, state: s, model, reach: reach[model.0 as usize] }
            }
        }).collect::<Box<[PaletteSlot]>>();
        // The 4³ subcells holding a state that can collide: a sweep through the others meets nothing.
        let (mut collide, mut collide_beyond) = (0u64, 0u64);
        for (i, &k) in index.iter().enumerate() {
            let r = slots[k as usize].reach;
            if r != 0 {
                let bit = 1u64 << (((i >> 10) << 4) | (((i >> 6) & 3) << 2) | ((i >> 2) & 3));
                collide |= bit;
                if r == BEYOND { collide_beyond |= bit; }
            }
        }
        Self { slots, width, shift, data: data.into(), mixed, other, collide, collide_beyond, flags }
    }
    #[inline(always)]
    fn index(&self, i: usize) -> usize {
        if self.width == 0 { return 0; }
        let bit = i << self.shift;
        ((self.data[bit >> 6] >> (bit & 63)) & ((1u64 << self.width) - 1)) as usize
    }
    #[inline(always)]
    fn slot_at(&self, i: usize) -> PaletteSlot {
        self.slots[self.index(i)]
    }
    #[inline(always)]
    fn block_of(&self, slot: PaletteSlot, p: BlockPos) -> &Block {
        if (self.flags & FLAG_OFFSET) == 0 {
            slot.block
        } else {
            slot.model.block_at(p)
        }
    }
}

/// Immutable inclusive box. All cells and chunks in this box are validated before
/// construction succeeds. Queries outside it panic: callers must load the entire
/// collision/fluid footprint before running the law, including its read halo.
/// Static block snapshots do not simulate world ticks, entities, or world borders.
/// Cells are canonical states in the box's 16³ sections.
pub struct WorldSnapshot {
    min: BlockPos,
    max: BlockPos,
    /// The first section's coordinates and the sections along each axis.
    origin: BlockPos,
    span: [usize; 3],
    sections: Box<[Section]>,
    difficulty: u8,
    natural_regeneration: bool,
    fluids: bool,
    inside: bool,
    unknown: BTreeMap<BlockPos, String>,
}
impl WorldSnapshot {
    pub fn load(world_dir: impl AsRef<Path>, min: BlockPos, max: BlockPos) -> Result<Self, LoadError> {
        Self::load_with_limit(world_dir, min, max, SNAPSHOT_BYTES)
    }
    /// Limit covers the box's cells at two bytes each; shared model data and the reader's
    /// separately bounded cache and temporary parsing buffers are not included.
    pub fn load_with_limit(world_dir: impl AsRef<Path>, min: BlockPos, max: BlockPos, max_bytes: usize) -> Result<Self, LoadError> {
        Self::read(world_dir.as_ref(), min, max, max_bytes, false)
    }
    /// Keep unknown palette entries for the corridor and its fallible replay.
    pub fn load_for_plan(world_dir: &Path, min: BlockPos, max: BlockPos) -> Result<Self, LoadError> {
        Self::read(world_dir, min, max, SNAPSHOT_BYTES, true)
    }
    fn read(world_dir: &Path, min: BlockPos, max: BlockPos, max_bytes: usize, partial: bool) -> Result<Self, LoadError> {
        if (0..3).any(|i| min[i] > max[i]) { return Err(LoadError::Invalid("snapshot min exceeds max".into())); }
        let widths: [usize;3] = std::array::from_fn(|i| (max[i] as i64 - min[i] as i64 + 1) as usize);
        let count = widths.into_iter().try_fold(1usize, |n,w| n.checked_mul(w))
            .ok_or_else(|| LoadError::Invalid("snapshot volume overflow".into()))?;
        if count.checked_mul(2).is_none_or(|n| n > max_bytes) { return Err(LoadError::Unsupported(format!("snapshot needs {} cells, beyond {max_bytes}-byte limit", count))); }
        let mut reader = WorldReader::open(world_dir)?;
        if reader.datapacks != ["vanilla"] { return Err(LoadError::Unsupported(format!("law models require only the vanilla datapack; enabled {:?}",reader.datapacks))); }
        let model_version = cell::DATA_VERSION;
        if reader.data_version() != model_version { return Err(LoadError::Unsupported(format!("law models have DataVersion {model_version}, world has {}",reader.data_version()))); }
        let generation = metadata(&reader.world_dir.join("data/minecraft/world_gen_settings.dat"))?;
        let generation = nbt::root(&generation)?;
        check_version(generation, reader.data_version(), "world_gen_settings.dat")?;
        let dimension = generation.required("data")?.required("dimensions")?.required("minecraft:overworld")?.required("type")?.string()?;
        if dimension != "minecraft:overworld" { return Err(LoadError::Unsupported(format!("snapshot dimension type {dimension}; needs vanilla minecraft:overworld"))); }
        let rules = metadata(&reader.world_dir.join("data/minecraft/game_rules.dat"))?;
        let rules = nbt::root(&rules)?;
        check_version(rules, reader.data_version(), "game_rules.dat")?;
        let natural_regeneration = match rules.required("data")?.required("minecraft:natural_health_regeneration")?.byte()? {
            0 => false, 1 => true, _ => return Err(LoadError::Invalid("natural_health_regeneration is not boolean".into())),
        };
        let (mut fluids, mut inside) = (false, false);
        let mut unknown = BTreeMap::new();
        let read = |p: BlockPos| {
            let state = reader.block_state(p)?;
            let id = match state_id(state) {
                Ok(id) => id,
                Err(reason) if partial => {
                    fluids = true;
                    inside = true;
                    unknown.insert(p, format!("{state}: {reason}"));
                    return Ok::<_, LoadError>(UNKNOWN_STATE);
                }
                Err(reason) => return Err(LoadError::Unmodeled { position: p, state: state.clone(), reason }),
            };
            let block = id.model_block();
            fluids |= block.fluid.kind != FluidKind::Empty;
            inside |= block.inside != Inside::None || block.stuck_multiplier.is_some();
            Ok::<_, LoadError>(id)
        };
        let (origin, span, sections) = Self::sections(min, max, read)?;
        Ok(Self { min, max, origin, span, sections, difficulty: reader.difficulty, natural_regeneration, fluids, inside, unknown })
    }
    /// The box `min..=max` holding `state(p)` at each cell, built in memory (peaceful,
    /// natural regeneration on): the same sections a saved world loads into.
    pub fn from_fn(min: BlockPos, max: BlockPos, mut state: impl FnMut(BlockPos) -> StateId) -> Self {
        let (mut fluids, mut inside) = (false, false);
        let (origin, span, sections) = Self::sections(min, max, |p| {
            let id = state(p);
            let block = id.model_block();
            fluids |= block.fluid.kind != FluidKind::Empty;
            inside |= block.inside != Inside::None || block.stuck_multiplier.is_some();
            Ok(id)
        }).expect("a generator cannot fail");
        Self { min, max, origin, span, sections, difficulty: 0, natural_regeneration: true, fluids, inside, unknown: BTreeMap::new() }
    }
    fn sections(min: BlockPos, max: BlockPos, mut read: impl FnMut(BlockPos) -> Result<StateId, LoadError>) -> Result<(BlockPos, [usize; 3], Box<[Section]>), LoadError> {
        let origin = min.map(|v| v >> 4);
        let span: [usize; 3] = std::array::from_fn(|i| ((max[i] >> 4) - origin[i] + 1) as usize);
        let mut sections: Vec<Option<Section>> = (0..span.iter().product()).map(|_| None).collect();
        // Chunk order avoids region rereads for every interleaved row of a large box.
        for sz in 0..span[2] { for sx in 0..span[0] { for sy in 0..span[1] {
            let base = [0, 1, 2].map(|i| (origin[i] + [sx, sy, sz][i] as i32) * 16);
            let (lo, hi) = ([0, 1, 2].map(|i| min[i].max(base[i])), [0, 1, 2].map(|i| max[i].min(base[i] + 15)));
            // Cells outside the box are never asked; they hold the state of its first cell here.
            let mut cells = [read(lo)?; 4096];
            for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] { cells[local([x, y, z])] = read([x, y, z])?; } } }
            sections[(sz * span[1] + sy) * span[0] + sx] = Some(Section::new(&cells));
        } } }
        Ok((origin, span, sections.into_iter().map(|s| s.expect("every section loaded")).collect()))
    }
    pub fn bounds(&self) -> (BlockPos, BlockPos) { (self.min,self.max) }
    #[inline(always)]
    fn covers(&self, p: BlockPos) -> bool {
        p[0] >= self.min[0] && p[0] <= self.max[0]
            && p[1] >= self.min[1] && p[1] <= self.max[1]
            && p[2] >= self.min[2] && p[2] <= self.max[2]
    }
    #[cold]
    #[inline(never)]
    fn outside_box(&self, p: BlockPos) -> ! {
        panic!("snapshot query {p:?} outside validated box {:?}..{:?}; load the law's entire read footprint", self.min, self.max);
    }
    #[inline(always)]
    fn assert_covered(&self, p: BlockPos) {
        if !self.covers(p) { self.outside_box(p); }
    }
    #[inline(always)]
    fn section_idx(&self, sx: i32, sy: i32, sz: i32) -> usize {
        let oz = (sz - self.origin[2]) as usize;
        let oy = (sy - self.origin[1]) as usize;
        let ox = (sx - self.origin[0]) as usize;
        (oz * self.span[1] + oy) * self.span[0] + ox
    }
    #[inline(always)]
    fn section(&self, p: BlockPos) -> &Section {
        self.assert_covered(p);
        &self.sections[self.section_idx(p[0] >> 4, p[1] >> 4, p[2] >> 4)]
    }
    fn section_flags_in(&self, lo: BlockPos, hi: BlockPos, mask: u8) -> bool {
        if lo[0] > hi[0] || lo[1] > hi[1] || lo[2] > hi[2] { return false; }
        let s_lo = [lo[0] >> 4, lo[1] >> 4, lo[2] >> 4];
        let s_hi = [hi[0] >> 4, hi[1] >> 4, hi[2] >> 4];
        if lo[0] >= self.min[0] && lo[1] >= self.min[1] && lo[2] >= self.min[2]
            && s_hi[0] <= (self.max[0] >> 4) && s_hi[1] <= (self.max[1] >> 4) && s_hi[2] <= (self.max[2] >> 4)
        {
            if s_lo == s_hi {
                return (self.sections[self.section_idx(s_lo[0], s_lo[1], s_lo[2])].flags & mask) != 0;
            }
            for sz in s_lo[2]..=s_hi[2] {
                let z_off = (sz - self.origin[2]) as usize * self.span[1];
                for sy in s_lo[1]..=s_hi[1] {
                    let row = (z_off + (sy - self.origin[1]) as usize) * self.span[0];
                    for sx in s_lo[0]..=s_hi[0] {
                        if (self.sections[row + (sx - self.origin[0]) as usize].flags & mask) != 0 { return true; }
                    }
                }
            }
            return false;
        }
        for sz in s_lo[2]..=s_hi[2] {
            for sy in s_lo[1]..=s_hi[1] {
                for sx in s_lo[0]..=s_hi[0] {
                    let base = [sx * 16, sy * 16, sz * 16];
                    let p = [0, 1, 2].map(|i| lo[i].max(base[i]));
                    if !self.covers(p) || (self.section(p).flags & mask) != 0 {
                        return true;
                    }
                }
            }
        }
        false
    }
    #[inline]
    fn scan_collision(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> (ScanResult, BlockPos, BlockPos) {
        if lo[0] > hi[0] || lo[1] > hi[1] || lo[2] > hi[2] {
            return (ScanResult::Empty, lo, hi);
        }
        if !self.covers(lo) || !self.covers(hi) {
            let near_lo = [law::math::floor(bounds.min.x), law::math::floor(bounds.min.y), law::math::floor(bounds.min.z)];
            let near_hi = [law::math::floor(bounds.max.x), law::math::floor(bounds.max.y), law::math::floor(bounds.max.z)];
            return (ScanResult::Uncovered, near_lo, near_hi);
        }
        let s_lo = [lo[0] >> 4, lo[1] >> 4, lo[2] >> 4];
        let s_hi = [hi[0] >> 4, hi[1] >> 4, hi[2] >> 4];
        if s_lo == s_hi {
            let idx = self.section_idx(s_lo[0], s_lo[1], s_lo[2]);
            let sec = &self.sections[idx];
            if sec.collide == 0 {
                return (ScanResult::Empty, lo, hi);
            }
        }
        let near_lo = [
            law::math::floor(bounds.min.x).max(lo[0]),
            law::math::floor(bounds.min.y).max(lo[1]),
            law::math::floor(bounds.min.z).max(lo[2]),
        ];
        let near_hi = [
            law::math::floor(bounds.max.x).min(hi[0]),
            law::math::floor(bounds.max.y).min(hi[1]),
            law::math::floor(bounds.max.z).min(hi[2]),
        ];
        let mut beyond = false;
        let mut box_lo = [i32::MAX; 3];
        let mut box_hi = [i32::MIN; 3];
        let mut active_count = 0usize;
        let mut last_idx = 0usize;
        for sz in s_lo[2]..=s_hi[2] {
            let z_off = (sz - self.origin[2]) as usize * self.span[1];
            for sy in s_lo[1]..=s_hi[1] {
                let row = (z_off + (sy - self.origin[1]) as usize) * self.span[0];
                for sx in s_lo[0]..=s_hi[0] {
                    let idx = row + (sx - self.origin[0]) as usize;
                    let sec = &self.sections[idx];
                    if sec.collide == 0 { continue; }
                    let base = [sx * 16, sy * 16, sz * 16];
                    let mut touched = false;
                    if sec.collide_beyond != 0 {
                        let hb_lo = [lo[0].max(base[0]), lo[1].max(base[1]), lo[2].max(base[2])];
                        let hb_hi = [hi[0].min(base[0] + 15), hi[1].min(base[1] + 15), hi[2].min(base[2] + 15)];
                        let mask = subcell_mask(
                            [(hb_lo[0] - base[0]) >> 2, (hb_lo[1] - base[1]) >> 2, (hb_lo[2] - base[2]) >> 2],
                            [(hb_hi[0] - base[0]) >> 2, (hb_hi[1] - base[1]) >> 2, (hb_hi[2] - base[2]) >> 2],
                        );
                        let bm = sec.collide_beyond & mask;
                        if bm != 0 {
                            beyond = true;
                            touched = true;
                            let qy_min = (bm.trailing_zeros() as i32) >> 4;
                            let qy_max = (63 - bm.leading_zeros() as i32) >> 4;
                            let y0 = hb_lo[1].max(base[1] + (qy_min << 2));
                            let y1 = hb_hi[1].min(base[1] + (qy_max << 2) + 3);
                            box_lo[0] = box_lo[0].min(hb_lo[0]);
                            box_lo[1] = box_lo[1].min(y0);
                            box_lo[2] = box_lo[2].min(hb_lo[2]);
                            box_hi[0] = box_hi[0].max(hb_hi[0]);
                            box_hi[1] = box_hi[1].max(y1);
                            box_hi[2] = box_hi[2].max(hb_hi[2]);
                        }
                    }
                    let n_lo = [near_lo[0].max(base[0]), near_lo[1].max(base[1]), near_lo[2].max(base[2])];
                    let n_hi = [near_hi[0].min(base[0] + 15), near_hi[1].min(base[1] + 15), near_hi[2].min(base[2] + 15)];
                    if n_lo[0] <= n_hi[0] && n_lo[1] <= n_hi[1] && n_lo[2] <= n_hi[2] {
                        let mask = subcell_mask(
                            [(n_lo[0] - base[0]) >> 2, (n_lo[1] - base[1]) >> 2, (n_lo[2] - base[2]) >> 2],
                            [(n_hi[0] - base[0]) >> 2, (n_hi[1] - base[1]) >> 2, (n_hi[2] - base[2]) >> 2],
                        );
                        let om = sec.collide & mask;
                        if om != 0 {
                            touched = true;
                            let qy_min = (om.trailing_zeros() as i32) >> 4;
                            let qy_max = (63 - om.leading_zeros() as i32) >> 4;
                            let y0 = n_lo[1].max(base[1] + (qy_min << 2));
                            let y1 = n_hi[1].min(base[1] + (qy_max << 2) + 3);
                            box_lo[0] = box_lo[0].min(n_lo[0]);
                            box_lo[1] = box_lo[1].min(y0);
                            box_lo[2] = box_lo[2].min(n_lo[2]);
                            box_hi[0] = box_hi[0].max(n_hi[0]);
                            box_hi[1] = box_hi[1].max(y1);
                            box_hi[2] = box_hi[2].max(n_hi[2]);
                        }
                    }
                    if touched {
                        active_count += 1;
                        last_idx = idx;
                    }
                }
            }
        }
        let res = if active_count == 0 {
            ScanResult::Empty
        } else if active_count == 1 {
            ScanResult::Single { section_idx: last_idx, beyond, lo: box_lo, hi: box_hi }
        } else {
            ScanResult::Multi { beyond, lo: box_lo, hi: box_hi }
        };
        (res, near_lo, near_hi)
    }
    /// The one state every cell of `lo..=hi` holds, if one does. Each 4³ subcell the box meets
    /// answers from its section's summary; cells are read only in subcells holding two states.
    pub fn uniform_state(&self, lo: BlockPos, hi: BlockPos) -> Option<StateId> {
        if !self.covers(lo) || !self.covers(hi) { return None; }
        let mut found = None;
        let mut agree = |s: StateId| s != UNKNOWN_STATE && *found.get_or_insert(s) == s;
        for sz in lo[2] >> 4..=hi[2] >> 4 { for sy in lo[1] >> 4..=hi[1] >> 4 { for sx in lo[0] >> 4..=hi[0] >> 4 {
            let base = [sx * 16, sy * 16, sz * 16];
            let section = self.section([0, 1, 2].map(|i| lo[i].max(base[i])));
            if section.width == 0 {
                if !agree(section.slots[0].state) { return None; }
                continue;
            }
            let (a, b): (BlockPos, BlockPos) = ([0, 1, 2].map(|i| lo[i].max(base[i]) - base[i]), [0, 1, 2].map(|i| hi[i].min(base[i] + 15) - base[i]));
            for qy in a[1] >> 2..=b[1] >> 2 { for qz in a[2] >> 2..=b[2] >> 2 { for qx in a[0] >> 2..=b[0] >> 2 {
                let bit = 1u64 << ((qy << 4) | (qz << 2) | qx);
                if section.other & bit == 0 { if !agree(section.slots[0].state) { return None; } continue; }
                if section.mixed & bit == 0 { if !agree(section.slots[section.index(((qy << 10) | (qz << 6) | (qx << 2)) as usize)].state) { return None; } continue; }
                for y in a[1].max(qy * 4)..=b[1].min(qy * 4 + 3) { for z in a[2].max(qz * 4)..=b[2].min(qz * 4 + 3) { for x in a[0].max(qx * 4)..=b[0].min(qx * 4 + 3) {
                    if !agree(section.slots[section.index(((y << 8) | (z << 4) | x) as usize)].state) { return None; }
                } } }
            } } }
        } } }
        found
    }
}
impl WorldSnapshot {
    #[inline(always)]
    fn for_each_collision_slot(
        &self,
        bounds: Aabb,
        lo: BlockPos,
        hi: BlockPos,
        mut visit: impl FnMut(&Section, PaletteSlot, BlockPos) -> bool,
    ) -> bool {
        let (scan, near_lo, near_hi) = self.scan_collision(bounds, lo, hi);
        let near = |x: i32, y: i32, z: i32| {
            z >= near_lo[2] && z <= near_hi[2] && y >= near_lo[1] && y <= near_hi[1] && x >= near_lo[0] && x <= near_hi[0]
        };
        match scan {
            ScanResult::Empty => true,
            ScanResult::Uncovered => {
                for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] {
                    let p = [x, y, z];
                    let sec = self.section(p);
                    let slot = sec.slot_at(local(p));
                    if (slot.reach == BEYOND || slot.reach == OWN_CELL && near(x, y, z)) && !visit(sec, slot, p) {
                        return false;
                    }
                } } }
                true
            }
            ScanResult::Single { section_idx, beyond, lo: s_lo, hi: s_hi } => {
                let sec = &self.sections[section_idx];
                if sec.width == 0 {
                    let slot = sec.slots[0];
                    if slot.reach == 0 { return true; }
                    for z in s_lo[2]..=s_hi[2] { for y in s_lo[1]..=s_hi[1] { for x in s_lo[0]..=s_hi[0] {
                        if (slot.reach == BEYOND || !beyond || near(x, y, z)) && !visit(sec, slot, [x, y, z]) {
                            return false;
                        }
                    } } }
                } else {
                    for z in s_lo[2]..=s_hi[2] { for y in s_lo[1]..=s_hi[1] { for x in s_lo[0]..=s_hi[0] {
                        let slot = sec.slot_at(local([x, y, z]));
                        if (slot.reach == BEYOND || slot.reach == OWN_CELL && (!beyond || near(x, y, z)))
                            && !visit(sec, slot, [x, y, z])
                        {
                            return false;
                        }
                    } } }
                }
                true
            }
            ScanResult::Multi { beyond, lo: m_lo, hi: m_hi } => {
                for z in m_lo[2]..=m_hi[2] {
                    let z_off = ((z >> 4) - self.origin[2]) as usize * self.span[1];
                    for y in m_lo[1]..=m_hi[1] {
                        let row = (z_off + ((y >> 4) - self.origin[1]) as usize) * self.span[0];
                        for x in m_lo[0]..=m_hi[0] {
                            let sec = &self.sections[row + ((x >> 4) - self.origin[0]) as usize];
                            if sec.collide == 0 { continue; }
                            let slot = sec.slot_at(local([x, y, z]));
                            if (slot.reach == BEYOND || slot.reach == OWN_CELL && (!beyond || near(x, y, z)))
                                && !visit(sec, slot, [x, y, z])
                            {
                                return false;
                            }
                        }
                    }
                }
                true
            }
        }
    }
}
enum ScanResult {
    Uncovered,
    Empty,
    Single { section_idx: usize, beyond: bool, lo: BlockPos, hi: BlockPos },
    Multi { beyond: bool, lo: BlockPos, hi: BlockPos },
}
#[inline(always)]
fn subcell_mask(a: BlockPos, b: BlockPos) -> u64 {
    let x_len = b[0] - a[0] + 1;
    let z_len = b[2] - a[2] + 1;
    let y_len = b[1] - a[1] + 1;
    let xm = ((1u64 << x_len) - 1) << a[0];
    let z_nibbles = (0x1111u64 >> ((4 - z_len) * 4)) << (a[2] * 4);
    let y_planes = (0x0001_0001_0001_0001u64 >> ((4 - y_len) * 16)) << (a[1] * 16);
    (xm * z_nibbles) * y_planes
}
#[inline(always)]
fn local(p: BlockPos) -> usize { (((p[1] & 15) << 8) | ((p[2] & 15) << 4) | (p[0] & 15)) as usize }
fn check_version(root: nbt::Tag<'_>, expected: i32, file: &str) -> Result<(),LoadError> {
    let found = root.required("DataVersion")?.int()?;
    if expected != found { return Err(LoadError::Invalid(format!("{file} DataVersion {found} differs from level.dat {expected}"))); }
    Ok(())
}
impl World for WorldSnapshot {
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
        if !self.covers(p) { return Err("outside the saved snapshot"); }
        if !self.unknown.is_empty() {
            if let Some(why) = self.unknown.get(&p) { return Err(why.as_str()); }
        }
        Ok(self.block(p))
    }
    #[inline]
    fn block(&self, p: BlockPos) -> &Block {
        let section = self.section(p);
        if (section.flags & FLAG_UNKNOWN) != 0 {
            if let Some(why) = self.unknown.get(&p) { panic!("unmodeled cell at {p:?}: {why}"); }
        }
        section.block_of(section.slot_at(local(p)), p)
    }
    fn state_id(&self, p: BlockPos) -> Option<StateId> {
        let section = self.section(p);
        let s = section.slot_at(local(p)).state;
        (s != UNKNOWN_STATE).then_some(s)
    }
    /// A box of one state answers as one cell unless its model is offset per position.
    fn uniform(&self, lo: BlockPos, hi: BlockPos) -> Option<&Block> {
        let model = self.uniform_state(lo, hi)?.model()?;
        model.fixed().then(|| model.block_at(lo))
    }
    fn loaded(&self, p: BlockPos) -> bool { self.assert_covered(p); true }
    fn min_y(&self) -> i32 { -64 }
    fn difficulty(&self) -> u8 { self.difficulty }
    fn natural_regeneration(&self) -> bool { self.natural_regeneration }
    fn has_fluid(&self) -> bool { self.fluids }
    fn has_inside_effects(&self) -> bool { self.inside || self.fluids }
    fn has_fluid_in(&self, lo: BlockPos, hi: BlockPos) -> bool {
        self.fluids && self.section_flags_in(lo, hi, FLAG_FLUID)
    }
    fn has_inside_effects_in(&self, lo: BlockPos, hi: BlockPos) -> bool {
        (self.inside || self.fluids) && self.section_flags_in(lo, hi, FLAG_INSIDE)
    }
    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        match self.scan_collision(bounds, lo, hi).0 {
            ScanResult::Empty => None,
            ScanResult::Uncovered => Some((lo, hi)),
            ScanResult::Single { lo, hi, .. } | ScanResult::Multi { lo, hi, .. } => Some((lo, hi)),
        }
    }
    /// The law's order (z, y, x), from the cells' own models: a cell that never collides
    /// is left out, and one whose collision stays in its cell is left out of the halo,
    /// where it cannot meet `bounds`. A cell outside the snapshot panics where the full scan
    /// would first ask it.
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
        self.for_each_collision_slot(bounds, lo, hi, |_, _, p| visit(p))
    }
    fn visit_collisions(&self, bounds: Aabb, context: CollisionContext, mut visit: impl FnMut(ShapeView<'_>) -> bool) -> bool {
        if !self.visit_extra_collisions(bounds, context, &mut visit) { return false; }
        let tight_lo = block_pos(bounds.min.minus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        let tight_hi = block_pos(bounds.max.plus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        let (lo, hi) = (tight_lo.map(|v| v - 1), tight_hi.map(|v| v + 1));
        self.for_each_collision_slot(bounds, lo, hi, |sec, slot, p| {
            if slot.model == UNKNOWN_MODEL {
                let _ = self.block(p);
                unreachable!();
            }
            let local_shape = block_collision_shape(sec.block_of(slot, p), p[1], context);
            if local_shape.is_empty() { return true; }
            let offset = Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
            let shape = local_shape.at(offset);
            let intersects = if local_shape.is_block() {
                bounds.intersects(Aabb { min: offset, max: offset.plus(Vec3::new(1.0, 1.0, 1.0)) })
            } else {
                shape.intersects(bounds)
            };
            !intersects || visit(shape)
        })
    }
    fn collides_with_suffocating_block(&self, bounds: Aabb, context: CollisionContext) -> bool {
        let tight_lo = block_pos(bounds.min.minus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        let tight_hi = block_pos(bounds.max.plus(Vec3::new(1.0e-7, 1.0e-7, 1.0e-7)));
        let (lo, hi) = (tight_lo.map(|v| v - 1), tight_hi.map(|v| v + 1));
        !self.for_each_collision_slot(bounds, lo, hi, |sec, slot, p| {
            if slot.model == UNKNOWN_MODEL {
                let _ = self.block(p);
                unreachable!();
            }
            let block = sec.block_of(slot, p);
            if !block.suffocating { return true; }
            let local_shape = block_collision_shape(block, p[1], context);
            let offset = Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
            let intersects = if local_shape.is_block() {
                bounds.intersects(Aabb { min: offset, max: offset.plus(Vec3::new(1.0, 1.0, 1.0)) })
            } else {
                local_shape.at(offset).intersects(bounds)
            };
            !intersects
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn block(state:&str)->&'static Block {
        let (name,tail)=state.split_once('[').unwrap_or((state,""));
        let properties=tail.trim_end_matches(']').split(',').filter(|p|!p.is_empty())
            .map(|p| { let (k,v)=p.split_once('=').unwrap(); (k.to_owned(),v.to_owned()) }).collect();
        model(&BlockState { name: name.to_owned(), properties }).expect("supported state")
    }
    #[test]
    fn vanilla_tags_and_movement_properties_are_present() {
        let ladder=block("minecraft:ladder[facing=north,waterlogged=false]");
        assert!(ladder.climbable); assert_eq!(ladder.ladder_facing,Some(2));
        assert!(block("minecraft:vine[east=false,north=false,south=true,up=false,west=false]").climbable);
        assert!(block("minecraft:scaffolding[bottom=false,distance=0,waterlogged=false]").climbable);
        let slime=block("minecraft:slime_block");
        assert_eq!(slime.friction.to_bits(),0.8f32.to_bits());assert_eq!(slime.bounce,1.0);assert!(slime.slime);
        let honey=block("minecraft:honey_block");
        assert_eq!(honey.speed_factor.to_bits(),0.4f32.to_bits());assert_eq!(honey.jump_factor,0.5);
        assert_eq!(honey.inside,Inside::Honey);
        let source=block("minecraft:water[level=0]");
        assert_eq!(source.fluid.raw_amount,Some(8));assert_eq!(source.fluid.raw_falling,Some(false));
        let falling=block("minecraft:water[level=8]");assert_eq!(falling.fluid.raw_falling,Some(true));
        assert!(ladder.resets_fall_distance);
        assert!(honey.suppresses_bounce);
        let slab=block("minecraft:stone_slab[type=bottom,waterlogged=true]");
        assert_eq!(slab.fluid.kind,FluidKind::Water);assert!(!slab.water_block);
    }
    #[test]
    fn ported_hooks_fluids_and_fall_multipliers_are_present() {
        for name in ["minecraft:water_cauldron", "minecraft:powder_snow_cauldron"] {
            for level in 1..=3 {
                let cauldron = block(&format!("{name}[level={level}]"));
                assert_eq!(cauldron.inside, Inside::Extinguish);
                assert!(!cauldron.world_hook);
                let top = cauldron.inside_shape.as_ref().unwrap();
                assert!(top.intersects(Aabb::new(0.2, 0.3, 0.2, 0.8, 0.5625 + (level - 1) as f64 * 0.1875, 0.8)));
            }
        }
        let lava_cauldron = block("minecraft:lava_cauldron");
        assert_eq!(lava_cauldron.inside, Inside::Lava);
        assert_eq!(
            lava_cauldron.inside_shape.as_ref().unwrap().coords,
            block("minecraft:water_cauldron[level=3]").inside_shape.as_ref().unwrap().coords
        );
        let portal = block("minecraft:end_portal");
        assert_eq!(portal.inside, Inside::EndPortal);
        assert!(portal.end_portal_or_gateway);
        assert_eq!(
            portal.inside_shape.as_ref().unwrap().coords[1],
            vec![0.375, 0.75]
        );
        let gateway = block("minecraft:end_gateway");
        assert_eq!(gateway.inside, Inside::EndPortal);
        assert!(gateway.end_portal_or_gateway);
        assert!(gateway.inside_shape.as_ref().unwrap().is_block());
        assert_eq!(block("minecraft:nether_portal[axis=x]").inside, Inside::NetherPortal);
        assert_eq!(block("minecraft:wither_rose").inside, Inside::WitherRose);
        for tilt in ["none", "unstable", "partial", "full"] {
            let leaf = block(&format!("minecraft:big_dripleaf[facing=north,tilt={tilt},waterlogged=false]"));
            assert_eq!(leaf.inside, Inside::None);
            assert!(leaf.world_hook);
            assert_eq!(leaf.shape.is_empty(), tilt == "full");
        }
        assert_eq!(block("minecraft:fire[age=0,east=false,north=false,south=false,up=false,west=false]").inside, Inside::Fire { soul: false });
        assert_eq!(block("minecraft:soul_fire").inside, Inside::Fire { soul: true });
        assert_eq!(block("minecraft:campfire[facing=north,lit=true,signal_fire=false,waterlogged=false]").inside, Inside::Campfire { soul: false });
        assert_eq!(block("minecraft:soul_campfire[facing=north,lit=false,signal_fire=false,waterlogged=false]").inside, Inside::None);
        assert_eq!(block("minecraft:cactus[age=0]").inside, Inside::Cactus);
        assert!(block("minecraft:farmland[moisture=0]").world_hook);
        let stalagmite = block("minecraft:pointed_dripstone[thickness=tip,vertical_direction=up,waterlogged=false]");
        assert_eq!(stalagmite.fall_on_multiplier.to_bits(), 2.0f32.to_bits());
        assert_eq!(block("minecraft:pointed_dripstone[thickness=base,vertical_direction=up,waterlogged=false]").fall_on_multiplier.to_bits(), 1.0f32.to_bits());
        assert!(block("minecraft:wheat[age=0]").world_hook);
        assert!(block("minecraft:detector_rail[powered=false,shape=north_south,waterlogged=false]").world_hook);
        assert!(block("minecraft:redstone_ore[lit=false]").world_hook);
        assert!(block("minecraft:turtle_egg[eggs=1,hatch=0]").world_hook);
    }
    #[test]
    fn section_uniform_and_subcell_summary_masks() {
        let air = state_id(&BlockState { name: "minecraft:air".into(), properties: Vec::new() }).unwrap();
        let stone = state_id(&BlockState { name: "minecraft:stone".into(), properties: Vec::new() }).unwrap();
        let dirt = state_id(&BlockState { name: "minecraft:dirt".into(), properties: Vec::new() }).unwrap();

        let uniform = Section::new(&[air; 4096]);
        assert_eq!(uniform.width, 0);
        assert_eq!(uniform.other, 0);
        assert_eq!(uniform.mixed, 0);

        let mut cells = [air; 4096];
        // Fill subcell (qx=1, qz=0, qy=0) uniformly with stone, and subcell (qx=2, qz=0, qy=0) mixed with dirt.
        for y in 0..4 { for z in 0..4 { for x in 4..8 { cells[(y << 8) | (z << 4) | x] = stone; } } }
        cells[8] = dirt;
        let sec = Section::new(&cells);
        assert!(sec.width > 0);
        assert_eq!(sec.other, (1u64 << 1) | (1u64 << 2));
        assert_eq!(sec.mixed, 1u64 << 2);

        let snap = WorldSnapshot {
            min: [0, 0, 0], max: [31, 15, 15], origin: [0, 0, 0], span: [2, 1, 1],
            sections: vec![uniform, sec].into(),
            difficulty: 2, natural_regeneration: true, fluids: false, inside: false, unknown: BTreeMap::new(),
        };
        assert_eq!(snap.uniform_state([0, 0, 0], [15, 15, 15]), Some(air));
        assert_eq!(snap.uniform_state([20, 0, 0], [23, 3, 3]), Some(stone));
        assert_eq!(snap.uniform_state([16, 0, 0], [20, 3, 3]), None);
        assert_eq!(snap.uniform_state([24, 0, 0], [27, 3, 3]), None);
        assert_eq!(snap.uniform_state([25, 0, 0], [27, 3, 3]), Some(air));
    }
}

