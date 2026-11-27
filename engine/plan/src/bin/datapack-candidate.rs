//! Propose and replay a sentence against the saved terrain. The detached game
//! can judge the command effects on these movement samples.
use law::{Input, player::GameType};
use plan::{
    Face, Goal, Kit, PlanOutcome, flight::settled_start,
    source::{GeneratorSuperflatSource, ObservedWorld}, store::GeneratorStore,
};
use std::{fmt::Write, path::PathBuf, process::ExitCode};
use world::datapack::DatapackSnapshot;

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("expected world path")?);
    let samples = PathBuf::from(args.next().ok_or("expected movement sample path")?);
    let coords: Vec<i32> = args.map(|value| {
        value.to_str().and_then(|n| n.parse().ok()).ok_or("expected integer cell coordinate")
    }).collect::<Result<_, _>>()?;
    let &[ax, ay, az, bx, by, bz, lx, ly, lz, hx, hy, hz] = coords.as_slice() else {
        return Err("usage: datapack-candidate WORLD SAMPLES AX AY AZ BX BY BZ LO_X LO_Y LO_Z HI_X HI_Y HI_Z".into());
    };

    let (a, b) = ([ax, ay, az], [bx, by, bz]);
    let snapshot = DatapackSnapshot::load(root, [lx, ly, lz], [hx, hy, hz]).map_err(|e| e.to_string())?;
    let terrain = snapshot.saved_terrain();
    let mut state = settled_start(terrain, a)?;
    state.set_game_mode(GameType::Adventure);
    let goal = Goal { cell: b, face: Some(Face::Up), settle: true };
    let floor = goal.lower_bound(&state);
    let source = GeneratorSuperflatSource::snapshot(terrain, a)?;
    let mut codec = law::codec::Interner::default();
    let store = GeneratorStore::default();
    let clock = std::time::Instant::now();
    let result = plan::solve(&state, goal, Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
    let elapsed = clock.elapsed();
    let PlanOutcome::Ready(plan) = result.outcome else {
        return Err(format!("{:?} {:?}", result.outcome, result.explanation));
    };

    let world = ObservedWorld::new(terrain);
    let programs = words::tiny_programs();
    let mut rows = String::new();
    let mut step = |state: &mut law::PlayerState, input: Input| -> Result<(), String> {
        law::try_tick(state, &world, &input).map_err(|e| e.0)?;
        world.check()?;
        let p = state.position;
        writeln!(rows, "{} {} {} {} {} {} {}", state.t, p.x, p.y, p.z,
            state.yaw, state.pitch, state.on_ground).map_err(|e| e.to_string())
    };
    for word in &plan.words {
        for elapsed in 0..word.ticks {
            let (_, input) = programs[word.program as usize].select(&world, &state, elapsed, word.args)
                .ok_or("no word row matches during terrain replay")?;
            step(&mut state, input)?;
        }
    }
    if state.t != u64::from(plan.ticks) || !goal.reached_boxes(state.bounding_box(), &state) {
        return Err("the candidate did not replay to B".into());
    }
    std::fs::write(samples, rows).map_err(|e| e.to_string())?;
    println!("TERRAIN CANDIDATE {} ticks, floor {floor}, {} words, thinking {} us, A support {a:?}, B support {b:?}",
        plan.ticks, plan.words.len(), elapsed.as_micros());
    for word in plan.words {
        println!("program={} count={} args={:?}", word.program, word.ticks, word.args);
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("datapack-candidate: {error}");
            ExitCode::FAILURE
        }
    }
}
