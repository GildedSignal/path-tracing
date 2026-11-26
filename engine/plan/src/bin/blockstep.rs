//! BlockStep's protocol (Braam 2022, thesis section 5.2) on a saved world: from a start column, a goal
//! at (±d, ±d), bare legs, solved by `plan::solve` and replayed through `law::try_tick`.
//! usage: blockstep <world dir> <tasks file: name x z dx dz per line> <script dir>
use law::World;
use plan::flight::{solve_track, Track};
use std::{path::PathBuf, time::Instant};

/// The ground a player stands on in column (x, z): the highest cell that collides or holds water,
/// passing through leaves and logs as a tree canopy, never as ground.
fn ground(world: &world::WorldSnapshot, x: i32, z: i32, top: i32) -> Option<(law::BlockPos, bool)> {
    (-63..=top).rev().map(|y| [x, y, z]).find_map(|p| {
        let name = world.state_id(p)?.block().name();
        if name.ends_with("_leaves") || name.ends_with("_log") || name.ends_with("_wood") { return None; }
        let block = world.block(p);
        if block.fluid.kind != law::FluidKind::Empty { return Some((p, true)); }
        (!block.shape.is_empty()).then_some((p, false))
    })
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let (dir, tasks, out) = (PathBuf::from(&args[1]), std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?, PathBuf::from(&args[3]));
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for line in tasks.lines().filter(|l| !l.trim().is_empty()) {
        let f: Vec<&str> = line.split_whitespace().collect();
        let n: Vec<i32> = f[1..5].iter().map(|v| v.parse().unwrap()).collect();
        let (x, z, dx, dz) = (n[0], n[1], n[2], n[3]);
        let (lo, hi) = ([x.min(x + dx) - 24, -64, z.min(z + dz) - 24], [x.max(x + dx) + 24, 319, z.max(z + dz) + 24]);
        let world = world::WorldSnapshot::load_for_plan(&dir, lo, hi).map_err(|e| format!("{e:?}"))?;
        // Within 3 cells of the column, the nearest dry ground that settles a standing player.
        let near = |cx: i32, cz: i32, dry: bool| (0..=3).flat_map(|r| (-r..=r).flat_map(move |i| (-r..=r).map(move |j| (cx + i, cz + j))))
            .filter_map(|(px, pz)| ground(&world, px, pz, 318)).find(|&(_, wet)| !(dry && wet));
        let (Some((a, _)), Some((b, _))) = (near(x, z, true), near(x + dx, z + dz, false)) else { println!("{} NO-GROUND", f[0]); continue };
        let (ylo, yhi) = ((a[1].min(b[1]) - 24).max(-64), (a[1].max(b[1]) + 40).min(319));
        let (read_lo, read_hi) = ([lo[0], ylo, lo[2]], [hi[0], yhi, hi[2]]);
        let world = world::WorldSnapshot::load_for_plan(&dir, read_lo, read_hi).map_err(|e| format!("{e:?}"))?;
        let track = Track { name: f[0].into(), world, read: (read_lo, read_hi), body: ([read_lo[0] + 3, read_lo[1] + 1, read_lo[2] + 3], [read_hi[0] - 3, read_hi[1] - 3, read_hi[2] - 3]),
            a, b, b_face: None, his: Vec::new(), items: String::new(), pad: 0 };
        let floor = ((dx as f64).hypot(dz as f64) * 2248.0 / 800.0).ceil() as u32;
        let clock = Instant::now();
        let start = match track.settled_start() { Ok(s) => s, Err(e) => { println!("{} NO-START {e}", f[0]); continue } };
        let source = plan::source::GeneratorSuperflatSource::snapshot(&track.world, a)?;
        let store = plan::store::GeneratorStore::default();
        let mut codec = law::codec::Interner::default();
        let r = plan::solve(&start, track.goal(), plan::Kit::BareLegs, &source, &store, &mut codec, u64::MAX);
        let think_ms = clock.elapsed().as_secs_f64() * 1000.0;
        match r.outcome {
            plan::PlanOutcome::Ready(p) => {
                let replay = solve_track(&track, &out.join(format!("{}.jsonl", f[0]))).map(|_| "replayed".to_string()).unwrap_or_else(|e| format!("replay-failed {e}"));
                println!("{} T={} s={:.2} floor={} think_ms={:.1} a={:?} b={:?} {replay}", f[0], p.ticks, p.ticks as f64 / 20.0, floor, think_ms, a, b);
            }
            other => println!("{} {:?} think_ms={:.1} a={:?} b={:?}", f[0], other, think_ms, a, b),
        }
    }
    Ok(())
}
