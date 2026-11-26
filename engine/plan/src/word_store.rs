//! Recursive coordinate-free Word generators, Zipf emergence, and certified route cache.
//! Exploration never certifies; only the single winning route is recorded and guarded.
use std::collections::{HashMap, HashSet};
use law::{
    Block, BlockPos, CollisionContext, FluidKind, Inside, PlayerState, VoxelShape, World,
    codec,
};
use crate::{Goal, KitOp, inline::List, kit::water_source_state, lattice, solve::WordStep, state_view::StateAt};

/// Peaceful-only resource guard with an exact-bits fallback when survival
/// depends on `food_tick_timer`-gated regeneration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    pub class: [u64; 4],
    pub health: F32Bits,
    pub fall_distance: u64,
    pub exact: [u32; 5],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F32Bits(pub u32);

impl Guard {
    pub fn of(s: &PlayerState) -> Self {
        Self {
            class: codec::resource_guards(s.food_level, s.health),
            health: F32Bits(s.health.to_bits()),
            fall_distance: s.fall_distance.to_bits(),
            exact: [
                s.health.to_bits(),
                s.food_level as u32,
                s.saturation.to_bits(),
                s.exhaustion.to_bits(),
                s.food_tick_timer,
            ],
        }
    }

    pub fn admits(self, entry: Self, damage_no_regen: f64) -> bool {
        let entry_health = f32::from_bits(entry.health.0);
        self.class == entry.class
            && self.fall_distance == entry.fall_distance
            && entry_health > 0.0
            && (f64::from(entry_health) > damage_no_regen || self.exact == entry.exact)
    }
}

/// Local `World` wrapper disabling `natural_regeneration` to measure unmitigated
/// damage on a health-changing piece without altering `engine/law`.
pub struct NoRegen<'a, W>(pub &'a W);

impl<W: World> World for NoRegen<'_, W> {
    fn natural_regeneration(&self) -> bool { false }
    fn block(&self, p: BlockPos) -> &Block { self.0.block(p) }
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> { self.0.known_block(p) }
    fn hook_block(&self, p: BlockPos) -> &Block { self.0.hook_block(p) }
    fn state_id(&self, p: BlockPos) -> Option<law::basis::cell::StateId> { self.0.state_id(p) }
    fn uniform(&self, lo: BlockPos, hi: BlockPos) -> Option<&Block> { self.0.uniform(lo, hi) }
    fn fluid_block(&self, p: BlockPos) -> &Block { self.0.fluid_block(p) }
    fn raw_fluid_block(&self, p: BlockPos) -> Option<&Block> { self.0.raw_fluid_block(p) }
    fn loaded(&self, p: BlockPos) -> bool { self.0.loaded(p) }
    fn min_y(&self) -> i32 { self.0.min_y() }
    fn fast_lava(&self) -> bool { self.0.fast_lava() }
    fn difficulty(&self) -> u8 { self.0.difficulty() }
    fn has_fluid(&self) -> bool { self.0.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.0.has_inside_effects() }
    fn has_fluid_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.0.has_fluid_in(lo, hi) }
    fn has_inside_effects_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.0.has_inside_effects_in(lo, hi) }
    fn collision_cell_bounds(&self, b: law::Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        self.0.collision_cell_bounds(b, lo, hi)
    }
    fn visit_collision_cells(&self, b: law::Aabb, lo: BlockPos, hi: BlockPos, visit: impl FnMut(BlockPos) -> bool) -> bool {
        self.0.visit_collision_cells(b, lo, hi, visit)
    }
    fn collision_shape(&self, p: BlockPos, c: CollisionContext) -> &VoxelShape { self.0.collision_shape(p, c) }
    fn extra_collisions(&self, b: law::Aabb, c: CollisionContext) -> Vec<VoxelShape> { self.0.extra_collisions(b, c) }
    fn visit_extra_collisions(&self, b: law::Aabb, c: CollisionContext, v: impl FnMut(law::shapes::ShapeView<'_>) -> bool) -> bool {
        self.0.visit_extra_collisions(b, c, v)
    }
    fn observe_tick(&self) { self.0.observe_tick(); }
}

#[derive(Clone, Debug)]
pub struct Route {
    pub words: List<WordStep, 12>,
    pub ticks: u64,
    pub end: StateAt,
}

