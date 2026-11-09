//! Job 1.2: what a pattern entry costs on Julien's tracks, in bytes and microseconds.
//! The v1 numbers come from jobs/1.2-footprints/v1-cost.rs at commit 8801299.
use std::{mem::size_of, path::PathBuf, time::Instant};
use law::{Input, PlayerState, Vec3, footprint::{Footprint, RecordingWorld}, pattern::{Entry, Key, PatternTable, Shape}};
use world::WorldSnapshot;

fn fixture() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world") }
fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

// What does an entry cost? Printed, not asserted: run in release for the microseconds.
#[test]
fn entry_cost_on_the_tracks() {
    std::panic::set_hook(Box::new(|_| {}));
    let tracks = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lab/tracks");
    let mut snapshots = Vec::new();
    for line in include_str!("fixtures/tracks.txt").lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let json = std::fs::read_to_string(tracks.join(format!("{}.json", f[0]))).unwrap();
        snapshots.push((WorldSnapshot::load(fixture(), [n(1), n(2), n(3)], [n(4), n(5), n(6)]).unwrap(), point_of(&json, "A"), point_of(&json, "B")));
    }
    for ticks in [1, 10] {
        // Words first, then the table's work on them, timed.
        let mut words = Vec::new();
        for (index, (snapshot, a, b)) in snapshots.iter().enumerate() {
            let toward = ((a[0]-b[0]) as f32).atan2((b[2]-a[2]) as f32).to_degrees();
            let mut state = PlayerState { position: Vec3::new(a[0] as f64 + 0.5, (a[1] + 1) as f64, a[2] as f64 + 0.5), on_ground: true, ..PlayerState::default() };
            for word in 0..4 {
                let recorder = RecordingWorld::new(snapshot);
                let start = state.clone();
                let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    for t in 0..ticks { recorder.tick(&mut state, &Input { forward: true, sprint: true, jump: t % 5 == 0, yaw: toward + 7.3 * word as f32, ..Input::default() }); }
                }));
                if ran.is_err() { break; }
                words.push((index, recorder.take(), start));
            }
        }
        let mut table = PatternTable::default();
        let mut time = |work: &mut dyn FnMut(&mut PatternTable<()>, &WorldSnapshot, &Footprint, &PlayerState)| {
            let clock = Instant::now();
            for (index, footprint, start) in &words { work(&mut table, &snapshots[*index].0, footprint, start); }
            clock.elapsed().as_secs_f64() * 1e6 / words.len() as f64
        };
        let cold = time(&mut |table, world, footprint, start| { table.insert(world, footprint, start, ()).unwrap(); });
        let lookup = time(&mut |table, world, _, start| assert!(table.lookup(world, start).is_some()));
        let warm = time(&mut |table, world, footprint, start| { table.insert(world, footprint, start, ()).unwrap(); });
        let cells = words.iter().map(|(_, f, _)| f.reads.len()).sum::<usize>() / words.len();
        let own: usize = table.entries().map(|(key, _)| size_of::<Key>() + 2 * key.ids.len() + size_of::<Entry<()>>()).sum();
        let shapes: usize = table.shapes().iter().map(|s: &Shape| size_of::<Shape>() + 13 * s.cells.len()).sum();
        let starts: usize = table.starts().iter().map(|s| 16 + 8 * s.len()).sum();
        let shared = shapes + starts + 16 * table.digests().len();
        let n = table.entries().count();
        println!("{ticks}-tick words: {}, {cells} read cells each; bytes per entry {} own + {} shared ({} shapes, {} start states, \
            {} digests); us per key {cold:.1} cold, {warm:.1} warm; {lookup:.1} us per lookup",
            words.len(), own / n, shared / n, table.shapes().len(), table.starts().len(), table.digests().len());
    }
}
