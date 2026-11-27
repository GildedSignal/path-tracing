//! Load the saved world's ordered function resources and their cause graph.
//! This is source data. Reading a command does not certify its effects.
pub mod effect;
pub mod clock;
pub mod player;

use crate::{LoadError, WorldReader, WorldSnapshot, nbt, region};
use law::BlockPos;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct DatapackGraph {
    pub functions: BTreeMap<String, Vec<Command>>,
    pub tags: BTreeMap<String, Vec<String>>,
    pub external_packs: Vec<String>,
    pub data_resources: BTreeSet<String>,
    pub game_tick: i64,
    pub scheduled: Vec<ScheduledFunction>,
}

#[derive(Clone, Debug)]
pub struct ScheduledFunction {
    pub trigger_tick: i64,
    pub id: String,
    pub target: String,
}

/// Saved terrain and its unresolved command sources. A terrain query describes
/// the saved cells; it cannot certify a flight that depends on command execution.
pub struct DatapackSnapshot {
    terrain: WorldSnapshot,
    pub graph: DatapackGraph,
    pub command_blocks: Vec<CommandBlock>,
    pub pending_block_ticks: Vec<SavedBlockTick>,
}

impl DatapackSnapshot {
    pub fn load(world: impl AsRef<Path>, lo: BlockPos, hi: BlockPos) -> Result<Self, LoadError> {
        let mut reader = WorldReader::open(world)?;
        let terrain = WorldSnapshot::read_datapack_terrain(&mut reader, lo, hi)?;
        let graph = DatapackGraph::read(&reader)?;
        let (command_blocks, pending_block_ticks) = read_cell_sources(&reader, lo, hi)?;
        Ok(Self { terrain, graph, command_blocks, pending_block_ticks })
    }

    /// Terrain at the saved tick, for inspection or to propose a sentence.
    pub fn saved_terrain(&self) -> &WorldSnapshot {
        &self.terrain
    }
}

#[derive(Debug)]
pub struct Command {
    pub line: usize,
    /// Tokens inside an SNBT object, selector or quoted string stay together.
    pub tokens: Vec<String>,
}

#[derive(Debug)]
pub struct CommandBlock {
    pub cell: BlockPos,
    pub automatic: bool,
    pub powered: bool,
    pub command: Command,
}

#[derive(Debug)]
pub struct SavedBlockTick {
    pub cell: BlockPos,
    pub block: String,
    pub delay: i32,
    pub priority: i32,
}

impl Command {
    pub fn verb(&self) -> &str {
        self.tokens[0].as_str()
    }

    /// A function reference may sit under execute, return, a scheduler, or a
    /// function condition. Arguments and macros remain in the source tokens.
    pub fn calls(&self) -> impl Iterator<Item = &str> {
        self.tokens.windows(3).filter_map(|part| {
            (matches!(part[0].trim_start_matches('$'), "run" | "if" | "unless" | "schedule")
                && part[1] == "function").then_some(part[2].as_str())
        }).chain(self.tokens.get(1).filter(|_| self.verb().trim_start_matches('$') == "function").map(String::as_str))
    }

    /// Terminal verb and arguments in the command. Execute modifiers, including
    /// stores, are still available to the consumer in the full token sequence.
    pub fn action(&self) -> &[String] {
        let mut tokens = self.tokens.as_slice();
        loop {
            match tokens.first().map(|word| word.trim_start_matches('$')) {
                Some("execute") => {
                    let Some(run) = tokens.iter().position(|word| word == "run") else {
                        return tokens;
                    };
                    tokens = &tokens[run + 1..];
                }
                Some("return") if tokens.get(1).is_some_and(|word| word == "run") => tokens = &tokens[2..],
                _ => return tokens,
            }
        }
    }

    /// Direct cell writes of setblock/fill. The source must be resolved before
    /// relative coordinates can be bound. Neighbour effects are separate.
    pub fn direct_block_writes(&self, source: Option<law::Vec3>) -> Option<(BlockPos, BlockPos)> {
        let action = self.action();
        let coordinates = |words: &[String]| -> Option<BlockPos> {
            let at = |axis| -> Option<i32> {
                let word: &String = words.get(axis)?;
                if word.starts_with('~') {
                    let source = source?;
                    let at = [source.x, source.y, source.z][axis];
                    return Some(law::math::floor(effect::block_coordinate(word, at)?));
                }
                word.parse().ok()
            };
            Some([at(0)?, at(1)?, at(2)?])
        };
        let a = coordinates(action.get(1..4)?)?;
        let b = match action[0].as_str() {
            "setblock" => a,
            "fill" => coordinates(action.get(4..7)?)?,
            _ => return None,
        };
        Some((std::array::from_fn(|i| a[i].min(b[i])), std::array::from_fn(|i| a[i].max(b[i]))))
    }
}

