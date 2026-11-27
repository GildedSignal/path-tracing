//! Propose-first raw defect crossing + raw learning on miss/entropy.
//! Never calls `RecordingWorld`; runs purely on raw `law::try_tick` against the corridor source and `TopoField`.
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};
use law::{
    Aabb, BlockPos, CollisionContext, FluidKind, PlayerState, World,
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
    kit::{FieldEdit, HandStep, KitCounts, KitOp, KitView, ObligationLedger, cell_box, digest_edits},
    solve::{Stage, ThinkingCosts, WordEvents, WordStep, extend_corridor},
    source::{CorridorProof, CorridorSource, ObservedWorld},
    state_view::StateAt,
    store::{GeneratorStore, SPRINT, SPRINT_JUMP, WALK, sprint_allowed},
    word_store::{
        AimRule, DefectSig, EntryClass, ObservedStep, WordId, WordStore, approach_frame,
        defect_sig,
    },
};

const MAX_STEPS: usize = 128;
const MAX_TOPO_CELLS: usize = 120_000;

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

/// The legs' price of one cell of the field, in ticks, read from the law's own gait for this
/// body: `h` per cell across at the gait's fixed point, and what a cell up (the jump arc until
/// it stands one higher) or down (the arc's fall of one cell from its apex) adds over the cell
/// across it overlaps (section 4: `max(T_a, T_b) - T_a`). `reach` is how many unsupported cells
/// one jump arc carries the body across: the arc's advance at its fixed point plus the body's
/// width.
#[derive(Clone, Copy)]
pub(crate) struct Legs {
    pub(crate) h: f32,
    pub(crate) up: f32,
    pub(crate) down: f32,
    pub(crate) reach: u8,
}

/// One cycle of a gait is the affine contraction `v -> a v + b` (section 10): its advance from
/// the fixed point `b / (1 - a)`.
pub(crate) fn cycle_advance(rows: &[crate::store::Phase]) -> f64 {
    let (a, b) = rows.iter().fold((1.0, 0.0), |(a, b), p| {
        let f = p.factor * p.drag;
        (a * f, (b + p.impulse + p.forward) * f)
    });
    let mut v = b / (1.0 - a);
    let mut advance = 0.0;
    for p in rows {
        let x = v + p.impulse + p.forward;
        advance += x;
        v = x * p.factor * p.drag;
    }
    advance
}

impl Legs {
    pub(crate) fn of<S: CorridorSource>(store: &GeneratorStore, label: LabelId, source: &S, from: &PlayerState) -> Option<Self> {
        let fallback = Some(Self { h: 2.81, up: 6.19, down: 0.0, reach: 4 });
        let Ok(walk) = store.gait(label, source, from, if sprint_allowed(from) { SPRINT_JUMP } else { WALK }) else { return fallback; };
        let Ok(jump) = store.gait(label, source, from, SPRINT_JUMP) else { return fallback; };
        let advance = cycle_advance(&walk.phases);
        if !(advance > 0.0) { return None; }
        let h = walk.phases.len() as f64 / advance;
        let y: Vec<f64> = jump.phases.iter().map(|p| p.after.position.y - jump.top).collect();
        let apex = y.iter().copied().fold(f64::MIN, f64::max);
        let peak = y.iter().position(|&v| v == apex)?;
        let up = peak + y[peak..].iter().position(|&v| v < 1.0)? + 1;
        let down = y[peak..].iter().position(|&v| v <= apex - 1.0)?;
        let body = Aabb::player(law::Vec3::default(), from.pose.height());
        let reach = (cycle_advance(&jump.phases) + (body.max.x - body.min.x)).floor().clamp(0.0, f64::from(u8::MAX - 1)) as u8;
        Some(Self { h: h as f32, up: (up as f64 - h).max(0.0) as f32, down: (down as f64 - h).max(0.0) as f32, reach })
    }

    /// The legs of a cell class (support, feet, eyes): the law ticked under each gait program in a
    /// box of those three layers, repeated outward (the eyes' block upward), until the body's
    /// motion repeats exactly. `h` is the fastest horizontal cycle; a cycle that rises or sinks
    /// without touching a floor (swimming) gives `up` and `down` as its rates over `h`. `None`:
    /// no program advances there (no tool applies).
    pub(crate) fn learn(class: [&law::Block; 3], from: &PlayerState) -> Option<Self> {
        struct Layers<'a>([&'a law::Block; 3]);
        impl World for Layers<'_> {
            fn block(&self, p: BlockPos) -> &law::Block { self.0[p[1].clamp(0, 2) as usize] }
            fn difficulty(&self) -> u8 { 0 }
        }
        let w = Layers(class);
        let floor = !class[0].never_collides();
        let top = class[0].shape.coords[1].last().copied().filter(|_| floor).unwrap_or(1.0);
        let start_y = if floor { top } else { 1.0 };
        let rest = || {
            let mut s = from.clone();
            s.position = law::Vec3::new(0.5, start_y, 0.5);
            s.velocity = law::Vec3::default();
            s.on_ground = floor;
            s.main_supporting_block = floor.then_some([0, 0, 0]);
            s.no_jump_delay = 0;
            s.yaw = 0.0;
            s
        };
        let input = |program: u16| law::Input { forward: true, sprint: program != WALK, jump: program == SPRINT_JUMP, ..Default::default() };
        let tick = |s: &mut PlayerState, program: u16| {
            let ok = law::try_tick(s, &w, &input(program)).is_ok();
            (s.food_level, s.saturation, s.exhaustion) = (from.food_level, from.saturation, from.exhaustion);
            ok
        };
        // A contraction reaches its fixed point bit for bit within the float's 53 bits of memory.
        const MEMORY: u32 = 4096;
        let programs: &[u16] = if sprint_allowed(from) { &[WALK, SPRINT, SPRINT_JUMP] } else { &[WALK] };
        let mut h = f64::INFINITY;
        for &program in programs {
            let mut s = rest();
            let mut seen: HashMap<(u64, u64, u64, bool, u8, u8, bool, bool), (u32, f64)> = HashMap::new();
            for t in 0..MEMORY {
                let key = (s.velocity.x.to_bits(), s.velocity.y.to_bits(), s.velocity.z.to_bits(), s.on_ground, s.no_jump_delay as u8, s.pose as u8, s.in_water, s.eye_in_water);
                if let Some(&(t0, z0)) = seen.get(&key) {
                    let dz = s.position.z - z0;
                    if dz > 0.0 { h = h.min(f64::from(t - t0) / dz); }
                    break;
                }
                seen.insert(key, (t, s.position.z));
                if !tick(&mut s, program) { break; }
            }
        }
        if !h.is_finite() { return None; }
        // A cell up or down: the ticks from rest until the body is a cell higher (rising) or lower
        // (sinking), over the cell across it overlaps; never within memory, no such edge.
        let cell = |program: u16, dy: f64| {
            let mut s = rest();
            (1..=MEMORY).find(|_| tick(&mut s, program) && (s.position.y - start_y) * dy.signum() >= dy.abs())
                .map_or(f32::INFINITY, |t| (f64::from(t) - h).max(0.0) as f32)
        };
        Some(Self { h: h as f32, up: cell(SPRINT_JUMP, 1.0), down: cell(WALK, -1.0), reach: 0 })
    }
}

const DIRS: [BlockPos; 10] = [
    [1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1], [0, 1, 0], [0, -1, 0],
    [1, 0, 1], [1, 0, -1], [-1, 0, 1], [-1, 0, -1],
];
const SEED: u8 = 10;
/// The descent's next transition needs the way cleared, a floor under the next cell (a gap
/// longer than a jump), or a floor under this one (to jump up from where nothing holds the body).
const CLEAR: u8 = 1;
const FLOOR_NEXT: u8 = 2;
const FLOOR_HERE: u8 = 3;

/// T-to-go for one goal: one backward wave over (cell, run) states, the run the unsupported
/// cells crossed since the body last stood (at most the legs' reach). Its edges are the legs
/// and, where the kit edits, the hyperedges the kit prices (`KitView::edge`). Bare legs is the
/// empty kit: the same wave with no edit edges. `next` is the descent (the neighbour the wave
/// came from); `edit` marks a state whose next transition needs the hand. Zero block names.
struct TopoField {
    min: BlockPos,
    max: BlockPos,
    dx: usize,
    dy: usize,
    runs: usize,
    dist: Vec<f32>,
    next: Vec<u8>,
    edit: Vec<u8>,
    fits: Vec<u8>,
}

/// The body's way from cell `a` into its neighbour `b`: the hull of its probe in each, so a
/// thin wall on their shared face (a closed door, a pane) blocks the way and not the cells.
#[inline]
fn way(a: BlockPos, b: BlockPos) -> Aabb {
    let lo = |i: usize| f64::from(a[i].min(b[i]));
    let hi = |i: usize| f64::from(a[i].max(b[i]));
    Aabb::new(lo(0) + 0.25, lo(1) + 0.05, lo(2) + 0.25, hi(0) + 0.75, hi(1) + 1.55, hi(2) + 0.75)
}

#[inline]
pub(crate) fn way_clear(w: &impl World, a: BlockPos, b: BlockPos) -> bool {
    let feet_y = f64::from(a[1].min(b[1]));
    w.no_collision(way(a, b), CollisionContext { feet_y, descending: false, fall_distance: 0.0, leather_boots: false })
}

#[inline]
pub(crate) fn body_fits(w: &impl World, p: BlockPos) -> bool {
    way_clear(w, p, p)
}

