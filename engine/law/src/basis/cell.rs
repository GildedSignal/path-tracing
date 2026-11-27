//! `cell`: a cell holds a canonical block state, and its physics is the world crate's
//! generated tables' row for it. `states.tsv` names every block in registry order (its row is the
//! block's id) with its properties' values, so a state's id is Mojang's own; `blocks.tsv` holds the
//! model of each state its relevant properties select (`maps.tsv`); `models.tsv` holds one row per
//! distinct model: boxes, friction, speed and jump factors, the inside and step hooks; `census.tsv`
//! names the blocks refused. `rules.tsv` holds the generated state-local answers used by edits;
//! `cell_rule_index.tsv` maps the canonical states to those rows.
use crate::block_hooks::xz_offset_from_keys;
use crate::piston::{BlockKind as PistonKind, PushReaction};
use crate::{Aabb, Block, ContactDamage, Fluid, FluidFlowBlock, FluidKind, GroundedContact, Inside, Vec3, VoxelShape};
use std::{collections::BTreeMap, sync::OnceLock};

/// A block's row in `states.tsv`: its registry id in the selected jar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u16);
/// A cell's identity: `Block.BLOCK_STATE_REGISTRY` in the selected jar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateId(pub u16);
/// A state's physics: its row in `models.tsv` (states the law treats alike share one).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModelId(pub u16);

/// How far a model's collision reaches, in any context.
pub const NOWHERE: u8 = 0;
pub const OWN_CELL: u8 = 1;
pub const BEYOND: u8 = 2;
const NO_MODEL: u16 = u16::MAX;
pub const MINECRAFT_VERSION: &str = env!("PATHTRACING_LAW_VERSION");

macro_rules! jar_table {
    ($name:literal) => { include_str!(concat!(env!("PATHTRACING_DATA_DIR"), "/", $name)) };
}

pub const DATA_VERSION: i32 = match i32::from_str_radix(jar_table!("version.txt").trim_ascii(), 10) {
    Ok(version) => version,
    Err(_) => panic!("invalid generated DataVersion"),
};
pub const STATE_CENSUS: &str = jar_table!("census.tsv");
pub const BLOCK_MODELS: &str = jar_table!("blocks.tsv");
pub const DEFAULT_STATES: &str = jar_table!("defaults.txt");
pub const ITEM_USE_EFFECTS: &str = jar_table!("use-effects.tsv");

/// A block's states: the first one's id, its default's index, each property's values in order
/// (the first property is the most significant digit of a state's index).
struct States { first: u16, default: u16, properties: Vec<(&'static str, Vec<&'static str>)> }
pub struct Models {
    names: Vec<&'static str>,
    rows: BTreeMap<&'static str, BlockId>,
    states: Vec<States>,
    /// `STATE_TO_BLOCK` and `STATE_TO_MODEL`, by state id; a refused state has no model.
    state_block: Box<[BlockId]>,
    state_model: Box<[u16]>,
    pub blocks: Vec<Block>,
    pub offset_variants: Vec<Option<Box<[Block]>>>,
    pub reach: Vec<u8>,
    refusals: Vec<Option<&'static str>>,
    rules: Vec<Rules>,
    state_rules: Box<[u16]>,
}

/// State-local facts from the jar. `None` on a context-sensitive getter means it needs a world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub destroy_speed: f32,
    pub requires_correct_tool: bool,
    pub redstone_conductor: Option<bool>,
    pub sturdy_faces: Option<u8>,
    pub push_reaction: PushReaction,
    pub blocks_motion: bool,
    pub replaceable_by_water: bool,
    pub replaceable_by_lava: bool,
    pub can_be_replaced: bool,
    pub is_signal_source: bool,
    pub has_block_entity: bool,
    pub is_air: bool,
    pub water_container: WaterContainer,
    pub fluid_boundary: Option<FluidBoundary>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FluidBoundary { pub washed_away: bool, pub solid: bool }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterContainer { None, Accepts, Rejects, Other }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurviveRule {
    #[default]
    Always,
    /// Requires a sturdy UP face below.
    FloorSturdy,
    /// Requires a sturdy face on the attachment side.
    Attached,
}

