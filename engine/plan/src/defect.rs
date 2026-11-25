//! Propose-first raw defect crossing + raw learning on miss/entropy.
//! Never calls `RecordingWorld`; runs purely on raw `law::try_tick` against the corridor source and `TopoField`.
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};
use law::{
    Aabb, BlockPos, CollisionContext, FluidKind, PlayerState, Vec3, World,
    basis::cell::StateId,
    codec::{Interner, LabelId},
    sim::WorldDelta,
};
use words::{
    Arguments, Program,
    program::{HeldPredicate::{Always, Never, OnGround}, tiny_program_id},
};
use crate::{
    Goal, cut,
    inline::List,
    kit::{
        KitCounts, KitOp, KitView, ObligationLedger, RestoreMode, VerbKind,
        WRITE_FALL_RESET, WRITE_SHAPE_CLEAR, WRITE_SHAPE_SOLID, WRITE_SHAPE_TOGGLE,
        air_state, can_place_at, digest_edits, powder_snow_state, state_static,
        toggle_open_state, water_source_state,
    },
    solve::{Stage, ThinkingCosts, WordEvents, WordStep, extend_corridor},
    source::{CorridorProof, CorridorSource, ObservedWorld},
    state_view::StateAt,
    store::{GeneratorStore, sprint_allowed},
    word_store::{
        AimRule, DefectSig, EntryClass, ObservedStep, WordId, WordStore, approach_frame,
        defect_sig,
    },
};

const MAX_STEPS: usize = 128;
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

/// Backward unfolded shortest-path field from goal over passable cells; zero block names, never prunes.
struct TopoField {
    min: BlockPos,
    max: BlockPos,
    dx: usize,
    dy: usize,
    dist: Vec<u16>,
}

