//! Living mirror of the server's loaded chunks + cached `.mca` disk terrain, implementing `law::World` and `plan::source::CorridorSource`.
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
    path::Path,
};

use law::{
    Aabb, Block, BlockPos, PlayerState, World,
    codec::{
        self, Canonical, Interner, LabelId, LegsRole,
        proof::{Refusal, Source},
    },
    footprint::EnvironmentRead,
};
use plan::source::{CorridorProof, CorridorResult, CorridorSource};

use crate::disk_world::{DiskWorld, RawDiskSection};

#[derive(Default)]
pub struct FxHasher(u64);

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        const K: u64 = 0x517cc1b727220a95;
        let mut h = self.0;
        let mut i = 0;
        while i + 8 <= bytes.len() {
            let v = u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            h = (h.rotate_left(5) ^ v).wrapping_mul(K);
            i += 8;
        }
        if i + 4 <= bytes.len() {
            let v = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as u64;
            h = (h.rotate_left(5) ^ v).wrapping_mul(K);
            i += 4;
        }
        while i < bytes.len() {
            h = (h.rotate_left(5) ^ (bytes[i] as u64)).wrapping_mul(K);
            i += 1;
        }
        self.0 = h;
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.0 = (self.0.rotate_left(5) ^ (i as u32 as u64)).wrapping_mul(0x517cc1b727220a95);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxHashSet<K> = HashSet<K, FxBuildHasher>;

#[inline]
pub fn is_hooked_block(b: &Block) -> bool {
    b.world_hook
        || b.inside != law::Inside::None
        || b.grounded_contact != law::GroundedContact::None
        || b.fluid.kind != law::FluidKind::Empty
        || b.stuck_multiplier.is_some()
}

#[inline]
fn has_fluid_block(b: &Block) -> bool {
    b.fluid.kind != law::FluidKind::Empty
}

#[inline]
fn has_inside_block(b: &Block) -> bool {
    b.inside != law::Inside::None || b.stuck_multiplier.is_some()
}

fn same_model_result(a: &Result<&'static Block, String>, b: &Result<&'static Block, String>) -> bool {
    match (a, b) {
        (Ok(ba), Ok(bb)) => std::ptr::eq(*ba, *bb),
        (Err(ea), Err(eb)) => ea == eb,
        _ => false,
    }
}

#[derive(Clone)]
struct PaletteSection {
    palette: Vec<Result<&'static Block, String>>,
    indices: Box<[u16; 4096]>,
    uniform_y: [Option<u16>; 16],
    has_fluid: bool,
    has_inside: bool,
    has_hooked: bool,
}

impl PaletteSection {
    fn from_raw(palette: Vec<Result<&'static Block, String>>, indices: Box<[u16; 4096]>) -> Self {
        let mut s = Self {
            palette,
            indices,
            uniform_y: [None; 16],
            has_fluid: false,
            has_inside: false,
            has_hooked: false,
        };
        s.recompute_meta();
        s
    }

    fn recompute_meta(&mut self) {
        for ly in 0..16 {
            let base = ly * 256;
            let first = self.indices[base];
            self.uniform_y[ly] = if self.indices[base + 1..base + 256]
                .iter()
                .all(|&v| v == first)
            {
                Some(first)
            } else {
                None
            };
        }
        let mut used = vec![false; self.palette.len()];
        for &idx in self.indices.iter() {
            if let Some(u) = used.get_mut(idx as usize) {
                *u = true;
            }
        }
        let mut has_fluid = false;
        let mut has_inside = false;
        let mut has_hooked = false;
        for (i, entry) in self.palette.iter().enumerate() {
            if used[i]
                && let Ok(b) = entry
            {
                has_fluid |= has_fluid_block(b);
                has_inside |= has_inside_block(b);
                has_hooked |= is_hooked_block(b);
            }
        }
        self.has_fluid = has_fluid;
        self.has_inside = has_inside;
        self.has_hooked = has_hooked;
    }
}

