//! Probe: one strip of a block between two stone floors, solved bare-legs. Scratch, not committed.
use law::basis::cell::{BlockId, StateId};
use plan::flight::Track;
use std::time::Instant;

fn main() {
    let id = |n: &str| BlockId::of(n).default_state();
    let water = StateId::parse("minecraft:water", |p| (p == "level").then_some("0")).unwrap();
    let pool = move |depth: i32, wide: i32| move |y: i32, z: i32| if (10..10 + wide).contains(&z) { if (1 - depth..=0).contains(&y) { water } else if y < 1 - depth { id("stone") } else { id("air") } } else if y == 0 { id("stone") } else { id("air") };
    let cases: Vec<(&str, Box<dyn Fn(i32, i32) -> StateId>)> = vec![
        ("flat", Box::new(move |y, _z| if y == 0 { id("stone") } else { id("air") })),
        ("water1x6", Box::new(pool(1, 6))),
        ("water2x6", Box::new(pool(2, 6))),
        ("water3x10", Box::new(pool(3, 10))),
        ("water5x20", Box::new(pool(5, 20))),
        ("leaf_litter", Box::new(move |y, z| if y == 0 { id("grass_block") } else if y == 1 && (10..16).contains(&z) { id("leaf_litter") } else { id("air") })),
        ("powder_snow", Box::new(move |y, z| if y == 0 { if (10..16).contains(&z) { id("powder_snow") } else { id("snow_block") } } else if y < 0 { id("stone") } else { id("air") })),
        ("drop2", Box::new(move |y, z| if z < 10 { if y <= 2 { id("stone") } else { id("air") } } else if y <= 0 { id("stone") } else { id("air") })),
    ];
    for (name, f) in cases {
        let (lo, hi) = ([-24, -8, -8], [24, 24, 48]);
        let world = world::WorldSnapshot::from_fn(lo, hi, |[_, y, z]| f(y, z));
        let a_y = if name == "drop2" { 2 } else { 0 };
        let track = Track { name: name.into(), world, read: (lo, hi), body: ([lo[0] + 3, lo[1] + 1, lo[2] + 3], [hi[0] - 3, hi[1] - 3, hi[2] - 3]), a: [0, a_y, 0], b: [0, 0, 36], b_face: None, his: vec![], items: String::new(), pad: 0 };
        let start = match track.settled_start() { Ok(s) => s, Err(e) => { println!("{name}: no start {e}"); continue } };
        let source = plan::source::GeneratorSuperflatSource::snapshot(&track.world, track.a).unwrap();
        let store = plan::store::GeneratorStore::default();
        let mut codec = law::codec::Interner::default();
        let t = Instant::now();
        let r = plan::solve(&start, track.goal(), plan::Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        match r.outcome {
            plan::PlanOutcome::Ready(p) => {
                let replay = plan::flight::solve_track(&track, std::path::Path::new(&format!("D:/path-tracing-run/strip-{name}.jsonl"))).map(|_| "replayed").unwrap_or("REPLAY FAILED");
                println!("{name}: T={} think_ms={ms:.1} {replay}", p.ticks)
            }
            o => println!("{name}: {o:?} think_ms={ms:.1} why={:?}", r.explanation),
        }
    }
}
