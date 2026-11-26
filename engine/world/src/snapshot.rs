use crate::{ BlockState, LoadError, WorldReader, nbt, region::metadata};
use law::{
    basis::cell::{self, models, ModelId, StateId, BEYOND, OWN_CELL},
    Aabb, Block, BlockPos, FluidKind, Inside, World,
};
use std::{collections::BTreeMap, path::Path};

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

/// One 16³ section, its cells at `(y << 8) | (z << 4) | x`: a palette of states with their
/// models, each cell's palette index packed at a power-of-two width (none when the section is
/// uniform), and a summary of its 4³ subcells (bit `(y/4 << 4) | (z/4 << 2) | x/4`): `mixed`
/// marks those holding more than one state, `other` those holding a state unlike the first.
struct Section { states: Box<[StateId]>, models: Box<[ModelId]>, width: u32, data: Box<[u64]>, mixed: u64, other: u64, collide: u64, flags: u8 }
impl Section {
    fn new(cells: &[StateId; 4096]) -> Self {
        let mut states = vec![cells[0]];
        let index: Vec<u16> = cells.iter().map(|s| match states.iter().position(|p| p == s) {
            Some(k) => k as u16,
            None => { states.push(*s); (states.len() - 1) as u16 }
        }).collect();
        // The width in bits: none for one state, else the power of two that holds every index.
        let width = if states.len() == 1 { 0 } else { (u16::BITS - (states.len() as u16 - 1).leading_zeros()).next_power_of_two() };
        let mut data = vec![0u64; width as usize * 64];
        let (mut mixed, mut other) = (0, 0);
        for (i, &k) in index.iter().enumerate() {
            if width > 0 { data[i * width as usize / 64] |= (k as u64) << (i * width as usize % 64); }
            let bit = 1u64 << (((i >> 10) << 4) | (((i >> 6) & 3) << 2) | ((i >> 2) & 3));
            if k != 0 { other |= bit; }
            if k != index[i & 0xccc] { mixed |= bit; }
        }
        let mut flags = 0u8;
        let models = states.iter().map(|&s| {
            if s == UNKNOWN_STATE {
                flags |= 3;
                UNKNOWN_MODEL
            } else {
                let b = s.model_block();
                if b.fluid.kind != FluidKind::Empty { flags |= 1; }
                if b.has_inside_effects() { flags |= 2; }
                s.model().expect("a modeled state")
            }
        }).collect::<Box<[ModelId]>>();
        // The 4³ subcells holding a state that can collide: a sweep through the others meets nothing.
        let reach = &cell::models().reach;
        let collides: Vec<bool> = models.iter().map(|&m| m == UNKNOWN_MODEL || reach[m.0 as usize] != 0).collect();
        let collide = index.iter().enumerate().filter(|&(_, &k)| collides[k as usize])
            .fold(0u64, |c, (i, _)| c | 1u64 << (((i >> 10) << 4) | (((i >> 6) & 3) << 2) | ((i >> 2) & 3)));
        Self { states: states.into(), models, width, data: data.into(), mixed, other, collide, flags }
    }
    #[inline]
    fn index(&self, i: usize) -> usize {
        if self.width == 0 { return 0; }
        let bit = i * self.width as usize;
        ((self.data[bit / 64] >> (bit % 64)) & ((1 << self.width) - 1)) as usize
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
        let model_version: i32 = include_str!("../data/version.txt").trim().parse().expect("generated DataVersion");
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
    /// The box `min..=max` holding `state(p)` at each cell, built in memory (normal difficulty,
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
        Self { min, max, origin, span, sections, difficulty: 2, natural_regeneration: true, fluids, inside, unknown: BTreeMap::new() }
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
    fn section_flags_in(&self, lo: BlockPos, hi: BlockPos, mask: u8) -> bool {
        for sz in lo[2] >> 4..=hi[2] >> 4 {
            for sy in lo[1] >> 4..=hi[1] >> 4 {
                for sx in lo[0] >> 4..=hi[0] >> 4 {
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
    /// Whether a 4³ subcell met by `lo..=hi` holds a state that can collide.
    fn may_collide(&self, lo: BlockPos, hi: BlockPos) -> bool {
        for sz in lo[2] >> 4..=hi[2] >> 4 { for sy in lo[1] >> 4..=hi[1] >> 4 { for sx in lo[0] >> 4..=hi[0] >> 4 {
            let base = [sx * 16, sy * 16, sz * 16];
            let section = self.section([0, 1, 2].map(|i| lo[i].max(base[i])));
            if section.collide == 0 { continue; }
            let (a, b): (BlockPos, BlockPos) = ([0, 1, 2].map(|i| (lo[i].max(base[i]) - base[i]) >> 2), [0, 1, 2].map(|i| (hi[i].min(base[i] + 15) - base[i]) >> 2));
            for qy in a[1]..=b[1] { for qz in a[2]..=b[2] { for qx in a[0]..=b[0] {
                if section.collide >> ((qy << 4) | (qz << 2) | qx) & 1 != 0 { return true; }
            } } }
        } } }
        false
    }
    fn covers(&self, p: BlockPos) -> bool { (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i]) }
    fn assert_covered(&self, p: BlockPos) {
        assert!(self.covers(p), "snapshot query {p:?} outside validated box {:?}..{:?}; load the law's entire read footprint",self.min,self.max);
    }
    #[inline]
    fn section(&self, p: BlockPos) -> &Section {
        self.assert_covered(p);
        let s = [0, 1, 2].map(|a| ((p[a] >> 4) - self.origin[a]) as usize);
        &self.sections[(s[2] * self.span[1] + s[1]) * self.span[0] + s[0]]
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
                if !agree(section.states[0]) { return None; }
                continue;
            }
            let (a, b): (BlockPos, BlockPos) = ([0, 1, 2].map(|i| lo[i].max(base[i]) - base[i]), [0, 1, 2].map(|i| hi[i].min(base[i] + 15) - base[i]));
            for qy in a[1] >> 2..=b[1] >> 2 { for qz in a[2] >> 2..=b[2] >> 2 { for qx in a[0] >> 2..=b[0] >> 2 {
                let bit = 1u64 << ((qy << 4) | (qz << 2) | qx);
                if section.other & bit == 0 { if !agree(section.states[0]) { return None; } continue; }
                if section.mixed & bit == 0 { if !agree(section.states[section.index(((qy << 10) | (qz << 6) | (qx << 2)) as usize)]) { return None; } continue; }
                for y in a[1].max(qy * 4)..=b[1].min(qy * 4 + 3) { for z in a[2].max(qz * 4)..=b[2].min(qz * 4 + 3) { for x in a[0].max(qx * 4)..=b[0].min(qx * 4 + 3) {
                    if !agree(section.states[section.index(((y << 8) | (z << 4) | x) as usize)]) { return None; }
                } } }
            } } }
        } } }
        found
    }
}
#[inline]
fn local(p: BlockPos) -> usize { (((p[1] & 15) << 8) | ((p[2] & 15) << 4) | (p[0] & 15)) as usize }
fn check_version(root: nbt::Tag<'_>, expected: i32, file: &str) -> Result<(),LoadError> {
    let found = root.required("DataVersion")?.int()?;
    if expected != found { return Err(LoadError::Invalid(format!("{file} DataVersion {found} differs from level.dat {expected}"))); }
    Ok(())
}
impl World for WorldSnapshot {
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
        if !self.covers(p) { return Err("outside the saved snapshot"); }
        if let Some(why) = self.unknown.get(&p) { return Err(why.as_str()); }
        Ok(self.block(p))
    }
    #[inline]
    fn block(&self, p: BlockPos) -> &Block {
        if let Some(why) = self.unknown.get(&p) { panic!("unmodeled cell at {p:?}: {why}"); }
        let section = self.section(p);
        section.models[section.index(local(p))].block_at(p)
    }
    fn state_id(&self, p: BlockPos) -> Option<StateId> {
        let section = self.section(p);
        let s = section.states[section.index(local(p))];
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
        self.fluids && self.section_flags_in(lo, hi, 1)
    }
    fn has_inside_effects_in(&self, lo: BlockPos, hi: BlockPos) -> bool {
        (self.inside || self.fluids) && self.section_flags_in(lo, hi, 2)
    }
    /// The law's order (z, y, x), from the cells' own models: a cell that never collides
    /// is left out, and one whose collision stays in its cell is left out of the halo,
    /// where it cannot meet `bounds`. A cell outside the snapshot panics where the full scan
    /// would first ask it.
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
        if self.covers(lo) && self.covers(hi) && !self.may_collide(lo, hi) { return true; }
        let near_lo = [bounds.min.x, bounds.min.y, bounds.min.z].map(law::math::floor);
        let near_hi = [bounds.max.x, bounds.max.y, bounds.max.z].map(law::math::floor);
        let near = |a: usize, v: i32| v >= near_lo[a] && v <= near_hi[a];
        let reach = &models().reach;
        for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] {
            let p = [x, y, z];
            let section = self.section(p);
            let m = section.models[section.index(local(p))];
            let r = if m == UNKNOWN_MODEL { BEYOND } else { reach[m.0 as usize] };
            if (r == BEYOND || r == OWN_CELL && near(2, z) && near(1, y) && near(0, x)) && !visit(p) { return false; }
        } } }
        true
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

