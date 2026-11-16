use std::{collections::HashMap, io::Write, path::PathBuf, time::Instant};
use law::{Block, BlockPos, CollisionContext, World};
use crate::*;
use words::{Program, ProposalTable};

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
        let t: u32 = run[at + 4..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap();
        if his.iter().all(|&known| t < known) { items = run.find("\"itemsUsed\":").map(|at| run[at + 12..run[at..].find('}').unwrap() + at + 1].to_owned()).unwrap_or_default(); }
        his.push(t);
    }
    let mut last = String::new();
    for pad in [6, 3, 2, 0] {
        let (min, max) = ([n[0] - pad, n[1], n[2] - pad], [n[3] + pad, n[4] + 8, n[5] + pad]);
        // Three cells inside what was loaded: the law's collision scan reads that far past the body.
        let margin = 3;
        match world::WorldSnapshot::load(world, min, max) {
            Ok(world) => return Ok(Track { name, world, read: (min, max), body: ([min[0] + margin, min[1] + 1, min[2] + margin], [max[0] - margin, max[1] - 3, max[2] - margin]), a, b, his, items, pad }),
            Err(error) => last = format!("{error:?}"),
        }
    }
    Err(last)
}
pub fn spread(values: &[f64]) -> String {
    if values.is_empty() { return "n=0".to_owned(); }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |p: f64| sorted[((sorted.len() - 1) as f64 * p).round() as usize];
    format!("n={} min={:.3} p50={:.3} p90={:.3} p99={:.3} max={:.3}", sorted.len(), sorted[0], at(0.5), at(0.9), at(0.99), sorted[sorted.len() - 1])
}
/// A program by its keys: F forward, L or R strafe, S sprint, then when it jumps and sneaks
/// (n never, a always, g on the ground).
fn label(index: u16) -> String {
    let i = index as usize;
    let when = ["n", "a", "g"];
    format!("F{}{}j{}s{}", ["L", "", "R"][i / 18], if (i / 9) % 2 == 1 { "S" } else { "" }, when[(i / 3) % 3], when[i % 3])
}
fn say(steps: &[Step]) -> String {
    steps.iter().map(|s| format!("{}@{}x{}{}", label(s.program), s.args.yaw_index, s.ticks, match s.lane { Lane::Replay => "", Lane::Arc => "~", Lane::Gait => "*", Lane::Cut => "|" })).collect::<Vec<_>>().join(" ")
}
// The recorder's input JSONL, extended with the law's raw start and prediction bits.
fn write_script(path: &std::path::Path, timeline: &Still<'_, world::WorldSnapshot>, programs: &[Program], start: &law::PlayerState, certificate: &Certificate) -> Result<PathBuf, String> {
    let mut out = Vec::new();
    let mut state = start.clone();
    writeln!(out, "{{\"start\":[{},{},{},{},{}],\"start_bits\":{:?}}}", state.position.x, state.position.y, state.position.z, state.yaw, state.pitch, state.bits()).map_err(|e| e.to_string())?;
    let mut tick = 0;
    for step in &certificate.steps {
        for elapsed in 0..step.ticks {
            let world = timeline.at(state.t);
            let (_, input) = programs[step.program as usize].select(world, &state, elapsed, step.args).ok_or("no row matches")?;
            law::try_tick(&mut state, world, &input).map_err(|e| e.0)?;
            writeln!(out, "{{\"t\":{},\"forward\":{},\"back\":{},\"left\":{},\"right\":{},\"jump\":{},\"sneak\":{},\"sprint\":{},\"yaw\":{},\"pitch\":{},\"yaw_bits\":{},\"pitch_bits\":{},\"position_bits\":[{},{},{}],\"velocity_bits\":[{},{},{}],\"onGround\":{},\"state_bits\":{:?}}}",
                tick, input.forward, input.back, input.left, input.right, input.jump, input.sneak, input.sprint, input.yaw, input.pitch, input.yaw.to_bits(), input.pitch.to_bits(), state.position.x.to_bits(), state.position.y.to_bits(), state.position.z.to_bits(), state.velocity.x.to_bits(), state.velocity.y.to_bits(), state.velocity.z.to_bits(), state.on_ground, state.bits()).map_err(|e| e.to_string())?;
            tick += 1;
        }
    }
    if tick != certificate.ticks || state.bits() != certificate.end.bits() {
        return Err("script replay differs from certificate".into());
    }
    std::fs::write(&path, out).map_err(|e| e.to_string())?;
    Ok(path.to_path_buf())
}
#[derive(Default)]
#[allow(dead_code)]
pub struct Row { pub found: bool, pub ticks: u32, pub clock: Option<u32>, pub thinking: f64, pub first: Option<f64> }

