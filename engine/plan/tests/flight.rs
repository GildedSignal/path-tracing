//! Job 3.2b's flight, offline: the planner on Julien's 18 tracks, from A to B
//! (jobs/3.2b-planner/prompt.md, "Measure"). Three ways to a sentence, each from a cold table:
//! - `3`: step 3 alone, A* by cost plus the lower bound, every aim at once;
//! - `5`: step 5 alone, sets backward, points forward to the first sentence, then the slide;
//! - `b`: both, the backward sets order the search, which goes on past the first sentence
//!   while thinking can still pay, then the slide.
//! Thinking stops at `PLAN_CAP` times Julien's best T in wall time (20: the bot's own
//! abandon rule, bot/tracks.md). Counts are exact; times are this machine's under whatever
//! load it had. Printed, never gated: `jobs/3.2b-planner/flight.ps1`.
use std::{collections::HashMap, io::Write, path::PathBuf};
use law::World;
use plan::flight::{track, fly, spread, Row};
use words::tiny_programs;

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

/// The script consumer must receive the same bytes as the flight exporter.
#[test]
#[ignore = "a release flight comparison"]
fn binary_exports_the_flight_script() {
    let dir = std::env::temp_dir().join(format!("plan-track-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let harness = dir.join("harness.jsonl");
    let binary = dir.join("binary.jsonl");
    let line = include_str!("../../world/tests/fixtures/tracks.txt").lines()
        .find(|line| line.split_whitespace().next() == Some("grassjumpeasy")).unwrap();
    let track = track(line).unwrap();
    let row = plan::flight::fly_with_budget(&track, '5', &tiny_programs(), &mut HashMap::new(), 20.0, false, &mut Vec::new(), None, Some(&harness));
    assert!(row.found);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_plan-track"))
        .args(["grassjumpeasy", "--start", "settled", "--mode", "5", "--out"])
        .arg(&binary).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
    assert_eq!(std::fs::read(&harness).unwrap(), std::fs::read(&binary).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}
