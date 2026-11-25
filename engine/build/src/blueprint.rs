//! Blueprint parser over the law's canonical 26.2 `StateId`, and the build's survival and
//! orientation rules per state.

use law::basis::cell::{self, BlockId};
use law::{Aabb, Block, BlockPos, Vec3, VoxelShape};
use std::collections::BTreeMap;

pub use law::basis::cell::StateId;
pub use law::neighbor::Direction;

/// The name `Direction` carries in block-state properties (`facing=north`).
fn direction(name: &str) -> Option<Direction> {
    cell::DIRECTION.iter().position(|&d| d == name).map(|i| Direction::from_3d_data(i as u8))
}

/// A yaw `y` with `Direction::from_y_rot(y) == d` for a horizontal `d` (`BY_2D` index × 90°).
pub fn canonical_yaw(d: Direction) -> f32 {
    Direction::BY_2D.iter().position(|&h| h == d).map_or(0.0, |i| i as f32 * 90.0)
}

/// Structural survival constraint (`canSurvive` / gravity / fluid basin).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurvivalRule {
    /// Standard block: needs at least one adjacent block present in `W(t)` to click against.
    AnyNeighbor,
    /// Must have `[x, y - 1, z]` present and solid/supporting in `W(t)` before placement and throughout life.
    NeedBelow,
    /// Gravity block (`sand`, `gravel`, `anvil`, `*_concrete_powder`): needs `[x, y - 1, z]` when static,
    /// or can be dropped from `[x, y + h, z]` in 4D to fall onto `[x, y - 1, z]`.
    Gravity,
    /// Must have `[x, y + 1, z]` present in `W(t)` (e.g. `lantern[hanging=true]`, `bell[attachment=ceiling]`).
    NeedAbove,
    /// Must have `support_dir.relative(p)` present in `W(t)` (e.g. wall bell, ladder).
    NeedSide(Direction),
    /// Water cell: requires floor `[x, y - 1, z]` and any non-water horizontal boundary neighbor in the blueprint
    /// to exist before pouring so water never spills outside its basin.
    WaterBasin,
}

/// What the build reads of a state beyond the law: its spec for messages, its rules.
#[derive(Clone, Debug)]
pub struct StateEntry {
    pub raw: String,
    pub block: &'static Block,
    pub survival: SurvivalRule,
    pub required_yaw: Option<f32>,
}

/// The states a blueprint names, keyed by their canonical 26.2 id.
pub struct Registry {
    entries: BTreeMap<StateId, StateEntry>,
    pub air: StateId,
    pub scaffolding: StateId,
}

impl Registry {
    pub fn new() -> Self {
        let mut reg = Self { entries: BTreeMap::new(), air: StateId(0), scaffolding: StateId(0) };
        reg.air = reg.intern("minecraft:air").expect("air model");
        reg.scaffolding = reg
            .intern("minecraft:scaffolding[bottom=false,distance=0,waterlogged=false]")
            .expect("scaffolding model");
        reg
    }

