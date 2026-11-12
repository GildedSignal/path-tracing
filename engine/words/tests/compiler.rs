use std::{path::PathBuf, time::Instant};
use law::{*, footprint::RecordingWorld, state_reads::StateField as F};
use words::*;

struct Flat { ground: Block, air: Block, ceiling: Option<Block> }
impl Flat {
    fn named(name: &str, ceiling: bool) -> Self {
        let block = world::BlockState { name: format!("minecraft:{name}"), properties: Vec::new() };
        Self { ground: world::model(&block).unwrap().clone(), air: Block::default(), ceiling: ceiling.then(Block::full) }
    }
}
impl World for Flat {
    fn block(&self, p: BlockPos) -> &Block {
        if p[1] == -1 { &self.ground } else if p[1] == 2 { self.ceiling.as_ref().unwrap_or(&self.air) } else { &self.air }
    }
    fn has_fluid(&self) -> bool { false }
    fn has_inside_effects(&self) -> bool { false }
}
fn seed() -> PlayerState {
    PlayerState { position: Vec3::new(8.5, 0.0, 8.5), on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..PlayerState::default() }
}
fn held(input: Input) -> Program { Program { rows: vec![Row { when: Predicate::Always, input }] } }
fn sprint() -> Input { Input { forward: true, sprint: true, ..Input::default() } }
fn jumping() -> Program {
    Program { rows: vec![Row { when: Predicate::OnGround, input: Input { jump: true, ..sprint() } }, Row { when: Predicate::Always, input: sprint() }] }
}
fn fed(state: &mut PlayerState) {
    state.food_level = 20;
    state.saturation = 20.0;
    state.exhaustion = 0.0;
    state.food_tick_timer = 0;
}
fn stonesimple() -> world::WorldSnapshot {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../world/tests/fixtures/world");
    world::WorldSnapshot::load(fixture, [1, -64, 22], [9, -54, 59]).unwrap()
}
/// Chains segments, checks each against the law ticked directly, and returns the exits.
fn exits(world: &impl World, mut state: PlayerState, program: &Program, steps: usize) -> Vec<Exit> {
    let args = Arguments::default();
    (0..steps).map(|_| {
        let segment = run_segment(world, &state, program, args).unwrap();
        let mut direct = state.clone();
        for elapsed in 0..segment.ticks {
            let input = program.select(world, &direct, elapsed, args).unwrap().1;
            tick(&mut direct, world, &input);
        }
        assert_eq!(direct.bits(), segment.state.bits());
        state = segment.state;
        segment.exit
    }).collect()
}
fn fired(exits: &[Exit], kind: ExitKind) -> bool { exits.iter().any(|exit| exit.contains(kind)) }

#[test]
fn each_event_kind_fires_on_a_minimal_fixture() {
    let moving = held(sprint());
    let mut world = BoxWorld::default();
    world.fill([6, -1, 6], [10, -1, 15], Block::full());
    let mut start = seed();
    start.position.z = 9.6;
    start.velocity.z = 0.28;
    // Stone to stone: the body's block changes; past the first segment, which finds the
    // support, its contents do not.
    let back = PlayerState { position: Vec3::new(8.5, 0.0, 8.6), ..start.clone() };
    let stone = exits(&world, back.clone(), &moving, 6);
    assert!(fired(&stone, ExitKind::BodyBlock) && !fired(&stone[1..], ExitKind::SupportContents));
    // Stone to ice.
    world.fill([6, -1, 10], [10, -1, 15], Block { friction: 0.98, ..Block::full() });
    let ice = exits(&world, back, &moving, 6);
    assert!(fired(&ice[1..], ExitKind::SupportContents) && fired(&ice, ExitKind::BodyBlock));
    // A carpet step-up: on_ground stays true, the support changes.
    world.set([8, 0, 10], Block::boxes(&[Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0 / 16.0, 1.0)]));
    let carpet = run_segment(&world, &start, &moving, Arguments::default()).unwrap();
    assert!(carpet.state.on_ground && carpet.state.position.y == 1.0 / 16.0);
    assert!(carpet.exit.contains(ExitKind::State(F::main_supporting_block)) && !carpet.exit.contains(ExitKind::State(F::on_ground)));
    // Entering water.
    world.set([8, 0, 10], Block { water_block: true, fluid: Fluid { kind: FluidKind::Water, height: 8.0 / 9.0, ..Fluid::default() }, ..Block::default() });
    assert!(fired(&exits(&world, start, &moving, 6), ExitKind::State(F::in_water)));
    // A ceiling bonk, and the program's row changing on landing.
    let bonk = exits(&Flat::named("stone", true), seed(), &jumping(), 6);
    assert!(fired(&bonk, ExitKind::State(F::vertical_collision)) && fired(&bonk, ExitKind::ActiveRow));
    // Losing sprint at food 6: a guard fails and the flag cuts.
    let mut hungry = seed();
    hungry.set_sprinting(true);
    hungry.food_level = 6;
    let segment = run_segment(&Flat::named("stone", false), &hungry, &moving, Arguments::default()).unwrap();
    assert!(segment.ticks == 1 && segment.exit.contains(ExitKind::State(F::sprinting)) && !segment.guards.hold());
}