pub(crate) fn compose_chain(from: &PlayerState, _goal: Goal, words: List<WordStep, 12>, work: &mut Work) -> Route {
    work.waves += 1;
    work.edge_reads += words.len() as u64;
    work.settled += words.len() as u64 + 1;
    let ticks = words.iter().rev().fold(0u64, |tail, w| tail.checked_add(w.ticks as u64).expect("tick cost overflow"));
    let end = words.last().map(|w| w.end.clone()).unwrap_or_else(|| StateAt::point(from.clone()));
    Route { ticks, words, end }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Work {
    pub waves: u64,
    pub edge_reads: u64,
    pub settled: u64,
    pub law_ticks: u64,
    pub proposals_tried: u64,
    pub proposal_hits: u64,
    pub learn_ticks: u64,
    pub certify_ticks: u64,
    pub words_total: u64,
    pub compounds_total: u64,
    pub acting_law_ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordId(pub usize);

/// Goal-relative or cardinal-frame aim rule — never a world-fixed angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AimRule {
    /// Offset in 65536-bin sine units from `lattice::bracket(goal_dx, goal_dz)[0]`.
    GoalOffset(i16),
    /// Cardinal multiple of 90° relative to approach frame `rot` (`0=fwd, 1=right, 2=back, 3=left`).
    RelativeCardinal(u8),
    /// A carried-kit click or hold at a frame-relative cell, moving along a goal-offset yaw.
    Click(KitTemplate),
}

/// The verb of a kit word: what the click does, never which block or where in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KitVerb {
    Interact,
    Break,
    Place,
    BucketWater,
    BucketPowderSnow,
    BucketPickup,
}

/// A coordinate-free, rotation-invariant kit action: its cell is `(fwd, up, right)` from the
/// body's cell in `approach_frame`, its after-state comes from the verb (or, for a `restore`,
/// from the open obligation at that cell), and the body moves along `GoalOffset(yaw)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KitTemplate {
    pub verb: KitVerb,
    pub fwd: i8,
    pub up: i8,
    pub right: i8,
    pub restore: bool,
    pub yaw: i16,
}

impl KitTemplate {
    /// The template of `op` clicked from `s`, moving along `yaw`.
    pub fn of(op: KitOp, s: &PlayerState, goal: Goal, yaw: u16, restore: bool) -> Option<Self> {
        let (_, fwd, right) = approach_frame(s, goal);
        let p = op.pos();
        let d = [p[0] - s.position.x.floor() as i32, p[1] - s.position.y.floor() as i32, p[2] - s.position.z.floor() as i32];
        let verb = match op {
            KitOp::Interact { .. } => KitVerb::Interact,
            KitOp::Break { .. } => KitVerb::Break,
            KitOp::Place { .. } => KitVerb::Place,
            KitOp::BucketEmpty { after, .. } if after == water_source_state() => KitVerb::BucketWater,
            KitOp::BucketEmpty { .. } => KitVerb::BucketPowderSnow,
            KitOp::BucketPickup { .. } => KitVerb::BucketPickup,
        };
        let base = AimRule::GoalOffset(0).resolve(s, goal, 0);
        Some(Self {
            verb,
            fwd: i8::try_from(d[0] * fwd[0] + d[2] * fwd[1]).ok()?,
            up: i8::try_from(d[1]).ok()?,
            right: i8::try_from(d[0] * right[0] + d[2] * right[1]).ok()?,
            restore,
            yaw: yaw.wrapping_sub(base) as i16,
        })
    }

    /// The world cell this template clicks from `s`.
    pub fn cell(self, s: &PlayerState, goal: Goal) -> BlockPos {
        let (_, fwd, right) = approach_frame(s, goal);
        let (f, r) = (i32::from(self.fwd), i32::from(self.right));
        [
            s.position.x.floor() as i32 + f * fwd[0] + r * right[0],
            s.position.y.floor() as i32 + i32::from(self.up),
            s.position.z.floor() as i32 + f * fwd[1] + r * right[1],
        ]
    }
}

impl AimRule {
    pub fn resolve(self, s: &PlayerState, goal: Goal, rot: u8) -> u16 {
        match self {
            Self::GoalOffset(delta) => {
                let d = goal.center().minus(s.position);
                let base = lattice::bracket(d.x, d.z)[0];
                base.wrapping_add(delta as u16)
            }
            Self::RelativeCardinal(k) => u16::from((rot.wrapping_add(k)) & 3) * 16384,
            Self::Click(t) => Self::GoalOffset(t.yaw).resolve(s, goal, rot),
        }
    }

