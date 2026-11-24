//! `cell`: a cell holds a canonical 26.2 block state, and its physics is the world crate's
//! generated tables' row for it. `states.tsv` names every block in registry order (its row is the
//! block's id) with its properties' values, so a state's id is Mojang's own; `blocks.tsv` holds the
//! model of each state its relevant properties select (`maps.tsv`); `models.tsv` holds one row per
//! distinct model: boxes, friction, speed and jump factors, the inside and step hooks; `census.tsv`
//! names the blocks refused. `rules.tsv` (beside this file) holds what a rule reads of a block and
//! the model does not: hardness, push reaction, conductor, signal source, survival.
use crate::block_hooks::{OffsetCollisionSpec, SpeleothemThickness, TipDirection, offset_collision_shape_for_keys};
use crate::piston::{BlockKind as PistonKind, PushReaction};
use crate::{Aabb, Block, ContactDamage, Fluid, FluidFlowBlock, FluidKind, GroundedContact, Inside, VoxelShape};
use std::{collections::BTreeMap, sync::OnceLock};

/// A block's row in `states.tsv`: its 26.2 registry id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u16);
/// A cell's identity: the 26.2 block-state registry id (`Block.BLOCK_STATE_REGISTRY`), exact for
/// each of the 32,366 states.
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
    rules: Vec<Option<Rules>>,
}

