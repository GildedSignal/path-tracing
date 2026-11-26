//! Factored `< 1 MB` kit projections, admissible `KitCone`, law-probed `VerbEffect`s,
//! double-entry restoration `ObligationLedger`, and backward S96 shadow prices.
//!
//! No tool-specific search code lives here: every tool/item enters as a carried law input
//! projected orthogonally across legs, break, static cell facts, and backward read/write joins.
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};
use law::{
    BlockPos, PlayerState, Vec3, World,
    basis::cell::{BlockId, StateId, models},
    codec, footprint::RecordingWorld,
    mining::{MiningModifiers, MiningSample, player_destroy_speed},
    sim::WorldDelta,
};
use crate::Goal;

pub const STATE_COUNT: usize = 32_366;
pub const BLOCK_COUNT: usize = 1_196;

// `dimension_flags` bits
pub const DIM_WATER_EVAPORATES: u8 = 1 << 0;
pub const DIM_FAST_LAVA: u8 = 1 << 1;

// `verb_mask` bits (presence >= 1 only; stack decrements 64 -> 63 keep mask unchanged)
pub const VERB_INTERACT: u64 = 1 << 0;
pub const VERB_BREAK: u64 = 1 << 1;
pub const VERB_PLACE: u64 = 1 << 2;
pub const VERB_BUCKET_WATER: u64 = 1 << 3;
pub const VERB_BUCKET_POWDER_SNOW: u64 = 1 << 4;
pub const VERB_BOAT: u64 = 1 << 5;
pub const VERB_PEARL: u64 = 1 << 6;
pub const VERB_ELYTRA: u64 = 1 << 7;

// Law-probed `Footprint::writes` effect mask
pub const WRITE_SHAPE_TOGGLE: u32 = 1 << 0;
pub const WRITE_SHAPE_CLEAR: u32 = 1 << 1;
pub const WRITE_SHAPE_SOLID: u32 = 1 << 2;
pub const WRITE_FLUID_WATER: u32 = 1 << 3;
pub const WRITE_FALL_RESET: u32 = 1 << 4;
pub const WRITE_BOAT_SURFACE: u32 = 1 << 5;
pub const WRITE_TELEPORT: u32 = 1 << 6;

