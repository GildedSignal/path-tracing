use crate::{ BlockState, LoadError, WorldReader, nbt, region::metadata};
use law::{Aabb, Block, BlockPos, ContactDamage, Fluid, FluidFlowBlock, FluidKind, GroundedContact, Inside, VoxelShape, World};
use std::{collections::BTreeMap, path::Path, sync::OnceLock};

const SNAPSHOT_BYTES: usize = 128 * 1024 * 1024;
/// A shared generator: the properties that change a model, and their values' slots.
struct Map { properties: Vec<&'static str>, slots: BTreeMap<&'static str, usize> }
struct Models { maps: Vec<Map>, tables: BTreeMap<&'static str, (usize, Vec<u16>)>, blocks: Vec<Block>, reach: Vec<u8>, refusals: BTreeMap<&'static str, &'static str> }
/// How far a model's collision reaches, in any context.
const NOWHERE: u8 = 0;
const OWN_CELL: u8 = 1;
const BEYOND: u8 = 2;
fn models() -> &'static Models {
    static MODELS: OnceLock<Models> = OnceLock::new();
    MODELS.get_or_init(|| {
        let blocks = include_str!("../data/models.tsv").lines().map(parse_model).collect::<Vec<_>>();
        // maps.tsv: relevant properties, then "values:slot" entries in that property order.
        let maps = include_str!("../data/maps.tsv").lines().map(|line| {
            let (properties, entries) = line.split_once('\t').expect("generated map row");
            let slots = entries.split(' ').map(|e| {
                let (key, slot) = e.rsplit_once(':').expect("generated map entry");
                (key, slot.parse().expect("generated slot"))
            }).collect();
            Map { properties: properties.split(',').filter(|p| !p.is_empty()).collect(), slots }
        }).collect::<Vec<_>>();
        // blocks.tsv: block, its map, then the model id of each slot.
        let tables = include_str!("../data/blocks.tsv").lines().map(|line| {
            let [name, map, ids] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated block row") };
            let map: usize = map.parse().expect("generated map index");
            let ids = ids.split(',').map(|id| id.parse().expect("generated model id")).collect::<Vec<u16>>();
            assert!(map < maps.len() && ids.iter().all(|&id| (id as usize) < blocks.len()));
            assert!(maps[map].slots.values().all(|&s| s < ids.len()), "generated slots of {name}");
            (name, (map, ids))
        }).collect();
        // census.tsv: block, bucket, state count. Every bucket other than "modeled" names a refusal.
        let mut refusals = BTreeMap::new();
        for line in include_str!("../data/census.tsv").lines().filter(|l| !l.starts_with('#')) {
            let [block, bucket, _] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated census row") };
            if bucket != "modeled" { assert!(refusals.insert(block, bucket).is_none(), "one refusal reason per block: {block}"); }
        }
        let reach = blocks.iter().map(|b| if b.never_collides() { NOWHERE } else if b.collision_within_cell() { OWN_CELL } else { BEYOND }).collect();
        Models { maps, tables, blocks, reach, refusals }
    })
}
/// Project the state onto its block's relevant properties, then look the values up.
fn lookup(state: &BlockState) -> Result<u16, String> {
    let registry = models();
    if let Some(reason) = registry.refusals.get(state.name.as_str()) { return Err((*reason).into()); }
    let Some((map, ids)) = registry.tables.get(state.name.as_str()) else { return Err(format!("{} is not a 26.2 registry block", state.name)); };
    let map = &registry.maps[*map];
    map.properties.iter().map(|p| state.properties.iter().find(|(k,_)| k == p).map(|(_,v)| v.as_str()))
        .collect::<Option<Vec<_>>>().and_then(|values| map.slots.get(values.join(",").as_str()).map(|&slot| ids[slot]))
        .ok_or_else(|| format!("state unknown to the 26.2 table of {}", state.name))
}
/// The shared model table, indexed by model id. Every admitted state maps to one row.
pub fn model_table() -> &'static [Block] { &models().blocks }
/// The law's model of a block state, or the reason the generated rule refuses it.
pub fn model(state: &BlockState) -> Result<&'static Block, String> {
    lookup(state).map(|id| &models().blocks[id as usize])
}
fn shape(raw: &str) -> VoxelShape {
    let boxes = raw.split(';').filter(|s| !s.is_empty()).map(|s| {
        let v = s.split(',').map(|s| s.parse::<f64>().expect("generated box number")).collect::<Vec<_>>();
        assert_eq!(v.len(), 6);
        Aabb::new(v[0],v[1],v[2],v[3],v[4],v[5])
    }).collect::<Vec<_>>();
    VoxelShape::boxes(&boxes)
}
fn parse_model(line: &str) -> Block {
    let f: Vec<_> = line.split('\t').collect();
    assert_eq!(f.len(), 20, "generated model schema");
    let fp = |i: usize| f32::from_bits(u32::from_str_radix(f[i],16).expect("generated float bits"));
    let bits: u16 = f[5].parse().unwrap();
    let flag = |i: usize| bits & (1u16<<i) != 0;
    let direction = |i: usize| match f[i] { "-1" => None, s => Some(s.parse().unwrap()) };
    let amount: u8 = f[9].parse().unwrap();
    let faces: u8 = f[12].parse().unwrap();
    Block {
        shape: shape(f[15]), inside_shape: Some(shape(f[16])),
        above_shape: Some(shape(f[17])), below_shape: Some(shape(f[18])), sneak_shape: Some(shape(f[19])),
        friction: fp(0), speed_factor: fp(1), jump_factor: fp(2), bounce: fp(3), fall_on_multiplier: fp(4),
        air: flag(0), suffocating: flag(1), water_block: flag(2), climbable: flag(3), scaffolding: flag(4),
        slime: flag(5), bed: flag(6), fence: flag(7), wall_or_gate: flag(8), suppresses_bounce: flag(9),
        resets_fall_distance: flag(10), end_portal_or_gateway: flag(11), world_hook: flag(12),
        ladder_facing: direction(6), open_trapdoor_facing: direction(7),
        fluid: Fluid {
            kind: match f[8] { "0" => FluidKind::Empty, "1" => FluidKind::Water, _ => panic!("generated fluid kind") },
            raw_amount: (amount != 0).then_some(amount),
            raw_falling: (amount != 0).then(|| f[10].parse().unwrap()),
            ..Fluid::default()
        },
        fluid_flow: Some(FluidFlowBlock { blocks_motion: f[11].parse().unwrap(), solid_faces: std::array::from_fn(|i| faces & (1<<i) != 0) }),
        inside: match f[13] { "none" => Inside::None, "cobweb" => Inside::Cobweb, "powderSnow" => Inside::PowderSnow, "honey" => Inside::Honey, _ => panic!("generated inside hook") },
        grounded_contact: match f[14] { "none" => GroundedContact::None, "hotFloor" => GroundedContact::DamageUnlessCareful(ContactDamage::HotFloor), _ => panic!("generated step hook") },
        ..Block::default()
    }
}

