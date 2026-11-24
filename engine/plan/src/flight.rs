use std::{io::Write, path::PathBuf, time::Instant};
use law::{BlockPos, CollisionContext, World};
use crate::*;
use words::Program;

pub struct Track { pub name: String, pub world: world::WorldSnapshot, pub read: (BlockPos, BlockPos), pub body: (BlockPos, BlockPos), pub a: BlockPos, pub b: BlockPos, pub his: Vec<u32>, pub items: String, pub pad: i32 }
fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}
pub fn track(line: &str) -> Result<Track, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    track_at(line, &root.join("../../lab/tracks"), &root.join("../world/tests/fixtures/world"))
}

/// The same flight input, with saved track records and the dimension view chosen by the caller.
pub fn track_at(line: &str, tracks: &std::path::Path, world: &std::path::Path) -> Result<Track, String> {
    let mut fields = line.split_whitespace();
    let name = fields.next().ok_or("missing track name")?.to_owned();
    if name.is_empty() || !name.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-') { return Err("track names use [a-z0-9_-]+".into()); }
    let n = fields.map(str::parse::<i32>).collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    if n.len() != 6 { return Err("track bounds need six coordinates".into()); }
    let json = std::fs::read_to_string(tracks.join(format!("{name}.json"))).map_err(|e| e.to_string())?;
    let (a, b) = (point_of(&json, "A"), point_of(&json, "B"));
    // Julien's times: every finished run of his on this track, and what the best one used.
    let runs = std::fs::read_to_string(tracks.join(format!("runs/{name}.jsonl"))).unwrap_or_default();
    let mut his = Vec::new();
    let mut items = String::new();
    for run in runs.lines() {
        let Some(at) = run.find("\"T\":") else { continue };
        if !run.contains("\"player\":\"Brain_Forge\"") { continue; }
        let t: u32 = run[at + 4..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().unwrap() + u32::from(!run.contains("\"clockOrigin\":\"lastOnA\""));
        if his.iter().all(|&known| t < known) { items = run.find("\"itemsUsed\":").map(|at| run[at + 12..run[at..].find('}').unwrap() + at + 1].to_owned()).unwrap_or_default(); }
        his.push(t);
    }
    let mut last = String::new();
    for pad in [6, 3, 2, 0] {
        let (min, max) = ([n[0] - pad, n[1], n[2] - pad], [n[3] + pad, n[4] + 8, n[5] + pad]);
        // Three cells inside what was loaded: the law's collision scan reads that far past the body.
        let margin = 3;
        match world::WorldSnapshot::load_for_plan(world, min, max) {
            Ok(world) => return Ok(Track { name, world, read: (min, max), body: ([min[0] + margin, min[1] + 1, min[2] + margin], [max[0] - margin, max[1] - 3, max[2] - margin]), a, b, his, items, pad }),
            Err(error) => last = format!("{error:?}"),
        }
    }
    Err(last)
}
impl Track {
    fn settled_start(&self) -> Result<law::PlayerState,String> {
        let context = CollisionContext { feet_y: self.a[1] as f64 + 2.0, descending: false, fall_distance: 0.0, leather_boots: false };
        let world=source::ObservedWorld::new(&self.world);
        let shape=world.collision_shape(self.a,context);
        world.check()?;
        let top = shape.coords[1].last().copied().ok_or("A has no support surface")?;
        let mut state = law::PlayerState {
            position: law::Vec3::new(self.a[0] as f64 + 0.5, self.a[1] as f64 + top, self.a[2] as f64 + 0.5),
            on_ground: true, main_supporting_block: Some(self.a), saturation: 20.0, ..Default::default()
        };
        state.velocity.y = -state.gravity * f64::from(0.98f32);
        let position = state.position;
        let velocity = state.velocity;
        law::try_tick(&mut state, &world, &Default::default()).map_err(|e| e.0)?;
        world.check()?;
        if !state.on_ground || state.position != position || state.velocity != velocity || state.main_supporting_block != Some(self.a) {
            return Err(format!("no settled support at {:?}", self.a));
        }
        state.t = 0;
        Ok(state)
    }
}
// The recorder's input JSONL, extended with the law's raw start and prediction bits.
fn write_script(path: &std::path::Path, track:&Track, programs: &[Program], start: &law::PlayerState, steps: impl Iterator<Item=(u16, words::Arguments, u32)>, predicted: u32, goal: Goal) -> Result<u32, String> {
    let mut out = Vec::new();
    let mut state = start.clone();
    writeln!(out, "{{\"start\":[{},{},{},{},{}],\"start_bits\":{:?}}}", state.position.x, state.position.y, state.position.z, state.yaw, state.pitch, state.bits()).map_err(|e| e.to_string())?;
    let mut tick = 0;
    let mut last_on_a=0;
    let world=source::ObservedWorld::new(&track.world);
    let mut touch = goal.touched(start,start).then_some(0);
    for (program,args,count) in steps {
        for elapsed in 0..count {
            if state.on_ground && state.main_supporting_block==Some(track.a) {last_on_a=tick;}
            let (_, input) = programs[program as usize].select(&world, &state, elapsed, args).ok_or("no row matches")?;
            let before = state.bounding_box();
            law::try_tick(&mut state, &world, &input).map_err(|e| e.0)?;
            world.check()?;
            if goal.touched_boxes(before,state.bounding_box()) { touch.get_or_insert(tick+1); }
            writeln!(out, "{{\"t\":{},\"forward\":{},\"back\":{},\"left\":{},\"right\":{},\"jump\":{},\"sneak\":{},\"sprint\":{},\"yaw\":{},\"pitch\":{},\"yaw_bits\":{},\"pitch_bits\":{},\"position_bits\":[{},{},{}],\"velocity_bits\":[{},{},{}],\"onGround\":{},\"state_bits\":{:?}}}",
                tick, input.forward, input.back, input.left, input.right, input.jump, input.sneak, input.sprint, input.yaw, input.pitch, input.yaw.to_bits(), input.pitch.to_bits(), state.position.x.to_bits(), state.position.y.to_bits(), state.position.z.to_bits(), state.velocity.x.to_bits(), state.velocity.y.to_bits(), state.velocity.z.to_bits(), state.on_ground, state.bits()).map_err(|e| e.to_string())?;
            tick += 1;
        }
    }
    if touch != Some(predicted) || tick != predicted {
        return Err(format!("B {:?}: predicted {predicted}, replay first touch {touch:?}, length {tick}",goal.cell));
    }
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(tick-last_on_a)
}
pub fn solve_track(track:&Track,path:&std::path::Path)->Result<(u32,u128),String> {
    let start=track.settled_start()?;
    let goal=Goal {cell:track.b};
    let source=source::GeneratorSuperflatSource::snapshot(&track.world,track.a)?;
    let store=store::GeneratorStore::default();
    let mut codec=law::codec::Interner::default();
    let clock=Instant::now();
    let result=solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
    let micros=clock.elapsed().as_micros();
    if std::env::var_os("PLAN_MEASURE_REPEAT").is_some() {
        let prepared=store.preparation_ticks();
        let repeat=Instant::now();
        let again=solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
        let repeat_ns=repeat.elapsed().as_nanos();
        let ticks=|r:&crate::solve::SolveResult|match &r.outcome {PlanOutcome::Ready(p)=>Some(p.ticks),_=>None};
        if ticks(&result)!=ticks(&again) {return Err("same command changed verdict".into());}
        let w = store.words.borrow().work;
        eprintln!("MEASURE first_us={micros} repeat_ns={repeat_ns} prepared={prepared}/{} his={:?} proposals={} hits={} learn_ticks={} certify_ticks={} words={} compounds={} stages={:?}",
            store.preparation_ticks()-prepared, track.his.iter().min(),
            w.proposals_tried, w.proposal_hits, w.learn_ticks, w.certify_ticks, w.words_total, w.compounds_total, result.stages);
    }
    let PlanOutcome::Ready(plan)=result.outcome else {
        return Err(format!("{:?} {:?}",result.outcome,result.explanation));
    };
    let ticks=plan.ticks.try_into().map_err(|_|"script tick width")?;
    let clock=write_script(path,track,&words::tiny_programs(),&start,
        plan.words.iter().map(|w|(w.program,w.args,w.ticks)),ticks,goal)?;
    if std::env::var_os("PLAN_MEASURE_REPEAT").is_some() {
        eprintln!("REPLAY first_touch={ticks} last_on_A={} clock={clock}",ticks-clock);
    }
    Ok((clock,micros))
}