/// Command blocks and pending block ticks whose positions are inside the box.
fn read_cell_sources(world: &WorldReader, lo: BlockPos, hi: BlockPos) -> Result<(Vec<CommandBlock>, Vec<SavedBlockTick>), LoadError> {
        let directory = world.world_dir.join("dimensions/minecraft/overworld/region");
        let mut blocks = Vec::new();
        let mut ticks = Vec::new();
        let cell_of = |tag: nbt::Tag<'_>| -> Result<Option<BlockPos>, LoadError> {
            let cell = [tag.required("x")?.int()?, tag.required("y")?.int()?, tag.required("z")?.int()?];
            Ok((0..3).all(|axis| lo[axis] <= cell[axis] && cell[axis] <= hi[axis]).then_some(cell))
        };
        for rz in (lo[2] >> 9)..=(hi[2] >> 9) {
            for rx in (lo[0] >> 9)..=(hi[0] >> 9) {
                let path = directory.join(format!("r.{rx}.{rz}.mca"));
                let bytes = match region::read_copy(&path, region::MAX_REGION_BYTES) {
                    Err(LoadError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => continue,
                    result => result?,
                };
                for z in (lo[2] >> 4).max(rz * 32)..=(hi[2] >> 4).min(rz * 32 + 31) {
                    for x in (lo[0] >> 4).max(rx * 32)..=(hi[0] >> 4).min(rx * 32 + 31) {
                        let payload = match region::chunk_payload(&bytes, [x, z]) {
                            Err(LoadError::MissingChunk { .. }) => continue,
                            result => result?,
                        };
                        let chunk = nbt::root(&payload)?;
                        if let Some(pending) = chunk.get("block_ticks")? {
                            for entry in pending.list(10)? {
                                if let Some(cell) = cell_of(entry)? {
                                    ticks.push(SavedBlockTick { cell, block: entry.required("i")?.string()?,
                                        delay: entry.required("t")?.int()?, priority: entry.required("p")?.int()? });
                                }
                            }
                        }
                        let Some(entities) = chunk.get("block_entities")? else {
                            continue;
                        };
                        for entity in entities.list(10)? {
                            if entity.required("id")?.string()? != "minecraft:command_block" {
                                continue;
                            }
                            let Some(cell) = cell_of(entity)? else { continue; };
                            let source = entity.required("Command")?.string()?;
                            if source.trim().is_empty() {
                                continue;
                            }
                            let tokens = tokenize(source.trim().trim_start_matches('/').trim_start())
                                .map_err(|error| LoadError::Invalid(format!("command block at {cell:?}: {error}")))?;
                            if tokens.is_empty() {
                                return Err(LoadError::Invalid(format!("empty command block command at {cell:?}")));
                            }
                            blocks.push(CommandBlock {
                                cell,
                                automatic: entity.get("auto")?.map(nbt::Tag::byte).transpose()?.unwrap_or(0) != 0,
                                powered: entity.get("powered")?.map(nbt::Tag::byte).transpose()?.unwrap_or(0) != 0,
                                command: Command { line: 0, tokens },
                            });
                        }
                    }
                }
            }
        }
        Ok((blocks, ticks))
}

impl DatapackGraph {
    pub fn load(world: impl AsRef<Path>) -> Result<Self, LoadError> {
        let reader = WorldReader::open(world)?;
        Self::read(&reader)
    }

