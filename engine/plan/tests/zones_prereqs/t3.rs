//! U1/P3 falsifier: do adjacent yaw counts suffice for the mandated floor goal?
//! Counts alone still miss: keep that regression alongside the corrected acceptance.
use law::{codec, state_reads::{StateField as F, StateReads}, PlayerState, World};
use std::{collections::HashMap, time::Instant};
use words::{Arguments, Program};

const AXIS: u16 = 49152;
const TARGET: plan::Goal = plan::Goal { cell: [1_000_000, -61, 0] };

fn args(index: u16) -> Arguments {
    let radians = (index as f64 + 0.5) * std::f64::consts::TAU / 65536.0;
    Arguments { refined_yaw: Some((radians / 0.017453292f32 as f64) as f32), ..Default::default() }
}
fn motor(s: &PlayerState) -> Vec<u64> {
    let mut words = Vec::new();
    for &field in F::ALL {
        match field {
            F::t | F::main_supporting_block | F::saturation | F::exhaustion | F::food_tick_timer => (),
            // Retain the vertical phase, which is exactly what this premise misses.
            F::position => words.push(s.position.y.to_bits()),
            F::food_level | F::health => words.extend_from_slice(
                &codec::state_answers(s, StateReads(1u128 << field as u8))),
            _ => s.push_field_bits(field, &mut words),
        }
    }
    words
}
fn setup() -> (law::BoxWorld, PlayerState, Program) {
    let lines = include_str!("../../../world/tests/fixtures/tracks.txt");
    let track = plan::flight::track(lines.lines().find(|l| l.starts_with("straight100 ")).unwrap()).unwrap();
    let stone = track.world.block(track.a).clone();
    // fill stores a generator record, never a million cells.
    let mut world = law::BoxWorld::default();
    world.difficulty = 0;
    world.fill([-2_000_000, -61, -2_000_000], [2_000_000, -61, 2_000_000], stone);
    let start = plan::settle(&plan::Still(&world), [0.5, -60.0, 0.5], 0).unwrap();
    let bits = start.bits();
    let start = PlayerState::from_observed_bits(&bits, &vec![true; bits.len()]).unwrap();
    (world, start, words::tiny_programs().remove(30)) // P1's F S j a s n.
}
struct Prepared { states: Vec<PlayerState>, dx: Vec<f64>, first: usize, period: usize }
fn prepare(world: &impl World, start: &PlayerState, program: &Program, aim: Arguments) -> Prepared {
    let mut s = start.clone();
    let mut states = vec![s.clone()];
    let mut dx = Vec::new();
    let mut seen = HashMap::new();
    seen.insert(motor(&s), 0);
    let (first, period) = loop {
        let input = program.select(world, &s, (states.len()-1) as u32, aim).unwrap().1;
        let (wanted, moved) = law::tick_with_collision(&mut s, world, &input).unwrap();
        assert_eq!((wanted.x.to_bits(), wanted.z.to_bits()), (moved.x.to_bits(), moved.z.to_bits()));
        dx.push(moved.x);
        states.push(s.clone());
        if let Some(&n) = seen.get(&motor(&s)) { break (n, states.len()-1-n); }
        assert!(states.len() < 4096, "P1 preparation budget exhausted");
        seen.insert(motor(&s), states.len()-1);
    };
    assert!(dx.iter().all(|&d| d > 0.0));
    Prepared { states, dx, first, period }
}
impl Prepared {
    fn at(&self, tick: u64) -> PlayerState {
        if tick < self.states.len() as u64 { return self.states[tick as usize].clone(); }
        let n = tick-self.first as u64;
        let mut s = self.states[self.first+n as usize%self.period].clone();
        s.position.x = super::t2::fold(self.states[self.first].position.x,
            &self.dx[self.first..self.first+self.period], n).value;
        // Relax z to the goal centre: lack of a lateral solution cannot cause
        // this counterexample. Every lateral offset still fails in x/y.
        s.position.z = TARGET.center().z;
        s.t = tick;
        s
    }
    fn lower_tick(&self, predicate: impl Fn(&PlayerState) -> bool) -> u64 {
        let mut hi = 1;
        while !predicate(&self.at(hi)) { hi *= 2; }
        let mut lo = 0;
        while lo < hi {
            let mid = lo+(hi-lo)/2;
            if predicate(&self.at(mid)) { hi = mid; } else { lo = mid+1; }
        }
        lo
    }
    fn window(&self) -> (u64, u64) {
        // Oracle window only, not a production inverse. Dimensions come from
        // the law, and Goal::touched is the sole acceptance predicate.
        let first = self.lower_tick(|s| s.bounding_box().max.x > TARGET.zone().min.x);
        let past = self.lower_tick(|s| s.bounding_box().min.x >= TARGET.zone().max.x);
        (first, past+1)
    }
    fn hits(&self) -> Vec<u64> {
        let (first, past) = self.window();
        (first..=past).filter(|&t| TARGET.touched(&self.at(t-1), &self.at(t))).collect()
    }
}

