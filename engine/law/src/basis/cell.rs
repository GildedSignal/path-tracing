//! `cell`: a block is an id into the world crate's generated tables, and its physics is that
//! table's row. `blocks.tsv` names every 26.2 block (its row is the block's id) with the model of
//! each state its relevant properties select (`maps.tsv`); `models.tsv` holds one row per distinct
//! model: boxes, friction, speed and jump factors, the inside and step hooks; `census.tsv` names
//! the blocks refused. `rules.tsv` (beside this file) holds what a rule reads of a block and the
//! model does not: hardness, push reaction, conductor, signal source, survival.
use crate::block_hooks::{OffsetCollisionSpec, SpeleothemThickness, TipDirection, offset_collision_shape_for_keys};
use crate::piston::{BlockKind as PistonKind, PushReaction};
use crate::{Aabb, Block, ContactDamage, Fluid, FluidFlowBlock, FluidKind, GroundedContact, Inside, VoxelShape};
use std::{collections::BTreeMap, sync::OnceLock};

/// A block's row in `census.tsv` (the registry order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u32);

/// How far a model's collision reaches, in any context.
pub const NOWHERE: u8 = 0;
pub const OWN_CELL: u8 = 1;
pub const BEYOND: u8 = 2;

/// A shared generator: the properties that change a model, and their values' slots.
struct Map { properties: Vec<&'static str>, slots: BTreeMap<&'static str, usize> }
pub struct Models {
    maps: Vec<Map>,
    names: Vec<&'static str>,
    rows: BTreeMap<&'static str, u32>,
    tables: BTreeMap<&'static str, (usize, Vec<u16>)>,
    pub blocks: Vec<Block>,
    pub offset_variants: Vec<Option<Box<[Block]>>>,
    pub reach: Vec<u8>,
    refusals: BTreeMap<&'static str, &'static str>,
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
            let slots = entries.split(' ').map(|e| {
                let (key, slot) = e.rsplit_once(':').expect("generated map entry");
                (key, slot.parse().expect("generated slot"))
            }).collect();
            Map { properties: properties.split(',').filter(|p| !p.is_empty()).collect(), slots }
        }).collect::<Vec<_>>();
        // blocks.tsv: block, its map, then the model id of each slot.
        let tables = include_str!("../../../world/data/blocks.tsv").lines().map(|line| {
            let [name, map, ids] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated block row") };
            let map: usize = map.parse().expect("generated map index");
            let ids = ids.split(',').map(|id| id.parse().expect("generated model id")).collect::<Vec<u16>>();
            assert!(map < maps.len() && ids.iter().all(|&id| (id as usize) < blocks.len()));
            assert!(maps[map].slots.values().all(|&s| s < ids.len()), "generated slots of {name}");
            (name, (map, ids))
        }).collect::<BTreeMap<_, _>>();
        // census.tsv: block, bucket, state count, in registry order (a block's id is its row).
        // Every bucket other than "modeled" names a refusal.
        let (mut refusals, mut names) = (BTreeMap::new(), Vec::new());
        for line in include_str!("../../../world/data/census.tsv").lines().filter(|l| !l.starts_with('#')) {
            let [block, bucket, _] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("generated census row") };
            names.push(block);
            if bucket != "modeled" { assert!(refusals.insert(block, bucket).is_none(), "one refusal reason per block: {block}"); }
        }
        let rows: BTreeMap<_, _> = names.iter().enumerate().map(|(i, &n)| (n, i as u32)).collect();
        let mut rules = vec![None; names.len()];
        for line in include_str!("rules.tsv").lines().filter(|l| !l.starts_with('#')) {
            let (name, row) = parse_rules(line);
            let at = *rows.get(format!("minecraft:{name}").as_str()).unwrap_or_else(|| panic!("rules.tsv: {name} is not a 26.2 block"));
            rules[at as usize] = Some(row);
        }
        Models { maps, names, rows, tables, blocks, offset_variants, reach, refusals, rules }
    })
}

impl BlockId {
    /// The block named `name` (`minecraft:` may be left out).
    pub fn of(name: &str) -> Self {
        let full = if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") };
        Self(*models().rows.get(full.as_str()).unwrap_or_else(|| panic!("{full} is not a 26.2 block")))
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

/// Project a state onto its block's relevant properties, then look the values up.
pub fn lookup<'a>(name: &str, properties: impl Fn(&str) -> Option<&'a str>) -> Result<u16, String> {
    let registry = models();
    if let Some(reason) = registry.refusals.get(name) { return Err((*reason).into()); }
    let Some((map, ids)) = registry.tables.get(name) else { return Err(format!("{name} is not a 26.2 registry block")); };
    let map = &registry.maps[*map];
    map.properties.iter().map(|p| properties(p)).collect::<Option<Vec<_>>>()
        .and_then(|values| map.slots.get(values.join(",").as_str()).map(|&slot| ids[slot]))
        .ok_or_else(|| format!("state unknown to the 26.2 table of {name}"))
}

/// The model of `block` with `properties` (name, value): the table's row, shared.
pub fn model(block: BlockId, properties: &[(&str, &str)]) -> &'static Block {
    let name = models().names[block.0 as usize];
    let id = lookup(name, |p| properties.iter().find(|(k, _)| *k == p).map(|(_, v)| *v))
        .unwrap_or_else(|e| panic!("{name} {properties:?}: {e}"));
    &models().blocks[id as usize]
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
