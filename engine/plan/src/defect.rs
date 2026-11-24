//! Propose-first raw defect crossing + raw learning on miss/entropy.
//! Never calls `RecordingWorld`; runs purely on raw `law::try_tick` via `LocalGrid` + `TopoField`.
use std::{
    cell::Cell,
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};
use law::{
    Aabb, Block, BlockPos, CollisionContext, FluidKind, PlayerState, World,
    codec::{Interner, LabelId},
};
use words::{
    Arguments, Program,
    program::{HeldPredicate::{Always, Never, OnGround}, tiny_program_id},
};
use crate::{
    Goal, cut,
    inline::List,
    solve::{Stage, ThinkingCosts, WordEvents, WordStep, extend_corridor},
    source::{CorridorProof, CorridorSource},
    state_view::StateAt,
    store::{GeneratorStore, sprint_allowed},
    word_store::{
        AimRule, DefectSig, EntryClass, ObservedStep, WordId, WordStore, approach_frame,
        defect_sig,
    },
};

const MAX_STEPS: usize = 128;
const MAX_GRID_CELLS: usize = 196_608;
const MAX_TOPO_CELLS: usize = 48_000;

const P_FSJ: u16 = tiny_program_id(0, true, Always, Never);
const P_FSG: u16 = tiny_program_id(0, true, OnGround, Never);
const P_FS: u16 = tiny_program_id(0, true, Never, Never);
const P_FG: u16 = tiny_program_id(0, false, OnGround, Never);
const P_FJ: u16 = tiny_program_id(0, false, Always, Never);
const P_F: u16 = tiny_program_id(0, false, Never, Never);
const P_FC: u16 = tiny_program_id(0, false, Never, Always);
const P_FCJ: u16 = tiny_program_id(0, false, OnGround, Always);

const SECONDARY_STRAFE: &[u16] = &[
    tiny_program_id(-1, true, Never, Never),
    tiny_program_id(1, true, Never, Never),
    tiny_program_id(-1, false, Never, Never),
    tiny_program_id(1, false, Never, Never),
];

#[derive(Clone, Copy)]
enum CachedCell<'a> {
    Unseen,
    Unknown,
    Unloaded(&'a Block),
    Known(&'a Block),
}

/// Lazy cached window over `S: World` eliminating repeated BTreeMap lookups during defect search.
struct LocalGrid<'a, S: World> {
    source: &'a S,
    min: BlockPos,
    max: BlockPos,
    dx: usize,
    dy: usize,
    cells: Box<[Cell<CachedCell<'a>>]>,
    missing: Cell<Option<BlockPos>>,
    empty: Block,
    difficulty: u8,
    natural_regeneration: bool,
    min_y: i32,
    fast_lava: bool,
    has_fluid: bool,
    has_inside: bool,
}

impl<'a, S: World> LocalGrid<'a, S> {
    fn new(source: &'a S, bounds: Aabb) -> Self {
        let min = [
            bounds.min.x.floor() as i32 - 2,
            bounds.min.y.floor() as i32 - 2,
            bounds.min.z.floor() as i32 - 2,
        ];
        let max = [
            bounds.max.x.ceil() as i32 + 2,
            bounds.max.y.ceil() as i32 + 2,
            bounds.max.z.ceil() as i32 + 2,
        ];
        let dx = (max[0] - min[0] + 1).max(1) as usize;
        let dy = (max[1] - min[1] + 1).max(1) as usize;
        let dz = (max[2] - min[2] + 1).max(1) as usize;
        let vol = dx.saturating_mul(dy).saturating_mul(dz);
        let cells = if vol <= MAX_GRID_CELLS {
            vec![Cell::new(CachedCell::Unseen); vol].into_boxed_slice()
        } else {
            Box::new([])
        };
        Self {
            source,
            min,
            max,
            dx,
            dy,
            cells,
            missing: Cell::new(None),
            empty: Block::default(),
            difficulty: source.difficulty(),
            natural_regeneration: source.natural_regeneration(),
            min_y: source.min_y(),
            fast_lava: source.fast_lava(),
            has_fluid: source.has_fluid(),
            has_inside: source.has_inside_effects(),
        }
    }

    #[inline]
    fn reset_missing(&self) {
        self.missing.set(None);
    }

    #[inline]
    fn check(&self) -> Result<(), String> {
        match self.missing.get() {
            Some(p) => Err(format!("cell {p:?}: unavailable")),
            None => Ok(()),
        }
    }

    #[inline]
    fn slot(&self, p: BlockPos) -> Option<&Cell<CachedCell<'a>>> {
        if self.cells.is_empty()
            || p[0] < self.min[0] || p[0] > self.max[0]
            || p[1] < self.min[1] || p[1] > self.max[1]
            || p[2] < self.min[2] || p[2] > self.max[2]
        {
            return None;
        }
        let ix = (p[0] - self.min[0]) as usize;
        let iy = (p[1] - self.min[1]) as usize;
        let iz = (p[2] - self.min[2]) as usize;
        Some(&self.cells[(iz * self.dy + iy) * self.dx + ix])
    }

    #[inline]
    fn lookup(&self, p: BlockPos) -> Option<(&Block, bool)> {
        if let Some(cell) = self.slot(p) {
            match cell.get() {
                CachedCell::Known(b) => return Some((b, true)),
                CachedCell::Unloaded(b) => return Some((b, false)),
                CachedCell::Unknown => {
                    if self.missing.get().is_none() {
                        self.missing.set(Some(p));
                    }
                    return None;
                }
                CachedCell::Unseen => {
                    let entry = match self.source.known_block(p) {
                        Ok(b) => {
                            if self.source.loaded(p) {
                                CachedCell::Known(b)
                            } else {
                                CachedCell::Unloaded(b)
                            }
                        }
                        Err(_) => CachedCell::Unknown,
                    };
                    cell.set(entry);
                    return match entry {
                        CachedCell::Known(b) => Some((b, true)),
                        CachedCell::Unloaded(b) => Some((b, false)),
                        CachedCell::Unknown => {
                            if self.missing.get().is_none() {
                                self.missing.set(Some(p));
                            }
                            None
                        }
                        CachedCell::Unseen => unreachable!(),
                    };
                }
            }
        }
        match self.source.known_block(p) {
            Ok(b) => Some((b, self.source.loaded(p))),
            Err(_) => {
                if self.missing.get().is_none() {
                    self.missing.set(Some(p));
                }
                None
            }
        }
    }
}