pub fn models() -> &'static Models {
    static MODELS: OnceLock<Models> = OnceLock::new();
    MODELS.get_or_init(|| {
        let (mut blocks, mut offset_variants, mut reach) = (Vec::new(), Vec::new(), Vec::new());
        for line in jar_table!("models.tsv").lines() {
            let (block, offset) = parse_model(line);
            let (variants, r) = match offset {
                None => (None, if block.never_collides() { NOWHERE } else if block.collision_within_cell() { OWN_CELL } else { BEYOND }),
                Some(cap) => {
                    let list = (0..256).map(|k| {
                        // The jar row is measured at the origin, with seed keys (0, 0).
                        let offset = xz_offset_from_keys((k >> 4) as u8, (k & 15) as u8, cap);
                        let shifted = block.shape.moved(Vec3::new(offset.x + f64::from(cap), 0.0, offset.z + f64::from(cap)));
                        let mut v = block.clone();
                        v.shape = shifted.clone();
                        v.above_shape = Some(shifted.clone());
                        v.below_shape = Some(shifted.clone());
                        v.sneak_shape = Some(shifted.clone());
                        v.inside_cell_shape = Some(shifted);
                        assert!(!v.never_collides() && v.collision_within_cell(), "offset collision must remain within cell");
                        v
                    }).collect();
                    (Some(list), OWN_CELL)
                }
            };
            blocks.push(block);
            offset_variants.push(variants);
            reach.push(r);
        }
        // maps.tsv: relevant properties, then "values:slot" entries in that property order.
        let maps = jar_table!("maps.tsv").lines().map(|line| {
            let (properties, entries) = line.split_once('\t').expect("generated map row");
            let slots: BTreeMap<_, usize> = entries.split(' ').map(|e| {
                let (key, slot) = e.rsplit_once(':').expect("generated map entry");
                (key, slot.parse().expect("generated slot"))
            }).collect();
            (properties.split(',').filter(|p| !p.is_empty()).collect::<Vec<_>>(), slots)
        }).collect::<Vec<_>>();
        // blocks.tsv: block, its map, then the model id of each slot.
        let tables = BLOCK_MODELS.lines().map(|line| {
            let [name, map, ids] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated block row") };
            let ids = ids.split(',').map(|id| id.parse().expect("generated model id")).collect::<Vec<u16>>();
            (name, (&maps[map.parse::<usize>().expect("generated map index")], ids))
        }).collect::<BTreeMap<_, _>>();
        // census.tsv: block, bucket, state count; every bucket other than "modeled" names a refusal.
        let census: BTreeMap<_, _> = STATE_CENSUS.lines().filter(|l| !l.starts_with('#')).map(|line| {
            let [block, bucket, _] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated census row") };
            (block, bucket)
        }).collect();
        // states.tsv: every block in registry order, its default's index, its properties' values.
        let (mut names, mut states, mut refusals, mut state_block, mut state_model) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for line in jar_table!("states.tsv").lines() {
            let mut f = line.split('\t');
            let name = f.next().expect("generated state row");
            let default = f.next().and_then(|d| d.parse().ok()).expect("generated default");
            let properties: Vec<_> = f.map(|p| { let (k, v) = p.split_once(':').expect("generated property"); (k, v.split(',').collect::<Vec<_>>()) }).collect();
            let count: usize = properties.iter().map(|(_, v)| v.len()).product();
            let bucket = census[name];
            for local in 0..count {
                state_block.push(BlockId(names.len() as u16));
                state_model.push(tables.get(name).filter(|_| bucket == "modeled").map_or(NO_MODEL, |((relevant, slots), ids)| {
                    ids[slots[relevant.iter().map(|p| digit(&properties, local, p).expect("relevant property")).collect::<Vec<_>>().join(",").as_str()]]
                }));
            }
            states.push(States { first: (state_block.len() - count) as u16, default, properties });
            refusals.push((bucket != "modeled").then_some(bucket));
            names.push(name);
        }
        assert!(state_block.len() < u16::MAX as usize, "registry ids must fit in a StateId");
        let rows: BTreeMap<_, _> = names.iter().enumerate().map(|(i, &n)| (n, BlockId(i as u16))).collect();
        let (mut rules, mut rule_rows, mut rule_ids) = (Vec::new(), Vec::new(), BTreeMap::new());
        for line in jar_table!("rules.tsv").lines().filter(|line| !line.starts_with('#')) {
            let (_, row) = line.split_once('\t').expect("generated rule name");
            rule_rows.push(*rule_ids.entry(row).or_insert_with(|| {
                let id = u16::try_from(rules.len()).expect("rule id fits in u16");
                rules.push(parse_rules(row));
                id
            }));
        }
        let mut state_rules = Vec::with_capacity(state_block.len());
        for line in jar_table!("cell_rule_index.tsv").lines().filter(|line| !line.starts_with('#')) {
            let (end, id) = line.split_once('\t').expect("generated rule index");
            let (end, row): (usize, usize) = (end.parse().unwrap(), id.parse().unwrap());
            assert!(end > state_rules.len() && end <= state_block.len());
            state_rules.resize(end, rule_rows[row]);
        }
        assert_eq!(state_rules.len(), state_block.len());
        Models { names, rows, states, state_block: state_block.into(), state_model: state_model.into(), blocks, offset_variants, reach, refusals, rules, state_rules: state_rules.into() }
    })
}