// `StateStatic` packed flags
const FLAG_REPLACEABLE: u16 = 1 << 6;
const FLAG_WATERLOGGED: u16 = 1 << 7;
const FLAG_UPPER_HALF: u16 = 1 << 8;
const FLAG_LOWER_HALF: u16 = 1 << 9;
const FLAG_WATER_SOURCE: u16 = 1 << 10;
const FLAG_FLUID: u16 = 1 << 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kit {
    BareLegs,
    Carried(KitSpec),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BodyMovementKey {
    pub boots_depth_strider: u8,
    pub soul_speed: u8,
    pub leather_boots: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ToolProfileKey {
    pub pickaxe_tier: u8,
    pub axe_tier: u8,
    pub shovel_tier: u8,
    pub efficiency: u8,
    pub silk_touch: bool,
    pub haste: u8,
}

impl ToolProfileKey {
    pub fn silk_diamond() -> Self {
        Self { pickaxe_tier: 4, axe_tier: 0, shovel_tier: 0, efficiency: 5, silk_touch: true, haste: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KitSpec {
    pub body: BodyMovementKey,
    pub tool: ToolProfileKey,
    pub blocks: u16,
    pub place_block: StateId,
    pub interact: bool,
    pub can_break: bool,
    pub water_buckets: u8,
    pub powder_snow_buckets: u8,
    pub boats: u8,
    pub pearls: u8,
    pub elytra: bool,
    pub dimension_flags: u8,
    pub restore_required: bool,
    pub restore_blocks: Option<[Option<BlockId>; 9]>,
}

impl KitSpec {
    pub fn empty() -> Self {
        Self {
            body: BodyMovementKey::default(),
            tool: ToolProfileKey::default(),
            blocks: 0,
            place_block: default_stone_state(),
            interact: false,
            can_break: false,
            water_buckets: 0,
            powder_snow_buckets: 0,
            boats: 0,
            pearls: 0,
            elytra: false,
            dimension_flags: 0,
            restore_required: true,
            restore_blocks: None,
        }
    }

    pub fn can_restore_break(&self, block: BlockId, mode: RestoreMode) -> bool {
        !self.restore_required || (mode != RestoreMode::Irreversible && self.restore_blocks.is_none_or(|held| held.contains(&Some(block))))
    }

    pub fn interact_only() -> Self {
        Self { interact: true, ..Self::empty() }
    }

    pub fn counts(&self) -> KitCounts {
        KitCounts {
            blocks: self.blocks,
            water_buckets: self.water_buckets,
            powder_snow_buckets: self.powder_snow_buckets,
            boats: self.boats,
            pearls: self.pearls,
        }
    }

    /// Bitset of carried verb families with `count >= 1`.
    /// Stack count changes (`64 -> 63`) preserve `verb_mask`; only `0 <-> >=1` flips a bit.
    pub fn verb_mask(&self) -> u64 {
        let mut m = 0u64;
        if self.interact { m |= VERB_INTERACT; }
        if self.can_break { m |= VERB_BREAK; }
        if self.blocks > 0 { m |= VERB_PLACE; }
        if self.water_buckets > 0 { m |= VERB_BUCKET_WATER; }
        if self.powder_snow_buckets > 0 { m |= VERB_BUCKET_POWDER_SNOW; }
        if self.boats > 0 { m |= VERB_BOAT; }
        if self.pearls > 0 { m |= VERB_PEARL; }
        if self.elytra { m |= VERB_ELYTRA; }
        m
    }

    pub fn capability_key(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.body.hash(&mut h);
        self.tool.hash(&mut h);
        self.verb_mask().hash(&mut h);
        self.place_block.hash(&mut h);
        self.dimension_flags.hash(&mut h);
        self.restore_required.hash(&mut h);
        self.restore_blocks.hash(&mut h);
        h.finish()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KitCounts {
    pub blocks: u16,
    pub water_buckets: u8,
    pub powder_snow_buckets: u8,
    pub boats: u8,
    pub pearls: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InteractKind {
    None = 0,
    WoodenDoor = 1,
    IronDoor = 2,
    WoodenTrapdoor = 3,
    IronTrapdoor = 4,
    FenceGate = 5,
    Button = 6,
    Lever = 7,
}

impl InteractKind {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::WoodenDoor,
            2 => Self::IronDoor,
            3 => Self::WoodenTrapdoor,
            4 => Self::IronTrapdoor,
            5 => Self::FenceGate,
            6 => Self::Button,
            7 => Self::Lever,
            _ => Self::None,
        }
    }

    pub fn is_hand_toggleable(self) -> bool {
        matches!(self, Self::WoodenDoor | Self::WoodenTrapdoor | Self::FenceGate)
    }
}

/// 2-byte packed kit-independent static state fact (`32,366 * 2 B = 64.7 KB` global).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct StateStatic(pub u16);

impl StateStatic {
    #[inline]
    pub fn sturdy_faces(self) -> u8 { (self.0 & 0x3f) as u8 }
    #[inline]
    pub fn any_sturdy(self) -> bool { (self.0 & 0x3f) != 0 }
    #[inline]
    pub fn is_sturdy(self, face_idx: usize) -> bool { (self.0 & (1u16 << face_idx)) != 0 }
    #[inline]
    pub fn replaceable(self) -> bool { (self.0 & FLAG_REPLACEABLE) != 0 }
    #[inline]
    pub fn waterlogged(self) -> bool { (self.0 & FLAG_WATERLOGGED) != 0 }
    #[inline]
    pub fn upper_half(self) -> bool { (self.0 & FLAG_UPPER_HALF) != 0 }
    #[inline]
    pub fn lower_half(self) -> bool { (self.0 & FLAG_LOWER_HALF) != 0 }
    #[inline]
    pub fn water_source(self) -> bool { (self.0 & FLAG_WATER_SOURCE) != 0 }
    #[inline]
    pub fn has_fluid(self) -> bool { (self.0 & FLAG_FLUID) != 0 }
    #[inline]
    pub fn interact_kind(self) -> InteractKind { InteractKind::from_u8((self.0 >> 12) as u8 & 0x0f) }
}

pub fn default_stone_state() -> StateId {
    static SID: OnceLock<StateId> = OnceLock::new();
    *SID.get_or_init(|| StateId::parse("minecraft:cobblestone", |_| None).expect("cobblestone state"))
}

pub fn air_state() -> StateId {
    static SID: OnceLock<StateId> = OnceLock::new();
    *SID.get_or_init(|| StateId::parse("minecraft:air", |_| None).expect("air state"))
}

pub fn water_source_state() -> StateId {
    static SID: OnceLock<StateId> = OnceLock::new();
    *SID.get_or_init(|| StateId::parse("minecraft:water", |p| (p == "level").then_some("0")).expect("water source state"))
}

pub fn powder_snow_state() -> StateId {
    static SID: OnceLock<StateId> = OnceLock::new();
    *SID.get_or_init(|| StateId::parse("minecraft:powder_snow", |_| None).expect("powder_snow state"))
}

fn shape_sturdy_mask(shape: &law::VoxelShape) -> u8 {
    if shape.is_empty() {
        return 0;
    }
    if shape.is_block() {
        return 0x3f;
    }
    let nx = shape.coords[0].len().saturating_sub(1);
    let ny = shape.coords[1].len().saturating_sub(1);
    let nz = shape.coords[2].len().saturating_sub(1);
    if nx == 0 || ny == 0 || nz == 0 {
        return 0;
    }
    let full_slice = |axis: usize, fixed_idx: usize, expected_coord: f64| -> bool {
        let coord = if fixed_idx == 0 { shape.coords[axis][0] } else { shape.coords[axis][fixed_idx + 1] };
        if (coord - expected_coord).abs() > 1e-6 {
            return false;
        }
        let mut area = 0.0f64;
        match axis {
            1 => {
                for ix in 0..nx {
                    let dx = shape.coords[0][ix + 1] - shape.coords[0][ix];
                    for iz in 0..nz {
                        if shape.is_full([ix, fixed_idx, iz]) {
                            area += dx * (shape.coords[2][iz + 1] - shape.coords[2][iz]);
                        }
                    }
                }
            }
            2 => {
                for ix in 0..nx {
                    let dx = shape.coords[0][ix + 1] - shape.coords[0][ix];
                    for iy in 0..ny {
                        if shape.is_full([ix, iy, fixed_idx]) {
                            area += dx * (shape.coords[1][iy + 1] - shape.coords[1][iy]);
                        }
                    }
                }
            }
            _ => {
                for iy in 0..ny {
                    let dy = shape.coords[1][iy + 1] - shape.coords[1][iy];
                    for iz in 0..nz {
                        if shape.is_full([fixed_idx, iy, iz]) {
                            area += dy * (shape.coords[2][iz + 1] - shape.coords[2][iz]);
                        }
                    }
                }
            }
        }
        (area - 1.0).abs() <= 1e-5
    };
    let mut mask = 0u8;
    if full_slice(1, 0, 0.0) { mask |= 1 << 0; }
    if full_slice(1, ny - 1, 1.0) { mask |= 1 << 1; }
    if full_slice(2, 0, 0.0) { mask |= 1 << 2; }
    if full_slice(2, nz - 1, 1.0) { mask |= 1 << 3; }
    if full_slice(0, 0, 0.0) { mask |= 1 << 4; }
    if full_slice(0, nx - 1, 1.0) { mask |= 1 << 5; }
    mask
}

pub fn state_static_table() -> &'static [StateStatic] {
    static TABLE: OnceLock<Box<[StateStatic]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let m = models();
        let mut out = Vec::with_capacity(STATE_COUNT);
        for raw in 0..STATE_COUNT as u16 {
            let sid = StateId(raw);
            let bname = sid.block().name();
            let mut bits = 0u16;
            if let Some(mid) = sid.model() {
                let blk = &m.blocks[mid.0 as usize];
                bits |= u16::from(shape_sturdy_mask(&blk.shape));
                if blk.air || matches!(bname, "air" | "cave_air" | "void_air" | "water" | "lava" | "short_grass" | "tall_grass" | "fern" | "large_fern" | "dead_bush" | "snow" | "vine") {
                    bits |= FLAG_REPLACEABLE;
                }
                if blk.fluid.kind != law::FluidKind::Empty {
                    bits |= FLAG_FLUID;
                    if blk.fluid.kind == law::FluidKind::Water && sid.get("level") == Some("0") {
                        bits |= FLAG_WATER_SOURCE;
                    }
                }
            } else if matches!(bname, "air" | "cave_air" | "void_air" | "water" | "lava") {
                bits |= FLAG_REPLACEABLE;
            }
            if sid.get("waterlogged") == Some("true") {
                bits |= FLAG_WATERLOGGED | FLAG_FLUID;
            }
            match sid.get("half") {
                Some("upper") => bits |= FLAG_UPPER_HALF,
                Some("lower") => bits |= FLAG_LOWER_HALF,
                _ => {}
            }
            let interact = if bname == "iron_door" {
                InteractKind::IronDoor
            } else if bname.ends_with("_door") {
                InteractKind::WoodenDoor
            } else if bname == "iron_trapdoor" {
                InteractKind::IronTrapdoor
            } else if bname.ends_with("_trapdoor") {
                InteractKind::WoodenTrapdoor
            } else if bname.ends_with("_fence_gate") {
                InteractKind::FenceGate
            } else if bname.ends_with("_button") {
                InteractKind::Button
            } else if bname == "lever" {
                InteractKind::Lever
            } else {
                InteractKind::None
            };
            bits |= ((interact as u16) & 0x0f) << 12;
            out.push(StateStatic(bits));
        }
        out.into_boxed_slice()
    })
}

#[inline]
pub fn state_static(sid: StateId) -> StateStatic {
    state_static_table().get(sid.0 as usize).copied().unwrap_or(StateStatic(0))
}

/// Flips the `open` property of any door/trapdoor/fence_gate preserving every other property.
pub fn toggle_open_state(sid: StateId) -> Option<StateId> {
    let cur = sid.get("open")?;
    let next = if cur == "true" { "false" } else { "true" };
    let full = format!("minecraft:{}", sid.block().name());
    StateId::parse(&full, |p| if p == "open" { Some(next) } else { sid.get(p) }).ok()
}

/// 591 models * 4 roles * 2 B = ~4.7 KB per distinct `BodyMovementKey`.
pub struct LegsProjection {
    pub body_key: BodyMovementKey,
    pub by_model: Box<[[u16; 4]]>,
}

impl LegsProjection {
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.by_model.len() * std::mem::size_of::<[u16; 4]>()
    }
}

fn legs_cache() -> &'static Mutex<HashMap<BodyMovementKey, Arc<LegsProjection>>> {
    static CACHE: OnceLock<Mutex<HashMap<BodyMovementKey, Arc<LegsProjection>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn legs_projection(body_key: BodyMovementKey) -> Arc<LegsProjection> {
    let mut guard = legs_cache().lock().expect("legs cache lock");
    if let Some(hit) = guard.get(&body_key) {
        return hit.clone();
    }
    let m = models();
    let mut rows = Vec::with_capacity(m.blocks.len());
    for blk in &m.blocks {
        let mut h = 0xcbf29ce484222325u64;
        let mix = |h: &mut u64, v: u64| { *h ^= v; *h = h.wrapping_mul(0x100000001b3); };
        mix(&mut h, u64::from(blk.air));
        mix(&mut h, u64::from(blk.friction.to_bits()));
        mix(&mut h, u64::from(blk.speed_factor.to_bits()));
        mix(&mut h, u64::from(blk.jump_factor.to_bits()));
        mix(&mut h, u64::from(blk.climbable));
        mix(&mut h, u64::from(blk.shape.is_empty()));
        mix(&mut h, u64::from(blk.shape.is_block()));
        mix(&mut h, blk.fluid.kind as u64);
        if blk.fluid.kind == law::FluidKind::Water {
            mix(&mut h, u64::from(body_key.boots_depth_strider));
        }
        if blk.inside == law::Inside::PowderSnow {
            mix(&mut h, u64::from(body_key.leather_boots));
        }
        if (blk.speed_factor - 0.4).abs() < 1e-6 && blk.jump_factor == 1.0 {
            mix(&mut h, u64::from(body_key.soul_speed));
        }
        let c0 = (h & 0xffff) as u16;
        let c1 = ((h >> 16) & 0xffff) as u16;
        let c2 = ((h >> 32) & 0xffff) as u16;
        let c3 = ((h >> 48) & 0xffff) as u16;
        rows.push([c0, c1, c2, c3]);
    }
    let proj = Arc::new(LegsProjection { body_key, by_model: rows.into_boxed_slice() });
    guard.insert(body_key, proj.clone());
    proj
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RestoreMode {
    IdenticalDrop = 0,
    WaterloggedLeavesWater = 1,
    PartnerHalf = 2,
    Irreversible = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BreakRow {
    pub break_ticks_ground: u16,
    pub break_ticks_air: u16,
    pub restore_mode: RestoreMode,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PrefTool {
    None,
    Pickaxe(u8),
    Axe,
    Shovel,
    ShearsOrSword,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct BlockBreakStatic {
    hardness_bits: u32,
    requires_correct: bool,
    pref: PrefTool,
    silk_only: bool,
    base_restore: RestoreMode,
}

fn parsed_tsv_rules() -> &'static HashMap<&'static str, (f32, bool, bool)> {
    static TSV: OnceLock<HashMap<&'static str, (f32, bool, bool)>> = OnceLock::new();
    TSV.get_or_init(|| {
        let mut map = HashMap::new();
        for line in include_str!("../../law/src/basis/rules.tsv").lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let f: Vec<_> = line.split('\t').collect();
            if f.len() == 13 {
                let h = f[1].parse::<f32>().unwrap_or(1.5);
                let req = f[2] == "y";
                let nbt = f[11] == "y";
                map.insert(f[0], (h, req, nbt));
            }
        }
        map
    })
}

fn block_break_static(bname: &str) -> BlockBreakStatic {
    let tsv = parsed_tsv_rules();
    let (tsv_h, tsv_req, tsv_nbt) = tsv.get(bname).copied().unwrap_or((1.5, false, false));
    let nbt_irreversible = tsv_nbt
        || matches!(
            bname,
            "chest" | "trapped_chest" | "ender_chest" | "barrel" | "furnace" | "blast_furnace"
                | "smoker" | "hopper" | "dispenser" | "dropper" | "brewing_stand" | "beacon"
                | "spawner" | "trial_spawner" | "vault" | "jukebox" | "lectern" | "crafter"
                | "chiseled_bookshelf" | "beehive" | "bee_nest" | "campfire" | "soul_campfire"
        )
        || bname.ends_with("shulker_box")
        || bname.ends_with("_sign")
        || bname.ends_with("_banner");
    let unbreakable = tsv_h < 0.0
        || matches!(
            bname,
            "bedrock" | "barrier" | "end_portal" | "end_portal_frame" | "end_gateway"
                | "nether_portal" | "command_block" | "repeating_command_block" | "chain_command_block"
                | "structure_block" | "jigsaw" | "light" | "reinforced_deepslate" | "moving_piston"
        );
    if unbreakable {
        return BlockBreakStatic {
            hardness_bits: (-1.0f32).to_bits(),
            requires_correct: true,
            pref: PrefTool::Pickaxe(255),
            silk_only: false,
            base_restore: RestoreMode::Irreversible,
        };
    }
    if matches!(bname, "air" | "cave_air" | "void_air" | "water" | "lava") {
        return BlockBreakStatic {
            hardness_bits: (-1.0f32).to_bits(),
            requires_correct: false,
            pref: PrefTool::None,
            silk_only: false,
            base_restore: RestoreMode::Irreversible,
        };
    }
    let silk_only = bname == "cobweb"
        || matches!(
            bname,
            "stone" | "deepslate" | "grass_block" | "mycelium" | "podzol" | "glowstone"
                | "sea_lantern" | "clay" | "snow_block" | "bookshelf" | "ice" | "packed_ice" | "blue_ice"
        )
        || bname.contains("glass")
        || bname.ends_with("_ore")
        || bname.ends_with("_amethyst_bud")
        || bname == "amethyst_cluster";
    let never_drops_self = bname == "budding_amethyst";

    let base_restore = if nbt_irreversible || never_drops_self {
        RestoreMode::Irreversible
    } else if bname.ends_with("_door") || bname.ends_with("_bed") {
        RestoreMode::PartnerHalf
    } else {
        RestoreMode::IdenticalDrop
    };

    if matches!(bname, "obsidian" | "crying_obsidian" | "respawn_anchor" | "ancient_debris") {
        return BlockBreakStatic { hardness_bits: 50.0f32.to_bits(), requires_correct: true, pref: PrefTool::Pickaxe(4), silk_only, base_restore };
    }
    if bname == "cobweb" {
        return BlockBreakStatic { hardness_bits: 4.0f32.to_bits(), requires_correct: true, pref: PrefTool::ShearsOrSword, silk_only: true, base_restore: RestoreMode::IdenticalDrop };
    }
    if matches!(bname, "dirt" | "grass_block" | "sand" | "red_sand" | "gravel" | "mud" | "clay" | "soul_sand" | "soul_soil" | "podzol" | "mycelium") {
        let h = match bname { "grass_block" | "gravel" | "clay" => 0.6f32, _ => 0.5f32 };
        return BlockBreakStatic { hardness_bits: h.to_bits(), requires_correct: false, pref: PrefTool::Shovel, silk_only, base_restore };
    }
    if matches!(bname, "slime_block" | "honey_block" | "scaffolding") {
        return BlockBreakStatic { hardness_bits: 0.0f32.to_bits(), requires_correct: false, pref: PrefTool::None, silk_only, base_restore };
    }
    if bname.ends_with("_log") || bname.ends_with("_wood") || bname.ends_with("_planks")
        || matches!(bname, "oak_door" | "spruce_door" | "birch_door" | "jungle_door" | "acacia_door" | "dark_oak_door" | "mangrove_door" | "cherry_door" | "bamboo_door" | "crimson_door" | "warped_door" | "pale_oak_door")
        || bname.ends_with("_fence") || bname.ends_with("_fence_gate")
        || (bname.ends_with("_trapdoor") && bname != "iron_trapdoor" && !bname.contains("copper_trapdoor"))
    {
        let h: f32 = if bname.ends_with("_door") || bname.ends_with("_trapdoor") { 3.0 } else { 2.0 };
        return BlockBreakStatic { hardness_bits: h.to_bits(), requires_correct: false, pref: PrefTool::Axe, silk_only, base_restore };
    }
    if tsv.contains_key(bname) {
        return BlockBreakStatic {
            hardness_bits: tsv_h.to_bits(),
            requires_correct: tsv_req,
            pref: if tsv_req { PrefTool::Pickaxe(1) } else { PrefTool::None },
            silk_only,
            base_restore,
        };
    }
    if matches!(bname, "granite" | "diorite" | "andesite" | "cobblestone" | "amethyst_block" | "tuff" | "calcite")
        || bname.ends_with("_coral_block") || bname.ends_with("_bricks") || bname.ends_with("sandstone")
    {
        return BlockBreakStatic { hardness_bits: 1.5f32.to_bits(), requires_correct: true, pref: PrefTool::Pickaxe(1), silk_only, base_restore };
    }
    if bname.ends_with("glass") {
        return BlockBreakStatic { hardness_bits: 0.3f32.to_bits(), requires_correct: false, pref: PrefTool::None, silk_only, base_restore };
    }
    BlockBreakStatic { hardness_bits: 1.5f32.to_bits(), requires_correct: false, pref: PrefTool::Pickaxe(1), silk_only, base_restore }
}

struct BlockBreakClasses {
    by_block: Box<[u16]>,
    classes: Box<[BlockBreakStatic]>,
}

impl BlockBreakClasses {
    fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.by_block.len() * std::mem::size_of::<u16>()
            + self.classes.len() * std::mem::size_of::<BlockBreakStatic>()
    }
}

fn block_break_classes() -> &'static BlockBreakClasses {
    static CLASSES: OnceLock<BlockBreakClasses> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let mut seen = vec![false; BLOCK_COUNT];
        let mut names = vec!["stone"; BLOCK_COUNT];
        for raw in 0..STATE_COUNT as u16 {
            let bid = StateId(raw).block();
            let idx = bid.0 as usize;
            if idx < BLOCK_COUNT && !seen[idx] {
                seen[idx] = true;
                names[idx] = bid.name();
            }
        }
        let mut map: HashMap<BlockBreakStatic, u16> = HashMap::new();
        let mut classes = Vec::new();
        let mut by_block = Vec::with_capacity(BLOCK_COUNT);
        for &bname in &names {
            let key = block_break_static(bname);
            let cid = *map.entry(key).or_insert_with(|| {
                let id = classes.len() as u16;
                classes.push(key);
                id
            });
            by_block.push(cid);
        }
        BlockBreakClasses {
            by_block: by_block.into_boxed_slice(),
            classes: classes.into_boxed_slice(),
        }
    })
}

fn tier_speed(tier: u8) -> f32 {
    match tier {
        0 => 1.0,
        1 => 2.0,
        2 => 4.0,
        3 => 6.0,
        4 => 8.0,
        _ => 9.0,
    }
}

fn break_row_for_static(spec: BlockBreakStatic, tool: ToolProfileKey) -> BreakRow {
    let hardness = f32::from_bits(spec.hardness_bits);
    let restore_mode = if spec.silk_only && !tool.silk_touch {
        RestoreMode::Irreversible
    } else {
        spec.base_restore
    };
    if hardness < 0.0 {
        return BreakRow { break_ticks_ground: u16::MAX, break_ticks_air: u16::MAX, restore_mode };
    }
    if hardness == 0.0 {
        return BreakRow { break_ticks_ground: 1, break_ticks_air: 1, restore_mode };
    }
    let (tool_tier, matched_tool) = match spec.pref {
        PrefTool::None => (0, true),
        PrefTool::Pickaxe(req) => (tool.pickaxe_tier, tool.pickaxe_tier > 0 && tool.pickaxe_tier >= req),
        PrefTool::Axe => (tool.axe_tier, tool.axe_tier > 0),
        PrefTool::Shovel => (tool.shovel_tier, tool.shovel_tier > 0),
        PrefTool::ShearsOrSword => (if tool.silk_touch { 4 } else { 0 }, true),
    };
    let tool_base = if (matched_tool && tool_tier > 0) || (matches!(spec.pref, PrefTool::Pickaxe(_)) && tool.pickaxe_tier > 0) {
        tier_speed(tool_tier)
    } else {
        1.0
    };
    let eff = if tool.efficiency > 0 { f64::from(tool.efficiency * tool.efficiency) + 1.0 } else { 0.0 };
    let haste_amp = if tool.haste > 0 { Some(i32::from(tool.haste - 1)) } else { None };
    let correct = !spec.requires_correct || matched_tool;
    let calc = |on_ground: bool| -> u16 {
        let mods = MiningModifiers {
            tool_base_speed: tool_base,
            mining_efficiency: eff,
            haste_or_conduit_amp: haste_amp,
            on_ground,
            ..Default::default()
        };
        let speed = player_destroy_speed(mods);
        let sample = MiningSample {
            target: law::mining::MiningTarget { position: [0, 0, 0], identity: 1 },
            destroy_speed: speed,
            hardness,
            correct_tool: correct,
        };
        sample.break_ticks().and_then(|t| u16::try_from(t).ok()).unwrap_or(u16::MAX)
    };
    BreakRow {
        break_ticks_ground: calc(true),
        break_ticks_air: calc(false),
        restore_mode,
    }
}

/// Factored equivalence-class break table (`<= 48` classes * 6 B = `< 300 B` per `ToolProfileKey`).
pub struct BreakProjection {
    pub tool_key: ToolProfileKey,
    pub by_class: Box<[BreakRow]>,
}

impl BreakProjection {
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.by_class.len() * std::mem::size_of::<BreakRow>()
    }

    #[inline]
    pub fn get(&self, bid: BlockId, sid: StateId) -> BreakRow {
        let classes = block_break_classes();
        let cid = classes.by_block.get(bid.0 as usize).copied().unwrap_or(0) as usize;
        let mut row = self.by_class.get(cid).copied().unwrap_or(BreakRow {
            break_ticks_ground: u16::MAX,
            break_ticks_air: u16::MAX,
            restore_mode: RestoreMode::Irreversible,
        });
        if state_static(sid).waterlogged() && row.restore_mode == RestoreMode::IdenticalDrop {
            row.restore_mode = RestoreMode::WaterloggedLeavesWater;
        }
        row
    }
}

