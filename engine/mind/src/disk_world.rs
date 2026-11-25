//! Cached parallel Anvil `.mca` chunk loader for offline/unloaded Virtual corridors.
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use law::basis::cell::StateId;
use crate::world_mirror::Cell;

const MAX_REGION_BYTES: u64 = (2 + 1024 * 255) * 4096;
const MAX_NBT_BYTES: usize = 16 * 1024 * 1024;
const MIN_SECTION: i8 = -4;
const MAX_SECTION: i8 = 19;

pub enum RawDiskSection {
    AllAir,
    Layered([Cell; 16]),
    Palette {
        palette: Vec<Cell>,
        indices: Box<[u16; 4096]>,
    },
}

pub struct DiskWorld {
    region_dir: PathBuf,
    regions: HashMap<[i32; 2], Option<Arc<[u8]>>>,
}

impl DiskWorld {
    pub fn open(world_dir: impl AsRef<Path>) -> Self {
        let region_dir = world_dir
            .as_ref()
            .join("dimensions")
            .join("minecraft")
            .join("overworld")
            .join("region");
        Self {
            region_dir,
            regions: HashMap::new(),
        }
    }

    fn get_region(&mut self, region_coord: [i32; 2]) -> Option<Arc<[u8]>> {
        if let Some(hit) = self.regions.get(&region_coord) {
            return hit.clone();
        }
        let path = self
            .region_dir
            .join(format!("r.{}.{}.mca", region_coord[0], region_coord[1]));
        let loaded = read_region_shared(&path).map(Arc::<[u8]>::from);
        self.regions.insert(region_coord, loaded.clone());
        loaded
    }

    /// Reads any uncached `.mca` files once on the caller thread, then inflates and unpacks
    /// requested chunks in parallel across worker threads.
    pub fn load_chunks_parallel(
        &mut self,
        coords: &[[i32; 2]],
        air: StateId,
    ) -> Vec<([i32; 2], Vec<(i32, RawDiskSection)>)> {
        if coords.is_empty() || !self.region_dir.is_dir() {
            return Vec::new();
        }
        let mut unique_regions = HashSet::new();
        for &c in coords {
            unique_regions.insert([c[0].div_euclid(32), c[1].div_euclid(32)]);
        }
        for r in unique_regions {
            let _ = self.get_region(r);
        }

        let tasks: Vec<([i32; 2], Arc<[u8]>)> = coords
            .iter()
            .filter_map(|&c| {
                let rc = [c[0].div_euclid(32), c[1].div_euclid(32)];
                let bytes = self.regions.get(&rc).and_then(|o| o.as_ref()).cloned()?;
                Some((c, bytes))
            })
            .collect();
        if tasks.is_empty() {
            return Vec::new();
        }

        let threads = std::thread::available_parallelism()
            .map_or(4, |n| n.get())
            .min(12)
            .min(tasks.len());
        if threads <= 1 || tasks.len() < 4 {
            return tasks
                .into_iter()
                .filter_map(|(coord, region)| {
                    decode_chunk(&region, coord, air)
                        .ok()
                        .flatten()
                        .map(|secs| (coord, secs))
                })
                .collect();
        }

        let chunk_size = tasks.len().div_ceil(threads);
        let mut out = Vec::with_capacity(tasks.len());
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(threads);
            for slice in tasks.chunks(chunk_size) {
                handles.push(scope.spawn(move || {
                    let mut local = Vec::with_capacity(slice.len());
                    for (coord, region) in slice {
                        if let Ok(Some(secs)) = decode_chunk(region, *coord, air) {
                            local.push((*coord, secs));
                        }
                    }
                    local
                }));
            }
            for h in handles {
                if let Ok(mut part) = h.join() {
                    out.append(&mut part);
                }
            }
        });
        out
    }
}

fn read_region_shared(path: &Path) -> Option<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1 | 2 | 4); // FILE_SHARE_READ | WRITE | DELETE
    }
    let file = options.open(path).ok()?;
    if file.metadata().ok()?.len() > MAX_REGION_BYTES {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_REGION_BYTES + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_REGION_BYTES {
        return None;
    }
    Some(bytes)
}

fn chunk_payload(region: &[u8], coord: [i32; 2]) -> Result<Option<Vec<u8>>, String> {
    if region.len() < 8192 {
        return Ok(None);
    }
    let index = (coord[0].rem_euclid(32) + 32 * coord[1].rem_euclid(32)) as usize * 4;
    let location = u32::from_be_bytes(region[index..index + 4].try_into().unwrap());
    if location == 0 {
        return Ok(None);
    }
    let sector = (location >> 8) as usize;
    let count = (location & 255) as usize;
    if sector < 2 || count == 0 {
        return Ok(None);
    }
    let start = sector * 4096;
    let Some(prefix) = region.get(start..start + 5) else {
        return Ok(None);
    };
    let length = u32::from_be_bytes(prefix[..4].try_into().unwrap()) as usize;
    if (prefix[4] & 0x80) != 0 || length < 1 || length + 4 > count * 4096 {
        return Ok(None);
    }
    let Some(payload) = region.get(start + 5..start + 4 + length) else {
        return Ok(None);
    };
    match prefix[4] {
        2 => world::inflate::zlib(payload).map(Some),
        3 => {
            if payload.len() > MAX_NBT_BYTES {
                return Err("uncompressed NBT exceeds bound".into());
            }
            Ok(Some(payload.to_vec()))
        }
        _ => Ok(None),
    }
}