#[test]
fn t3_two_yaw() {
    let (source,start)=super::u1::setup();
    let store=plan::store::GeneratorStore::default();let mut codec=law::codec::Interner::default();
    for cell in [[1_000_000,-61,0],[1_000_000,-61,330_000],[1000,-61,1000],[-1000,-61,1000],[-1000,-61,-1000],[1000,-61,-1000],[0,-61,-1000]] {
        let goal=plan::Goal {cell};
        let result=plan::solve(&start,goal,plan::Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
        let p=super::u1::ready(result);
        super::u1::replay(&source,&start,&p,goal);
        println!("T3_NATIVE goal={cell:?} ticks={} words={} lattice={:?}",p.ticks,p.words.len(),p.lattice_cost);
    }
    #[cfg(not(debug_assertions))]
    {
        let (world,start,program)=setup();
        let prepared=prepare(&world,&start,&program,args(AXIS));
        super::t3_single::all_native_yaws_miss(&world,&start,&program,prepared.period);
    }
}

fn xy(s: &PlayerState) -> [u64; 7] {
    [s.position.x.to_bits(), s.position.y.to_bits(), s.velocity.x.to_bits(), s.velocity.y.to_bits(),
        s.on_ground as u64, s.no_jump_delay as u64, s.fall_distance.to_bits()]
}
#[test]
fn u1_two_yaw_counterexample() {
    let (world, start, program) = setup();
    assert_eq!(world.difficulty(), 0);
    let prepared = prepare(&world, &start, &program, args(AXIS));
    println!("U1_PREMISE first={} period={} preparation_ticks={} start_bits={:?}",
        prepared.first, prepared.period, prepared.dx.len(), start.bits());
    // Exact native coefficients, not a small-angle approximation. Both adjacent
    // directions give sin=-1, so neither provides a longitudinal control.
    let mut switch_ticks = 0;
    let mut switches = 0;
    let mut durations = Vec::new();
    for index in [AXIS-1, AXIS, AXIS+1] {
        let radians = (args(index).yaw()*0.017453292f32) as f64;
        assert_eq!(law::math::sin(radians).to_bits(), (-1.0f32).to_bits());
        let other = prepare(&world, &start, &program, args(index));
        assert_eq!(other.period, prepared.period);
        for phase in 0..other.period {
            assert_eq!(other.dx[other.first+phase].to_bits(),
                prepared.dx[prepared.first+(other.first-prepared.first+phase)%prepared.period].to_bits());
        }
        for n in 0..other.states.len() { assert_eq!(xy(&other.states[n]), xy(&prepared.at(n as u64))); }
        println!("U1_COEFFICIENT yaw={index} sin_bits={} cos_bits={} transient={} period={}",
            law::math::sin(radians).to_bits(), law::math::cos(radians).to_bits(), other.first, other.period);
        if index == AXIS { continue; }
        // Both switch directions at every cycle phase. Carry the real motor
        // state and run to exact recurrence; never rotate velocity instantly.
        for (source, to) in [(&prepared, index), (&other, AXIS)] {
            for phase in 0..source.period {
                let mut switched = source.states[source.first+phase].clone();
                let mut seen = HashMap::new();
                seen.insert(motor(&switched), 0);
                let mut n = 0;
                loop {
                    let input = program.select(&world, &switched, n, args(to)).unwrap().1;
                    law::try_tick(&mut switched, &world, &input).unwrap();
                    n += 1;
                    assert_eq!(xy(&switched), xy(&prepared.at((source.first+phase) as u64+n as u64)),
                        "switch to yaw={to} phase={phase} tick={n}");
                    if seen.insert(motor(&switched), n).is_some() { break; }
                }
                switches += 1;
                switch_ticks += n;
                durations.push(n);
            }
        }
    }
    let (first, past) = prepared.window();
    assert!(prepared.hits().is_empty());
    println!("U1_SWITCHES cases={switches} law_ticks={switch_ticks} xy_equal=true");
    durations.sort_unstable();
    println!("U1_SWITCH_DURATION min={} median={} p99={} max={}",
        durations[0], durations[durations.len()/2], durations[durations.len()-1], durations[durations.len()-1]);
    println!("U1_WINDOW first={first} past={past} hits=0");
    let mut min_vertical_gap = f64::INFINITY;
    for t in first..=past {
        let (a,b) = (prepared.at(t-1), prepared.at(t));
        let tube = plan::goal::hull(a.bounding_box(), b.bounding_box());
        if tube.max.x > TARGET.zone().min.x && tube.min.x < TARGET.zone().max.x {
            min_vertical_gap = min_vertical_gap.min(tube.min.y-TARGET.zone().max.y);
        }
        println!("U1_WITNESS tick={t} before={:?} after={:?} touch={}", a.position, b.position, TARGET.touched(&a,&b));
    }
    assert!(min_vertical_gap > 0.0);
    println!("U1_MARGIN vertical_gap={min_vertical_gap:?}");
    // Independent full streaming oracle. It never reads a P2 map, and tests
    // Goal::touched at every tick before the body has passed B.
    let mut reference = start.clone();
    let mut native_hits = Vec::new();
    let mut witness_states = 0;
    let mut release = first-1;
    while !prepared.at(release).on_ground { release -= 1; }
    let mut release_state = None;
    for n in 1..=past {
        let before = reference.clone();
        let input = program.select(&world, &reference, (n-1) as u32, args(AXIS)).unwrap().1;
        law::try_tick(&mut reference, &world, &input).unwrap();
        if n == release { release_state = Some(reference.clone()); }
        if TARGET.touched(&before, &reference) { native_hits.push(n); }
        if n >= first-1 {
            assert_eq!(xy(&reference), xy(&prepared.at(n)), "independent stream tick {n}");
            witness_states += 1;
        }
    }
    assert!(native_hits.is_empty());
    assert!(reference.bounding_box().min.x >= TARGET.zone().max.x);
    println!("U1_STREAM law_ticks={past} witness_states={witness_states} hits=0");
    // Positive control: the same goal is reachable by a word change. This is
    // evidence for re-deriving the boundary map, not a production special case.
    let mut finish = release_state.unwrap();
    println!("U1_CONTROL_START tick={release} state_bits={:?}", finish.bits());
    let mut finish_tick = release;
    let mut finish_hit = None;
    let terminal_program = words::tiny_programs().remove(27); // F S n, an existing word.
    while finish.position.x <= reference.position.x {
        let before = finish.clone();
        let input = terminal_program.select(&world, &finish, 0, args(AXIS)).unwrap().1;
        law::try_tick(&mut finish, &world, &input).unwrap();
        finish_tick += 1;
        if TARGET.touched(&before, &finish) { finish_hit = Some(finish_tick); break; }
    }
    assert!(finish_hit.is_some());
    println!("U1_CONTROL release_tick={release} arrival_tick={} goal_touched=true", finish_hit.unwrap());
    // Falsifier-query latency only. These are not solve measurements. Fixture
    // IO/world creation and the independent streaming witness are excluded.
    for cold in [true, false] {
        let mut timings = Vec::new();
        for _ in 0..if cfg!(debug_assertions) {3} else {100} {
            let clock = Instant::now();
            let fresh;
            let p = if cold { fresh = prepare(&world, &start, &program, args(AXIS)); &fresh } else { &prepared };
            assert!(std::hint::black_box(p.hits()).is_empty());
            timings.push(clock.elapsed().as_nanos());
        }
        timings.sort_unstable();
        println!("U1_TIMING cold={cold} n={} min_ns={} median_ns={} p99_ns={} max_ns={} sorted_ns={timings:?}",
            timings.len(), timings[0], (timings[(timings.len()-1)/2]+timings[timings.len()/2])/2,
            timings[(99*timings.len()).div_ceil(100)-1], timings[timings.len()-1]);
    }
}
