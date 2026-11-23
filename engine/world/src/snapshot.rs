use crate::{ BlockState, LoadError, WorldReader, nbt, region::metadata};
use law::{
    basis::cell::{self, Models, models, BEYOND, OWN_CELL},
    block_hooks::get_seed,
    Aabb, Block, BlockPos, FluidKind, Inside, World,
};
use std::path::Path;

const SNAPSHOT_BYTES: usize = 128 * 1024 * 1024;
/// Project the state onto its block's relevant properties, then look the values up (the law's table).
fn lookup(state: &BlockState) -> Result<u16, String> {
    cell::lookup(&state.name, |p| state.properties.iter().find(|(k, _)| k == p).map(|(_, v)| v.as_str()))
}
/// The shared model table, indexed by model id. Every admitted state maps to one row.
pub fn model_table() -> &'static [Block] { &models().blocks }
/// The law's model of a block state, or the reason the generated rule refuses it.
pub fn model(state: &BlockState) -> Result<&'static Block, String> {
    lookup(state).map(|id| &models().blocks[id as usize])
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
        let id = self.cells[index(self.min, self.widths, p)] as usize;
        if let Some(variants) = &self.models.offset_variants[id] {
            let l = get_seed(p[0], 0, p[2]);
            let key = (((l & 15) as usize) << 4) | (((l >> 8) & 15) as usize);
            return &variants[key];
        }
        &self.models.blocks[id]
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
}