fn decode_chunk(
    region: &[u8],
    coord: [i32; 2],
    air: StateId,
) -> Result<Option<Vec<(i32, RawDiskSection)>>, String> {
    let Some(bytes) = chunk_payload(region, coord)? else {
        return Ok(None);
    };
    let root = nbt_root(&bytes)?;
    let status = root.required("Status")?.string()?;
    if status != "minecraft:full" {
        return Ok(None);
    }
    let x_pos = root.required("xPos")?.int()?;
    let z_pos = root.required("zPos")?.int()?;
    if [x_pos, z_pos] != coord {
        return Ok(None);
    }

    let mut out = Vec::with_capacity(24);
    for section in root.required("sections")?.list(10)? {
        let y = section.required("Y")?.byte()?;
        if !(MIN_SECTION..=MAX_SECTION).contains(&y) {
            continue;
        }
        let Some(states) = section.get("block_states")? else {
            continue;
        };
        let mut palette = Vec::new();
        for entry in states.required("palette")?.list(10)? {
            let name = entry.required("Name")?.string()?;
            let mut properties = match entry.get("Properties")? {
                Some(p) => p
                    .fields()?
                    .into_iter()
                    .map(|(k, v)| Ok((k, v.string()?)))
                    .collect::<Result<Vec<_>, String>>()?,
                None => Vec::new(),
            };
            properties.sort();
            palette.push(world::state_id(&world::BlockState { name, properties }));
        }
        if palette.is_empty() || palette.len() > 4096 {
            continue;
        }
        if palette.len() == 1 {
            let only = &palette[0];
            if let Ok(b) = only
                && *b == air
            {
                out.push((y as i32, RawDiskSection::AllAir));
            } else {
                let layers: [Cell; 16] =
                    std::array::from_fn(|_| only.clone());
                out.push((y as i32, RawDiskSection::Layered(layers)));
            }
            continue;
        }

        let bits = (usize::BITS - (palette.len() - 1).leading_zeros()).max(4) as usize;
        let per_long = 64 / bits;
        let longs = states.required("data")?.longs()?;
        if longs.len() / 8 != 4096usize.div_ceil(per_long) {
            continue;
        }
        let mut indices = Box::new([0u16; 4096]);
        let mask = (1usize << bits) - 1;
        for (i, index) in indices.iter_mut().enumerate() {
            let at = (i / per_long) * 8;
            let word = u64::from_be_bytes(longs[at..at + 8].try_into().unwrap());
            let value = ((word >> ((i % per_long) * bits)) as usize) & mask;
            if value >= palette.len() {
                return Err("palette index out of range in disk chunk".into());
            }
            *index = value as u16;
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
        if all_layers_uniform {
            if layer_ids.iter().all(|&id| {
                palette[id]
                    .as_ref()
                    .is_ok_and(|&b| b == air)
            }) {
                out.push((y as i32, RawDiskSection::AllAir));
            } else {
                let layers: [Cell; 16] =
                    std::array::from_fn(|ly| palette[layer_ids[ly]].clone());
                out.push((y as i32, RawDiskSection::Layered(layers)));
            }
        } else {
            out.push((y as i32, RawDiskSection::Palette { palette, indices }));
        }
    }
    Ok(Some(out))
}

#[derive(Clone, Copy)]
struct Tag<'a> {
    kind: u8,
    bytes: &'a [u8],
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).ok_or("NBT length overflow")?;
        let data = self.bytes.get(self.at..end).ok_or("truncated NBT tag")?;
        self.at = end;
        Ok(data)
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn string_bytes(&mut self) -> Result<&'a [u8], String> {
        let n = u16::from_be_bytes(self.take(2)?.try_into().unwrap()) as usize;
        self.take(n)
    }

    fn count(&mut self) -> Result<usize, String> {
        let n = i32::from_be_bytes(self.take(4)?.try_into().unwrap());
        usize::try_from(n).map_err(|_| "negative NBT length".into())
    }

    fn tag(&mut self, kind: u8, depth: usize) -> Result<Tag<'a>, String> {
        if depth > 64 {
            return Err("NBT nesting exceeds 64".into());
        }
        let start = self.at;
        match kind {
            1 => {
                self.take(1)?;
            }
            2 => {
                self.take(2)?;
            }
            3 | 5 => {
                self.take(4)?;
            }
            4 | 6 => {
                self.take(8)?;
            }
            7 | 11 | 12 => {
                let n = self.count()?;
                let width = match kind {
                    7 => 1,
                    11 => 4,
                    _ => 8,
                };
                self.take(n.checked_mul(width).ok_or("NBT array overflow")?)?;
            }
            8 => {
                self.string_bytes()?;
            }
            9 => {
                let child = self.byte()?;
                let n = self.count()?;
                if child > 12 || (child == 0 && n != 0) {
                    return Err("invalid NBT list type".into());
                }
                if n > self.bytes.len() - self.at {
                    return Err("truncated NBT list".into());
                }
                for _ in 0..n {
                    self.tag(child, depth + 1)?;
                }
            }
            10 => loop {
                let child = self.byte()?;
                if child == 0 {
                    break;
                }
                self.string_bytes()?;
                self.tag(child, depth + 1)?;
            },
            _ => return Err("unknown NBT tag type".into()),
        }
        Ok(Tag {
            kind,
            bytes: &self.bytes[start..self.at],
        })
    }
}

