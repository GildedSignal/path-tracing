use crate::{LoadError, inflate, nbt};
use std::{fmt, fs::OpenOptions, io::Read, mem::size_of, path::{Path, PathBuf}};

pub const DEFAULT_CACHE_BYTES: usize = 64 * 1024 * 1024;
const CACHE_SLOTS: usize = 16;
// The location table has 1024 entries of at most 255 inline sectors each.
const MAX_REGION_BYTES: u64 = (2 + 1024 * 255) * 4096;
const MAX_NBT_BYTES: usize = 16 * 1024 * 1024;
const MIN_SECTION: i8 = -4;
const MAX_SECTION: i8 = 19;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockState {
    pub name: String,
    /// Sorted by property name. No properties or names are discarded.
    pub properties: Vec<(String, String)>,
}
impl BlockState {
    fn heap_bytes(&self) -> usize {
        self.name.capacity() + self.properties.capacity() * size_of::<(String, String)>()
            + self.properties.iter().map(|(k,v)| k.capacity() + v.capacity()).sum::<usize>()
    }
}
impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for (i, (k,v)) in self.properties.iter().enumerate() {
            write!(f, "{}{k}={v}", if i == 0 { "[" } else { "," })?;
        }
        if !self.properties.is_empty() { write!(f, "]")?; }
        Ok(())
    }
}
struct Section { palette: Vec<BlockState>, indices: Option<Box<[u16; 4096]>> }
/// Sections Y = -4..=19 in order: a full 26.2 overworld chunk saves every one.
struct Chunk { sections: Vec<Section> }
impl Chunk {
    /// `y` is within the build height; `WorldReader::block_state` answers outside it.
    fn state(&self, [x,y,z]: [i32;3]) -> &BlockState {
        let section = &self.sections[(y.div_euclid(16) - MIN_SECTION as i32) as usize];
        let index = (y.rem_euclid(16)*256 + z.rem_euclid(16)*16 + x.rem_euclid(16)) as usize;
        &section.palette[section.indices.as_ref().map_or(0, |v| v[index] as usize)]
    }
    fn heap_bytes(&self) -> usize {
        self.sections.capacity()*size_of::<Section>() + self.sections.iter().map(|s| {
            s.palette.capacity()*size_of::<BlockState>() + s.palette.iter().map(BlockState::heap_bytes).sum::<usize>()
                + if s.indices.is_some() { 4096*size_of::<u16>() } else { 0 }
        }).sum::<usize>()
    }
}
struct CacheEntry { coord: [i32;2], chunk: Chunk, bytes: usize }