/// The value of property `name` in the state of local index `local`.
fn digit(properties: &[(&'static str, Vec<&'static str>)], mut local: usize, name: &str) -> Option<&'static str> {
    for (p, values) in properties.iter().rev() {
        if *p == name { return Some(values[local % values.len()]); }
        local /= values.len();
    }
    None
}

impl BlockId {
    /// The block named `name` (`minecraft:` may be left out).
    pub fn of(name: &str) -> Self {
        let full = if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") };
        *models().rows.get(full.as_str()).unwrap_or_else(|| panic!("{full} is not a {MINECRAFT_VERSION} block"))
    }
    /// The name without `minecraft:`.
    pub fn name(self) -> &'static str {
        models().names[self.0 as usize].trim_start_matches("minecraft:")
    }
    pub fn default_state(self) -> StateId {
        let s = &models().states[self.0 as usize];
        StateId(s.first + s.default)
    }

    pub fn rules(self) -> Rules { self.default_state().rules() }
}

impl Rules {
    pub fn replaces_with(self, kind: FlowKind) -> bool {
        match kind {
            FlowKind::Water => self.replaceable_by_water,
            FlowKind::Lava { .. } => self.replaceable_by_lava,
        }
    }
}

impl StateId {
    /// The state of block `name` whose properties `value` names; one it leaves out takes the
    /// default state's value.
    pub fn parse<'a>(name: &str, value: impl Fn(&str) -> Option<&'a str>) -> Result<Self, String> {
        let m = models();
        let block = *m.rows.get(name).ok_or_else(|| format!("{name} is not a {MINECRAFT_VERSION} registry block"))?;
        let s = &m.states[block.0 as usize];
        let (mut local, mut place) = (0, 1);
        for (p, values) in s.properties.iter().rev() {
            let v = value(p).or_else(|| digit(&s.properties, s.default as usize, p));
            local += values.iter().position(|x| Some(*x) == v).ok_or_else(|| format!("state unknown to the {MINECRAFT_VERSION} table of {name}"))? * place;
            place *= values.len();
        }
        Ok(Self(s.first + local as u16))
    }
    /// The state of `block` with `properties`, the rest at the default's values.
    pub fn of(block: BlockId, properties: &[(&str, &str)]) -> Self {
        let name = models().names[block.0 as usize];
        Self::parse(name, |p| properties.iter().find(|(k, _)| *k == p).map(|(_, v)| *v)).unwrap_or_else(|e| panic!("{name} {properties:?}: {e}"))
    }
    pub fn block(self) -> BlockId { models().state_block[self.0 as usize] }
    pub fn rules(self) -> Rules {
        let m = models();
        m.rules[m.state_rules[self.0 as usize] as usize]
    }

    /// Set a single property without losing any of the state's other digits.
    pub fn with(self, name: &str, value: &str) -> Self {
        let s = &models().states[self.block().0 as usize];
        let local = usize::from(self.0 - s.first);
        let mut stride = 1;
        for (property, values) in s.properties.iter().rev() {
            if *property == name {
                let old = local / stride % values.len();
                let new = values.iter().position(|v| *v == value).expect("valid state property value");
                return Self((usize::from(self.0) - old * stride + new * stride) as u16);
            }
            stride *= values.len();
        }
        panic!("{} has no {name} property", self.block().name());
    }

    pub fn with_bool(self, name: &str, value: bool) -> Self {
        self.with(name, if value { "true" } else { "false" })
    }
    /// The value of property `name`, if the block has it.
    pub fn get(self, name: &str) -> Option<&'static str> {
        let s = &models().states[self.block().0 as usize];
        digit(&s.properties, (self.0 - s.first) as usize, name)
    }
    /// The model row; a refused state (a non-local shape) has none.
    pub fn model(self) -> Option<ModelId> {
        Some(models().state_model[self.0 as usize]).filter(|&m| m != NO_MODEL).map(ModelId)
    }
    /// The model's row, unshifted. A moving piston's shape lives in its block entity, not here.
    pub fn model_block(self) -> &'static Block {
        match self.model() {
            Some(ModelId(id)) => &models().blocks[id as usize],
            None => {
                assert_eq!(self.block().name(), "moving_piston", "the law refuses this state");
                static MOVING: OnceLock<Block> = OnceLock::new();
                MOVING.get_or_init(|| Block { air: false, fluid_flow: Some(FluidFlowBlock { blocks_motion: true, solid_faces: [false; 4] }), ..Block::default() })
            }
        }
    }
    /// What the law reads of the cell at `p` holding this state.
    pub fn block_at(self, p: crate::BlockPos) -> &'static Block {
        self.model().map_or_else(|| self.model_block(), |m| m.block_at(p))
    }
}

