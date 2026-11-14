//! Job 3.2b's flight, offline: the planner on Julien's 18 tracks, from A to B
//! (jobs/3.2b-planner/prompt.md, "Measure"). Three ways to a sentence, each from a cold table:
//! - `3`: step 3 alone, A* by cost plus the lower bound, every aim at once;
//! - `5`: step 5 alone, sets backward, points forward to the first sentence, then the slide;
//! - `b`: both, the backward sets order the search, which goes on past the first sentence
//!   while thinking can still pay, then the slide.
//! Thinking stops at `PLAN_CAP` times Julien's best T in wall time (20: the bot's own
//! abandon rule, bot/tracks.md). Counts are exact; times are this machine's under whatever
//! load it had. Printed, never gated: `jobs/3.2b-planner/flight.ps1`.
use std::{collections::HashMap, io::Write, path::PathBuf, time::Instant};
use law::{Block, BlockPos, CollisionContext, World};
use plan::*;
use words::{Program, ProposalTable, tiny_programs};

struct Track { name: String, world: world::WorldSnapshot, read: (BlockPos, BlockPos), body: (BlockPos, BlockPos), a: BlockPos, b: BlockPos, his: Vec<u32>, items: String, pad: i32 }
fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}
fn track(line: &str) -> Result<Track, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut fields = line.split_whitespace();
    let name = fields.next().unwrap().to_owned();
    let n: Vec<i32> = fields.map(|v| v.parse().unwrap()).collect();
    let json = std::fs::read_to_string(root.join(format!("../../lab/tracks/{name}.json"))).map_err(|e| e.to_string())?;
    let (a, b) = (point_of(&json, "A"), point_of(&json, "B"));
    // Julien's times: every finished run of his on this track, and what the best one used.
    let runs = std::fs::read_to_string(root.join(format!("../../lab/tracks/runs/{name}.jsonl"))).unwrap_or_default();
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
        match world::WorldSnapshot::load(root.join("../world/tests/fixtures/world"), min, max) {
            Ok(world) => return Ok(Track { name, world, read: (min, max), body: ([min[0] + margin, min[1] + 1, min[2] + margin], [max[0] - margin, max[1] - 3, max[2] - margin]), a, b, his, items, pad }),
            Err(error) => last = format!("{error:?}"),
        }
    }
    Err(last)
}
fn spread(values: &[f64]) -> String {
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
fn write_script(track: &Track, mode: char, timeline: &Still<'_, world::WorldSnapshot>, programs: &[Program], start: &law::PlayerState, certificate: &Certificate) -> Result<PathBuf, String> {
    let dir = PathBuf::from(std::env::var("PLAN_RECORD").map_err(|e| e.to_string())?);
    let path = dir.join(format!("{}-{}.jsonl", track.name, mode));
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
    Ok(path)
}
#[derive(Default)]
#[allow(dead_code)]
struct Row { found: bool, ticks: u32, clock: Option<u32>, thinking: f64, first: Option<f64> }

fn fly(track: &Track, mode: char, programs: &[Program], reaches: &mut HashMap<usize, Reach>, cap: f64, audit: bool, out: &mut dyn Write) -> Row {
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
    let budget = his.map_or(300.0, |t| t as f64 * 0.05 * cap);
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
            if std::env::var_os("PLAN_RECORD").is_some() {
                match write_script(track, mode, &timeline, programs, &start, certificate) {
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

#[test]
#[ignore = "a release measurement over every track fixture"]
fn the_flight_on_the_tracks() {
    let only = std::env::var("PLAN_TRACKS").ok().map(|names| names.split(',').map(str::to_owned).collect::<Vec<_>>());
    let modes: Vec<char> = std::env::var("PLAN_MODES").unwrap_or("3,5,b".into()).split(',').map(|m| m.chars().next().unwrap()).collect();
    let cap: f64 = std::env::var("PLAN_CAP").ok().map_or(20.0, |c| c.parse().unwrap());
    let audit = std::env::var("PLAN_AUDIT").is_ok();
    let mut out: Box<dyn Write> = match std::env::var("PLAN_RECORD") {
        Ok(dir) => Box::new(Tee(std::fs::File::create(PathBuf::from(dir).join("flight.txt")).unwrap())),
        Err(_) => Box::new(std::io::stdout()),
    };
    let programs = tiny_programs();
    let mut reaches = HashMap::new();
    let mut rows: HashMap<char, Vec<(String, Row, Option<u32>)>> = HashMap::new();
    for line in include_str!("../../world/tests/fixtures/tracks.txt").lines() {
        let name = line.split_whitespace().next().unwrap();
        if only.as_ref().is_some_and(|names| !names.iter().any(|n| n == name)) { continue; }
        let track = match track(line) { Ok(track) => track, Err(error) => { writeln!(out, "[{name}] NO SENTENCE in any way: the track does not load: {error}").unwrap(); continue; } };
        writeln!(out, "[{name}] loaded with pad {}: A {:?} B {:?}, body box {:?}, Julien's T {:?} using {}", track.pad, track.a, track.b, track.body, track.his, if track.items.is_empty() { "{}" } else { &track.items }).unwrap();
        for &mode in &modes {
            // A read past the snapshot panics in the world reader: it ends this way on this track, by name.
            let row = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let mut buffer = Vec::new(); let row = fly(&track, mode, &programs, &mut reaches, cap, audit, &mut buffer); (row, buffer) })) {
                Ok((row, buffer)) => { out.write_all(&buffer).unwrap(); row }
                Err(panic) => { writeln!(out, "[{name}] {mode} NO SENTENCE: ANOMALY the run panicked: {}", panic.downcast_ref::<String>().cloned().or(panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()).unwrap(); Row::default() }
            };
            rows.entry(mode).or_default().push((name.to_owned(), row, track.his.iter().copied().min()));
        }
    }
    for &mode in &modes {
        let rows = &rows[&mode];
        let found: Vec<&(String, Row, Option<u32>)> = rows.iter().filter(|r| r.1.found).collect();
        let thinking: Vec<f64> = rows.iter().map(|r| r.1.thinking).collect();
        let first: Vec<f64> = found.iter().filter_map(|r| r.1.first).collect();
        let ratio: Vec<f64> = found.iter().filter_map(|r| Some(r.1.clock? as f64 / r.2? as f64)).collect();
        let beaten = found.iter().filter(|r| matches!((r.1.clock, r.2), (Some(ours), Some(his)) if ours > his)).map(|r| r.0.as_str()).collect::<Vec<_>>();
        let none = rows.iter().filter(|r| !r.1.found).map(|r| r.0.as_str()).collect::<Vec<_>>();
        writeln!(out, "[all tracks] {mode}: sentences on {} of {}; thinking seconds, every track: {}; seconds to the first sentence, where one came: {}; our T over Julien's on the clock: {}; his T beats ours on {:?}; no sentence on {:?}",
            found.len(), rows.len(), spread(&thinking), spread(&first), spread(&ratio), beaten, none).unwrap();
    }
}
struct Tee(std::fs::File);
impl Write for Tee {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> { std::io::stdout().write_all(bytes)?; self.0.write_all(bytes)?; Ok(bytes.len()) }
    fn flush(&mut self) -> std::io::Result<()> { self.0.flush() }
}

/// A look at the cells around a track's B and A: `PLAN_TRACKS=name cargo test ... probe -- --ignored`.
#[test]
#[ignore = "a probe"]
fn probe_the_cells_around_a_track() {
    let only = std::env::var("PLAN_TRACKS").unwrap_or_default();
    for line in include_str!("../../world/tests/fixtures/tracks.txt").lines().filter(|line| only.split(',').any(|n| line.starts_with(n))) {
        let Ok(track) = track(line) else { continue };
        for (what, at) in [("A", track.a), ("B", track.b)] {
            println!("[{}] {what} {at:?}", track.name);
            for dy in (-2..=3).rev() {
                let mut row = format!("  y{:+}: ", dy);
                for dz in -3..=3 { for dx in -3..=3 {
                    let p = [at[0] + dx, at[1] + dy, at[2] + dz];
                    let inside = (0..3).all(|a| p[a] >= track.read.0[a] && p[a] <= track.read.1[a]);
                    let cell = if !inside { "??".to_string() } else {
                        let block = track.world.block(p);
                        let top = block.shape.coords[1].last().copied();
                        if block.air { " .".into() } else { match top { Some(t) if !block.shape.is_empty() => format!("{:2.0}", t * 16.0), _ => if block.fluid.kind != law::FluidKind::Empty { " ~".into() } else { " '".into() } } }
                    };
                    row += &cell; row.push(' ');
                } row += "| "; }
                println!("{row}");
            }
        }
    }
}