enum Section {
    AllAir,
    Layered {
        layers: Box<[Result<&'static Block, String>; 16]>,
        has_fluid: bool,
        has_inside: bool,
        has_hooked: bool,
    },
    Palette(PaletteSection),
}

impl Section {
    fn layered(layers: Box<[Result<&'static Block, String>; 16]>) -> Self {
        let mut has_fluid = false;
        let mut has_inside = false;
        let mut has_hooked = false;
        for entry in layers.iter() {
            if let Ok(b) = entry {
                has_fluid |= has_fluid_block(b);
                has_inside |= has_inside_block(b);
                has_hooked |= is_hooked_block(b);
            }
        }
        Self::Layered {
            layers,
            has_fluid,
            has_inside,
            has_hooked,
        }
    }

    #[inline]
    fn flags(&self) -> (bool, bool, bool) {
        match self {
            Self::AllAir => (false, false, false),
            Self::Layered {
                has_fluid,
                has_inside,
                has_hooked,
                ..
            } => (*has_fluid, *has_inside, *has_hooked),
            Self::Palette(ps) => (ps.has_fluid, ps.has_inside, ps.has_hooked),
        }
    }
}

struct SectionEntry {
    section: Section,
    from_disk: bool,
}

impl SectionEntry {
    #[inline]
    fn counts(&self) -> (usize, usize, usize) {
        let (fl, ins, _) = self.section.flags();
        (usize::from(fl), usize::from(ins), usize::from(self.from_disk))
    }
}

pub struct WorldMirror {
    pub session: u64,
    difficulty: u8,
    natural_regeneration: bool,
    sections: FxHashMap<[i32; 3], SectionEntry>,
    server_loaded_chunks: FxHashSet<[i32; 2]>,
    disk_world: Option<DiskWorld>,
    air_block: &'static Block,
    model_cache: HashMap<String, Result<&'static Block, String>>,
    table_ptr: usize,
    table_len: usize,
    fluid_section_count: usize,
    inside_section_count: usize,
    disk_section_count: usize,
    edit_rev: u64,
}

impl WorldMirror {
    pub fn new() -> Self {
        let table = world::model_table();
        let air_block = world::model(&world::BlockState {
            name: "minecraft:air".to_owned(),
            properties: Vec::new(),
        })
        .expect("minecraft:air in model table");
        let mut model_cache = HashMap::new();
        model_cache.insert("minecraft:air".to_owned(), Ok(air_block));
        Self {
            session: 0,
            difficulty: 0,
            natural_regeneration: true,
            sections: FxHashMap::default(),
            server_loaded_chunks: FxHashSet::default(),
            disk_world: None,
            air_block,
            model_cache,
            table_ptr: table.as_ptr() as usize,
            table_len: table.len(),
            fluid_section_count: 0,
            inside_section_count: 0,
            disk_section_count: 0,
            edit_rev: 1,
        }
    }

    pub fn reset(&mut self, session: u64, difficulty: u8, natural_regen: bool) {
        self.session = session;
        self.difficulty = difficulty;
        self.natural_regeneration = natural_regen;
        self.sections.clear();
        self.server_loaded_chunks.clear();
        self.fluid_section_count = 0;
        self.inside_section_count = 0;
        self.disk_section_count = 0;
        self.edit_rev = self.edit_rev.wrapping_add(1).max(1);
    }

    pub fn set_world_dir(&mut self, world_dir: &Path) {
        self.disk_world = Some(DiskWorld::open(world_dir));
    }

    pub fn set_rules(&mut self, difficulty: u8, natural_regen: bool) {
        self.difficulty = difficulty;
        self.natural_regeneration = natural_regen;
        self.edit_rev = self.edit_rev.wrapping_add(1).max(1);
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    pub fn disk_section_count(&self) -> usize {
        self.disk_section_count
    }

    #[inline]
    pub fn is_server_loaded(&self, cx: i32, cz: i32) -> bool {
        self.server_loaded_chunks.contains(&[cx, cz])
    }

    fn insert_section(&mut self, key: [i32; 3], entry: SectionEntry) {
        let (nf, ni, nd) = entry.counts();
        if let Some(old) = self.sections.insert(key, entry) {
            let (of, oi, od) = old.counts();
            self.fluid_section_count = self.fluid_section_count + nf - of;
            self.inside_section_count = self.inside_section_count + ni - oi;
            self.disk_section_count = self.disk_section_count + nd - od;
        } else {
            self.fluid_section_count += nf;
            self.inside_section_count += ni;
            self.disk_section_count += nd;
        }
    }

    fn resolve_model(&mut self, text: &str) -> Result<&'static Block, String> {
        if let Some(hit) = self.model_cache.get(text) {
            return hit.clone();
        }
        let (name, tail) = text.split_once('[').unwrap_or((text, ""));
        let mut properties = Vec::new();
        if !tail.is_empty() {
            for pair in tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()) {
                if let Some((k, v)) = pair.split_once('=') {
                    properties.push((k.to_owned(), v.to_owned()));
                } else {
                    let err = Err(format!("invalid block property in {text}"));
                    self.model_cache.insert(text.to_owned(), err.clone());
                    return err;
                }
            }
        }
        let res = world::model(&world::BlockState {
            name: name.to_owned(),
            properties,
        });
        self.model_cache.insert(text.to_owned(), res.clone());
        res
    }