#[test]
fn the_class_is_what_the_law_reads_before_writing() {
    let world = Flat::named("stone", false);
    let start = seed();
    let recording = RecordingWorld::new(&world);
    let mut end = start.clone();
    recording.tick(&mut end, &sprint());
    let reads = recording.take().state_reads;
    println!("state fields read on one ground tick: {:?}", reads.fields().collect::<Vec<_>>());
    for field in [F::input, F::yaw, F::pitch, F::movement_this_tick, F::was_eye_in_water, F::was_in_powder_snow, F::speed] {
        assert!(!reads.contains(field), "overwritten before read: {field:?}");
    }
    assert!(reads.contains(F::velocity) && reads.contains(F::position) && reads.contains(F::food_level));
    // Resources above their guards and the position leave the class: no grid.
    let mut same = start.clone();
    same.food_level = 12;
    same.health = 13.0;
    same.exhaustion = 2.3;
    same.position = same.position.plus(Vec3::new(3.0, 0.0, 4.25 + 1.0 / 64.0));
    assert_eq!(class(&start, reads), class(&same, reads));
    // A guard is a class: at food 6 the law stops the sprint (movement.rs:209).
    assert_ne!(class(&start, reads), class(&PlayerState { food_level: 6, ..start.clone() }, reads));
    // Velocity enters by the law's comparisons alone: 0.05 is the back-off step
    // (movement.rs:551), and nothing separates 0.06 from 0.2.
    let moving = |z: f64| class(&PlayerState { velocity: Vec3::new(0.0, 0.0, z), ..seed() }, reads);
    assert_ne!(moving(0.04), moving(0.06));
    assert_eq!(moving(0.06), moving(0.2));
    assert_ne!(moving(0.2), moving(-0.2));
    // The budget refuses; ticks since the start and the support's end are predicates.
    let idle = held(Input::default());
    let settled = run_segment(&world, &start, &idle, Arguments::default()).unwrap().state;
    assert_eq!(run_segment(&world, &settled, &idle, Arguments { max_ticks: 3, ..Arguments::default() }).unwrap_err(), SegmentError::NoEvent { ticks: 3 });
    let timed = Program { rows: vec![Row { when: Predicate::Since(2), input: Input { left: true, ..Input::default() } }, Row { when: Predicate::Always, input: Input::default() }] };
    let segment = run_segment(&world, &settled, &timed, Arguments::default()).unwrap();
    assert!(segment.ticks == 2 && segment.exit.contains(ExitKind::ActiveRow));
    let mut edge = BoxWorld::default();
    edge.fill([7, -1, 7], [9, -1, 9], Block::full());
    let mut leaving = start.clone();
    leaving.position.z = 10.1;
    leaving.velocity.z = 0.5;
    let takeoff = Program { rows: vec![Row { when: Predicate::SupportEnds, input: Input { jump: true, ..Input::default() } }, Row { when: Predicate::Always, input: Input::default() }] };
    let recording = RecordingWorld::new(&edge);
    assert_eq!(takeoff.select(&recording, &leaving, 0, Arguments::default()).unwrap().0, 0);
    assert!(!recording.take().reads.is_empty());
    // 54 tiny programs; 64 aims, each on its own sine-table entry.
    assert_eq!(tiny_programs().len(), 54);
    let sine_table = include_bytes!("../../law/src/sin.bin");
    let entry = |index: usize| u32::from_le_bytes(sine_table[index * 4..index * 4 + 4].try_into().unwrap());
    for yaw_index in 0..64 {
        let radians = (Arguments { yaw_index, ..Arguments::default() }.yaw() * 0.017453292f32) as f64;
        let index = yaw_index as usize * 1024;
        assert_eq!(law::math::sin(radians).to_bits(), entry(index));
        assert_eq!(law::math::cos(radians).to_bits(), entry((index + 16384) % 65536));
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "attractor enumeration is a release run")]
fn gaits_are_exact_recurrences_and_the_head_hitter_is_the_speed_of_light() {
    for (name, block, ceiling) in [("stone", "stone", false), ("ice", "ice", false), ("blue_ice", "blue_ice", false),
        ("slime", "slime_block", false), ("soul_sand", "soul_sand", false), ("honey", "honey_block", false),
        ("stone_ceiling", "stone", true), ("blue_ice_ceiling", "blue_ice", true)] {
        let world = Flat::named(block, ceiling);
        let clock = Instant::now();
        let mut report = discover_gaits(&world, &seed(), &tiny_programs(), Arguments::default(), 20000);
        let fastest = report.gaits.iter().max_by(|a, b| a.speed.total_cmp(&b.speed)).unwrap().clone();
        let mut periods = report.gaits.iter().map(|gait| gait.ticks).collect::<Vec<_>>();
        periods.sort();
        println!("gaits ground={name} count={} repeats={} refusals={} unresolved={:?} cycles kept={} fastest={:.6} b/s period={} ticks cycle={:.16} blocks; periods={periods:?}; {:.1}s",
            report.gaits.len(), report.repeats.len(), report.refusals.len(), report.unresolved, report.cycles(), fastest.speed, fastest.ticks,
            fastest.displacement.horizontal_sqr().sqrt(), clock.elapsed().as_secs_f64());
        assert!(report.refusals.is_empty(), "{name} {:?}", report.refusals);
        assert!(report.unresolved.is_empty(), "{name} {:?}", report.unresolved);
        // Each gait, ticked from its start for one period: every field but position has the same bits, and
        // position has advanced by the displacement. The cycle is kept once and found again from its start.
        assert_eq!(report.cycles(), report.gaits.len());
        for gait in report.gaits.clone() {
            let mut state = gait.start.clone();
            for increment in &gait.increments {
                fed(&mut state);
                let from = state.position;
                let input = gait.program.select(&world, &state, gait.elapsed, gait.arguments).unwrap().1;
                tick(&mut state, &world, &input);
                assert_eq!(state.position.minus(from), *increment);
            }
            fed(&mut state);
            assert_eq!(recurrence(&state, 0), recurrence(&gait.start, 0), "{name}");
            assert_eq!(state.position.minus(gait.start.position), gait.displacement);
            // On the ground some tick of every cycle is clamped in y, and the body rests on the plane.
            assert!(gait.clamped.len() == gait.increments.len() && gait.clamped.iter().any(|axes| axes[1]), "{name}");
            assert!(report.gaits_at(&world, &state, 0).iter().any(|found| found.program == gait.program && found.ticks == gait.ticks), "{name}");
        }
        // The fastest gait is the sprint-jump: straight, sprinting, jump held, never sneaking.
        if name == "stone" || ceiling {
            assert!(fastest.program.rows[3].input == sprint() && fastest.start.sprinting && !fastest.start.shift_key_down && fastest.ticks > 1, "{name}");
        }
        if name == "stone" { assert_eq!(format!("{:.6}", fastest.speed), "7.126767"); }
        // Under a 2-block ceiling it is the head-hitter, cut by the ceiling every third tick. On blue ice it is
        // lab/wiki/speed-of-light-window.md: 19.902885797033985 b/s, a 3-tick orbit of 2.9854328695550976 blocks.
        if ceiling { assert_eq!(fastest.ticks, 3, "{name}"); }
        if name == "blue_ice_ceiling" { assert_eq!(format!("{:.6}", fastest.speed), "19.902886"); }
    }
}

