use law::{tick, Block, BoxWorld, Input, PlayerState, Vec3};
use std::{hint::black_box, time::Instant};

fn main() {
    const N: usize = 20_000;
    const REPS: usize = 7;
    for name in ["walk", "sprint-jump", "ceiling", "bumpy"] {
        let mut world = BoxWorld::default();
        world.fill([-10,-1,-10], [10,-1,20000], Block::full());
        if name == "ceiling" { world.fill([-10,2,-10], [10,2,20000], Block::full()); }
        if name == "bumpy" {
            for z in (4..8000).step_by(4) {
                world.set([0,0,z], Block::boxes(&[law::Aabb::new(0.0,0.0,0.0,1.0,0.5,1.0)]));
            }
        }
        let mut times = Vec::new();
        for _ in 0..REPS {
            let mut state = PlayerState { on_ground:true, velocity:Vec3::new(0.0,-0.0784000015258789,0.0), ..PlayerState::default() };
            let start = Instant::now();
            for _ in 0..N {
                let input = Input { forward:true, sprint:name != "walk", jump:name != "walk" && state.on_ground, ..Input::default() };
                tick(black_box(&mut state), black_box(&world), black_box(&input));
                state.food_level=20; state.saturation=5.0; state.exhaustion=0.0;
            }
            times.push(start.elapsed().as_nanos() as f64 / N as f64);
            black_box(state);
        }
        times.sort_by(f64::total_cmp);
        println!("{name}: n={REPS}, ns/tick min={:.1} median={:.1} max={:.1}; samples={times:?}", times[0], times[REPS/2], times[REPS-1]);
    }
}