/// Static saved-world reader. Cache entries are private and never escape by ownership.
/// A query borrows its result until the next mutable query. Reopen to observe edits.
pub struct WorldReader {
    region_dir: PathBuf,
    data_version: i32,
    cache: Vec<CacheEntry>,
    cache_limit: usize,
    heap_bytes: usize,
    void: BlockState,
    pub(crate) world_dir: PathBuf,
    pub(crate) difficulty: u8,
    pub(crate) datapacks: Vec<String>,
}
impl WorldReader {
    pub fn open(world_dir: impl AsRef<Path>) -> Result<Self, LoadError> {
        Self::with_cache_limit(world_dir, DEFAULT_CACHE_BYTES)
    }
    pub fn with_cache_limit(world_dir: impl AsRef<Path>, cache_limit: usize) -> Result<Self, LoadError> {
        let world_dir = world_dir.as_ref();
        let region_dir = world_dir.join("dimensions/minecraft/overworld/region");
        if !region_dir.is_dir() { return Err(LoadError::Unsupported(format!("26.2 layout requires {}", region_dir.display()))); }
        let level = metadata(&world_dir.join("level.dat"))?;
        let data = nbt::root(&level)?.required("Data")?;
        let data_version = data.required("DataVersion")?.int()?;
        let version = data.required("Version")?.required("Name")?.string()?;
        if version != "26.2" { return Err(LoadError::Unsupported(format!("level.dat version {version}; expected 26.2"))); }
        let difficulty = match data.required("difficulty_settings")?.required("difficulty")?.string()?.as_str() {
            "peaceful" => 0, "easy" => 1, "normal" => 2, "hard" => 3,
            name => return Err(LoadError::Unsupported(format!("difficulty {name}"))),
        };
        let datapacks = data.required("DataPacks")?.required("Enabled")?.list(8)?.into_iter().map(|p| p.string()).collect::<Result<Vec<_>,_>>()?;
        let cache = Vec::with_capacity(CACHE_SLOTS);
        let void = BlockState { name: "minecraft:void_air".into(), properties: Vec::new() };
        let heap_bytes = cache.capacity()*size_of::<CacheEntry>() + void.heap_bytes();
        if heap_bytes > cache_limit { return Err(LoadError::CacheTooSmall { needed: heap_bytes, limit: cache_limit }); }
        Ok(Self { region_dir, data_version, cache, cache_limit, heap_bytes, void, world_dir: world_dir.to_owned(), difficulty, datapacks })
    }
    pub fn data_version(&self) -> i32 { self.data_version }
    /// Owned cache allocations including slot storage and the shared void-air state;
    /// excludes allocator bookkeeping, path strings and the temporary input/parser buffers.
    pub fn cache_bytes(&self) -> usize { self.heap_bytes }
    pub fn cache_limit(&self) -> usize { self.cache_limit }
    pub fn cached_chunks(&self) -> usize { self.cache.len() }
    /// Outside the overworld build height (-64..=319) the answer is void air, before
    /// any chunk is read, as in `Level::getBlockState`.
    pub fn block_state(&mut self, position: [i32;3]) -> Result<&BlockState, LoadError> {
        if !(-64..320).contains(&position[1]) { return Ok(&self.void); }
        let coord = [position[0].div_euclid(16), position[2].div_euclid(16)];
        let index = self.ensure_chunk(coord).map_err(|e| e.at(position))?;
        Ok(self.cache[index].chunk.state(position))
    }
    fn ensure_chunk(&mut self, coord: [i32;2]) -> Result<usize, LoadError> {
        if let Some(index) = self.cache.iter().position(|e| e.coord == coord) {
            let hit = self.cache.remove(index); self.cache.push(hit);
            return Ok(self.cache.len()-1);
        }
        let path = self.region_dir.join(format!("r.{}.{}.mca", coord[0].div_euclid(32), coord[1].div_euclid(32)));
        let bytes = match read_copy(&path, MAX_REGION_BYTES) {
            Err(LoadError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => return Err(LoadError::MissingChunk { x: coord[0], z: coord[1] }),
            result => result?,
        };
        let payload = chunk_payload(&bytes, coord)?;
        let chunk = parse_chunk(&payload, coord, self.data_version)?;
        let bytes = chunk.heap_bytes();
        let base = self.cache.capacity()*size_of::<CacheEntry>() + self.void.heap_bytes();
        if base + bytes > self.cache_limit { return Err(LoadError::CacheTooSmall { needed: base + bytes, limit: self.cache_limit }); }
        while self.heap_bytes + bytes > self.cache_limit || self.cache.len() == CACHE_SLOTS {
            self.heap_bytes -= self.cache.remove(0).bytes;
        }
        self.heap_bytes += bytes;
        self.cache.push(CacheEntry { coord, chunk, bytes });
        Ok(self.cache.len()-1)
    }
}

pub(crate) fn read_copy(path: &Path, limit: u64) -> Result<Vec<u8>, LoadError> {
    let io = |source| LoadError::Io { path: path.to_owned(), source };
    let mut options = OpenOptions::new(); options.read(true);
    #[cfg(windows)] {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1 | 2 | 4); // FILE_SHARE_READ | WRITE | DELETE; never acquire a writer lock.
    }
    let file = options.open(path).map_err(io)?;
    if file.metadata().map_err(io)?.len() > limit { return Err(LoadError::Unsupported(format!("{} exceeds {limit} byte input bound", path.display()))); }
    let mut bytes = Vec::new();
    file.take(limit+1).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len() as u64 > limit { return Err(LoadError::Invalid(format!("{} grew beyond input bound while copying", path.display()))); }
    Ok(bytes)
}
pub(crate) fn metadata(path: &Path) -> Result<Vec<u8>, LoadError> {
    let bytes = read_copy(path, MAX_NBT_BYTES as u64)?;
    inflate::gzip(&bytes).map_err(|e| LoadError::Invalid(format!("{}: {e}", path.display())))
}
fn chunk_payload(region: &[u8], coord: [i32;2]) -> Result<Vec<u8>, LoadError> {
    let bad = |s: &str| LoadError::Invalid(format!("region chunk {coord:?}: {s}"));
    if region.len() < 8192 { return Err(bad("truncated Anvil header")); }
    let index = (coord[0].rem_euclid(32) + 32*coord[1].rem_euclid(32)) as usize * 4;
    let location = u32::from_be_bytes(region[index..index+4].try_into().unwrap());
    if location == 0 { return Err(LoadError::MissingChunk { x: coord[0], z: coord[1] }); }
    let sector = (location >> 8) as usize;
    let count = (location & 255) as usize;
    if sector < 2 || count == 0 { return Err(bad("invalid sector allocation")); }
    let start = sector*4096;
    let prefix = region.get(start..start+5).ok_or_else(|| bad("chunk header outside file copy"))?;
    let length = u32::from_be_bytes(prefix[..4].try_into().unwrap()) as usize;
    if prefix[4] & 0x80 != 0 { return Err(LoadError::Unsupported(format!("external .mcc chunk {coord:?} (compression byte {})", prefix[4]))); }
    if length < 1 || length + 4 > count*4096 { return Err(bad("chunk length exceeds sector allocation")); }
    let payload = region.get(start+5..start+4+length).ok_or_else(|| bad("chunk payload outside file copy"))?;
    match prefix[4] {
        2 => inflate::zlib(payload).map_err(|e| bad(&e)),
        3 => {
            if payload.len() > MAX_NBT_BYTES { return Err(bad("uncompressed NBT exceeds bound")); }
            Ok(payload.to_vec())
        }
        code => Err(LoadError::Unsupported(format!("chunk {coord:?} compression {code}{}", if code == 4 { " (LZ4)" } else { "" }))),
    }
}
fn parse_chunk(bytes: &[u8], coord: [i32;2], expected: i32) -> Result<Chunk, LoadError> {
    let root = nbt::root(bytes)?;
    let found = root.required("DataVersion")?.int()?;
    if found != expected { return Err(LoadError::VersionMismatch { chunk: coord, expected, found }); }
    if [root.required("xPos")?.int()?, root.required("zPos")?.int()?] != coord { return Err(LoadError::Invalid(format!("chunk coordinates differ from region slot {coord:?}"))); }
    let status = root.required("Status")?.string()?;
    if status != "minecraft:full" { return Err(LoadError::Unsupported(format!("chunk {coord:?} is {status}, not minecraft:full"))); }
    let mut sections: Vec<Option<Section>> = (MIN_SECTION..=MAX_SECTION).map(|_| None).collect();
    let mut seen = Vec::new();
    for section in root.required("sections")?.list(10)? {
        let y = section.required("Y")?.byte()?;
        if seen.contains(&y) { return Err(LoadError::Invalid(format!("duplicate section Y={y}"))); }
        seen.push(y);
        let states = section.get("block_states")?;
        // Vanilla may save light-only sections one beyond the build height; they hold no blocks.
        if !(MIN_SECTION..=MAX_SECTION).contains(&y) {
            if states.is_some() { return Err(LoadError::Invalid(format!("block_states in section Y={y}, outside {MIN_SECTION}..={MAX_SECTION}"))); }
            continue;
        }
        let states = states.ok_or_else(|| LoadError::Invalid(format!("section Y={y} has no block_states")))?;
        let mut palette = Vec::new();
        for entry in states.required("palette")?.list(10)? {
            let name = entry.required("Name")?.string()?;
            let mut properties = match entry.get("Properties")? {
                Some(p) => p.fields()?.into_iter().map(|(k,v)| Ok((k,v.string()?))).collect::<Result<Vec<_>,LoadError>>()?,
                None => Vec::new(),
            };
            properties.sort();
            let state = BlockState { name, properties };
            if palette.contains(&state) { return Err(LoadError::Invalid("duplicate palette entry".into())); }
            palette.push(state);
        }
        if palette.is_empty() || palette.len() > 4096 { return Err(LoadError::Invalid("block palette length outside 1..=4096".into())); }
        let indices = if palette.len() == 1 {
            if states.get("data")?.is_some() { return Err(LoadError::Invalid("single-entry palette has unexpected packed data".into())); }
            None
        } else {
            let bits = (usize::BITS - (palette.len()-1).leading_zeros()).max(4) as usize;
            let per_long = 64/bits;
            let longs = states.required("data")?.longs()?;
            if longs.len()/8 != 4096usize.div_ceil(per_long) { return Err(LoadError::Invalid(format!("packed data length for {}-entry palette, {bits} bits", palette.len()))); }
            let mut indices = Box::new([0u16;4096]);
            for (i, index) in indices.iter_mut().enumerate() {
                let at = (i/per_long)*8;
                let word = u64::from_be_bytes(longs[at..at+8].try_into().unwrap());
                let value = ((word >> ((i%per_long)*bits)) & ((1<<bits)-1)) as usize;
                if value >= palette.len() { return Err(LoadError::Invalid(format!("palette index {value} outside {} entries at section {y}, cell {i}", palette.len()))); }
                *index = value as u16;
            }
            Some(indices)
        };
        sections[(y - MIN_SECTION) as usize] = Some(Section { palette, indices });
    }
    let sections = sections.into_iter().zip(MIN_SECTION..).map(|(s, y)| s.ok_or_else(|| LoadError::Invalid(format!("missing section Y={y}"))))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Chunk { sections })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn string(out:&mut Vec<u8>,s:&str) { out.extend((s.len() as u16).to_be_bytes());out.extend(s.as_bytes()); }
    fn named(out:&mut Vec<u8>,kind:u8,name:&str) {out.push(kind);string(out,name);}
    fn int(out:&mut Vec<u8>,name:&str,value:i32) { named(out,3,name);out.extend(value.to_be_bytes()); }
    fn text(out:&mut Vec<u8>,name:&str,value:&str) { named(out,8,name);string(out,value); }
    // Synthetic inputs target packing and refusal boundaries only. The separate
    // course test supplies independent ground truth for the complete decoder.
    fn chunk(coord:[i32;2],version:i32,palette:usize) -> Vec<u8> {
        build(coord,version,palette,&(-3..=19).map(|y|(y,true)).collect::<Vec<_>>())
    }
    /// `others` are (Y, has block_states) sections written before Y = -4, which comes last
    /// with `palette` test states; others with block states hold only air.
    fn build(coord:[i32;2],version:i32,palette:usize,others:&[(i8,bool)]) -> Vec<u8> {
        let mut out=vec![10,0,0];
        int(&mut out,"DataVersion",version);int(&mut out,"xPos",coord[0]);int(&mut out,"zPos",coord[1]);text(&mut out,"Status","minecraft:full");
        named(&mut out,9,"sections");out.push(10);out.extend((others.len() as i32+1).to_be_bytes());
        for &(y,states) in others {
            named(&mut out,1,"Y");out.push(y as u8);
            if states {
                named(&mut out,10,"block_states");
                named(&mut out,9,"palette");out.push(10);out.extend(1i32.to_be_bytes());
                text(&mut out,"Name","minecraft:air");out.extend([0,0]);
            }
            out.push(0);
        }
        named(&mut out,1,"Y");out.push((-4i8) as u8);
        named(&mut out,10,"block_states");
        named(&mut out,9,"palette");out.push(10);out.extend((palette as i32).to_be_bytes());
        for i in 0..palette {text(&mut out,"Name",&format!("minecraft:test_{i}"));out.push(0);}
        if palette>1 {
            let bits=(usize::BITS-(palette-1).leading_zeros()).max(4) as usize;
            let count=4096usize.div_ceil(64/bits);
            named(&mut out,12,"data");out.extend((count as i32).to_be_bytes());
            let mut words=vec![0u64;count];
            // Last entry in one padded long, first in the next, with the four
            // unused high bits separating them for the 17-entry palette.
            words[0]=((palette-1) as u64) << ((64/bits-1)*bits);
            words[1]=1;
            for word in words {out.extend(word.to_be_bytes());}
        }
        out.extend([0,0,0]);out
    }
    fn region(coord:[i32;2],payload:&[u8],compression:u8) -> Vec<u8> {
        let mut out=vec![0u8;8192];
        let index=(coord[0].rem_euclid(32)+32*coord[1].rem_euclid(32)) as usize*4;
        let count=(payload.len()+5).div_ceil(4096);
        out[index..index+4].copy_from_slice(&(0x200u32+count as u32).to_be_bytes());
        out.extend(((payload.len()+1) as u32).to_be_bytes());out.push(compression);out.extend(payload);out
    }
    #[test]
    fn padded_5_and_9_bit_palettes_negative_coordinates_and_single_palette() {
        let coord=[-1,-33];
        for palette in [17,257] {
            let bytes=chunk(coord,4903,palette);
            let copied=region(coord,&bytes,3);
            let decoded=parse_chunk(&chunk_payload(&copied,coord).unwrap(),coord,4903).unwrap();
            let per_long=64/(usize::BITS-(palette-1).leading_zeros()).max(4) as i32;
            assert_eq!(decoded.state([-16+per_long-1,-64,-528]).name,format!("minecraft:test_{}",palette-1));
            assert_eq!(decoded.state([-16+per_long,-64,-528]).name,"minecraft:test_1");
            assert_eq!(decoded.state([-16,-64,-528]).name,"minecraft:test_0");
            assert_eq!(decoded.state([-16,-48,-528]).name,"minecraft:air");
        }
        let decoded=parse_chunk(&chunk(coord,4903,1),coord,4903).unwrap();
        assert_eq!(decoded.state([-1,-49,-513]).name,"minecraft:test_0");
    }
    #[test]
    fn version_coordinates_lengths_and_unsupported_formats_refuse() {
        let coord=[0,0];let bytes=chunk(coord,4903,1);
        assert!(matches!(parse_chunk(&bytes,coord,4904),Err(LoadError::VersionMismatch {expected:4904,found:4903,..})));
        assert!(parse_chunk(&bytes,[1,0],4903).is_err());
        let mut encoded=region(coord,&bytes,3);
        assert!(chunk_payload(&encoded,[1,0]).is_err());
        assert!(chunk_payload(&encoded[..8191],coord).is_err());
        encoded[8196]=4;
        let error=chunk_payload(&encoded,coord).unwrap_err().to_string();assert!(error.contains("LZ4"));
        encoded[8196]=0x82;
        assert!(chunk_payload(&encoded,coord).unwrap_err().to_string().contains("external .mcc"));
        encoded[8196]=3;encoded[8192..8196].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(chunk_payload(&encoded,coord).is_err());
        for len in [0,5,bytes.len()-1] {assert!(parse_chunk(&bytes[..len],coord,4903).is_err());}
    }
    #[test]
    fn invalid_palette_index_fails() {
        let coord=[0,0];let mut bytes=chunk(coord,4903,17);
        // The first packed word starts just before 342 words and the compound ends.
        let word_start=bytes.len()-3-342*8;
        bytes[word_start+7]=31;
        assert!(parse_chunk(&bytes,coord,4903).err().unwrap().to_string().contains("palette index"));
    }
    #[test]
    fn a_missing_section_or_block_states_fails_but_light_only_sections_pass() {
        let coord=[0,0];
        let full=(-3..=19).map(|y|(y,true)).collect::<Vec<_>>();
        let error=|others:&[(i8,bool)]| parse_chunk(&build(coord,4903,1,others),coord,4903).err().map(|e|e.to_string());
        assert!(error(&full[1..]).unwrap().contains("missing section Y=-3"));
        let mut bare=full.clone();bare[0].1=false;
        assert!(error(&bare).unwrap().contains("section Y=-3 has no block_states"));
        let mut light=full.clone();light.extend([(-5,false),(20,false)]);
        assert_eq!(error(&light),None);
    }
}

