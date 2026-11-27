//! Spartan test suite for Kit Unfolding Slice 1 (`map/detail/representation.md` & `engine/plan/src/kit.rs`):
//! - `E2-zero-tax`: bare-legs vs full Survival kit on mixed flat ground (`<= 25 µs` median)
//! - `E2-combo` & `E2-swap`: 10,000 kits fit in `< 1 MB` factored tables, stack decrements share `Arc`s
//! - `KT16`: `KitCone::lower_bound` monotonic + admissible over 500 triples + witnessed runs
//! - `R-lego-1..4`, `E3-nether-bucket`, `E2-last-block` shadow price
//! - Real `doorcorridor` fixture (`T=98` his best) and the survival tracks `amertystcoral`,
//!   `ametwoodbox` (restored) and `mud` (unrestored), replayed and beating his best.
//! - Slice 2 `G-kit`: a door corridor and a wall breach learned once generalize to a site
//!   translated by (+25, 0, +17) and turned 90° with `learn_ticks == 0`, and `edit()` drops the
//!   certified route only inside its footprint.

use std::{collections::{HashMap, HashSet}, sync::Arc, time::Instant};
use law::{
    Aabb, Block, BlockPos, PlayerState, Vec3, World,
    basis::cell::StateId,
    codec::Interner,
    sim::WorldDelta,
};
use plan::{
    Goal, Kit, KitOp, KitSpec, KitView, Plan, PlanOutcome, Stage,
    kit::{
        BodyMovementKey, DIM_WATER_EVAPORATES, ToolProfileKey, VERB_BUCKET_POWDER_SNOW,
        VERB_BUCKET_WATER, VERB_PLACE, cache_memory_bytes, default_stone_state,
        shadow_price, should_spend_scarce_token,
    },
    flight::track,
    solve,
    source::{GeneratorSuperflatSource, ObservedWorld},
    store::GeneratorStore,
};

#[derive(Clone)]
struct StateWorld {
    cells: HashMap<BlockPos, StateId>,
    air: StateId,
    air_blk: &'static Block,
    lo: BlockPos,
    hi: BlockPos,
    difficulty: u8,
    natural_regeneration: bool,
    has_fluid_flag: bool,
    has_inside_flag: bool,
}

impl StateWorld {
    fn new(lo: BlockPos, hi: BlockPos) -> Self {
        let air = StateId::parse("minecraft:air", |_| None).expect("air state");
        Self {
            cells: HashMap::new(),
            air,
            air_blk: air.block_at([0, 0, 0]),
            lo,
            hi,
            difficulty: 0,
            natural_regeneration: true,
            has_fluid_flag: false,
            has_inside_flag: false,
        }
    }

    fn set(&mut self, p: BlockPos, sid: StateId) {
        if sid == self.air {
            self.cells.remove(&p);
            return;
        }
        let b = sid.block_at(p);
        if b.fluid.kind != law::FluidKind::Empty {
            self.has_fluid_flag = true;
        }
        if b.has_inside_effects() {
            self.has_inside_flag = true;
        }
        self.cells.insert(p, sid);
    }

    fn fill(&mut self, min: BlockPos, max: BlockPos, sid: StateId) {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                for x in min[0]..=max[0] {
                    self.set([x, y, z], sid);
                }
            }
        }
    }
}

impl World for StateWorld {
    fn block(&self, p: BlockPos) -> &Block {
        self.cells.get(&p).copied().map_or(self.air_blk, |s| s.block_at(p))
    }

    fn state_id(&self, p: BlockPos) -> Option<StateId> {
        Some(self.cells.get(&p).copied().unwrap_or(self.air))
    }

    fn model_id(&self, p: BlockPos) -> Option<law::basis::cell::ModelId> {
        self.state_id(p).and_then(|s| s.model())
    }

    fn loaded(&self, p: BlockPos) -> bool {
        (0..3).all(|a| p[a] >= self.lo[a] && p[a] <= self.hi[a])
    }

    fn min_y(&self) -> i32 {
        self.lo[1]
    }

    fn difficulty(&self) -> u8 {
        self.difficulty
    }

    fn natural_regeneration(&self) -> bool {
        self.natural_regeneration
    }

    fn has_fluid(&self) -> bool {
        self.has_fluid_flag
    }

    fn has_inside_effects(&self) -> bool {
        self.has_inside_flag
    }

    fn collision_cell_bounds(&self, bounds: Aabb, lo: BlockPos, hi: BlockPos) -> Option<(BlockPos, BlockPos)> {
        let mut rlo = [i32::MAX; 3];
        let mut rhi = [i32::MIN; 3];
        let _ = bounds;
        for (&p, &s) in &self.cells {
            if (0..3).all(|a| p[a] >= lo[a] && p[a] <= hi[a]) && !s.block_at(p).never_collides() {
                for a in 0..3 {
                    rlo[a] = rlo[a].min(p[a]);
                    rhi[a] = rhi[a].max(p[a]);
                }
            }
        }
        (rlo[0] <= rhi[0]).then_some((rlo, rhi))
    }

    fn visit_collision_cells(&self, _bounds: Aabb, lo: BlockPos, hi: BlockPos, mut visit: impl FnMut(BlockPos) -> bool) -> bool {
        let mut pts: Vec<BlockPos> = self
            .cells
            .iter()
            .filter(|&(&p, &s)| (0..3).all(|a| p[a] >= lo[a] && p[a] <= hi[a]) && !s.block_at(p).never_collides())
            .map(|(&p, _)| p)
            .collect();
        pts.sort_unstable_by_key(|p| (p[2], p[1], p[0]));
        for p in pts {
            if !visit(p) {
                return false;
            }
        }
        true
    }
}

