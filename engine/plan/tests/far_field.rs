//! Stage 1 falsifiers: thin bridge across chasm (KT13), long parkour streaming prefixes,
//! and per-section FarField invalidation.
use std::{collections::HashSet, time::Instant};
use law::{Block, BlockPos, World, codec::Interner};
use plan::{
    Face, Goal, Kit, PlanOutcome,
    solve, solve_step,
    source::{GeneratorSuperflatSource, ObservedWorld},
    store::GeneratorStore,
};

struct BlockSetWorld {
    air: Block,
    stone: Block,
    min: BlockPos,
    max: BlockPos,
    solids: HashSet<BlockPos>,
}

impl World for BlockSetWorld {
    fn block(&self, p: BlockPos) -> &Block {
        if self.solids.contains(&p) { &self.stone } else { &self.air }
    }
    fn difficulty(&self) -> u8 { 0 }
    fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
        if (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i]) {
            Ok(self.block(p))
        } else {
            Err("outside test bounds")
        }
    }
    fn loaded(&self, p: BlockPos) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }
    fn has_fluid(&self) -> bool { false }
    fn has_inside_effects(&self) -> bool { false }
}

fn fill(solids: &mut HashSet<BlockPos>, lo: BlockPos, hi: BlockPos) {
    for z in lo[2]..=hi[2] {
        for y in lo[1]..=hi[1] {
            for x in lo[0]..=hi[0] {
                solids.insert([x, y, z]);
            }
        }
    }
}

fn verify_plan_replay(start: &law::PlayerState, goal: Goal, source: &GeneratorSuperflatSource, p: &plan::Plan) {
    let programs = words::tiny_programs();
    let obs = ObservedWorld::new(source);
    let mut sim = start.clone();
    let mut touch = goal.reached(&sim, &sim).then_some(0u64);
    for w in &p.words {
        let prog = &programs[w.program as usize];
        for elapsed in 0..w.ticks {
            let before = sim.bounding_box();
            let (_, inp) = prog.select(&obs, &sim, elapsed, w.args).expect("program row matches");
            law::try_tick(&mut sim, &obs, &inp).expect("law tick ok");
            obs.check().expect("all read cells covered");
            if goal.touched_boxes(before, sim.bounding_box()) {
                touch.get_or_insert(sim.t - start.t);
            }
        }
        assert_eq!(sim.position, w.end.position, "word end pos mismatch at t={}", sim.t);
        assert_eq!(sim.velocity, w.end.velocity, "word end vel mismatch at t={}", sim.t);
    }
    assert_eq!(touch, Some(p.ticks), "goal touch {:?} != predicted {}", touch, p.ticks);
}