/// The cells whose shapes block the way from `a` into `b` (the body's two cells in each).
pub(crate) fn blockers(w: &impl World, a: BlockPos, b: BlockPos) -> List<BlockPos, 4> {
    let hull = way(a, b);
    let feet_y = f64::from(a[1].min(b[1]));
    let mut out = List::new();
    for c in [a, [a[0], a[1] + 1, a[2]], b, [b[0], b[1] + 1, b[2]]] {
        let cb = cell_box(c);
        let part = Aabb::new(
            hull.min.x.max(cb.min.x), hull.min.y.max(cb.min.y), hull.min.z.max(cb.min.z),
            hull.max.x.min(cb.max.x), hull.max.y.min(cb.max.y), hull.max.z.min(cb.max.z),
        );
        let inside = part.min.x < part.max.x && part.min.y < part.max.y && part.min.z < part.max.z;
        if inside && !out.contains(&c) && !w.block(c).never_collides()
            && !w.no_collision(part, CollisionContext { feet_y, descending: false, fall_distance: 0.0, leather_boots: false })
        {
            out.push(c);
        }
    }
    out
}

#[inline]
pub(crate) fn has_step_support(w: &impl World, p: BlockPos) -> bool {
    let b = w.block(p);
    if !b.never_collides() || b.climbable || b.fluid.kind != FluidKind::Empty {
        return true;
    }
    let b = w.block([p[0], p[1] - 1, p[2]]);
    !b.never_collides() || b.climbable
}

#[inline]
pub(crate) fn below(c: BlockPos) -> BlockPos {
    [c[0], c[1] - 1, c[2]]
}

#[derive(Clone)]
pub(crate) struct TopoCacheEntry {
    provenance: [u8; 32],
    goal: Goal,
    kit_key: u64,
    legs_bits: (u32, u32, u32, u8),
    min: BlockPos,
    max: BlockPos,
    field: Arc<TopoField>,
}

/// Small LRU cache of built `TopoField` waves across adjacent windows and retries.
#[derive(Default)]
pub(crate) struct TopoCache {
    entries: Vec<TopoCacheEntry>,
    builds: u64,
}

impl TopoCache {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn invalidate_section(&mut self, sec: BlockPos) {
        let smin = [sec[0] * 16 - 1, sec[1] * 16 - 1, sec[2] * 16 - 1];
        let smax = [sec[0] * 16 + 16, sec[1] * 16 + 16, sec[2] * 16 + 16];
        self.entries.retain(|e| {
            e.max[0] < smin[0] || e.min[0] > smax[0]
                || e.max[1] < smin[1] || e.min[1] > smax[1]
                || e.max[2] < smin[2] || e.min[2] > smax[2]
        });
    }

    pub(crate) fn export_entries(&self) -> Vec<TopoCacheEntry> {
        self.entries.clone()
    }

    pub(crate) fn import_entries(&mut self, incoming: &[TopoCacheEntry]) {
        for e in incoming {
            let dup = self.entries.iter().any(|cur| {
                cur.provenance == e.provenance
                    && cur.goal == e.goal
                    && cur.kit_key == e.kit_key
                    && cur.legs_bits == e.legs_bits
                    && cur.min == e.min
                    && cur.max == e.max
            });
            if !dup {
                if self.entries.len() >= 8 {
                    self.entries.remove(0);
                }
                self.entries.push(e.clone());
            }
        }
    }

    pub(crate) fn builds(&self) -> u64 {
        self.builds
    }
}

#[derive(Clone, Copy)]
struct Landing {
    yaw: u16,
    dist: f64,
    dy: i32,
    gap: bool,
}

impl TopoField {
    /// No field: every query is unknown and the search falls back on its distance bound.
    fn empty() -> Self {
        Self { min: [0; 3], max: [-1, 0, 0], dx: 0, dy: 0, runs: 1, dist: Vec::new(), next: Vec::new(), edit: Vec::new(), fits: Vec::new() }
    }

    fn bounds_for(bounds: Aabb, from: &PlayerState, goal: Goal, kit: Option<&KitView>) -> (BlockPos, BlockPos) {
        let min = [bounds.min.x, bounds.min.y, bounds.min.z].map(|v| v.floor() as i32 - 2);
        let max = [bounds.max.x, bounds.max.y, bounds.max.z].map(|v| v.ceil() as i32 + 2);
        let fx = from.position.x.floor() as i32;
        let fy = from.position.y.floor() as i32;
        let fz = from.position.z.floor() as i32;
        // Downward the field reaches as far as the body can fall and live: the law's safe fall
        // plus its health, and without end under a kit that resets falls.
        let cushioned = kit.is_some_and(|kv| kv.makes(FieldEdit::Cushion, kv.spec.counts()));
        let depth = if cushioned { i32::MAX } else { (from.profile.safe_fall_distance + f64::from(from.health)).ceil() as i32 };
        let min = [
            fx.min(goal.cell[0]).saturating_sub(20).max(min[0]),
            fy.min(goal.cell[1]).saturating_sub(depth).max(min[1]),
            fz.min(goal.cell[2]).saturating_sub(20).max(min[2]),
        ];
        let max = [
            fx.max(goal.cell[0]).saturating_add(20).min(max[0]),
            fy.max(goal.cell[1]).saturating_add(8).min(max[1]),
            fz.max(goal.cell[2]).saturating_add(20).min(max[2]),
        ];
        (min, max)
    }

    fn get_or_build<S: World>(
        store: &GeneratorStore,
        label: LabelId,
        provenance: [u8; 32],
        kit_key: u64,
        grid: &ObservedWorld<'_, S>,
        bounds: Aabb,
        from: &PlayerState,
        goal: Goal,
        kit: Option<&KitView>,
        legs: Legs,
    ) -> Arc<Self> {
        let (req_min, req_max) = Self::bounds_for(bounds, from, goal, kit);
        let fy = from.position.y.floor() as i32;
        let legs_bits = (legs.h.to_bits(), legs.up.to_bits(), legs.down.to_bits(), legs.reach);
        let hit_pos = store.topo.borrow().entries.iter().position(|e| {
            let y_ok = (e.min[1] <= req_min[1] && e.max[1] >= req_max[1])
                || (e.min[1] <= fy.min(goal.cell[1]) - 2 && e.max[1] >= fy.max(goal.cell[1]) + 4);
            e.provenance == provenance
                && e.goal == goal
                && e.kit_key == kit_key
                && e.legs_bits == legs_bits
                && e.min[0] <= req_min[0] && e.max[0] >= req_max[0]
                && y_ok
                && e.min[2] <= req_min[2] && e.max[2] >= req_max[2]
                && e.field.query(from).is_finite()
        });
        if let Some(pos) = hit_pos {
            let mut cache = store.topo.borrow_mut();
            let entry = cache.entries.remove(pos);
            let field = Arc::clone(&entry.field);
            cache.entries.push(entry);
            return field;
        }
        let (min, max) = (req_min, req_max);
        let field = Arc::new(Self::build_box(store, label, grid, min, max, from, goal, kit, legs, true));
        if !field.dist.is_empty() {
            let mut cache = store.topo.borrow_mut();
            cache.builds += 1;
            if cache.entries.len() >= 8 {
                cache.entries.remove(0);
            }
            cache.entries.push(TopoCacheEntry {
                provenance,
                goal,
                kit_key,
                legs_bits,
                min: field.min,
                max: field.max,
                field: Arc::clone(&field),
            });
        }
        field
    }