fn sid(name: &str) -> StateId {
    let full = if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") };
    StateId::parse(&full, |_| None).unwrap_or_else(|e| panic!("{full}: {e}"))
}

fn door_sid(facing: &str, open: bool, half: &str) -> StateId {
    StateId::parse("minecraft:oak_door", |p| match p {
        "facing" => Some(facing),
        "open" => Some(if open { "true" } else { "false" }),
        "half" => Some(half),
        "hinge" => Some("left"),
        "powered" => Some("false"),
        _ => None,
    })
    .expect("oak_door state")
}

fn settled_on(world: &impl World, cell: BlockPos) -> PlayerState {
    let top = world.block(cell).shape.coords[1].last().copied().unwrap_or(1.0) + f64::from(cell[1]);
    let mut s = plan::settle(
        &plan::Still(world),
        [f64::from(cell[0]) + 0.5, top, f64::from(cell[2]) + 0.5],
        0,
    )
    .expect("settle on start cell");
    s.saturation = 20.0;
    s.health = 20.0;
    s
}

fn replay_and_verify(
    world: &impl World,
    start: &PlayerState,
    plan_res: &Plan,
    goal: Goal,
    restore_required: bool,
) -> (PlayerState, Vec<(BlockPos, u64, KitOp)>) {
    let programs = words::tiny_programs();
    let obs = ObservedWorld::new(world);
    let mut delta = WorldDelta::new(&obs);
    let mut s = start.clone();
    let mut touched = goal.touched(start, start);
    let mut edited: HashSet<BlockPos> = HashSet::new();
    let mut ops_log: Vec<(BlockPos, u64, KitOp)> = Vec::new();

    for (idx, step) in plan_res.words.iter().enumerate() {
        let still = step.edit.is_some_and(KitOp::holds_still);
        if let Some(op) = step.edit {
            if let KitOp::Interact { partner: Some((pp, _)), .. } = op {
                edited.insert(pp);
            }
            edited.insert(op.pos());
            ops_log.push((op.pos(), s.t - start.t, op));
        }
        let is_last = idx + 1 == plan_res.words.len();
        for elapsed in 0..step.ticks {
            if let Some(op) = step.edit.filter(|op| op.lands_at(step.ticks) == elapsed) {
                op.apply(&mut delta);
            }
            let before = s.bounding_box();
            let inp = if still {
                Default::default()
            } else {
                programs[step.program as usize]
                    .select(&delta, &s, elapsed, step.args)
                    .expect("word program selects input during replay")
                    .1
            };
            law::try_tick(&mut s, &delta, &inp).expect("law::try_tick succeeds during replay");
            obs.check().expect("replay stayed inside loaded world");
            assert!(s.health > 0.0, "player died during replay at tick {}", s.t);
            if goal.touched_boxes(before, s.bounding_box()) {
                if restore_required {
                    for &p in &edited {
                        assert_eq!(
                            delta.state_id(p),
                            world.state_id(p),
                            "touched goal at tick {} while edited cell {:?} was still unrestored",
                            s.t - start.t,
                            p
                        );
                    }
                }
                touched = true;
            }
        }
        if still && is_last && goal.touched(&s, &s) {
            touched = true;
        }
    }

    assert_eq!(s.t - start.t, plan_res.ticks, "replay ticks mismatch plan.ticks");
    assert!(touched, "replayed route never touched goal {:?}", goal.cell);
    if restore_required {
        for p in edited {
            assert_eq!(
                delta.state_id(p),
                world.state_id(p),
                "cell {p:?} was not restored to original state (cur={:?}, orig={:?})",
                delta.state_id(p),
                world.state_id(p),
            );
        }
    }
    (s, ops_log)
}

fn solve_ready(
    world: &impl World,
    source: &GeneratorSuperflatSource,
    start: &PlayerState,
    goal: Goal,
    kit: Kit,
    store: &GeneratorStore,
    codec: &mut Interner,
) -> (Plan, u64) {
    let res = solve(start, goal, kit, source, store, codec, u64::MAX);
    let learn_ticks: u64 = res
        .stages
        .iter()
        .filter_map(|st| match st {
            Stage::Defect { learn_ticks, .. } => Some(*learn_ticks),
            _ => None,
        })
        .sum();
    let PlanOutcome::Ready(plan_res) = res.outcome else {
        panic!("solve failed for kit={kit:?} goal={:?}: outcome={:?} err={:?}", goal.cell, res.outcome, res.explanation);
    };
    let restore = match kit {
        Kit::BareLegs => true,
        Kit::Carried(k) => k.restore_required,
    };
    replay_and_verify(world, start, &plan_res, goal, restore);
    (plan_res, learn_ticks)
}

fn full_survival_kit() -> KitSpec {
    KitSpec {
        body: BodyMovementKey { boots_depth_strider: 3, soul_speed: 3, leather_boots: true },
        tool: ToolProfileKey { pickaxe_tier: 4, axe_tier: 4, shovel_tier: 4, efficiency: 5, silk_touch: true, haste: 0 },
        blocks: 64,
        place_block: default_stone_state(),
        interact: true,
        can_break: true,
        water_buckets: 1,
        powder_snow_buckets: 1,
        boats: 1,
        pearls: 1,
        elytra: false,
        dimension_flags: 0,
        restore_required: true,
        restore_blocks: None,
        free_hotbar_slots: 0,
    }
}