fn place(state: &PlayerState, block: BlockPos) -> PlayerState {
    let shift: [i32; 3] = std::array::from_fn(|a| block[a] - body_block(state)[a]);
    let mut placed = state.clone();
    placed.position = placed.position.plus(Vec3::new(shift[0] as f64, shift[1] as f64, shift[2] as f64));
    placed.main_supporting_block = placed.main_supporting_block.map(|p| std::array::from_fn(|a| p[a] + shift[a]));
    placed
}
fn percentile(sorted: &[f64], p: f64) -> f64 { sorted[((sorted.len() - 1) as f64 * p).round() as usize] }

#[test]
#[cfg_attr(debug_assertions, ignore = "the bench is a release run")]
fn stonesimple_a_second_column_hits_and_replays() {
    let world = stonesimple();
    let (program, args) = (jumping(), Arguments::default());
    let report = discover_gaits(&Flat::named("stone", false), &seed(), std::slice::from_ref(&program), args, 20000);
    let gait = &report.gaits[0];
    // The first run, from one column, compiles the table.
    let mut table = ProposalTable::default();
    let mut state = place(&gait.start, [5, -60, 28]);
    let mut classes = Vec::new();
    for _ in 0..40 {
        let segment = run_segment(&world, &state, &program, args).unwrap();
        classes.push(class(&state, segment.footprint.state_reads));
        table.learn(&world, &state, &program, args, &segment).unwrap();
        state = segment.state;
    }
    // The second run, from another column, takes every segment from a hit and replays it.
    let mut state = place(&gait.start, [7, -60, 29]);
    let (mut starts, mut ticks, mut as_proposed) = (Vec::new(), 0, 0);
    for expected in &classes {
        let proposals = table.proposals(&world, &state);
        let proposal = *proposals.iter().find(|p| table.program(table.proposal(p.id).program) == &program).expect("a hit");
        let certified = table.replay(&world, &state, proposal).unwrap();
        as_proposed += certified.as_proposed as u32;
        assert_eq!(&class(&state, certified.segment.footprint.state_reads), expected);
        assert_eq!(certified.segment.state.bits(), run_segment(&world, &state, &program, args).unwrap().state.bits());
        ticks += certified.segment.ticks;
        starts.push((state, proposal, certified.segment.ticks));
        state = certified.segment.state;
    }
    // Every surprise left its witness, and the witness replays to the outcome it recorded.
    let c = table.counters;
    assert_eq!((c.hits, c.misses, c.refusals, c.surprises as u32, table.witnesses.len() as u32), (40, 0, 0, 40 - as_proposed, 40 - as_proposed));
    for witness in &table.witnesses {
        let again = run_segment(&world, &witness.start, table.program(witness.program), witness.arguments).unwrap();
        assert!(Outcome::of(&again) == witness.actual && witness.actual != witness.predicted);
    }
    println!("stonesimple second column: hits 40/40 segments, {as_proposed}/40 as proposed, {ticks}/{ticks} ticks, {:.2} ticks/segment, classes={} proposals={} masks={}",
        ticks as f64 / 40.0, table.classes().len(), table.all().len(), table.masks().len());
    // The bench: the same 40 lookups and certifying replays, 100 passes, with what the replay is made of.
    let time = |name: &str, run: &mut dyn FnMut(&PlayerState, Found, u32)| {
        let mut costs = Vec::new();
        for _ in 0..100 {
            for (state, proposal, ticks) in &starts {
                let clock = Instant::now();
                run(state, *proposal, *ticks);
                costs.push(clock.elapsed().as_secs_f64() * 1e6);
            }
        }
        costs.sort_by(f64::total_cmp);
        println!("us/segment {name}: min={:.2} p50={:.2} p90={:.2} p99={:.2} max={:.2} (n={})", costs[0], percentile(&costs, 0.5), percentile(&costs, 0.9),
            percentile(&costs, 0.99), costs[costs.len() - 1], costs.len());
    };
    let table = std::cell::RefCell::new(table);
    time("lookup + replay", &mut |state, proposal, _| {
        let found = table.borrow_mut().proposals(&world, state);
        std::hint::black_box((found, table.borrow_mut().replay(&world, state, proposal).unwrap()));
    });
    time("  the lookup", &mut |state, _, _| { std::hint::black_box(table.borrow_mut().proposals(&world, state)); });
    time("  the replay", &mut |state, proposal, _| { std::hint::black_box(table.borrow_mut().replay(&world, state, proposal).unwrap()); });
    time("    the recorded segment", &mut |state, _, _| { std::hint::black_box(run_segment(&world, state, &program, args).unwrap()); });
    time("    the law alone, unrecorded", &mut |state, _, ticks| {
        let mut direct = state.clone();
        for elapsed in 0..ticks {
            let input = program.select(&world, &direct, elapsed, args).unwrap().1;
            tick(&mut direct, &world, &input);
        }
        std::hint::black_box(direct);
    });
}

