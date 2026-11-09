//! Job 1.2: which horizontal frames does the law replay bit for bit on Julien's tracks?
use std::{cell::Cell, collections::HashMap, path::PathBuf};
use law::{Block, BlockPos, Input, PlayerState, Vec3, World, tick, pattern::{FRAMES, Frame, model_digest}};
use world::{WorldSnapshot, model_table};

fn fixture() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world") }

// Independent of pattern.rs: mirror x, then turn (x, z) -> (-z, x), about the world origin.
fn point(f: Frame, mut x: f64, mut z: f64) -> (f64, f64) {
    if f.mirrored { x = -x; }
    for _ in 0..f.quarter_turns { (x, z) = (-z, x); }
    (x, z)
}
fn cell(f: Frame, [mut x, y, mut z]: BlockPos) -> BlockPos {
    if f.mirrored { x = -x-1; }
    for _ in 0..f.quarter_turns { (x, z) = (-z-1, x); }
    [x, y, z]
}
fn uncell(f: Frame, [mut x, y, mut z]: BlockPos) -> BlockPos {
    for _ in 0..f.quarter_turns { (x, z) = (z, -x-1); }
    if f.mirrored { x = -x-1; }
    [x, y, z]
}
fn vector(f: Frame, v: Vec3) -> Vec3 { let (x, z) = point(f, v.x, v.z); Vec3::new(x, v.y, z) }
fn yaw(f: Frame, mut yaw: f32) -> f32 {
    if f.mirrored { yaw = -yaw; }
    for _ in 0..f.quarter_turns { yaw += 90.0; }
    yaw
}
fn input(f: Frame, i: Input) -> Input {
    let (left, right) = if f.mirrored { (i.right, i.left) } else { (i.left, i.right) };
    Input { left, right, yaw: yaw(f, i.yaw), ..i }
}
fn state(f: Frame, s: &PlayerState) -> PlayerState {
    let mut t = s.clone();
    t.position = vector(f, s.position); t.velocity = vector(f, s.velocity);
    t.yaw = yaw(f, s.yaw); t.input = input(f, s.input);
    t.movement_this_tick = s.movement_this_tick.map(|v| vector(f, v));
    if f.quarter_turns % 2 == 1 { let v = s.stuck_speed_multiplier; t.stuck_speed_multiplier = Vec3::new(v.z, v.y, v.x); }
    t.main_supporting_block = s.main_supporting_block.map(|c| cell(f, c));
    t
}

/// Bits with signed zeros folded: negating a zero is a frame artifact, not physics.
fn fold(mut s: PlayerState) -> Vec<u64> {
    let z = |v: Vec3| Vec3::new(v.x + 0.0, v.y + 0.0, v.z + 0.0);
    s.position = z(s.position); s.velocity = z(s.velocity); s.stuck_speed_multiplier = z(s.stuck_speed_multiplier);
    s.movement_this_tick = s.movement_this_tick.map(z);
    s.bits()
}

/// The snapshot seen through a frame, each model replaced by the model the table holds for its turned content.
struct Turned<'a> { inner: &'a WorldSnapshot, frame: Frame, turned: &'a HashMap<*const Block, Option<&'static Block>>, missing: Cell<bool> }
impl World for Turned<'_> {
    fn block(&self, p: BlockPos) -> &Block {
        let block = self.inner.block(uncell(self.frame, p));
        self.turned[&(block as *const Block)].unwrap_or_else(|| { self.missing.set(true); block })
    }
    fn loaded(&self, p: BlockPos) -> bool { self.inner.loaded(uncell(self.frame, p)) }
    fn difficulty(&self) -> u8 { self.inner.difficulty() }
    fn natural_regeneration(&self) -> bool { self.inner.natural_regeneration() }
    fn has_fluid(&self) -> bool { self.inner.has_fluid() }
    fn has_inside_effects(&self) -> bool { self.inner.has_inside_effects() }
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); self.0 >> 33 }
    fn chance(&mut self, n: u64) -> bool { self.next() % n == 0 }
}