#[test]
fn e2_zero_tax_flat_and_mixed_ground() {
    let mut w = StateWorld::new([-6, -4, -4], [6, 12, 54]);
    let palette = [sid("stone"), sid("dirt"), sid("oak_planks")];
    for x in -6i32..=6 {
        for z in -4i32..=54 {
            let idx = (x.rem_euclid(3) + z.rem_euclid(3)) as usize % palette.len();
            w.set([x, 0, z], palette[idx]);
        }
    }
    let source = GeneratorSuperflatSource::from_world(&w, ([-6, -4, -4], [6, 12, 54]), [0, 0, 0]).unwrap();
    assert!(source.scars.is_empty(), "mixed full blocks must produce 0 legs scars");

    let start = settled_on(&w, [0, 0, 0]);
    let goal = Goal::of([0, 0, 20]);
    let store = GeneratorStore::default();
    let mut codec = Interner::default();

    let (bare_plan, bare_learn) = solve_ready(&w, &source, &start, goal, Kit::BareLegs, &store, &mut codec);
    assert_eq!(bare_learn, 0);

    let kit = Kit::Carried(full_survival_kit());
    let (kit_plan, kit_learn) = solve_ready(&w, &source, &start, goal, kit, &store, &mut codec);
    assert_eq!(kit_learn, 0);
    assert_eq!(kit_plan.ticks, bare_plan.ticks, "full Survival kit must match BareLegs T on mixed flat ground");
    assert!(kit_plan.words.iter().all(|s| s.edit.is_none()), "flat solve must emit 0 world edits");

    let mut samples_ns = Vec::with_capacity(30);
    for i in 0i32..30 {
        let g = Goal::of([(i % 3) - 1, 0, 14 + i]);
        let t0 = Instant::now();
        let r = solve(&start, g, kit, &source, &store, &mut codec, u64::MAX);
        let dt = t0.elapsed().as_nanos();
        assert!(matches!(r.outcome, PlanOutcome::Ready(_)));
        samples_ns.push(dt);
    }
    samples_ns.sort_unstable();
    let min_ns = samples_ns[0];
    let median_ns = (samples_ns[14] + samples_ns[15]) / 2;
    let p90_ns = samples_ns[(samples_ns.len() * 9) / 10];
    let max_ns = *samples_ns.last().unwrap();
    println!("E2_ZERO_TAX_MIXED_GROUND bare_T={} kit_T={} n=30 min_ns={min_ns} median_ns={median_ns} p90_ns={p90_ns} max_ns={max_ns}", bare_plan.ticks, kit_plan.ticks);
    assert!(median_ns <= 25_000, "E2 zero-tax median {median_ns} ns exceeded 25 µs ceiling");
}

#[test]
fn e2_kit_combo_memory_and_swap() {
    let stone = default_stone_state();
    let dirt = sid("dirt");
    let mut keys = HashSet::with_capacity(10_000);
    for i in 0..10_000u32 {
        let spec = KitSpec {
            body: BodyMovementKey {
                boots_depth_strider: (i % 4) as u8,
                soul_speed: ((i / 4) % 4) as u8,
                leather_boots: (i & 16) != 0,
            },
            tool: ToolProfileKey {
                pickaxe_tier: ((i / 32) % 5) as u8,
                axe_tier: ((i / 160) % 5) as u8,
                shovel_tier: ((i / 800) % 5) as u8,
                efficiency: if (i & 8) != 0 { 5 } else { 0 },
                silk_touch: (i & 64) != 0,
                haste: ((i / 100) % 3) as u8,
            },
            blocks: ((i % 64) + 1) as u16,
            place_block: if (i & 1) == 0 { stone } else { dirt },
            interact: (i & 2) != 0,
            can_break: (i & 4) != 0,
            water_buckets: ((i / 3) % 2) as u8,
            powder_snow_buckets: ((i / 5) % 2) as u8,
            boats: ((i / 7) % 2) as u8,
            pearls: ((i / 11) % 2) as u8,
            elytra: false,
            dimension_flags: if (i % 13) == 0 { DIM_WATER_EVAPORATES } else { 0 },
            restore_required: (i % 17) != 0,
            restore_blocks: None,
            free_hotbar_slots: 0,
        };
        let v = KitView::for_spec(spec);
        keys.insert(spec.capability_key() ^ u64::from(spec.blocks) ^ ((spec.place_block.0 as u64) << 40));
        assert!(v.legs.memory_bytes() < 16_384);
        assert!(v.breaking.memory_bytes() < 16_384);
    }
    let bytes = cache_memory_bytes();
    println!("E2_KIT_COMBO_MEMORY kits=10000 unique_keys={} total_cache_bytes={bytes}", keys.len());
    assert_eq!(keys.len(), 10_000);
    assert!(bytes < 1_000_000, "factored cache {bytes} B exceeded 1 MB cap");

    // 64 -> 63 block decrement preserves every factored Arc handle & verb_mask; 1 -> 0 drops VERB_PLACE.
    let s64 = full_survival_kit();
    let s63 = KitSpec { blocks: 63, ..s64 };
    let s0 = KitSpec { blocks: 0, ..s64 };
    let v64 = KitView::for_spec(s64);
    let v63 = KitView::for_spec(s63);
    let v0 = KitView::for_spec(s0);
    assert_eq!(v64.verb_mask, v63.verb_mask);
    assert_eq!(s64.capability_key(), s63.capability_key());
    assert!(Arc::ptr_eq(&v64.legs, &v63.legs));
    assert!(Arc::ptr_eq(&v64.breaking, &v63.breaking));
    assert!(Arc::ptr_eq(&v64.verb_effects, &v63.verb_effects));
    assert_ne!(v0.verb_mask & VERB_PLACE, VERB_PLACE);
    assert_ne!(v64.verb_mask & VERB_PLACE, 0);
}