    pub(crate) fn read(reader: &WorldReader) -> Result<Self, LoadError> {
        let level = region::metadata(&reader.world_dir.join("level.dat"))?;
        let game_tick = nbt::root(&level)?.required("Data")?.required("Time")?.long()?;
        let scheduled_path = reader.world_dir.join("data/minecraft/scheduled_events.dat");
        let scheduled = match region::metadata(&scheduled_path) {
            Ok(data) => read_schedule(&data)?,
            Err(LoadError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        let mut graph = Self {
            functions: BTreeMap::new(),
            tags: BTreeMap::new(),
            external_packs: Vec::new(),
            data_resources: BTreeSet::new(),
            game_tick,
            scheduled,
        };
        for pack in &reader.datapacks {
            if pack == "vanilla" {
                continue;
            }
            let Some(directory) = pack.strip_prefix("file/") else {
                graph.external_packs.push(pack.clone());
                continue;
            };
            if !matches!(Path::new(directory).components().collect::<Vec<_>>().as_slice(), [std::path::Component::Normal(_)]) {
                return Err(LoadError::Invalid(format!("invalid enabled datapack name {pack:?}")));
            }
            let pack_root = reader.world_dir.join("datapacks").join(directory);
            if !pack_root.is_dir() {
                return Err(LoadError::Unsupported(format!("enabled datapack {pack}: expected an unpacked directory")));
            }
            for namespace in entries(&pack_root.join("data"))? {
                if !namespace.is_dir() {
                    continue;
                }
                let ns = namespace.file_name().and_then(|s| s.to_str())
                    .ok_or_else(|| LoadError::Invalid("non-UTF-8 datapack namespace".into()))?;
                let mut pending = entries(&namespace)?;
                while let Some(path) = pending.pop() {
                    if path.is_dir() {
                        pending.extend(entries(&path)?);
                    } else {
                        graph.resource(ns, &namespace, &path)?;
                    }
                }
            }
        }
        Ok(graph)
    }

    fn resource(&mut self, ns: &str, namespace: &Path, path: &Path) -> Result<(), LoadError> {
        let relative = path.strip_prefix(namespace).expect("resource below namespace");
        let relative = relative.to_str()
            .ok_or_else(|| LoadError::Invalid("non-UTF-8 datapack resource path".into()))?
            .replace('\\', "/");
        if let Some(name) = relative.strip_prefix("function/").and_then(|p| p.strip_suffix(".mcfunction")) {
            let commands = read_text(path)?.lines().enumerate().filter_map(|(index, raw)| {
                let source = raw.trim();
                if source.is_empty() || source.starts_with('#') {
                    return None;
                }
                let line = index + 1;
                Some(tokenize(source).map(|tokens| Command { line, tokens })
                    .map_err(|error| LoadError::Invalid(format!("{}:{line}: {error}", path.display()))))
            }).collect::<Result<_, _>>()?;
            self.functions.insert(format!("{ns}:{name}"), commands);
        } else if let Some(name) = relative.strip_prefix("tags/function/").and_then(|p| p.strip_suffix(".json")) {
            let tag = TagFile::parse(&read_text(path)?)
                .map_err(|error| LoadError::Invalid(format!("{}: {error}", path.display())))?;
            let values = self.tags.entry(format!("{ns}:{name}")).or_default();
            if tag.replace {
                values.clear();
            }
            for value in tag.values {
                let value = function_id(&value);
                if !values.contains(&value) {
                    values.push(value);
                }
            }
        } else {
            self.data_resources.insert(format!("{ns}:{relative}"));
        }
        Ok(())
    }

    /// Static reachability includes scheduling edges and condition functions.
    /// Sources other than load and tick (e.g. a command block) supply extra roots.
    pub fn reachable(&self, roots: impl IntoIterator<Item = String>) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut pending: Vec<_> = roots.into_iter().map(|name| function_id(&name)).collect();
        let mut visited = BTreeSet::new();
        let mut functions = BTreeSet::new();
        let mut missing = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }
            if let Some(tag) = name.strip_prefix('#') {
                if let Some(values) = self.tags.get(tag) {
                    pending.extend(values.iter().cloned());
                }
            } else if let Some(commands) = self.functions.get(&name) {
                functions.insert(name);
                pending.extend(commands.iter().flat_map(Command::calls).map(function_id));
            } else {
                missing.insert(name);
            }
        }
        (functions, missing)
    }
}

fn read_schedule(bytes: &[u8]) -> Result<Vec<ScheduledFunction>, LoadError> {
    let events = nbt::root(bytes)?.required("data")?.required("events")?;
    let mut functions = Vec::new();
    for entry in events.list(10)? {
        let callback = entry.required("callback")?;
        let id = callback.required("id")?.string()?;
        let target = match callback.required("type")?.string()?.as_str() {
            "minecraft:function" => function_id(&id),
            "minecraft:function_tag" => function_id(&format!("#{id}")),
            other => return Err(LoadError::Unsupported(format!("scheduled callback type {other}"))),
        };
        functions.push(ScheduledFunction {
            trigger_tick: entry.required("trigger_time")?.long()?,
            id: entry.required("id")?.string()?,
            target,
        });
    }
    // TimerQueue breaks same-tick ties by insertion order, including on reload.
    functions.sort_by_key(|entry| entry.trigger_tick);
    Ok(functions)
}