fn point_of(json: &str, key: &str) -> [i32; 3] {
    let tail = &json[json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4..];
    let v: Vec<i32> = tail[..tail.find(']').unwrap()].split(',').map(|s| s.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

// Which frames replay the tracks bit for bit? Only those may enter the key.
#[test]
fn frames_replay_rate_on_the_tracks() {
    let table = model_table();
    let identity: HashMap<u128, &'static Block> = table.iter().map(|b| (model_digest(b, FRAMES[0]).unwrap().unwrap(), b)).collect();
    let turned: Vec<HashMap<*const Block, Option<&'static Block>>> = FRAMES.iter().map(|&f| table.iter()
        .map(|b| (b as *const Block, identity.get(&model_digest(b, f).unwrap().unwrap()).copied())).collect()).collect();
    let tracks = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lab/tracks");
    // [frame][free yaw, yaw on the grid] = (exact runs, runs)
    let mut rate = [[(0, 0); 2]; 8];
    let mut largest = [0f64; 8];
    std::panic::set_hook(Box::new(|_| {}));
    for line in include_str!("fixtures/tracks.txt").lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let (min, max) = ([n(1), n(2), n(3)], [n(4), n(5), n(6)]);
        let snapshot = WorldSnapshot::load(fixture(), min, max).unwrap();
        let json = std::fs::read_to_string(tracks.join(format!("{}.json", f[0]))).unwrap();
        let (a, b) = (point_of(&json, "A"), point_of(&json, "B"));
        let toward = ((a[0]-b[0]) as f32).atan2((b[2]-a[2]) as f32).to_degrees();
        for seed in 0..8u64 {
            let grid = seed % 2 == 1;
            let mut random = Random((seed * 7919).wrapping_add(n(1) as u64));
            let inputs: Vec<Input> = (0..60).map(|_| Input {
                forward: !random.chance(8), left: random.chance(6), right: random.chance(6), jump: random.chance(3),
                sneak: random.chance(10), sprint: random.chance(2), pitch: 0.0,
                yaw: if grid { (toward / 90.0).round() * 90.0 } else { toward + (random.next() % 6000) as f32 / 100.0 - 30.0 },
                ..Input::default()
            }).collect();
            // A is the block under the feet.
            let start = PlayerState { position: Vec3::new(a[0] as f64 + 0.5, (a[1] + 1) as f64, a[2] as f64 + 0.5), on_ground: true, ..PlayerState::default() };
            let mut length = 0;
            for (index, frame) in FRAMES.into_iter().enumerate() {
                let world = Turned { inner: &snapshot, frame, turned: &turned[index], missing: Cell::new(false) };
                let (mut original, mut moved) = (start.clone(), state(frame, &start));
                let (mut exact, mut ticks) = (true, 0);
                for i in &inputs {
                    // A run ends where the body's read halo leaves the snapshot, which panics.
                    let step = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        tick(&mut original, &snapshot, i);
                        tick(&mut moved, &world, &input(frame, *i));
                    }));
                    if step.is_err() { break; }
                    ticks += 1;
                    if fold(state(frame, &original)) != fold(moved.clone()) {
                        exact = false;
                        let (e, m) = (state(frame, &original), &moved);
                        let gap = [e.position.x-m.position.x, e.position.y-m.position.y, e.position.z-m.position.z,
                            e.velocity.x-m.velocity.x, e.velocity.y-m.velocity.y, e.velocity.z-m.velocity.z];
                        largest[index] = gap.iter().fold(largest[index], |l, g| l.max(g.abs()));
                        break;
                    }
                }
                if index == 0 { length = ticks; }
                if world.missing.get() || length < 10 { continue; }
                let r = &mut rate[index][grid as usize];
                r.0 += exact as u32; r.1 += 1;
            }
        }
    }
    for (index, (frame, [free, grid])) in FRAMES.iter().zip(rate).enumerate() {
        println!("{frame:?}: free yaw {}/{} exact, yaw on the grid {}/{} exact; largest first gap {:e}",
            free.0, free.1, grid.0, grid.1, largest[index]);
    }
    // The key keeps the frames that replay every run: the identity alone.
    assert!(rate[0].iter().all(|r| r.0 == r.1 && r.1 > 0));
    assert!(rate[1..].iter().all(|[free, grid]| free.0 < free.1 && grid.0 < grid.1));
}
