//! Blueprint parser, block state interning against `world::model`, and 26.2 survival/orientation rules.

use law::{Aabb, Block, BlockPos, Vec3, VoxelShape};
use std::collections::BTreeMap;
use std::sync::OnceLock;
use world::BlockState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl Direction {
    pub const VALUES: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    pub const HORIZONTAL: [Self; 4] = [Self::North, Self::South, Self::West, Self::East];

    pub const fn step(self) -> [i32; 3] {
        match self {
            Self::Down => [0, -1, 0],
            Self::Up => [0, 1, 0],
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
            Self::East => [1, 0, 0],
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub const fn relative(self, p: BlockPos) -> BlockPos {
        let [dx, dy, dz] = self.step();
        [p[0] + dx, p[1] + dy, p[2] + dz]
    }

    /// `Direction.fromYRot` (26.2): `0 -> South`, `90 -> West`, `180 -> North`, `270 -> East`.
    pub fn from_y_rot(yaw: f32) -> Self {
        let idx = (((yaw as f64) / 90.0 + 0.5).floor() as i32) & 3;
        [Self::South, Self::West, Self::North, Self::East][idx as usize]
    }

    /// Canonical cardinal yaw `f32` such that `Direction::from_y_rot(yaw) == self`.
    pub const fn canonical_yaw(self) -> f32 {
        match self {
            Self::South => 0.0,
            Self::West => 90.0,
            Self::North => 180.0,
            Self::East => 270.0,
            Self::Down | Self::Up => 0.0,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "down" => Some(Self::Down),
            "up" => Some(Self::Up),
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            "east" => Some(Self::East),
            _ => None,
        }
    }
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

#[derive(Clone, Debug)]
pub struct StateEntry {
    pub id: StateId,
    pub item: ItemId,
    pub raw: String,
    pub state: BlockState,
    pub block: &'static Block,
    pub survival: SurvivalRule,
    pub required_yaw: Option<f32>,
    pub is_scaffolding: bool,
    pub is_shulker: bool,
}

pub struct Registry {
    defaults: BTreeMap<String, Vec<(String, String)>>,
    pub entries: Vec<StateEntry>,
    by_raw: BTreeMap<String, StateId>,
    items: BTreeMap<String, ItemId>,
    pub item_names: Vec<String>,
    pub air: StateId,
    pub stone_bricks: StateId,
    pub scaffolding: StateId,
    pub shulker_box: StateId,
}

static DEFAULT_TABLE: OnceLock<BTreeMap<String, Vec<(String, String)>>> = OnceLock::new();

fn default_properties() -> &'static BTreeMap<String, Vec<(String, String)>> {
    DEFAULT_TABLE.get_or_init(|| {
        let raw = include_str!("../../world/data/defaults.txt");
        let mut map = BTreeMap::new();
        for line in raw.lines().filter(|l| !l.trim().is_empty()) {
            let (name, tail) = line.split_once('[').unwrap_or((line, ""));
            let mut props = Vec::new();
            for pair in tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()) {
                if let Some((k, v)) = pair.split_once('=') {
                    props.push((k.to_owned(), v.to_owned()));
                }
            }
            props.sort();
            map.insert(name.to_owned(), props);
        }
        map
    })
}

impl Registry {
    pub fn new() -> Self {
        let mut reg = Self {
            defaults: default_properties().clone(),
            entries: Vec::new(),
            by_raw: BTreeMap::new(),
            items: BTreeMap::new(),
            item_names: Vec::new(),
            air: StateId(0),
            stone_bricks: StateId(0),
            scaffolding: StateId(0),
            shulker_box: StateId(0),
        };
        reg.air = reg.intern("minecraft:air").expect("air model");
        reg.stone_bricks = reg.intern("minecraft:stone_bricks").expect("stone_bricks model");
        reg.scaffolding = reg
            .intern("minecraft:scaffolding[bottom=false,distance=0,waterlogged=false]")
            .expect("scaffolding model");
        // Use barrel (modeled full-cube 27-slot container) as the law's modeled container representative for staging shulkers/chests.
        reg.shulker_box = reg
            .intern("minecraft:barrel[facing=up,open=false]")
            .expect("container model");
        reg
    }

    pub fn intern(&mut self, spec: &str) -> Result<StateId, String> {
        let spec = spec.trim();
        let (raw_name, tail) = spec.split_once('[').unwrap_or((spec, ""));
        let name = if raw_name.contains(':') {
            raw_name.to_owned()
        } else {
            format!("minecraft:{raw_name}")
        };
        let mut props = self
            .defaults
            .get(&name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        for pair in tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()) {
            let (k, v) = pair
                .split_once('=')
                .ok_or_else(|| format!("invalid property '{pair}' in '{spec}'"))?;
            props.insert(k.to_owned(), v.to_owned());
        }
        let properties: Vec<(String, String)> = props.into_iter().collect();
        let canonical = if properties.is_empty() {
            name.clone()
        } else {
            let joined = properties
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",");
            format!("{name}[{joined}]")
        };
        if let Some(&id) = self.by_raw.get(&canonical) {
            return Ok(id);
        }
        let state = BlockState {
            name: name.clone(),
            properties: properties.clone(),
        };
        let block = world::model(&state)
            .map_err(|why| format!("unmodeled block state '{canonical}': {why}"))?;
        let item = match self.items.get(&name) {
            Some(&id) => id,
            None => {
                let id = ItemId(self.item_names.len() as u16);
                self.item_names.push(name.clone());
                self.items.insert(name.clone(), id);
                id
            }
        };
        let (survival, required_yaw) = classify_rules(&name, &properties);
        let is_scaffolding = name == "minecraft:scaffolding";
        let is_shulker = name == "minecraft:barrel" || name.ends_with("shulker_box");
        let id = StateId(self.entries.len() as u16);
        self.entries.push(StateEntry {
            id,
            item,
            raw: canonical.clone(),
            state,
            block,
            survival,
            required_yaw,
            is_scaffolding,
            is_shulker,
        });
        self.by_raw.insert(canonical, id);
        Ok(id)
    }

    pub fn entry(&self, id: StateId) -> &StateEntry {
        &self.entries[id.0 as usize]
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

fn prop<'a>(props: &'a [(String, String)], key: &str) -> Option<&'a str> {
    props
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn classify_rules(name: &str, props: &[(String, String)]) -> (SurvivalRule, Option<f32>) {
    let short = name.strip_prefix("minecraft:").unwrap_or(name);
    let mut required_yaw = None;
    if let Some(f) = prop(props, "facing").and_then(Direction::parse) {
        if f != Direction::Up && f != Direction::Down {
            // Stairs place with horizontal_direction == facing; trapdoors on top/bottom place with opposite.
            if short.ends_with("_trapdoor") {
                required_yaw = Some(f.opposite().canonical_yaw());
            } else {
                required_yaw = Some(f.canonical_yaw());
            }
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
        if prop(props, "hanging") == Some("true") {
            SurvivalRule::NeedAbove
        } else {
            SurvivalRule::NeedBelow
        }
    } else if short == "bell" {
        match prop(props, "attachment") {
            Some("ceiling") => SurvivalRule::NeedAbove,
            Some("single_wall") | Some("double_wall") => {
                let f = prop(props, "facing")
                    .and_then(Direction::parse)
                    .unwrap_or(Direction::North);
                SurvivalRule::NeedSide(f)
            }
            _ => SurvivalRule::NeedBelow,
        }
    } else if short == "ladder" {
        let f = prop(props, "facing")
            .and_then(Direction::parse)
            .unwrap_or(Direction::North);
        SurvivalRule::NeedSide(f.opposite())
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
        || (short.ends_with("_door") && prop(props, "half") == Some("upper"))
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

/// Exact 26.2 `AABB.distanceToSqr(Vec3)` and `Player.isWithinBlockInteractionRange(pos, 0.0)` (4.5 blocks).
pub fn within_reach_4_5(eye: Vec3, cell: BlockPos) -> bool {
    let min_x = cell[0] as f64;
    let min_y = cell[1] as f64;
    let min_z = cell[2] as f64;
    let dx = (min_x - eye.x).max(eye.x - (min_x + 1.0)).max(0.0);
    let dy = (min_y - eye.y).max(eye.y - (min_y + 1.0)).max(0.0);
    let dz = (min_z - eye.z).max(eye.z - (min_z + 1.0)).max(0.0);
    dx * dx + dy * dy + dz * dz < 4.5 * 4.5
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
/// excluding the endpoint cell `target` itself, using `law::blocks::traverse_blocks`.
pub fn ray_intermediate_cells(eye: Vec3, target: BlockPos, clicked_face: Direction) -> Vec<BlockPos> {
    let [dx, dy, dz] = clicked_face.opposite().step();
    let aim = Vec3::new(
        target[0] as f64 + 0.5 + 0.45 * (dx as f64),
        target[1] as f64 + 0.5 + 0.45 * (dy as f64),
        target[2] as f64 + 0.5 + 0.45 * (dz as f64),
    );
    let mut cells = Vec::new();
    law::blocks::traverse_blocks(eye, aim, |pos| {
        if pos == target {
            return true;
        }
        cells.push(pos);
        false
    });
    cells
}