// The hit rates from arbitrary offsets, the class against version 1's, are in tests/vocabulary.rs.

#[test]
fn the_class_proposes_across_resources_and_a_changed_world_only_reorders() {
    let world = Flat::named("stone", false);
    let (start, program, args) = (seed(), jumping(), Arguments::default());
    let segment = run_segment(&world, &start, &program, args).unwrap();
    let mut table = ProposalTable::default();
    table.learn(&world, &start, &program, args, &segment).unwrap();
    // Another stomach: the same class proposes, the replay carries the real resources.
    let mut tired = start.clone();
    tired.exhaustion = 1.25;
    let proposals = table.proposals(&world, &tired);
    assert!(proposals[0].here);
    let certified = table.replay(&world, &tired, proposals[0]).unwrap();
    assert!(certified.as_proposed);
    assert_ne!(certified.segment.state.exhaustion, segment.state.exhaustion);
    assert_eq!(certified.segment.uses.food, 0);
    // A certificate's key is computed only when asked, in the store the caller names.
    let mut store = law::pattern::PatternTable::<()>::default();
    certified.key(&world, &tired, &mut store, ()).unwrap();
    assert!(store.lookup(&world, &tired).is_some() && store.lookup(&world, &start).is_none());
    // The same class over ice: the world part differs, so the proposal comes after any that
    // holds here, and still comes. Its replay there takes the ice as its world part.
    let ice = Flat::named("ice", false);
    let found = table.proposals(&ice, &start);
    assert!(found.len() == 1 && !found[0].here);
    table.replay(&ice, &start, found[0]).unwrap();
    assert!(table.proposals(&ice, &start)[0].here && !table.proposals(&world, &start)[0].here);
    // Inventing: every program at the aim, each distinct piece learned once; the audit of
    // the hit that follows finds nothing better.
    let (found, ids) = table.invent(&world, &start, &tiny_programs(), &[args]);
    assert_eq!((found.runs, found.refusals, found.pieces.len() as u32 + found.duplicates), (54, 0, 54));
    assert!(ids.iter().all(Option::is_some));
    let proposals = table.proposals(&world, &start);
    assert!(proposals.len() >= found.pieces.len());
    let hit: Vec<_> = proposals.iter().map(|&p| (p, table.replay(&world, &start, p).unwrap())).collect();
    let (regret, _) = table.regret(&world, &start, &hit, &tiny_programs(), &[args]);
    assert_eq!(regret, Regret::default());
    println!("inventing from rest on stone: {} runs, {} distinct pieces, {} proposals", found.runs, found.pieces.len(), proposals.len());
    let width = width(&world, &start, &program, args, &segment);
    println!("width of the first jump segment: {width:?}");
    assert!(!width.late);
}
