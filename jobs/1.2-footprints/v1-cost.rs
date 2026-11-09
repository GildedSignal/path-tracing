// v1 key cost harness: run as engine/world/tests/key_cost.rs at commit 8801299 (job 1.2's before numbers).
//! Scratch: v1 key cost on track words.
use std::path::PathBuf;
use law::{Input, PlayerState, Vec3, footprint::RecordingWorld, pattern::PatternKeys};
use world::WorldSnapshot;
fn fixture() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world") }
fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}
#[test]
fn v1_cost() { for ticks in [1, 10] { cost(ticks); } }
const TICKS_UNUSED: u8 = 0;
fn cost(TICKS: usize) {
    std::panic::set_hook(Box::new(|_| {}));
    let tracks = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lab/tracks");
    let (mut bytes, mut micros, mut words, mut cells) = (0usize, 0f64, 0usize, 0usize);
    for line in include_str!("fixtures/tracks.txt").lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let snapshot = WorldSnapshot::load(fixture(), [n(1), n(2), n(3)], [n(4), n(5), n(6)]).unwrap();
        let json = std::fs::read_to_string(tracks.join(format!("{}.json", f[0]))).unwrap();
        let (a, b) = (point_of(&json, "A"), point_of(&json, "B"));
        let toward = ((a[0]-b[0]) as f32).atan2((b[2]-a[2]) as f32).to_degrees();
        let mut state = PlayerState { position: Vec3::new(a[0] as f64 + 0.5, (a[1] + 1) as f64, a[2] as f64 + 0.5), on_ground: true, ..PlayerState::default() };
        for word in 0..4 {
            let recorder = RecordingWorld::new(&snapshot);
            let start = [state.position.x.floor() as i32, state.position.y.floor() as i32, state.position.z.floor() as i32];
            let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for t in 0..TICKS { recorder.tick(&mut state, &Input { forward: true, sprint: true, jump: t % 5 == 0, yaw: toward + 7.3 * word as f32, ..Input::default() }); }
            }));
            if ok.is_err() { break; }
            let footprint = recorder.take();
            let clock = std::time::Instant::now();
            let keys = PatternKeys::capture(&snapshot, &footprint, start).unwrap();
            micros += clock.elapsed().as_secs_f64() * 1e6;
            bytes += keys.frames.iter().map(|(_, k)| k.contents.len() + 16).sum::<usize>();
            words += 1; cells += footprint.reads.len();
        }
    }
    println!("v1: {words} {TICKS}-tick words, {} read cells/word, {} bytes/entry, {:.1} us/key", cells / words, bytes / words, micros / words as f64);
}
