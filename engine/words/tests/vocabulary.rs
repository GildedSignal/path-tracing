//! Job 3.2c's measure (jobs/3.2c-vocabulary-v2/prompt.md): the class of vocabulary.md
//! version 2 against version 1's, under the same table, on Julien's 18 tracks.
//!
//! The walk protocol is the class lane's (lab/lenses/class-without-numbers.md): 64 seeded
//! runs, each walking the 54 tiny programs from the track's A at an arbitrary offset, 24
//! decisions at most. A decision is what a planner does at a node: look the state up, replay
//! every proposal it gets, and on a miss invent (every program at the walk's aim). The walk
//! then follows its own program's piece. Counts are exact; times are from this machine
//! under whatever load it had, so read them as ratios. Printed, never gated:
//! `jobs/3.2c-vocabulary-v2/measure.ps1`.
use std::{collections::HashSet, io::Write, panic::AssertUnwindSafe, path::PathBuf, time::Instant};
use law::{PlayerState, Vec3, state_reads::{StateField as F, StateReads}};
use words::*;

const RUNS: usize = 64;
const WALK_CAP: usize = 24;
/// One decision in this many is audited: the invention is run beside the table.
const AUDIT_EVERY: u64 = 8;
/// One decision in this many prices the whole miss: 54 programs at 64 aims.
const FULL_EVERY: u64 = 1024;
/// Witnesses written in full per track and class; every one is counted.
const WITNESSES_WRITTEN: u64 = 2000;

/// Version 1's class (vocabulary.md version 1, section 3), kept here as the baseline: the
/// position in three bins per axis cut at 0.3 and 0.7, heights in sixteenths, the velocity
/// to four bits below its leading bit, the support relative to the body's block.
fn v1_class(state: &PlayerState, reads: StateReads) -> Box<[u64]> {
    let block = body_block(state);
    let mut words = vec![reads.0 as u64, (reads.0 >> 64) as u64];
    for field in reads.fields() {
        match field {
            F::t => {}
            _ if field.is_resource() => {}
            F::position => {
                let within = |a: usize, p: f64| p - block[a] as f64;
                let bin = |a: usize, p: f64| [0.3, 0.7].iter().filter(|&&edge| edge <= within(a, p)).count() as u64;
                words.extend([bin(0, state.position.x), (within(1, state.position.y) * 16.0).floor() as u64, bin(2, state.position.z)]);
            }
            F::velocity => {
                let v = state.velocity;
                let largest = v.x.abs().max(v.y.abs()).max(v.z.abs());
                if largest == 0.0 { words.extend([0; 4]); continue; }
                let exponent = (largest.to_bits() >> 52) as i32 - 1023;
                let quantum = 2f64.powi(exponent - 4);
                words.extend([exponent as u64, (v.x / quantum).round() as i64 as u64, (v.y / quantum).round() as i64 as u64, (v.z / quantum).round() as i64 as u64]);
            }
            F::fall_distance => words.push((state.fall_distance * 16.0).floor() as i64 as u64),
            F::main_supporting_block => match state.main_supporting_block {
                Some(support) => words.extend((0..3).map(|a| (support[a] - block[a]) as u64 & 0xffff_ffff)),
                None => words.push(u64::MAX),
            },
            _ => state.push_field_bits(field, &mut words),
        }
    }
    words.into()
}

struct Rng(u64);
impl Rng {
    fn offset(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 - 0.5
    }
}
struct Track { name: String, world: world::WorldSnapshot, lo: [i32; 3], hi: [i32; 3], start: [i32; 3], args: Arguments, pad: i32 }
fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}
/// One of Julien's tracks: its snapshot padded past its box, a start column on A, and the
/// aim of the 64 nearest to B. Walks stay three cells inside what was loaded.
fn track(line: &str) -> Result<Track, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut fields = line.split_whitespace();
    let name = fields.next().unwrap().to_owned();
    let n: Vec<i32> = fields.map(|v| v.parse().unwrap()).collect();
    let json = std::fs::read_to_string(root.join(format!("../../lab/tracks/{name}.json"))).map_err(|e| e.to_string())?;
    let (a, b) = (point_of(&json, "A"), point_of(&json, "B"));
    let degrees = ((a[0] - b[0]) as f64).atan2((b[2] - a[2]) as f64).to_degrees();
    let args = Arguments { yaw_index: (degrees / 5.625).round().rem_euclid(64.0) as u8, ..Arguments::default() };
    let mut last = String::new();
    for pad in [6, 3] {
        let (min, max) = ([n[0] - pad, n[1], n[2] - pad], [n[3] + pad, n[4] + 8, n[5] + pad]);
        match world::WorldSnapshot::load(root.join("../world/tests/fixtures/world"), min, max) {
            Ok(world) => return Ok(Track { name, world, lo: [min[0] + 3, min[1] + 1, min[2] + 3], hi: [max[0] - 3, max[1] - 3, max[2] - 3], start: [a[0], a[1] + 1, a[2]], args, pad }),
            Err(error) => last = format!("{error:?}"),
        }
    }
    Err(last)
}
fn seed_at(column: [i32; 3], rng: &mut Rng) -> PlayerState {
    PlayerState { position: Vec3::new(column[0] as f64 + 0.5 + 0.9 * rng.offset(), column[1] as f64, column[2] as f64 + 0.5 + 0.9 * rng.offset()),
        on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..PlayerState::default() }
}

