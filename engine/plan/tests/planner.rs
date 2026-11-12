//! The planner's own checks: each answers one question, fast.
use law::{Block, BoxWorld, PlayerState};
use plan::*;
use words::{ProposalTable, recurrence, tiny_programs};

fn options(order: Order) -> Options { Options { order, arcs: true, gaits: true, audit: true, improve: false, nodes: 200_000 } }
fn floor(world: &mut BoxWorld, from: [i32; 3], to: [i32; 3]) { world.fill(from, to, Block::full()); }

/// Does a sentence arrive on open ground, certified from the exact start, with its tube in four dimensions?
#[test]
fn a_sentence_on_a_floor_is_certified_and_its_tube_carries_ticks() {
    let mut world = BoxWorld::default();
    floor(&mut world, [-4, -1, -4], [4, -1, 16]);
    let timeline = Still(&world);
    let start = settle(&timeline, [0.5, 0.0, 0.5], 0).unwrap();
    let goal = Goal { cell: [0, -1, 9] };
    let body = ([-3, -3, -3], [3, 4, 15]);
    let reach = Reach::compile(&Block::full(), &tiny_programs(), 20.0, 8.0).unwrap();
    println!("reach: tools={} contacts={} landings={} kept={} compile={:.0} ms", reach.tools, reach.contacts, reach.landings, reach.kept, reach.compile_us / 1e3);
    let field = Field::build(&world, &reach, body, &goal, start.main_supporting_block);
    println!("field: standable={} reached={} goals={} checks={} build={:.1} ms; T at start {:?}", field.standable, field.reached, field.goals, field.checks, field.build_us / 1e3, field.cells.get(&[0, -1, 0]).map(|c| c.t));
    assert!(field.cells.contains_key(&[0, -1, 0]));
    let mut table = ProposalTable::default();
    let problem = Problem { timeline: &timeline, start: start.clone(), goal, body, read: ([-4, -4, -4], [4, 5, 16]), a: Some([0, -1, 0]) };
    let mut planner = Planner::new(problem, tiny_programs(), &mut table, Some(field), options(Order::Field));
    let outcome = planner.run(&mut |seconds, _, _| seconds > 20.0);
    println!("floor: stopped {:?} {:.2}s nearest {:?} stats {:?}", outcome.stopped, outcome.seconds, outcome.nearest.as_ref().map(|n| (n.0.position, n.0.on_ground, n.1, n.2)), (planner.stats.arcs, planner.stats.arc_ticks, planner.stats.solutions.len(), planner.stats.nodes, planner.stats.expansions, planner.stats.replays, planner.stats.misses, planner.stats.dead, planner.stats.dominated, planner.stats.no_event, &planner.stats.refusals, &planner.stats.uncertified));
    let (steps, certificate) = outcome.sentence.expect("a sentence");
    let stats = &planner.stats;
    println!("floor: T={} clock={:?} lower={} steps={} thinking={:.3}s nodes={} replays={} arcs={} arc ticks={} audits={} differ={} compared={:?} differs={:?}",
        certificate.ticks, certificate.clock, outcome.lower, steps.len(), outcome.seconds, stats.nodes, stats.replays, stats.arcs, stats.arc_ticks, stats.audits, stats.audit_differs, stats.compared, stats.differs);
    assert!(outcome.lower <= certificate.ticks);
    assert!(!certificate.tube.is_empty() && certificate.tube.iter().all(|&(tick, _)| tick < certificate.ticks as u64));
    // The arithmetic lane agrees with the law bit for bit wherever it was audited or compared.
    assert_eq!((stats.audit_differs, stats.differs), (0, [0; 4]));
    // The slide never raises T and returns a certified sentence.
    let ((_, slid), report) = slide(&timeline, &tiny_programs(), &start, &goal, Some([0, -1, 0]), (steps, certificate.clone()), &mut || false);
    println!("slide: {} -> {} ticks, {} kept, {} replays, room {:?}", report.before, report.after, report.kept, report.replays, report.room);
    assert!(slid.ticks <= certificate.ticks);
}