impl ModelId {
    /// The row the law reads at `p`: shifted there for an offset model (bamboo, dripstone).
    #[inline]
    pub fn block_at(self, p: crate::BlockPos) -> &'static Block {
        let m = models();
        if let Some(variants) = &m.offset_variants[self.0 as usize] {
            let l = crate::block_hooks::get_seed(p[0], 0, p[2]);
            return &variants[(((l & 15) as usize) << 4) | (((l >> 8) & 15) as usize)];
        }
        &m.blocks[self.0 as usize]
    }
    /// Whether the row is the same at every cell (no offset model).
    pub fn fixed(self) -> bool {
        models().offset_variants[self.0 as usize].is_none()
    }
}

/// A state, or the reason the law refuses it.
pub fn lookup<'a>(name: &str, properties: impl Fn(&str) -> Option<&'a str>) -> Result<StateId, String> {
    let state = StateId::parse(name, properties)?;
    match models().refusals[state.block().0 as usize] { Some(reason) => Err(reason.into()), None => Ok(state) }
}

fn parse_rules(row: &str) -> Rules {
    let f: Vec<_> = row.split('\t').collect();
    assert!((12..=13).contains(&f.len()), "generated rule schema: {row}");
    let yes = |i: usize| f[i].parse::<bool>().expect("generated rule flag");
    let push_reaction = match f[4] {
        "NORMAL" | "PUSH_PULL" => PushReaction::Normal,
        "DESTROY" | "POPPED" => PushReaction::Destroy,
        "BLOCK" | "IMMOVEABLE" => PushReaction::Block,
        "IGNORE" | "IGNORE_ENTITY" => PushReaction::Ignore,
        "PUSH_ONLY" | "PUSH" => PushReaction::PushOnly,
        s => panic!("generated push reaction {s}"),
    };
    let (air, water_container) = f[11].split_once(':').expect("generated air and water flags");
    let water_container = match water_container {
        "no" => WaterContainer::None, "accepts" => WaterContainer::Accepts,
        "rejects" => WaterContainer::Rejects, "other" => WaterContainer::Other, s => panic!("generated water container {s}"),
    };
    Rules {
        destroy_speed: f[0].parse().expect("generated hardness"),
        requires_correct_tool: f[1] == "y", redstone_conductor: (f[2] != "-").then(|| yes(2)),
        sturdy_faces: (f[3] != "-").then(|| f[3].parse().expect("generated sturdy mask")),
        push_reaction, blocks_motion: yes(5), replaceable_by_water: yes(6), replaceable_by_lava: yes(7),
        can_be_replaced: yes(8), is_signal_source: yes(9), has_block_entity: f[10] == "y",
        is_air: air.parse().expect("generated air flag"), water_container,
        fluid_boundary: f.get(12).map(|flags| {
            let (washed_away, solid) = flags.split_once(':').expect("generated fluid boundary");
            FluidBoundary { washed_away: washed_away.parse().unwrap(), solid: solid.parse().unwrap() }
        }),
    }
}