fn break_cache() -> &'static Mutex<HashMap<ToolProfileKey, Arc<BreakProjection>>> {
    static CACHE: OnceLock<Mutex<HashMap<ToolProfileKey, Arc<BreakProjection>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn break_projection(tool_key: ToolProfileKey) -> Arc<BreakProjection> {
    let mut guard = break_cache().lock().expect("break cache lock");
    if let Some(hit) = guard.get(&tool_key) {
        return hit.clone();
    }
    let classes = block_break_classes();
    let mut rows = Vec::with_capacity(classes.classes.len());
    for &spec in classes.classes.iter() {
        rows.push(break_row_for_static(spec, tool_key));
    }
    let proj = Arc::new(BreakProjection { tool_key, by_class: rows.into_boxed_slice() });
    guard.insert(tool_key, proj.clone());
    proj
}

/// Admissible lower-bound velocity envelope for the active kit (Kill test 16).
#[derive(Clone, Copy, Debug)]
pub struct KitCone {
    pub c_horiz: f64,
    pub c_down: f64,
    pub has_pearl: bool,
}

impl KitCone {
    pub fn for_kit(verb_mask: u64) -> Self {
        let mut c_horiz = crate::goal::C;
        if verb_mask & VERB_BOAT != 0 {
            c_horiz = c_horiz.max(75.0 / 20.0);
        }
        if verb_mask & VERB_ELYTRA != 0 {
            c_horiz = c_horiz.max(100.0 / 20.0);
        }
        Self { c_horiz, c_down: 78.4 / 20.0, has_pearl: (verb_mask & VERB_PEARL) != 0 }
    }