    pub fn from_yaw(yaw: u16, goal_base: u16, rot: u8) -> Self {
        if yaw == goal_base {
            return Self::GoalOffset(0);
        }
        if yaw.is_multiple_of(16384) {
            return Self::RelativeCardinal(((yaw / 16384) as u8).wrapping_sub(rot) & 3);
        }
        Self::GoalOffset(yaw.wrapping_sub(goal_base) as i16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntryClass {
    pub on_ground: bool,
    pub pose: u8,
    pub speed_band: u8,
    pub vy_sign: i8,
    pub sprint_ok: bool,
    pub on_climbable: bool,
    pub in_fluid: bool,
}

impl EntryClass {
    pub fn of(s: &PlayerState) -> Self {
        let h2 = s.velocity.horizontal_sqr();
        let speed_band = if h2 < 0.0025 {
            0
        } else if h2 < 0.0324 {
            1
        } else if h2 < 0.1024 {
            2
        } else {
            3
        };
        let vy_sign = if s.velocity.y > 0.05 {
            1
        } else if s.velocity.y < -0.05 {
            -1
        } else {
            0
        };
        Self {
            on_ground: s.on_ground,
            pose: s.pose as u8,
            speed_band,
            vy_sign,
            sprint_ok: codec::resource_guards(s.food_level, s.health)[3] != 0,
            on_climbable: s.on_climbable,
            in_fluid: s.in_water || s.in_lava,
        }
    }

    /// Two classes compose along a cycle when discrete regime matches (speed can warm or stabilize).
    pub fn compatible(self, other: Self) -> bool {
        self.on_ground == other.on_ground
            && self.pose == other.pose
            && self.sprint_ok == other.sprint_ok
            && self.on_climbable == other.on_climbable
            && self.in_fluid == other.in_fluid
            && (self.speed_band == other.speed_band || (self.speed_band >= 1 && other.speed_band >= 1))
    }
}

/// Cardinal approach orientation `rot in 0..4` aligned with Minecraft yaw quadrants:
/// `0 = +Z (yaw 0)`, `1 = -X (yaw 90)`, `2 = -Z (yaw 180)`, `3 = +X (yaw 270)`.
pub fn approach_frame(s: &PlayerState, goal: Goal) -> (u8, [i32; 2], [i32; 2]) {
    let d = goal.center().minus(s.position);
    let (dx, dz) = if d.horizontal_sqr() > 1e-6 {
        (d.x, d.z)
    } else {
        (s.velocity.x, s.velocity.z)
    };
    let (rot, fwd, right) = if dz.abs() >= dx.abs() {
        if dz >= 0.0 {
            (0, [0, 1], [-1, 0])
        } else {
            (2, [0, -1], [1, 0])
        }
    } else if dx < 0.0 {
        (1, [-1, 0], [0, -1])
    } else {
        (3, [1, 0], [0, 1])
    };
    (rot, fwd, right)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DefectSig {
    pub fine: u64,
    pub coarse: u64,
    pub rot: u8,
}

#[inline]
fn fnv(mut h: u64, val: u64) -> u64 {
    h ^= val;
    h.wrapping_mul(0x100000001b3)
}

fn hash_rotated_coords(mut h: u64, coords: &[f64], mirror: bool) -> u64 {
    h = fnv(h, coords.len() as u64);
    if mirror {
        for &c in coords.iter().rev() {
            h = fnv(h, (1.0 - c).to_bits());
        }
    } else {
        for &c in coords {
            h = fnv(h, c.to_bits());
        }
    }
    h
}

/// Zero-allocation, block-name-free physics presence tag matching `codec::legs_model`
/// rotated into the local approach frame so flowers/buttons/plates match air and
/// identical terrain rotated 90 degrees matches identically.
fn cell_legs_tag(loaded: bool, b: &Block, rot: u8) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    h = fnv(h, u64::from(loaded));
    if !loaded {
        return h;
    }
    let has_inside = b.inside != Inside::None || b.stuck_multiplier.is_some();
    let effective_air = has_inside && b.air;
    h = fnv(h, u64::from(effective_air));
    h = fnv(h, b.shape.is_empty() as u64 | ((b.shape.is_block() as u64) << 1));
    if !b.shape.is_empty() && !b.shape.is_block() {
        let (ax_f, mir_f, ax_r, mir_r) = match rot & 3 {
            0 => (2, false, 0, true),
            1 => (0, true, 2, true),
            2 => (2, true, 0, false),
            _ => (0, false, 2, false),
        };
        h = hash_rotated_coords(h, &b.shape.coords[ax_f], mir_f);
        h = hash_rotated_coords(h, &b.shape.coords[1], false);
        h = hash_rotated_coords(h, &b.shape.coords[ax_r], mir_r);
    }
    h = fnv(h, b.above_shape.is_some() as u64 | ((b.below_shape.is_some() as u64) << 1)
        | ((b.sneak_shape.is_some() as u64) << 2) | ((b.inside_cell_shape.is_some() as u64) << 3));
    h = fnv(h, b.friction.to_bits() as u64);
    h = fnv(h, b.speed_factor.to_bits() as u64);
    h = fnv(h, b.jump_factor.to_bits() as u64);
    let inside_tag = match b.inside {
        Inside::None => 0u64,
        Inside::Cobweb => 1,
        Inside::PowderSnow => 2,
        Inside::Honey => 3,
        Inside::Unsupported(_) => 4,
        Inside::BubbleColumn { drag } => 5 + u64::from(drag),
        Inside::Fire { soul } => 7 + u64::from(soul),
        Inside::Campfire { soul } => 9 + u64::from(soul),
        Inside::BerryBush { hurt } => 11 + u64::from(hurt),
        Inside::Cactus => 13,
        Inside::Extinguish => 14,
        Inside::Lava => 15,
        Inside::EndPortal => 16,
        Inside::NetherPortal => 17,
        Inside::WitherRose => 18,
    };
    let flags = (b.climbable as u64)
        | ((b.scaffolding as u64) << 1)
        | ((b.bed as u64) << 2)
        | ((b.bounce != 0.0) as u64) << 3
        | ((b.slime as u64) << 4)
        | ((b.fence as u64) << 5)
        | ((b.wall_or_gate as u64) << 6)
        | ((b.resets_fall_distance as u64) << 7)
        | ((b.end_portal_or_gateway as u64) << 8)
        | ((b.fluid.kind as u64) << 10)
        | (inside_tag << 13);
    h = fnv(h, flags);
    if b.fluid.kind != FluidKind::Empty {
        h = fnv(h, b.fluid.height.to_bits() as u64);
    }
    if let Some(m) = b.stuck_multiplier {
        h = fnv(h, m.x.to_bits());
        h = fnv(h, m.y.to_bits());
    }
    h
}

pub fn defect_sig(world: &impl World, s: &PlayerState, goal: Goal) -> DefectSig {
    let (rot, fwd, right) = approach_frame(s, goal);
    let ox = s.position.x.floor() as i32;
    let oy = s.position.y.floor() as i32;
    let oz = s.position.z.floor() as i32;
    let mut coarse = 0xcbf29ce484222325u64;
    let mut fine = 0xcbf29ce484222325u64;
    coarse = fnv(coarse, u64::from(s.main_supporting_block.is_some()));
    fine = fnv(fine, u64::from(s.main_supporting_block.is_some()));

    for df in -1i32..=4 {
        for dr in -1i32..=1 {
            for dy in -1i32..=2 {
                let p = [
                    ox + df * fwd[0] + dr * right[0],
                    oy + dy,
                    oz + df * fwd[1] + dr * right[1],
                ];
                let tag = match world.known_block(p) {
                    Ok(b) => cell_legs_tag(world.loaded(p), b, rot),
                    Err(_) => 0xdead_beef_cafe_0001,
                };
                fine = fnv(fine, tag);
                if (0..=3).contains(&df) && dr == 0 && (-1..=1).contains(&dy) {
                    coarse = fnv(coarse, tag);
                }
            }
        }
    }
    DefectSig { fine, coarse, rot }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WordBody {
    Atomic { program: u16, aim: AimRule, max_ticks: u16 },
    /// Recursive compound word of arbitrary length and arbitrary nesting depth.
    Compound(Vec<WordId>),
}

#[derive(Clone, Debug)]
pub struct Word {
    pub id: WordId,
    pub entry: EntryClass,
    pub exit: EntryClass,
    pub defect_fine: u64,
    pub defect_coarse: u64,
    pub body: WordBody,
    pub delta_along: f32,
    pub delta_y: i8,
    pub ticks_min: u32,
    pub ticks_max: u32,
    /// Universal self-chaining rule: valid whenever exit regime feeds entry regime and makes progress.
    pub self_chains: bool,
    /// Holds a kit click somewhere inside: only a carried kit runs it.
    pub clicks: bool,
    pub hits: u64,
    pub ticks_served: u64,
}

impl Word {
    fn zipf_score(&self) -> u128 {
        let base = u128::from(self.hits).saturating_mul(u128::from(self.ticks_served.max(1)));
        let depth_bonus = if matches!(self.body, WordBody::Compound(_)) { 4 } else { 1 };
        let chain_bonus = if self.self_chains { 2 } else { 1 };
        base * depth_bonus * chain_bonus
    }
}

#[derive(Clone, Debug)]
pub struct ObservedStep {
    pub program: u16,
    pub aim: AimRule,
    pub ticks: u32,
    pub entry: EntryClass,
    pub exit: EntryClass,
    pub defect_fine: u64,
    pub defect_coarse: u64,
    pub delta_along: f32,
    pub delta_y: i8,
}

#[derive(Clone, Debug)]
pub struct CachedRoute {
    pub kit_key: u64,
    pub motion_key: Vec<u64>,
    pub guard: Guard,
    pub damage_no_regen: f64,
    pub route: Route,
    pub lo: BlockPos,
    pub hi: BlockPos,
    /// Single-winner read cells (`(pos, mask, legs_or_full_digest)`) verified by `RecordingWorld`.
    pub reads: Vec<(BlockPos, u8, u128)>,
    pub env: (u8, bool, i32, bool),
}

pub(crate) fn motion_key(s: &PlayerState) -> Vec<u64> {
    use law::state_reads::StateField as F;
    let mut key = Vec::with_capacity(48);
    for &field in F::ALL {
        if matches!(
            field,
            F::t | F::fall_distance | F::health | F::food_level | F::saturation | F::exhaustion | F::food_tick_timer
        ) {
            continue;
        }
        s.push_field_bits(field, &mut key);
    }
    key
}

fn rebase_route(orig: &Route, start_t: u64) -> Route {
    let mut route = orig.clone();
    let dt = start_t as i64 - route.words.first().map_or(start_t, |w| w.end.t - u64::from(w.ticks)) as i64;
    if dt != 0 {
        let rebase_t = |t: u64| (t as i64 + dt) as u64;
        for w in &mut route.words {
            w.expected_end_t = rebase_t(w.expected_end_t);
            w.end.t = rebase_t(w.end.t);
        }
        route.end.t = rebase_t(route.end.t);
    }
    route
}

#[derive(Default)]
pub struct WordStore {
    pub work: Work,
    words: Vec<Word>,
    by_atomic: HashMap<(EntryClass, u64, u16, AimRule), WordId>,
    by_compound: HashMap<Vec<WordId>, WordId>,
    pair_counts: HashMap<(WordId, WordId), u32>,
    fine_index: HashMap<(u64, EntryClass), Vec<WordId>>,
    coarse_index: HashMap<(u64, EntryClass), Vec<WordId>>,
    zipf_head: HashMap<EntryClass, Vec<WordId>>,
    routes: HashMap<BlockPos, Vec<CachedRoute>>,
}

impl WordStore {
    pub fn word(&self, id: WordId) -> &Word {
        &self.words[id.0]
    }

    pub fn has_certificates(&self) -> bool {
        !self.routes.is_empty()
    }

    pub fn route(&self, start: &PlayerState, goal: Goal) -> Option<Route> {
        self.route_for_kit(start, goal, 0)
    }

    pub fn route_for_kit(&self, start: &PlayerState, goal: Goal, kit_key: u64) -> Option<Route> {
        let list = self.routes.get(&goal.cell)?;
        let mk = motion_key(start);
        let guard = Guard::of(start);
        let entry = list
            .iter()
            .find(|c| c.kit_key == kit_key && c.motion_key == mk && c.guard.admits(guard, c.damage_no_regen))?;
        Some(rebase_route(&entry.route, start.t))
    }

    pub fn route_for_kit_in_world(
        &self,
        start: &PlayerState,
        goal: Goal,
        kit_key: u64,
        world: &impl World,
    ) -> Option<Route> {
        let list = self.routes.get(&goal.cell)?;
        let mk = motion_key(start);
        let guard = Guard::of(start);
        let env = (world.difficulty(), world.natural_regeneration(), world.min_y(), world.fast_lava());
        let entry = list.iter().find(|c| {
            c.kit_key == kit_key
                && c.env == env
                && c.motion_key == mk
                && c.guard.admits(guard, c.damage_no_regen)
                && c.reads.iter().all(|&(p, mask, digest)| {
                    world
                        .known_block(p)
                        .ok()
                        .and_then(|b| codec::cell(b, world.loaded(p), mask).ok())
                        .is_some_and(|cell| codec::digest(cell.bytes()) == digest)
                })
        })?;
        Some(rebase_route(&entry.route, start.t))
    }

    pub fn file_route(&mut self, start: &PlayerState, goal: Goal, cached: CachedRoute) {
        self.file_route_for_kit(start, goal, 0, cached);
    }

    pub fn file_route_for_kit(&mut self, _start: &PlayerState, goal: Goal, kit_key: u64, mut cached: CachedRoute) {
        cached.kit_key = kit_key;
        let bucket = self.routes.entry(goal.cell).or_default();
        if let Some(slot) = bucket.iter_mut().find(|c| {
            c.kit_key == cached.kit_key
                && c.motion_key == cached.motion_key
                && c.guard == cached.guard
                && c.damage_no_regen == cached.damage_no_regen
        }) {
            *slot = cached;
        } else {
            bucket.push(cached);
        }
    }

    /// Invalidate only cached routes whose recorded cell answers changed at `p`.
    pub fn edit(&mut self, codec: &mut codec::Interner, p: BlockPos, world: &impl World) -> usize {
        let _ = codec;
        if self.routes.is_empty() {
            return 0;
        }
        let mut removed = 0usize;
        self.routes.retain(|_, list| {
            let before = list.len();
            list.retain(|entry| {
                let in_box = (0..3).all(|a| entry.lo[a] <= p[a] && p[a] <= entry.hi[a]);
                if !in_box {
                    return true;
                }
                let mut broken = false;
                for &(cp, mask, digest) in &entry.reads {
                    if cp != p {
                        continue;
                    }
                    let Ok(b) = world.known_block(p) else { broken = true; break; };
                    let cur = match codec::cell(b, world.loaded(p), mask) {
                        Ok(c) => codec::digest(c.bytes()),
                        Err(_) => { broken = true; break; }
                    };
                    if cur != digest {
                        broken = true;
                        break;
                    }
                }
                !broken
            });
            removed += before - list.len();
            !list.is_empty()
        });
        removed
    }

    /// Invalidate cached routes whose recorded reads inside `section` changed; skips non-overlapping sections in O(routes).
    pub fn edit_section(&mut self, codec: &mut codec::Interner, section: BlockPos, world: &impl World) -> usize {
        let _ = codec;
        if self.routes.is_empty() {
            return 0;
        }
        let sx0 = section[0] * 16; let sy0 = section[1] * 16; let sz0 = section[2] * 16;
        let sx1 = sx0 + 15; let sy1 = sy0 + 15; let sz1 = sz0 + 15;
        let mut removed = 0usize;
        self.routes.retain(|_, list| {
            let before = list.len();
            list.retain(|entry| {
                let overlaps = entry.lo[0] <= sx1 && entry.hi[0] >= sx0
                    && entry.lo[1] <= sy1 && entry.hi[1] >= sy0
                    && entry.lo[2] <= sz1 && entry.hi[2] >= sz0;
                if !overlaps {
                    return true;
                }
                let mut broken = false;
                for &(cp, mask, digest) in &entry.reads {
                    if cp[0] < sx0 || cp[0] > sx1 || cp[1] < sy0 || cp[1] > sy1 || cp[2] < sz0 || cp[2] > sz1 {
                        continue;
                    }
                    let Ok(b) = world.known_block(cp) else { broken = true; break; };
                    let cur = match codec::cell(b, world.loaded(cp), mask) {
                        Ok(c) => codec::digest(c.bytes()),
                        Err(_) => { broken = true; break; }
                    };
                    if cur != digest {
                        broken = true;
                        break;
                    }
                }
                !broken
            });
            removed += before - list.len();
            !list.is_empty()
        });
        removed
    }

    pub fn environment_event(&mut self) -> usize {
        let count = self.routes.values().map(Vec::len).sum();
        self.routes.clear();
        count
    }

    fn fill_stage<'a, F: Fn(EntryClass) -> Option<&'a Vec<WordId>>>(
        entries: &[EntryClass; 4],
        stage_cap: usize,
        total_cap: usize,
        out: &mut Vec<WordId>,
        seen: &mut HashSet<WordId>,
        lookup: F,
    ) {
        let start = out.len();
        for &e in entries {
            if out.len() - start >= stage_cap || out.len() >= total_cap {
                break;
            }
            if let Some(list) = lookup(e) {
                for &id in list {
                    if out.len() - start >= stage_cap || out.len() >= total_cap {
                        break;
                    }
                    if seen.insert(id) {
                        out.push(id);
                    }
                }
            }
        }
    }

    fn collect_matching(
        &self,
        sig: DefectSig,
        entry: EntryClass,
        fine_cap: usize,
        coarse_cap: usize,
        head_cap: usize,
        total_cap: usize,
    ) -> (Vec<WordId>, usize) {
        let bands: [u8; 4] = match entry.speed_band {
            0 => [0, 1, 2, 3],
            1 => [1, 2, 0, 3],
            2 => [2, 3, 1, 0],
            _ => [3, 2, 1, 0],
        };
        let mut entries = [entry; 4];
        for (slot, &b) in entries.iter_mut().zip(bands.iter()) {
            slot.speed_band = b;
        }
        let mut out = Vec::with_capacity(total_cap);
        let mut seen = HashSet::new();
        Self::fill_stage(&entries, fine_cap, total_cap, &mut out, &mut seen, |e| {
            self.fine_index.get(&(sig.fine, e))
        });
        let fine = out.len();
        Self::fill_stage(&entries, coarse_cap, total_cap, &mut out, &mut seen, |e| {
            self.coarse_index.get(&(sig.coarse, e))
        });
        if head_cap > 0 {
            Self::fill_stage(&entries, head_cap, total_cap, &mut out, &mut seen, |e| {
                self.zipf_head.get(&e)
            });
        }
        (out, fine)
    }

    /// Top Zipf-ranked matching words (fine defect -> coarse defect -> general regime head),
    /// capped at 14 words total; words with kit clicks only when a kit is carried. The count is
    /// flag marks a word proposed because this exact defect was seen (`fine`); the rest are guesses.
    pub fn propose(&self, sig: DefectSig, entry: EntryClass, kit: bool) -> Vec<(WordId, bool)> {
        let (out, fine) = self.collect_matching(sig, entry, 8, 4, 4, 14);
        out.into_iter().enumerate().filter(|&(_, w)| kit || !self.words[w.0].clicks).map(|(i, w)| (w, i < fine)).collect()
    }

    /// Tight defect-local movement words for Phase B guidance (fine matches only, max 4).
    pub fn propose_local(&self, sig: DefectSig, entry: EntryClass) -> Vec<WordId> {
        let (mut out, _) = self.collect_matching(sig, entry, 4, 0, 0, 4);
        out.retain(|&w| !self.words[w.0].clicks);
        out
    }

    /// Record a winning sequence, emerging compounds by frequency and pruning dominated / tail noise.
    pub fn promote_route(&mut self, steps: &[ObservedStep]) {
        if steps.is_empty() {
            return;
        }
        let mut seq = Vec::with_capacity(steps.len());
        for s in steps {
            let body = WordBody::Atomic { program: s.program, aim: s.aim, max_ticks: 96 };
            let key = (s.entry, s.defect_coarse, s.program, s.aim);
            let id = if let Some(&existing) = self.by_atomic.get(&key) {
                let w = &mut self.words[existing.0];
                w.hits += 1;
                w.ticks_served += u64::from(s.ticks);
                w.ticks_min = w.ticks_min.min(s.ticks);
                w.ticks_max = w.ticks_max.max(s.ticks);
                w.delta_along = w.delta_along.max(s.delta_along);
                existing
            } else {
                let id = WordId(self.words.len());
                let self_chains = s.entry.compatible(s.exit) && (s.delta_along > 0.25 || s.delta_y != 0);
                self.words.push(Word {
                    id,
                    entry: s.entry,
                    exit: s.exit,
                    defect_fine: s.defect_fine,
                    defect_coarse: s.defect_coarse,
                    body,
                    delta_along: s.delta_along,
                    delta_y: s.delta_y,
                    ticks_min: s.ticks,
                    ticks_max: s.ticks,
                    self_chains,
                    clicks: matches!(s.aim, AimRule::Click(_)),
                    hits: 1,
                    ticks_served: u64::from(s.ticks),
                });
                self.by_atomic.insert(key, id);
                id
            };
            seq.push(id);
        }

        // A crossing with kit clicks is one motif (an edit means something only with its
        // restore): it is kept whole, indexed alone at its first step's defect, and offered
        // only to a carried kit, so movement-only proposals stay as they were.
        if seq.iter().any(|w| self.words[w.0].clicks) {
            let cid = self.intern(&seq);
            self.rebuild_indexes(&[], &[cid]);
            self.work.words_total = self.words.len() as u64;
            self.work.compounds_total = self.words.iter().filter(|w| matches!(w.body, WordBody::Compound(_))).count() as u64;
            return;
        }

        // Emerge compounds naturally when adjacent pairs recur (across or within routes).
        let mut created = Vec::new();
        while seq.len() >= 2 {
            let mut local_pairs: HashMap<(WordId, WordId), u32> = HashMap::new();
            for win in seq.windows(2) {
                let p = (win[0], win[1]);
                *local_pairs.entry(p).or_default() += 1;
                *self.pair_counts.entry(p).or_default() += 1;
            }
            let best = seq
                .windows(2)
                .map(|w| (w[0], w[1]))
                .max_by_key(|p| {
                    let g = *self.pair_counts.get(p).unwrap_or(&0);
                    let l = *local_pairs.get(p).unwrap_or(&0);
                    (g >= 2 || l >= 2, g + l)
                });
            let Some(pair) = best else { break };
            let g = *self.pair_counts.get(&pair).unwrap_or(&0);
            let l = *local_pairs.get(&pair).unwrap_or(&0);
            if g < 2 && l < 2 {
                break;
            }
            let cid = self.intern(&[pair.0, pair.1]);
            created.push(cid);
            let mut folded = Vec::with_capacity(seq.len());
            let mut i = 0;
            while i < seq.len() {
                if i + 1 < seq.len() && (seq[i], seq[i + 1]) == pair {
                    folded.push(cid);
                    i += 2;
                } else {
                    folded.push(seq[i]);
                    i += 1;
                }
            }
            seq = folded;
        }

        seq.extend(created);
        self.rebuild_indexes(steps, &seq);
        self.work.words_total = self.words.len() as u64;
        self.work.compounds_total = self.words.iter().filter(|w| matches!(w.body, WordBody::Compound(_))).count() as u64;
    }

    /// The compound of `parts` in order, flattened to atomics; its regime runs from the first
    /// part's entry to the last part's exit.
    fn intern(&mut self, parts: &[WordId]) -> WordId {
        let mut flat = Vec::new();
        for &id in parts {
            match &self.words[id.0].body {
                WordBody::Compound(atoms) => flat.extend_from_slice(atoms),
                WordBody::Atomic { .. } => flat.push(id),
            }
        }
        if let Some(&cid) = self.by_compound.get(&flat) {
            let w = &mut self.words[cid.0];
            w.hits += 1;
            w.ticks_served += u64::from(w.ticks_min.max(1));
            return cid;
        }
        let (first, last) = (self.words[parts[0].0].clone(), self.words[parts[parts.len() - 1].0].clone());
        let ps = || parts.iter().map(|p| &self.words[p.0]);
        let ticks_min = ps().fold(0u32, |a, w| a.saturating_add(w.ticks_min));
        let ticks_max = ps().fold(0u32, |a, w| a.saturating_add(w.ticks_max));
        let delta_along = ps().map(|w| w.delta_along).sum::<f32>();
        let delta_y = ps().fold(0i8, |a, w| a.saturating_add(w.delta_y));
        let clicks = ps().any(|w| w.clicks);
        let cid = WordId(self.words.len());
        self.words.push(Word {
            id: cid,
            entry: first.entry,
            exit: last.exit,
            defect_fine: first.defect_fine,
            defect_coarse: first.defect_coarse,
            body: WordBody::Compound(flat.clone()),
            delta_along,
            delta_y,
            ticks_min,
            ticks_max,
            self_chains: first.entry.compatible(last.exit) && (delta_along > 0.25 || delta_y != 0),
            clicks,
            hits: 2,
            ticks_served: u64::from(ticks_min.max(1)) * 2,
        });
        self.by_compound.insert(flat, cid);
        cid
    }

    fn rebuild_indexes(&mut self, steps: &[ObservedStep], extra: &[WordId]) {
        // Index all newly created or updated words and prune dominated / tail entries by Zipf rank.
        let mut touched = Vec::with_capacity(steps.len() + extra.len());
        for s in steps {
            if let Some(&id) = self.by_atomic.get(&(s.entry, s.defect_coarse, s.program, s.aim)) {
                touched.push(id);
            }
        }
        touched.extend_from_slice(extra);
        for id in touched {
            let w = self.words[id.0].clone();
            let insert_sorted = |map: &mut HashMap<(u64, EntryClass), Vec<WordId>>, key: (u64, EntryClass), cap: usize| {
                let list = map.entry(key).or_default();
                if !list.contains(&id) {
                    list.push(id);
                }
                // Via negativa: remove entries strictly dominated on same defect + regime + delta_y.
                let words = &self.words;
                let mut dominated = vec![false; list.len()];
                for (i, &wi) in list.iter().enumerate() {
                    let a = &words[wi.0];
                    for (j, &wj) in list.iter().enumerate() {
                        if i == j { continue; }
                        let b = &words[wj.0];
                        if a.body == b.body { continue; }
                        if b.exit == a.exit && b.delta_y == a.delta_y && b.delta_along >= a.delta_along + 0.1 && b.ticks_max <= a.ticks_min {
                            dominated[i] = true;
                            break;
                        }
                    }
                }
                let mut idx = 0;
                list.retain(|_| { let keep = !dominated[idx]; idx += 1; keep });
                list.sort_by(|&a, &b| words[b.0].zipf_score().cmp(&words[a.0].zipf_score()));
                list.truncate(cap);
            };
            insert_sorted(&mut self.fine_index, (w.defect_fine, w.entry), 12);
            insert_sorted(&mut self.coarse_index, (w.defect_coarse, w.entry), 10);

            let head = self.zipf_head.entry(w.entry).or_default();
            if !head.contains(&id) {
                head.push(id);
            }
            let words = &self.words;
            head.sort_by(|&a, &b| words[b.0].zipf_score().cmp(&words[a.0].zipf_score()));
            head.truncate(8);
        }
    }

    pub fn flatten_atomic<'a>(&'a self, id: WordId, out: &mut Vec<(u16, AimRule, u16)>) {
        match &self.words[id.0].body {
            WordBody::Atomic { program, aim, max_ticks } => out.push((*program, *aim, *max_ticks)),
            WordBody::Compound(parts) => {
                for &p in parts {
                    self.flatten_atomic(p, out);
                }
            }
        }
    }
}