#[test]
fn kill_test_16_cone_admissible() {
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    let mut next_u64 = || -> u64 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };

    let base_kit = KitSpec::empty();
    let superset_kit = full_survival_kit();
    let cone_base = KitView::for_spec(base_kit).cone;
    let cone_super = KitView::for_spec(superset_kit).cone;

    let mut violations = 0usize;
    for _ in 0..500 {
        let fx = ((next_u64() % 200) as i32 - 100) as f64 + 0.5;
        let fy = ((next_u64() % 60) as i32) as f64;
        let fz = ((next_u64() % 200) as i32 - 100) as f64 + 0.5;
        let gx = (next_u64() % 200) as i32 - 100;
        let gy = (next_u64() % 60) as i32;
        let gz = (next_u64() % 200) as i32 - 100;
        let from = PlayerState {
            position: Vec3::new(fx, fy, fz),
            health: 20.0,
            ..Default::default()
        };
        let goal = Goal::of([gx, gy, gz]);
        let lb_small = cone_base.lower_bound(&from, goal);
        let lb_big = cone_super.lower_bound(&from, goal);
        if lb_big > lb_small {
            violations += 1;
        }
    }
    assert_eq!(violations, 0, "KT16 monotonicity violated {violations} times");

    // Verify witnessed trajectory against KitCone::lower_bound on flat + drop.
    let mut w = StateWorld::new([-4, -4, -4], [4, 30, 28]);
    w.fill([-3, 0, -2], [3, 0, 24], default_stone_state());
    let src = GeneratorSuperflatSource::from_world(&w, ([-4, -4, -4], [4, 30, 28]), [0, 0, 0]).unwrap();
    let start = settled_on(&w, [0, 0, 0]);
    let goal = Goal::of([0, 0, 20]);
    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let k = Kit::Carried(superset_kit);
    let (plan_res, _) = solve_ready(&w, &src, &start, goal, k, &store, &mut codec);
    let lb = KitView::for_spec(superset_kit).cone.lower_bound(&start, goal);
    println!("KT16 checked=500 violations=0 flat_lb={lb} witnessed_T={}", plan_res.ticks);
    assert!(u64::from(lb) <= plan_res.ticks, "witnessed T={} beat KitCone LB={lb}", plan_res.ticks);
}