fn function_id(name: &str) -> String {
    let (prefix, path) = name.strip_prefix('#').map_or(("", name), |p| ("#", p));
    if path.contains(':') {
        name.to_owned()
    } else {
        format!("{prefix}minecraft:{path}")
    }
}

fn entries(path: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let io = |source| LoadError::Io { path: path.to_owned(), source };
    let mut entries: Vec<_> = fs::read_dir(path).map_err(io)?
        .map(|entry| {
            let entry = entry.map_err(io)?;
            if entry.file_type().map_err(io)?.is_symlink() {
                return Err(LoadError::Unsupported(format!("linked datapack resource {}", entry.path().display())));
            }
            Ok(entry.path())
        })
        .collect::<Result<_, _>>()?;
    entries.sort_unstable();
    Ok(entries)
}

fn read_text(path: &Path) -> Result<String, LoadError> {
    fs::read_to_string(path).map_err(|source| LoadError::Io { path: path.to_owned(), source })
}

pub(crate) fn tokenize(source: &str) -> Result<Vec<String>, &'static str> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut nesting = Vec::new();
    let mut quote = None;
    let mut escape = false;
    for (at, c) in source.char_indices() {
        if let Some(q) = quote {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '[' => nesting.push(']'),
            '{' => nesting.push('}'),
            '(' => nesting.push(')'),
            ']' | '}' | ')' if nesting.pop() != Some(c) => return Err("mismatched command delimiters"),
            c if c.is_ascii_whitespace() && nesting.is_empty() => {
                if at > start {
                    tokens.push(source[start..at].to_owned());
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    if quote.is_some() || !nesting.is_empty() {
        return Err("unclosed command argument");
    }
    if start < source.len() {
        tokens.push(source[start..].to_owned());
    }
    Ok(tokens)
}

/// Function tag files have just a replace flag and a list of resource IDs.
/// Tag entries with fallback/optional semantics require the resource resolver.
struct TagFile {
    replace: bool,
    values: Vec<String>,
}

impl TagFile {
    fn parse(source: &str) -> Result<Self, &'static str> {
        let mut source = source.trim().strip_prefix('{').ok_or("expected function tag object")?.trim();
        let mut replace = None;
        let mut values = None;
        loop {
            let (key, rest) = Self::string(source)?;
            source = rest.trim_start().strip_prefix(':').ok_or("expected field separator")?.trim_start();
            match key {
                "replace" if replace.is_none() => {
                    let (value, rest) = source.strip_prefix("true").map(|s| (true, s))
                        .or_else(|| source.strip_prefix("false").map(|s| (false, s)))
                        .ok_or("replace must be boolean")?;
                    replace = Some(value);
                    source = rest;
                }
                "values" if values.is_none() => {
                    let mut ids = Vec::new();
                    source = source.strip_prefix('[').ok_or("expected function tag list")?.trim_start();
                    if source.starts_with(']') {
                        source = &source[1..];
                    } else {
                        loop {
                            let (id, rest) = Self::string(source)?;
                            ids.push(id.to_owned());
                            let rest = rest.trim_start();
                            if let Some(end) = rest.strip_prefix(']') {
                                source = end;
                                break;
                            }
                            source = rest.strip_prefix(',').ok_or("expected tag list separator")?.trim_start();
                        }
                    }
                    values = Some(ids);
                }
                _ => return Err("unexpected or duplicate function tag field"),
            }
            source = source.trim_start();
            if source == "}" {
                return Ok(Self { replace: replace.unwrap_or(false), values: values.ok_or("missing values")? });
            }
            source = source.strip_prefix(',').ok_or("expected tag field separator")?.trim_start();
        }
    }

    fn string(source: &str) -> Result<(&str, &str), &'static str> {
        let source = source.strip_prefix('"').ok_or("expected plain resource string")?;
        let (value, rest) = source.split_once('"').ok_or("unclosed resource string")?;
        if value.is_empty() || value.bytes().any(|b| !matches!(b, b'a'..=b'z' | b'0'..=b'9' | b':' | b'/' | b'#' | b'.' | b'_' | b'-')) {
            return Err("unsupported resource string");
        }
        Ok((value, rest))
    }
}