fn shape(raw: &str) -> VoxelShape {
    let boxes = raw.split(';').filter(|s| !s.is_empty()).map(|s| {
        let v = s.split(',').map(|s| s.parse::<f64>().expect("generated box number")).collect::<Vec<_>>();
        assert_eq!(v.len(), 6);
        Aabb::new(v[0], v[1], v[2], v[3], v[4], v[5])
    }).collect::<Vec<_>>();
    VoxelShape::boxes(&boxes)
}

fn parse_model(line: &str) -> (Block, Option<f32>) {
    let f: Vec<_> = line.split('\t').collect();
    assert_eq!(f.len(), 22, "generated model schema");
    let fp = |i: usize| f32::from_bits(u32::from_str_radix(f[i], 16).expect("generated float bits"));
    let bits: u16 = f[5].parse().unwrap();
    let flag = |i: usize| bits & (1u16 << i) != 0;
    let direction = |i: usize| match f[i] { "-1" => None, s => Some(s.parse().unwrap()) };
    let amount: u8 = f[9].parse().unwrap();
    let faces: u8 = f[12].parse().unwrap();
    let block = Block {
        shape: shape(f[15]), inside_shape: Some(shape(f[16])),
        above_shape: Some(shape(f[17])), below_shape: Some(shape(f[18])), sneak_shape: Some(shape(f[19])), inside_cell_shape: Some(shape(f[20])),
        friction: fp(0), speed_factor: fp(1), jump_factor: fp(2), bounce: fp(3), fall_on_multiplier: fp(4),
        air: flag(0), suffocating: flag(1), water_block: flag(2), climbable: flag(3), scaffolding: flag(4),
        slime: flag(5), bed: flag(6), fence: flag(7), wall_or_gate: flag(8), suppresses_bounce: flag(9),
        resets_fall_distance: flag(10), end_portal_or_gateway: flag(11), world_hook: flag(12),
        ladder_facing: direction(6), open_trapdoor_facing: direction(7),
        fluid: Fluid {
            kind: match f[8] { "0" => FluidKind::Empty, "1" => FluidKind::Water, "2" => FluidKind::Lava, _ => panic!("generated fluid kind") },
            raw_amount: (amount != 0).then_some(amount),
            raw_falling: (amount != 0).then(|| f[10].parse().unwrap()),
            ..Fluid::default()
        },
        fluid_flow: Some(FluidFlowBlock { blocks_motion: f[11].parse().unwrap(), solid_faces: std::array::from_fn(|i| faces & (1 << i) != 0) }),
        inside: match f[13] {
            "none" => Inside::None, "cobweb" => Inside::Cobweb, "powderSnow" => Inside::PowderSnow, "honey" => Inside::Honey,
            "bubbleColumnUp" => Inside::BubbleColumn { drag: false }, "bubbleColumnDown" => Inside::BubbleColumn { drag: true },
            "cactus" => Inside::Cactus, "campfire" => Inside::Campfire { soul: false }, "soulCampfire" => Inside::Campfire { soul: true },
            "fire" => Inside::Fire { soul: false }, "soulFire" => Inside::Fire { soul: true },
            "berryBush" => Inside::BerryBush { hurt: false }, "berryBushHurt" => Inside::BerryBush { hurt: true },
            "extinguish" => Inside::Extinguish, "lava" => Inside::Lava, "endPortal" => Inside::EndPortal,
            "netherPortal" => Inside::NetherPortal, "witherRose" => Inside::WitherRose,
            other => panic!("generated inside hook {other}"),
        },
        grounded_contact: match f[14] { "none" => GroundedContact::None, "hotFloor" => GroundedContact::DamageUnlessCareful(ContactDamage::HotFloor), _ => panic!("generated step hook") },
        ..Block::default()
    };
    let offset = match f[21].split(':').next().unwrap() {
        "none" => None,
        "bamboo" => Some(crate::block_hooks::BAMBOO_MAX_HORIZONTAL_OFFSET),
        "speleothem" => Some(crate::block_hooks::SPELEOTHEM_MAX_HORIZONTAL_OFFSET),
        other => panic!("generated offset spec {other}"),
    };
    (block, offset)
}