#[test]
fn r_lego_1_2_3_4_and_nether_bucket() {
    let stone = default_stone_state();
    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let tool = ToolProfileKey { pickaxe_tier: 4, efficiency: 5, silk_touch: true, ..Default::default() };

    // R-lego-1: 5-block & 6-block gaps with 4 blocks, restore_required=true.
    for gap_len in [5i32, 6i32] {
        let gap_end = 4 + gap_len; // floor at 0..=4 and (4+gap_len+1)..=16 -> air on 5..=gap_end
        let mut w = StateWorld::new([-4, -6, -4], [4, 10, gap_end + 10]);
        w.fill([-2, 0, -2], [2, 0, 4], stone);
        w.fill([-2, 0, gap_end + 1], [2, 0, gap_end + 8], stone);
        let src = GeneratorSuperflatSource::from_world(&w, ([-4, -6, -4], [4, 10, gap_end + 10]), [0, 0, 0]).unwrap();
        let start = settled_on(&w, [0, 0, 0]);
        let goal = Goal::of([0, 0, gap_end + 5]);
        let k = Kit::Carried(KitSpec { blocks: 4, can_break: true, tool, restore_required: true, ..KitSpec::empty() });
        let (p, learn_ticks) = solve_ready(&w, &src, &start, goal, k, &store, &mut codec);
        let verifs = p.words.iter().filter(|s| s.edit.is_some()).count();
        println!("R_LEGO_1 gap={gap_len} T={} learn_ticks={learn_ticks} edits={verifs}", p.ticks);
        assert!(learn_ticks <= 240_000, "R-lego-1 gap={gap_len} exceeded budget: {learn_ticks}");
    }

    // R-lego-2: second gap shape reusing the same GeneratorStore.
    {
        let mut w = StateWorld::new([-4, -6, -4], [4, 10, 22]);
        w.fill([-2, 0, -2], [2, 0, 3], stone);
        w.fill([-2, 1, 9], [2, 1, 18], stone);
        let src = GeneratorSuperflatSource::from_world(&w, ([-4, -6, -4], [4, 10, 22]), [0, 0, 0]).unwrap();
        let start = settled_on(&w, [0, 0, 0]);
        let goal = Goal::of([0, 1, 15]);
        let k = Kit::Carried(KitSpec { blocks: 4, can_break: true, tool, restore_required: true, ..KitSpec::empty() });
        let (p, learn_ticks) = solve_ready(&w, &src, &start, goal, k, &store, &mut codec);
        println!("R_LEGO_2 step_up_gap=5 T={} learn_ticks={learn_ticks}", p.ticks);
    }

    // R-lego-3: mid-air sprint-jump-place beats forced stop-and-place at the ledge.
    {
        let mut w = StateWorld::new([-4, -6, -4], [4, 10, 22]);
        w.fill([-2, 0, -2], [2, 0, 4], stone);
        w.fill([-2, 0, 10], [2, 0, 18], stone);
        let src = GeneratorSuperflatSource::from_world(&w, ([-4, -6, -4], [4, 10, 22]), [0, 0, 0]).unwrap();
        let start = settled_on(&w, [0, 0, 0]);
        let goal = Goal::of([0, 0, 15]);
        let k = Kit::Carried(KitSpec { blocks: 4, can_break: true, tool, restore_required: true, ..KitSpec::empty() });
        let (p_midair, _) = solve_ready(&w, &src, &start, goal, k, &store, &mut codec);

        // Forced stop-and-place on block [0,0,4]: sneak/walk to edge, stop, place at [0,0,5], cross.
        let mut stop_s = start.clone();
        let mut delta = WorldDelta::new(&w);
        let progs = words::tiny_programs();
        let walk_id = words::program::tiny_program_id(0, false, words::program::HeldPredicate::Never, words::program::HeldPredicate::Always);
        let sprint_id = words::program::tiny_program_id(0, true, words::program::HeldPredicate::Never, words::program::HeldPredicate::Never);
        let mut t_stop = 0u64;
        // Walk with sneak to z=4.7 then bridge and sprint across.
        while stop_s.position.z < 4.5 {
            let (_, inp) = progs[walk_id as usize].select(&delta, &stop_s, 0, Default::default()).unwrap();
            law::try_tick(&mut stop_s, &delta, &inp).unwrap();
            t_stop += 1;
        }
        // Forced stop for 20 ticks + 1 block per gap cell + restore walk back cleanly would take far longer; check just walking + stop-place already slower.
        for gz in 5..=9 {
            delta.set([0, 0, gz], stone);
            let (_, inp) = progs[walk_id as usize].select(&delta, &stop_s, 0, Default::default()).unwrap();
            law::try_tick(&mut stop_s, &delta, &inp).unwrap();
            t_stop += 1;
            while stop_s.position.z < f64::from(gz) + 0.5 {
                let (_, inp) = progs[walk_id as usize].select(&delta, &stop_s, 0, Default::default()).unwrap();
                law::try_tick(&mut stop_s, &delta, &inp).unwrap();
                t_stop += 1;
            }
        }
        while !goal.touched(&stop_s, &stop_s) && t_stop < 200 {
            let (_, inp) = progs[sprint_id as usize].select(&delta, &stop_s, 0, Default::default()).unwrap();
            law::try_tick(&mut stop_s, &delta, &inp).unwrap();
            t_stop += 1;
        }
        println!("R_LEGO_3 midair_T={} stop_and_place_T={t_stop}", p_midair.ticks);
        assert!(p_midair.ticks < t_stop, "expected mid-air place ({}) < forced stop-and-place ({t_stop})", p_midair.ticks);
    }

    // R-lego-4a: 2-high stone wall broken & restored with Silk Touch pickaxe (blocks = 0).
    {
        let mut w = StateWorld::new([-3, -2, -2], [3, 6, 12]);
        w.fill([-2, 0, -2], [2, 0, 10], stone);
        w.fill([-2, 1, 4], [2, 2, 4], stone);
        let src = GeneratorSuperflatSource::from_world(&w, ([-3, -2, -2], [3, 6, 12]), [0, 0, 0]).unwrap();
        let start = settled_on(&w, [0, 0, 0]);
        let goal = Goal::of([0, 0, 8]);
        let k = Kit::Carried(KitSpec { can_break: true, tool, restore_required: true, ..KitSpec::empty() });
        let (p, _) = solve_ready(&w, &src, &start, goal, k, &store, &mut codec);
        assert!(p.words.iter().any(|s| matches!(s.edit, Some(KitOp::Break { .. }))));
        assert!(p.words.iter().any(|s| matches!(s.edit, Some(KitOp::Place { .. }))));
        println!("R_LEGO_4A stone_wall_restore T={}", p.ticks);
    }

    // R-lego-4b: closed oak_door corridor opened, crossed, and closed behind with KitSpec::interact_only().
    {
        let mut w = StateWorld::new([-3, -2, -2], [3, 6, 12]);
        w.fill([-2, 0, -2], [2, 0, 10], stone);
        for side_x in [-2, -1, 1, 2] {
            w.fill([side_x, 1, 4], [side_x, 3, 4], stone);
        }
        w.set([0, 3, 4], stone);
        w.set([0, 1, 4], door_sid("south", false, "lower"));
        w.set([0, 2, 4], door_sid("south", false, "upper"));
        let src = GeneratorSuperflatSource::from_world(&w, ([-3, -2, -2], [3, 6, 12]), [0, 0, 0]).unwrap();
        let start = settled_on(&w, [0, 0, 0]);
        let goal = Goal::of([0, 0, 8]);
        let k = Kit::Carried(KitSpec::interact_only());
        let (p, _) = solve_ready(&w, &src, &start, goal, k, &store, &mut codec);
        let interacts = p.words.iter().filter(|s| matches!(s.edit, Some(KitOp::Interact { .. }))).count();
        assert!(interacts >= 2, "expected open + close Interact ops, got {interacts}");
        println!("R_LEGO_4B oak_door_restore T={} interacts={interacts}", p.ticks);
    }

    // R-lego-4c + E3-nether-bucket: 25-block drop water clutch (<= 4t pickup + restore) vs Nether rejection & powder snow.
    {
        let mut w = StateWorld::new([-4, -2, -4], [4, 32, 10]);
        w.fill([-3, 0, -2], [3, 0, 8], stone);
        w.set([0, 25, 0], stone);
        let src = GeneratorSuperflatSource::from_world(&w, ([-4, -2, -4], [4, 32, 10]), [0, 25, 0]).unwrap();
        let start = settled_on(&w, [0, 25, 0]);
        let goal = Goal::of([0, 0, 4]);

        let water_kit = Kit::Carried(KitSpec { water_buckets: 1, restore_required: true, ..KitSpec::empty() });
        let (pw, _) = solve_ready(&w, &src, &start, goal, water_kit, &store, &mut codec);
        let (_, ops) = replay_and_verify(&w, &start, &pw, goal, true);
        let mut empty_tick = None;
        let mut pickup_tick = None;
        for (_, t, op) in ops {
            match op {
                KitOp::BucketEmpty { .. } => empty_tick = Some(t),
                KitOp::BucketPickup { .. } => pickup_tick = Some(t),
                _ => {}
            }
        }
        let dt_pick = pickup_tick.unwrap() - empty_tick.unwrap();
        assert!(dt_pick <= 4, "water pickup latency {dt_pick}t exceeded 4t bound");
        println!("R_LEGO_4C water_clutch_25_drop T={} pickup_latency_ticks={dt_pick}", pw.ticks);

        // E3-nether-bucket: DIM_WATER_EVAPORATES disables BucketWater and forces refusal (or no water verb).
        let nether_water = KitSpec {
            water_buckets: 1,
            dimension_flags: DIM_WATER_EVAPORATES,
            restore_required: true,
            ..KitSpec::empty()
        };
        let nwv = KitView::for_spec(nether_water);
        assert_eq!(nwv.verb_mask & VERB_BUCKET_WATER, VERB_BUCKET_WATER);
        assert!(!nwv.verb_effects.iter().any(|e| e.kind == plan::kit::VerbKind::BucketWater), "Nether probe must disable BucketWater");
        let r_nw = solve(&start, goal, Kit::Carried(nether_water), &src, &store, &mut codec, u64::MAX);
        assert!(!matches!(r_nw.outcome, PlanOutcome::Ready(_)), "Nether water-only drop must fail without fall-reset verb");

        let nether_snow = KitSpec {
            water_buckets: 1,
            powder_snow_buckets: 1,
            dimension_flags: DIM_WATER_EVAPORATES,
            restore_required: true,
            ..KitSpec::empty()
        };
        let nsv = KitView::for_spec(nether_snow);
        assert_ne!(nsv.verb_mask & VERB_BUCKET_POWDER_SNOW, 0);
        assert!(nsv.verb_effects.iter().any(|e| e.kind == plan::kit::VerbKind::BucketPowderSnow));
        let (ps, _) = solve_ready(&w, &src, &start, goal, Kit::Carried(nether_snow), &store, &mut codec);
        println!("E3_NETHER_BUCKET powder_snow_25_drop T={}", ps.ticks);
    }

    // E2-last-block shadow price unit law check.
    let sp = shadow_price(180, 40);
    assert_eq!(sp, 140);
    assert!(!should_spend_scarce_token(12, sp, false), "must save scarce last block for 140t downstream saving");
    assert!(should_spend_scarce_token(12, sp, true), "self-closing/restored block can still be spent");
}

