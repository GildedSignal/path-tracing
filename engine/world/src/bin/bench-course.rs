//! SpectraPhysics' `bench/tick.js` course, ticked by the law on our 16³ sections: 50 players
//! sprint-jumping over stone with bottom slabs, post fences and water, yaw +90° every 40 ticks,
//! 200 warm-up ticks, 2,000 timed, reported per player-tick. Their side: `node bench/tick.js`.
use law::{basis::cell::{BlockId, StateId}, try_tick, Input, PlayerState, Vec3};
use std::{hint::black_box, time::Instant};
use world::WorldSnapshot;

fn main() {
    // `exact`: their course (stone ±30, off which 49 of their 50 players fall); `floored`: stone to ±128, every player
    // stays on the ground; `straight`: one player on plain stone, the box-world bench (`law` `bench`) on our sections.
    let variant = std::env::args().nth(1).unwrap_or_else(|| "exact".into());
    let straight = variant == "straight";
    let (players, ticks, warm, reps) = (if straight { 1usize } else { 50usize }, 2000usize, 200usize, 7usize);
    let (floor, lo, hi) = if straight { (0, [-16, 56, -48], [15, 79, 1023]) } else if variant == "floored" { (128, [-136, 56, -136], [135, 79, 135]) } else { (30, [-96, -12288, -96], [95, 95, 95]) };
    let id = |name: &str| BlockId::of(name).default_state();
    let slab = id("smooth_stone_slab").with("type", "bottom").with_bool("waterlogged", false);
    let fence = ["east", "west", "north", "south", "waterlogged"].iter().fold(id("oak_fence"), |s, p| s.with_bool(p, false));
    let water = StateId::parse("minecraft:water", |p| (p == "level").then_some("0")).unwrap();
    let course = WorldSnapshot::from_fn(lo, hi, |[x, y, z]| {
        let column = (x + 28).rem_euclid(6);
        let strip = (-28..=28).contains(&(x - column));
        match () {
            _ if straight && y == 63 && (-8..=8).contains(&x) && (-30..=1000).contains(&z) => id("stone"),
            _ if straight => id("air"),
            _ if y == 63 && (-floor..=floor).contains(&x) && (-floor..=floor).contains(&z) => id("stone"),
            _ if strip && column == 3 || strip && column == 4 => if y == 64 && (5..=28).contains(&z) { water } else { id("air") },
            _ if strip && column == 2 && (64..=65).contains(&y) && (-28..=-10).contains(&z) => fence,
            _ if strip && column == 0 && y == 64 && (-28..=28).contains(&z) => slab,
            _ => id("air"),
        }
    });
    let mut times = Vec::new();
    let mut refused = 0usize;
    for _ in 0..reps {
        let mut sims: Vec<(PlayerState, f32)> = (0..players).map(|i| {
            let pos = if straight { Vec3::new(0.5, 64.0, 0.5) } else { Vec3::new(-25.0 + (i % 10) as f64 * 5.0, 64.0, -25.0 + (i / 10) as f64 * 5.0) };
            (PlayerState { position: pos, on_ground: true, ..PlayerState::default() }, if straight { 0.0 } else { ((i * 37) % 360) as f32 })
        }).collect();
        let mut run = |t: usize, sims: &mut Vec<(PlayerState, f32)>, turn: bool| {
            for (s, yaw) in sims.iter_mut() {
                if turn && !straight && t % 40 == 0 { *yaw = (*yaw + 90.0) % 360.0; }
                let input = Input { forward: true, sprint: true, jump: !straight || s.on_ground, yaw: *yaw, ..Input::default() };
                if try_tick(black_box(s), black_box(&course), black_box(&input)).is_err() { refused += 1; }
                s.food_level = 20; s.saturation = 5.0; s.exhaustion = 0.0;
            }
        };
        for t in 0..warm { run(t, &mut sims, false); }
        let start = Instant::now();
        for t in 0..ticks { run(t, &mut sims, true); }
        times.push(start.elapsed().as_nanos() as f64 / (ticks * players) as f64);
        black_box(&sims);
    }
    times.sort_by(f64::total_cmp);
    println!("{variant}: {players} players x {ticks} ticks, n={reps}, ns per player-tick min={:.1} median={:.1} max={:.1}; refused ticks={refused}",
        times[0], times[reps / 2], times[reps - 1]);
}