/// What a rule reads of a block beyond its model (`rules.tsv`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub destroy_speed: f32,
    pub requires_correct_tool: bool,
    pub is_redstone_conductor: bool,
    /// Sturdy faces, `[Down, Up, North, South, West, East]`.
    pub sturdy_faces: [bool; 6],
    pub push_reaction: PushReaction,
    pub piston_kind: PistonKind,
    pub blocks_motion: bool,
    pub replaceable_by_fluid: bool,
    pub can_be_replaced: bool,
    pub is_signal_source: bool,
    pub has_block_entity: bool,
    pub survive_rule: SurviveRule,
}

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
        for line in include_str!("../../../world/data/models.tsv").lines() {
            let (block, offset) = parse_model(line);
            let (variants, r) = match offset {
                OffsetCollisionSpec::None => (None, if block.never_collides() { NOWHERE } else if block.collision_within_cell() { OWN_CELL } else { BEYOND }),
                spec => {
                    let list = (0..256).map(|k| {
                        let shifted = offset_collision_shape_for_keys(spec, (k >> 4) as u8, (k & 15) as u8).expect("non-none offset spec");
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
        let maps = include_str!("../../../world/data/maps.tsv").lines().map(|line| {
            let (properties, entries) = line.split_once('\t').expect("generated map row");
            let slots: BTreeMap<_, usize> = entries.split(' ').map(|e| {
                let (key, slot) = e.rsplit_once(':').expect("generated map entry");
                (key, slot.parse().expect("generated slot"))
            }).collect();
            (properties.split(',').filter(|p| !p.is_empty()).collect::<Vec<_>>(), slots)
        }).collect::<Vec<_>>();
        // blocks.tsv: block, its map, then the model id of each slot.
        let tables = include_str!("../../../world/data/blocks.tsv").lines().map(|line| {
            let [name, map, ids] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated block row") };
            let ids = ids.split(',').map(|id| id.parse().expect("generated model id")).collect::<Vec<u16>>();
            (name, (&maps[map.parse::<usize>().expect("generated map index")], ids))
        }).collect::<BTreeMap<_, _>>();
        // census.tsv: block, bucket, state count; every bucket other than "modeled" names a refusal.
        let census: BTreeMap<_, _> = include_str!("../../../world/data/census.tsv").lines().filter(|l| !l.starts_with('#')).map(|line| {
            let [block, bucket, _] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated census row") };
            (block, bucket)
        }).collect();
        // states.tsv: every block in registry order, its default's index, its properties' values.
        let (mut names, mut states, mut refusals, mut state_block, mut state_model) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for line in include_str!("../../../world/data/states.tsv").lines() {
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
        assert_eq!(state_block.len(), 32_366, "26.2 registry states");
        let rows: BTreeMap<_, _> = names.iter().enumerate().map(|(i, &n)| (n, BlockId(i as u16))).collect();
        let mut rules = vec![None; names.len()];
        for line in include_str!("rules.tsv").lines().filter(|l| !l.starts_with('#')) {
            let (name, row) = parse_rules(line);
            let at = *rows.get(format!("minecraft:{name}").as_str()).unwrap_or_else(|| panic!("rules.tsv: {name} is not a 26.2 block"));
            rules[at.0 as usize] = Some(row);
        }
        Models { names, rows, states, state_block: state_block.into(), state_model: state_model.into(), blocks, offset_variants, reach, refusals, rules }
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
        *models().rows.get(full.as_str()).unwrap_or_else(|| panic!("{full} is not a 26.2 block"))
    }
    /// The name without `minecraft:`.
    pub fn name(self) -> &'static str {
        models().names[self.0 as usize].trim_start_matches("minecraft:")
    }
    /// The rules row; a block `rules.tsv` does not list has none.
    pub fn rules(self) -> Rules {
        models().rules[self.0 as usize].unwrap_or_else(|| panic!("rules.tsv has no row for {}", self.name()))
    }
}

impl StateId {
    /// The state of block `name` whose properties `value` names; one it leaves out takes the
    /// default state's value.
    pub fn parse<'a>(name: &str, value: impl Fn(&str) -> Option<&'a str>) -> Result<Self, String> {
        let m = models();
        let block = *m.rows.get(name).ok_or_else(|| format!("{name} is not a 26.2 registry block"))?;
        let s = &m.states[block.0 as usize];
        let (mut local, mut place) = (0, 1);
        for (p, values) in s.properties.iter().rev() {
            let v = value(p).or_else(|| digit(&s.properties, s.default as usize, p));
            local += values.iter().position(|x| Some(*x) == v).ok_or_else(|| format!("state unknown to the 26.2 table of {name}"))? * place;
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

fn parse_rules(line: &str) -> (&str, Rules) {
    let f: Vec<_> = line.split('\t').collect();
    assert_eq!(f.len(), 13, "rules.tsv schema: {line}");
    let yes = |i: usize| match f[i] { "y" => true, "n" => false, s => panic!("rules.tsv flag {s}") };
    let sturdy = match f[4] { "all" => [true; 6], "none" => [false; 6], s => panic!("rules.tsv sturdy {s}") };
    let push_reaction = match f[5] { "normal" => PushReaction::Normal, "destroy" => PushReaction::Destroy, "block" => PushReaction::Block, s => panic!("rules.tsv push {s}") };
    let piston_kind = match f[6] {
        "other" => PistonKind::Other, "obsidian" => PistonKind::Obsidian, "slime" => PistonKind::SlimeBlock,
        "honey" => PistonKind::HoneyBlock, "piston" => PistonKind::Piston { extended: false },
        "sticky" => PistonKind::StickyPiston { extended: false }, "head" => PistonKind::PistonHead,
        "moving" => PistonKind::MovingPiston, s => panic!("rules.tsv piston kind {s}"),
    };
    let survive_rule = match f[12] { "always" => SurviveRule::Always, "floor" => SurviveRule::FloorSturdy, "attached" => SurviveRule::Attached, s => panic!("rules.tsv survive {s}") };
    (f[0], Rules {
        destroy_speed: f[1].parse().expect("rules.tsv hardness"), requires_correct_tool: yes(2), is_redstone_conductor: yes(3),
        sturdy_faces: sturdy, push_reaction, piston_kind, blocks_motion: yes(7), replaceable_by_fluid: yes(8),
        can_be_replaced: yes(9), is_signal_source: yes(10), has_block_entity: yes(11), survive_rule,
    })
}

fn shape(raw: &str) -> VoxelShape {
    let boxes = raw.split(';').filter(|s| !s.is_empty()).map(|s| {
        let v = s.split(',').map(|s| s.parse::<f64>().expect("generated box number")).collect::<Vec<_>>();
        assert_eq!(v.len(), 6);
        Aabb::new(v[0], v[1], v[2], v[3], v[4], v[5])
    }).collect::<Vec<_>>();
    VoxelShape::boxes(&boxes)
}

fn parse_offset(raw: &str) -> OffsetCollisionSpec {
    match raw.split(':').collect::<Vec<_>>()[..] {
        ["none"] => OffsetCollisionSpec::None,
        ["bamboo"] => OffsetCollisionSpec::Bamboo,
        ["speleothem", thickness, tip] => OffsetCollisionSpec::Speleothem {
            thickness: match thickness {
                "tip_merge" => SpeleothemThickness::TipMerge, "tip" => SpeleothemThickness::Tip,
                "frustum" => SpeleothemThickness::Frustum, "middle" => SpeleothemThickness::Middle,
                "base" => SpeleothemThickness::Base, other => panic!("generated speleothem thickness {other}"),
            },
            tip_direction: match tip { "up" => TipDirection::Up, "down" => TipDirection::Down, other => panic!("generated tip direction {other}") },
        },
        _ => panic!("generated offset spec {raw}"),
    }
}

fn parse_model(line: &str) -> (Block, OffsetCollisionSpec) {
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
    (block, parse_offset(f[21]))
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
    fn cell(&self, pos: BlockPos) -> SimCell;
    fn set(&mut self, pos: BlockPos, cell: SimCell);
    /// `LevelTicks.hasScheduledTick`: a tick of `block` is pending at `pos`.
    fn has_scheduled_tick(&self, _pos: BlockPos, _block: BlockId) -> bool {
        false
    }
}

/// A plain map world (unset cells are air), for rules run on their own.
impl WorldMut for std::collections::HashMap<BlockPos, SimCell> {
    fn cell(&self, pos: BlockPos) -> SimCell {
        self.get(&pos).copied().unwrap_or_else(|| SimCell::air(BlockId::of("air")))
    }
    fn set(&mut self, pos: BlockPos, cell: SimCell) {
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

/// A rule's view of one canonical state: the block and the facts of its family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimCell {
    pub id: BlockId,
    pub state: CellState,
}

pub const DIRECTION: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
const FACE: [&str; 3] = ["floor", "wall", "ceiling"];
const ANVILS: [&str; 3] = ["anvil", "chipped_anvil", "damaged_anvil"];

impl SimCell {
    pub fn new(id: BlockId, state: CellState) -> Self {
        Self { id, state }
    }

    pub fn air(id: BlockId) -> Self {
        Self::new(id, CellState::Empty)
    }

    pub fn plain(id: BlockId) -> Self {
        Self::new(id, CellState::Plain)
    }

    pub fn water(id: BlockId, cell: FluidCell) -> Self {
        Self::new(id, CellState::Fluid { kind: FlowKind::Water, cell })
    }

    pub fn lava(id: BlockId, fast: bool, cell: FluidCell) -> Self {
        Self::new(id, CellState::Fluid { kind: FlowKind::Lava { fast }, cell })
    }

    /// The canonical state this cell is: the first of its block's states, the default first,
    /// that `from_state` reads as this cell. A cell no state holds is refused.
    pub fn state_id(&self) -> StateId {
        static CELLS: OnceLock<Vec<Vec<(CellState, StateId)>>> = OnceLock::new();
        let cells = CELLS.get_or_init(|| models().states.iter().map(|s| {
            let count = s.properties.iter().map(|(_, v)| v.len() as u16).product();
            let mut cells: Vec<(CellState, StateId)> = Vec::new();
            for id in std::iter::once(s.default).chain(0..count).map(|l| StateId(s.first + l)) {
                let cell = Self::from_state(id, false).state;
                if !cells.iter().any(|c| c.0 == cell) { cells.push((cell, id)); }
            }
            cells
        }).collect());
        let mut state = self.state;
        if let CellState::Fluid { kind: FlowKind::Lava { fast }, .. } = &mut state { *fast = false; }
        cells[self.id.0 as usize].iter().find(|c| c.0 == state).map(|c| c.1).unwrap_or_else(|| panic!("no canonical state holds {self:?}"))
    }

    /// The cell a canonical state is (`fast_lava` is the dimension's, not the state's).
    pub fn from_state(s: StateId, fast_lava: bool) -> Self {
        let id = s.block();
        let name = id.name();
        let get = |p: &str| s.get(p).unwrap_or_else(|| panic!("{name} has no {p}"));
        let on = |p: &str| get(p) == "true";
        let number = |p: &str| get(p).parse::<u8>().expect("a number property");
        let dir = |p: &str| Direction::ALL[DIRECTION.iter().position(|d| *d == get(p)).expect("a direction")];
        let face = || [AttachFace::Floor, AttachFace::Wall, AttachFace::Ceiling][FACE.iter().position(|f| *f == get("face")).expect("a face")];
        // BlockSetType: stone buttons and plates sense mobs only, stay 20 ticks, ignore arrows.
        let stone = name.starts_with("stone_") || name.starts_with("polished_blackstone_");
        let state = match name {
            "air" | "cave_air" | "void_air" => CellState::Empty,
            "water" | "lava" => {
                let l = number("level");
                let cell = FluidCell { amount: if l == 0 || l >= 8 { 8 } else { 8 - l }, falling: l >= 8, source: l == 0 };
                CellState::Fluid { kind: if name == "water" { FlowKind::Water } else { FlowKind::Lava { fast: fast_lava } }, cell }
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
        };
        Self { id, state }
    }

    /// The model the state selects (`blocks.tsv`, `maps.tsv`), unshifted.
    pub fn block(&self) -> &'static Block {
        self.state_id().model_block()
    }

    /// What a rule reads beyond the model: the block's rules row (a block `rules.tsv` lacks gets
    /// the projection of its model), with a piston's kind and sturdy faces from its state.
    pub fn props(&self) -> Rules {
        let mut props = models().rules[self.id.0 as usize].unwrap_or_else(|| {
            let b = self.block();
            let full = b.shape.is_block();
            Rules {
                destroy_speed: 1.5, requires_correct_tool: true, is_redstone_conductor: full,
                sturdy_faces: b.fluid_flow.map_or([full; 6], |ff| [full, full, ff.solid_faces[0], ff.solid_faces[2], ff.solid_faces[3], ff.solid_faces[1]]),
                push_reaction: PushReaction::Normal, piston_kind: PistonKind::Other,
                blocks_motion: b.fluid_flow.map_or(!b.shape.is_empty(), |ff| ff.blocks_motion),
                replaceable_by_fluid: true, can_be_replaced: false, is_signal_source: false, has_block_entity: false, survive_rule: SurviveRule::Always,
            }
        });
        match self.state {
            CellState::Piston { sticky, facing, extended } => {
                props.piston_kind = if sticky { PistonKind::StickyPiston { extended } } else { PistonKind::Piston { extended } };
                props.sturdy_faces = [!extended; 6];
                props.sturdy_faces[facing.opposite() as usize] = true;
            }
            CellState::PistonHead { facing, .. } => {
                props.sturdy_faces = [false; 6];
                props.sturdy_faces[facing as usize] = true;
            }
            _ => {}
        }
        props
    }

    #[inline]
    pub fn is_air(&self) -> bool {
        matches!(self.state, CellState::Empty)
    }

    /// Extracts fluid info (including waterlogged blocks) for `fluid` and `world_hooks`.
    pub fn fluid_info(&self) -> Option<(FlowKind, FluidCell)> {
        match self.state {
            CellState::Fluid { kind, cell } => Some((kind, cell)),
            CellState::TrapDoor { waterlogged: true, .. }
            | CellState::Scaffolding(ScaffoldingState { waterlogged: true, .. }) => Some((FlowKind::Water, FluidCell::WATER_SOURCE)),
            _ => None,
        }
    }

    pub fn is_waterlogged_container(&self) -> bool {
        matches!(self.state, CellState::TrapDoor { .. } | CellState::Scaffolding(_))
    }

    pub fn set_waterlogged(&mut self, wl: bool) -> bool {
        match &mut self.state {
            CellState::TrapDoor { waterlogged, .. } => *waterlogged = wl,
            CellState::Scaffolding(s) => s.waterlogged = wl,
            _ => return false,
        }
        true
    }

    pub fn as_piston_block(&self) -> PistonBlock {
        let props = self.props();
        PistonBlock {
            air: self.is_air(),
            destroy_speed: props.destroy_speed,
            has_block_entity: props.has_block_entity,
            push_reaction: props.push_reaction,
            kind: props.piston_kind,
        }
    }
}