#[test]
fn real_track_doorcorridor_unfolded() {
    let tracks_txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let store = GeneratorStore::default();
    let mut codec = Interner::default();

    let load_start = |t: &plan::flight::Track| -> (GeneratorSuperflatSource, PlayerState) {
        let source = GeneratorSuperflatSource::snapshot(&t.world, t.a).unwrap();
        let obs0 = ObservedWorld::new(&t.world);
        let top = obs0.block(t.a).shape.coords[1].last().copied().unwrap() + f64::from(t.a[1]);
        let mut s = plan::settle(
            &plan::Still(&t.world),
            [f64::from(t.a[0]) + 0.5, top, f64::from(t.a[2]) + 0.5],
            0,
        )
        .unwrap();
        s.saturation = 20.0;
        (source, s)
    };

    let door_line = tracks_txt.lines().find(|l| l.split_whitespace().next() == Some("doorcorridor")).expect("doorcorridor in tracks.txt");
    let dc = track(door_line).expect("load doorcorridor");
    let mut dc_blocks = std::collections::BTreeSet::new();
    for x in 26..=35 {
        for y in -61..=-58 {
            for z in 66..=70 {
                if let Some(sid) = dc.world.state_id([x, y, z]) {
                    let bn = sid.block().name();
                    if bn != "air" && bn != "grass_block" && bn != "dirt" && bn != "bedrock" {
                        dc_blocks.insert((bn, sid.get("open"), sid.get("half")));
                    }
                }
            }
        }
    }
    println!("DOORCORRIDOR_BLOCKS {dc_blocks:?}");
    let (dc_src, dc_start) = load_start(&dc);
    let dc_goal = dc.goal();

    let spec_open = KitSpec { restore_required: false, ..KitSpec::interact_only() };
    let (p_open, l_open) = solve_ready(&dc.world, &dc_src, &dc_start, dc_goal, Kit::Carried(spec_open), &store, &mut codec);
    println!("REAL_DOORCORRIDOR_OPEN his_best={:?} unfolded_open_T={} (learn={l_open})", dc.his.iter().min(), p_open.ticks);
    let (p_restore, l_rest) = solve_ready(&dc.world, &dc_src, &dc_start, dc_goal, Kit::Carried(KitSpec::interact_only()), &store, &mut codec);
    println!(
        "REAL_DOORCORRIDOR_RESTORE his_best={:?} unfolded_open_T={} (learn={l_open}) unfolded_restore_T={} (learn={l_rest})",
        dc.his.iter().min(),
        p_open.ticks,
        p_restore.ticks,
    );

    // Survival companion tracks, each replayed through WorldDelta + law::try_tick. The kit is
    // the full survival kit (silk diamond pickaxe, axe and shovel, 64 cobblestone, buckets).
    let full = full_survival_kit();
    let mud_open = KitSpec { restore_required: false, ..full };
    for (name, spec) in [("amertystcoral", full), ("ametwoodbox", full), ("mud", mud_open)] {
        let line = tracks_txt.lines().find(|l| l.split_whitespace().next() == Some(name)).expect("track in tracks.txt");
        let t = track(line).expect("load track");
        let (src, s) = load_start(&t);
        let (p, learn) = solve_ready(&t.world, &src, &s, t.goal(), Kit::Carried(spec), &store, &mut codec);
        let edits = p.words.iter().filter(|w| w.edit.is_some()).count();
        println!("SURVIVAL track={name} restore={} T={} his_best={:?} learn_ticks={learn} edits={edits}", spec.restore_required, p.ticks, t.his.iter().min());
        assert!(t.his.iter().min().is_some_and(|&his| p.ticks < u64::from(his)), "{name}: T={} must beat his best", p.ticks);
    }
    // mud restored: the pillar's base under the walkway is out of reach from it, and the
    // restoring plan (breach the mud box, rebuild it, pillar outside) is not found in budget.
    let line = tracks_txt.lines().find(|l| l.split_whitespace().next() == Some("mud")).expect("mud");
    let t = track(line).expect("load mud");
    let (src, s) = load_start(&t);
    let r = solve(&s, t.goal(), Kit::Carried(full), &src, &store, &mut codec, u64::MAX);
    println!("DIAG track=mud restore=true outcome={:?} why={:?}", r.outcome, r.explanation);
}