#[derive(Default)]
struct Stats {
    lookup: Vec<f32>, replay: Vec<f32>, invent: Vec<f32>, search: Vec<f32>, full: Vec<f32>, k: Vec<f32>,
    /// What the table spent, and the distinct certified pieces it handed out, with their ticks.
    table_us: f64, table_pieces: u64, table_ticks: u64,
    /// The same for inventing every piece, on the audited decisions.
    search_us: f64, search_pieces: u64, search_ticks: u64,
    /// Decisions and hits in runs 33 to 64; hits whose first proposal's world part held.
    late: u64, late_hits: u64, here: u64,
    replays: u64, as_proposed: u64,
    /// Hits that held the walk's own piece, and that did not.
    own: u64, absent: u64,
    /// On audited hits: the invention's distinct pieces, and those the hit also certified.
    audited: u64, invented: u64, covered: u64,
    ends: u64, stopped: u64, witnesses: u64, new_outcomes: u64,
}
fn us(clock: Instant) -> f32 { clock.elapsed().as_secs_f64() as f32 * 1e6 }
fn spread(values: &[f32]) -> String {
    if values.is_empty() { return "n=0".to_owned(); }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let at = |p: f64| sorted[((sorted.len() - 1) as f64 * p).round() as usize];
    format!("n={} min={:.1} p50={:.1} p90={:.1} p99={:.1} max={:.1}", sorted.len(), sorted[0], at(0.5), at(0.9), at(0.99), sorted[sorted.len() - 1])
}
fn median(values: &[f32]) -> f32 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    sorted.get(sorted.len() / 2).copied().unwrap_or(f32::NAN)
}