#[inline]
fn cell_passable<S: World>(grid: &ObservedWorld<'_, S>, p: BlockPos) -> bool {
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
fn has_step_support<S: World>(grid: &ObservedWorld<'_, S>, p: BlockPos) -> bool {
    let b = grid.block(p);
    if !b.never_collides() || b.climbable || b.fluid.kind != FluidKind::Empty {
        return true;
    }
    let below = [p[0], p[1] - 1, p[2]];
    let b = grid.block(below);
    let ok = !b.never_collides() || b.climbable;
    grid.reset_missing();
    ok
}

#[inline]
fn kit_step_cost<S: World>(grid: &ObservedWorld<'_, S>, kv: &KitView, cur: BlockPos, n: BlockPos) -> Option<u16> {
    let writes = kv.writes_mask();
    let passable = cell_passable(grid, n);
    if passable {
        let supported = has_step_support(grid, n);
        if !supported {
            if n[1] < cur[1] {
                if writes & WRITE_SHAPE_SOLID != 0 {
                    let below = [n[0], n[1] - 1, n[2]];
                    if let Some(sid) = grid.state_id(below) {
                        if state_static(sid).replaceable() {
                            return Some(11 + if kv.spec.restore_required { 4 } else { 0 });
                        }
                    }
                }
                return None;
            }
            if n[1] == cur[1] {
                if writes & WRITE_SHAPE_SOLID != 0 {
                    let below = [n[0], n[1] - 1, n[2]];
                    if let Some(sid) = grid.state_id(below) {
                        if state_static(sid).replaceable() {
                            return Some(5 + if kv.spec.restore_required { 1 } else { 0 });
                        }
                    }
                }
                return Some(5);
            }
        }
        return Some(if n[1] == cur[1] { 3 } else if n[1] < cur[1] { 7 } else { 2 });
    }
    let mut extra = 0u16;
    for cell in [n, [n[0], n[1] + 1, n[2]]] {
        let b = grid.block(cell);
        if b.never_collides() {
            continue;
        }
        let Some(sid) = grid.state_id(cell) else { return None };
        let st = state_static(sid);
        if writes & WRITE_SHAPE_TOGGLE != 0 && st.interact_kind().is_hand_toggleable() {
            extra = extra.saturating_add(5);
        } else if writes & WRITE_SHAPE_CLEAR != 0 {
            let row = kv.breaking.get(sid.block(), sid);
            if row.break_ticks_ground == u16::MAX
                || (kv.spec.restore_required && row.restore_mode == RestoreMode::Irreversible)
            {
                return None;
            }
            extra = extra.saturating_add(row.break_ticks_ground.min(200).max(1));
            if kv.spec.restore_required {
                extra = extra.saturating_add(4);
            }
        } else {
            return None;
        }
    }
    if extra == 0 {
        return None;
    }
    let base: u16 = if n[1] == cur[1] { 3 } else if n[1] < cur[1] { 7 } else { 2 };
    Some(base.saturating_add(extra))
}

impl TopoField {
    fn build<S: World>(grid: &ObservedWorld<'_, S>, bounds: Aabb, from: &PlayerState, goal: Goal, kit: Option<&KitView>) -> Self {
        let min = [bounds.min.x, bounds.min.y, bounds.min.z].map(|v| v.floor() as i32 - 2);
        let max = [bounds.max.x, bounds.max.y, bounds.max.z].map(|v| v.ceil() as i32 + 2);
        let fx = from.position.x.floor() as i32;
        let fy = from.position.y.floor() as i32;
        let fz = from.position.z.floor() as i32;
        let min = [
            fx.min(goal.cell[0]).saturating_sub(20).max(min[0]),
            fy.min(goal.cell[1]).saturating_sub(if kit.is_some() { 32 } else { 4 }).max(min[1]),
            fz.min(goal.cell[2]).saturating_sub(20).max(min[2]),
        ];
        let max = [
            fx.max(goal.cell[0]).saturating_add(20).min(max[0]),
            fy.max(goal.cell[1]).saturating_add(14).min(max[1]),
            fz.max(goal.cell[2]).saturating_add(20).min(max[2]),
        ];
        let dx = (max[0] - min[0] + 1).max(1) as usize;
        let dy = (max[1] - min[1] + 1).max(1) as usize;
        let dz = (max[2] - min[2] + 1).max(1) as usize;
        let vol = dx.saturating_mul(dy).saturating_mul(dz);
        if vol > MAX_TOPO_CELLS {
            return Self { min, max: [min[0] - 1, min[1], min[2]], dx: 0, dy: 0, dist: Vec::new() };
        }
        let mut field = Self { min, max, dx, dy, dist: vec![u16::MAX; vol] };

        let seeds = [
            goal.cell,
            [goal.cell[0], goal.cell[1] + 1, goal.cell[2]],
            [goal.cell[0], goal.cell[1] - 1, goal.cell[2]],
            [goal.cell[0] + 1, goal.cell[1], goal.cell[2]],
            [goal.cell[0] - 1, goal.cell[1], goal.cell[2]],
            [goal.cell[0], goal.cell[1], goal.cell[2] + 1],
            [goal.cell[0], goal.cell[1], goal.cell[2] - 1],
        ];
        const DIRS: [BlockPos; 6] = [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1], [0, 1, 0], [0, -1, 0]];

        if let Some(kv) = kit {
            let mut heap: BinaryHeap<Reverse<(u16, BlockPos)>> = BinaryHeap::new();
            for p in seeds {
                if let Some(i) = field.idx(p) {
                    if field.dist[i] == u16::MAX && cell_passable(grid, p) {
                        field.dist[i] = 0;
                        heap.push(Reverse((0, p)));
                    }
                }
            }
            let mut pops = 0usize;
            while let Some(Reverse((cur_d, cur))) = heap.pop() {
                if pops >= 12_000 {
                    break;
                }
                let Some(ci) = field.idx(cur) else { continue };
                if cur_d != field.dist[ci] {
                    continue;
                }
                pops += 1;
                for d in DIRS {
                    let n = [cur[0] + d[0], cur[1] + d[1], cur[2] + d[2]];
                    let Some(ni) = field.idx(n) else { continue };
                    let Some(step_w) = kit_step_cost(grid, kv, cur, n) else { continue };
                    let nd = cur_d.saturating_add(step_w);
                    if nd < field.dist[ni] {
                        field.dist[ni] = nd;
                        heap.push(Reverse((nd, n)));
                    }
                }
            }
            return field;
        }

        let mut buckets: [Vec<BlockPos>; 16] = std::array::from_fn(|_| Vec::new());
        for p in seeds {
            if let Some(i) = field.idx(p) {
                if field.dist[i] == u16::MAX && cell_passable(grid, p) {
                    field.dist[i] = 0;
                    buckets[0].push(p);
                }
            }
        }

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
    edits_hash: u64,
    open_ob: u16,
    has_blocks: bool,
}

impl Bucket {
    #[inline]
    fn of(s: &PlayerState, edits_hash: u64, open_ob: u16, has_blocks: bool) -> Self {
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
            edits_hash,
            open_ob,
            has_blocks,
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
    edits: Vec<(BlockPos, StateId)>,
    counts: KitCounts,
    ledger: ObligationLedger,
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
fn fast_can_fall(grid: &impl World, s: &PlayerState) -> bool {
    if !s.on_ground {
        return false;
    }
    let y = (s.position.y - 0.01).floor() as i32;
    if s.position.y <= f64::from(y) + 1.0 + 1e-5 {
        let b = s.bounding_box();
        let lo = [b.min.x.floor() as i32, y, b.min.z.floor() as i32];
        let hi = [b.max.x.floor() as i32, y, b.max.z.floor() as i32];
        if grid.uniform(lo, hi).is_some_and(|block| {
            block.shape.is_block()
                && block.above_shape.is_none()
                && block.below_shape.is_none()
                && block.sneak_shape.is_none()
                && block.inside_cell_shape.is_none()
        }) {
            return false;
        }
    }
    law::can_fall(s, grid, 0.0, 0.0, f64::from(s.profile.step_height))
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

fn exec_atomic_raw<W: World>(
    world: &W,
    grid_missing: Option<&ObservedWorld<'_, impl World>>,
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
    place_trigger: Option<StateId>,
    clutch_trigger: bool,
    restore_obs: &[BlockPos],
    kit_edge_stop: bool,
    raw_ticks: &mut u64,
) -> AtomicOutcome {
    let yaw_idx = aim.resolve(start, goal, sig.rot);
    let args = Arguments { refined_yaw: Some(crate::lattice::yaw(yaw_idx)), ..Default::default() };
    let program = &programs[program_id as usize];
    let held = program.held_input(args);
    let mut s = start.clone();
    let start_entry = EntryClass::of(start);
    let mut prev_can_fall = fast_can_fall(world, &s);
    let non_jump_prog = matches!(program_id, P_FS | P_F);
    let mut prev_edge_next_fall = kit_edge_stop && non_jump_prog && s.on_ground
        && law::can_fall(&s, world, s.velocity.x * 1.15, s.velocity.z * 1.15, f64::from(s.profile.step_height));
    let mut prev_in_ob: u32 = 0;
    for (i, &op) in restore_obs.iter().take(32).enumerate() {
        if s.bounding_box().intersects(cell_box(op)) {
            prev_in_ob |= 1u32 << i;
        }
    }
    let gc = goal.center();
    let (gx, gz) = dir_to_goal(&s, goal);
    let (hx, hz) = crate::lattice::direction(yaw_idx);
    let heading_dot_goal = hx * gx + hz * gz;
    let steering = heading_dot_goal < 0.72;
    let mut prev_open_goal = can_clear_probe(world, &s, gx, gz, 0.95);
    let mut prev_left_open = steering && can_clear_probe(world, &s, -hz, hx, 1.05);
    let mut prev_right_open = steering && can_clear_probe(world, &s, hz, -hx, 1.05);
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
            None => match program.select(world, &s, tick - 1, args) {
                Some((_, inp)) => inp,
                None => return AtomicOutcome::Failed,
            },
        };
        if let Some(g) = grid_missing { g.reset_missing(); }
        if law::try_tick(&mut s, world, &input).is_err() || grid_missing.is_some_and(|g| g.check().is_err()) {
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
        let cur_can_fall = fast_can_fall(world, &s);
        let support_ends = cur_can_fall && !prev_can_fall;
        prev_can_fall = cur_can_fall;
        let medium_changed = s.on_climbable != prev_climbable || s.in_water != prev_in_water;

        // Kit event exits: mid-air bridge place window and pre-impact fall clutch window.
        let mut kit_window = false;
        if let Some(place_blk) = place_trigger {
            if !s.on_ground && tick >= 2 && s.velocity.y < 0.05 {
                let fy = (s.position.y - 0.99).floor() as i32;
                let fx = s.position.x.floor() as i32;
                let fz = s.position.z.floor() as i32;
                let sx = if s.velocity.x > 0.05 { 1 } else if s.velocity.x < -0.05 { -1 } else { 0 };
                let sz = if s.velocity.z > 0.05 { 1 } else if s.velocity.z < -0.05 { -1 } else { 0 };
                for cell in [[fx, fy, fz], [fx + sx, fy, fz + sz], [fx + sx, fy - 1, fz + sz], [fx, fy - 1, fz]] {
                    if world.block(cell).never_collides() && can_place_at(world, &s, cell, place_blk) {
                        kit_window = true;
                        break;
                    }
                }
            }
        }
        if !kit_window && clutch_trigger && !s.on_ground && s.fall_distance > 2.2 && s.velocity.y < -0.15 {
            let fx = s.position.x.floor() as i32;
            let fz = s.position.z.floor() as i32;
            let next_y = (s.position.y + s.velocity.y).floor() as i32;
            let cur_y = s.position.y.floor() as i32;
            for check_y in (next_y - 1)..=cur_y {
                if !world.block([fx, check_y, fz]).never_collides() {
                    kit_window = true;
                    break;
                }
            }
        }
        if kit_edge_stop && non_jump_prog && s.on_ground && !cur_can_fall && s.velocity.horizontal_sqr() > 0.015 {
            let next_fall = law::can_fall(&s, world, s.velocity.x * 1.15, s.velocity.z * 1.15, f64::from(s.profile.step_height));
            if !kit_window && next_fall && !prev_edge_next_fall && tick >= 2 {
                kit_window = true;
            }
            prev_edge_next_fall = next_fall;
        } else {
            prev_edge_next_fall = false;
        }
        if !restore_obs.is_empty() {
            let eye = law::interact::get_eye_position(s.position, s.pose.eye_height());
            let s_gd2 = (gc.x - s.position.x).powi(2) + (gc.z - s.position.z).powi(2);
            for (i, &op) in restore_obs.iter().take(32).enumerate() {
                let now_in = s.bounding_box().intersects(cell_box(op));
                let was_in = (prev_in_ob & (1u32 << i)) != 0;
                if now_in {
                    prev_in_ob |= 1u32 << i;
                    continue;
                }
                prev_in_ob &= !(1u32 << i);
                let ob_gd2 = (gc.x - (f64::from(op[0]) + 0.5)).powi(2) + (gc.z - (f64::from(op[2]) + 0.5)).powi(2);
                if s_gd2 + 0.55 < ob_gd2 && law::interact::is_within_block_interaction_range(eye, 4.4, op, 0.0) {
                    let eye_d2 = (eye.x - (f64::from(op[0]) + 0.5)).powi(2)
                        + (eye.y - (f64::from(op[1]) + 0.5)).powi(2)
                        + (eye.z - (f64::from(op[2]) + 0.5)).powi(2);
                    if was_in || (tick >= 2 && eye_d2 > 3.6 * 3.6) {
                        kit_window = true;
                    }
                }
            }
        }

        // Check doorway/corner clearing once per shifted 0.5-block boundary centered on [k+0.25..k+0.75].
        let mut opening_cleared = false;
        let cur_half = [((s.position.x + 0.25) * 2.0).floor() as i32, ((s.position.z + 0.25) * 2.0).floor() as i32];
        if cur_half != last_half {
            last_half = cur_half;
            if s.on_ground && (steering || !prev_open_goal || prev_hcoll) {
                let (cgx, cgz) = dir_to_goal(&s, goal);
                let cur_open_goal = can_clear_probe(world, &s, cgx, cgz, 0.95);
                if tick >= 2 && !prev_open_goal && cur_open_goal {
                    opening_cleared = true;
                }
                prev_open_goal = cur_open_goal;
                if steering {
                    let l_open = can_clear_probe(world, &s, -hz, hx, 1.05);
                    let r_open = can_clear_probe(world, &s, hz, -hx, 1.05);
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

        if landed || stepped || hit_wall || stalled || stuck_on_ground || support_ends || medium_changed || opening_cleared || kit_window {
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
        edit: None,
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
    if state.crouching || state.pose != PlayerState::default().pose || !state.on_ground || goal.cell[1] != source.floor_y(state) {
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

fn exec_word_chain<S: CorridorSource, W: World>(
    world: &W,
    grid_missing: Option<&ObservedWorld<'_, S>>,
    word_store: &WordStore,
    programs: &[Program],
    start: &PlayerState,
    goal: Goal,
    wid: WordId,
    bounds: Aabb,
    label: LabelId,
    provenance: [u8; 32],
    sig: DefectSig,
    place_trigger: Option<StateId>,
    clutch_trigger: bool,
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
            world,
            grid_missing,
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
            place_trigger,
            clutch_trigger,
            &[],
            false,
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
                cur_sig = defect_sig(world, &s, goal);
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
fn priority_key_topo(
    arrival: u64,
    s: &PlayerState,
    goal: Goal,
    topo: &TopoField,
    ledger: &ObligationLedger,
    kit: Option<&KitView>,
) -> u64 {
    let d = topo.query(s);
    let base = if d < u16::MAX {
        arrival.saturating_add(u64::from(d))
    } else {
        arrival.saturating_add(500).saturating_add(euclid_h(s, goal))
    };
    if let Some(kv) = kit {
        if !ledger.open.is_empty() {
            return base.saturating_add(u64::from(ledger.open_debt_lb(s, goal, &kv.cone)));
        }
    }
    base
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

fn programs_for<W: World>(world: &W, s: &PlayerState) -> List<u16, 8> {
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
    let low_headroom = !can_clear_shift_y(world, s, 0.35);
    let on_ledge = s.on_ground && fast_can_fall(world, s);
    if s.crouching || low_headroom || on_ledge {
        out.push(P_FC);
    }
    if s.crouching {
        out.push(P_FCJ);
    }
    out
}

#[inline]
fn cell_box(p: BlockPos) -> Aabb {
    Aabb::new(
        f64::from(p[0]), f64::from(p[1]), f64::from(p[2]),
        f64::from(p[0] + 1), f64::from(p[1] + 1), f64::from(p[2] + 1),
    )
}

#[inline]
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

#[inline]
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
    let after = if let Some(target_open) = open_to_same {
        let cur = ps.get("open")? == "true";
        if cur == target_open { ps } else { toggle_open_state(ps)? }
    } else {
        toggle_open_state(ps)?
    };
    Some((partner_pos, after))
}

fn step_kit_node<S: World>(
    delta: &mut WorldDelta<'_, ObservedWorld<'_, S>>,
    grid: &ObservedWorld<'_, S>,
    programs: &[Program],
    start: &PlayerState,
    goal: Goal,
    prog_id: u16,
    yaw: u16,
    ticks: u32,
    op: KitOp,
    bounds: Aabb,
    label: LabelId,
    provenance: [u8; 32],
    raw_ticks: &mut u64,
) -> Option<(PlayerState, WordStep, bool)> {
    let mark = delta.checkpoint();
    let still = op.holds_still();
    let land = op.lands_at(ticks);
    let args = Arguments { refined_yaw: Some(crate::lattice::yaw(yaw)), ..Default::default() };
    let prog = &programs[prog_id as usize];
    let held = prog.held_input(args);
    let mut s = start.clone();
    let mut hit_goal = false;
    let mut actual = 0u32;
    for t in 0..ticks {
        if t == land {
            op.apply(delta);
        }
        let prev = s.bounding_box();
        let inp = if still {
            Default::default()
        } else {
            match held {
                Some(i) => i,
                None => match prog.select(delta, &s, t, args) {
                    Some((_, i)) => i,
                    None => {
                        delta.rollback(mark);
                        return None;
                    }
                },
            }
        };
        grid.reset_missing();
        if law::try_tick(&mut s, delta, &inp).is_err() || grid.check().is_err() {
            delta.rollback(mark);
            return None;
        }
        *raw_ticks += 1;
        actual += 1;
        if s.health <= 0.0 || !s.bounding_box().intersects(bounds) {
            delta.rollback(mark);
            return None;
        }
        if !still && goal.touched_boxes(prev, s.bounding_box()) {
            hit_goal = true;
            break;
        }
        if matches!(op, KitOp::BucketEmpty { .. })
            && actual >= 1
            && s.fall_distance < 0.5
            && (s.on_ground || s.in_water || s.velocity.y.abs() < 0.25)
        {
            break;
        }
    }
    if still && goal.touched(&s, &s) {
        hit_goal = true;
    }
    delta.rollback(mark);
    if actual == 0 {
        return None;
    }
    let step = WordStep {
        program: prog_id,
        args,
        ticks: actual,
        expected_end_t: s.t,
        events: WordEvents::default(),
        certificate: CorridorProof { label, min: [0; 3], max: [0; 3], version: provenance },
        end: StateAt::point(s.clone()),
        edit: Some(op),
    };
    Some((s, step, hit_goal))
}

struct KitCandidate {
    op: KitOp,
    ticks: u32,
    prog_id: u16,
    yaw: u16,
    score: u16,
}

fn expand_kit_ops<S: World>(
    delta: &mut WorldDelta<'_, ObservedWorld<'_, S>>,
    grid: &ObservedWorld<'_, S>,
    programs: &[Program],
    kv: &KitView,
    node: &ArenaNode,
    node_idx: usize,
    goal: Goal,
    topo: &TopoField,
    bounds: Aabb,
    label: LabelId,
    provenance: [u8; 32],
    best_ub: u64,
    incumbent_at: &mut Option<u64>,
    best: &mut Option<(u64, usize, Vec<WordStep>, Vec<ObservedStep>)>,
    arena: &mut Vec<ArenaNode>,
    best_bucket: &mut HashMap<Bucket, u64>,
    heap: &mut BinaryHeap<Reverse<(u64, u64, usize)>>,
    raw_learn_ticks: &mut u64,
) {
    let s = &node.state;
    let eye = law::interact::get_eye_position(s.position, s.pose.eye_height());
    let writes = kv.writes_mask();
    let fx = s.position.x.floor() as i32;
    let fy = s.position.y.floor() as i32;
    let fz = s.position.z.floor() as i32;
    let (_, fwd, _) = approach_frame(s, goal);
    let aim_yaw = AimRule::GoalOffset(0).resolve(s, goal, 0);
    let gc = goal.center();
    let s_goal_d2 = (gc.x - s.position.x).powi(2) + (gc.z - s.position.z).powi(2);
    let mut cands: Vec<KitCandidate> = Vec::new();

    // 0. Priority-0: close open obligations inside 4.5 eye reach (incl. <=4t bucket pickup).
    for ob in &node.ledger.open {
        if !law::interact::is_within_block_interaction_range(eye, 4.5, ob.pos, 0.0) {
            continue;
        }
        let Some(cur) = delta.state_id(ob.pos) else { continue };
        let st_orig = state_static(ob.original_state);
        let st_cur = state_static(cur);
        let ob_d2 = (gc.x - (f64::from(ob.pos[0]) + 0.5)).powi(2) + (gc.z - (f64::from(ob.pos[2]) + 0.5)).powi(2);
        let past_or_goal = goal.touched(s, s) || s_goal_d2 + 0.55 < ob_d2;

        if cur == water_source_state() || cur == powder_snow_state() {
            let age = node.arrival.saturating_sub(ob.opened_tick);
            let can_pick = (s.on_ground && s.fall_distance < 0.5) || age >= 1;
            if can_pick && (age <= 4 || cur == powder_snow_state()) {
                cands.push(KitCandidate {
                    op: KitOp::BucketPickup { pos: ob.pos, after: ob.original_state },
                    ticks: 1,
                    prog_id: P_FC,
                    yaw: aim_yaw,
                    score: 0,
                });
            }
            continue;
        }

        if st_orig.interact_kind().is_hand_toggleable() {
            if !s.bounding_box().intersects(cell_box(ob.pos)) && past_or_goal {
                let partner = if st_orig.lower_half() || st_orig.upper_half() {
                    let pp = if st_orig.lower_half() { [ob.pos[0], ob.pos[1] + 1, ob.pos[2]] } else { [ob.pos[0], ob.pos[1] - 1, ob.pos[2]] };
                    grid.state_id(pp).filter(|ps| ps.block() == ob.original_state.block()).map(|ps| (pp, ps))
                } else {
                    None
                };
                cands.push(KitCandidate {
                    op: KitOp::Interact { pos: ob.pos, after: ob.original_state, partner },
                    ticks: 1,
                    prog_id: P_FC,
                    yaw: aim_yaw,
                    score: 1,
                });
            }
            continue;
        }

        if ob.original_state == air_state() || st_orig.replaceable() {
            let body_box = s.bounding_box();
            let cbox = cell_box(ob.pos);
            let standing_on = (body_box.min.y - cbox.max.y).abs() < 0.08
                && body_box.max.x > cbox.min.x && body_box.min.x < cbox.max.x
                && body_box.max.z > cbox.min.z && body_box.min.z < cbox.max.z;
            if !standing_on && !body_box.intersects(cbox) && past_or_goal {
                let row = kv.breaking.get(cur.block(), cur);
                let bt = if s.on_ground { row.break_ticks_ground } else { row.break_ticks_air };
                if bt < u16::MAX && bt <= 240 && st_cur.interact_kind() == crate::kit::InteractKind::None {
                    cands.push(KitCandidate {
                        op: KitOp::Break { pos: ob.pos, after: ob.original_state, break_ticks: bt.max(1) },
                        ticks: u32::from(bt.max(1)),
                        prog_id: P_FC,
                        yaw: aim_yaw,
                        score: 2 + bt.min(200),
                    });
                }
            }
        } else if !s.bounding_box().intersects(cell_box(ob.pos)) && past_or_goal && can_place_at(delta, s, ob.pos, ob.original_state) {
            cands.push(KitCandidate {
                op: KitOp::Place { pos: ob.pos, after: ob.original_state },
                ticks: 1,
                prog_id: P_FC,
                yaw: aim_yaw,
                score: 2,
            });
        }
    }

    // 1. Fall reset clutch (`BucketWater` or Nether `BucketPowderSnow`).
    if writes & WRITE_FALL_RESET != 0 && !s.on_ground && s.fall_distance > 2.2 && s.velocity.y < -0.1 {
        for dy in (-4..=0).rev() {
            let floor_p = [fx, fy + dy, fz];
            if delta.block(floor_p).never_collides() {
                continue;
            }
            let land = [fx, fy + dy + 1, fz];
            if !law::interact::is_within_block_interaction_range(eye, 4.5, land, 0.0) {
                break;
            }
            let Some(lsid) = delta.state_id(land) else { break };
            if !state_static(lsid).replaceable() {
                break;
            }
            if kv.verb_effects.iter().any(|v| v.kind == VerbKind::BucketWater) && node.counts.water_buckets > 0 {
                cands.push(KitCandidate {
                    op: KitOp::BucketEmpty { pos: land, after: water_source_state() },
                    ticks: 4,
                    prog_id: P_F,
                    yaw: aim_yaw,
                    score: 3,
                });
            } else if kv.verb_effects.iter().any(|v| v.kind == VerbKind::BucketPowderSnow) && node.counts.powder_snow_buckets > 0 {
                cands.push(KitCandidate {
                    op: KitOp::BucketEmpty { pos: land, after: powder_snow_state() },
                    ticks: 4,
                    prog_id: P_F,
                    yaw: aim_yaw,
                    score: 3,
                });
            }
            break;
        }
    }

    // 2. Interact (`Door`, `TrapDoor`, `FenceGate`) and `Break` (blocked forward cells).
    if writes & (WRITE_SHAPE_TOGGLE | WRITE_SHAPE_CLEAR) != 0 {
        for dx in -2..=2 {
            for dz in -2..=2 {
                for dy in 0..=1 {
                    let p = [fx + dx, fy + dy, fz + dz];
                    if !law::interact::is_within_block_interaction_range(eye, 4.5, p, 0.0) {
                        continue;
                    }
                    let blk = delta.block(p);
                    if blk.never_collides() {
                        continue;
                    }
                    let Some(sid) = delta.state_id(p) else { continue };
                    let st = state_static(sid);
                    if writes & WRITE_SHAPE_TOGGLE != 0 && st.interact_kind().is_hand_toggleable() {
                        if st.upper_half() && delta.state_id([p[0], p[1] - 1, p[2]]).is_some_and(|ls| ls.block() == sid.block()) {
                            continue;
                        }
                        if node.ledger.open.iter().any(|o| o.pos == p) || node.edits.iter().any(|&(ep, _)| ep == p) {
                            continue;
                        }
                        if let Some(after) = toggle_open_state(sid) {
                            let target_open = after.get("open").map(|v| v == "true");
                            let partner = door_partner(delta, p, sid, target_open);
                            let stand_d = topo.idx([p[0], fy, p[2]]).map_or(500, |i| topo.dist[i]);
                            cands.push(KitCandidate {
                                op: KitOp::Interact { pos: p, after, partner },
                                ticks: 1,
                                prog_id: P_FS,
                                yaw: aim_yaw,
                                score: 5 + stand_d.min(200),
                            });
                        }
                        continue;
                    }
                    if writes & WRITE_SHAPE_CLEAR != 0 && !st.interact_kind().is_hand_toggleable() {
                        let dot_fwd = (dx * fwd[0] + dz * fwd[1]) >= 0;
                        if !dot_fwd && (dx != 0 || dz != 0) {
                            continue;
                        }
                        let row = kv.breaking.get(sid.block(), sid);
                        let bt = if s.on_ground { row.break_ticks_ground } else { row.break_ticks_air };
                        if bt == u16::MAX || bt > 240 || (kv.spec.restore_required && row.restore_mode == RestoreMode::Irreversible) {
                            continue;
                        }
                        let clear_sid = if st.waterlogged() { water_source_state() } else { air_state() };
                        let stand_d = topo.idx([p[0], fy, p[2]]).map_or(500, |i| topo.dist[i]);
                        cands.push(KitCandidate {
                            op: KitOp::Break { pos: p, after: clear_sid, break_ticks: bt.max(1) },
                            ticks: u32::from(bt.max(1)),
                            prog_id: P_FC,
                            yaw: aim_yaw,
                            score: 10 + bt + stand_d.min(200),
                        });
                    }
                }
            }
        }
    }

    // 3. Place (`Bridge` / `Mid-air catch` / `Pillar`).
    if writes & WRITE_SHAPE_SOLID != 0 && node.counts.blocks > 0 {
        let ys: &[i32] = if s.on_ground {
            let need_step_up = goal.cell[1] >= fy || s.horizontal_collision;
            if need_step_up { &[fy - 1, fy] } else { &[fy - 1] }
        } else if goal.cell[1] >= fy {
            &[fy - 2, fy - 1, fy]
        } else {
            &[fy - 2, fy - 1]
        };
        let mut place_cands: Vec<(u16, BlockPos, u16, u16)> = Vec::new();
        for dx in -3..=3 {
            for dz in -3..=3 {
                for &py in ys {
                    let p = [fx + dx, py, fz + dz];
                    if node.edits.iter().any(|&(ep, _)| ep == p) {
                        continue;
                    }
                    if !delta.block(p).never_collides() || !can_place_at(delta, s, p, kv.spec.place_block) {
                        continue;
                    }
                    let d_top = topo.idx([p[0], p[1] + 1, p[2]]).map_or(u16::MAX, |i| topo.dist[i]);
                    if d_top == u16::MAX {
                        continue;
                    }
                    let forward_pref = if (p[0] - fx) * fwd[0] + (p[2] - fz) * fwd[1] >= 0 { 0 } else { 20 };
                    let prog = if s.on_ground {
                        if py == fy { P_FSG } else { P_FS }
                    } else {
                        P_FS
                    };
                    let step_center = Vec3::new(f64::from(p[0]) + 0.5, f64::from(p[1]) + 1.0, f64::from(p[2]) + 0.5);
                    let to_p = Goal { cell: [p[0], p[1] + 1, p[2]] };
                    let dir_yaw = if s.position.minus(step_center).horizontal_sqr() > 0.04 {
                        AimRule::GoalOffset(0).resolve(s, to_p, 0)
                    } else {
                        aim_yaw
                    };
                    place_cands.push((d_top.saturating_add(forward_pref), p, prog, dir_yaw));
                }
            }
        }
        place_cands.sort_unstable_by_key(|c| c.0);
        place_cands.truncate(2);
        // The pillar: airborne below a higher goal, the cell under the feet is always offered,
        // whatever the field says of the cells nearer the goal.
        let under = [fx, fy - 1, fz];
        if !s.on_ground && goal.cell[1] >= fy && !place_cands.iter().any(|c| c.1 == under)
            && !node.edits.iter().any(|&(ep, _)| ep == under)
            && delta.block(under).never_collides() && can_place_at(delta, s, under, kv.spec.place_block)
        {
            let d_top = topo.idx([fx, fy, fz]).map_or(u16::MAX, |i| topo.dist[i]);
            place_cands.push((d_top, under, P_FS, aim_yaw));
        }
        for (sc, p, prog_id, pyaw) in place_cands {
            cands.push(KitCandidate {
                op: KitOp::Place { pos: p, after: kv.spec.place_block },
                ticks: 1,
                prog_id,
                yaw: pyaw,
                score: 8 + sc.min(300),
            });
        }
    }

    cands.sort_unstable_by_key(|c| c.score);
    for cand in cands.into_iter().take(6) {
        let Some((end_s, step, hit_goal)) = step_kit_node(
            delta, grid, programs, s, goal, cand.prog_id, cand.yaw, cand.ticks,
            cand.op, bounds, label, provenance, raw_learn_ticks,
        ) else {
            continue;
        };
        let arr = node.arrival.saturating_add(u64::from(step.ticks));
        if arr >= best_ub {
            continue;
        }
        let restore = node.ledger.open.iter().any(|o| o.pos == cand.op.pos());
        let obs = kit_obs(s, &end_s, goal, cand.prog_id, crate::word_store::KitTemplate::of(cand.op, s, goal, cand.yaw, restore), step.ticks);
        let (mut next_edits, mut next_ledger, mut next_counts) = (node.edits.clone(), node.ledger.clone(), node.counts);
        let before = delta.state_id(cand.op.pos()).unwrap_or_else(air_state);
        book(kv, grid, before, cand.op, node.arrival, &mut next_edits, &mut next_ledger, &mut next_counts);

        let ledger_clear = !kv.spec.restore_required || next_ledger.open.is_empty();
        if hit_goal && ledger_clear {
            let (pl, full_s, full_o) = reconstruct(arena, node_idx, vec![step], vec![obs]);
            *best = Some((arr, pl, full_s, full_o));
            incumbent_at.get_or_insert(*raw_learn_ticks);
            return;
        }
        let ehash = digest_edits(&next_edits);
        let b = Bucket::of(&end_s, ehash, next_ledger.open.len() as u16, next_counts.blocks > 0);
        if best_bucket.get(&b).is_some_and(|&prev| prev <= arr) {
            continue;
        }
        let p = priority_key_topo(arr, &end_s, goal, topo, &next_ledger, Some(kv));
        let nid = arena.len();
        arena.push(ArenaNode {
            state: end_s,
            arrival: arr,
            depth: node.depth + 1,
            prefix_len: node.prefix_len,
            parent: Some(node_idx),
            edge_steps: vec![step],
            edge_obs: vec![obs],
            edits: next_edits,
            counts: next_counts,
            ledger: next_ledger,
        });
        heap.push(Reverse((p, arr, nid)));
    }
}

/// What one kit step leaves behind: the edit journal, the restore ledger and the counts.
/// A restore (an edit at an open obligation's cell) gives back what its first edit took.
fn book<S: World>(
    kv: &KitView,
    grid: &ObservedWorld<'_, S>,
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
                update_edits(edits, grid, pp, pa);
            }
            (after, 1)
        }
        KitOp::Break { after, .. } => {
            if restore { counts.blocks = counts.blocks.saturating_add(1); }
            (after, 1)
        }
        KitOp::Place { after, .. } => {
            if !restore { counts.blocks = counts.blocks.saturating_sub(1); }
            (after, kv.breaking.get(after.block(), after).break_ticks_ground.min(200).max(1))
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
    update_edits(edits, grid, pos, after);
    if kv.spec.restore_required || matches!(op, KitOp::BucketEmpty { .. } | KitOp::BucketPickup { .. }) {
        ledger.record_edit(pos, before, after, tick, undo);
    }
}

/// The observation of one kit step, its click kept as a frame-relative template.
fn kit_obs(start: &PlayerState, end: &PlayerState, goal: Goal, program: u16, t: Option<crate::word_store::KitTemplate>, ticks: u32) -> ObservedStep {
    let (_, fwd, _) = approach_frame(start, goal);
    let (dx, dz) = (end.position.x - start.position.x, end.position.z - start.position.z);
    ObservedStep {
        program,
        aim: t.map_or(AimRule::GoalOffset(0), AimRule::Click),
        ticks,
        entry: EntryClass::of(start),
        exit: EntryClass::of(end),
        defect_fine: 0,
        defect_coarse: 0,
        delta_along: (dx * f64::from(fwd[0]) + dz * f64::from(fwd[1])) as f32,
        delta_y: (end.position.y - start.position.y).round().clamp(-127.0, 127.0) as i8,
    }
}

/// A kit template made concrete from `s`: its cell from the approach frame, its after-state from
/// the verb or the open obligation there, and its ticks; `None` where the live cell refuses it.
fn instantiate(
    t: crate::word_store::KitTemplate,
    world: &impl World,
    s: &PlayerState,
    goal: Goal,
    kv: &KitView,
    ledger: &ObligationLedger,
    counts: KitCounts,
) -> Option<(KitOp, u32)> {
    use crate::word_store::KitVerb;
    let pos = t.cell(s, goal);
    let cur = world.state_id(pos)?;
    let ob = ledger.open.iter().find(|o| o.pos == pos);
    if t.restore != ob.is_some() {
        return None;
    }
    let has = |k: VerbKind| kv.verb_effects.iter().any(|v| v.kind == k);
    let solid = !world.block(pos).never_collides();
    Some(match t.verb {
        KitVerb::Interact => {
            if !has(VerbKind::Interact) || !state_static(cur).interact_kind().is_hand_toggleable() {
                return None;
            }
            let after = toggle_open_state(cur)?;
            (KitOp::Interact { pos, after, partner: door_partner(world, pos, cur, after.get("open").map(|v| v == "true")) }, 1)
        }
        KitVerb::Break => {
            let row = kv.breaking.get(cur.block(), cur);
            let bt = if s.on_ground { row.break_ticks_ground } else { row.break_ticks_air };
            let fresh_ok = has(VerbKind::Break) && !(kv.spec.restore_required && row.restore_mode == RestoreMode::Irreversible);
            if !solid || bt == u16::MAX || bt > 240 || ob.is_none() && !fresh_ok {
                return None;
            }
            let after = ob.map_or_else(|| if state_static(cur).waterlogged() { water_source_state() } else { air_state() }, |o| o.original_state);
            (KitOp::Break { pos, after, break_ticks: bt.max(1) }, u32::from(bt.max(1)))
        }
        KitVerb::Place => {
            let after = ob.map_or(kv.spec.place_block, |o| o.original_state);
            if solid || ob.is_none() && (!has(VerbKind::Place) || counts.blocks == 0) || !can_place_at(world, s, pos, after) {
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
            if !has(kind) || n == 0 || !state_static(cur).replaceable() {
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
    })
}

/// One proposed word run on the live delta with a carried kit, appended to `node`: movement
/// atomics stop on the same kit events Phase B learned them with, click atomics are
/// instantiated from their template. Edits stay in `delta`; `Some(true)` when the goal is
/// touched with nothing left to restore.
fn exec_kit_chain<S: World>(
    delta: &mut WorldDelta<'_, ObservedWorld<'_, S>>,
    grid: &ObservedWorld<'_, S>,
    ws: &WordStore,
    programs: &[Program],
    kv: &KitView,
    node: &mut ArenaNode,
    goal: Goal,
    wid: WordId,
    bounds: Aabb,
    label: LabelId,
    provenance: [u8; 32],
    raw_ticks: &mut u64,
) -> Option<bool> {
    let mut seq = Vec::new();
    ws.flatten_atomic(wid, &mut seq);
    let restore_required = kv.spec.restore_required;
    for (prog, aim, max_t) in seq {
        let s = node.state.clone();
        let (end, step, obs, hit) = if let AimRule::Click(t) = aim {
            let (op, ticks) = instantiate(t, &*delta, &s, goal, kv, &node.ledger, node.counts)?;
            let yaw = aim.resolve(&s, goal, 0);
            let (end, step, hit) = step_kit_node(delta, grid, programs, &s, goal, prog, yaw, ticks, op, bounds, label, provenance, raw_ticks)?;
            let before = delta.state_id(op.pos()).unwrap_or_else(air_state);
            op.apply(delta);
            book(kv, grid, before, op, node.arrival, &mut node.edits, &mut node.ledger, &mut node.counts);
            let obs = kit_obs(&s, &end, goal, prog, Some(t), step.ticks);
            (end, step, obs, hit)
        } else {
            let restore_obs: Vec<BlockPos> = if restore_required { node.ledger.open.iter().map(|o| o.pos).collect() } else { Vec::new() };
            let place = (kv.writes_mask() & WRITE_SHAPE_SOLID != 0 && node.counts.blocks > 0).then_some(kv.spec.place_block);
            let clutch = kv.writes_mask() & WRITE_FALL_RESET != 0 && (node.counts.water_buckets > 0 || node.counts.powder_snow_buckets > 0);
            let sig = defect_sig(grid, &s, goal);
            match exec_atomic_raw(&*delta, Some(grid), programs, &s, goal, prog, aim, u32::from(max_t), bounds, label, provenance, sig, place, clutch, &restore_obs, true, raw_ticks) {
                AtomicOutcome::Failed => return None,
                AtomicOutcome::Stopped { end, step, hit_goal, obs } => (end, step, obs, hit_goal),
            }
        };
        node.arrival = node.arrival.saturating_add(u64::from(step.ticks));
        node.depth += 1;
        node.state = end;
        node.edge_steps.push(step);
        node.edge_obs.push(obs);
        if hit && (!restore_required || node.ledger.open.is_empty()) {
            return Some(true);
        }
    }
    Some(false)
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
    kit: Option<&KitView>,
    stages: &mut List<Stage, 16>,
    thinking: &mut ThinkingCosts,
) -> Result<(Vec<WordStep>, Vec<ObservedStep>), (usize, String)> {
    let clock = std::time::Instant::now();
    let programs = words::tiny_programs();
    let bounds = bounds_aabb(source, from, goal, &win);
    let grid = ObservedWorld::defect(source);
    let provenance = source.provenance();
    let init_counts = kit.map_or(KitCounts { blocks: 0, water_buckets: 0, powder_snow_buckets: 0, boats: 0, pearls: 0 }, |k| k.spec.counts());
    let restore_required = kit.is_some_and(|k| k.spec.restore_required);
    let place_trigger = kit.and_then(|k| (k.writes_mask() & WRITE_SHAPE_SOLID != 0 && init_counts.blocks > 0).then_some(k.spec.place_block));
    let clutch_trigger = kit.is_some_and(|k| k.writes_mask() & WRITE_FALL_RESET != 0);

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
            edits: Vec::new(),
            counts: init_counts,
            ledger: ObligationLedger::default(),
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
            edits: Vec::new(),
            counts: init_counts,
            ledger: ObligationLedger::default(),
        });
    }

    let mut proposals_tried = 0u64;
    let mut proposal_hits = 0u64;
    let mut raw_learn_ticks = 0u64;

    // Phase A: propose from the WordStore only, no learning: raw try_tick on the observed world;
    // with a kit, on its delta, each click instantiated from its frame-relative template.
    let mut best: Option<(u64, usize, Vec<WordStep>, Vec<ObservedStep>)> = None;
    let propose_budget_ticks = 4_000u64;
    let mut prop_ticks = 0u64;
    let has_words = store.words.borrow().work.words_total > 0;

    if has_words {
        let mut heap: BinaryHeap<Reverse<(u64, u64, usize)>> = BinaryHeap::new();
        let mut arena: Vec<ArenaNode> = Vec::new();
        let mut best_bucket: HashMap<Bucket, u64> = HashMap::new();
        let mut delta = WorldDelta::new(&grid);
        let lb = |s: &PlayerState| u64::from(kit.map_or_else(|| goal.lower_bound(s), |kv| kv.cone.lower_bound(s, goal)));
        let clear = |n: &ArenaNode| !restore_required || n.ledger.open.is_empty();
        let bucket = |n: &ArenaNode| Bucket::of(&n.state, digest_edits(&n.edits), n.ledger.open.len() as u16, n.counts.blocks > 0);
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
                edits: Vec::new(),
                counts: init_counts,
                ledger: ObligationLedger::default(),
            });
            heap.push(Reverse((p, s.arrival, idx)));
        }
        let mut pops = 0u32;
        while let Some(Reverse((_, _, idx))) = heap.pop() {
            if prop_ticks >= propose_budget_ticks || pops >= 64 {
                break;
            }
            let ub = best.as_ref().map_or(u64::MAX, |b| b.0);
            if arena[idx].arrival.saturating_add(lb(&arena[idx].state)) >= ub || arena[idx].depth >= MAX_STEPS {
                continue;
            }
            let b = bucket(&arena[idx]);
            if best_bucket.get(&b).is_some_and(|&prev| prev <= arena[idx].arrival) {
                continue;
            }
            best_bucket.insert(b, arena[idx].arrival);
            pops += 1;

            if goal.touched(&arena[idx].state, &arena[idx].state) && clear(&arena[idx]) {
                let (pl, steps, obs) = reconstruct(&arena, idx, Vec::new(), Vec::new());
                best = Some((arena[idx].arrival, pl, steps, obs));
                proposal_hits += 1;
                break;
            }

            let sig = defect_sig(&grid, &arena[idx].state, goal);
            let entry = EntryClass::of(&arena[idx].state);
            let wids = store.words.borrow().propose(sig, entry, kit.is_some());
            for wid in wids {
                if prop_ticks >= propose_budget_ticks {
                    break;
                }
                proposals_tried += 1;
                let ws = store.words.borrow();
                let max_repeats = if ws.word(wid).self_chains { 16 } else { 1 };
                let mut child = ArenaNode {
                    state: arena[idx].state.clone(),
                    arrival: arena[idx].arrival,
                    depth: arena[idx].depth,
                    prefix_len: arena[idx].prefix_len,
                    parent: Some(idx),
                    edge_steps: Vec::new(),
                    edge_obs: Vec::new(),
                    edits: arena[idx].edits.clone(),
                    counts: arena[idx].counts,
                    ledger: arena[idx].ledger.clone(),
                };
                if kit.is_some() {
                    delta.rollback(0);
                    for &(ep, es) in &arena[idx].edits {
                        delta.set(ep, es);
                    }
                }
                for rep in 0..max_repeats {
                    let prev_h = euclid_h(&child.state, goal);
                    let hit = match kit {
                        Some(kv) => exec_kit_chain(&mut delta, &grid, &ws, &programs, kv, &mut child, goal, wid, bounds, label, provenance, &mut prop_ticks),
                        None => {
                            let cur_sig = if rep == 0 { sig } else { defect_sig(&grid, &child.state, goal) };
                            exec_word_chain(&grid, Some(&grid), &ws, &programs, &child.state, goal, wid, bounds, label, provenance, cur_sig, None, false, &mut prop_ticks)
                                .map(|(ex, st_list, ob_list, hit)| {
                                    child.arrival = child.arrival.saturating_add(st_list.iter().map(|w| u64::from(w.ticks)).sum());
                                    child.depth += st_list.len();
                                    child.edge_steps.extend(st_list);
                                    child.edge_obs.extend(ob_list);
                                    child.state = ex;
                                    hit
                                })
                        }
                    };
                    let Some(hit) = hit else { break };
                    if child.depth > MAX_STEPS || child.arrival >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                        break;
                    }
                    if hit {
                        let (pl, full_s, full_o) = reconstruct(&arena, idx, child.edge_steps.clone(), child.edge_obs.clone());
                        proposal_hits += 1;
                        best = Some((child.arrival, pl, full_s, full_o));
                        break;
                    }
                    if child.edits.is_empty() && clear(&child) {
                        if let Some(tail) = try_tail_extend(&child.state, goal, source, store, codec) {
                            let tot = child.arrival.saturating_add(tail.iter().map(|w| u64::from(w.ticks)).sum());
                            if tot < best.as_ref().map_or(u64::MAX, |b| b.0) {
                                let mut chain_steps = child.edge_steps.clone();
                                chain_steps.extend(tail);
                                let (pl, full_s, full_o) = reconstruct(&arena, idx, chain_steps, child.edge_obs.clone());
                                proposal_hits += 1;
                                best = Some((tot, pl, full_s, full_o));
                            }
                            break;
                        }
                    }
                    if rep >= 1 && euclid_h(&child.state, goal) > prev_h.saturating_add(6) {
                        break;
                    }
                    if best_bucket.get(&bucket(&child)).is_some_and(|&prev| prev <= child.arrival) {
                        continue;
                    }
                    let p = child.arrival.saturating_add(euclid_h(&child.state, goal));
                    let nid = arena.len();
                    arena.push(ArenaNode {
                        state: child.state.clone(),
                        arrival: child.arrival,
                        depth: child.depth,
                        prefix_len: child.prefix_len,
                        parent: Some(idx),
                        edge_steps: child.edge_steps.clone(),
                        edge_obs: child.edge_obs.clone(),
                        edits: child.edits.clone(),
                        counts: child.counts,
                        ledger: child.ledger.clone(),
                    });
                    heap.push(Reverse((p, child.arrival, nid)));
                }
            }
            if best.is_some() {
                break;
            }
        }
    }

    // Phase B: Raw best-first learning search guided by unfolded TopoField (ONLY when Phase A missed).
    if best.is_none() {
        let topo = TopoField::build(&grid, bounds, from, goal, kit);
        let mut heap: BinaryHeap<Reverse<(u64, u64, usize)>> = BinaryHeap::new();
        let mut arena: Vec<ArenaNode> = Vec::with_capacity(2048);
        let mut best_bucket: HashMap<Bucket, u64> = HashMap::with_capacity(2048);

        for s in seeds {
            let idx = arena.len();
            let p = priority_key_topo(s.arrival, &s.state, goal, &topo, &s.ledger, kit);
            heap.push(Reverse((p, s.arrival, idx)));
            arena.push(s);
        }

        let cold_cap = (96_000u64 + (win.exceptions as u64).min(80) * 1_800).min(240_000);
        let word_budget = cold_cap / 5;
        let mut phase_b_word_ticks = 0u64;
        let mut incumbent_at: Option<u64> = None;
        let mut delta = WorldDelta::new(&grid);
        let has_words_b = store.words.borrow().work.words_total > 0;

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
            let node_lb = kit.map_or_else(
                || u64::from(goal.lower_bound(&arena[idx].state)),
                |kv| u64::from(kv.cone.lower_bound(&arena[idx].state, goal))
                    .max(if restore_required { u64::from(arena[idx].ledger.open_debt_lb(&arena[idx].state, goal, &kv.cone)) } else { 0 }),
            );
            if arena[idx].arrival.saturating_add(node_lb) >= ub {
                continue;
            }
            if arena[idx].depth >= MAX_STEPS {
                continue;
            }
            let node_ehash = digest_edits(&arena[idx].edits);
            let b = Bucket::of(&arena[idx].state, node_ehash, arena[idx].ledger.open.len() as u16, arena[idx].counts.blocks > 0);
            if best_bucket.get(&b).is_some_and(|&prev| prev <= arena[idx].arrival) {
                continue;
            }
            best_bucket.insert(b, arena[idx].arrival);

            if goal.touched(&arena[idx].state, &arena[idx].state)
                && (!restore_required || arena[idx].ledger.open.is_empty())
            {
                if arena[idx].arrival < ub {
                    let (pl, steps, obs) = reconstruct(&arena, idx, Vec::new(), Vec::new());
                    best = Some((arena[idx].arrival, pl, steps, obs));
                    incumbent_at.get_or_insert(raw_learn_ticks);
                }
                continue;
            }

            if arena[idx].edits.is_empty() && (!restore_required || arena[idx].ledger.open.is_empty()) {
                if let Some(tail) = try_tail_extend(&arena[idx].state, goal, source, store, codec) {
                    let tail_t: u64 = tail.iter().map(|w| u64::from(w.ticks)).sum();
                    let tot = arena[idx].arrival.saturating_add(tail_t);
                    if tot < ub {
                        let (pl, steps, obs) = reconstruct(&arena, idx, tail, Vec::new());
                        best = Some((tot, pl, steps, obs));
                        incumbent_at.get_or_insert(raw_learn_ticks);
                    }
                }
            }

            delta.rollback(0);
            for &(ep, es) in &arena[idx].edits {
                delta.set(ep, es);
            }

            if let Some(kv) = kit {
                let ub_now = best.as_ref().map_or(u64::MAX, |b| b.0);
                let snap_node = ArenaNode {
                    state: arena[idx].state.clone(),
                    arrival: arena[idx].arrival,
                    depth: arena[idx].depth,
                    prefix_len: arena[idx].prefix_len,
                    parent: arena[idx].parent,
                    edge_steps: Vec::new(),
                    edge_obs: Vec::new(),
                    edits: arena[idx].edits.clone(),
                    counts: arena[idx].counts,
                    ledger: arena[idx].ledger.clone(),
                };
                expand_kit_ops(
                    &mut delta, &grid, &programs, kv, &snap_node, idx, goal, &topo,
                    bounds, label, provenance, ub_now, &mut incumbent_at, &mut best,
                    &mut arena, &mut best_bucket, &mut heap, &mut raw_learn_ticks,
                );
            }

            let state_here = arena[idx].state.clone();
            let arr_here = arena[idx].arrival;
            let depth_here = arena[idx].depth;
            let pl_here = arena[idx].prefix_len;
            let cur_edits = arena[idx].edits.clone();
            let cur_counts = arena[idx].counts;
            let cur_ledger = arena[idx].ledger.clone();
            let cur_place_trig = place_trigger.filter(|_| cur_counts.blocks > 0);
            let cur_clutch_trig = clutch_trigger && (cur_counts.water_buckets > 0 || cur_counts.powder_snow_buckets > 0);
            let ledger_clear = !restore_required || cur_ledger.open.is_empty();

            let sig = if has_words_b && phase_b_word_ticks < word_budget && cur_edits.is_empty() {
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
                            Some(&grid),
                            &ws,
                            &programs,
                            &cur_s,
                            goal,
                            wid,
                            bounds,
                            label,
                            provenance,
                            cur_sig,
                            cur_place_trig,
                            cur_clutch_trig,
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
                        if hit && ledger_clear {
                            let (pl, full_s, full_o) = reconstruct(&arena, idx, cur_steps, cur_obs);
                            best = Some((cur_arr, pl, full_s, full_o));
                            incumbent_at.get_or_insert(raw_learn_ticks);
                            break;
                        }
                        let cb = Bucket::of(&cur_s, node_ehash, cur_ledger.open.len() as u16, cur_counts.blocks > 0);
                        if best_bucket.get(&cb).is_some_and(|&prev| prev <= cur_arr) {
                            continue;
                        }
                        let p = priority_key_topo(cur_arr, &cur_s, goal, &topo, &cur_ledger, kit);
                        let nid = arena.len();
                        arena.push(ArenaNode {
                            state: cur_s.clone(),
                            arrival: cur_arr,
                            depth: cur_depth,
                            prefix_len: pl_here,
                            parent: Some(idx),
                            edge_steps: cur_steps.clone(),
                            edge_obs: cur_obs.clone(),
                            edits: cur_edits.clone(),
                            counts: cur_counts,
                            ledger: cur_ledger.clone(),
                        });
                        heap.push(Reverse((p, cur_arr, nid)));
                    }
                }
                sig
            } else if cur_edits.is_empty() {
                DefectSig { fine: 0, coarse: 0, rot: approach_frame(&state_here, goal).0 }
            } else {
                DefectSig { fine: 0, coarse: 0, rot: approach_frame(&state_here, goal).0 }
            };

            let (gx, gz) = dir_to_goal(&state_here, goal);
            let restore_obs: Vec<BlockPos> = if restore_required {
                cur_ledger.open.iter().map(|o| o.pos).collect()
            } else {
                Vec::new()
            };
            let eye_here = law::interact::get_eye_position(state_here.position, state_here.pose.eye_height());
            let has_far_open_ob = restore_obs.iter().any(|&op| {
                !law::interact::is_within_block_interaction_range(eye_here, 4.4, op, 0.0)
            });
            let direct_blocked = state_here.horizontal_collision
                || !can_clear_probe(&delta, &state_here, gx, gz, 0.75)
                || state_here.velocity.horizontal_sqr() < 0.01
                || has_far_open_ob;
            let aims = candidate_aims(&state_here, goal, sig.rot, direct_blocked);
            let progs = programs_for(&delta, &state_here);
            let sprint_ok = sprint_allowed(&state_here);
            let kit_edge_stop = kit.is_some();

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
                        &delta,
                        Some(&grid),
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
                        cur_place_trig,
                        cur_clutch_trig,
                        &restore_obs,
                        kit_edge_stop,
                        &mut raw_learn_ticks,
                    ) {
                        AtomicOutcome::Failed => {}
                        AtomicOutcome::Stopped { end, step, hit_goal, obs } => {
                            expanded += 1;
                            let arr = arr_here.saturating_add(u64::from(step.ticks));
                            if arr >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                                continue;
                            }
                            if hit_goal && ledger_clear {
                                let (pl, full_s, full_o) = reconstruct(&arena, idx, vec![step], vec![obs]);
                                best = Some((arr, pl, full_s, full_o));
                                incumbent_at.get_or_insert(raw_learn_ticks);
                                continue;
                            }
                            if best_bucket.get(&Bucket::of(&end, node_ehash, cur_ledger.open.len() as u16, cur_counts.blocks > 0)).is_some_and(|&prev| prev <= arr) {
                                continue;
                            }
                            let p = priority_key_topo(arr, &end, goal, &topo, &cur_ledger, kit);
                            let nid = arena.len();
                            arena.push(ArenaNode {
                                state: end,
                                arrival: arr,
                                depth: depth_here + 1,
                                prefix_len: pl_here,
                                parent: Some(idx),
                                edge_steps: vec![step],
                                edge_obs: vec![obs],
                                edits: cur_edits.clone(),
                                counts: cur_counts,
                                ledger: cur_ledger.clone(),
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
                            &delta,
                            Some(&grid),
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
                            cur_place_trig,
                            cur_clutch_trig,
                            &restore_obs,
                            kit_edge_stop,
                            &mut raw_learn_ticks,
                        ) {
                            let arr = arr_here.saturating_add(u64::from(step.ticks));
                            if arr >= best.as_ref().map_or(u64::MAX, |b| b.0) {
                                continue;
                            }
                            if hit_goal && ledger_clear {
                                let (pl, full_s, full_o) = reconstruct(&arena, idx, vec![step], vec![obs]);
                                best = Some((arr, pl, full_s, full_o));
                                incumbent_at.get_or_insert(raw_learn_ticks);
                            } else {
                                if best_bucket.get(&Bucket::of(&end, node_ehash, cur_ledger.open.len() as u16, cur_counts.blocks > 0)).is_some_and(|&prev| prev <= arr) {
                                    continue;
                                }
                                let p = priority_key_topo(arr, &end, goal, &topo, &cur_ledger, kit);
                                let nid = arena.len();
                                arena.push(ArenaNode {
                                    state: end,
                                    arrival: arr,
                                    depth: depth_here + 1,
                                    prefix_len: pl_here,
                                    parent: Some(idx),
                                    edge_steps: vec![step],
                                    edge_obs: vec![obs],
                                    edits: cur_edits.clone(),
                                    counts: cur_counts,
                                    ledger: cur_ledger.clone(),
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