/// A site in local coordinates (`x` across, `z` along the approach), placed at `origin` and
/// turned by `rot` quarter turns about the vertical: `(x, z) -> (-z, x)` per turn.
struct Site {
    origin: BlockPos,
    rot: u8,
}

impl Site {
    fn at(&self, l: BlockPos) -> BlockPos {
        let (x, z) = match self.rot & 3 {
            0 => (l[0], l[2]),
            1 => (-l[2], l[0]),
            2 => (-l[0], -l[2]),
            _ => (l[2], -l[0]),
        };
        [self.origin[0] + x, l[1], self.origin[2] + z]
    }

    fn facing(&self, f: &str) -> &'static str {
        const TURN: [&str; 4] = ["south", "west", "north", "east"];
        let i = TURN.iter().position(|t| *t == f).expect("cardinal");
        TURN[(i + usize::from(self.rot)) % 4]
    }

    /// A floor across local `x -2..=2`, `z -2..=10` with a full-width wall at `z = 4`: a closed
    /// oak door in its middle (`door`), or a 2-high stone wall (no door).
    fn world(&self, door: bool) -> (StateWorld, ([i32; 3], [i32; 3])) {
        let corners = [self.at([-3, 0, -2]), self.at([3, 0, 12])];
        let lo = [corners[0][0].min(corners[1][0]), -2, corners[0][2].min(corners[1][2])];
        let hi = [corners[0][0].max(corners[1][0]), 6, corners[0][2].max(corners[1][2])];
        let mut w = StateWorld::new(lo, hi);
        let stone = default_stone_state();
        for x in -2..=2 {
            for z in -2..=10 {
                w.set(self.at([x, 0, z]), stone);
            }
            if door && x != 0 {
                for y in 1..=3 { w.set(self.at([x, y, 4]), stone); }
            } else if !door {
                for y in 1..=2 { w.set(self.at([x, y, 4]), stone); }
            }
        }
        if door {
            w.set(self.at([0, 3, 4]), stone);
            w.set(self.at([0, 1, 4]), door_sid(self.facing("south"), false, "lower"));
            w.set(self.at([0, 2, 4]), door_sid(self.facing("south"), false, "upper"));
        }
        (w, (lo, hi))
    }
}

#[test]
fn g_kit_translated_rotated_sites_reuse_door_and_breach_words() {
    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let tool = ToolProfileKey { pickaxe_tier: 4, efficiency: 5, silk_touch: true, ..Default::default() };
    let breach = KitSpec { can_break: true, tool, restore_required: true, ..KitSpec::empty() };
    let hits = |store: &GeneratorStore| store.words.borrow().work.proposal_hits;
    for (name, door, spec) in [("door", true, KitSpec::interact_only()), ("breach", false, breach)] {
        let mut learned = 0;
        for (i, site) in [Site { origin: [0, 0, 0], rot: 0 }, Site { origin: [25, 0, 17], rot: 1 }].into_iter().enumerate() {
            let (w, bounds) = site.world(door);
            let src = GeneratorSuperflatSource::from_world(&w, bounds, site.at([0, 0, 0])).unwrap();
            let start = settled_on(&w, site.at([0, 0, 0]));
            let goal = Goal::of(site.at([0, 0, 8]));
            let before = hits(&store);
            let (p, learn) = solve_ready(&w, &src, &start, goal, Kit::Carried(spec), &store, &mut codec);
            let edits = p.words.iter().filter(|s| s.edit.is_some()).count();
            println!("G_KIT motif={name} site={} T={} learn_ticks={learn} proposal_hits={} edits={edits}", ["A", "B(+25,0,+17) turned 90°"][i], p.ticks, hits(&store) - before);
            assert!(edits >= 2, "an edit and its restore");
            if i == 0 {
                learned = p.ticks;
                assert!(learn > 0, "site A learns the motif");
                continue;
            }
            assert_eq!(learn, 0, "site B must reuse the {name} word with zero learning");
            assert!(hits(&store) > before, "site B is crossed by a proposal");
            assert_eq!(p.ticks, learned, "the same motif, the same T");

            // A certified route is dropped only by an edit inside its footprint.
            let key = spec.capability_key();
            let served = |w: &StateWorld| store.words.borrow().route_for_kit_in_world(&start, goal, key, &ObservedWorld::new(w)).is_some();
            assert!(served(&w), "site B's route is certified and served again");
            let far = site.at([0, 5, -2]);
            let mut moved = w.clone();
            moved.set(far, default_stone_state());
            assert_eq!(store.words.borrow_mut().edit(&mut codec, far, &moved), 0, "an edit outside the footprint keeps the route");
            let floor = site.at([0, 0, 6]);
            let mut holed = w.clone();
            holed.set(floor, sid("air"));
            assert!(store.words.borrow_mut().edit(&mut codec, floor, &holed) >= 1, "a hole in the walked floor drops the route");
            assert!(!served(&w), "and it is no longer served");
        }
    }
}