    pub fn set_section_air(&mut self, cx: i32, cy: i32, cz: i32) {
        self.server_loaded_chunks.insert([cx, cz]);
        self.insert_section(
            [cx, cy, cz],
            SectionEntry {
                section: Section::AllAir,
                from_disk: false,
            },
        );
        self.edit_rev = self.edit_rev.wrapping_add(1).max(1);
    }

    pub fn set_section_rle(
        &mut self,
        cx: i32,
        cy: i32,
        cz: i32,
        palette_names: &[&str],
        runs: &[(u16, u16)],
    ) -> Result<(), String> {
        let mut palette = Vec::with_capacity(palette_names.len());
        for &name in palette_names {
            palette.push(self.resolve_model(name));
        }
        let mut indices = Box::new([0u16; 4096]);
        let mut cursor = 0usize;
        for &(run_len, pidx) in runs {
            let pidx_usize = pidx as usize;
            if pidx_usize >= palette.len() {
                return Err("palette index out of range".into());
            }
            let end = cursor + run_len as usize;
            if end > 4096 {
                return Err("RLE section exceeds 4096 blocks".into());
            }
            indices[cursor..end].fill(pidx);
            cursor = end;
        }
        if cursor != 4096 {
            return Err(format!("RLE section incomplete: {cursor}/4096"));
        }

        let mut all_layers_uniform = true;
        let mut layer_ids = [0usize; 16];
        for (ly, lid) in layer_ids.iter_mut().enumerate() {
            let base = ly * 256;
            let first = indices[base] as usize;
            if indices[base + 1..base + 256]
                .iter()
                .all(|&v| v as usize == first)
            {
                *lid = first;
            } else {
                all_layers_uniform = false;
                break;
            }
        }

        let section = if all_layers_uniform {
            if layer_ids.iter().all(|&id| {
                palette[id]
                    .as_ref()
                    .is_ok_and(|&b| std::ptr::eq(b, self.air_block))
            }) {
                Section::AllAir
            } else {
                let layers: Box<[Result<&'static Block, String>; 16]> =
                    Box::new(std::array::from_fn(|ly| palette[layer_ids[ly]].clone()));
                Section::layered(layers)
            }
        } else {
            Section::Palette(PaletteSection::from_raw(palette, indices))
        };

        self.server_loaded_chunks.insert([cx, cz]);
        self.insert_section(
            [cx, cy, cz],
            SectionEntry {
                section,
                from_disk: false,
            },
        );
        self.edit_rev = self.edit_rev.wrapping_add(1).max(1);
        Ok(())
    }

    pub fn ensure_chunks_from_disk(&mut self, coords: &[[i32; 2]]) -> usize {
        if coords.is_empty() || self.disk_world.is_none() {
            return 0;
        }
        let mut seen = FxHashSet::default();
        let mut missing = Vec::new();
        for &c in coords {
            if seen.insert(c) && !self.sections.contains_key(&[c[0], -4, c[1]]) {
                missing.push(c);
            }
        }
        if missing.is_empty() {
            return 0;
        }
        let loaded = self
            .disk_world
            .as_mut()
            .unwrap()
            .load_chunks_parallel(&missing, self.air_block);
        let mut inserted_chunks = 0usize;
        for (coord, secs) in loaded {
            if self.sections.contains_key(&[coord[0], -4, coord[1]]) {
                continue;
            }
            inserted_chunks += 1;
            for (cy, raw) in secs {
                let key = [coord[0], cy, coord[1]];
                if self.sections.contains_key(&key) {
                    continue;
                }
                let section = match raw {
                    RawDiskSection::AllAir => Section::AllAir,
                    RawDiskSection::Layered(layers) => Section::layered(Box::new(layers)),
                    RawDiskSection::Palette { palette, indices } => {
                        Section::Palette(PaletteSection::from_raw(palette, indices))
                    }
                };
                self.insert_section(
                    key,
                    SectionEntry {
                        section,
                        from_disk: true,
                    },
                );
            }
        }
        if inserted_chunks > 0 {
            self.edit_rev = self.edit_rev.wrapping_add(1).max(1);
        }
        inserted_chunks
    }

    pub fn set_block_delta(&mut self, p: BlockPos, state_name: &str) -> bool {
        let sec_key = [p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)];
        let ly = p[1].rem_euclid(16) as usize;
        let lz = p[2].rem_euclid(16) as usize;
        let lx = p[0].rem_euclid(16) as usize;
        let cell_idx = (ly * 16 + lz) * 16 + lx;
        let model = self.resolve_model(state_name);

        let Some(entry) = self.sections.get_mut(&sec_key) else {
            return false;
        };
        let (of, oi, _) = entry.counts();
        let changed = match &mut entry.section {
            Section::AllAir => {
                if let Ok(b) = model
                    && std::ptr::eq(b, self.air_block)
                {
                    return false;
                }
                let mut indices = Box::new([0u16; 4096]);
                indices[cell_idx] = 1;
                entry.section = Section::Palette(PaletteSection::from_raw(
                    vec![Ok(self.air_block), model],
                    indices,
                ));
                true
            }
            Section::Layered { layers, .. } => {
                let old_entry = &layers[ly];
                if same_model_result(old_entry, &model) {
                    return false;
                }
                let mut palette: Vec<Result<&'static Block, String>> = Vec::with_capacity(4);
                let mut indices = Box::new([0u16; 4096]);
                for y in 0..16 {
                    let layer_model = &layers[y];
                    let pid = if let Some(pos) =
                        palette.iter().position(|e| same_model_result(e, layer_model))
                    {
                        pos as u16
                    } else {
                        let id = palette.len() as u16;
                        palette.push(layer_model.clone());
                        id
                    };
                    indices[y * 256..(y + 1) * 256].fill(pid);
                }
                let new_pid = if let Some(pos) =
                    palette.iter().position(|e| same_model_result(e, &model))
                {
                    pos as u16
                } else {
                    let id = palette.len() as u16;
                    palette.push(model);
                    id
                };
                indices[cell_idx] = new_pid;
                entry.section = Section::Palette(PaletteSection::from_raw(palette, indices));
                true
            }
            Section::Palette(ps) => {
                let old_id = ps.indices[cell_idx] as usize;
                let changed = !same_model_result(&ps.palette[old_id], &model);
                let target_id = if let Some(pos) =
                    ps.palette.iter().position(|e| same_model_result(e, &model))
                {
                    pos as u16
                } else {
                    let id = ps.palette.len() as u16;
                    ps.palette.push(model);
                    id
                };
                ps.indices[cell_idx] = target_id;
                if changed {
                    ps.recompute_meta();
                }
                changed
            }
        };

        if changed {
            let (nf, ni, _) = entry.counts();
            self.fluid_section_count = self.fluid_section_count + nf - of;
            self.inside_section_count = self.inside_section_count + ni - oi;
            self.edit_rev = self.edit_rev.wrapping_add(1).max(1);
        }
        changed
    }