    /// The state a `/setblock` spec names (`name[k=v,...]`); properties left out take the
    /// default state's values, and a state the law refuses is an error.
    pub fn intern(&mut self, spec: &str) -> Result<StateId, String> {
        let spec = spec.trim();
        let (raw_name, tail) = spec.split_once('[').unwrap_or((spec, ""));
        let name = if raw_name.contains(':') { raw_name.to_owned() } else { format!("minecraft:{raw_name}") };
        let mut props = Vec::new();
        for pair in tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()) {
            props.push(pair.split_once('=').ok_or_else(|| format!("invalid property '{pair}' in '{spec}'"))?);
        }
        let id = cell::lookup(&name, |p| props.iter().find(|(k, _)| *k == p).map(|(_, v)| *v))
            .map_err(|why| format!("unmodeled block state '{spec}': {why}"))?;
        self.entries.entry(id).or_insert_with(|| {
            let (survival, required_yaw) = classify_rules(id);
            StateEntry { raw: format!("{name}{}", if tail.is_empty() { String::new() } else { format!("[{tail}") }), block: id.model_block(), survival, required_yaw }
        });
        Ok(id)
    }

    pub fn entry(&self, id: StateId) -> &StateEntry {
        &self.entries[&id]
    }

    /// The item a state is placed from: its block.
    pub fn item(&self, id: StateId) -> BlockId {
        id.block()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_rules(id: StateId) -> (SurvivalRule, Option<f32>) {
    let short = id.block().name();
    let prop = |key: &str| id.get(key);
    let mut required_yaw = None;
    if let Some(f) = prop("facing").and_then(direction) {
        if f.is_horizontal() {
            // Stairs place with horizontal_direction == facing; trapdoors on top/bottom place with opposite.
            required_yaw = Some(canonical_yaw(if short.ends_with("_trapdoor") { f.opposite() } else { f }));
        }
    }

    let survival = if short == "water" {
        SurvivalRule::WaterBasin
    } else if short == "sand"
        || short == "red_sand"
        || short == "gravel"
        || short == "anvil"
        || short.ends_with("_concrete_powder")
    {
        SurvivalRule::Gravity
    } else if short == "lantern" || short == "soul_lantern" {
        if prop("hanging") == Some("true") {
            SurvivalRule::NeedAbove
        } else {
            SurvivalRule::NeedBelow
        }
    } else if short == "bell" {
        match prop("attachment") {
            Some("ceiling") => SurvivalRule::NeedAbove,
            Some("single_wall") | Some("double_wall") => {
                SurvivalRule::NeedSide(prop("facing").and_then(direction).unwrap_or(Direction::North))
            }
            _ => SurvivalRule::NeedBelow,
        }
    } else if short == "ladder" {
        SurvivalRule::NeedSide(prop("facing").and_then(direction).unwrap_or(Direction::North).opposite())
    } else if short.ends_with("_carpet")
        || short.starts_with("potted_")
        || matches!(
            short,
            "allium"
                | "azalea"
                | "flowering_azalea"
                | "fern"
                | "lily_of_the_valley"
                | "poppy"
                | "short_grass"
                | "sea_pickle"
                | "torch"
                | "redstone_wire"
        )
        || (short.ends_with("_door") && prop("half") == Some("upper"))
    {
        SurvivalRule::NeedBelow
    } else {
        SurvivalRule::AnyNeighbor
    };

    (survival, required_yaw)
}

#[derive(Clone, Debug)]
pub struct Blueprint {
    pub name: String,
    pub ground_y: i32,
    pub min: BlockPos,
    pub max: BlockPos,
    pub target: BTreeMap<BlockPos, StateId>,
    pub command_count: usize,
    pub raw_placements: usize,
}

impl Blueprint {
    pub fn empty(name: impl Into<String>, ground_y: i32) -> Self {
        Self {
            name: name.into(),
            ground_y,
            min: [i32::MAX; 3],
            max: [i32::MIN; 3],
            target: BTreeMap::new(),
            command_count: 0,
            raw_placements: 0,
        }
    }

    pub fn set(&mut self, p: BlockPos, state: StateId, air: StateId) {
        for a in 0..3 {
            self.min[a] = self.min[a].min(p[a]);
            self.max[a] = self.max[a].max(p[a]);
        }
        if state == air {
            self.target.remove(&p);
        } else {
            self.target.insert(p, state);
        }
    }

    pub fn parse_commands(
        name: impl Into<String>,
        ground_y: i32,
        script: &str,
        registry: &mut Registry,
    ) -> Result<Self, String> {
        let mut bp = Self::empty(name, ground_y);
        let air = registry.air;
        for (line_no, raw_line) in script.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.first().copied() {
                Some("/fill") => {
                    if parts.len() < 8 {
                        return Err(format!("line {}: malformed /fill", line_no + 1));
                    }
                    let c: Result<Vec<i32>, _> = parts[1..7].iter().map(|s| s.parse()).collect();
                    let c = c.map_err(|e| format!("line {}: {e}", line_no + 1))?;
                    let state = registry.intern(parts[7])?;
                    let (x0, x1) = (c[0].min(c[3]), c[0].max(c[3]));
                    let (y0, y1) = (c[1].min(c[4]), c[1].max(c[4]));
                    let (z0, z1) = (c[2].min(c[5]), c[2].max(c[5]));
                    bp.command_count += 1;
                    for x in x0..=x1 {
                        for y in y0..=y1 {
                            for z in z0..=z1 {
                                bp.raw_placements += 1;
                                bp.set([x, y, z], state, air);
                            }
                        }
                    }
                }
                Some("/setblock") => {
                    if parts.len() < 5 {
                        return Err(format!("line {}: malformed /setblock", line_no + 1));
                    }
                    let c: Result<Vec<i32>, _> = parts[1..4].iter().map(|s| s.parse()).collect();
                    let c = c.map_err(|e| format!("line {}: {e}", line_no + 1))?;
                    let state = registry.intern(parts[4])?;
                    bp.command_count += 1;
                    bp.raw_placements += 1;
                    bp.set([c[0], c[1], c[2]], state, air);
                }
                other => {
                    return Err(format!(
                        "line {}: unsupported command {:?}",
                        line_no + 1,
                        other
                    ))
                }
            }
        }
        Ok(bp)
    }
}