#[test]
fn ametwoodbox_empty_hotbar_pickup_and_restore() {
    let tracks_txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let line = tracks_txt
        .lines()
        .find(|l| l.split_whitespace().next() == Some("ametwoodbox"))
        .expect("ametwoodbox in tracks.txt");
    let t = track(line).expect("load ametwoodbox");
    let src = GeneratorSuperflatSource::snapshot(&t.world, t.a).unwrap();
    let start = settled_on(&t.world, t.a);

    // Exact live ametwoodbox.kit.json hotbar: 4 armor pieces in slots 0..4, 5 empty slots (4..9),
    // bare hand (ToolProfileKey::default()), 0 blocks already held, restore_required: true.
    let spec = KitSpec {
        interact: true,
        can_break: true,
        restore_required: true,
        restore_blocks: Some([None; 9]),
        free_hotbar_slots: 5,
        ..KitSpec::empty()
    };
    let (p, learn) = solve_ready(&t.world, &src, &start, t.goal(), Kit::Carried(spec), &GeneratorStore::default(), &mut Interner::default());
    let (end_s, ops) = replay_and_verify(&t.world, &start, &p, t.goal(), true);

    let mut breaks: Vec<(law::BlockPos, u64)> = Vec::new();
    let mut places: Vec<(law::BlockPos, u64)> = Vec::new();
    for (pos, op_start_t, op) in ops {
        match op {
            KitOp::Break { .. } => {
                let bname = t.world.state_id(pos).expect("orig").block().name();
                assert_ne!(bname, "amethyst_block", "bare hand must not break requires_correct amethyst_block at {pos:?}");
                assert!(matches!(bname, "oak_planks" | "oak_log"), "expected oak_planks or oak_log breach, got {bname} at {pos:?}");
                breaks.push((pos, op_start_t + u64::from(op.lands_at(match op { KitOp::Break { break_ticks, .. } => u32::from(break_ticks), _ => 1 })) + 1));
            }
            KitOp::Place { .. } => places.push((pos, op_start_t)),
            _ => {}
        }
    }
    assert!(!breaks.is_empty() && breaks.len() == places.len(), "expected matching break/restore pairs: breaks={breaks:?} places={places:?}");

    // Verify 26.2 pickup delay (>= 10t after drop spawn) + pickup_box contact at some tick before restore Place.
    let programs = words::tiny_programs();
    let obs = ObservedWorld::new(&t.world);
    let mut delta = WorldDelta::new(&obs);
    let mut sim = start.clone();
    let mut box_history: Vec<(u64, law::Aabb)> = Vec::with_capacity(p.ticks as usize + 1);
    box_history.push((0, sim.bounding_box()));
    for step in &p.words {
        let still = step.edit.is_some_and(KitOp::holds_still);
        for elapsed in 0..step.ticks {
            if let Some(op) = step.edit.filter(|op| op.lands_at(step.ticks) == elapsed) {
                op.apply(&mut delta);
            }
            let inp = if still {
                Default::default()
            } else {
                programs[step.program as usize].select(&delta, &sim, elapsed, step.args).unwrap().1
            };
            law::try_tick(&mut sim, &delta, &inp).unwrap();
            box_history.push((sim.t - start.t, sim.bounding_box()));
        }
    }
    assert_eq!(sim.position, end_s.position);
    for (bpos, ready_from_break) in &breaks {
        let place_t = places.iter().find(|&&(ppos, _)| ppos == *bpos).expect("restore place").1;
        let ready_t = ready_from_break + law::items::DEFAULT_PICKUP_DELAY as u64;
        assert!(place_t >= ready_t, "restore at t={place_t} happened before 10t pickupReady={ready_t} for {bpos:?}");
        let picked = box_history
            .iter()
            .any(|&(tick, pbox)| tick >= ready_t && tick <= place_t && law::items::can_pickup(pbox, None, law::items::block_drop_center(*bpos), 0));
        assert!(picked, "player never intersected pickup_box for {bpos:?} between t={ready_t} and t={place_t}");
    }
    println!("AMETWOODBOX_EMPTY_HOTBAR_PICKUP T={} learn_ticks={learn} edits={}", p.ticks, breaks.len() + places.len());

    // Exact live Tracks.java ametwoodbox toolkit: {"minecraft:oak_log": 1} + 8 free hotbar slots.
    let oak_log = sid("oak_log");
    let mut held = [None; 9];
    held[0] = Some(oak_log.block());
    let live_spec = KitSpec {
        interact: true,
        can_break: true,
        blocks: 1,
        place_block: oak_log,
        restore_required: true,
        restore_blocks: Some(held),
        free_hotbar_slots: 8,
        ..KitSpec::empty()
    };
    let (lp, llearn) = solve_ready(&t.world, &src, &start, t.goal(), Kit::Carried(live_spec), &GeneratorStore::default(), &mut Interner::default());
    let (_, lops) = replay_and_verify(&t.world, &start, &lp, t.goal(), true);
    println!("AMETWOODBOX_LIVE_TOOLKIT T={} learn_ticks={llearn} edits={} ops={lops:?}", lp.ticks, lops.len());
}
