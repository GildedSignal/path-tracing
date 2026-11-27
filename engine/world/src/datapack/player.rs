//! Command evaluation for a single player in the snapshot's overworld.
//! A missing selector rule is a refusal at the player's cell and tick.
use super::{Command, DatapackGraph, clock::FunctionClock, effect::{RespawnPoint, Source, block_coordinate}};
use law::{PlayerState, Refusal, Vec3, World, block_pos};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct PlayerFacts {
    pub tags: BTreeSet<String>,
    pub scores: BTreeMap<String, i32>,
    pub respawn: Option<RespawnPoint>,
    pub clock: Option<FunctionClock>,
}

#[derive(Clone, Copy)]
pub struct Context {
    pub source: Source,
    pub as_player: bool,
}

pub struct PlayerCommands<'a, W, B> {
    pub graph: &'a DatapackGraph,
    pub world: &'a W,
    /// The loaded registry's block predicate, including any enabled tag edits.
    pub matches_block: B,
    pub player: &'a mut PlayerState,
    pub facts: &'a mut PlayerFacts,
}

impl<W: World, B: Fn(&W, law::BlockPos, &str) -> Result<bool, Refusal>> PlayerCommands<'_, W, B> {
    pub fn call(&mut self, function: &str, context: Context) -> Result<(), Refusal> {
        self.run(function, context, &mut Vec::new())
    }

    pub fn command(&mut self, command: &Command, context: Context) -> Result<(), Refusal> {
        self.execute(command, context, &mut Vec::new())
            .map_err(|e| self.at(command.line, e))
    }

    fn run(&mut self, function: &str, context: Context, stack: &mut Vec<String>) -> Result<(), Refusal> {
        let id = super::function_id(function);
        let graph = self.graph;
        if stack.contains(&id) {
            return Err(Refusal(format!("recursive datapack function {id}")));
        }
        stack.push(id.clone());
        if let Some(tag) = id.strip_prefix('#') {
            let members = graph.tags.get(tag).ok_or_else(|| Refusal(format!("missing datapack function tag {tag}")))?;
            for member in members {
                self.run(member, context, stack)?;
            }
        } else {
            let commands = graph.functions.get(&id).ok_or_else(|| Refusal(format!("missing datapack function {id}")))?;
            for command in commands {
                self.execute(command, context, stack).map_err(|e| {
                    let e = self.at(command.line, e);
                    Refusal(format!("{id}: {}", e.0))
                })?;
            }
        }
        stack.pop();
        Ok(())
    }

    fn execute(&mut self, command: &Command, mut context: Context, stack: &mut Vec<String>) -> Result<(), Refusal> {
        let words: Vec<_> = command.tokens.iter().map(String::as_str).collect();
        let mut args = words.as_slice();
        let refuse = |what: &str| Refusal(format!("datapack player command: {what}"));
        while let ["execute", tail @ ..] = args {
            args = tail;
            loop {
                args = match args {
                    ["as", selector, tail @ ..] => {
                        if !self.select(selector, context)? { return Ok(()); }
                        context.as_player = true;
                        tail
                    }
                    ["rotated", "as", selector, tail @ ..] => {
                        if !self.select(selector, context)? { return Ok(()); }
                        (context.source.yaw, context.source.pitch) = (self.player.yaw, self.player.pitch);
                        tail
                    }
                    ["at", selector, tail @ ..] => {
                        if !self.select(selector, context)? { return Ok(()); }
                        context.source = Source { position: self.player.position, yaw: self.player.yaw, pitch: self.player.pitch };
                        tail
                    }
                    ["positioned", x, y, z, tail @ ..] => {
                        context.source.position = position([x, y, z], context.source.position, false)
                            .ok_or_else(|| refuse("unsupported positioned coordinates"))?;
                        tail
                    }
                    [condition @ ("if" | "unless"), "block", x, y, z, block, tail @ ..] => {
                        let p = position([x, y, z], context.source.position, true)
                            .ok_or_else(|| refuse("unsupported block coordinates"))?;
                        if (self.matches_block)(self.world, block_pos(p), block)? != (*condition == "if") {
                            return Ok(());
                        }
                        tail
                    }
                    [condition @ ("if" | "unless"), "entity", selector, tail @ ..] => {
                        if self.select(selector, context)? != (*condition == "if") {
                            return Ok(());
                        }
                        tail
                    }
                    ["run", tail @ ..] => { args = tail; break; }
                    _ => return Err(refuse(args.first().copied().unwrap_or("empty execute"))),
                };
            }
        }
        match args {
            ["function", id] => self.run(id, context, stack),
            ["schedule", args @ ..] => self.facts.clock.as_mut()
                .ok_or_else(|| refuse("no server function clock"))?.command(self.graph, args),
            ["tag", selector, operation @ ("add" | "remove"), tag] => {
                if self.select(selector, context)? {
                    if *operation == "add" {
                        self.facts.tags.insert((*tag).into());
                    } else {
                        self.facts.tags.remove(*tag);
                    }
                }
                Ok(())
            }
            ["scoreboard", "players", "set", selector, objective, value] => {
                if self.select(selector, context)? {
                    let score = self.facts.scores.get_mut(*objective)
                        .ok_or_else(|| refuse("scoreboard objective without an observed player score"))?;
                    *score = value.parse().map_err(|_| refuse("invalid scoreboard value"))?;
                }
                Ok(())
            }
            ["effect", "give", selector, ..] | ["spawnpoint", selector, ..] | ["gamemode", _, selector] => {
                if self.select(selector, context)? {
                    command.player_effect(context.source)?.apply(self.player, &mut self.facts.respawn)?;
                }
                Ok(())
            }
            // This interpreter only follows branches with no observable command
            // result. An execute store or a conditional function is refused above.
            ["playsound" | "title" | "tellraw" | "particle", ..] => Ok(()),
            ["fill" | "setblock", ..] => Err(match command.direct_block_writes(Some(context.source.position)) {
                Some((lo, hi)) => Refusal(format!("unmodeled block write to {lo:?}..{hi:?} and its neighbour effects")),
                None => refuse("block write with unresolved coordinates"),
            }),
            _ => Err(refuse(args.first().copied().unwrap_or("empty action"))),
        }
    }

    fn select(&self, selector: &str, context: Context) -> Result<bool, Refusal> {
        let failed = || Refusal(format!("unsupported player selector {selector}"));
        let base = selector.get(..2).ok_or_else(failed)?;
        match base {
            "@s" if !context.as_player => return Ok(false),
            "@s" | "@a" => {}
            _ => return Err(failed()),
        }
        let rest = &selector[2..];
        if rest.is_empty() { return Ok(true); }
        let rest = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')).ok_or_else(failed)?;
        let mut origin = context.source.position;
        let mut delta = [None; 3];
        let mut distance = None;
        // Selector options are a conjunction. An unknown score or option may
        // matter inside the selector even when it is irrelevant outside it.
        let mut unresolved = None;
        for item in selector_fields(rest).ok_or_else(failed)? {
            let (name, value) = item.split_once('=').ok_or_else(failed)?;
            match name {
                "tag" => {
                    let (value, negate) = value.strip_prefix('!').map_or((value, false), |v| (v, true));
                    let present = if value.is_empty() { !self.facts.tags.is_empty() } else { self.facts.tags.contains(value) };
                    if present == negate { return Ok(false); }
                }
                "scores" => {
                    let scores = value.strip_prefix('{').and_then(|s| s.strip_suffix('}')).ok_or_else(failed)?;
                    for rule in scores.split(',') {
                        let (objective, interval) = rule.split_once('=').ok_or_else(failed)?;
                        let Some(score) = self.facts.scores.get(objective) else {
                            unresolved.get_or_insert_with(|| Refusal(format!("unknown player score {objective} in {selector}")));
                            continue;
                        };
                        if !int_range(interval, *score).ok_or_else(failed)? { return Ok(false); }
                    }
                }
                "gamemode" => {
                    let (mode, negate) = value.strip_prefix('!').map_or((value, false), |v| (v, true));
                    let mode = match mode {
                        "survival" => law::player::GameType::Survival,
                        "adventure" => law::player::GameType::Adventure,
                        "creative" => law::player::GameType::Creative,
                        "spectator" => law::player::GameType::Spectator,
                        _ => return Err(failed()),
                    };
                    if (self.player.profile.game_mode == mode) == negate { return Ok(false); }
                }
                "distance" => distance = Some(value),
                "x" => origin.x = finite(value).ok_or_else(failed)?,
                "y" => origin.y = finite(value).ok_or_else(failed)?,
                "z" => origin.z = finite(value).ok_or_else(failed)?,
                "dx" | "dy" | "dz" => {
                    let axis = (name.as_bytes()[1] - b'x') as usize;
                    delta[axis] = Some(finite(value).ok_or_else(failed)?);
                }
                _ => { unresolved.get_or_insert_with(failed); }
            }
        }
        if delta.iter().any(Option::is_some) {
            let d = delta.map(|value| value.unwrap_or(0.0));
            // EntitySelectorParser.createAabb uses one full cell at the end.
            let bounds = law::Aabb::new(
                d[0].min(0.0), d[1].min(0.0), d[2].min(0.0),
                d[0].max(0.0) + 1.0, d[1].max(0.0) + 1.0, d[2].max(0.0) + 1.0,
            ).moved(origin);
            if !bounds.intersects(self.player.bounding_box()) { return Ok(false); }
        }
        if let Some(distance) = distance {
            let (lo, hi) = if let Some((lo, hi)) = distance.split_once("..") {
                (if lo.is_empty() { 0.0 } else { finite(lo).ok_or_else(failed)? },
                 if hi.is_empty() { f64::INFINITY } else { finite(hi).ok_or_else(failed)? })
            } else {
                let exact = finite(distance).ok_or_else(failed)?;
                (exact, exact)
            };
            if lo < 0.0 || lo > hi { return Err(failed()); }
            let squared = self.player.position.minus(origin).length_sqr();
            if squared < lo * lo || squared > hi * hi { return Ok(false); }
        }
        unresolved.map_or(Ok(true), Err)
    }

    fn at(&self, line: usize, cause: Refusal) -> Refusal {
        Refusal(format!("line {line} at cell {:?}, player tick {}: {}", block_pos(self.player.position), self.player.t, cause.0))
    }
}