/// Does the planner use a platform that exists only on a schedule, at the ticks it exists?
#[test]
fn a_platform_that_exists_only_on_a_schedule_is_crossed_while_it_stands() {
    let (mut gap, mut bridged) = (BoxWorld::default(), BoxWorld::default());
    for world in [&mut gap, &mut bridged] { floor(world, [0, -1, 0], [0, -1, 1]); floor(world, [0, -1, 9], [0, -1, 10]); }
    floor(&mut bridged, [0, -1, 2], [0, -1, 8]);
    let timeline = Scheduled::new(gap.clone()).then(40, bridged).then(200, gap.clone());
    let start = settle(&timeline, [0.5, 0.0, 0.5], 0).unwrap();
    let goal = Goal { cell: [0, -1, 10] };
    let body = ([-2, -3, -2], [2, 3, 12]);
    let reach = Reach::compile(&Block::full(), &tiny_programs(), 16.0, 4.0).unwrap();
    let mut field: Option<Field> = None;
    for frame in timeline.frames() {
        let built = Field::build(frame, &reach, body, &goal, None);
        match &mut field { Some(field) => field.merge(built), None => field = Some(built) }
    }
    let mut table = ProposalTable::default();
    let problem = Problem { timeline: &timeline, start: start.clone(), goal, body, read: ([-4, -5, -4], [4, 5, 14]), a: Some([0, -1, 0]) };
    let mut planner = Planner::new(problem, tiny_programs(), &mut table, field, Options { audit: false, ..options(Order::Field) });
    let outcome = planner.run(&mut |seconds, _, _| seconds > 60.0);
    println!("schedule: stopped {:?} after {:.2}s, nodes={} cut ticks={}", outcome.stopped, outcome.seconds, planner.stats.nodes, planner.stats.cut_ticks);
    let (steps, certificate) = outcome.sentence.expect("a sentence across the platform");
    println!("schedule: T={} steps={}", certificate.ticks, steps.len());
    // Its tube is over the gap only at ticks when the platform stands.
    let over: Vec<u64> = certificate.tube.iter().filter(|(_, cell)| cell[1] == 0 && (3..=7).contains(&cell[2])).map(|&(tick, _)| tick).collect();
    assert!(!over.is_empty() && over.iter().all(|&tick| (40..200).contains(&tick)), "{over:?}");
    // The same steps in the world without the platform do not arrive.
    assert!(certify(&Still(&gap), &tiny_programs(), &start, &steps, &goal, None, false).is_err());
}

/// Does a gait's cycle advance by its increments, bit for bit in position, on uniform ground?
#[test]
fn a_gait_cycle_advances_by_its_increments_on_uniform_ground() {
    let mut world = BoxWorld::default();
    floor(&mut world, [-400, -1, -400], [400, -1, 400]);
    let timeline = Still(&world);
    let fed = |state: &PlayerState| PlayerState { food_level: 20, saturation: 20.0, exhaustion: 0.0, food_tick_timer: 0, ..state.clone() };
    let mut report = Vec::new();
    for (index, program) in tiny_programs().iter().enumerate() {
        let mut states = vec![settle(&timeline, [0.5, 0.0, 0.5], 0).unwrap()];
        let mut state = states[0].clone();
        advance(&timeline, &mut state, program, words::Arguments::default(), 600, &mut |_, after| { states.push(after.clone()); false }).unwrap();
        // The first recurrence, found as the search finds it: an earlier state with the same bits.
        let words: Vec<_> = states.iter().map(|s| recurrence(&fed(s), 0)).collect();
        let Some((from, to)) = (1..states.len()).find_map(|to| (0..to).rev().find(|&from| words[from] == words[to]).map(|from| (from, to))) else { println!("program {index}: no recurrence in 600 ticks; v={:?}", states[600].velocity); report.push((index, None)); continue };
        let recording = law::footprint::RecordingWorld::new(&world);
        let mut replay = states[from].clone();
        for elapsed in 0..(to - from) as u32 { let input = program.select(&recording, &replay, elapsed, words::Arguments::default()).unwrap().1; law::tick(&mut replay, &recording, &input); }
        let Some(cycle) = plan::arith::Cycle::observed(&states[from..=to], recording.take().reads.keys().copied()) else { println!("program {index}: cycle {from}..{to} not taken"); report.push((index, None)); continue };
        assert!(cycle.layered(&world, &states[to], &|_| true));
        let (mut predicted, mut differs, mut ticks) = (states[to].clone(), 0, 0);
        for lap in 0..8 {
            if to + (lap + 1) * (to - from) >= states.len() { break; }
            cycle.advance(&mut predicted, &states[from..=to], &mut |_, _| false);
            let truth = &states[to + (lap + 1) * (to - from)];
            ticks += to - from;
            differs += (predicted.position != truth.position || predicted.velocity != truth.velocity || predicted.main_supporting_block != truth.main_supporting_block || predicted.on_ground != truth.on_ground) as u32;
        }
        report.push((index, Some((from, to - from, ticks, differs))));
    }
    let found: Vec<_> = report.iter().filter_map(|r| r.1.map(|v| (r.0, v))).collect();
    println!("gaits found on a floor within 600 ticks: {} of {} programs; (program, (first tick, period, ticks advanced, laps that differ)): {found:?}", found.len(), report.len());
    assert!(!found.is_empty());
    assert!(found.iter().all(|f| f.1.3 == 0), "the gait lane differs from the law");
}