    pub fn unload_chunk(&mut self, cx: i32, cz: i32) {
        self.server_loaded_chunks.remove(&[cx, cz]);
    }

    pub fn min_loaded_y(&self) -> i32 {
        -64
    }

    pub fn max_loaded_y(&self) -> i32 {
        319
    }

    fn cell_result(&self, p: BlockPos) -> Option<&Result<&'static Block, String>> {
        let entry = self
            .sections
            .get(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)])?;
        match &entry.section {
            Section::AllAir => None,
            Section::Layered { layers, .. } => {
                Some(&layers[p[1].rem_euclid(16) as usize])
            }
            Section::Palette(ps) => {
                let idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16
                    + p[0].rem_euclid(16)) as usize;
                Some(&ps.palette[ps.indices[idx] as usize])
            }
        }
    }

    pub fn cell_block(&self, p: BlockPos) -> Result<&'static Block, String> {
        let Some(entry) = self
            .sections
            .get(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)])
        else {
            return Err(format!("unloaded cell {p:?}"));
        };
        match &entry.section {
            Section::AllAir => Ok(self.air_block),
            Section::Layered { layers, .. } => layers[p[1].rem_euclid(16) as usize].clone(),
            Section::Palette(ps) => {
                let idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16
                    + p[0].rem_euclid(16)) as usize;
                ps.palette[ps.indices[idx] as usize].clone()
            }
        }
    }

    pub fn check_corridor_valid(&self, lo: BlockPos, hi: BlockPos) -> Result<(), String> {
        let sx0 = lo[0].div_euclid(16);
        let sx1 = hi[0].div_euclid(16);
        let sy0 = lo[1].div_euclid(16);
        let sy1 = hi[1].div_euclid(16);
        let sz0 = lo[2].div_euclid(16);
        let sz1 = hi[2].div_euclid(16);
        for sz in sz0..=sz1 {
            for sy in sy0..=sy1 {
                for sx in sx0..=sx1 {
                    let Some(entry) = self.sections.get(&[sx, sy, sz]) else {
                        return Err(format!("unloaded section [{sx}, {sy}, {sz}]"));
                    };
                    let x0 = (lo[0] - sx * 16).max(0) as usize;
                    let x1 = (hi[0] - sx * 16).min(15) as usize;
                    let y0 = (lo[1] - sy * 16).max(0) as usize;
                    let y1 = (hi[1] - sy * 16).min(15) as usize;
                    let z0 = (lo[2] - sz * 16).max(0) as usize;
                    let z1 = (hi[2] - sz * 16).min(15) as usize;
                    match &entry.section {
                        Section::AllAir => {}
                        Section::Layered { layers, .. } => {
                            for ly in y0..=y1 {
                                if let Err(why) = &layers[ly] {
                                    let p = [sx * 16 + x0 as i32, sy * 16 + ly as i32, sz * 16 + z0 as i32];
                                    return Err(format!("unmodeled block at {p:?}: {why}"));
                                }
                            }
                        }
                        Section::Palette(ps) => {
                            for ly in y0..=y1 {
                                if let Some(u) = ps.uniform_y[ly] {
                                    if let Err(why) = &ps.palette[u as usize] {
                                        let p = [sx * 16 + x0 as i32, sy * 16 + ly as i32, sz * 16 + z0 as i32];
                                        return Err(format!("unmodeled block at {p:?}: {why}"));
                                    }
                                    continue;
                                }
                                for lz in z0..=z1 {
                                    let row = (ly * 16 + lz) * 16;
                                    for lx in x0..=x1 {
                                        if let Err(why) = &ps.palette[ps.indices[row + lx] as usize] {
                                            let p = [sx * 16 + lx as i32, sy * 16 + ly as i32, sz * 16 + lz as i32];
                                            return Err(format!("unmodeled block at {p:?}: {why}"));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn is_air(&self, p: BlockPos) -> bool {
        self.cell_block(p).map(|b| b.air).unwrap_or(true)
    }

    pub fn corridor_has_hooked(&self, lo: BlockPos, hi: BlockPos) -> Option<BlockPos> {
        let sx0 = lo[0].div_euclid(16);
        let sx1 = hi[0].div_euclid(16);
        let sy0 = lo[1].div_euclid(16);
        let sy1 = hi[1].div_euclid(16);
        let sz0 = lo[2].div_euclid(16);
        let sz1 = hi[2].div_euclid(16);
        let mut first: Option<BlockPos> = None;
        for sz in sz0..=sz1 {
            for sy in sy0..=sy1 {
                for sx in sx0..=sx1 {
                    let Some(entry) = self.sections.get(&[sx, sy, sz]) else {
                        continue;
                    };
                    if !entry.section.flags().2 {
                        continue;
                    }
                    let x0 = (lo[0] - sx * 16).max(0) as usize;
                    let x1 = (hi[0] - sx * 16).min(15) as usize;
                    let y0 = (lo[1] - sy * 16).max(0) as usize;
                    let y1 = (hi[1] - sy * 16).min(15) as usize;
                    let z0 = (lo[2] - sz * 16).max(0) as usize;
                    let z1 = (hi[2] - sz * 16).min(15) as usize;
                    match &entry.section {
                        Section::AllAir => {}
                        Section::Layered { layers, .. } => {
                            for ly in y0..=y1 {
                                if let Ok(b) = layers[ly]
                                    && is_hooked_block(b)
                                {
                                    let p = [sx * 16 + x0 as i32, sy * 16 + ly as i32, sz * 16 + z0 as i32];
                                    if first.is_none_or(|f| p < f) {
                                        first = Some(p);
                                    }
                                }
                            }
                        }
                        Section::Palette(ps) => {
                            for ly in y0..=y1 {
                                if let Some(u) = ps.uniform_y[ly] {
                                    if let Ok(b) = ps.palette[u as usize]
                                        && is_hooked_block(b)
                                    {
                                        let p = [sx * 16 + x0 as i32, sy * 16 + ly as i32, sz * 16 + z0 as i32];
                                        if first.is_none_or(|f| p < f) {
                                            first = Some(p);
                                        }
                                    }
                                    continue;
                                }
                                for lz in z0..=z1 {
                                    let row = (ly * 16 + lz) * 16;
                                    for lx in x0..=x1 {
                                        if let Ok(b) = ps.palette[ps.indices[row + lx] as usize]
                                            && is_hooked_block(b)
                                        {
                                            let p = [sx * 16 + lx as i32, sy * 16 + ly as i32, sz * 16 + lz as i32];
                                            if first.is_none_or(|f| p < f) {
                                                first = Some(p);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        first
    }

    fn provenance_version(&self) -> [u8; 32] {
        let mut bytes = Vec::with_capacity(48);
        bytes.extend(codec::SCHEMA);
        bytes.extend(self.session.to_le_bytes());
        bytes.extend(self.edit_rev.to_le_bytes());
        let mut out = [0u8; 32];
        out[..16].copy_from_slice(&codec::digest(&bytes).to_le_bytes());
        bytes.push(1);
        out[16..].copy_from_slice(&codec::digest(&bytes).to_le_bytes());
        out
    }

    fn slice_has_collision_in(&self, sx: i32, sy: i32, sz: i32, ly: usize) -> bool {
        let Some(entry) = self.sections.get(&[sx, sy, sz]) else {
            return true;
        };
        match &entry.section {
            Section::AllAir => false,
            Section::Layered { layers, .. } => match &layers[ly] {
                Ok(b) => !b.never_collides(),
                Err(_) => true,
            },
            Section::Palette(ps) => {
                if let Some(u) = ps.uniform_y[ly] {
                    match &ps.palette[u as usize] {
                        Ok(b) => !b.never_collides(),
                        Err(_) => true,
                    }
                } else {
                    true
                }
            }
        }
    }
}

impl World for WorldMirror {
    fn block(&self, p: BlockPos) -> &Block {
        let Some(entry) = self
            .sections
            .get(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)])
        else {
            panic!("world mirror unloaded cell {p:?}");
        };
        match &entry.section {
            Section::AllAir => self.air_block,
            Section::Layered { layers, .. } => match &layers[p[1].rem_euclid(16) as usize] {
                Ok(block) => block,
                Err(why) => panic!("world mirror unmodeled cell {p:?}: {why}"),
            },
            Section::Palette(ps) => {
                let idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16
                    + p[0].rem_euclid(16)) as usize;
                match &ps.palette[ps.indices[idx] as usize] {
                    Ok(block) => block,
                    Err(why) => panic!("world mirror unmodeled cell {p:?}: {why}"),
                }
            }
        }
    }

    fn model_id(&self, p: BlockPos) -> Option<u16> {
        let block = match self.cell_result(p) {
            Some(Ok(b)) => *b,
            None if self.loaded(p) => self.air_block,
            _ => return None,
        };
        let ptr = (block as *const Block) as usize;
        let stride = std::mem::size_of::<Block>();
        if ptr >= self.table_ptr {
            let idx = (ptr - self.table_ptr) / stride;
            if idx < self.table_len {
                return Some(idx as u16);
            }
        }
        None
    }

    fn loaded(&self, p: BlockPos) -> bool {
        self.sections
            .contains_key(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)])
    }

    fn difficulty(&self) -> u8 {
        self.difficulty
    }

    fn natural_regeneration(&self) -> bool {
        self.natural_regeneration
    }

    fn min_y(&self) -> i32 {
        self.min_loaded_y()
    }

    fn has_fluid(&self) -> bool {
        self.fluid_section_count > 0
    }

    fn has_inside_effects(&self) -> bool {
        self.inside_section_count > 0
    }

    fn visit_collision_cells(
        &self,
        _bounds: Aabb,
        lo: BlockPos,
        hi: BlockPos,
        mut visit: impl FnMut(BlockPos) -> bool,
    ) -> bool {
        for z in lo[2]..=hi[2] {
            let sz = z.div_euclid(16);
            for y in lo[1]..=hi[1] {
                let sy = y.div_euclid(16);
                let ly = y.rem_euclid(16) as usize;
                let mut cur_sx = i32::MIN;
                let mut active = true;
                for x in lo[0]..=hi[0] {
                    let sx = x.div_euclid(16);
                    if sx != cur_sx {
                        cur_sx = sx;
                        active = self.slice_has_collision_in(sx, sy, sz, ly);
                    }
                    if active && !visit([x, y, z]) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Source for WorldMirror {
    fn cell(&self, _tick: u64, p: BlockPos, mask: u8) -> Result<Canonical, Refusal> {
        if !self.loaded(p) {
            return Err(Refusal::Coverage);
        }
        codec::cell(self.block(p), true, mask).map_err(Refusal::Law)
    }

    fn environment(&self, _tick: u64, q: &EnvironmentRead) -> Result<Canonical, Refusal> {
        codec::environment(self, q).map_err(Refusal::Law)
    }
}

fn legs_differs(a: &Block, b: &Block) -> Result<bool, law::pattern::PatternError> {
    if std::ptr::eq(a, b) {
        return Ok(false);
    }
    for role in [
        LegsRole::Support02,
        LegsRole::Support05,
        LegsRole::BodyFeet,
        LegsRole::Inside,
    ] {
        if codec::legs_bytes(a, true, role)? != codec::legs_bytes(b, true, role)? {
            return Ok(true);
        }
    }
    Ok(false)
}

impl CorridorSource for WorldMirror {
    fn support_cell(&self, s: &PlayerState) -> BlockPos {
        s.main_supporting_block
            .filter(|&c| self.loaded(c) && !self.is_air(c))
            .unwrap_or_else(|| {
                let p = [
                    s.position.x.floor() as i32,
                    s.position.y.floor() as i32,
                    s.position.z.floor() as i32,
                ];
                (1..=8)
                    .map(|d| [p[0], p[1] - d, p[2]])
                    .find(|&c| self.loaded(c) && !self.is_air(c))
                    .unwrap_or([p[0], p[1] - 1, p[2]])
            })
    }

    fn entry_label(&self, codec: &mut Interner, state: &PlayerState) -> Result<LabelId, Refusal> {
        let feet = [
            state.position.x.floor() as i32,
            state.position.y.floor() as i32,
            state.position.z.floor() as i32,
        ];
        let support = state
            .main_supporting_block
            .unwrap_or([feet[0], feet[1] - 1, feet[2]]);
        let mut classes = [None; 4];
        for (i, (p, role)) in [
            (support, LegsRole::Support02),
            (support, LegsRole::Support05),
            (feet, LegsRole::BodyFeet),
            (feet, LegsRole::Inside),
        ]
        .into_iter()
        .enumerate()
        {
            if !self.loaded(p) {
                return Err(Refusal::Loaded);
            }
            classes[i] = Some(codec.legs(self.block(p), true, role).map_err(Refusal::Law)?);
        }
        let classes = classes.map(Option::unwrap);
        codec.bare_legs_label(classes).ok_or(Refusal::Schema)
    }

    fn corridor_query(
        &self,
        codec: &mut Interner,
        expected: LabelId,
        min: BlockPos,
        max: BlockPos,
    ) -> CorridorResult {
        if min[0] > max[0] || min[1] > max[1] || min[2] > max[2] {
            return CorridorResult::Unknown(Refusal::StateOrContext);
        }
        let sx0 = min[0].div_euclid(16);
        let sx1 = max[0].div_euclid(16);
        let sy0 = min[1].div_euclid(16);
        let sy1 = max[1].div_euclid(16);
        let sz0 = min[2].div_euclid(16);
        let sz1 = max[2].div_euclid(16);
        for sz in sz0..=sz1 {
            for sy in sy0..=sy1 {
                for sx in sx0..=sx1 {
                    if !self.sections.contains_key(&[sx, sy, sz]) {
                        return CorridorResult::Unknown(Refusal::Loaded);
                    }
                }
            }
        }

        let support_pos = [min[0], min[1], min[2]];
        let Ok(floor_block) = self.cell_block(support_pos) else {
            return CorridorResult::Unknown(Refusal::StateOrContext);
        };
        let mut classes = [None; 4];
        for (i, (b, role)) in [
            (floor_block, LegsRole::Support02),
            (floor_block, LegsRole::Support05),
            (self.air_block, LegsRole::BodyFeet),
            (self.air_block, LegsRole::Inside),
        ]
        .into_iter()
        .enumerate()
        {
            match codec.legs(b, true, role) {
                Ok(c) => classes[i] = Some(c),
                Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
            }
        }
        let Some(base_label) = codec.bare_legs_label(classes.map(Option::unwrap)) else {
            return CorridorResult::Unknown(Refusal::Schema);
        };
        if base_label != expected {
            return CorridorResult::Exception {
                first_pos: support_pos,
                exceptions: 1,
            };
        }

        let mut first: Option<BlockPos> = None;
        let mut exceptions = 0usize;
        let mut record = |p: BlockPos, count: usize| {
            exceptions += count;
            if first.is_none_or(|f| p < f) {
                first = Some(p);
            }
        };

        for sz in sz0..=sz1 {
            let z0 = (min[2] - sz * 16).max(0) as usize;
            let z1 = (max[2] - sz * 16).min(15) as usize;
            let nz = z1 - z0 + 1;
            for sx in sx0..=sx1 {
                let x0 = (min[0] - sx * 16).max(0) as usize;
                let x1 = (max[0] - sx * 16).min(15) as usize;
                let nx = x1 - x0 + 1;
                for sy in sy0..=sy1 {
                    let y0 = (min[1] - sy * 16).max(0) as usize;
                    let y1 = (max[1] - sy * 16).min(15) as usize;
                    let entry = self.sections.get(&[sx, sy, sz]).unwrap();
                    match &entry.section {
                        Section::AllAir => {
                            for ly in y0..=y1 {
                                let wy = sy * 16 + ly as i32;
                                let expected_b = if wy == min[1] { floor_block } else { self.air_block };
                                match legs_differs(self.air_block, expected_b) {
                                    Ok(false) => {}
                                    Ok(true) => record(
                                        [sx * 16 + x0 as i32, wy, sz * 16 + z0 as i32],
                                        nx * nz,
                                    ),
                                    Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
                                }
                            }
                        }
                        Section::Layered { layers, .. } => {
                            for ly in y0..=y1 {
                                let wy = sy * 16 + ly as i32;
                                let expected_b = if wy == min[1] { floor_block } else { self.air_block };
                                let Ok(b) = layers[ly] else {
                                    return CorridorResult::Unknown(Refusal::StateOrContext);
                                };
                                match legs_differs(b, expected_b) {
                                    Ok(false) => {}
                                    Ok(true) => record(
                                        [sx * 16 + x0 as i32, wy, sz * 16 + z0 as i32],
                                        nx * nz,
                                    ),
                                    Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
                                }
                            }
                        }
                        Section::Palette(ps) => {
                            for ly in y0..=y1 {
                                let wy = sy * 16 + ly as i32;
                                let expected_b = if wy == min[1] { floor_block } else { self.air_block };
                                if let Some(u) = ps.uniform_y[ly] {
                                    let Ok(b) = ps.palette[u as usize] else {
                                        return CorridorResult::Unknown(Refusal::StateOrContext);
                                    };
                                    match legs_differs(b, expected_b) {
                                        Ok(false) => {}
                                        Ok(true) => record(
                                            [sx * 16 + x0 as i32, wy, sz * 16 + z0 as i32],
                                            nx * nz,
                                        ),
                                        Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
                                    }
                                    continue;
                                }
                                for lx in x0..=x1 {
                                    let wx = sx * 16 + lx as i32;
                                    for lz in z0..=z1 {
                                        let idx = (ly * 16 + lz) * 16 + lx;
                                        let Ok(b) = ps.palette[ps.indices[idx] as usize] else {
                                            return CorridorResult::Unknown(Refusal::StateOrContext);
                                        };
                                        match legs_differs(b, expected_b) {
                                            Ok(false) => {}
                                            Ok(true) => record(
                                                [wx, wy, sz * 16 + lz as i32],
                                                1,
                                            ),
                                            Err(e) => return CorridorResult::Unknown(Refusal::Law(e)),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(first_pos) = first {
            CorridorResult::Exception {
                first_pos,
                exceptions,
            }
        } else {
            CorridorResult::Clear(CorridorProof {
                label: expected,
                min,
                max,
                version: self.provenance_version(),
            })
        }
    }
}
