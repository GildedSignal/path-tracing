use crate::{ LoadError, WorldReader, nbt, region::metadata};
use law::{Aabb, Block, BlockPos, ContactDamage, Fluid, FluidFlowBlock, FluidKind, GroundedContact, Inside, VoxelShape, World};
use std::{collections::BTreeMap, path::Path, sync::OnceLock};

const SNAPSHOT_BYTES: usize = 128 * 1024 * 1024;
struct Models { states: BTreeMap<&'static str, u16>, blocks: Vec<Block> }
fn models() -> &'static Models {
    static MODELS: OnceLock<Models> = OnceLock::new();
    MODELS.get_or_init(|| {
        let blocks = include_str!("../data/models.tsv").lines().map(parse_model).collect::<Vec<_>>();
        let states = include_str!("../data/states.tsv").lines().map(|line| {
            let (state, id) = line.split_once('\t').expect("generated state row");
            let id: u16 = id.parse().expect("generated model id");
            assert!((id as usize) < blocks.len());
            (state, id)
        }).collect();
        Models { states, blocks }
    })
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
        resets_fall_distance: flag(10), end_portal_or_gateway: flag(11),
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
                            let key = state.to_string();
                            let id = *registry.states.get(key.as_str()).ok_or_else(|| LoadError::Unmodeled { position: p, state: state.clone() })?;
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
    fn assert_covered(&self, p: BlockPos) {
        assert!((0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i]), "snapshot query {p:?} outside validated box {:?}..{:?}; load the law's entire read footprint",self.min,self.max);
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
    fn min_y(&self) -> i32 { -64 }
    fn difficulty(&self) -> u8 { self.difficulty }
    fn natural_regeneration(&self) -> bool { self.natural_regeneration }
    fn has_fluid(&self) -> bool { self.fluids }
    fn has_inside_effects(&self) -> bool { self.inside || self.fluids }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn block(state:&str)->&'static Block {
        let registry=models();
        &registry.blocks[*registry.states.get(state).expect("supported state") as usize]
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