fn nbt_root(bytes: &[u8]) -> Result<Tag<'_>, String> {
    let mut c = Cursor { bytes, at: 0 };
    if c.byte()? != 10 {
        return Err("NBT root is not compound".into());
    }
    c.string_bytes()?;
    let tag = c.tag(10, 0)?;
    if c.at != bytes.len() {
        return Err("trailing NBT bytes".into());
    }
    Ok(tag)
}

impl<'a> Tag<'a> {
    fn fields(self) -> Result<Vec<(String, Tag<'a>)>, String> {
        if self.kind != 10 {
            return Err("expected NBT compound".into());
        }
        let mut c = Cursor {
            bytes: self.bytes,
            at: 0,
        };
        let mut out = Vec::new();
        loop {
            let kind = c.byte()?;
            if kind == 0 {
                break;
            }
            let name = mutf8(c.string_bytes()?)?;
            out.push((name, c.tag(kind, 0)?));
        }
        Ok(out)
    }

    fn get(self, name: &str) -> Result<Option<Tag<'a>>, String> {
        if self.kind != 10 {
            return Err("expected NBT compound".into());
        }
        let mut c = Cursor {
            bytes: self.bytes,
            at: 0,
        };
        let mut found = None;
        loop {
            let kind = c.byte()?;
            if kind == 0 {
                return Ok(found);
            }
            let key = c.string_bytes()?;
            let tag = c.tag(kind, 0)?;
            if key == name.as_bytes() {
                found = Some(tag);
            }
        }
    }

    fn required(self, name: &str) -> Result<Tag<'a>, String> {
        self.get(name)?
            .ok_or_else(|| format!("missing NBT field {name}"))
    }

    fn int(self) -> Result<i32, String> {
        if self.kind != 3 {
            return Err("expected NBT int".into());
        }
        Ok(i32::from_be_bytes(self.bytes.try_into().unwrap()))
    }

    fn byte(self) -> Result<i8, String> {
        if self.kind != 1 {
            return Err("expected NBT byte".into());
        }
        Ok(self.bytes[0] as i8)
    }

    fn string(self) -> Result<String, String> {
        if self.kind != 8 {
            return Err("expected NBT string".into());
        }
        mutf8(&self.bytes[2..])
    }

    fn list(self, expected_kind: u8) -> Result<Vec<Tag<'a>>, String> {
        if self.kind != 9 {
            return Err("expected NBT list".into());
        }
        let mut c = Cursor {
            bytes: self.bytes,
            at: 0,
        };
        let kind = c.byte()?;
        let n = c.count()?;
        if kind != expected_kind && !(kind == 0 && n == 0) {
            return Err("unexpected NBT list element type".into());
        }
        (0..n).map(|_| c.tag(kind, 0)).collect()
    }

    fn longs(self) -> Result<&'a [u8], String> {
        if self.kind != 12 {
            return Err("expected NBT long array".into());
        }
        Ok(&self.bytes[4..])
    }
}

fn mutf8(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_ascii() {
        return Ok(std::str::from_utf8(bytes).unwrap().to_owned());
    }
    let mut units = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let a = bytes[at];
        at += 1;
        let (mut value, rest) = match a {
            1..=127 => (a as u16, 0),
            0xc0..=0xdf => ((a & 31) as u16, 1),
            0xe0..=0xef => ((a & 15) as u16, 2),
            _ => return Err("invalid modified UTF-8".into()),
        };
        for _ in 0..rest {
            let &b = bytes.get(at).ok_or("truncated UTF-8")?;
            if b & 0xc0 != 0x80 {
                return Err("invalid UTF-8 continuation".into());
            }
            value = (value << 6) | (b & 63) as u16;
            at += 1;
        }
        units.push(value);
    }
    String::from_utf16(&units).map_err(|_| "unpaired UTF-16 surrogate".into())
}