impl<S: World> World for LocalGrid<'_, S> {
    #[inline]
    fn observe_axis_contact(&self, c: law::contact::AxisContact) {
        self.source.observe_axis_contact(c);
    }

    #[inline]
    fn observe_tick(&self) {
        self.source.observe_tick();
    }

    #[inline]
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
        match self.lookup(p) {
            Some((b, _)) => Ok(b),
            None => Err("unavailable"),
        }
    }

    #[inline]
    fn block(&self, p: BlockPos) -> &Block {
        match self.lookup(p) {
            Some((b, _)) => b,
            None => &self.empty,
        }
    }

    #[inline]
    fn loaded(&self, p: BlockPos) -> bool {
        matches!(self.lookup(p), Some((_, true)))
    }

    #[inline]
    fn min_y(&self) -> i32 {
        self.min_y
    }

    #[inline]
    fn fast_lava(&self) -> bool {
        self.fast_lava
    }

    #[inline]
    fn difficulty(&self) -> u8 {
        self.difficulty
    }

    #[inline]
    fn natural_regeneration(&self) -> bool {
        self.natural_regeneration
    }

    #[inline]
    fn has_fluid(&self) -> bool {
        self.has_fluid
    }

    #[inline]
    fn has_inside_effects(&self) -> bool {
        self.has_inside
    }

    #[inline]
    fn visit_collision_cells(&self, _bounds: Aabb, lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
        if !self.cells.is_empty()
            && lo[0] >= self.min[0] && hi[0] <= self.max[0]
            && lo[1] >= self.min[1] && hi[1] <= self.max[1]
            && lo[2] >= self.min[2] && hi[2] <= self.max[2]
        {
            let base_x = (lo[0] - self.min[0]) as usize;
            for z in lo[2]..=hi[2] {
                let iz = (z - self.min[2]) as usize * self.dy;
                for y in lo[1]..=hi[1] {
                    let row = (iz + (y - self.min[1]) as usize) * self.dx + base_x;
                    for x in lo[0]..=hi[0] {
                        let cell = &self.cells[row + (x - lo[0]) as usize];
                        let entry = match cell.get() {
                            CachedCell::Known(b) => Some((b, true)),
                            CachedCell::Unloaded(_) => continue,
                            CachedCell::Unknown => {
                                if self.missing.get().is_none() {
                                    self.missing.set(Some([x, y, z]));
                                }
                                return false;
                            }
                            CachedCell::Unseen => self.lookup([x, y, z]),
                        };
                        let Some((b, loaded)) = entry else {
                            return false;
                        };
                        if loaded && !b.never_collides() && !visit([x, y, z]) {
                            return false;
                        }
                    }
                }
            }
            return true;
        }
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                for x in lo[0]..=hi[0] {
                    let p = [x, y, z];
                    let Some((b, loaded)) = self.lookup(p) else {
                        return false;
                    };
                    if loaded && !b.never_collides() && !visit(p) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Backward unfolded shortest-path field from goal over passable cells; zero block names, never prunes.
struct TopoField {
    min: BlockPos,
    max: BlockPos,
    dx: usize,
    dy: usize,
    dist: Vec<u16>,
}

#[inline]
fn cell_passable<S: World>(grid: &LocalGrid<'_, S>, p: BlockPos) -> bool {
    let probe = Aabb::new(
        f64::from(p[0]) + 0.25,
        f64::from(p[1]) + 0.05,
        f64::from(p[2]) + 0.25,
        f64::from(p[0]) + 0.75,
        f64::from(p[1]) + 1.55,
        f64::from(p[2]) + 0.75,
    );
    let ctx = CollisionContext { feet_y: f64::from(p[1]), descending: false, fall_distance: 0.0, leather_boots: false };
    let clear = grid.no_collision(probe, ctx);
    grid.reset_missing();
    clear
}

#[inline]
fn has_step_support<S: World>(grid: &LocalGrid<'_, S>, p: BlockPos) -> bool {
    let b = grid.block(p);
    if !b.never_collides() || b.climbable || b.fluid.kind != FluidKind::Empty {
        return true;
    }
    let below = [p[0], p[1] - 1, p[2]];
    let ok = !grid.block(below).never_collides() || grid.block(below).climbable;
    grid.reset_missing();
    ok
}

impl TopoField {
    fn build<S: World>(grid: &LocalGrid<'_, S>, from: &PlayerState, goal: Goal) -> Self {
        let fx = from.position.x.floor() as i32;
        let fy = from.position.y.floor() as i32;
        let fz = from.position.z.floor() as i32;
        let min = [
            fx.min(goal.cell[0]).saturating_sub(20).max(grid.min[0]),
            fy.min(goal.cell[1]).saturating_sub(4).max(grid.min[1]),
            fz.min(goal.cell[2]).saturating_sub(20).max(grid.min[2]),
        ];
        let max = [
            fx.max(goal.cell[0]).saturating_add(20).min(grid.max[0]),
            fy.max(goal.cell[1]).saturating_add(14).min(grid.max[1]),
            fz.max(goal.cell[2]).saturating_add(20).min(grid.max[2]),
        ];
        let dx = (max[0] - min[0] + 1).max(1) as usize;
        let dy = (max[1] - min[1] + 1).max(1) as usize;
        let dz = (max[2] - min[2] + 1).max(1) as usize;
        let vol = dx.saturating_mul(dy).saturating_mul(dz);
        if vol > MAX_TOPO_CELLS {
            return Self { min, max: [min[0] - 1, min[1], min[2]], dx: 0, dy: 0, dist: Vec::new() };
        }
        let mut field = Self { min, max, dx, dy, dist: vec![u16::MAX; vol] };
        let mut buckets: [Vec<BlockPos>; 16] = std::array::from_fn(|_| Vec::new());

        let seeds = [
            goal.cell,
            [goal.cell[0], goal.cell[1] + 1, goal.cell[2]],
            [goal.cell[0], goal.cell[1] - 1, goal.cell[2]],
            [goal.cell[0] + 1, goal.cell[1], goal.cell[2]],
            [goal.cell[0] - 1, goal.cell[1], goal.cell[2]],
            [goal.cell[0], goal.cell[1], goal.cell[2] + 1],
            [goal.cell[0], goal.cell[1], goal.cell[2] - 1],
        ];
        for p in seeds {
            if let Some(i) = field.idx(p) {
                if field.dist[i] == u16::MAX && cell_passable(grid, p) {
                    field.dist[i] = 0;
                    buckets[0].push(p);
                }
            }
        }

        const DIRS: [BlockPos; 6] = [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1], [0, 1, 0], [0, -1, 0]];
        let mut cost = 0usize;
        let mut idle = 0usize;
        let mut pops = 0usize;
        while idle < buckets.len() && pops < 12_000 {
            let bi = cost & 15;
            let Some(cur) = buckets[bi].pop() else {
                cost += 1;
                idle += 1;
                continue;
            };
            idle = 0;
            let Some(ci) = field.idx(cur) else { continue };
            let cur_d = field.dist[ci];
            if usize::from(cur_d) != cost {
                continue;
            }
            pops += 1;
            for d in DIRS {
                let n = [cur[0] + d[0], cur[1] + d[1], cur[2] + d[2]];
                let Some(ni) = field.idx(n) else { continue };
                let step_w: u16 = if n[1] == cur[1] {
                    3
                } else if n[1] < cur[1] {
                    if !has_step_support(grid, n) {
                        continue;
                    }
                    7
                } else {
                    2
                };
                let nd = cur_d.saturating_add(step_w);
                if nd < field.dist[ni] && cell_passable(grid, n) {
                    field.dist[ni] = nd;
                    buckets[(nd as usize) & 15].push(n);
                }
            }
        }
        field
    }

    #[inline]
    fn idx(&self, p: BlockPos) -> Option<usize> {
        if self.dist.is_empty()
            || p[0] < self.min[0] || p[0] > self.max[0]
            || p[1] < self.min[1] || p[1] > self.max[1]
            || p[2] < self.min[2] || p[2] > self.max[2]
        {
            return None;
        }
        let ix = (p[0] - self.min[0]) as usize;
        let iy = (p[1] - self.min[1]) as usize;
        let iz = (p[2] - self.min[2]) as usize;
        Some((iz * self.dy + iy) * self.dx + ix)
    }

    #[inline]
    fn query(&self, s: &PlayerState) -> u16 {
        if self.dist.is_empty() {
            return u16::MAX;
        }
        let fx = s.position.x.floor() as i32;
        let fy = (s.position.y + 0.05).floor() as i32;
        let fz = s.position.z.floor() as i32;
        let mut best = u16::MAX;
        for dy in 0..=1 {
            if let Some(i) = self.idx([fx, fy + dy, fz]) {
                best = best.min(self.dist[i]);
            }
        }
        if best == u16::MAX {
            for ox in [-0.24, 0.24] {
                for oz in [-0.24, 0.24] {
                    let cx = (s.position.x + ox).floor() as i32;
                    let cz = (s.position.z + oz).floor() as i32;
                    if let Some(i) = self.idx([cx, fy, cz]) {
                        best = best.min(self.dist[i]);
                    }
                }
            }
        }
        best
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Bucket {
    qx: i32,
    qy: i32,
    qz: i32,
    on_ground: bool,
    pose: u8,
    in_fluid: bool,
    climbable: bool,
    fast: bool,
    quad: u8,
}

impl Bucket {
    #[inline]
    fn of(s: &PlayerState) -> Self {
        let h2 = s.velocity.horizontal_sqr();
        let fast = h2 >= 0.04;
        let quad = if h2 < 0.01 {
            4
        } else {
            let ang = s.velocity.z.atan2(s.velocity.x).rem_euclid(std::f64::consts::TAU);
            ((ang / std::f64::consts::TAU) * 4.0).floor() as u8 & 3
        };
        Self {
            qx: ((s.position.x + 0.25) * 2.0).floor() as i32,
            qy: (s.position.y * 2.0).floor() as i32,
            qz: ((s.position.z + 0.25) * 2.0).floor() as i32,
            on_ground: s.on_ground,
            pose: s.pose as u8,
            in_fluid: s.in_water || s.in_lava,
            climbable: s.on_climbable,
            fast,
            quad,
        }
    }
}

struct ArenaNode {
    state: PlayerState,
    arrival: u64,
    depth: usize,
    prefix_len: usize,
    parent: Option<usize>,
    edge_steps: Vec<WordStep>,
    edge_obs: Vec<ObservedStep>,
}

fn reconstruct(
    arena: &[ArenaNode],
    leaf: usize,
    extra_steps: Vec<WordStep>,
    extra_obs: Vec<ObservedStep>,
) -> (usize, Vec<WordStep>, Vec<ObservedStep>) {
    let mut chain = Vec::new();
    let mut cur = Some(leaf);
    while let Some(idx) = cur {
        chain.push(idx);
        cur = arena[idx].parent;
    }
    chain.reverse();
    let prefix_len = arena[leaf].prefix_len;
    let mut steps = Vec::new();
    let mut obs = Vec::new();
    for idx in chain {
        steps.extend(arena[idx].edge_steps.iter().cloned());
        obs.extend(arena[idx].edge_obs.iter().cloned());
    }
    steps.extend(extra_steps);
    obs.extend(extra_obs);
    (prefix_len, steps, obs)
}

fn bounds_aabb<S: CorridorSource>(source: &S, from: &PlayerState, goal: Goal, win: &cut::Window) -> Aabb {
    let mut lo = [
        from.position.x.floor() as i32 - 20,
        from.position.y.floor() as i32 - 8,
        from.position.z.floor() as i32 - 20,
    ];
    let mut hi = [
        from.position.x.floor() as i32 + 20,
        from.position.y.floor() as i32 + 20,
        from.position.z.floor() as i32 + 20,
    ];
    for p in [goal.cell, win.first, win.bounds.0, win.bounds.1] {
        for a in [0, 2] {
            lo[a] = lo[a].min(p[a] - 20);
            hi[a] = hi[a].max(p[a] + 20);
        }
        lo[1] = lo[1].min(p[1] - 6);
        hi[1] = hi[1].max(p[1] + 12);
    }
    if let Some((bmin, bmax)) = source.bounds_box() {
        for a in [0, 2] {
            lo[a] = lo[a].max(bmin[a]);
            hi[a] = hi[a].min(bmax[a]);
        }
        lo[1] = bmin[1];
        hi[1] = bmax[1];
    }
    Aabb::new(
        f64::from(lo[0]),
        f64::from(lo[1]),
        f64::from(lo[2]),
        f64::from(hi[0]) + 1.0,
        f64::from(hi[1]) + 1.0,
        f64::from(hi[2]) + 1.0,
    )
}

#[inline]
fn dir_to_goal(s: &PlayerState, goal: Goal) -> (f64, f64) {
    let d = goal.center().minus(s.position);
    let len = d.horizontal_sqr().sqrt();
    if len > 1e-6 {
        (d.x / len, d.z / len)
    } else {
        (0.0, 1.0)
    }
}

#[inline]
fn fast_can_fall<S: World>(grid: &LocalGrid<'_, S>, s: &PlayerState) -> bool {
    if !s.on_ground {
        return false;
    }
    let y = (s.position.y - 0.01).floor() as i32;
    if s.position.y <= f64::from(y) + 1.0 + 1e-5 {
        let b = s.bounding_box();
        let x0 = b.min.x.floor() as i32;
        let x1 = b.max.x.floor() as i32;
        let z0 = b.min.z.floor() as i32;
        let z1 = b.max.z.floor() as i32;
        let mut all_full = true;
        'outer: for cz in z0..=z1 {
            for cx in x0..=x1 {
                if let Some(cell) = grid.slot([cx, y, cz]) {
                    if let CachedCell::Known(blk) = cell.get() {
                        if blk.shape.is_block()
                            && blk.above_shape.is_none()
                            && blk.below_shape.is_none()
                            && blk.sneak_shape.is_none()
                            && blk.inside_cell_shape.is_none()
                        {
                            continue;
                        }
                    }
                }
                all_full = false;
                break 'outer;
            }
        }
        if all_full {
            return false;
        }
    }
    law::can_fall(s, grid, 0.0, 0.0, f64::from(s.step_height))
}

#[inline]
fn can_clear_probe<W: World>(w: &W, s: &PlayerState, ux: f64, uz: f64, reach: f64) -> bool {
    let b = s.bounding_box();
    let dx = ux * reach;
    let dz = uz * reach;
    let swept = Aabb::new(
        b.min.x + dx.min(0.0),
        b.min.y + 0.01,
        b.min.z + dz.min(0.0),
        b.max.x + dx.max(0.0),
        b.max.y,
        b.max.z + dz.max(0.0),
    );
    w.no_collision(swept, CollisionContext::of(s))
}

#[inline]
fn can_clear_shift_y<W: World>(w: &W, s: &PlayerState, dy: f64) -> bool {
    let b = s.bounding_box();
    let shifted = Aabb::new(b.min.x, b.min.y + dy, b.min.z, b.max.x, b.max.y + dy, b.max.z);
    w.no_collision(shifted, CollisionContext::of(s))
}

enum AtomicOutcome {
    Stopped { end: PlayerState, step: WordStep, hit_goal: bool, obs: ObservedStep },
    Failed,
}

fn exec_atomic_raw<S: World>(
    grid: &LocalGrid<'_, S>,
    programs: &[Program],
    start: &PlayerState,
    goal: Goal,
    program_id: u16,
    aim: AimRule,
    max_ticks: u32,
    bounds: Aabb,
    site_label: LabelId,
    provenance_hash: [u8; 32],
    sig: DefectSig,
    raw_ticks: &mut u64,
) -> AtomicOutcome {
    let yaw_idx = aim.resolve(start, goal, sig.rot);
    let args = Arguments { refined_yaw: Some(crate::lattice::yaw(yaw_idx)), ..Default::default() };
    let program = &programs[program_id as usize];
    let held = program.held_input(args);
    let mut s = start.clone();
    let start_entry = EntryClass::of(start);
    let mut prev_can_fall = fast_can_fall(grid, &s);
    let (gx, gz) = dir_to_goal(&s, goal);
    let (hx, hz) = crate::lattice::direction(yaw_idx);
    let heading_dot_goal = hx * gx + hz * gz;
    let steering = heading_dot_goal < 0.72;
    let mut prev_open_goal = can_clear_probe(grid, &s, gx, gz, 0.95);
    let mut prev_left_open = steering && can_clear_probe(grid, &s, -hz, hx, 1.05);
    let mut prev_right_open = steering && can_clear_probe(grid, &s, hz, -hx, 1.05);
    let mut last_half = [((s.position.x + 0.25) * 2.0).floor() as i32, ((s.position.z + 0.25) * 2.0).floor() as i32];

    let cap = max_ticks.min(if steering && s.on_ground && !s.on_climbable && !s.in_water { 24 } else { 80 });
    let mut ticks = 0u32;
    let mut hit_goal = false;

    for tick in 1..=cap {
        let prev_box = s.bounding_box();
        let prev_pos = s.position;
        let prev_ground = s.on_ground;
        let prev_hcoll = s.horizontal_collision;
        let prev_climbable = s.on_climbable;
        let prev_in_water = s.in_water;

        let input = match held {
            Some(inp) => inp,
            None => match program.select(grid, &s, tick - 1, args) {
                Some((_, inp)) => inp,
                None => return AtomicOutcome::Failed,
            },
        };
        grid.reset_missing();
        if law::try_tick(&mut s, grid, &input).is_err() || grid.check().is_err() {
            return AtomicOutcome::Failed;
        }
        *raw_ticks += 1;
        ticks = tick;

        if s.health <= 0.0 || !s.bounding_box().intersects(bounds) {
            return AtomicOutcome::Failed;
        }
        if goal.touched_boxes(prev_box, s.bounding_box()) {
            hit_goal = true;
            break;
        }
        let landed = !prev_ground && s.on_ground;
        let stepped = prev_ground && s.on_ground && s.position.y != prev_pos.y;
        let hit_wall = s.on_ground && s.horizontal_collision && !prev_hcoll;
        let stalled = s.on_ground && s.movement_this_tick.is_none();
        let stuck_on_ground = s.on_ground
            && prev_ground
            && (s.position.y - prev_pos.y).abs() < 1e-6
            && s.position.minus(prev_pos).horizontal_sqr() < 1e-6
            && tick >= 2;
        let cur_can_fall = fast_can_fall(grid, &s);
        let support_ends = cur_can_fall && !prev_can_fall;
        prev_can_fall = cur_can_fall;
        let medium_changed = s.on_climbable != prev_climbable || s.in_water != prev_in_water;

        // Check doorway/corner clearing once per shifted 0.5-block boundary centered on [k+0.25..k+0.75].
        let mut opening_cleared = false;
        let cur_half = [((s.position.x + 0.25) * 2.0).floor() as i32, ((s.position.z + 0.25) * 2.0).floor() as i32];
        if cur_half != last_half {
            last_half = cur_half;
            if s.on_ground && (steering || !prev_open_goal || prev_hcoll) {
                let (cgx, cgz) = dir_to_goal(&s, goal);
                let cur_open_goal = can_clear_probe(grid, &s, cgx, cgz, 0.95);
                if tick >= 2 && !prev_open_goal && cur_open_goal {
                    opening_cleared = true;
                }
                prev_open_goal = cur_open_goal;
                if steering {
                    let l_open = can_clear_probe(grid, &s, -hz, hx, 1.05);
                    let r_open = can_clear_probe(grid, &s, hz, -hx, 1.05);
                    if tick >= 2 && ((!prev_left_open && l_open) || (!prev_right_open && r_open)) {
                        opening_cleared = true;
                    }
                    prev_left_open = l_open;
                    prev_right_open = r_open;
                }
            }
        }

        // Abort hopeless midair stall against a wall when descending with zero speed.
        if !s.on_ground && s.horizontal_collision && s.velocity.horizontal_sqr() < 1e-6 && s.velocity.y < -0.1 && tick > 4 {
            return AtomicOutcome::Failed;
        }

        if landed || stepped || hit_wall || stalled || stuck_on_ground || support_ends || medium_changed || opening_cleared {
            break;
        }
    }

    if ticks == 0 {
        return AtomicOutcome::Failed;
    }
    if s.position == start.position && s.velocity == start.velocity && s.pose == start.pose && !hit_goal {
        return AtomicOutcome::Failed;
    }

    let (_, fwd, _) = approach_frame(start, goal);
    let dx = s.position.x - start.position.x;
    let dz = s.position.z - start.position.z;
    let delta_along = (dx * f64::from(fwd[0]) + dz * f64::from(fwd[1])) as f32;
    let delta_y = (s.position.y - start.position.y).round().clamp(-127.0, 127.0) as i8;
    let step = WordStep {
        program: program_id,
        args,
        ticks,
        expected_end_t: s.t,
        events: WordEvents::default(),
        certificate: CorridorProof {
            label: site_label,
            min: [0; 3],
            max: [0; 3],
            version: provenance_hash,
        },
        end: StateAt::point(s.clone()),
    };
    let obs_step = ObservedStep {
        program: program_id,
        aim,
        ticks,
        entry: start_entry,
        exit: EntryClass::of(&s),
        defect_fine: sig.fine,
        defect_coarse: sig.coarse,
        delta_along,
        delta_y,
    };
    AtomicOutcome::Stopped { end: s, step, hit_goal, obs: obs_step }
}

fn try_tail_extend<S: CorridorSource>(
    state: &PlayerState,
    goal: Goal,
    source: &S,
    store: &GeneratorStore,
    codec: &mut Interner,
) -> Option<Vec<WordStep>> {
    if !state.on_ground || goal.cell[1] != source.floor_y(state) {
        return None;
    }
    if !state.main_supporting_block.is_some_and(|p| p[1] == goal.cell[1]) {
        return None;
    }
    let label = source.entry_label(codec, state).ok()?;
    let Ok(words) = extend_corridor(state, goal, label, source, store, codec) else { return None; };
    if words.is_empty() {
        return None;
    }
    Some(words.into_iter().collect())
}

fn exec_word_chain<S: CorridorSource>(
    grid: &LocalGrid<'_, S>,
    word_store: &WordStore,
    programs: &[Program],
    start: &PlayerState,
    goal: Goal,
    wid: WordId,
    bounds: Aabb,
    label: LabelId,
    provenance: [u8; 32],
    sig: DefectSig,
    raw_ticks: &mut u64,
) -> Option<(PlayerState, Vec<WordStep>, Vec<ObservedStep>, bool)> {
    let mut seq = Vec::new();
    word_store.flatten_atomic(wid, &mut seq);
    let mut s = start.clone();
    let mut steps = Vec::with_capacity(seq.len());
    let mut obs_list = Vec::with_capacity(seq.len());
    let mut cur_sig = sig;
    for (prog, aim, max_t) in seq {
        match exec_atomic_raw(
            grid,
            programs,
            &s,
            goal,
            prog,
            aim,
            u32::from(max_t),
            bounds,
            label,
            provenance,
            cur_sig,
            raw_ticks,
        ) {
            AtomicOutcome::Failed => return None,
            AtomicOutcome::Stopped { end, step, hit_goal, obs } => {
                steps.push(step);
                obs_list.push(obs);
                if hit_goal {
                    return Some((end, steps, obs_list, true));
                }
                s = end;
                cur_sig = defect_sig(grid, &s, goal);
            }
        }
    }
    Some((s, steps, obs_list, false))
}

#[inline]
fn euclid_h(s: &PlayerState, goal: Goal) -> u64 {
    let horiz = ((goal.center().x - s.position.x).powi(2) + (goal.center().z - s.position.z).powi(2)).sqrt();
    let dy_up = (goal.zone().min.y - s.position.y).max(0.0);
    let dy_dn = (s.position.y - goal.zone().max.y).max(0.0);
    ((horiz / 0.28).max(dy_up * 7.5) + dy_up * 2.5 + dy_dn * 1.5).ceil() as u64
}

#[inline]
fn priority_key_topo(arrival: u64, s: &PlayerState, goal: Goal, topo: &TopoField) -> u64 {
    let d = topo.query(s);
    if d < u16::MAX {
        arrival.saturating_add(u64::from(d))
    } else {
        arrival.saturating_add(500).saturating_add(euclid_h(s, goal))
    }
}

fn candidate_aims(s: &PlayerState, goal: Goal, rot: u8, direct_blocked: bool) -> List<AimRule, 12> {
    const CORE: [AimRule; 8] = [
        AimRule::GoalOffset(0),
        AimRule::RelativeCardinal(0),
        AimRule::GoalOffset(4096),
        AimRule::GoalOffset(-4096),
        AimRule::GoalOffset(8192),
        AimRule::GoalOffset(-8192),
        AimRule::RelativeCardinal(1),
        AimRule::RelativeCardinal(3),
    ];
    const ESCAPE: [AimRule; 3] = [
        AimRule::GoalOffset(16384),
        AimRule::GoalOffset(-16384),
        AimRule::RelativeCardinal(2),
    ];
    let mut yaws = [u16::MAX; 12];
    let mut out = List::new();
    let mut push_unique = |rule: AimRule| {
        let yaw = rule.resolve(s, goal, rot);
        if !yaws[..out.len()].contains(&yaw) && out.len() < 12 {
            yaws[out.len()] = yaw;
            out.push(rule);
        }
    };
    for &rule in &CORE {
        push_unique(rule);
    }
    if direct_blocked {
        for &rule in &ESCAPE {
            push_unique(rule);
        }
    }
    out
}

fn programs_for<S: World>(grid: &LocalGrid<'_, S>, s: &PlayerState) -> List<u16, 8> {
    let mut out = List::new();
    if s.in_water || s.in_lava || s.on_climbable {
        for &p in &[P_FSJ, P_FJ, P_FS, P_F] {
            out.push(p);
        }
        return out;
    }
    for &p in &[P_FSG, P_FS, P_FG, P_F] {
        out.push(p);
    }
    let low_headroom = !can_clear_shift_y(grid, s, 0.35);
    let on_ledge = s.on_ground && fast_can_fall(grid, s);
    if s.crouching || low_headroom || on_ledge {
        out.push(P_FC);
    }
    if s.crouching {
        out.push(P_FCJ);
    }
    out
}

/// Cross the defect: Phase A proposes from generalized `WordStore` in microseconds;
/// Phase B learns raw only on miss; Phase C certifies ONLY the single winner.
pub(super) fn solve_defect<S: CorridorSource>(
    from: &PlayerState,
    goal: Goal,
    label: LabelId,
    win: cut::Window,
    source: &S,
    store: &GeneratorStore,
    codec: &mut Interner,
    stages: &mut List<Stage, 16>,
    thinking: &mut ThinkingCosts,
) -> Result<(Vec<WordStep>, Vec<ObservedStep>), (usize, String)> {
    let clock = std::time::Instant::now();
    let programs = words::tiny_programs();
    let bounds = bounds_aabb(source, from, goal, &win);
    let grid = LocalGrid::new(source, bounds);
    let provenance = source.provenance();

    let mut seeds: Vec<ArenaNode> = Vec::with_capacity(win.prefixes.len().max(1));
    for pref in win.prefixes {
        let arrival: u64 = pref.iter().map(|w| u64::from(w.ticks)).sum();
        let state = pref.last().map_or_else(|| from.clone(), |w| w.end.materialize());
        let prefix_len = pref.len();
        seeds.push(ArenaNode {
            state,
            arrival,
            depth: prefix_len,
            prefix_len,
            parent: None,
            edge_steps: pref.into_iter().collect(),
            edge_obs: Vec::new(),
        });
    }
    if seeds.is_empty() {
        seeds.push(ArenaNode {
            state: from.clone(),
            arrival: 0,
            depth: 0,
            prefix_len: 0,
            parent: None,
            edge_steps: Vec::new(),
            edge_obs: Vec::new(),
        });
    }

    let mut proposals_tried = 0u64;
    let mut proposal_hits = 0u64;
    let mut raw_learn_ticks = 0u64;

    // Phase A: Propose from WordStore ONLY (raw try_tick on cached LocalGrid, zero learning bloat).
    let mut best: Option<(u64, usize, Vec<WordStep>, Vec<ObservedStep>)> = None;
    let propose_budget_ticks = 4_000u64;
    let mut prop_ticks = 0u64;
    let has_words = store.words.borrow().work.words_total > 0;

    if has_words {
        let mut heap: BinaryHeap<Reverse<(u64, u64, usize)>> = BinaryHeap::new();
        let mut arena: Vec<ArenaNode> = Vec::new();
        let mut best_bucket: HashMap<Bucket, u64> = HashMap::new();
        for s in &seeds {
            let idx = arena.len();
            let p = s.arrival.saturating_add(euclid_h(&s.state, goal));
            arena.push(ArenaNode {
                state: s.state.clone(),
                arrival: s.arrival,
                depth: s.depth,
                prefix_len: s.prefix_len,
                parent: None,
                edge_steps: s.edge_steps.clone(),
                edge_obs: Vec::new(),
            });
            heap.push(Reverse((p, s.arrival, idx)));
        }
        let mut pops = 0u32;
        while let Some(Reverse((_, _, idx))) = heap.pop() {
            if prop_ticks >= propose_budget_ticks || pops >= 64 {
                break;
            }
            let ub = best.as_ref().map_or(u64::MAX, |b| b.0);
            if arena[idx].arrival.saturating_add(u64::from(goal.lower_bound(&arena[idx].state))) >= ub {
                continue;
            }
            if arena[idx].depth >= MAX_STEPS {
                continue;
            }
            let b = Bucket::of(&arena[idx].state);
            if best_bucket.get(&b).is_some_and(|&prev| prev <= arena[idx].arrival) {
                continue;
            }
            best_bucket.insert(b, arena[idx].arrival);
            pops += 1;

            if goal.touched(&arena[idx].state, &arena[idx].state) {
                let (pl, steps, obs) = reconstruct(&arena, idx, Vec::new(), Vec::new());
                best = Some((arena[idx].arrival, pl, steps, obs));
                proposal_hits += 1;
                break;
            }

            let sig = defect_sig(&grid, &arena[idx].state, goal);
            let entry = EntryClass::of(&arena[idx].state);
            let wids = store.words.borrow().propose(sig, entry);
            for wid in wids {
                if prop_ticks >= propose_budget_ticks {
                    break;
                }
                proposals_tried += 1;
                let ws = store.words.borrow();
                let self_chains = ws.word(wid).self_chains;
                let mut cur_s = arena[idx].state.clone();
                let mut cur_steps = Vec::new();
                let mut cur_obs = Vec::new();
                let mut cur_arr = arena[idx].arrival;
                let mut cur_depth = arena[idx].depth;
                let max_repeats = if self_chains { 16 } else { 1 };
                for rep in 0..max_repeats {
                    let prev_h = euclid_h(&cur_s, goal);
                    let cur_sig = if rep == 0 { sig } else { defect_sig(&grid, &cur_s, goal) };
                    let Some((ex, st_list, ob_list, hit)) = exec_word_chain(
                        &grid,
                        &ws,
                        &programs,
                        &cur_s,
                        goal,
                        wid,
                        bounds,
                        label,
                        provenance,
                        cur_sig,
                        &mut prop_ticks,
                    ) else {
                        break;
                    };
                    let dt: u64 = st_list.iter().map(|w| u64::from(w.ticks)).sum();
                    cur_arr = cur_arr.saturating_add(dt);
                    cur_depth += st_list.len();
                    cur_steps.extend(st_list);
                    cur_obs.extend(ob_list);
                    cur_s = ex;
                    if cur_depth > MAX_STEPS || cur_arr >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                        break;
                    }
                    if hit {
                        let (pl, full_s, full_o) = reconstruct(&arena, idx, cur_steps, cur_obs);
                        proposal_hits += 1;
                        best = Some((cur_arr, pl, full_s, full_o));
                        break;
                    }
                    if let Some(tail) = try_tail_extend(&cur_s, goal, source, store, codec) {
                        let tail_t: u64 = tail.iter().map(|w| u64::from(w.ticks)).sum();
                        let tot = cur_arr.saturating_add(tail_t);
                        if tot < best.as_ref().map_or(u64::MAX, |b| b.0) {
                            let mut chain_steps = cur_steps.clone();
                            chain_steps.extend(tail);
                            let (pl, full_s, full_o) = reconstruct(&arena, idx, chain_steps, cur_obs.clone());
                            proposal_hits += 1;
                            best = Some((tot, pl, full_s, full_o));
                        }
                        break;
                    }
                    if rep >= 1 && euclid_h(&cur_s, goal) > prev_h.saturating_add(6) {
                        break;
                    }
                    let cb = Bucket::of(&cur_s);
                    if best_bucket.get(&cb).is_some_and(|&prev| prev <= cur_arr) {
                        continue;
                    }
                    let p = cur_arr.saturating_add(euclid_h(&cur_s, goal));
                    let nid = arena.len();
                    arena.push(ArenaNode {
                        state: cur_s.clone(),
                        arrival: cur_arr,
                        depth: cur_depth,
                        prefix_len: arena[idx].prefix_len,
                        parent: Some(idx),
                        edge_steps: cur_steps.clone(),
                        edge_obs: cur_obs.clone(),
                    });
                    heap.push(Reverse((p, cur_arr, nid)));
                }
            }
            if best.is_some() {
                break;
            }
        }
    }

    // Phase B: Raw best-first learning search guided by local-topology BFS (ONLY when Phase A missed).
    if best.is_none() {
        let topo = TopoField::build(&grid, from, goal);
        let mut heap: BinaryHeap<Reverse<(u64, u64, usize)>> = BinaryHeap::new();
        let mut arena: Vec<ArenaNode> = Vec::with_capacity(2048);
        let mut best_bucket: HashMap<Bucket, u64> = HashMap::with_capacity(2048);

        for s in seeds {
            let idx = arena.len();
            let p = priority_key_topo(s.arrival, &s.state, goal, &topo);
            heap.push(Reverse((p, s.arrival, idx)));
            arena.push(s);
        }

        let cold_cap = (96_000u64 + (win.exceptions as u64).min(80) * 1_800).min(240_000);
        let word_budget = cold_cap / 5;
        let mut phase_b_word_ticks = 0u64;
        let mut incumbent_at: Option<u64> = None;

        while let Some(Reverse((_, _, idx))) = heap.pop() {
            if raw_learn_ticks >= cold_cap {
                break;
            }
            if let Some(found_t) = incumbent_at {
                if raw_learn_ticks >= found_t.saturating_add(4_500) {
                    break;
                }
            }
            let ub = best.as_ref().map_or(u64::MAX, |b| b.0);
            if arena[idx].arrival.saturating_add(u64::from(goal.lower_bound(&arena[idx].state))) >= ub {
                continue;
            }
            if arena[idx].depth >= MAX_STEPS {
                continue;
            }
            let b = Bucket::of(&arena[idx].state);
            if best_bucket.get(&b).is_some_and(|&prev| prev <= arena[idx].arrival) {
                continue;
            }
            best_bucket.insert(b, arena[idx].arrival);

            if goal.touched(&arena[idx].state, &arena[idx].state) {
                if arena[idx].arrival < ub {
                    let (pl, steps, obs) = reconstruct(&arena, idx, Vec::new(), Vec::new());
                    best = Some((arena[idx].arrival, pl, steps, obs));
                    incumbent_at.get_or_insert(raw_learn_ticks);
                }
                continue;
            }

            if let Some(tail) = try_tail_extend(&arena[idx].state, goal, source, store, codec) {
                let tail_t: u64 = tail.iter().map(|w| u64::from(w.ticks)).sum();
                let tot = arena[idx].arrival.saturating_add(tail_t);
                if tot < ub {
                    let (pl, steps, obs) = reconstruct(&arena, idx, tail, Vec::new());
                    best = Some((tot, pl, steps, obs));
                    incumbent_at.get_or_insert(raw_learn_ticks);
                }
            }

            let state_here = arena[idx].state.clone();
            let arr_here = arena[idx].arrival;
            let depth_here = arena[idx].depth;
            let pl_here = arena[idx].prefix_len;

            let sig = if has_words && phase_b_word_ticks < word_budget {
                let sig = defect_sig(&grid, &state_here, goal);
                let entry = EntryClass::of(&state_here);
                let wids = store.words.borrow().propose_local(sig, entry);
                for wid in wids {
                    if raw_learn_ticks >= cold_cap || phase_b_word_ticks >= word_budget {
                        break;
                    }
                    let ws = store.words.borrow();
                    let self_chains = ws.word(wid).self_chains;
                    let mut cur_s = state_here.clone();
                    let mut cur_steps = Vec::new();
                    let mut cur_obs = Vec::new();
                    let mut cur_arr = arr_here;
                    let mut cur_depth = depth_here;
                    let reps = if self_chains { 4 } else { 1 };
                    for _rep in 0..reps {
                        let cur_sig = if _rep == 0 { sig } else { defect_sig(&grid, &cur_s, goal) };
                        let mut word_ticks = 0u64;
                        let chain = exec_word_chain(
                            &grid,
                            &ws,
                            &programs,
                            &cur_s,
                            goal,
                            wid,
                            bounds,
                            label,
                            provenance,
                            cur_sig,
                            &mut word_ticks,
                        );
                        raw_learn_ticks += word_ticks;
                        phase_b_word_ticks += word_ticks;
                        let Some((ex, st_list, ob_list, hit)) = chain else {
                            break;
                        };
                        let dt: u64 = st_list.iter().map(|w| u64::from(w.ticks)).sum();
                        cur_arr = cur_arr.saturating_add(dt);
                        cur_depth += st_list.len();
                        cur_steps.extend(st_list);
                        cur_obs.extend(ob_list);
                        cur_s = ex;
                        if cur_depth > MAX_STEPS || cur_arr >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                            break;
                        }
                        if hit {
                            let (pl, full_s, full_o) = reconstruct(&arena, idx, cur_steps, cur_obs);
                            best = Some((cur_arr, pl, full_s, full_o));
                            incumbent_at.get_or_insert(raw_learn_ticks);
                            break;
                        }
                        let cb = Bucket::of(&cur_s);
                        if best_bucket.get(&cb).is_some_and(|&prev| prev <= cur_arr) {
                            continue;
                        }
                        let p = priority_key_topo(cur_arr, &cur_s, goal, &topo);
                        let nid = arena.len();
                        arena.push(ArenaNode {
                            state: cur_s.clone(),
                            arrival: cur_arr,
                            depth: cur_depth,
                            prefix_len: pl_here,
                            parent: Some(idx),
                            edge_steps: cur_steps.clone(),
                            edge_obs: cur_obs.clone(),
                        });
                        heap.push(Reverse((p, cur_arr, nid)));
                    }
                }
                sig
            } else {
                DefectSig { fine: 0, coarse: 0, rot: approach_frame(&state_here, goal).0 }
            };

            let (gx, gz) = dir_to_goal(&state_here, goal);
            let direct_blocked = state_here.horizontal_collision
                || !can_clear_probe(&grid, &state_here, gx, gz, 0.75)
                || state_here.velocity.horizontal_sqr() < 0.01;
            let aims = candidate_aims(&state_here, goal, sig.rot, direct_blocked);
            let progs = programs_for(&grid, &state_here);
            let sprint_ok = sprint_allowed(&state_here);

            let mut expanded = 0usize;
            for &prog in &progs {
                let is_sprint_core = prog == P_FSG || prog == P_FS || prog == P_FSJ;
                let aim_slice = if is_sprint_core || !sprint_ok {
                    &aims[..]
                } else {
                    &aims[..aims.len().min(6)]
                };
                for &aim in aim_slice {
                    if raw_learn_ticks >= cold_cap {
                        break;
                    }
                    match exec_atomic_raw(
                        &grid,
                        &programs,
                        &state_here,
                        goal,
                        prog,
                        aim,
                        96,
                        bounds,
                        label,
                        provenance,
                        sig,
                        &mut raw_learn_ticks,
                    ) {
                        AtomicOutcome::Failed => {}
                        AtomicOutcome::Stopped { end, step, hit_goal, obs } => {
                            expanded += 1;
                            let arr = arr_here.saturating_add(u64::from(step.ticks));
                            if arr >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                                continue;
                            }
                            if hit_goal {
                                let (pl, full_s, full_o) = reconstruct(&arena, idx, vec![step], vec![obs]);
                                best = Some((arr, pl, full_s, full_o));
                                incumbent_at.get_or_insert(raw_learn_ticks);
                                continue;
                            }
                            if best_bucket.get(&Bucket::of(&end)).is_some_and(|&prev| prev <= arr) {
                                continue;
                            }
                            let p = priority_key_topo(arr, &end, goal, &topo);
                            let nid = arena.len();
                            arena.push(ArenaNode {
                                state: end,
                                arrival: arr,
                                depth: depth_here + 1,
                                prefix_len: pl_here,
                                parent: Some(idx),
                                edge_steps: vec![step],
                                edge_obs: vec![obs],
                            });
                            heap.push(Reverse((p, arr, nid)));
                        }
                    }
                }
            }

            if expanded == 0 {
                for &prog in SECONDARY_STRAFE {
                    for &aim in &aims[..aims.len().min(4)] {
                        if raw_learn_ticks >= cold_cap {
                            break;
                        }
                        if let AtomicOutcome::Stopped { end, step, hit_goal, obs } = exec_atomic_raw(
                            &grid,
                            &programs,
                            &state_here,
                            goal,
                            prog,
                            aim,
                            48,
                            bounds,
                            label,
                            provenance,
                            sig,
                            &mut raw_learn_ticks,
                        ) {
                            let arr = arr_here.saturating_add(u64::from(step.ticks));
                            if arr >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                                continue;
                            }
                            if hit_goal {
                                let (pl, full_s, full_o) = reconstruct(&arena, idx, vec![step], vec![obs]);
                                best = Some((arr, pl, full_s, full_o));
                                incumbent_at.get_or_insert(raw_learn_ticks);
                            } else {
                                if best_bucket.get(&Bucket::of(&end)).is_some_and(|&prev| prev <= arr) {
                                    continue;
                                }
                                let p = priority_key_topo(arr, &end, goal, &topo);
                                let nid = arena.len();
                                arena.push(ArenaNode {
                                    state: end,
                                    arrival: arr,
                                    depth: depth_here + 1,
                                    prefix_len: pl_here,
                                    parent: Some(idx),
                                    edge_steps: vec![step],
                                    edge_obs: vec![obs],
                                });
                                heap.push(Reverse((p, arr, nid)));
                            }
                        }
                    }
                }
            }
        }
    }

    let nanos = clock.elapsed().as_nanos();
    thinking.output_ns += nanos;
    {
        let mut ws = store.words.borrow_mut();
        ws.work.proposals_tried += proposals_tried;
        ws.work.proposal_hits += proposal_hits;
        ws.work.learn_ticks += raw_learn_ticks;
        ws.work.law_ticks += prop_ticks + raw_learn_ticks;
    }

    let Some((_, prefix_len, steps, mut obs)) = best else {
        stages.push(Stage::Defect {
            proposals: proposals_tried as u32,
            learn_ticks: raw_learn_ticks,
            certify_ticks: 0,
            nanos,
        });
        return Err((
            win.exceptions,
            format!(
                "defect uncrossed at {:?}: proposals={}, learn_ticks={}",
                win.first,
                proposals_tried,
                raw_learn_ticks
            ),
        ));
    };

    for (i, ob) in obs.iter_mut().enumerate() {
        if ob.defect_coarse == 0 {
            let s_before = if prefix_len + i == 0 {
                from.clone()
            } else {
                steps[prefix_len + i - 1].end.materialize()
            };
            let ssig = defect_sig(&grid, &s_before, goal);
            ob.defect_fine = ssig.fine;
            ob.defect_coarse = ssig.coarse;
        }
    }

    stages.push(Stage::Defect {
        proposals: proposals_tried as u32,
        learn_ticks: raw_learn_ticks,
        certify_ticks: 0,
        nanos,
    });
    Ok((steps, obs))
}