/// One decision at `state`, then the walk's own piece. None: the law refused the walk's piece.
fn decide(track: &Track, table: &mut ProposalTable, stats: &mut Stats, state: &PlayerState, program: usize, programs: &[Program],
    aims: &[Arguments], late: bool, sink: &mut Option<std::fs::File>) -> Option<PlayerState> {
    let (world, args) = (&track.world, track.args);
    let decision = table.counters.decisions;
    let audit = decision % AUDIT_EVERY == 0;
    let clock = Instant::now();
    let found = table.proposals(world, state);
    let mut spent = us(clock);
    stats.lookup.push(spent);
    stats.late += late as u64;
    // The distinct successors this decision certified: exit bits, ticks, proposal.
    let mut successors = Vec::<(Vec<u64>, u32, Option<u32>)>::new();
    let mut searched = None;
    if found.is_empty() {
        if audit {
            let clock = Instant::now();
            let invented = pieces(world, state, programs, &[args]);
            searched = Some((us(clock), invented));
        }
        let clock = Instant::now();
        let (invented, ids) = table.invent(world, state, programs, &[args]);
        stats.invent.push(us(clock));
        spent += us(clock);
        successors.extend(invented.pieces.iter().zip(ids).map(|(piece, id)| (piece.segment.state.bits(), piece.segment.ticks, id)));
    } else {
        stats.k.push(found.len() as f32);
        stats.late_hits += late as u64;
        stats.here += found[0].here as u64;
        let mut hit = Vec::new();
        for &proposal in &found {
            let clock = Instant::now();
            let replayed = table.replay(world, state, proposal);
            stats.replay.push(us(clock));
            spent += us(clock);
            stats.replays += 1;
            if let Ok(certified) = replayed {
                stats.as_proposed += certified.as_proposed as u64;
                hit.push((proposal, certified));
            }
        }
        // Every surprise is counted. Those that showed a proposal a new outcome are the
        // class's news; the first of them are written in full.
        for witness in std::mem::take(&mut table.witnesses) {
            stats.witnesses += 1;
            if table.proposal(witness.proposal).outcomes.iter().any(|&(outcome, seen)| outcome == witness.actual && seen == 1) {
                stats.new_outcomes += 1;
                if let Some(file) = sink.as_mut().filter(|_| stats.new_outcomes <= WITNESSES_WRITTEN) { writeln!(file, "{}", witness.line(table)).unwrap(); }
            }
        }
        let mut seen = HashSet::new();
        for (proposal, certified) in &hit {
            let bits = certified.segment.state.bits();
            if seen.insert(bits.clone()) { successors.push((bits, certified.segment.ticks, Some(proposal.id))); }
        }
        if audit {
            let clock = Instant::now();
            let (_, invented) = table.regret(world, state, &hit, programs, &[args]);
            stats.audited += 1;
            stats.invented += invented.pieces.len() as u64;
            stats.covered += invented.pieces.iter().filter(|piece| seen.contains(&piece.segment.state.bits())).count() as u64;
            searched = Some((us(clock), invented));
        }
    }
    stats.table_us += spent as f64;
    stats.table_pieces += successors.len() as u64;
    stats.table_ticks += successors.iter().map(|s| s.1 as u64).sum::<u64>();
    if let Some((time, invented)) = searched {
        stats.search.push(time);
        stats.search_us += time as f64;
        stats.search_pieces += invented.pieces.len() as u64;
        stats.search_ticks += invented.pieces.iter().map(|piece| piece.segment.ticks as u64).sum::<u64>();
    }
    if decision % FULL_EVERY == 0 && !aims.is_empty() {
        let clock = Instant::now();
        std::hint::black_box(pieces(world, state, programs, aims));
        stats.full.push(us(clock));
    }
    let own = run_segment(world, state, &programs[program], args).ok()?;
    let bits = own.state.bits();
    match successors.iter().find(|successor| successor.0 == bits) {
        Some(&(_, ticks, id)) => {
            if let Some(id) = id { table.walked(id, ticks); }
            stats.own += !found.is_empty() as u64;
        }
        None => stats.absent += !found.is_empty() as u64,
    }
    Some(own.state)
}

fn walk(track: &Track, name: &str, class_of: ClassOf, ordered: bool, programs: &[Program], aims: &[Arguments], record: Option<&PathBuf>) -> (Stats, ProposalTable) {
    let mut table = ProposalTable::new(class_of);
    table.ordered = ordered;
    let mut stats = Stats::default();
    let mut sink = record.map(|dir| std::fs::File::create(dir.join(format!("witnesses-{}-{}.tsv", track.name, name.replace([' ', ','], "-")))).unwrap());
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let inside = |state: &PlayerState| (0..3).all(|a| (track.lo[a]..=track.hi[a]).contains(&body_block(state)[a]));
    for run in 0..RUNS {
        for program in 0..programs.len() {
            let mut state = seed_at(track.start, &mut rng);
            for _ in 0..WALK_CAP {
                if !inside(&state) { break; }
                // A read past the snapshot panics: the walk ends there.
                match std::panic::catch_unwind(AssertUnwindSafe(|| decide(track, &mut table, &mut stats, &state, program, programs, aims, run >= RUNS / 2, &mut sink))) {
                    Ok(Some(next)) => state = next,
                    Ok(None) => { stats.stopped += 1; break; }
                    Err(_) => { stats.ends += 1; break; }
                }
            }
        }
    }
    (stats, table)
}