    fn build_box<S: World>(
        store: &GeneratorStore,
        label: LabelId,
        grid: &ObservedWorld<'_, S>,
        min: BlockPos,
        max: BlockPos,
        from: &PlayerState,
        goal: Goal,
        kit: Option<&KitView>,
        legs: Legs,
        stop_at_from: bool,
    ) -> Self {
        let fx = from.position.x.floor() as i32;
        let fy = from.position.y.floor() as i32;
        let fz = from.position.z.floor() as i32;
        let dx = (max[0] - min[0] + 1).max(1) as usize;
        let dy = (max[1] - min[1] + 1).max(1) as usize;
        let dz = (max[2] - min[2] + 1).max(1) as usize;
        let vol = dx.saturating_mul(dy).saturating_mul(dz);
        if vol > MAX_TOPO_CELLS {
            return Self::empty();
        }
        let runs = usize::from(legs.reach) + 1;
        let states = vol * runs;
        let mut field = Self { min, max, dx, dy, runs, dist: vec![f32::INFINITY; states], next: vec![SEED; states], edit: vec![0; states], fits: vec![0u8; vol] };
        // Edits on the best path: of two equal T, the wave keeps the one with fewer.
        let mut edits = vec![0u8; states];
        let mut heap: BinaryHeap<Reverse<(u32, u8, BlockPos, u8)>> = BinaryHeap::new();
        // Seed cells whose body at y touches the goal, plus ceiling-underneath support for Face::Down.
        let height = from.pose.height();
        let [gx, gy, gz] = goal.cell;
        let mut seed_cell = |p: BlockPos| {
            let Some(ci) = field.idx(p) else { return };
            let fits = body_fits(grid, p);
            grid.reset_missing();
            if fits {
                field.fits[ci] = 2;
                for r in 0..runs {
                    field.dist[ci * runs + r] = 0.0;
                    heap.push(Reverse((0, 0, p, r as u8)));
                }
            }
        };
        for y in gy - 1..=gy + 1 {
            for z in gz - 1..=gz + 1 {
                for x in gx - 1..=gx + 1 {
                    if (x - gx).abs() + (z - gz).abs() > 1 { continue; }
                    let corner = |d: f64| Aabb::player(law::Vec3::new(f64::from(x) + d, f64::from(y), f64::from(z) + d), height);
                    if goal.touched_boxes(corner(0.0), corner(1.0)) { seed_cell([x, y, z]); }
                }
            }
        }
        if goal.face.is_none() || goal.face == Some(crate::Face::Down) {
            for z in gz - 1..=gz + 1 {
                for x in gx - 1..=gx + 1 {
                    if (x - gx).abs() + (z - gz).abs() > 1 { continue; }
                    let p = [x, gy - 2, z];
                    let sup = has_step_support(grid, p);
                    grid.reset_missing();
                    if sup { seed_cell(p); }
                }
            }
        }
        let priced = |cells: &[BlockPos], edit: FieldEdit| kit.and_then(|kv| kv.edge(grid, cells, edit));
        // Stop once the body's starting state is settled with a 25-tick margin; cache support & edge checks across runs.
        let from_cell = if body_fits(grid, [fx, fy, fz]) { [fx, fy, fz] } else { [fx, fy + 1, fz] };
        grid.reset_missing();
        let from_ci = stop_at_from.then(|| field.idx(from_cell)).flatten();
        let from_needs_r0 = has_step_support(grid, from_cell);
        grid.reset_missing();
        let mut from_settled_d: Option<f32> = None;
        let mut sup_cache = vec![0u8; vol];
        let mut edge_checked = vec![0u16; vol];
        let mut edge_open = vec![0u16; vol];
        let mut settled = vec![false; vol];
        // The legs of the cell entered: the ground's, unless a body cell there is wet or stuck,
        // where its class (support, feet, eyes) answers with its own learned legs.
        let mut class_of = vec![0u8; vol];
        let mut class_legs: Vec<Option<Legs>> = Vec::new();
        let mut class_ids: HashMap<[*const law::Block; 3], u8> = HashMap::new();
        let mut legs_at = |ci: usize, p: BlockPos| -> Option<Legs> {
            if class_of[ci] == 0 {
                let up = [p[0], p[1] + 1, p[2]];
                let (feet, eyes) = (grid.block(p), grid.block(up));
                class_of[ci] = if !feet.has_inside_effects() && !eyes.has_inside_effects() { 1 } else {
                    let class = [grid.block(below(p)), feet, eyes];
                    let ptrs = class.map(|b| b as *const law::Block);
                    *class_ids.entry(ptrs).or_insert_with(|| {
                        let key: Vec<u8> = class.iter().flat_map(|b| law::codec::legs_bytes(b, true, law::codec::LegsRole::BodyFeet).unwrap_or_default()).collect();
                        let learned = *store.classes.borrow_mut().entry((label, key)).or_insert_with(|| Legs::learn(class, from));
                        class_legs.push(learned);
                        (class_legs.len() + 1).min(usize::from(u8::MAX)) as u8
                    })
                };
                grid.reset_missing();
            }
            match class_of[ci] { 1 => Some(legs), k => class_legs[usize::from(k) - 2] }
        };
        let mut pops = 0usize;
        while let Some(Reverse((bits, e, cur, rc))) = heap.pop() {
            let cur_d = f32::from_bits(bits);
            if pops >= 40_000 || (stop_at_from && from_settled_d.is_some_and(|fd| cur_d > fd + 25.0)) {
                break;
            }
            let Some(ci) = field.idx(cur) else { continue };
            let si = ci * runs + usize::from(rc);
            if cur_d != field.dist[si] || e != edits[si] {
                continue;
            }
            if !std::mem::replace(&mut settled[ci], true) {
                pops += 1;
            }
            if Some(ci) == from_ci && (!from_needs_r0 || rc == 0) {
                from_settled_d.get_or_insert(cur_d);
            }
            let cur_sup = match sup_cache[ci] {
                1 => false,
                2 => true,
                _ => {
                    let s = has_step_support(grid, cur);
                    grid.reset_missing();
                    sup_cache[ci] = if s { 2 } else { 1 };
                    s
                }
            };
            for (k, d) in DIRS.iter().enumerate() {
                let n = [cur[0] + d[0], cur[1] + d[1], cur[2] + d[2]];
                let Some(ni) = field.idx(n) else { continue };
                // The move happens in the medium the body leaves: the class of `n` prices it, and
                // a class where no tool applies is left by no edge.
                let Some(lg) = legs_at(ni, n) else { continue };
                // The body moves from `n` to `cur`: up when `n` is below. Its run there is 0 on a
                // floor or after a jump up or a fall, and one more for each unsupported cell across.
                let up = n[1] < cur[1];
                let across = n[1] == cur[1];
                let diag = k >= 6;
                let leg = cur_d + if up { lg.up } else if diag { lg.h * std::f32::consts::SQRT_2 } else if across { lg.h } else { lg.down };
                if (0..runs).all(|r| field.dist[ni * runs + r] < leg) {
                    continue;
                }
                let n_sup = match sup_cache[ni] {
                    1 => false,
                    2 => true,
                    _ => {
                        let s = has_step_support(grid, n);
                        grid.reset_missing();
                        sup_cache[ni] = if s { 2 } else { 1 };
                        s
                    }
                };
                let bit = 1u16 << k;
                let open = if (edge_checked[ni] & bit) != 0 {
                    (edge_open[ni] & bit) != 0
                } else {
                    let o = way_clear(grid, n, cur);
                    grid.reset_missing();
                    edge_checked[ni] |= bit;
                    if o { edge_open[ni] |= bit; }
                    o
                };
                if diag && !open { continue; }
                // The edit the way itself needs, and its price.
                let clear = if open {
                    Some((0.0, 0))
                } else {
                    let cells = blockers(grid, n, cur);
                    let t = priced(&cells, FieldEdit::Clear);
                    grid.reset_missing();
                    t.map(|(t, _)| (t, CLEAR))
                };
                let Some((clear_t, clear_mark)) = clear else { continue };
                if open {
                    field.fits[ni] = 2;
                } else if field.fits[ni] == 0 {
                    field.fits[ni] = if body_fits(grid, n) { 2 } else { 1 };
                    grid.reset_missing();
                }
                // The runs at `n` that lead to (cur, rc), each with what the legs alone cannot do.
                let mut options: List<(usize, f32, u8), 6> = List::new();
                let all_runs = if n_sup { 0..1 } else { 0..runs };
                if across && !cur_sup {
                    if rc >= 1 && (!n_sup || rc == 1) {
                        options.push((usize::from(rc) - 1, 0.0, 0));
                    }
                    if !diag && rc == 0 && !has_step_support(grid, below(cur)) {
                        if let Some((t, anchored)) = priced(&[below(cur)], FieldEdit::Floor) {
                            let r_range = if anchored { all_runs.clone() } else { 0..1 };
                            r_range.for_each(|r| options.push((r, t, FLOOR_NEXT)));
                        }
                    }
                    grid.reset_missing();
                } else if !up && !across && !cur_sup {
                    // Falling through empty air never resets horizontal air run `rc`.
                    if !n_sup {
                        options.push((usize::from(rc), 0.0, 0));
                    } else if rc == 0 {
                        options.push((0, 0.0, 0));
                    }
                } else if rc == 0 {
                    if up && !n_sup {
                        if let Some((t, _)) = priced(&[below(n)], FieldEdit::Floor) {
                            (0..runs).for_each(|r| options.push((r, t, FLOOR_HERE)));
                        }
                        grid.reset_missing();
                    } else {
                        all_runs.for_each(|r| options.push((r, 0.0, 0)));
                    }
                }
                for (rn, t, floor_mark) in options {
                    if clear_mark != 0 && floor_mark != 0 {
                        continue;
                    }
                    let mark = clear_mark | floor_mark;
                    let nd = leg + clear_t + t;
                    let ne = e.saturating_add(u8::from(mark != 0));
                    let ns = ni * runs + rn;
                    if nd < field.dist[ns] || nd == field.dist[ns] && ne < edits[ns] {
                        field.dist[ns] = nd;
                        edits[ns] = ne;
                        field.next[ns] = k as u8;
                        field.edit[ns] = mark;
                        heap.push(Reverse((nd.to_bits(), ne, n, rn as u8)));
                    }
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
    fn pos(&self, i: usize) -> BlockPos {
        let ix = i % self.dx;
        let iy = (i / self.dx) % self.dy;
        let iz = i / (self.dx * self.dy);
        [self.min[0] + ix as i32, self.min[1] + iy as i32, self.min[2] + iz as i32]
    }

    /// The cell's best state: the run that leaves the body the most of the field.
    fn best(&self, ci: usize) -> usize {
        (ci * self.runs..(ci + 1) * self.runs).min_by(|&a, &b| self.dist[a].total_cmp(&self.dist[b])).unwrap_or(ci * self.runs)
    }

    /// The field state the body is in: its feet cell when the body fits there, else the cell
    /// above (partial-block feet cell like slab/fence or unreachable `fy`), else a corner's.
    fn state_of(&self, s: &PlayerState) -> Option<usize> {
        if self.dist.is_empty() {
            return None;
        }
        let fx = s.position.x.floor() as i32;
        let fy = (s.position.y + 0.05).floor() as i32;
        let fz = s.position.z.floor() as i32;
        let at = |p: BlockPos| {
            let ci = self.idx(p)?;
            let i = self.best(ci);
            (self.dist[i] < f32::INFINITY).then_some((ci, i))
        };
        if let Some((ci, i)) = at([fx, fy, fz]) {
            if self.fits[ci] == 2 {
                return Some(i);
            }
        }
        if let Some((_, i)) = at([fx, fy + 1, fz]) {
            return Some(i);
        }
        if let Some((_, i)) = at([fx, fy, fz]) {
            return Some(i);
        }
        let best = |cells: &mut dyn Iterator<Item = BlockPos>| {
            cells.filter_map(|p| self.idx(p)).map(|ci| self.best(ci)).filter(|&i| self.dist[i] < f32::INFINITY)
                .min_by(|&a, &b| self.dist[a].total_cmp(&self.dist[b]))
        };
        best(&mut [-0.24, 0.24].into_iter().flat_map(|ox| {
            [-0.24, 0.24].into_iter().map(move |oz| [(s.position.x + ox).floor() as i32, fy, (s.position.z + oz).floor() as i32])
        }))
    }

    #[inline]
    fn query(&self, s: &PlayerState) -> f32 {
        self.state_of(s).map_or(f32::INFINITY, |i| self.dist[i])
    }

    fn query_with_edits(&self, s: &PlayerState, edits: &[(BlockPos, StateId)], ledger: &ObligationLedger) -> f32 {
        if edits.is_empty() || ledger.open.is_empty() {
            return self.query(s);
        }
        let Some(mut si) = self.state_of(s) else { return f32::INFINITY };
        let base_d = self.dist[si];
        if !base_d.is_finite() {
            return f32::INFINITY;
        }
        let mut credit = 0.0f32;
        for _ in 0..6 {
            let Some((n, next)) = self.step(si) else { break };
            let Some(nci) = self.idx(next) else { break };
            let next_si = self.best(nci);
            let edge_drop = (self.dist[si] - self.dist[next_si] - 3.5).max(0.0);
            if self.edit[si] == CLEAR {
                let in_passage = |p: BlockPos| {
                    (p[0] == next[0] && p[2] == next[2] && (p[1] == next[1] || p[1] == next[1] + 1))
                        || (next[1] > n[1] && p == [n[0], next[1] + 1, n[2]])
                        || (next[1] < n[1] && p == [next[0], n[1] + 1, next[2]])
                };
                let step_credit: f32 = ledger
                    .open
                    .iter()
                    .filter(|o| {
                        o.clear_ticks_paid >= 10
                            && in_passage(o.pos)
                            && edits.iter().any(|&(ep, es)| ep == o.pos && es.model_block().never_collides())
                    })
                    .map(|o| f32::from(o.clear_ticks_paid))
                    .sum();
                credit += step_credit.min(edge_drop);
            } else if self.edit[si] == FLOOR_HERE || self.edit[si] == FLOOR_NEXT {
                let fcell = if self.edit[si] == FLOOR_HERE { below(n) } else { below(next) };
                if ledger.open.iter().any(|o| o.pos == fcell && o.undo_ticks_lb >= 10)
                    && edits.iter().any(|&(ep, es)| ep == fcell && !es.model_block().never_collides())
                {
                    credit += edge_drop;
                }
            }
            if self.dist[next_si] >= self.dist[si] {
                break;
            }
            si = next_si;
        }
        let mut best_d = (base_d - credit).max(0.0);
        let fx = s.position.x.floor() as i32;
        let fz = s.position.z.floor() as i32;
        for o in &ledger.open {
            if o.clear_ticks_paid >= 10
                && (o.pos[0] - fx).abs() <= 1
                && (o.pos[2] - fz).abs() <= 1
                && edits.iter().any(|&(ep, es)| ep == o.pos && es.model_block().never_collides())
            {
                for cand in [o.pos, [o.pos[0], o.pos[1] - 1, o.pos[2]]] {
                    if let Some(ci) = self.idx(cand) {
                        let cd = self.dist[self.best(ci)];
                        if cd.is_finite() {
                            best_d = best_d.min(cd + 6.0);
                        }
                    }
                }
            }
        }
        best_d
    }

    /// The cell of state `i` and the cell its descent enters next.
    fn step(&self, i: usize) -> Option<(BlockPos, BlockPos)> {
        let n = self.pos(i / self.runs);
        let k = self.next[i];
        (k != SEED).then(|| {
            let d = DIRS[usize::from(k)];
            (n, [n[0] - d[0], n[1] - d[1], n[2] - d[2]])
        })
    }

    /// Landing bearing, horizontal distance, vertical delta, and gap flag along the field's descent.
    fn landing(&self, grid: &impl World, s: &PlayerState) -> Option<Landing> {
        let mut si = self.state_of(s)?;
        let start = self.pos(si / self.runs);
        let mut target = None;
        let mut saw_gap = false;
        for _ in 0..8 {
            let Some((_, next_pos)) = self.step(si) else { break };
            if next_pos[0] != start[0] || next_pos[2] != start[2] {
                target = Some(next_pos);
            }
            let sup = has_step_support(grid, next_pos);
            if !sup {
                saw_gap = true;
            }
            if sup && target.is_some() {
                break;
            }
            let Some(nci) = self.idx(next_pos) else { break };
            let next_si = self.best(nci);
            if self.dist[next_si] >= self.dist[si] {
                break;
            }
            si = next_si;
        }
        let t = target?;
        let (dx, dz) = (f64::from(t[0]) + 0.5 - s.position.x, f64::from(t[2]) + 0.5 - s.position.z);
        let dist2 = dx * dx + dz * dz;
        (dist2 > 1e-6).then(|| Landing {
            yaw: crate::lattice::bracket(dx, dz)[0],
            dist: dist2.sqrt(),
            dy: t[1] - start[1],
            gap: saw_gap,
        })
    }

    /// Whether the way out of the body's state `i` is still blocked for an edit `kv` can make:
    /// the event that ends a moving word, so the hand acts where the body arrives (a floor's
    /// moment is the kit's own mid-air window).
    fn needs(&self, w: &impl World, i: usize, kv: &KitView, counts: KitCounts) -> bool {
        self.edit[i] == CLEAR && kv.makes(FieldEdit::Clear, counts) && self.step(i).is_some_and(|(n, next)| !blockers(w, n, next).is_empty())
    }

    /// The edits the field's descent needs at its next transition from the node, inside reach:
    /// the cushion a falling body's landing needs where the fall would hurt, and the way out of
    /// the body's state or the floor it marks. An edit further on is offered when the body
    /// gets there.
    fn hands<S: World>(
        &self,
        kv: &KitView,
        delta: &WorldDelta<'_, ObservedWorld<'_, S>>,
        grid: &ObservedWorld<'_, S>,
        node: &ArenaNode,
        goal: Goal,
    ) -> List<HandStep, 4> {
        let s = &node.state;
        let step = |cells: &[BlockPos], edit: FieldEdit| kv.hand_step(delta, s, cells, edit, node.counts, &node.ledger, &node.edits);
        let mut out = List::new();
        if !s.on_ground && s.velocity.y < 0.05 {
            let (x, z) = (s.position.x.floor() as i32, s.position.z.floor() as i32);
            if s.velocity.y < 0.0 {
                let mut y = s.position.y.floor() as i32;
                while y > self.min[1] && !has_step_support(delta, [x, y, z]) {
                    y -= 1;
                }
                let wet = delta.block([x, y, z]).fluid.kind != FluidKind::Empty;
                grid.reset_missing();
                if !wet && s.fall_distance + (s.position.y - f64::from(y)) > s.profile.safe_fall_distance {
                    out.extend(step(&[[x, y, z]], FieldEdit::Cushion));
                }
            }
            // Where the falling body lands next over a gap, or pillars toward a higher goal.
            let fy = (s.position.y - 0.99).floor() as i32;
            let ahead = |v: f64| if v > 0.05 { 1 } else if v < -0.05 { -1 } else { 0 };
            let under = [x, fy, z];
            if (fy <= goal.cell[1] || !has_step_support(delta, under)) && !out.iter().any(|h| h.op.pos() == under) {
                out.extend(step(&[under], FieldEdit::Floor));
            }
            let next_c = [x + ahead(s.velocity.x), fy, z + ahead(s.velocity.z)];
            if !has_step_support(delta, next_c) && !out.iter().any(|h| h.op.pos() == next_c) {
                out.extend(step(&[next_c], FieldEdit::Floor));
            }
            grid.reset_missing();
        }
        let Some(i) = self.state_of(s) else { return out };
        let Some((n, next)) = self.step(i) else { return out };
        let hand = match self.edit[i] {
            CLEAR => {
                let cells = blockers(delta, n, next);
                grid.reset_missing();
                step(&cells, FieldEdit::Clear)
            }
            FLOOR_NEXT => step(&[below(next)], FieldEdit::Floor),
            FLOOR_HERE => step(&[below(n)], FieldEdit::Floor),
            _ => None,
        };
        out.extend(hand.filter(|h| !out.iter().any(|o| o.op.pos() == h.op.pos())));
        if self.edit[i] != CLEAR && kv.makes(FieldEdit::Clear, node.counts) {
            let (_, fwd, _) = approach_frame(s, goal);
            for ahead in [[n[0] + fwd[0], n[1], n[2] + fwd[1]], [n[0] + fwd[0], n[1] + 1, n[2] + fwd[1]]] {
                if !way_clear(delta, n, ahead) {
                    let cells = blockers(delta, n, ahead);
                    grid.reset_missing();
                    if let Some(h) = step(&cells, FieldEdit::Clear) {
                        if !out.iter().any(|o| o.op.pos() == h.op.pos()) && out.len() < 4 {
                            out.push(h);
                        }
                    }
                }
            }
        }
        out
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
    pending_drops: u16,
}

impl Bucket {
    #[inline]
    fn of(s: &PlayerState, edits_hash: u64, open_ob: u16, has_blocks: bool, pending_drops: u16) -> Self {
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
            pending_drops,
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

impl ArenaNode {
    fn seed(state: PlayerState, arrival: u64, edge_steps: Vec<WordStep>, counts: KitCounts) -> Self {
        let prefix_len = edge_steps.len();
        Self {
            state, arrival, depth: prefix_len, prefix_len, parent: None, edge_steps,
            edge_obs: Vec::new(), edits: Vec::new(), counts, ledger: ObligationLedger::default(),
        }
    }
}

fn reconstruct(arena: &[ArenaNode], leaf: usize, extra_steps: Vec<WordStep>, extra_obs: Vec<ObservedStep>) -> (usize, Vec<WordStep>, Vec<ObservedStep>) {
    let mut chain = Vec::new();
    let mut cur = Some(leaf);
    while let Some(idx) = cur { chain.push(idx); cur = arena[idx].parent; }
    chain.reverse();
    let (mut steps, mut obs) = (Vec::new(), Vec::new());
    for idx in chain { steps.extend(arena[idx].edge_steps.iter().cloned()); obs.extend(arena[idx].edge_obs.iter().cloned()); }
    steps.extend(extra_steps);
    obs.extend(extra_obs);
    (arena[leaf].prefix_len, steps, obs)
}

fn bounds_aabb<S: CorridorSource>(source: &S, from: &PlayerState, goal: Goal, win: &cut::Window) -> Aabb {
    let (mut lo, mut hi) = (
        [from.position.x.floor() as i32 - 20, from.position.y.floor() as i32 - 8, from.position.z.floor() as i32 - 20],
        [from.position.x.floor() as i32 + 20, from.position.y.floor() as i32 + 20, from.position.z.floor() as i32 + 20],
    );
    for p in [goal.cell, win.first, win.bounds.0, win.bounds.1] {
        for a in [0, 2] { lo[a] = lo[a].min(p[a] - 20); hi[a] = hi[a].max(p[a] + 20); }
        lo[1] = lo[1].min(p[1] - 6);
        hi[1] = hi[1].max(p[1] + 12);
    }
    if let Some((bmin, bmax)) = source.bounds_box() {
        for a in [0, 2] { lo[a] = lo[a].max(bmin[a]); hi[a] = hi[a].min(bmax[a]); }
        lo[1] = bmin[1];
        hi[1] = bmax[1];
    }
    Aabb::new(f64::from(lo[0]), f64::from(lo[1]), f64::from(lo[2]), f64::from(hi[0]) + 1.0, f64::from(hi[1]) + 1.0, f64::from(hi[2]) + 1.0)
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

#[derive(Clone, Copy)]
struct Hand<'a> {
    kv: &'a KitView,
    counts: KitCounts,
    field: Option<&'a TopoField>,
    next_aim: Option<AimRule>,
}

enum AtomicOutcome {
    Stopped { end: PlayerState, step: WordStep, hit_goal: bool, obs: ObservedStep },
    Failed,
}

#[allow(clippy::too_many_arguments)]
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
    hand: Option<Hand<'_>>,
    restore_obs: &[BlockPos],
    kit_edge_stop: bool,
    arrival: u64,
    mut ledger: Option<&mut ObligationLedger>,
    drop_wait_ticks: u32,
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
    let mut last_cell = hand.and_then(|h| h.field).and_then(|f| f.state_of(start));
    let cell_of = |p: &PlayerState| [p.position.x.floor() as i32, (p.position.y + 0.05).floor() as i32, p.position.z.floor() as i32];
    let mut last_pos_cell = cell_of(start);

    let cap = max_ticks
        .min(if steering && s.on_ground && !s.on_climbable && !s.in_water { 24 } else { 80 })
        .max(drop_wait_ticks);
    let mut ticks = 0u32;
    let mut hit_goal = false;
    let mut any_drop_collected = false;

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
        let holding_for_drop = program_id == words::program::P_BRAKE && drop_wait_ticks > 0 && tick < drop_wait_ticks;
        let collected_now = ledger
            .as_deref_mut()
            .is_some_and(|l| l.observe_pickup(&s, arrival.saturating_add(u64::from(tick))));
        if collected_now {
            any_drop_collected = true;
        }
        if goal.reached_boxes(prev_box, &s) {
            hit_goal = true;
            break;
        }
        let landed = !prev_ground && s.on_ground;
        let stepped = prev_ground && s.on_ground && s.position.y != prev_pos.y;
        let hit_wall = s.on_ground && s.horizontal_collision && !prev_hcoll;
        let stalled = !holding_for_drop && s.on_ground && s.movement_this_tick.is_none();
        let stuck_on_ground = !holding_for_drop
            && s.on_ground
            && prev_ground
            && (s.position.y - prev_pos.y).abs() < 1e-6
            && s.position.minus(prev_pos).horizontal_sqr() < 1e-6
            && tick >= 2;
        let cur_can_fall = fast_can_fall(world, &s);
        let support_ends = cur_can_fall && !prev_can_fall;
        prev_can_fall = cur_can_fall;
        let medium_changed = s.on_climbable != prev_climbable || s.in_water != prev_in_water;

        let mut kit_window = hand.is_some_and(|h| h.kv.window(world, &s, tick, h.counts)) || collected_now;
        if let Some((h, f)) = hand.and_then(|h| h.field.map(|f| (h, f))) {
            let c = f.state_of(&s);
            if c != last_cell {
                last_cell = c;
                kit_window |= c.is_some_and(|i| f.needs(world, i, h.kv, h.counts));
            }
        }
        if let Some((h, next_aim)) = hand.and_then(|h| h.next_aim.map(|a| (h, a))) {
            let cur_cell = cell_of(&s);
            if cur_cell != last_pos_cell {
                last_pos_cell = cur_cell;
                if let Some(tc) = h.kv.click_block_target(world, &s, goal, next_aim) {
                    if !way_clear(world, cur_cell, [tc[0], cur_cell[1], tc[2]]) {
                        kit_window = true;
                    }
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

        // Abort hopeless midair stall or uncushioned cliff fall past both start and goal.
        if !s.on_ground && s.horizontal_collision && s.velocity.horizontal_sqr() < 1e-6 && s.velocity.y < -0.1 && tick > 4 {
            return AtomicOutcome::Failed;
        }
        if !s.on_ground && s.velocity.y < -0.5 && !s.in_water && !s.in_lava
            && s.position.y < start.position.y - 4.0 && s.position.y < goal.zone().min.y - 2.0
            && !hand.is_some_and(|h| h.kv.makes(FieldEdit::Cushion, h.counts))
        {
            return AtomicOutcome::Failed;
        }

        let braked = !holding_for_drop && program_id == words::program::P_BRAKE && s.on_ground && s.velocity.horizontal_sqr() <= 0.01;
        if landed || stepped || hit_wall || stalled || stuck_on_ground || support_ends || medium_changed || opening_cleared || kit_window || braked {
            break;
        }
    }

    if ticks == 0 {
        return AtomicOutcome::Failed;
    }
    if s.position == start.position && s.velocity == start.velocity && s.pose == start.pose && !hit_goal && !any_drop_collected {
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
    if state.crouching || state.in_water || state.in_lava || goal.settle || state.pose != PlayerState::default().pose || !state.on_ground || goal.cell[1] != source.floor_y(state) {
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
    hand: Option<Hand<'_>>,
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
            hand,
            &[],
            false,
            0,
            None,
            0,
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
    edits: &[(BlockPos, StateId)],
    ledger: &ObligationLedger,
    kit: Option<&KitView>,
) -> u64 {
    let d = topo.query_with_edits(s, edits, ledger);
    let mut key = arrival as f64 + if d < f32::INFINITY { f64::from(d) * 1.35 } else { 500.0 + euclid_h(s, goal) as f64 };
    if let Some(kv) = kit {
        if !ledger.open.is_empty() {
            key += f64::from(ledger.open_debt_lb(s, goal, arrival, &kv.cone));
        }
    }
    key.ceil() as u64
}

fn candidate_aims(s: &PlayerState, goal: Goal, rot: u8, direct_blocked: bool, landing: Option<Landing>) -> List<AimRule, 12> {
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
    let mut far_from_goal = false;
    if let Some(l) = landing {
        let base = AimRule::GoalOffset(0).resolve(s, goal, rot);
        let delta = l.yaw.wrapping_sub(base) as i16;
        far_from_goal = delta.unsigned_abs() > 8192;
        push_unique(AimRule::GoalOffset(delta));
        push_unique(AimRule::GoalOffset(delta.wrapping_add(4096)));
        push_unique(AimRule::GoalOffset(delta.wrapping_sub(4096)));
    }
    let tight = direct_blocked || far_from_goal || landing.is_none_or(|l| l.gap || l.dist <= 3.0);
    for &rule in &CORE[..if tight { 8 } else { 6 }] {
        push_unique(rule);
    }
    if direct_blocked || far_from_goal {
        for &rule in &ESCAPE {
            push_unique(rule);
        }
    }
    out
}

fn programs_for<W: World>(world: &W, s: &PlayerState, goal: Goal, landing: Option<Landing>, direct_blocked: bool, wait_for_drop: bool) -> List<u16, 8> {
    let mut out = List::new();
    if s.in_water || s.in_lava || s.on_climbable {
        for &p in &[P_FSJ, P_FJ, P_FS, P_F] {
            out.push(p);
        }
        if (goal.settle && s.on_ground && goal.gap(s) <= 2.5) || (wait_for_drop && s.on_ground) {
            out.push(words::program::P_BRAKE);
        }
        return out;
    }
    let sprint_ok = sprint_allowed(s);
    let low_headroom = !can_clear_shift_y(world, s, 0.35);
    let on_ledge = s.on_ground && fast_can_fall(world, s);
    let tight = direct_blocked || on_ledge || landing.is_none_or(|l| l.gap || l.dy > 0 || l.dist <= 4.2);
    if !s.on_ground {
        // Mid-air: OnGround jump never fires before `landed` breaks exec_atomic_raw, so FSG==FS and FG==F.
        if sprint_ok {
            out.push(P_FS);
        }
        out.push(P_F);
    } else if sprint_ok {
        out.push(P_FSG);
        out.push(P_FS);
        if tight {
            out.push(P_FG);
        }
        if direct_blocked || on_ledge || goal.settle {
            out.push(P_F);
        }
    } else {
        out.push(P_FG);
        out.push(P_F);
    }
    if s.crouching || low_headroom || (on_ledge && landing.is_some_and(|l| l.dy < 0 && !l.gap)) {
        out.push(P_FC);
    }
    if s.crouching {
        out.push(P_FCJ);
    }
    if (goal.settle && s.on_ground && goal.gap(s) <= 2.5) || (wait_for_drop && s.on_ground) {
        out.push(words::program::P_BRAKE);
    }
    out
}

#[allow(clippy::too_many_arguments)]
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
    start_arrival: u64,
    ledger: &mut ObligationLedger,
    restore_required: bool,
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
        ledger.observe_pickup(&s, start_arrival.saturating_add(u64::from(actual)));
        if s.health <= 0.0 || !s.bounding_box().intersects(bounds) {
            delta.rollback(mark);
            return None;
        }
        if goal.reached_boxes(prev, &s) {
            if t < land {
                if restore_required {
                    delta.rollback(mark);
                    return None;
                }
            } else {
                hit_goal = true;
                if !still {
                    break;
                }
            }
        }
        if op.settled(&s) {
            break;
        }
    }
    if still && goal.reached(&s, &s) {
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

/// The word the body came in on at `node`: the hand acts while the body keeps it (section 4's
/// parallel channels); from rest, the held phase of the corridor's gait (as `extend` holds it).
fn carried(node: &ArenaNode, goal: Goal) -> (u16, u16) {
    let s = &node.state;
    let program = node.edge_steps.last().map_or(if sprint_allowed(s) { SPRINT } else { WALK }, |w| w.program);
    let aim = node.edge_obs.last().map_or(AimRule::GoalOffset(0), |o| o.aim);
    (program, aim.resolve(s, goal, approach_frame(s, goal).0))
}

struct PhaseB<'a> {
    goal: Goal,
    topo: &'a TopoField,
    kit: Option<&'a KitView>,
    arena: Vec<ArenaNode>,
    best_bucket: HashMap<Bucket, u64>,
    heap: BinaryHeap<Reverse<(u64, u64, usize)>>,
    best: &'a mut Option<(u64, usize, Vec<WordStep>, Vec<ObservedStep>)>,
    incumbent_at: Option<u64>,
}

impl PhaseB<'_> {
    #[allow(clippy::too_many_arguments)]
    fn push_or_win(
        &mut self,
        end: PlayerState,
        arr: u64,
        depth: usize,
        parent: usize,
        steps: Vec<WordStep>,
        obs: Vec<ObservedStep>,
        edits: Vec<(BlockPos, StateId)>,
        counts: KitCounts,
        ledger: ObligationLedger,
        ehash: u64,
        hit: bool,
        ledger_clear: bool,
        learn_ticks: u64,
    ) -> bool {
        if arr >= self.best.as_ref().map_or(u64::MAX, |b| b.0) { return false; }
        if hit {
            if ledger_clear {
                let (pl, fs, fo) = reconstruct(&self.arena, parent, steps, obs);
                *self.best = Some((arr, pl, fs, fo));
                self.incumbent_at.get_or_insert(learn_ticks);
                return true;
            }
            return false;
        }
        let b = Bucket::of(&end, ehash, ledger.open.len() as u16, counts.blocks > 0, ledger.pending_drops());
        if self.best_bucket.get(&b).is_some_and(|&prev| prev <= arr) { return false; }
        let p = priority_key_topo(arr, &end, self.goal, self.topo, &edits, &ledger, self.kit);
        let nid = self.arena.len();
        self.arena.push(ArenaNode {
            state: end, arrival: arr, depth, prefix_len: self.arena[parent].prefix_len, parent: Some(parent),
            edge_steps: steps, edge_obs: obs, edits, counts, ledger,
        });
        self.heap.push(Reverse((p, u64::MAX - arr, nid)));
        false
    }
}

/// One hand action from arena node `idx`, run on the delta, appended to the search; `true`
/// when it touches the goal with nothing left to restore (the incumbent is set).
#[allow(clippy::too_many_arguments)]
fn push_hand<S: World>(
    delta: &mut WorldDelta<'_, ObservedWorld<'_, S>>,
    grid: &ObservedWorld<'_, S>,
    programs: &[Program],
    kv: &KitView,
    h: HandStep,
    idx: usize,
    bounds: Aabb,
    label: LabelId,
    provenance: [u8; 32],
    pb: &mut PhaseB<'_>,
    raw_learn_ticks: &mut u64,
) -> bool {
    let (prog, yaw) = carried(&pb.arena[idx], pb.goal);
    let s = pb.arena[idx].state.clone();
    let (mut edits, mut ledger, mut counts) = (pb.arena[idx].edits.clone(), pb.arena[idx].ledger.clone(), pb.arena[idx].counts);
    let Some((end_s, step, hit_goal)) = step_kit_node(delta, grid, programs, &s, pb.goal, prog, yaw, h.ticks, h.op, bounds, label, provenance, pb.arena[idx].arrival, &mut ledger, kv.spec.restore_required, raw_learn_ticks) else {
        return false;
    };
    let Some(before) = delta.state_id(h.op.pos()) else { return false };
    let obs = kit_obs(&s, &end_s, pb.goal, prog, crate::word_store::KitTemplate::of(h.op, &s, pb.goal, yaw, h.restore), step.ticks);
    let (arr, depth) = (pb.arena[idx].arrival.saturating_add(u64::from(step.ticks)), pb.arena[idx].depth + 1);
    kv.book(grid, before, h.op, pb.arena[idx].arrival, step.ticks, &mut edits, &mut ledger, &mut counts);
    let ehash = digest_edits(&edits);
    let clear = !kv.spec.restore_required || ledger.open.is_empty();
    pb.push_or_win(end_s, arr, depth, idx, vec![step], vec![obs], edits, counts, ledger, ehash, hit_goal, clear, *raw_learn_ticks)
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
    for si in 0..seq.len() {
        let (prog, aim, max_t) = seq[si];
        let s = node.state.clone();
        let (end, step, obs, hit) = if let Some(h) = kv.instantiate_step(aim, &*delta, &s, goal, &node.ledger, node.counts) {
            let yaw = aim.resolve(&s, goal, 0);
            let (end, step, hit) = step_kit_node(delta, grid, programs, &s, goal, prog, yaw, h.ticks, h.op, bounds, label, provenance, node.arrival, &mut node.ledger, restore_required, raw_ticks)?;
            let before = delta.state_id(h.op.pos())?;
            h.op.apply(delta);
            kv.book(grid, before, h.op, node.arrival, step.ticks, &mut node.edits, &mut node.ledger, &mut node.counts);
            let obs = kit_obs(&s, &end, goal, prog, h.template, step.ticks);
            (end, step, obs, hit)
        } else {
            if matches!(aim, AimRule::Click(_)) {
                return None;
            }
            let restore_obs: Vec<BlockPos> = if restore_required { node.ledger.open.iter().map(|o| o.pos).collect() } else { Vec::new() };
            let sig = defect_sig(grid, &s, goal);
            let next_aim = seq.get(si + 1).map(|&(_, a, _)| a);
            let drop_wait = node.ledger.wait_ticks_here(&s, node.arrival).unwrap_or(0);
            match exec_atomic_raw(&*delta, Some(grid), programs, &s, goal, prog, aim, u32::from(max_t), bounds, label, provenance, sig, Some(Hand { kv, counts: node.counts, field: None, next_aim }), &restore_obs, kv.makes(FieldEdit::Floor, node.counts), node.arrival, Some(&mut node.ledger), drop_wait, raw_ticks) {
                AtomicOutcome::Failed => return None,
                AtomicOutcome::Stopped { end, step, hit_goal, obs } => (end, step, obs, hit_goal),
            }
        };
        node.arrival = node.arrival.saturating_add(u64::from(step.ticks));
        node.depth += 1;
        node.state = end;
        node.edge_steps.push(step);
        node.edge_obs.push(obs);
        if hit {
            if !restore_required || node.ledger.open.is_empty() {
                return Some(true);
            }
            return None;
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
    kit_key: u64,
    stages: &mut List<Stage, 16>,
    thinking: &mut ThinkingCosts,
) -> Result<(Vec<WordStep>, Vec<ObservedStep>), (usize, String)> {
    let clock = std::time::Instant::now();
    let programs = words::tiny_programs();
    let bounds = bounds_aabb(source, from, goal, &win);
    let grid = ObservedWorld::defect(source);
    let provenance = source.provenance();
    let init_counts = kit.map_or(KitCounts::default(), |k| k.spec.counts());
    let restore_required = kit.is_some_and(|k| k.spec.restore_required);

    let mut seeds: Vec<ArenaNode> = win.prefixes.into_iter().map(|pref| {
        let arrival: u64 = pref.iter().map(|w| u64::from(w.ticks)).sum();
        let state = pref.last().map_or_else(|| from.clone(), |w| w.end.materialize());
        ArenaNode::seed(state, arrival, pref.into_iter().collect(), init_counts)
    }).collect();
    if seeds.is_empty() { seeds.push(ArenaNode::seed(from.clone(), 0, Vec::new(), init_counts)); }

    let mut proposals_tried = 0u64;
    let mut proposal_hits = 0u64;
    let mut raw_learn_ticks = 0u64;

    // Phase A: propose from the WordStore only, no learning: raw try_tick on the observed world;
    // with a kit, on its delta, each click instantiated from its frame-relative template.
    let mut best: Option<(u64, usize, Vec<WordStep>, Vec<ObservedStep>)> = None;
    // A route stitched from guessed words (coarse or regime-head, not this defect's own) leaves
    // the bracket open: its T is not known to be the site's, so learning still runs under it.
    let mut best_guessed = false;
    let mut prop_ticks = 0u64;
    let has_words = store.words.borrow().work.words_total > 0;

    if has_words {
        let mut heap: BinaryHeap<Reverse<(u64, u64, usize)>> = BinaryHeap::new();
        let mut arena: Vec<ArenaNode> = Vec::new();
        let mut guessed: Vec<bool> = Vec::new();
        let mut outcomes: Vec<(WordId, bool, u64)> = Vec::new();
        let mut best_bucket: HashMap<Bucket, u64> = HashMap::new();
        let mut delta = WorldDelta::new(&grid);
        let lb = |s: &PlayerState| u64::from(kit.map_or_else(|| goal.lower_bound(s), |kv| kv.cone.lower_bound(s, goal)));
        let clear = |n: &ArenaNode| !restore_required || n.ledger.open.is_empty();
        let bucket = |n: &ArenaNode| Bucket::of(&n.state, digest_edits(&n.edits), n.ledger.open.len() as u16, n.counts.blocks > 0, n.ledger.pending_drops());
        for s in &seeds {
            let idx = arena.len();
            let p = s.arrival.saturating_add(euclid_h(&s.state, goal));
            arena.push(ArenaNode::seed(s.state.clone(), s.arrival, s.edge_steps.clone(), init_counts));
            guessed.push(false);
            heap.push(Reverse((p, s.arrival, idx)));
        }
        let mut pops = 0u32;
        let mut saw_fine = false;
        while let Some(Reverse((_, _, idx))) = heap.pop() {
            let cap = if best.is_some() {
                if !best_guessed { 1_500 } else { 1_000 }
            } else if saw_fine {
                900
            } else {
                360
            };
            let pop_cap = if best.is_some() {
                if !best_guessed { 24 } else { 12 }
            } else if saw_fine {
                16
            } else {
                6
            };
            if prop_ticks >= cap || pops >= pop_cap {
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

            if goal.reached(&arena[idx].state, &arena[idx].state) && clear(&arena[idx]) {
                let (pl, steps, obs) = reconstruct(&arena, idx, Vec::new(), Vec::new());
                best = Some((arena[idx].arrival, pl, steps, obs));
                best_guessed = guessed[idx];
                proposal_hits += 1;
                continue;
            }

            let sig = defect_sig(&grid, &arena[idx].state, goal);
            let entry = EntryClass::of(&arena[idx].state);
            let desired_dy = goal.cell[1] - arena[idx].state.position.y.floor() as i32;
            let desired_dist = (goal.center().x - arena[idx].state.position.x).hypot(goal.center().z - arena[idx].state.position.z);
            let wids = store.words.borrow().propose_for_effect(sig, entry, Some((desired_dy, desired_dist)), kit.is_some());
            let start_h = euclid_h(&arena[idx].state, goal);
            for (wid, known) in wids {
                if known { saw_fine = true; }
                let cap = if best.is_some() {
                    if !best_guessed { 1_500 } else { 1_000 }
                } else if saw_fine {
                    900
                } else {
                    360
                };
                if prop_ticks >= cap {
                    break;
                }
                let child_guessed = guessed[idx] || !known;
                proposals_tried += 1;
                let ticks_before = prop_ticks;
                let mut progressed = false;
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
                            exec_word_chain(&grid, Some(&grid), &ws, &programs, &child.state, goal, wid, bounds, label, provenance, cur_sig, None, &mut prop_ticks)
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
                        progressed = true;
                        best = Some((child.arrival, pl, full_s, full_o));
                        best_guessed = child_guessed;
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
                                progressed = true;
                                best = Some((tot, pl, full_s, full_o));
                                best_guessed = child_guessed;
                            }
                            break;
                        }
                    }
                    let ch = euclid_h(&child.state, goal);
                    if ch + 2 <= start_h {
                        progressed = true;
                    }
                    if rep >= 1 && ch > prev_h.saturating_add(6) {
                        break;
                    }
                    if best_bucket.get(&bucket(&child)).is_some_and(|&prev| prev <= child.arrival) {
                        continue;
                    }
                    let p = child.arrival.saturating_add(ch);
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
                    guessed.push(child_guessed);
                    heap.push(Reverse((p, child.arrival, nid)));
                }
                outcomes.push((wid, progressed, prop_ticks.saturating_sub(ticks_before)));
            }
        }
        let mut ws = store.words.borrow_mut();
        for (wid, ok, spent) in outcomes {
            ws.record_outcome(wid, ok, spent);
        }
    }

    // Phase B: raw best-first learning guided by the field, when Phase A missed or its route is
    // a guess: then Phase A's T is the bound Phase B must beat.
    if best.is_none() || best_guessed {
        let topo = Legs::of(store, label, source, from).map_or_else(
            || Arc::new(TopoField::empty()),
            |legs| TopoField::get_or_build(store, label, provenance, kit_key, &grid, bounds, from, goal, kit, legs),
        );
        let has_guess = best.is_some();
        let all_seeds_unreachable = !topo.dist.is_empty() && seeds.iter().all(|n| topo.query(&n.state).is_infinite());
        let field_ticks = seeds.iter().map(|n| topo.query(&n.state)).filter(|d| d.is_finite()).fold(0.0f32, f32::max) as u64;
        let tight_guess = has_guess && field_ticks > 0 && best.as_ref().is_some_and(|b| b.0 <= (field_ticks as f64 * 1.15).ceil() as u64 + 2);
        let incumbent_at = tight_guess.then_some(0);
        let mut pb = PhaseB {
            goal, topo: &topo, kit, arena: Vec::with_capacity(2048),
            best_bucket: HashMap::with_capacity(2048), heap: BinaryHeap::new(), best: &mut best, incumbent_at,
        };
        for s in seeds {
            let idx = pb.arena.len();
            let p = priority_key_topo(s.arrival, &s.state, goal, &topo, &s.edits, &s.ledger, kit);
            pb.heap.push(Reverse((p, u64::MAX - s.arrival, idx)));
            pb.arena.push(s);
        }

        let base_cap = (96_000u64 + (win.exceptions as u64).min(80) * 1_800)
            .max(field_ticks.saturating_mul(1_200))
            .min(480_000);
        let cold_cap = if all_seeds_unreachable {
            base_cap.min(8_000)
        } else if has_guess && !tight_guess {
            base_cap.min(field_ticks.saturating_mul(260).clamp(12_000, 36_000))
        } else {
            base_cap
        };
        let word_budget = cold_cap / 5;
        let mut phase_b_word_ticks = 0u64;
        let mut delta = WorldDelta::new(&grid);
        let has_words_b = store.words.borrow().work.words_total > 0;

        while let Some(Reverse((_, _, idx))) = pb.heap.pop() {
            let polish = if tight_guess && pb.incumbent_at == Some(0) { 3_500 } else { 4_500 };
            if raw_learn_ticks >= cold_cap || pb.incumbent_at.is_some_and(|t| raw_learn_ticks >= t.saturating_add(polish)) {
                break;
            }
            let ub = pb.best.as_ref().map_or(u64::MAX, |b| b.0);
            let node_lb = kit.map_or_else(
                || u64::from(goal.lower_bound(&pb.arena[idx].state)),
                |kv| u64::from(kv.cone.lower_bound(&pb.arena[idx].state, goal))
                    .max(if restore_required { u64::from(pb.arena[idx].ledger.open_debt_lb(&pb.arena[idx].state, goal, pb.arena[idx].arrival, &kv.cone)) } else { 0 }),
            );
            if pb.arena[idx].arrival.saturating_add(node_lb) >= ub || pb.arena[idx].depth >= MAX_STEPS {
                continue;
            }
            let node_ehash = digest_edits(&pb.arena[idx].edits);
            let b = Bucket::of(&pb.arena[idx].state, node_ehash, pb.arena[idx].ledger.open.len() as u16, pb.arena[idx].counts.blocks > 0, pb.arena[idx].ledger.pending_drops());
            if pb.best_bucket.get(&b).is_some_and(|&prev| prev <= pb.arena[idx].arrival) {
                continue;
            }
            pb.best_bucket.insert(b, pb.arena[idx].arrival);

            if goal.reached(&pb.arena[idx].state, &pb.arena[idx].state) && (!restore_required || pb.arena[idx].ledger.open.is_empty()) {
                if pb.arena[idx].arrival < ub {
                    let (pl, steps, obs) = reconstruct(&pb.arena, idx, Vec::new(), Vec::new());
                    *pb.best = Some((pb.arena[idx].arrival, pl, steps, obs));
                    pb.incumbent_at.get_or_insert(raw_learn_ticks);
                }
                continue;
            }

            if pb.arena[idx].edits.is_empty() && (!restore_required || pb.arena[idx].ledger.open.is_empty()) {
                if let Some(tail) = try_tail_extend(&pb.arena[idx].state, goal, source, store, codec) {
                    let tot = pb.arena[idx].arrival.saturating_add(tail.iter().map(|w| u64::from(w.ticks)).sum::<u64>());
                    if tot < ub {
                        let (pl, steps, obs) = reconstruct(&pb.arena, idx, tail, Vec::new());
                        *pb.best = Some((tot, pl, steps, obs));
                        pb.incumbent_at.get_or_insert(raw_learn_ticks);
                    }
                }
            }

            delta.rollback(0);
            for &(ep, es) in &pb.arena[idx].edits {
                delta.set(ep, es);
            }

            if let Some(kv) = kit {
                let mut hands = kv.closures(&delta, &grid, &pb.arena[idx].state, goal, pb.arena[idx].arrival, &pb.arena[idx].ledger);
                hands.extend(topo.hands(kv, &delta, &grid, &pb.arena[idx], goal));
                for h in hands {
                    if push_hand(&mut delta, &grid, &programs, kv, h, idx, bounds, label, provenance, &mut pb, &mut raw_learn_ticks) {
                        break;
                    }
                }
            }

            let state_here = pb.arena[idx].state.clone();
            let arr_here = pb.arena[idx].arrival;
            let depth_here = pb.arena[idx].depth;
            let cur_edits = pb.arena[idx].edits.clone();
            let cur_counts = pb.arena[idx].counts;
            let cur_ledger = pb.arena[idx].ledger.clone();
            let hand = kit.map(|kv| Hand { kv, counts: cur_counts, field: Some(&topo), next_aim: None });
            let ledger_clear = !restore_required || cur_ledger.open.is_empty();

            let sig = if has_words_b && phase_b_word_ticks < word_budget && cur_edits.is_empty() {
                let sig = defect_sig(&grid, &state_here, goal);
                let wids = store.words.borrow().propose_local(sig, EntryClass::of(&state_here));
                for wid in wids {
                    if raw_learn_ticks >= cold_cap || phase_b_word_ticks >= word_budget { break; }
                    let ws = store.words.borrow();
                    let reps = if ws.word(wid).self_chains { 4 } else { 1 };
                    let (mut cur_s, mut cur_steps, mut cur_obs, mut cur_arr, mut cur_depth) =
                        (state_here.clone(), Vec::new(), Vec::new(), arr_here, depth_here);
                    for rep in 0..reps {
                        let cur_sig = if rep == 0 { sig } else { defect_sig(&grid, &cur_s, goal) };
                        let mut word_ticks = 0u64;
                        let chain = exec_word_chain(&grid, Some(&grid), &ws, &programs, &cur_s, goal, wid, bounds, label, provenance, cur_sig, hand, &mut word_ticks);
                        raw_learn_ticks += word_ticks;
                        phase_b_word_ticks += word_ticks;
                        let Some((ex, st_list, ob_list, hit)) = chain else { break };
                        cur_arr = cur_arr.saturating_add(st_list.iter().map(|w| u64::from(w.ticks)).sum());
                        cur_depth += st_list.len();
                        cur_steps.extend(st_list);
                        cur_obs.extend(ob_list);
                        cur_s = ex;
                        if cur_depth > MAX_STEPS
                            || pb.push_or_win(cur_s.clone(), cur_arr, cur_depth, idx, cur_steps.clone(), cur_obs.clone(), cur_edits.clone(), cur_counts, cur_ledger.clone(), node_ehash, hit, ledger_clear, raw_learn_ticks)
                        {
                            break;
                        }
                    }
                }
                sig
            } else {
                DefectSig { fine: 0, coarse: 0, rot: approach_frame(&state_here, goal).0 }
            };

            let (gx, gz) = dir_to_goal(&state_here, goal);
            let restore_obs: Vec<BlockPos> = if restore_required { cur_ledger.open.iter().map(|o| o.pos).collect() } else { Vec::new() };
            let eye_here = law::interact::get_eye_position(state_here.position, state_here.pose.eye_height());
            let has_far_open_ob = restore_obs.iter().any(|&op| !law::interact::is_within_block_interaction_range(eye_here, 4.4, op, 0.0));
            let direct_blocked = state_here.horizontal_collision || !can_clear_probe(&delta, &state_here, gx, gz, 0.75) || state_here.velocity.horizontal_sqr() < 0.01 || has_far_open_ob;
            let landing = topo.landing(&delta, &state_here);
            grid.reset_missing();
            let drop_wait = if restore_required { cur_ledger.wait_ticks_here(&state_here, arr_here).unwrap_or(0) } else { 0 };
            let aims = candidate_aims(&state_here, goal, sig.rot, direct_blocked, landing);
            let progs = programs_for(&delta, &state_here, goal, landing, direct_blocked, drop_wait > 0);
            let sprint_ok = sprint_allowed(&state_here);
            let kit_edge_stop = kit.is_some_and(|kv| kv.makes(FieldEdit::Floor, cur_counts));

            let mut expanded = 0usize;
            for (progs_slice, max_t, secondary) in [(&progs[..], 96u32, false), (SECONDARY_STRAFE, 48u32, true)] {
                if secondary && expanded > 0 { break; }
                for &prog in progs_slice {
                    let wait_for_drop_prog = drop_wait > 0 && prog == words::program::P_BRAKE;
                    let aim_cap = if wait_for_drop_prog {
                        1
                    } else if secondary {
                        aims.len().min(4)
                    } else if prog == P_FSG || prog == P_FS || prog == P_FSJ || !sprint_ok {
                        aims.len()
                    } else {
                        aims.len().min(5)
                    };
                    for &aim in &aims[..aim_cap] {
                        if raw_learn_ticks >= cold_cap { break; }
                        let mut next_ledger = cur_ledger.clone();
                        let ledger_arg = (restore_required && !next_ledger.open.is_empty()).then_some(&mut next_ledger);
                        if let AtomicOutcome::Stopped { end, step, hit_goal, obs } = exec_atomic_raw(&delta, Some(&grid), &programs, &state_here, goal, prog, aim, max_t, bounds, label, provenance, sig, hand, &restore_obs, kit_edge_stop, arr_here, ledger_arg, if wait_for_drop_prog { drop_wait } else { 0 }, &mut raw_learn_ticks) {
                            expanded += usize::from(!secondary);
                            pb.push_or_win(end, arr_here.saturating_add(u64::from(step.ticks)), depth_here + 1, idx, vec![step], vec![obs], cur_edits.clone(), cur_counts, next_ledger, node_ehash, hit_goal, ledger_clear, raw_learn_ticks);
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
    stages.push(Stage::Defect { proposals: proposals_tried as u32, learn_ticks: raw_learn_ticks, certify_ticks: 0, nanos });

    let Some((_, prefix_len, steps, mut obs)) = best else {
        return Err((win.exceptions, format!("defect uncrossed at {:?}: proposals={}, learn_ticks={}", win.first, proposals_tried, raw_learn_ticks)));
    };

    for (i, ob) in obs.iter_mut().enumerate() {
        if ob.defect_coarse == 0 {
            let s_before = if prefix_len + i == 0 { from.clone() } else { steps[prefix_len + i - 1].end.materialize() };
            let ssig = defect_sig(&grid, &s_before, goal);
            ob.defect_fine = ssig.fine;
            ob.defect_coarse = ssig.coarse;
        }
    }

    Ok((steps, obs))
}

pub(crate) fn prewarm_batch_topo<S: CorridorSource>(
    store: &GeneratorStore,
    source: &S,
    label: LabelId,
    sample_from: &PlayerState,
    goal: Goal,
    swarm_min: BlockPos,
    swarm_max: BlockPos,
) -> bool {
    let Some(legs) = Legs::of(store, label, source, sample_from) else {
        return false;
    };
    let legs_bits = (legs.h.to_bits(), legs.up.to_bits(), legs.down.to_bits(), legs.reach);
    let provenance = source.provenance();
    let depth = (sample_from.profile.safe_fall_distance + f64::from(sample_from.health)).ceil() as i32;
    let pad_lo_y = depth.min(24).max(6);
    let mut min = [
        swarm_min[0].min(goal.cell[0]).saturating_sub(20),
        swarm_min[1].min(goal.cell[1]).saturating_sub(pad_lo_y),
        swarm_min[2].min(goal.cell[2]).saturating_sub(20),
    ];
    let mut max = [
        swarm_max[0].max(goal.cell[0]).saturating_add(20),
        swarm_max[1].max(goal.cell[1]).saturating_add(8),
        swarm_max[2].max(goal.cell[2]).saturating_add(20),
    ];
    if let Some((bmin, bmax)) = source.bounds_box() {
        min[0] = min[0].max(bmin[0] - 2);
        min[1] = min[1].max(bmin[1] - 2);
        min[2] = min[2].max(bmin[2] - 2);
        max[0] = max[0].min(bmax[0] + 2);
        max[1] = max[1].min(bmax[1] + 2);
        max[2] = max[2].min(bmax[2] + 2);
    } else {
        min[1] = min[1].max(source.min_y() - 2);
    }
    let vol_of = |a: BlockPos, b: BlockPos| -> usize {
        (b[0] - a[0] + 1).max(1) as usize
            * (b[1] - a[1] + 1).max(1) as usize
            * (b[2] - a[2] + 1).max(1) as usize
    };
    if vol_of(min, max) > MAX_TOPO_CELLS {
        min[1] = swarm_min[1].min(goal.cell[1]).saturating_sub(4).max(source.min_y() - 2);
        max[1] = swarm_max[1].max(goal.cell[1]).saturating_add(8);
        if vol_of(min, max) > MAX_TOPO_CELLS {
            return false;
        }
    }
    if store.topo.borrow().entries.iter().any(|e| {
        e.provenance == provenance
            && e.goal == goal
            && e.kit_key == 0
            && e.legs_bits == legs_bits
            && e.min[0] <= min[0] && e.max[0] >= max[0]
            && e.min[1] <= min[1] && e.max[1] >= max[1]
            && e.min[2] <= min[2] && e.max[2] >= max[2]
    }) {
        return true;
    }
    let grid = ObservedWorld::defect(source);
    let field = Arc::new(TopoField::build_box(store, label, &grid, min, max, sample_from, goal, None, legs, false));
    if field.dist.is_empty() {
        return false;
    }
    let mut cache = store.topo.borrow_mut();
    cache.builds += 1;
    if cache.entries.len() >= 8 {
        cache.entries.remove(0);
    }
    cache.entries.push(TopoCacheEntry {
        provenance,
        goal,
        kit_key: 0,
        legs_bits,
        min: field.min,
        max: field.max,
        field,
    });
    true
}