#[test]
fn thin_bridge_across_chasm_kill_test_13() {
    let air = world::model(&world::BlockState { name: "minecraft:air".into(), properties: Vec::new() }).unwrap().clone();
    let stone = Block::full();
    let min = [-4, -12, -4];
    let max = [96, 10, 28];
    let mut solids = HashSet::new();
    // Side A platform [0..=8, 0, 0..=20] and Side B platform [76..=84, 0, 0..=20] over deep chasm.
    fill(&mut solids, [0, 0, 0], [8, 0, 20]);
    fill(&mut solids, [76, 0, 0], [84, 0, 20]);
    // 1-block wide bridge along z = 14 spanning x = 9..=75.
    fill(&mut solids, [9, 0, 14], [75, 0, 14]);
    let w = BlockSetWorld { air, stone, min, max, solids };
    let source = GeneratorSuperflatSource::from_world(&w, (min, max), [2, 0, 2]).unwrap();
    let mut start = plan::settle(&plan::Still(&source), [2.5, 1.0, 2.5], 0).unwrap();
    start.saturation = 20.0;
    let goal = Goal { cell: [80, 0, 2], face: Some(Face::Up), settle: false };

    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let step1 = solve_step(&start, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    assert!(matches!(step1.outcome, PlanOutcome::Prefix(_)), "expected streaming Prefix first: {:?}", step1.outcome);

    let full = solve(&start, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p) = full.outcome else {
        panic!("thin_bridge solve failed: {:?} {:?}", full.outcome, full.explanation);
    };
    verify_plan_replay(&start, goal, &source, &p);
}

#[test]
fn long_parkour_streams_prefixes_and_solves_end_to_end() {
    let air = world::model(&world::BlockState { name: "minecraft:air".into(), properties: Vec::new() }).unwrap().clone();
    let stone = Block::full();
    let min = [-4, -8, -4];
    let max = [124, 24, 18];
    let mut solids = HashSet::new();
    fill(&mut solids, [0, 0, 4], [5, 0, 8]);
    // Stepping pillars across a 110+ block parkour with 1-to-2-block air gaps and gentle step-ups.
    let mut x = 7;
    let mut y = 0;
    let mut z = 6;
    while x <= 108 {
        fill(&mut solids, [x, y, z - 1], [x + 2, y, z + 1]);
        x += 4;
        if x % 16 == 7 {
            y = (y + 1).min(6);
        }
        z = if (x / 4) % 2 == 0 { 6 } else { 8 };
    }
    fill(&mut solids, [110, y, 5], [116, y, 9]);
    let w = BlockSetWorld { air, stone, min, max, solids };
    let source = GeneratorSuperflatSource::from_world(&w, (min, max), [2, 0, 6]).unwrap();
    let mut start = plan::settle(&plan::Still(&source), [2.5, 1.0, 6.5], 0).unwrap();
    start.saturation = 20.0;
    let goal = Goal { cell: [113, y, 7], face: Some(Face::Up), settle: false };

    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let t0 = Instant::now();
    let step1 = solve_step(&start, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    let first_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(matches!(step1.outcome, PlanOutcome::Prefix(_)), "expected Prefix for 115-blk parkour: {:?}", step1.outcome);
    if !cfg!(debug_assertions) {
        assert!(first_ms < 150.0, "Window 1 first-move wait took {first_ms:.1} ms");
    }

    let full = solve(&start, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p) = full.outcome else {
        panic!("long_parkour solve failed: {:?} {:?}", full.outcome, full.explanation);
    };
    verify_plan_replay(&start, goal, &source, &p);
}

#[test]
fn far_field_local_section_edit_invalidation() {
    let air = world::model(&world::BlockState { name: "minecraft:air".into(), properties: Vec::new() }).unwrap().clone();
    let stone = Block::full();
    let min = [-4, -12, -4];
    let max = [76, 10, 26];
    let mut solids = HashSet::new();
    fill(&mut solids, [0, 0, 0], [8, 0, 20]);
    fill(&mut solids, [58, 0, 0], [66, 0, 20]);
    // Two bridges: z = 4 and z = 16.
    fill(&mut solids, [9, 0, 4], [57, 0, 4]);
    fill(&mut solids, [9, 0, 16], [57, 0, 16]);
    let w1 = BlockSetWorld { air: air.clone(), stone: stone.clone(), min, max, solids: solids.clone() };
    let source1 = GeneratorSuperflatSource::from_world(&w1, (min, max), [2, 0, 4]).unwrap();
    let mut start = plan::settle(&plan::Still(&source1), [2.5, 1.0, 4.5], 0).unwrap();
    start.saturation = 20.0;
    let goal = Goal { cell: [62, 0, 4], face: Some(Face::Up), settle: false };
    let store = GeneratorStore::default();
    let mut codec = Interner::default();
    let r1 = solve(&start, goal, Kit::BareLegs, &source1, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p1) = r1.outcome else { panic!("initial dual-bridge solve failed: {:?}", r1.outcome); };
    verify_plan_replay(&start, goal, &source1, &p1);

    // Block z = 4 bridge inside section [2, 0, 0] (x=32..34) with a 2-high wall and invalidate only section [2, 0, 0].
    fill(&mut solids, [32, 1, 4], [33, 2, 4]);
    let w2 = BlockSetWorld { air, stone, min, max, solids };
    let source2 = GeneratorSuperflatSource::from_world(&w2, (min, max), [2, 0, 4]).unwrap();
    store.words.borrow_mut().environment_event();
    store.invalidate_far_section([2, 0, 0]);
    let r2 = solve(&start, goal, Kit::BareLegs, &source2, &store, &mut codec, u64::MAX);
    let PlanOutcome::Ready(p2) = r2.outcome else { panic!("reroute after bridge block failed: {:?}", r2.outcome); };
    verify_plan_replay(&start, goal, &source2, &p2);
    assert!(p2.ticks > p1.ticks, "rerouted bridge ({}) should be longer than straight bridge ({})", p2.ticks, p1.ticks);
}
