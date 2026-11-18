//! Living mirror of the server's loaded overworld sections, implementing `law::World`.
use std::collections::HashMap;
use law::{Block, BlockPos, World};

#[derive(Clone)]
struct PaletteSection {
    palette: Vec<Result<&'static Block, String>>,
    indices: Box<[u16; 4096]>,
}

enum Section {
    AllAir,
    Palette(PaletteSection),
}

pub struct WorldMirror {
    pub session: u64,
    difficulty: u8,
    natural_regeneration: bool,
    sections: HashMap<[i32; 3], Section>,
    air_block: &'static Block,
    model_cache: HashMap<String, Result<&'static Block, String>>,
    table_ptr: usize,
    table_len: usize,
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
            sections: HashMap::new(),
            air_block,
            model_cache,
            table_ptr: table.as_ptr() as usize,
            table_len: table.len(),
        }
    }

    pub fn reset(&mut self, session: u64, difficulty: u8, natural_regen: bool) {
        self.session = session;
        self.difficulty = difficulty;
        self.natural_regeneration = natural_regen;
        self.sections.clear();
    }

    pub fn set_rules(&mut self, difficulty: u8, natural_regen: bool) {
        self.difficulty = difficulty;
        self.natural_regeneration = natural_regen;
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
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
        self.sections.insert([cx, cy, cz], Section::AllAir);
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
        self.sections
            .insert([cx, cy, cz], Section::Palette(PaletteSection { palette, indices }));
        Ok(())
    }

    pub fn set_block_delta(&mut self, p: BlockPos, state_name: &str) {
        let sec_key = [p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)];
        let cell_idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16 + p[0].rem_euclid(16)) as usize;
        let model = self.resolve_model(state_name);
        let Some(sec) = self.sections.get_mut(&sec_key) else {
            return;
        };
        match sec {
            Section::AllAir => {
                if let Ok(b) = model
                    && std::ptr::eq(b, self.air_block)
                {
                    return;
                }
                let mut indices = Box::new([0u16; 4096]);
                indices[cell_idx] = 1;
                *sec = Section::Palette(PaletteSection {
                    palette: vec![Ok(self.air_block), model],
                    indices,
                });
            }
            Section::Palette(ps) => {
                let target_id = if let Some(pos) = ps.palette.iter().position(|entry| match (entry, &model) {
                    (Ok(a), Ok(b)) => std::ptr::eq(*a, *b),
                    (Err(a), Err(b)) => a == b,
                    _ => false,
                }) {
                    pos as u16
                } else {
                    let id = ps.palette.len() as u16;
                    ps.palette.push(model);
                    id
                };
                ps.indices[cell_idx] = target_id;
            }
        }
    }

    pub fn unload_chunk(&mut self, cx: i32, cz: i32) {
        self.sections.retain(|k, _| k[0] != cx || k[2] != cz);
    }

    pub fn min_loaded_y(&self) -> i32 {
        self.sections
            .keys()
            .map(|k| k[1] * 16)
            .min()
            .unwrap_or(-64)
    }

    pub fn max_loaded_y(&self) -> i32 {
        self.sections
            .keys()
            .map(|k| k[1] * 16 + 15)
            .max()
            .unwrap_or(319)
    }

    fn cell_result(&self, p: BlockPos) -> Option<&Result<&'static Block, String>> {
        let sec = self.sections.get(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)])?;
        match sec {
            Section::AllAir => None,
            Section::Palette(ps) => {
                let idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16 + p[0].rem_euclid(16)) as usize;
                Some(&ps.palette[ps.indices[idx] as usize])
            }
        }
    }

    pub fn cell_block(&self, p: BlockPos) -> Result<&'static Block, String> {
        let Some(sec) = self.sections.get(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)]) else {
            return Err(format!("unloaded cell {p:?}"));
        };
        match sec {
            Section::AllAir => Ok(self.air_block),
            Section::Palette(ps) => {
                let idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16 + p[0].rem_euclid(16)) as usize;
                ps.palette[ps.indices[idx] as usize].clone()
            }
        }
    }

    pub fn check_corridor_valid(&self, lo: BlockPos, hi: BlockPos) -> Result<(), String> {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                for x in lo[0]..=hi[0] {
                    let p = [x, y, z];
                    let Some(sec) = self.sections.get(&[x.div_euclid(16), y.div_euclid(16), z.div_euclid(16)]) else {
                        return Err(format!("unloaded cell {p:?}"));
                    };
                    if let Section::Palette(ps) = sec {
                        let idx = ((y.rem_euclid(16) * 16 + z.rem_euclid(16)) * 16 + x.rem_euclid(16)) as usize;
                        if let Err(why) = &ps.palette[ps.indices[idx] as usize] {
                            return Err(format!("unmodeled block at {p:?}: {why}"));
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
}

impl World for WorldMirror {
    fn block(&self, p: BlockPos) -> &Block {
        match self.sections.get(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)]) {
            Some(Section::AllAir) => self.air_block,
            Some(Section::Palette(ps)) => {
                let idx = ((p[1].rem_euclid(16) * 16 + p[2].rem_euclid(16)) * 16 + p[0].rem_euclid(16)) as usize;
                match &ps.palette[ps.indices[idx] as usize] {
                    Ok(block) => block,
                    Err(why) => panic!("world mirror unmodeled cell {p:?}: {why}"),
                }
            }
            None => panic!("world mirror unloaded cell {p:?}"),
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
        self.sections.contains_key(&[p[0].div_euclid(16), p[1].div_euclid(16), p[2].div_euclid(16)])
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
        true
    }

    fn has_inside_effects(&self) -> bool {
        true
    }
}