// ============================================================================
// The cell: a block id and its family's state, one canonical state; one world to edit
// ============================================================================

use crate::falling::AnvilStage;
use crate::fluid::{FlowKind, FluidCell};
use crate::interact::{AttachFace, DoorHalf, LeverState};
use crate::neighbor::Direction;
use crate::piston::PistonBlock;
use crate::place::{DoorHinge, DoorPlaceState, Half, ScaffoldingState};
use crate::world_hooks::{ButtonState, PressurePlateKind, PressurePlateState, TripwireHookState, TripwireWireState};
use crate::BlockPos;

/// The edit side of the world: one cell representation, read and written by every rule.
pub trait WorldMut {
    fn cell(&self, pos: BlockPos) -> StateId;
    fn set(&mut self, pos: BlockPos, cell: StateId);
    fn sturdy(&self, pos: BlockPos, face: Direction) -> bool { self.cell(pos).is_sturdy(face) }
    fn can_survive(&self, pos: BlockPos, state: StateId) -> bool {
        let support = match state.survive_rule() {
            SurviveRule::Always => return true,
            SurviveRule::FloorSturdy => Direction::Down,
            SurviveRule::Attached => match state.kind() {
                CellState::Lever(lever) => lever.connected_direction().opposite(),
                CellState::Button(button) => button.connected_direction().opposite(),
                CellState::TripwireHook(hook) => hook.facing.opposite(),
                _ => state.direction("facing").opposite(),
            },
        };
        self.sturdy(support.relative(pos), support.opposite())
    }
    /// `LevelTicks.hasScheduledTick`: a tick of `block` is pending at `pos`.
    fn has_scheduled_tick(&self, _pos: BlockPos, _block: BlockId) -> bool {
        false
    }
}