fn report(track: &str, name: &str, stats: &Stats, table: &ProposalTable) {
    let c = table.counters;
    let pct = |a: u64, b: u64| 100.0 * a as f64 / b.max(1) as f64;
    let tag = format!("[{track}] {name}");
    println!("hit rate {tag}: decisions={} hits={} misses={} hit={:.2}% (runs 33-64: {:.2}% of {}); first proposal's world part held on {:.1}% of hits; as proposed {:.1}% of {} replays",
        c.decisions, c.hits, c.misses, pct(c.hits, c.decisions), pct(stats.late_hits, stats.late), stats.late, pct(stats.here, c.hits), pct(stats.as_proposed, stats.replays), stats.replays);
    println!("{tag} proposals replayed per decision: {}; distinct certified pieces per decision {:.2}", spread(&stats.k), stats.table_pieces as f64 / c.decisions.max(1) as f64);
    println!("{tag} us lookup: {}", spread(&stats.lookup));
    println!("{tag} us replay: {}", spread(&stats.replay));
    println!("{tag} us miss (invent 54 programs at one aim, learn): {}", spread(&stats.invent));
    println!("{tag} us inventing alone, 54 programs at one aim (audited decisions): {}", spread(&stats.search));
    if !stats.full.is_empty() { println!("{tag} us inventing alone, 54 programs at 64 aims: {}", spread(&stats.full)); }
    let (with, without) = (stats.table_us / stats.table_ticks.max(1) as f64, stats.search_us / stats.search_ticks.max(1) as f64);
    println!("{tag} us per certified tick: with the table {with:.3} ({} ticks, {} pieces), inventing every piece {without:.3} ({} ticks, {} pieces); certified ticks per us, table over inventing: {:.2}; \
        per decision: table {:.1} us, inventing {:.1} us",
        stats.table_ticks, stats.table_pieces, stats.search_ticks, stats.search_pieces, without / with, stats.table_us / c.decisions.max(1) as f64, stats.search_us / stats.search.len().max(1) as f64);
    println!("{tag} lock-in: the walk's own piece was absent from {:.2}% of hits ({} of {}); audited hits {}: the hit certified {:.1}% of the invention's distinct pieces; \
        regret {} ticks and {} exits no hit reached, over {} audits",
        pct(stats.absent, stats.own + stats.absent), stats.absent, stats.own + stats.absent, stats.audited, pct(stats.covered, stats.invented), c.regret_ticks, c.regret_unseen, c.regret_samples);
    let proposals = table.all();
    let mut walked: Vec<u64> = proposals.iter().map(|p| p.walked_ticks).filter(|&ticks| ticks > 0).collect();
    walked.sort_by(|a, b| b.cmp(a));
    let total: u64 = walked.iter().sum();
    let share = |fraction: f64| pct(walked.iter().take(((walked.len() as f64 * fraction).ceil() as usize).max(1)).sum::<u64>(), total);
    println!("{tag} vocabulary: classes={} proposals={} masks={} forms={}; proposals per class max={}; walked {} of them, once {}, never {}; of the walked ticks the top 1% carry {:.1}%, the top 10% {:.1}%, the top one {:.1}%",
        table.classes().len(), proposals.len(), table.masks().len(), table.forms(), table.classes().iter().map(|class| class.proposals.len()).max().unwrap_or(0),
        walked.len(), proposals.iter().filter(|p| p.walked == 1).count(), proposals.iter().filter(|p| p.walked == 0).count(), share(0.01), share(0.1), pct(walked.first().copied().unwrap_or(0), total));
    println!("{tag} growth, new proposals per thousand decisions: {:?}", table.growth);
    let surprised = table.classes().iter().filter(|class| class.surprises > 0).count();
    let worst = table.classes().iter().max_by_key(|class| class.surprises).map(|class| (class.surprises, class.replayed, class.proposals.len())).unwrap_or_default();
    println!("{tag} surprises={} (each a witness; {} showed a proposal a new outcome, the first {} of those written); classes that surprised {} of {}; the worst class: {} surprises in {} replays of {} proposals; \
        refusals={} walks ended at the snapshot's edge={} walks the law refused={}",
        c.surprises, stats.new_outcomes, stats.new_outcomes.min(WITNESSES_WRITTEN), surprised, table.classes().len(), worst.0, worst.1, worst.2, c.refusals, stats.ends, stats.stopped);
    assert_eq!(c.surprises, stats.witnesses, "every surprise left a witness");
    assert_eq!(c.replayed, stats.replays);
}