pub fn fly(track: &Track, mode: char, programs: &[Program], reaches: &mut HashMap<usize, Reach>, cap: f64, audit: bool, out: &mut dyn Write) -> Row {
    fly_with_budget(track, mode, programs, reaches, cap, audit, out, None, None)
}

pub fn fly_with_budget(track: &Track, mode: char, programs: &[Program], reaches: &mut HashMap<usize, Reach>, cap: f64, audit: bool, out: &mut dyn Write, seconds: Option<f64>, script: Option<&std::path::Path>) -> Row {
    let name = &track.name;
    let tag = format!("[{name}] {mode}");
    let timeline = Still(&track.world);
    let context = CollisionContext { feet_y: track.a[1] as f64 + 2.0, descending: false, fall_distance: 0.0, leather_boots: false };
    let top = track.world.collision_shape(track.a, context).coords[1].last().copied().unwrap_or(1.0);
    let start = match settle(&timeline, [track.a[0] as f64 + 0.5, track.a[1] as f64 + top, track.a[2] as f64 + 0.5], 0) {
        Ok(start) => start,
        Err(why) => { writeln!(out, "{tag} NO SENTENCE: no start: {why}").unwrap(); return Row::default(); }
    };
    let on_a = start.on_ground && start.main_supporting_block == Some(track.a);
    let goal = Goal { cell: track.b };
    let his = track.his.iter().copied().min();
    let budget = seconds.unwrap_or_else(|| his.map_or(300.0, |t| t as f64 * 0.05 * cap));
    let clock = Instant::now();
    // The backward pass, for the ways that use it. Its reach tables are compiled once per floor.
    let mut field = None;
    let mut compile = 0.0;
    if mode != '3' {
        let floor: &Block = start.main_supporting_block.map_or(track.world.block(track.a), |support| track.world.block(support));
        let (lo, hi) = track.body;
        let extent = (((hi[0] - lo[0] + 1) as f64).powi(2) + ((hi[2] - lo[2] + 1) as f64).powi(2)).sqrt();
        let key = floor as *const Block as usize ^ (extent.to_bits() as usize) ^ ((hi[1] - lo[1]) as usize) << 48;
        if !reaches.contains_key(&key) {
            match Reach::compile(floor, programs, extent, (hi[1] - lo[1] + 1) as f64) {
                Ok(reach) => { compile = reach.compile_us / 1e6; reaches.insert(key, reach); }
                Err(why) => { writeln!(out, "{tag} NO SENTENCE: the reach tables do not compile on A's floor: {why}").unwrap(); return Row::default(); }
            }
        }
        let reach = &reaches[&key];
        let built = Field::build(&track.world, reach, track.body, &goal, start.main_supporting_block, (0, None));
        writeln!(out, "{tag} sets backward: tools={} contacts={} landings={} kept={} compile {:.3}s (once per floor, not in thinking); standable={} reached={} goal cells={} checks={} build {:.3}s; T at A by the sets: {}",
            reach.tools, reach.contacts, reach.landings, reach.kept, compile, built.standable, built.reached, built.goals, built.checks, built.build_us / 1e6,
            start.main_supporting_block.and_then(|s| built.cells.get(&s)).map_or("none: no chain of sets reaches A".to_owned(), |c| c.t.to_string())).unwrap();
        // Where the chain of sets stops short of A: the unreached cells nearest A that stand beside a reached one.
        if start.main_supporting_block.is_none_or(|s| !built.cells.contains_key(&s)) {
            let near = |p: &BlockPos, q: &BlockPos| (p[0] - q[0]).abs() <= 2 && (p[2] - q[2]).abs() <= 2 && (p[1] - q[1]).abs() <= 2;
            let mut edge: Vec<_> = built.unreached.iter().filter_map(|(cell, y)| built.cells.values().filter(|set| near(&set.support, cell)).min_by_key(|set| set.t).map(|set| (*cell, *y, set.support, set.y, set.t))).collect();
            edge.sort_by_key(|e| (e.0[0] - track.a[0]).pow(2) + (e.0[1] - track.a[1]).pow(2) + (e.0[2] - track.a[2]).pow(2));
            for (cell, y, set, set_y, t) in edge.iter().take(4) {
                writeln!(out, "{tag} mirror failed: no tool's free arc or walk from {cell:?} (stand {y:.4}) lands on the set at {set:?} (stand {set_y:.4}, {t} ticks from B): rise {:.4}, {} cells across", set_y - y, (cell[0] - set[0]).abs().max((cell[2] - set[2]).abs())).unwrap();
            }
        }
        field = Some(built);
    }
    let search_start = clock.elapsed().as_secs_f64() - compile;
    let mut table = ProposalTable::default();
    let problem = Problem { timeline: &timeline, start: start.clone(), goal, body: track.body, read: track.read, a: Some(track.a) };
    let options = Options { order: if mode == '3' { Order::Admissible } else { Order::Field }, arcs: true, gaits: true, audit, improve: mode != '5', nodes: 1_500_000 };
    let mut planner = Planner::new(problem, programs.to_vec(), &mut table, field, options);
    // One objective: thinking is ticks too. Past the first sentence, thinking goes on only
    // while it could still pay: the ticks spent since cannot exceed what is left to save.
    let mut first: Option<f64> = None;
    let outcome = planner.run(&mut |seconds, best, lower| {
        if let (Some(_), None) = (best, first) { first = Some(seconds); }
        seconds + search_start > budget || matches!((best, first), (Some(best), Some(first)) if (seconds - first) * 20.0 >= (best - lower) as f64)
    });
    let stats = planner.stats.clone();
    let field = planner.field.take();
    drop(planner);
    let search = outcome.seconds;
    let mut sentence = outcome.sentence.clone();
    let mut slid = None;
    if let (Some(found), true) = (sentence.clone(), mode != '3') {
        let (better, report) = slide(&timeline, programs, &start, &goal, Some(track.a), found, &mut || clock.elapsed().as_secs_f64() - compile > budget);
        sentence = Some(better);
        slid = Some(report);
    }
    let thinking = clock.elapsed().as_secs_f64() - compile;
    let counters = table.counters;
    let pct = |a: u64, b: u64| 100.0 * a as f64 / b.max(1) as f64;
    let ticks = stats.replay_ticks + stats.arc_ticks + stats.gait_ticks + stats.cut_ticks;
    let per_decision: Vec<f64> = stats.per_decision.iter().map(|&k| k as f64).collect();
    let miss: Vec<f64> = stats.miss_us.iter().map(|&us| us as f64).collect();
    writeln!(out, "{tag} search: stopped {:?} after {:.3}s; nodes={} expansions={} decisions={} replays={} ({:.1} us each) misses={} over {} aims, inventing {} runs; dead={} dominated={} discarded by the bound={} no event={}; surprises={} of {} replays; table classes={} proposals={}",
        outcome.stopped, search, stats.nodes, stats.expansions, stats.decisions, stats.replays, stats.replay_us / stats.replays.max(1) as f64, stats.misses, stats.miss_aims, stats.invented_runs,
        stats.dead, stats.dominated, stats.pruned, stats.no_event, counters.surprises, counters.replayed, table.classes().len(), table.all().len()).unwrap();
    writeln!(out, "{tag} proposals replayed per decision: {}; us per miss: {}", spread(&per_decision), spread(&miss)).unwrap();
    writeln!(out, "{tag} ticks the search certified: {ticks} = replay {} ({:.1}%) + arc arithmetic {} ({:.1}%, {} arcs, {:.2} us per tick) + gait arithmetic {} ({} gaits) + cut {}; search us per certified tick {:.3}; arcs audited against a replay: {} ticks, {} differ; step ends compared with the certifying replay by lane [replay, arc, gait, cut]: {:?}, differing {:?}",
        stats.replay_ticks, pct(stats.replay_ticks, ticks), stats.arc_ticks, pct(stats.arc_ticks, ticks), stats.arcs, stats.arc_us / stats.arc_ticks.max(1) as f64, stats.gait_ticks, stats.gaits, stats.cut_ticks,
        search * 1e6 / ticks.max(1) as f64, stats.audits, stats.audit_differs, stats.compared, stats.differs).unwrap();
    for (rule, count, at) in &stats.refusals { writeln!(out, "{tag} the law refused: {rule} x{count}, first at {at:?}").unwrap(); }
    for why in stats.uncertified.iter().take(3) { writeln!(out, "{tag} ANOMALY a chain touched B in the search and its replay did not certify: {why}").unwrap(); }
    if let Some(report) = &slid {
        writeln!(out, "{tag} slide: {} -> {} ticks, {} slides kept in {} passes, {} replays of {} ticks, {:.3}s; joints by slides that still arrive (of 4): {:?}",
            report.before, report.after, report.kept, report.passes, report.replays, report.replayed_ticks, report.seconds,
            (0..=4).map(|room| report.room.iter().filter(|&&r| r == room).count()).collect::<Vec<_>>()).unwrap();
    }
    // Where the mirror failed: sets the backward pass chained that the forward pass never realised.
    if let Some(field) = &field {
        let mut unmet: Vec<_> = field.cells.values().filter(|cell| cell.tries > 0 && cell.arrivals == 0 && cell.via.is_some()).collect();
        unmet.sort_by_key(|cell| std::cmp::Reverse(cell.tries));
        let met = field.cells.values().filter(|cell| cell.arrivals > 0).count();
        writeln!(out, "{tag} mirror: sets the forward pass stood in {}, left for a nearer set {}, never left {}; premises asked at {} joins, broken {}; cells read per set: {}", field.cells.values().filter(|c| c.tries > 0).count(), met, unmet.len(), field.joins, field.broken,
            spread(&field.cells.values().map(|c| c.premises.len() as f64).collect::<Vec<_>>())).unwrap();
        for cell in unmet.iter().take(if sentence.is_some() { 2 } else { 6 }) {
            writeln!(out, "{tag} mirror failed: from {:?} (stand {:.4}) the sets said {} ticks to {:?} (stand {:.4}), a {} of {} ticks after {} run-up contacts, tool {}; {} expansions there, no arrival",
                cell.support, cell.y, cell.t, cell.via.unwrap(), field.cells[&cell.via.unwrap()].y, if cell.hop.2 { "walk" } else { "free arc" }, cell.hop.0, cell.hop.1, cell.hop.3, cell.tries).unwrap();
        }
    }
    let thinking_ticks = (thinking * 20.0).ceil() as u32;
    match &sentence {
        Some((steps, certificate)) => {
            let record_path = std::env::var_os("PLAN_RECORD").map(|dir| PathBuf::from(dir).join(format!("{}-{}.jsonl", track.name, mode)));
            if let Some(path) = script.or(record_path.as_deref()) {
                match write_script(path, &timeline, programs, &start, certificate) {
                    Ok(path) => writeln!(out, "{tag} script: {}", path.display()).unwrap(),
                    Err(error) => panic!("{tag} script failed: {error}"),
                }
            }
            let improvements = stats.solutions.iter().map(|(s, t)| format!("{t}@{s:.3}s")).collect::<Vec<_>>().join(" ");
            writeln!(out, "{tag} SENTENCE certified: T={} ticks from the start state, {} on the track's clock; Julien {} ({}); thinking {:.3}s = {} ticks; sum {} (clock {}); bracket [{} by the speed limit, {}], open list's least cost plus bound {:?}; first sentence after {:?}s; found {}; tube {} cells with ticks, {} cells read; {} steps: {}",
                certificate.ticks, certificate.clock.map_or("none".into(), |c| c.to_string()), his.map_or("none recorded".into(), |t| format!("{t} best of {:?}", track.his)), if track.items.is_empty() { "{}" } else { &track.items },
                thinking, thinking_ticks, certificate.ticks + thinking_ticks, certificate.clock.map_or("none".into(), |c| (c + thinking_ticks).to_string()), outcome.lower, certificate.ticks, outcome.frontier,
                stats.solutions.first().map(|s| s.0), improvements, certificate.tube.len(), certificate.reads, steps.len(), say(steps)).unwrap();
            if let (Some(his), Some(clock)) = (his, certificate.clock) {
                if clock > his { writeln!(out, "{tag} HIS T BEATS OURS by {} ticks on the clock ({} against {}), thinking aside", clock - his, clock, his).unwrap(); }
            }
            Row { found: true, ticks: certificate.ticks, clock: certificate.clock, thinking, first: stats.solutions.first().map(|s| s.0 + search_start) }
        }
        None => {
            let (state, g, anchor) = outcome.nearest.clone().unwrap();
            writeln!(out, "{tag} NO SENTENCE: A {:?}{} to B {:?}; start {:?}; Julien {:?} with {}; thinking {:.3}s of {:.1}s allowed; lower bound {}; tried: {} expansions, {} replays, {} inventions; the state nearest the goal by the order: position {:?} velocity {:?} on_ground={} support {:?} after {} ticks, its set {:?} -> {:?}",
                track.a, if on_a { "" } else { " (the settled start does not rest on A)" }, track.b, [start.position.x, start.position.y, start.position.z], track.his, if track.items.is_empty() { "{}" } else { &track.items },
                thinking, budget, outcome.lower, stats.expansions, stats.replays, stats.misses,
                [state.position.x, state.position.y, state.position.z], [state.velocity.x, state.velocity.y, state.velocity.z], state.on_ground, state.main_supporting_block, g, anchor,
                anchor.and_then(|a| field.as_ref().and_then(|f| f.cells.get(&a)).and_then(|c| c.via))).unwrap();
            Row { thinking, ..Row::default() }
        }
    }
}

