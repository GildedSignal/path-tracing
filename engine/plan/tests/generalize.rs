//! Integration tests for coordinate-free Word generalization, compound emergence,
//! NoRegen resource guard soundness, single-winner edit invalidation, and cross-track transfer.
use law::{Block, BlockPos, BoxWorld, PlayerState, World, codec::Interner};
use plan::{
    Goal, Kit, Plan, PlanOutcome, Stage, flight::track, solve,
    source::{GeneratorSuperflatSource, MeteredWorld, ObservedWorld},
    store::GeneratorStore,
    word_store::NoRegen,
};
use std::time::Instant;

fn rot_xz(lx: i32, lz: i32, rot: u8) -> (i32, i32) {
    match rot & 3 {
        0 => (lx, lz),
        1 => (-lz, lx),
        2 => (-lx, -lz),
        _ => (lz, -lx),
    }
}

fn learn_ticks(res: &plan::solve::SolveResult) -> u64 {
    res.stages.iter().filter_map(|s| match s {
        Stage::Defect { learn_ticks, .. } => Some(*learn_ticks),
        _ => None,
    }).sum()
}

fn replay_plan(world: &impl World, start: &PlayerState, plan: &Plan, goal: Goal) -> PlayerState {
    let programs = words::tiny_programs();
    let obs = ObservedWorld::new(world);
    let mut state = start.clone();
    let mut touched = goal.touched(start, start);
    for step in &plan.words {
        for elapsed in 0..step.ticks {
            let before = state.bounding_box();
            let (_, inp) = programs[step.program as usize]
                .select(&obs, &state, elapsed, step.args)
                .expect("word program row matches");
            law::try_tick(&mut state, &obs, &inp).expect("law tick succeeds");
            obs.check().expect("replay read an unknown cell");
            assert!(state.health > 0.0, "replayed plan killed player at t={}", state.t);
            if goal.touched_boxes(before, state.bounding_box()) {
                touched = true;
            }
        }
    }
    assert!(touched, "replayed plan did not touch goal {:?}", goal.cell);
    state
}

fn build_obstacle_world(origin: BlockPos, rot: u8) -> (BoxWorld, BlockPos, BlockPos, (BlockPos, BlockPos)) {
    let mut world = BoxWorld::default();
    let full = Block::full();
    let to_world = |lx: i32, ly: i32, lz: i32| -> BlockPos {
        let (rx, rz) = rot_xz(lx, lz, rot);
        [origin[0] + rx, origin[1] + ly, origin[2] + rz]
    };
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    for lx in -10..=10 {
        for lz in -4..=24 {
            let p = to_world(lx, 0, lz);
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
            world.blocks.insert(p, full.clone());
        }
    }
    // 3-wide, 2-high barrier at local z = 10 centered across x in -1..=1
    for lx in -1..=1 {
        for ly in 1..=2 {
            world.blocks.insert(to_world(lx, ly, 10), full.clone());
        }
    }
    lo[1] = origin[1] - 4;
    hi[1] = origin[1] + 12;
    world.minimum_y = origin[1] - 4;
    world.difficulty = 0;
    world.natural_regeneration = true;
    let a = to_world(0, 0, 0);
    let b = to_world(0, 0, 18);
    (world, a, b, (lo, hi))
}

fn settled_on(source: &GeneratorSuperflatSource, cell: BlockPos) -> PlayerState {
    let top = f64::from(cell[1]) + 1.0;
    let mut s = plan::settle(&plan::Still(source), [f64::from(cell[0]) + 0.5, top, f64::from(cell[2]) + 0.5], 0)
        .expect("settle on block");
    s.saturation = 20.0;
    s.t = 1;
    s
}