/// A plain map world (unset cells are air), for rules run on their own.
impl WorldMut for std::collections::HashMap<BlockPos, StateId> {
    fn cell(&self, pos: BlockPos) -> StateId {
        self.get(&pos).copied().unwrap_or(StateId::AIR)
    }
    fn set(&mut self, pos: BlockPos, cell: StateId) {
        self.insert(pos, cell);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallingKind {
    Sand,
    Anvil { facing_x_axis: bool, stage: AnvilStage },
    ConcretePowder,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Empty,
    Plain,
    Fluid { kind: FlowKind, cell: FluidCell },
    RedstoneWire { power: u8 },
    Lever(LeverState),
    Door(DoorPlaceState),
    TrapDoor { facing: Direction, half: Half, open: bool, powered: bool, waterlogged: bool },
    FenceGate { facing: Direction, open: bool, powered: bool },
    Button(ButtonState),
    PressurePlate(PressurePlateState),
    TripwireWire(TripwireWireState),
    TripwireHook(TripwireHookState),
    Scaffolding(ScaffoldingState),
    Piston { sticky: bool, facing: Direction, extended: bool },
    PistonHead { sticky: bool, facing: Direction },
    MovingPiston { sticky: bool, facing: Direction },
    Falling(FallingKind),
}

pub const DIRECTION: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
pub const FACE: [&str; 3] = ["floor", "wall", "ceiling"];
const ANVILS: [&str; 3] = ["anvil", "chipped_anvil", "damaged_anvil"];

impl StateId {
    pub const AIR: Self = Self(0);

    pub fn liquid(name: &str, cell: FluidCell) -> Self {
        Self::of(BlockId::of(name), &[("level", &cell.legacy_level().to_string())])
    }

    pub fn direction(self, name: &str) -> Direction {
        let value = self.get(name).expect("direction property");
        Direction::ALL[DIRECTION.iter().position(|d| *d == value).expect("direction value")]
    }

    /// Decode a rule's inputs from the canonical state.
    pub fn kind(self) -> CellState {
        let s = self;
        let id = s.block();
        let name = id.name();
        let get = |p: &str| s.get(p).unwrap_or_else(|| panic!("{name} has no {p}"));
        let on = |p: &str| get(p) == "true";
        let number = |p: &str| get(p).parse::<u8>().expect("a number property");
        let dir = |p: &str| s.direction(p);
        let face = || [AttachFace::Floor, AttachFace::Wall, AttachFace::Ceiling][FACE.iter().position(|f| *f == get("face")).expect("a face")];
        // BlockSetType: stone buttons and plates sense mobs only, stay 20 ticks, ignore arrows.
        let stone = name.starts_with("stone_") || name.starts_with("polished_blackstone_");
        match name {
            "air" | "cave_air" | "void_air" => CellState::Empty,
            "water" | "lava" => {
                let l = number("level");
                let cell = FluidCell { amount: if l == 0 || l >= 8 { 8 } else { 8 - l }, falling: l >= 8, source: l == 0 };
                CellState::Fluid { kind: if name == "water" { FlowKind::Water } else { FlowKind::Lava { fast: false } }, cell }
            }
            "redstone_wire" => CellState::RedstoneWire { power: number("power") },
            "lever" => CellState::Lever(LeverState { block_id: id, face: face(), facing: dir("facing"), powered: on("powered") }),
            "tripwire" => CellState::TripwireWire(TripwireWireState { block_id: id, powered: on("powered"), attached: on("attached"), disarmed: on("disarmed") }),
            "tripwire_hook" => CellState::TripwireHook(TripwireHookState { block_id: id, facing: dir("facing"), powered: on("powered"), attached: on("attached") }),
            "scaffolding" => CellState::Scaffolding(ScaffoldingState { distance: number("distance"), bottom: on("bottom"), waterlogged: on("waterlogged") }),
            "piston" | "sticky_piston" => CellState::Piston { sticky: name == "sticky_piston", facing: dir("facing"), extended: on("extended") },
            "piston_head" => CellState::PistonHead { sticky: get("type") == "sticky", facing: dir("facing") },
            "moving_piston" => CellState::MovingPiston { sticky: get("type") == "sticky", facing: dir("facing") },
            "sand" | "red_sand" | "gravel" => CellState::Falling(FallingKind::Sand),
            _ if ANVILS.contains(&name) => CellState::Falling(FallingKind::Anvil {
                facing_x_axis: matches!(get("facing"), "east" | "west"),
                stage: [AnvilStage::Intact, AnvilStage::Chipped, AnvilStage::Damaged][ANVILS.iter().position(|a| *a == name).expect("an anvil")],
            }),
            _ if name.ends_with("_concrete_powder") => CellState::Falling(FallingKind::ConcretePowder),
            _ if name.ends_with("_trapdoor") => CellState::TrapDoor {
                facing: dir("facing"), half: if get("half") == "top" { Half::Top } else { Half::Bottom }, open: on("open"), powered: on("powered"), waterlogged: on("waterlogged"),
            },
            _ if name.ends_with("_door") => CellState::Door(DoorPlaceState {
                block_id: id, facing: dir("facing"), half: if get("half") == "upper" { DoorHalf::Upper } else { DoorHalf::Lower },
                hinge: if get("hinge") == "left" { DoorHinge::Left } else { DoorHinge::Right }, open: on("open"), powered: on("powered"),
            }),
            _ if name.ends_with("_fence_gate") => CellState::FenceGate { facing: dir("facing"), open: on("open"), powered: on("powered") },
            _ if name.ends_with("_button") => CellState::Button(ButtonState {
                block_id: id, face: face(), facing: dir("facing"), powered: on("powered"), ticks_to_stay_pressed: if stone { 20 } else { 30 }, arrow_activatable: !stone,
            }),
            _ if name.ends_with("_pressure_plate") => CellState::PressurePlate(match s.get("power") {
                Some(power) => PressurePlateState { block_id: id, kind: PressurePlateKind::Weighted { max_weight: if name.starts_with("light") { 15 } else { 150 } }, signal: power.parse().expect("a power") },
                None => PressurePlateState { block_id: id, kind: PressurePlateKind::Binary { mobs_only: stone }, signal: if on("powered") { 15 } else { 0 } },
            }),
            _ => CellState::Plain,
        }
    }

    pub fn is_air(self) -> bool { self.rules().is_air }

    pub fn is_redstone_conductor(self) -> bool {
        self.rules().redstone_conductor.expect("redstone conductor needs a world context")
    }

    pub fn is_sturdy(self, face: Direction) -> bool {
        self.rules().sturdy_faces.expect("sturdy face needs a world context") & (1 << face as u8) != 0
    }

    pub fn survive_rule(self) -> SurviveRule {
        match self.block().name() {
            "torch" | "redstone_wire" | "stone_pressure_plate" => SurviveRule::FloorSturdy,
            "lever" | "stone_button" | "oak_button" | "tripwire_hook" | "ladder" => SurviveRule::Attached,
            _ => SurviveRule::Always,
        }
    }

    pub fn fluid_info(self, fast_lava: bool) -> Option<(FlowKind, FluidCell)> {
        match self.kind() {
            CellState::Fluid { kind, cell } => {
                let kind = if matches!(kind, FlowKind::Lava { .. }) { FlowKind::Lava { fast: fast_lava } } else { kind };
                Some((kind, cell))
            }
            _ if self.get("waterlogged") == Some("true") => Some((FlowKind::Water, FluidCell::WATER_SOURCE)),
            _ => None,
        }
    }

    pub fn is_waterlogged_container(self) -> bool {
        self.rules().water_container == WaterContainer::Accepts
    }

    pub fn piston_kind(self) -> PistonKind {
        let extended = || self.get("extended") == Some("true");
        match self.block().name() {
            "obsidian" => PistonKind::Obsidian,
            "crying_obsidian" => PistonKind::CryingObsidian,
            "respawn_anchor" => PistonKind::RespawnAnchor,
            "reinforced_deepslate" => PistonKind::ReinforcedDeepslate,
            "slime_block" => PistonKind::SlimeBlock,
            "honey_block" => PistonKind::HoneyBlock,
            "piston" => PistonKind::Piston { extended: extended() },
            "sticky_piston" => PistonKind::StickyPiston { extended: extended() },
            "piston_head" => PistonKind::PistonHead,
            "moving_piston" => PistonKind::MovingPiston,
            _ => PistonKind::Other,
        }
    }

    pub fn as_piston_block(self) -> PistonBlock {
        let props = self.rules();
        PistonBlock {
            air: props.is_air,
            destroy_speed: props.destroy_speed,
            has_block_entity: props.has_block_entity,
            push_reaction: props.push_reaction,
            kind: self.piston_kind(),
        }
    }
}