/// Immutable inclusive box. All cells and chunks in this box are validated before
/// construction succeeds. Queries outside it panic: callers must load the entire
/// collision/fluid footprint before running the law, including its read halo.
/// Static block snapshots do not simulate world ticks, entities, or world borders.
pub struct WorldSnapshot {
    min: BlockPos,
    max: BlockPos,
    widths: [usize;3],
    cells: Box<[u16]>,
    models: &'static Models,
    difficulty: u8,
    natural_regeneration: bool,
    fluids: bool,
    inside: bool,
}
impl WorldSnapshot {
    pub fn load(world_dir: impl AsRef<Path>, min: BlockPos, max: BlockPos) -> Result<Self, LoadError> {
        Self::load_with_limit(world_dir, min, max, SNAPSHOT_BYTES)
    }
    /// Limit covers the dense cell allocation; shared model data and the reader's
    /// separately bounded cache and temporary parsing buffers are not included.
    pub fn load_with_limit(world_dir: impl AsRef<Path>, min: BlockPos, max: BlockPos, max_bytes: usize) -> Result<Self, LoadError> {
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
        let registry = models();
        let mut cells = vec![0u16;count].into_boxed_slice();
        let mut fluids = false;
        let mut inside = false;
        // Chunk order avoids region rereads for every interleaved row of a large box.
        for cz in min[2].div_euclid(16)..=max[2].div_euclid(16) {
            for cx in min[0].div_euclid(16)..=max[0].div_euclid(16) {
                for z in min[2].max(cz*16)..=max[2].min(cz*16+15) {
                    for y in min[1]..=max[1] {
                        for x in min[0].max(cx*16)..=max[0].min(cx*16+15) {
                            let p = [x,y,z];
                            let state = reader.block_state(p)?;
                            let id = lookup(state).map_err(|reason| LoadError::Unmodeled { position: p, state: state.clone(), reason })?;
                            let block = &registry.blocks[id as usize];
                            fluids |= block.fluid.kind != FluidKind::Empty;
                            inside |= block.inside != Inside::None || block.stuck_multiplier.is_some();
                            cells[index(min,widths,p)] = id;
                        }
                    }
                }
            }
        }
        Ok(Self { min, max, widths, cells, models: registry, difficulty: reader.difficulty, natural_regeneration, fluids, inside })
    }
    pub fn bounds(&self) -> (BlockPos, BlockPos) { (self.min,self.max) }
    fn covers(&self, p: BlockPos) -> bool { (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i]) }
    fn assert_covered(&self, p: BlockPos) {
        assert!(self.covers(p), "snapshot query {p:?} outside validated box {:?}..{:?}; load the law's entire read footprint",self.min,self.max);
    }
}
fn index(min: BlockPos, widths: [usize;3], p: BlockPos) -> usize {
    ((p[2] as i64-min[2] as i64) as usize * widths[1] + (p[1] as i64-min[1] as i64) as usize)*widths[0] + (p[0] as i64-min[0] as i64) as usize
}
fn check_version(root: nbt::Tag<'_>, expected: i32, file: &str) -> Result<(),LoadError> {
    let found = root.required("DataVersion")?.int()?;
    if expected != found { return Err(LoadError::Invalid(format!("{file} DataVersion {found} differs from level.dat {expected}"))); }
    Ok(())
}
impl World for WorldSnapshot {
    fn block(&self, p: BlockPos) -> &Block {
        self.assert_covered(p);
        &self.models.blocks[self.cells[index(self.min,self.widths,p)] as usize]
    }
    fn loaded(&self, p: BlockPos) -> bool { self.assert_covered(p); true }
    fn model_id(&self, p: BlockPos) -> Option<u16> { self.assert_covered(p); Some(self.cells[index(self.min,self.widths,p)]) }
    fn min_y(&self) -> i32 { -64 }
    fn difficulty(&self) -> u8 { self.difficulty }
    fn natural_regeneration(&self) -> bool { self.natural_regeneration }
    fn has_fluid(&self) -> bool { self.fluids }
    fn has_inside_effects(&self) -> bool { self.inside || self.fluids }
    /// The law's order (z, y, x), from the cells' own models: a cell that never collides
    /// is left out, and one whose collision stays in its cell is left out of the halo,
    /// where it cannot meet `bounds`.
    fn visit_collision_cells(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
        let near_lo = [bounds.min.x, bounds.min.y, bounds.min.z].map(law::math::floor);
        let near_hi = [bounds.max.x, bounds.max.y, bounds.max.z].map(law::math::floor);
        let near = |a: usize, v: i32| v >= near_lo[a] && v <= near_hi[a];
        if !self.covers(lo) || !self.covers(hi) {
            // The full scan asks every cell in this order and stops only at a visit, so it
            // meets the first cell outside the snapshot exactly where this does.
            for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] {
                self.assert_covered([x, y, z]);
                let reach = self.models.reach[self.cells[index(self.min, self.widths, [x, y, z])] as usize];
                if (reach == BEYOND || reach == OWN_CELL && near(2, z) && near(1, y) && near(0, x)) && !visit([x, y, z]) { return false; }
            } } }
            return true;
        }
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                let row = index(self.min, self.widths, [lo[0], y, z]);
                let near_row = near(2, z) && near(1, y);
                for x in lo[0]..=hi[0] {
                    let reach = self.models.reach[self.cells[row + (x - lo[0]) as usize] as usize];
                    if (reach == BEYOND || reach == OWN_CELL && near_row && near(0, x)) && !visit([x, y, z]) { return false; }
                }
            }
        }
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
}