/// `Player.isWithinBlockInteractionRange(pos, 0.0)` at the survival reach (4.5 blocks).
pub fn within_reach_4_5(eye: Vec3, cell: BlockPos) -> bool {
    law::interact::is_within_block_interaction_range(eye, law::interact::DEFAULT_BLOCK_INTERACTION_RANGE, cell, 0.0)
}

/// Checks if a player body centered at `(stand[0] + 0.5, stand[1], stand[2] + 0.5)` with height `height` (1.8 standing, 0.6 crawl)
/// intersects unit cell `target` when `target_shape` is non-empty (`Level.isUnobstructed`).
pub fn body_intersects_block(stand: BlockPos, crouch_crawl: bool, target: BlockPos, target_shape: &VoxelShape) -> bool {
    if target_shape.is_empty() {
        return false;
    }
    let h = if crouch_crawl { 0.6 } else { 1.8 };
    let body = Aabb::new(
        stand[0] as f64 + 0.2,
        stand[1] as f64,
        stand[2] as f64 + 0.2,
        stand[0] as f64 + 0.8,
        stand[1] as f64 + h,
        stand[2] as f64 + 0.8,
    );
    let ox = target[0] as f64;
    let oy = target[1] as f64;
    let oz = target[2] as f64;
    let nx = target_shape.coords[0].len().saturating_sub(1);
    let ny = target_shape.coords[1].len().saturating_sub(1);
    let nz = target_shape.coords[2].len().saturating_sub(1);
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                if !target_shape.is_full([x, y, z]) {
                    continue;
                }
                let b = Aabb::new(
                    target_shape.coords[0][x] + ox,
                    target_shape.coords[1][y] + oy,
                    target_shape.coords[2][z] + oz,
                    target_shape.coords[0][x + 1] + ox,
                    target_shape.coords[1][y + 1] + oy,
                    target_shape.coords[2][z + 1] + oz,
                );
                if body.max.x > b.min.x + 1.0e-7
                    && body.min.x < b.max.x - 1.0e-7
                    && body.max.y > b.min.y + 1.0e-7
                    && body.min.y < b.max.y - 1.0e-7
                    && body.max.z > b.min.z + 1.0e-7
                    && body.min.z < b.max.z - 1.0e-7
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns all intermediate cells crossed by a ray from `eye` to the center of `target` (or `clicked_face`),
/// excluding the endpoint cell `target` itself, using `law::basis::ray::cells`.
pub fn ray_intermediate_cells(eye: Vec3, target: BlockPos, clicked_face: Direction) -> Vec<BlockPos> {
    let [dx, dy, dz] = clicked_face.opposite().step();
    let aim = Vec3::new(
        target[0] as f64 + 0.5 + 0.45 * (dx as f64),
        target[1] as f64 + 0.5 + 0.45 * (dy as f64),
        target[2] as f64 + 0.5 + 0.45 * (dz as f64),
    );
    let mut cells = Vec::new();
    law::basis::ray::cells(eye, aim, |pos| {
        if pos == target {
            return true;
        }
        cells.push(pos);
        false
    });
    cells
}