#[test]
fn ulp_shift_translate_rotate_zero_learn() {
    let store = GeneratorStore::default();
    let mut codec = Interner::default();

    let (w0, a0, b0, box0) = build_obstacle_world([0, 0, 0], 0);
    let src0 = GeneratorSuperflatSource::from_world(&w0, box0, a0).unwrap();
    let s0 = settled_on(&src0, a0);
    let g0 = Goal::of(b0);

    let r_cold = solve(&s0, g0, Kit::BareLegs, &src0, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_cold) = &r_cold.outcome else { panic!("cold solve failed: {:?}", r_cold.outcome); };
    assert!(learn_ticks(&r_cold) > 0, "cold solve should learn");
    replay_plan(&w0, &s0, p_cold, g0);

    // 1. Exact repeat -> cached route hit (0 certify & 0 learn ticks).
    let cert_before = store.words.borrow().work.certify_ticks;
    let r_rep = solve(&s0, g0, Kit::BareLegs, &src0, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_rep) = &r_rep.outcome else { panic!("repeat failed"); };
    assert_eq!(learn_ticks(&r_rep), 0);
    assert_eq!(store.words.borrow().work.certify_ticks, cert_before);
    replay_plan(&w0, &s0, p_rep, g0);

    // 2. +1 ULP start shift -> Phase A transfer (learn_ticks == 0).
    let mut s_ulp = s0.clone();
    s_ulp.position.x = f64::from_bits(s_ulp.position.x.to_bits() + 1);
    let learn_before = store.words.borrow().work.learn_ticks;
    let r_ulp = solve(&s_ulp, g0, Kit::BareLegs, &src0, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_ulp) = &r_ulp.outcome else { panic!("+1 ULP failed: {:?}", r_ulp.outcome); };
    assert_eq!(learn_ticks(&r_ulp), 0, "+1 ULP unexpectedly triggered learning");
    assert_eq!(store.words.borrow().work.learn_ticks - learn_before, 0);
    replay_plan(&w0, &s_ulp, p_ulp, g0);

    // 3. +0.05 start shift -> Phase A transfer (learn_ticks == 0).
    let mut s_shift = s0.clone();
    s_shift.position.x += 0.05;
    s_shift.position.z += 0.05;
    let learn_before = store.words.borrow().work.learn_ticks;
    let r_shift = solve(&s_shift, g0, Kit::BareLegs, &src0, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_shift) = &r_shift.outcome else { panic!("+0.05 failed: {:?}", r_shift.outcome); };
    assert_eq!(learn_ticks(&r_shift), 0, "+0.05 shift unexpectedly triggered learning");
    assert_eq!(store.words.borrow().work.learn_ticks - learn_before, 0);
    replay_plan(&w0, &s_shift, p_shift, g0);

    // 4. +25 X/Z translation -> Phase A transfer (learn_ticks == 0).
    let (wt, at, bt, boxt) = build_obstacle_world([25, 0, 25], 0);
    let srct = GeneratorSuperflatSource::from_world(&wt, boxt, at).unwrap();
    let st = settled_on(&srct, at);
    let gt = Goal::of(bt);
    let learn_before = store.words.borrow().work.learn_ticks;
    let r_tr = solve(&st, gt, Kit::BareLegs, &srct, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_tr) = &r_tr.outcome else { panic!("translation failed: {:?}", r_tr.outcome); };
    assert_eq!(learn_ticks(&r_tr), 0, "+25 translation unexpectedly triggered learning");
    assert_eq!(store.words.borrow().work.learn_ticks - learn_before, 0);
    replay_plan(&wt, &st, p_tr, gt);

    // 5. 90-degree rotation -> Phase A transfer (learn_ticks == 0).
    let (wr, ar, br, boxr) = build_obstacle_world([0, 0, 0], 1);
    let srcr = GeneratorSuperflatSource::from_world(&wr, boxr, ar).unwrap();
    let sr = settled_on(&srcr, ar);
    let gr = Goal::of(br);
    let learn_before = store.words.borrow().work.learn_ticks;
    let r_rot = solve(&sr, gr, Kit::BareLegs, &srcr, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_rot) = &r_rot.outcome else { panic!("90° rotation failed: {:?}", r_rot.outcome); };
    assert_eq!(learn_ticks(&r_rot), 0, "90° rotation unexpectedly triggered learning");
    assert_eq!(store.words.borrow().work.learn_ticks - learn_before, 0);
    replay_plan(&wr, &sr, p_rot, gr);
}