#[test]
#[ignore = "a release measurement over every track fixture"]
fn the_class_of_version_2_against_version_1_on_the_tracks() {
    if std::env::var("VOCABULARY_DEBUG").is_err() { std::panic::set_hook(Box::new(|_| {})); }
    let only = std::env::var("VOCABULARY_TRACKS").ok().map(|names| names.split(',').map(str::to_owned).collect::<Vec<_>>());
    let record = std::env::var("VOCABULARY_RECORD").ok().map(PathBuf::from);
    let programs = tiny_programs();
    let aims: Vec<Arguments> = (0..64).map(|yaw_index| Arguments { yaw_index, ..Arguments::default() }).collect();
    let variants: [(&str, ClassOf, bool); 3] = [("v1 class", v1_class, true), ("v2 class", class, true), ("v2 class, world part not consulted", class, false)];
    let mut totals: Vec<(&str, Stats, Counters, [u64; 3])> = variants.iter().map(|v| (v.0, Stats::default(), Counters::default(), [0; 3])).collect();
    let mut ratios: [Vec<f32>; 3] = Default::default();
    for line in include_str!("../../world/tests/fixtures/tracks.txt").lines() {
        let name = line.split_whitespace().next().unwrap();
        if only.as_ref().is_some_and(|names| !names.iter().any(|n| n == name)) { continue; }
        let track = match track(line) { Ok(track) => track, Err(error) => { println!("[{name}] refused: {error}"); continue; } };
        println!("[{name}] loaded with pad {}, start column {:?}, aim {} of 64", track.pad, track.start, track.args.yaw_index);
        for (variant, &(_, class_of, ordered)) in variants.iter().enumerate() {
            let clock = Instant::now();
            // The whole miss is priced once, beside the second walk: the states are the same.
            let (stats, table) = walk(&track, totals[variant].0, class_of, ordered, &programs, if variant == 1 { &aims } else { &[] }, record.as_ref());
            report(name, totals[variant].0, &stats, &table);
            println!("[{name}] {} walked in {:.1}s", totals[variant].0, clock.elapsed().as_secs_f64());
            if stats.table_ticks > 0 && stats.search_ticks > 0 {
                ratios[variant].push(((stats.search_us / stats.search_ticks as f64) / (stats.table_us / stats.table_ticks as f64)) as f32);
            }
            let (_, total, counters, vocabulary) = &mut totals[variant];
            // Times pool one sample in eight.
            for (pool, values) in [(&mut total.lookup, &stats.lookup), (&mut total.replay, &stats.replay), (&mut total.invent, &stats.invent), (&mut total.search, &stats.search), (&mut total.full, &stats.full), (&mut total.k, &stats.k)] {
                pool.extend(values.iter().step_by(if values.len() > 4096 { 8 } else { 1 }));
            }
            macro_rules! add { ($($field:ident),*) => { $(total.$field += stats.$field;)* } }
            add!(table_us, table_pieces, table_ticks, search_us, search_pieces, search_ticks, late, late_hits, here, replays, as_proposed, own, absent, audited, invented, covered, ends, stopped, witnesses, new_outcomes);
            let c = table.counters;
            macro_rules! count { ($($field:ident),*) => { $(counters.$field += c.$field;)* } }
            count!(decisions, hits, misses, returned, replayed, surprises, refusals, invented, learned, regret_samples, regret_ticks, regret_unseen);
            *vocabulary = [vocabulary[0] + table.classes().len() as u64, vocabulary[1] + table.all().len() as u64, vocabulary[2] + table.all().iter().filter(|p| p.walked == 1).count() as u64];
        }
    }
    for (variant, (name, stats, c, vocabulary)) in totals.iter().enumerate() {
        let pct = |a: u64, b: u64| 100.0 * a as f64 / b.max(1) as f64;
        let (with, without) = (stats.table_us / stats.table_ticks.max(1) as f64, stats.search_us / stats.search_ticks.max(1) as f64);
        println!("hit rate [all tracks] {name}: decisions={} hit={:.2}% (runs 33-64: {:.2}%); as proposed {:.1}% of {} replays; own piece absent from {:.2}% of hits; audited hits certify {:.1}% of the invention's pieces; \
            regret {:.3} ticks and {:.2} unseen exits per audit ({} audits)",
            c.decisions, pct(c.hits, c.decisions), pct(stats.late_hits, stats.late), pct(stats.as_proposed, stats.replays), stats.replays, pct(stats.absent, stats.own + stats.absent),
            pct(stats.covered, stats.invented), c.regret_ticks as f64 / c.regret_samples.max(1) as f64, c.regret_unseen as f64 / c.regret_samples.max(1) as f64, c.regret_samples);
        println!("[all tracks] {name} proposals replayed per decision: {}", spread(&stats.k));
        println!("[all tracks] {name} us lookup: {}; us replay: {}; us miss: {}; us inventing alone at one aim: {}; at 64 aims: {}",
            spread(&stats.lookup), spread(&stats.replay), spread(&stats.invent), spread(&stats.search), spread(&stats.full));
        println!("[all tracks] {name} us per certified tick: table {with:.3}, inventing every piece {without:.3}; certified ticks per us, table over inventing: {:.2} pooled, per track {}; \
            p50 prices: a miss is {:.1} replays, a lookup {:.2} replays",
            without / with, spread(&ratios[variant]), median(&stats.invent) / median(&stats.replay), median(&stats.lookup) / median(&stats.replay));
        println!("[all tracks] {name} vocabulary: classes={} proposals={} walked once={}; surprises={} new outcomes={} refusals={} walk ends={} law refusals of a walk={}",
            vocabulary[0], vocabulary[1], vocabulary[2], c.surprises, stats.new_outcomes, c.refusals, stats.ends, stats.stopped);
    }
}