    pub fn lower_bound(&self, from: &PlayerState, to: Goal) -> u32 {
        let h_gap = to.gap(from);
        let q = h_gap / self.c_horiz;
        let mut lb = if q <= 0.0 { 0 } else { f64::from_bits(q.to_bits().saturating_sub(1)).ceil() as u32 };
        let dy = (from.position.y - to.zone().max.y).max(0.0);
        if dy > 0.0 {
            let qd = dy / self.c_down;
            let dl = if qd <= 0.0 { 0 } else { f64::from_bits(qd.to_bits().saturating_sub(1)).ceil() as u32 };
            lb = lb.max(dl);
        }
        if self.has_pearl && from.health > 5.0 {
            let qp = h_gap / 1.5;
            let plb = if qp <= 0.0 { 0 } else { f64::from_bits(qp.to_bits().saturating_sub(1)).ceil() as u32 };
            lb = lb.min(plb);
        }
        lb
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerbKind {
    Interact,
    Break,
    Place,
    BucketWater,
    BucketPowderSnow,
    Boat,
    Pearl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerbEffect {
    pub kind: VerbKind,
    pub writes_mask: u32,
}

fn verb_effects_cache() -> &'static Mutex<HashMap<(u64, u8), Arc<[VerbEffect]>>> {
    static CACHE: OnceLock<Mutex<HashMap<(u64, u8), Arc<[VerbEffect]>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

struct ProbeWorld {
    floor: StateId,
    air: StateId,
    cell: StateId,
}

impl World for ProbeWorld {
    fn block(&self, p: BlockPos) -> &law::Block {
        if p == [0, 1, 0] {
            self.cell.model_block()
        } else if p[1] <= 0 {
            self.floor.model_block()
        } else {
            self.air.model_block()
        }
    }
    fn state_id(&self, p: BlockPos) -> Option<StateId> {
        Some(if p == [0, 1, 0] { self.cell } else if p[1] <= 0 { self.floor } else { self.air })
    }
    fn has_fluid(&self) -> bool { self.cell.model_block().fluid.kind != law::FluidKind::Empty }
    fn has_inside_effects(&self) -> bool { self.cell.model_block().has_inside_effects() }
    fn difficulty(&self) -> u8 { 0 }
}

fn test_fall_reset_probe(target: StateId) -> bool {
    let base = ProbeWorld { floor: default_stone_state(), air: air_state(), cell: air_state() };
    let mut delta = WorldDelta::new(&base);
    delta.set([0, 1, 0], target);
    let rec = RecordingWorld::new(&delta);
    let mut s = PlayerState {
        position: Vec3::new(0.5, 2.0, 0.5),
        velocity: Vec3::new(0.0, -0.8, 0.0),
        fall_distance: 25.0,
        health: 20.0,
        saturation: 20.0,
        ..Default::default()
    };
    let ok = (0..4).all(|_| law::try_tick(&mut s, &rec, &Default::default()).is_ok());
    let fp = rec.take();
    ok && s.health >= 20.0 && s.fall_distance < 1.0 && !fp.reads.is_empty()
}

pub fn probe_verbs(verb_mask: u64, dimension_flags: u8) -> Arc<[VerbEffect]> {
    let mut guard = verb_effects_cache().lock().expect("verb effects cache lock");
    if let Some(hit) = guard.get(&(verb_mask, dimension_flags)) {
        return hit.clone();
    }
    let mut out = Vec::new();
    let base = ProbeWorld { floor: default_stone_state(), air: air_state(), cell: air_state() };
    let mut delta = WorldDelta::new(&base);
    if verb_mask & VERB_INTERACT != 0 {
        if let Ok(closed_door) = StateId::parse("minecraft:oak_door", |p| match p { "open" => Some("false"), "half" => Some("lower"), _ => None }) {
            if let Some(open_door) = toggle_open_state(closed_door) {
                let mark = delta.checkpoint();
                delta.set([0, 1, 0], closed_door);
                let c0 = delta.block([0, 1, 0]).shape.coords.clone();
                delta.set([0, 1, 0], open_door);
                let c1 = delta.block([0, 1, 0]).shape.coords.clone();
                delta.rollback(mark);
                if c0 != c1 {
                    out.push(VerbEffect { kind: VerbKind::Interact, writes_mask: WRITE_SHAPE_TOGGLE });
                }
            }
        }
    }
    if verb_mask & VERB_BREAK != 0 {
        let mark = delta.checkpoint();
        delta.set([0, 1, 0], default_stone_state());
        let solid = !delta.block([0, 1, 0]).never_collides();
        delta.set([0, 1, 0], air_state());
        let empty = delta.block([0, 1, 0]).never_collides();
        delta.rollback(mark);
        if solid && empty {
            out.push(VerbEffect { kind: VerbKind::Break, writes_mask: WRITE_SHAPE_CLEAR });
        }
    }
    if verb_mask & VERB_PLACE != 0 {
        let mark = delta.checkpoint();
        let empty = delta.block([0, 1, 0]).never_collides();
        delta.set([0, 1, 0], default_stone_state());
        let solid = !delta.block([0, 1, 0]).never_collides();
        delta.rollback(mark);
        if empty && solid {
            out.push(VerbEffect { kind: VerbKind::Place, writes_mask: WRITE_SHAPE_SOLID });
        }
    }
    if verb_mask & VERB_BUCKET_WATER != 0 && (dimension_flags & DIM_WATER_EVAPORATES) == 0 {
        if test_fall_reset_probe(water_source_state()) {
            out.push(VerbEffect { kind: VerbKind::BucketWater, writes_mask: WRITE_FLUID_WATER | WRITE_FALL_RESET });
        }
    }
    if verb_mask & VERB_BUCKET_POWDER_SNOW != 0 && test_fall_reset_probe(powder_snow_state()) {
        out.push(VerbEffect { kind: VerbKind::BucketPowderSnow, writes_mask: WRITE_FALL_RESET });
    }
    if verb_mask & VERB_BOAT != 0 {
        out.push(VerbEffect { kind: VerbKind::Boat, writes_mask: WRITE_BOAT_SURFACE | WRITE_FALL_RESET });
    }
    if verb_mask & VERB_PEARL != 0 {
        out.push(VerbEffect { kind: VerbKind::Pearl, writes_mask: WRITE_TELEPORT });
    }
    let arc: Arc<[VerbEffect]> = out.into_boxed_slice().into();
    guard.insert((verb_mask, dimension_flags), arc.clone());
    arc
}

/// Active factored handle per bot (`~1 ns` clone, `0` rebuilds on stack decrement).
#[derive(Clone)]
pub struct KitView {
    pub spec: KitSpec,
    pub legs: Arc<LegsProjection>,
    pub breaking: Arc<BreakProjection>,
    pub verb_mask: u64,
    pub cone: Arc<KitCone>,
    pub verb_effects: Arc<[VerbEffect]>,
}

impl KitView {
    pub fn for_spec(spec: KitSpec) -> Self {
        let verb_mask = spec.verb_mask();
        Self {
            spec,
            legs: legs_projection(spec.body),
            breaking: break_projection(spec.tool),
            verb_mask,
            cone: Arc::new(KitCone::for_kit(verb_mask)),
            verb_effects: probe_verbs(verb_mask, spec.dimension_flags),
        }
    }

    pub fn writes_mask(&self) -> u32 {
        self.verb_effects.iter().fold(0, |acc, v| acc | v.writes_mask)
    }
}

pub fn cache_memory_bytes() -> usize {
    let static_bytes = state_static_table().len() * std::mem::size_of::<StateStatic>();
    let break_classes_bytes = block_break_classes().memory_bytes();
    let legs_bytes: usize = legs_cache().lock().expect("legs").values().map(|p| p.memory_bytes()).sum();
    let break_bytes: usize = break_cache().lock().expect("break").values().map(|p| p.memory_bytes()).sum();
    let verbs_bytes: usize = verb_effects_cache().lock().expect("verbs").values().map(|v| v.len() * std::mem::size_of::<VerbEffect>()).sum();
    static_bytes + break_classes_bytes + legs_bytes + break_bytes + verbs_bytes
}

/// World-editing kit action carried cleanly on `WordStep`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KitOp {
    Interact { pos: BlockPos, after: StateId, partner: Option<(BlockPos, StateId)> },
    Break { pos: BlockPos, after: StateId, break_ticks: u16 },
    Place { pos: BlockPos, after: StateId },
    BucketEmpty { pos: BlockPos, after: StateId },
    BucketPickup { pos: BlockPos, after: StateId },
}

impl KitOp {
    /// The edited cell (a door's partner half moves with it).
    pub fn pos(self) -> BlockPos {
        match self {
            Self::Interact { pos, .. } | Self::Break { pos, .. } | Self::Place { pos, .. }
            | Self::BucketEmpty { pos, .. } | Self::BucketPickup { pos, .. } => pos,
        }
    }

    /// A break holds still while it mines: no movement input on its ticks.
    pub fn holds_still(self) -> bool {
        matches!(self, Self::Break { .. })
    }

    /// The tick of a `ticks`-long step before whose physics the edit lands. A click lands
    /// before the first tick; a break lands before the last, because the client's
    /// `startDestroyBlock` + `continueDestroyBlock` on the first tick adds the first progress
    /// and the `n`-th call destroys the block before that tick's movement.
    pub fn lands_at(self, ticks: u32) -> u32 {
        if self.holds_still() { ticks.saturating_sub(1) } else { 0 }
    }

    pub fn apply<W: World>(self, delta: &mut WorldDelta<'_, W>) {
        match self {
            Self::Interact { pos, after, partner } => {
                delta.set(pos, after);
                if let Some((pp, pa)) = partner {
                    delta.set(pp, pa);
                }
            }
            Self::Break { pos, after, .. } | Self::Place { pos, after }
            | Self::BucketEmpty { pos, after } | Self::BucketPickup { pos, after } => delta.set(pos, after),
        }
    }
}

/// The tool a block is mined with fastest: what the bot holds while it breaks the block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Hand,
    Pickaxe,
    Axe,
    Shovel,
}

pub fn tool_for(sid: StateId) -> ToolKind {
    let classes = block_break_classes();
    let cid = classes.by_block.get(sid.block().0 as usize).copied().unwrap_or(0) as usize;
    match classes.classes.get(cid).map(|c| c.pref) {
        Some(PrefTool::Pickaxe(_)) => ToolKind::Pickaxe,
        Some(PrefTool::Axe) => ToolKind::Axe,
        Some(PrefTool::Shovel) => ToolKind::Shovel,
        _ => ToolKind::Hand,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Obligation {
    pub pos: BlockPos,
    pub original_state: StateId,
    pub opened_tick: u64,
    pub undo_ticks_lb: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObligationLedger {
    pub open: Vec<Obligation>,
}

impl ObligationLedger {
    pub fn record_edit(&mut self, pos: BlockPos, before: StateId, after: StateId, tick: u64, undo_ticks_lb: u16) {
        if let Some(i) = self.open.iter().position(|o| o.pos == pos) {
            if self.open[i].original_state == after {
                self.open.remove(i);
            }
            return;
        }
        if before != after {
            self.open.push(Obligation { pos, original_state: before, opened_tick: tick, undo_ticks_lb });
        }
    }

    pub fn open_debt_lb(&self, from: &PlayerState, to: Goal, cone: &KitCone) -> u32 {
        let mut best = 0u32;
        let mut undo_sum = 0u32;
        let eye = Vec3::new(from.position.x, from.position.y + f64::from(from.pose.eye_height()), from.position.z);
        for o in &self.open {
            let reach_dist = |p: BlockPos| {
                let d = ((f64::from(p[0]) + 0.5 - eye.x).abs() - 4.0).max(0.0);
                let dz = ((f64::from(p[2]) + 0.5 - eye.z).abs() - 4.0).max(0.0);
                (d * d + dz * dz).sqrt()
            };
            let u = u32::from(o.undo_ticks_lb);
            undo_sum = undo_sum.saturating_add(u);
            let to_ob = (reach_dist(o.pos) / cone.c_horiz).floor() as u32;
            let from_ob_goal = (to.gap(&from_pos_state(o.pos, from)) - 5.0).max(0.0);
            let back = (from_ob_goal / cone.c_horiz).floor() as u32;
            best = best.max(to_ob.saturating_add(u).saturating_add(back));
        }
        best.max(undo_sum)
    }

    pub fn content_hash(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for o in &self.open {
            h ^= o.pos[0] as u64; h = h.wrapping_mul(0x100000001b3);
            h ^= o.pos[1] as u64; h = h.wrapping_mul(0x100000001b3);
            h ^= o.pos[2] as u64; h = h.wrapping_mul(0x100000001b3);
            h ^= u64::from(o.original_state.0); h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

fn from_pos_state(p: BlockPos, template: &PlayerState) -> PlayerState {
    let mut s = template.clone();
    s.position = Vec3::new(f64::from(p[0]) + 0.5, f64::from(p[1]) + 1.0, f64::from(p[2]) + 0.5);
    s
}

/// Shadow price of token `r` at joint `J` (S96: "keep it for there").
#[inline]
pub fn shadow_price(t_without: u64, t_with: u64) -> u64 {
    t_without.saturating_sub(t_with)
}

#[inline]
pub fn should_spend_scarce_token(local_saving: u64, downstream_shadow_price: u64, self_closing: bool) -> bool {
    self_closing || local_saving > downstream_shadow_price
}

/// Legal block placement check (`STATES_STATIC` + `law::interact` 4.5 reach + body AABB).
pub fn can_place_at(
    world: &impl World,
    s: &PlayerState,
    pos: BlockPos,
    place_block: StateId,
) -> bool {
    let Some(cur) = world.state_id(pos) else { return false };
    let st = state_static(cur);
    if !st.replaceable() || st.interact_kind() != InteractKind::None {
        return false;
    }
    let eye = law::interact::get_eye_position(s.position, s.pose.eye_height());
    if !law::interact::is_within_block_interaction_range(eye, 4.5, pos, 0.0) {
        return false;
    }
    let placed_box = law::Aabb::new(
        f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2]),
        f64::from(pos[0] + 1), f64::from(pos[1] + 1), f64::from(pos[2] + 1),
    );
    if s.bounding_box().intersects(placed_box) {
        return false;
    }
    let _ = place_block;
    place_faces(world, pos).next().is_some()
}

/// Unit normals of the faces `[D, U, N, S, W, E]` (Minecraft's 3D data values).
pub const FACE_NORMALS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

/// The faces a block placed at `pos` can be clicked against: face `f` of the neighbour at
/// `pos - FACE_NORMALS[f]`. Vanilla places against any face of a block that has a shape and is
/// neither replaceable (a click there replaces it) nor used by the hand (a door or a switch
/// would toggle); the face need not be full or sturdy.
pub fn place_faces(world: &impl World, pos: BlockPos) -> impl Iterator<Item = usize> + '_ {
    (0..6).filter(move |&f| {
        let n = FACE_NORMALS[f];
        let np = [pos[0] - n[0], pos[1] - n[1], pos[2] - n[2]];
        world.state_id(np).is_some_and(|sid| {
            let st = state_static(sid);
            !st.replaceable() && st.interact_kind() == InteractKind::None && !world.block(np).never_collides()
        })
    })
}

/// Checks exact `RecordingWorld` digest check consistency on `kit` hash helper.
pub fn digest_edits(edits: &[(BlockPos, StateId)]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &(p, s) in edits {
        for v in p {
            h ^= v as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= u64::from(s.0);
        h = h.wrapping_mul(0x100000001b3);
    }
    let _ = codec::SCHEMA;
    h
}

// The kit's edits as edges of the field (representation.md sections 4 and 10). The search
// names none of what follows: it asks the field's descent for an edge and this interface for
// the hand action that makes it, so a new tool adds a verb here and no search code.

/// What a field edge asks of the kit where the legs alone cannot enter a cell `n`: `Clear`
/// opens the body's two cells at `n` (a click or a break each), `Floor` puts a block under
/// `n`, `Cushion` resets a fall that lands in `n`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FieldEdit { Clear = 1, Floor = 2, Cushion = 3 }

/// One hand action for the search to run on its delta while the body keeps its own word (a
/// break holds the body still); `restore` when it closes an open obligation.
#[derive(Clone, Copy, Debug)]
pub struct HandStep { pub op: KitOp, pub ticks: u32, pub restore: bool, pub template: Option<crate::word_store::KitTemplate> }

pub(crate) fn cell_box(p: BlockPos) -> law::Aabb {
    law::Aabb::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]), f64::from(p[0] + 1), f64::from(p[1] + 1), f64::from(p[2] + 1))
}

fn in_reach(s: &PlayerState, p: BlockPos) -> bool {
    let eye = law::interact::get_eye_position(s.position, s.pose.eye_height());
    law::interact::is_within_block_interaction_range(eye, 4.5, p, 0.0)
}

/// A door's other half, set to the same openness as the half clicked (or toggled).
fn door_partner(world: &impl World, pos: BlockPos, sid: StateId, open_to_same: Option<bool>) -> Option<(BlockPos, StateId)> {
    let st = state_static(sid);
    let partner_pos = if st.lower_half() {
        [pos[0], pos[1] + 1, pos[2]]
    } else if st.upper_half() {
        [pos[0], pos[1] - 1, pos[2]]
    } else {
        return None;
    };
    let ps = world.state_id(partner_pos)?;
    if ps.block() != sid.block() {
        return None;
    }
    let after = match open_to_same {
        Some(target_open) if (ps.get("open")? == "true") == target_open => ps,
        _ => toggle_open_state(ps)?,
    };
    Some((partner_pos, after))
}

fn update_edits(edits: &mut Vec<(BlockPos, StateId)>, base: &impl World, pos: BlockPos, after: StateId) {
    if base.state_id(pos) == Some(after) {
        if let Some(i) = edits.iter().position(|&(p, _)| p == pos) {
            edits.remove(i);
        }
        return;
    }
    if let Some(slot) = edits.iter_mut().find(|(p, _)| *p == pos) {
        slot.1 = after;
    } else {
        edits.push((pos, after));
        edits.sort_unstable_by_key(|&(p, _)| (p[2], p[1], p[0]));
    }
}

impl KitOp {
    /// A step that ends before its ticks: an emptied bucket once the fall it resets is spent.
    pub fn settled(self, s: &PlayerState) -> bool {
        matches!(self, Self::BucketEmpty { .. })
            && s.fall_distance < 0.5
            && (s.on_ground || s.in_water || s.velocity.y.abs() < 0.25)
    }
}

impl KitView {
    fn has(&self, k: VerbKind) -> bool {
        self.verb_effects.iter().any(|v| v.kind == k)
    }

    /// Whether the kit can make `edit` with what it still carries.
    pub fn makes(&self, edit: FieldEdit, counts: KitCounts) -> bool {
        match edit {
            FieldEdit::Clear => self.has(VerbKind::Interact) || self.has(VerbKind::Break),
            FieldEdit::Floor => self.has(VerbKind::Place) && counts.blocks > 0,
            FieldEdit::Cushion => {
                self.has(VerbKind::BucketWater) && counts.water_buckets > 0 || self.has(VerbKind::BucketPowderSnow) && counts.powder_snow_buckets > 0
            }
        }
    }

    /// The marginal T of a field edge over the legs that carry the body through it (section 4,
    /// `max(T_move, T_hand) - T_move`), for the `cells` the search's geometry names: the cells
    /// blocking the body's way (`Clear`), the cell a floor fills (`Floor`), the landing a fall
    /// reset fills (`Cushion`). A click and its undo ride the body's move inside reach (`0`); a
    /// break holds the body still for its ticks, and so does a placed floor's undo; `None`
    /// where the kit cannot, or the edit is irreversible under restoration.
    pub fn edge(&self, world: &impl World, cells: &[BlockPos], edit: FieldEdit) -> Option<(f32, bool)> {
        match edit {
            FieldEdit::Clear => {
                let mut ticks = 0u32;
                for &c in cells {
                    let sid = world.state_id(c)?;
                    if self.has(VerbKind::Interact) && state_static(sid).interact_kind().is_hand_toggleable() {
                        continue;
                    }
                    let row = self.breaking.get(sid.block(), sid);
                    if !self.has(VerbKind::Break) || row.break_ticks_ground == u16::MAX
                        || !self.spec.can_restore_break(sid.block(), row.restore_mode)
                    {
                        return None;
                    }
                    ticks += u32::from(row.break_ticks_ground.max(1));
                }
                (!cells.is_empty()).then_some((ticks as f32, true))
            }
            FieldEdit::Floor => {
                if !self.has(VerbKind::Place) || self.spec.blocks == 0 {
                    return None;
                }
                if !cells.iter().all(|&c| world.state_id(c).is_some_and(|sid| state_static(sid).replaceable())) {
                    return None;
                }
                let anchored = cells.iter().all(|&c| place_faces(world, c).next().is_some());
                if !self.spec.restore_required {
                    return Some((0.0, anchored));
                }
                let undo = self.breaking.get(self.spec.place_block.block(), self.spec.place_block).break_ticks_ground;
                (undo != u16::MAX).then_some((f32::from(undo.max(1)), anchored))
            }
            FieldEdit::Cushion => (self.has(VerbKind::BucketWater) || self.has(VerbKind::BucketPowderSnow)).then_some((0.0, true)),
        }
    }

    /// The hand action that makes `edit` on the first of `cells` still to edit, from `s` on
    /// `world`, or `None` when it is out of reach, already made (`edits`), or the kit has run out.
    pub fn hand_step(
        &self,
        world: &impl World,
        s: &PlayerState,
        cells: &[BlockPos],
        edit: FieldEdit,
        counts: KitCounts,
        ledger: &ObligationLedger,
        edits: &[(BlockPos, StateId)],
    ) -> Option<HandStep> {
        let fresh = |p: BlockPos| !edits.iter().any(|&(e, _)| e == p) && !ledger.open.iter().any(|o| o.pos == p);
        let step = |op: KitOp, ticks: u32| Some(HandStep { op, ticks, restore: false, template: None });
        let &c = cells.iter().find(|&&c| fresh(c))?;
        if !in_reach(s, c) {
            return None;
        }
        let sid = world.state_id(c)?;
        let st = state_static(sid);
        match edit {
            FieldEdit::Clear => {
                if self.has(VerbKind::Interact) && st.interact_kind().is_hand_toggleable() {
                    let after = toggle_open_state(sid)?;
                    let partner = door_partner(world, c, sid, after.get("open").map(|v| v == "true"));
                    return step(KitOp::Interact { pos: c, after, partner }, 1);
                }
                let row = self.breaking.get(sid.block(), sid);
                let bt = if s.on_ground { row.break_ticks_ground } else { row.break_ticks_air };
                if !self.has(VerbKind::Break) || bt == u16::MAX || !self.spec.can_restore_break(sid.block(), row.restore_mode) {
                    return None;
                }
                let after = if st.waterlogged() { water_source_state() } else { air_state() };
                step(KitOp::Break { pos: c, after, break_ticks: bt.max(1) }, u32::from(bt.max(1)))
            }
            FieldEdit::Floor => {
                if counts.blocks == 0 || !self.has(VerbKind::Place) || !world.block(c).never_collides()
                    || !can_place_at(world, s, c, self.spec.place_block)
                {
                    return None;
                }
                step(KitOp::Place { pos: c, after: self.spec.place_block }, 1)
            }
            FieldEdit::Cushion => {
                if !st.replaceable() {
                    return None;
                }
                let after = if self.has(VerbKind::BucketWater) && counts.water_buckets > 0 {
                    water_source_state()
                } else if self.has(VerbKind::BucketPowderSnow) && counts.powder_snow_buckets > 0 {
                    powder_snow_state()
                } else {
                    return None;
                };
                step(KitOp::BucketEmpty { pos: c, after }, 4)
            }
        }
    }

    /// The open obligations `s` can close now: each restore inside reach once the body is past
    /// it (or at the goal), a bucket picked back up before its water spreads.
    pub fn closures(&self, world: &impl World, base: &impl World, s: &PlayerState, goal: Goal, arrival: u64, ledger: &ObligationLedger) -> Vec<HandStep> {
        let gc = goal.center();
        let s_goal_d2 = (gc.x - s.position.x).powi(2) + (gc.z - s.position.z).powi(2);
        let body = s.bounding_box();
        let mut out = Vec::new();
        for ob in &ledger.open {
            if !in_reach(s, ob.pos) {
                continue;
            }
            let Some(cur) = world.state_id(ob.pos) else { continue };
            let st_orig = state_static(ob.original_state);
            let ob_d2 = (gc.x - (f64::from(ob.pos[0]) + 0.5)).powi(2) + (gc.z - (f64::from(ob.pos[2]) + 0.5)).powi(2);
            let past = goal.touched(s, s) || s_goal_d2 + 0.55 < ob_d2;
            let cbox = cell_box(ob.pos);
            let op = if cur == water_source_state() || cur == powder_snow_state() {
                let age = arrival.saturating_sub(ob.opened_tick);
                let can_pick = (s.on_ground && s.fall_distance < 0.5) || age >= 1;
                (can_pick && (age <= 4 || cur == powder_snow_state())).then_some((KitOp::BucketPickup { pos: ob.pos, after: ob.original_state }, 1))
            } else if st_orig.interact_kind().is_hand_toggleable() {
                (!body.intersects(cbox) && past).then(|| {
                    let pp = if st_orig.lower_half() { [ob.pos[0], ob.pos[1] + 1, ob.pos[2]] } else { [ob.pos[0], ob.pos[1] - 1, ob.pos[2]] };
                    let partner = (st_orig.lower_half() || st_orig.upper_half())
                        .then(|| base.state_id(pp).filter(|ps| ps.block() == ob.original_state.block()).map(|ps| (pp, ps)))
                        .flatten();
                    (KitOp::Interact { pos: ob.pos, after: ob.original_state, partner }, 1)
                })
            } else if ob.original_state == air_state() || st_orig.replaceable() {
                let standing_on = (body.min.y - cbox.max.y).abs() < 0.08
                    && body.max.x > cbox.min.x && body.min.x < cbox.max.x
                    && body.max.z > cbox.min.z && body.min.z < cbox.max.z;
                let row = self.breaking.get(cur.block(), cur);
                let bt = if s.on_ground { row.break_ticks_ground } else { row.break_ticks_air };
                (!standing_on && !body.intersects(cbox) && past && bt < u16::MAX && state_static(cur).interact_kind() == InteractKind::None)
                    .then_some((KitOp::Break { pos: ob.pos, after: ob.original_state, break_ticks: bt.max(1) }, u32::from(bt.max(1))))
            } else {
                (!body.intersects(cbox) && past && can_place_at(world, s, ob.pos, ob.original_state))
                    .then_some((KitOp::Place { pos: ob.pos, after: ob.original_state }, 1))
            };
            if let Some((op, ticks)) = op {
                out.push(HandStep { op, ticks, restore: true, template: None });
            }
        }
        out
    }

    /// If `aim` is a non-restore click targeting a blocking cell, return that cell so a moving
    /// word in Phase A stops upon adjacent contact identically to Phase B's field trigger.
    pub fn click_block_target(
        &self,
        world: &impl World,
        s: &PlayerState,
        goal: Goal,
        aim: crate::word_store::AimRule,
    ) -> Option<BlockPos> {
        let crate::word_store::AimRule::Click(t) = aim else { return None };
        if t.restore {
            return None;
        }
        let pos = t.cell(s, goal);
        (!world.block(pos).never_collides()).then_some(pos)
    }

    /// Instantiate `aim` if it is a learned click template (`None` otherwise or if refused).
    pub fn instantiate_step(
        &self,
        aim: crate::word_store::AimRule,
        world: &impl World,
        s: &PlayerState,
        goal: Goal,
        ledger: &ObligationLedger,
        counts: KitCounts,
    ) -> Option<HandStep> {
        let crate::word_store::AimRule::Click(t) = aim else { return None };
        let mut h = self.instantiate(t, world, s, goal, ledger, counts)?;
        h.template = Some(t);
        Some(h)
    }

    /// A learned click template made concrete from `s`: its cell from the approach frame, its
    /// after-state from the verb or the open obligation there; `None` where the live cell refuses it.
    pub fn instantiate(
        &self,
        t: crate::word_store::KitTemplate,
        world: &impl World,
        s: &PlayerState,
        goal: Goal,
        ledger: &ObligationLedger,
        counts: KitCounts,
    ) -> Option<HandStep> {
        use crate::word_store::KitVerb;
        let pos = t.cell(s, goal);
        let cur = world.state_id(pos)?;
        let ob = ledger.open.iter().find(|o| o.pos == pos);
        if t.restore != ob.is_some() {
            return None;
        }
        let solid = !world.block(pos).never_collides();
        let (op, ticks) = match t.verb {
            KitVerb::Interact => {
                if !self.has(VerbKind::Interact) || !state_static(cur).interact_kind().is_hand_toggleable() {
                    return None;
                }
                let after = toggle_open_state(cur)?;
                (KitOp::Interact { pos, after, partner: door_partner(world, pos, cur, after.get("open").map(|v| v == "true")) }, 1)
            }
            KitVerb::Break => {
                let row = self.breaking.get(cur.block(), cur);
                let bt = if s.on_ground { row.break_ticks_ground } else { row.break_ticks_air };
                let fresh_ok = self.has(VerbKind::Break) && self.spec.can_restore_break(cur.block(), row.restore_mode);
                if !solid || bt == u16::MAX || ob.is_none() && !fresh_ok {
                    return None;
                }
                let after = ob.map_or_else(|| if state_static(cur).waterlogged() { water_source_state() } else { air_state() }, |o| o.original_state);
                (KitOp::Break { pos, after, break_ticks: bt.max(1) }, u32::from(bt.max(1)))
            }
            KitVerb::Place => {
                let after = ob.map_or(self.spec.place_block, |o| o.original_state);
                if solid || ob.is_none() && (!self.has(VerbKind::Place) || counts.blocks == 0) || !can_place_at(world, s, pos, after) {
                    return None;
                }
                (KitOp::Place { pos, after }, 1)
            }
            KitVerb::BucketWater | KitVerb::BucketPowderSnow => {
                let (kind, n, after) = if t.verb == KitVerb::BucketWater {
                    (VerbKind::BucketWater, counts.water_buckets, water_source_state())
                } else {
                    (VerbKind::BucketPowderSnow, counts.powder_snow_buckets, powder_snow_state())
                };
                if !self.has(kind) || n == 0 || !state_static(cur).replaceable() {
                    return None;
                }
                (KitOp::BucketEmpty { pos, after }, 4)
            }
            KitVerb::BucketPickup => {
                if cur != water_source_state() && cur != powder_snow_state() {
                    return None;
                }
                (KitOp::BucketPickup { pos, after: ob?.original_state }, 1)
            }
        };
        Some(HandStep { op, ticks, restore: t.restore, template: Some(t) })
    }

    /// What one hand action leaves behind: the edit journal, the restore ledger and the counts.
    /// A restore (an edit at an open obligation's cell) gives back what its first edit took.
    pub fn book(
        &self,
        base: &impl World,
        before: StateId,
        op: KitOp,
        tick: u64,
        edits: &mut Vec<(BlockPos, StateId)>,
        ledger: &mut ObligationLedger,
        counts: &mut KitCounts,
    ) {
        let pos = op.pos();
        let restore = ledger.open.iter().any(|o| o.pos == pos);
        let (after, undo) = match op {
            KitOp::Interact { after, partner, .. } => {
                if let Some((pp, pa)) = partner {
                    update_edits(edits, base, pp, pa);
                }
                (after, 1)
            }
            KitOp::Break { after, .. } => {
                if restore { counts.blocks = counts.blocks.saturating_add(1); }
                (after, 1)
            }
            KitOp::Place { after, .. } => {
                if !restore { counts.blocks = counts.blocks.saturating_sub(1); }
                (after, self.breaking.get(after.block(), after).break_ticks_ground.max(1))
            }
            KitOp::BucketEmpty { after, .. } => {
                if after == water_source_state() { counts.water_buckets = counts.water_buckets.saturating_sub(1); }
                else { counts.powder_snow_buckets = counts.powder_snow_buckets.saturating_sub(1); }
                (after, 1)
            }
            KitOp::BucketPickup { after, .. } => {
                if before == water_source_state() { counts.water_buckets = counts.water_buckets.saturating_add(1); }
                else { counts.powder_snow_buckets = counts.powder_snow_buckets.saturating_add(1); }
                (after, 1)
            }
        };
        update_edits(edits, base, pos, after);
        if self.spec.restore_required || matches!(op, KitOp::BucketEmpty { .. } | KitOp::BucketPickup { .. }) {
            ledger.record_edit(pos, before, after, tick, undo);
        }
    }

    /// The kit's event exits for a moving word: a mid-air tick where a floor can go under or
    /// ahead of the body, and the tick before a fall a cushion could reset lands.
    pub fn window(&self, world: &impl World, s: &PlayerState, tick: u32, counts: KitCounts) -> bool {
        let floor = self.writes_mask() & WRITE_SHAPE_SOLID != 0 && counts.blocks > 0;
        let cushion = self.writes_mask() & WRITE_FALL_RESET != 0 && (counts.water_buckets > 0 || counts.powder_snow_buckets > 0);
        if floor && !s.on_ground && tick >= 2 && s.velocity.y < 0.05 {
            let fy = (s.position.y - 0.99).floor() as i32;
            let fx = s.position.x.floor() as i32;
            let fz = s.position.z.floor() as i32;
            let sx = if s.velocity.x > 0.05 { 1 } else if s.velocity.x < -0.05 { -1 } else { 0 };
            let sz = if s.velocity.z > 0.05 { 1 } else if s.velocity.z < -0.05 { -1 } else { 0 };
            for cell in [[fx, fy, fz], [fx + sx, fy, fz + sz], [fx + sx, fy - 1, fz + sz], [fx, fy - 1, fz]] {
                if world.block(cell).never_collides() && can_place_at(world, s, cell, self.spec.place_block) {
                    return true;
                }
            }
        }
        if cushion && !s.on_ground && s.fall_distance > 2.2 && s.velocity.y < -0.15 {
            let fx = s.position.x.floor() as i32;
            let fz = s.position.z.floor() as i32;
            let next_y = (s.position.y + s.velocity.y).floor() as i32;
            let cur_y = s.position.y.floor() as i32;
            return ((next_y - 1)..=cur_y).any(|y| !world.block([fx, y, fz]).never_collides());
        }
        false
    }
}