fn build_staircase_world(origin: BlockPos, rot: u8, steps: i32) -> (BoxWorld, BlockPos, BlockPos, (BlockPos, BlockPos)) {
    let mut world = BoxWorld::default();
    let full = Block::full();
    let to_world = |lx: i32, ly: i32, lz: i32| -> BlockPos {
        let (rx, rz) = rot_xz(lx, lz, rot);
        [origin[0] + rx, origin[1] + ly, origin[2] + rz]
    };
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    for lx in -6..=6 {
        for lz in -3..=(steps + 6) {
            let p = to_world(lx, 0, lz);
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
            world.blocks.insert(p, full.clone());
        }
    }
    for s in 1..=steps {
        for ly in 1..=s {
            world.blocks.insert(to_world(0, ly, s + 2), full.clone());
        }
    }
    // Goal pad on top of final step
    for lx in -1..=1 {
        for lz in (steps + 3)..=(steps + 4) {
            world.blocks.insert(to_world(lx, steps, lz), full.clone());
        }
    }
    lo[1] = origin[1] - 4;
    hi[1] = origin[1] + steps + 8;
    world.minimum_y = origin[1] - 4;
    world.difficulty = 0;
    world.natural_regeneration = true;
    (world, to_world(0, 0, 0), to_world(0, steps, steps + 3), (lo, hi))
}

#[test]
fn staircase_compound_emergence_and_self_chain() {
    let store = GeneratorStore::default();
    let mut codec = Interner::default();

    let (w8, a8, b8, box8) = build_staircase_world([0, 0, 0], 0, 8);
    let src8 = GeneratorSuperflatSource::from_world(&w8, box8, a8).unwrap();
    let s8 = settled_on(&src8, a8);
    let g8 = Goal::of(b8);
    let r8 = solve(&s8, g8, Kit::BareLegs, &src8, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p8) = &r8.outcome else { panic!("8-step staircase failed: {:?}", r8.outcome); };
    replay_plan(&w8, &s8, p8, g8);
    assert!(store.words.borrow().work.compounds_total > 0, "expected compound emergence on 8-step staircase");

    // Translated (+20, +3, +30) & 90°-rotated 12-step staircase must reuse emerged self-chaining words.
    let (w12, a12, b12, box12) = build_staircase_world([20, 3, 30], 3, 12);
    let src12 = GeneratorSuperflatSource::from_world(&w12, box12, a12).unwrap();
    let s12 = settled_on(&src12, a12);
    let g12 = Goal::of(b12);
    let learn_before = store.words.borrow().work.learn_ticks;
    let r12 = solve(&s12, g12, Kit::BareLegs, &src12, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p12) = &r12.outcome else { panic!("12-step rotated staircase failed: {:?}", r12.outcome); };
    assert_eq!(learn_ticks(&r12), 0, "rotated 12-step staircase unexpectedly triggered cold learning");
    assert_eq!(store.words.borrow().work.learn_ticks - learn_before, 0);
    replay_plan(&w12, &s12, p12, g12);
}