fn selector_fields(input: &str) -> Option<Vec<&str>> {
    let mut bracket = false;
    let mut fields = Vec::new();
    let mut start = 0;
    for (i, ch) in input.char_indices() {
        match ch {
            '{' if !bracket => bracket = true,
            '}' if bracket => bracket = false,
            '{' | '}' | '[' | ']' | '"' | '\'' => return None,
            ',' if !bracket => {
                fields.push(input[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if bracket { return None; }
    fields.push(input[start..].trim());
    Some(fields)
}

fn int_range(interval: &str, n: i32) -> Option<bool> {
    let Some((lo, hi)) = interval.split_once("..") else { return Some(n == interval.parse().ok()?); };
    let lo = if lo.is_empty() { i32::MIN } else { lo.parse().ok()? };
    let hi = if hi.is_empty() { i32::MAX } else { hi.parse().ok()? };
    (lo <= hi).then_some((lo..=hi).contains(&n))
}

fn finite(word: &str) -> Option<f64> {
    word.parse::<f64>().ok().filter(|x| x.is_finite())
}

fn position(words: [&str; 3], from: Vec3, block: bool) -> Option<Vec3> {
    let origins = [from.x, from.y, from.z];
    let coord = |axis: usize| {
        let word = words[axis];
        if block {
            return block_coordinate(word, origins[axis]);
        }
        if let Some(offset) = word.strip_prefix('~') {
            return Some(origins[axis] + if offset.is_empty() { 0.0 } else { finite(offset)? });
        }
        Some(finite(word)? + if axis != 1 && !word.contains('.') { 0.5 } else { 0.0 })
    };
    Some(Vec3::new(coord(0)?, coord(1)?, coord(2)?))
}