#[test]
fn no_regen_guard_and_single_winner_edit_invalidation() {
    let mut world = BoxWorld::default();
    let full = Block::full();
    for x in -4..=4 {
        for z in -2..=12 {
            world.blocks.insert([x, 0, z], full.clone());
        }
    }
    // Start pedestal at y = 8 -> drop 8.0 blocks onto y = 0 (head-hitter ceiling prevents high jump).
    for x in -1..=1 {
        for z in -1..=1 {
            world.blocks.insert([x, 8, z], full.clone());
        }
    }
    for x in -1..=1 {
        for z in 0..=4 {
            world.blocks.insert([x, 11, z], full.clone());
        }
    }
    world.minimum_y = -4;
    world.difficulty = 0;
    world.natural_regeneration = true;

    let mut source = GeneratorSuperflatSource::from_world(&world, ([-4, -4, -2], [4, 16, 12]), [0, 0, 0]).unwrap();
    let mut codec = Interner::default();
    let store = GeneratorStore::default();
    let goal = Goal::of([0, 0, 3]);

    // Wait: settled start on [0, 8, 0] with health=5.0 and food_tick_timer=9 survives via tick-1 regen,
    // while damage_no_regen >= 5.0 refuses health=5.0, food_tick_timer=0.
    let mut s_ref = plan::settle(&plan::Still(&source), [0.5, 9.0, 0.5], 0).unwrap();
    s_ref.saturation = 20.0;
    s_ref.health = 5.0;
    s_ref.food_tick_timer = 9;
    s_ref.t = 1;

    let res = solve(&s_ref, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(plan_ref) = &res.outcome else { panic!("drop solve failed: {:?}", res.outcome); };
    replay_plan(&world, &s_ref, plan_ref, goal);

    // 1. Refuse health = 5.0, food_tick_timer = 0 (lethal without regen).
    let mut s_lethal = s_ref.clone();
    s_lethal.food_tick_timer = 0;
    assert!(store.words.borrow().route(&s_lethal, goal).is_none(), "NoRegen must refuse lethal health=5.0, timer=0");
    // Verify NoRegen world actually makes this path lethal at health = 5.0.
    let nr = NoRegen(&world);
    let programs = words::tiny_programs();
    let mut sim = s_lethal.clone();
    for step in &plan_ref.words {
        for elapsed in 0..step.ticks {
            let (_, inp) = programs[step.program as usize].select(&nr, &sim, elapsed, step.args).unwrap();
            law::try_tick(&mut sim, &nr, &inp).unwrap();
        }
    }
    assert!(sim.health <= 0.0, "expected NoRegen death at health=5.0, got {}", sim.health);

    // 2. Admit health = 6.0, food_tick_timer = 0 (> damage_no_regen).
    let mut s_ok = s_lethal.clone();
    s_ok.health = 6.0;
    let r_ok = store.words.borrow().route(&s_ok, goal).expect("health=6.0 must be admitted");
    let cert_before = store.words.borrow().work.certify_ticks;
    let res_ok = solve(&s_ok, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    assert_eq!(store.words.borrow().work.certify_ticks, cert_before);
    if let PlanOutcome::Ready(p) = &res_ok.outcome {
        assert_eq!(p.ticks, r_ok.ticks);
        replay_plan(&world, &s_ok, p, goal);
    }

    // 3. Admit exact match (health = 5.0, food_tick_timer = 9) with rebased tick t = 21.
    let mut s_exact = s_ref.clone();
    s_exact.t = 21;
    let r_ex = store.words.borrow().route(&s_exact, goal).expect("exact resource state must be admitted");
    assert_eq!(r_ex.end.t, 21 + r_ex.ticks);
    let cert_before = store.words.borrow().work.certify_ticks;
    let res_ex = solve(&s_exact, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    assert_eq!(store.words.borrow().work.certify_ticks, cert_before);
    if let PlanOutcome::Ready(p) = &res_ex.outcome {
        replay_plan(&world, &s_exact, p, goal);
    }

    // 4. Edit outside the single-winner footprint leaves cache intact; edit inside invalidates it.
    let removed_out = store.words.borrow_mut().edit(&mut codec, [4, 1, 10], &source);
    assert_eq!(removed_out, 0, "outside edit must not invalidate cached route");
    assert!(store.words.borrow().route(&s_ok, goal).is_some());

    source.scars.insert([0, 1, 3], full);
    let removed_in = store.words.borrow_mut().edit(&mut codec, [0, 1, 3], &source);
    assert!(removed_in >= 1, "inside edit must invalidate cached route");
    assert!(store.words.borrow().route(&s_ok, goal).is_none());

    let res_after = solve(&s_ok, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p_after) = &res_after.outcome else { panic!("re-solve after edit failed: {:?}", res_after.outcome); };
    replay_plan(&source, &s_ok, p_after, goal);
}

#[test]
fn sequential_cross_track_transfer() {
    let tracks_txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let order = [
        "grassjumpeasy",
        "stonestaircase",
        "fencestraightbridge",
        "mangrovejumpeasy",
        "fences",
        "fencemaze",
        "redfakemaze",
        "carpetwater",
        "butoonpressureplates",
        "trapdoorparkour",
        "doorparkourblue",
    ];
    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let mut prev_tried = 0u64;
    let mut prev_hits = 0u64;
    let mut prev_learn = 0u64;
    let mut prev_cert = 0u64;

    for name in order {
        let line = tracks_txt.lines().find(|l| l.split_whitespace().next() == Some(name)).expect("track in tracks.txt");
        let t = track(line).expect("load track");
        let source = GeneratorSuperflatSource::snapshot(&t.world, t.a).unwrap();
        let goal = t.goal();
        let obs0 = ObservedWorld::new(&t.world);
        let top = obs0.block(t.a).shape.coords[1].last().copied().unwrap() + f64::from(t.a[1]);
        let mut start = plan::settle(&plan::Still(&t.world), [f64::from(t.a[0]) + 0.5, top, f64::from(t.a[2]) + 0.5], 0).unwrap();
        start.saturation = 20.0;
        let res = solve(&start, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
        let PlanOutcome::Ready(p) = &res.outcome else {
            panic!("track {name} failed in cross-track transfer: {:?}", res.outcome);
        };
        let end = replay_plan(&t.world, &start, p, goal);
        assert_eq!(end.t - start.t, p.ticks, "{name} replay ticks mismatch (prog0={})", p.words[0].program);
        let w = store.words.borrow().work;
        println!(
            "XFER track={name} T={} his={:?} tried_d={} hits_d={} words={} compounds={} learn_d={} cert_d={}",
            p.ticks,
            t.his.iter().min(),
            w.proposals_tried - prev_tried,
            w.proposal_hits - prev_hits,
            w.words_total,
            w.compounds_total,
            w.learn_ticks - prev_learn,
            w.certify_ticks - prev_cert,
        );
        prev_tried = w.proposals_tried;
        prev_hits = w.proposal_hits;
        prev_learn = w.learn_ticks;
        prev_cert = w.certify_ticks;
    }
    assert!(store.words.borrow().work.words_total > 0);
}

#[test]
fn post_site_flat_ground_stays_fast() {
    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let tracks_txt = include_str!("../../world/tests/fixtures/tracks.txt");
    for name in ["grassjumpeasy", "stonestaircase"] {
        let line = tracks_txt.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap();
        let t = track(line).unwrap();
        let source = GeneratorSuperflatSource::snapshot(&t.world, t.a).unwrap();
        let top = source.block(t.a).shape.coords[1].last().copied().unwrap() + f64::from(t.a[1]);
        let mut start = plan::settle(&plan::Still(&t.world), [f64::from(t.a[0]) + 0.5, top, f64::from(t.a[2]) + 0.5], 0).unwrap();
        start.saturation = 20.0;
        let _ = solve(&start, t.goal(), Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    }

    let flat = GeneratorSuperflatSource::new(0, Block::full(), [-1000, -4, -1000], [1000, 12, 1000]);
    let metered = MeteredWorld::new(&flat);
    let mut start = plan::settle(&plan::Still(&flat), [0.5, 1.0, 0.5], 0).unwrap();
    start.saturation = 20.0;
    // Warm up flat gait once.
    let _ = solve(&start, Goal::of([0, 0, 50]), Kit::BareLegs, &metered, &store, &mut codec, u64::MAX);
    let _ = metered.take_ticks();

    let mut times = Vec::with_capacity(20);
    for i in 1i32..=20 {
        let goal = Goal::of([i * 17, 0, 40 + i * 11]);
        let clock = Instant::now();
        let res = solve(&start, goal, Kit::BareLegs, &metered, &store, &mut codec, u64::MAX);
        times.push(clock.elapsed().as_nanos());
        assert!(matches!(res.outcome, PlanOutcome::Ready(_)));
        assert_eq!(metered.take_ticks(), 0, "post-site flat solve touched law at goal {:?}", goal.cell);
    }
    times.sort_unstable();
    let median = (times[9] + times[10]) / 2;
    println!("POST_SITE_FLAT median_ns={median} max_ns={}", times[19]);
    assert!(median < 50_000, "median {median} ns exceeded 50 µs budget");
}
