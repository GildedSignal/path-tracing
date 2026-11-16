//! Offline red-team lens for kill test 10 on the OLD hardtrack build. Reads only.
use law::{BoxWorld, Input, PlayerState};
use plan::{settle, Still};
use world::{model, BlockState};

fn block(raw: &str) -> law::Block {
    let (name, tail) = raw.split_once('[').unwrap_or((raw, ""));
    let mut properties = tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty())
        .map(|p| { let (k, v) = p.split_once('=').unwrap(); (k.into(), v.into()) }).collect::<Vec<_>>();
    properties.sort();
    model(&BlockState { name: format!("minecraft:{}", name.trim_start_matches("minecraft:")), properties }).unwrap().clone()
}
fn build(commands: &str) -> BoxWorld {
    let mut world = BoxWorld::default();
    world.difficulty = 0;
    world.fill([-120, -64, -120], [-50, -64, -50], block("bedrock"));
    world.fill([-120, -63, -120], [-50, -62, -50], block("dirt"));
    world.fill([-120, -61, -120], [-50, -61, -50], block("grass_block[snowy=false]"));
    for line in commands.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() { continue; }
        let f = line.split_whitespace().collect::<Vec<_>>();
        let at = |i: usize| -> [i32; 3] { std::array::from_fn(|a| f[i + a].parse().unwrap()) };
        let (lo, hi, raw) = match f[0] { "/fill" => (at(1), at(4), f[7]), "/setblock" => (at(1), at(1), f[4]), o => panic!("{o}") };
        let value = block(raw);
        for z in lo[2]..=hi[2] { for y in lo[1]..=hi[1] { for x in lo[0]..=hi[0] { world.set([x, y, z], value.clone()); } } }
    }
    world
}
const YAWS: usize = 64;
fn yaw(k: usize) -> f32 { k as f32 * 360.0 / YAWS as f32 }
fn input(keys: &str, yaw: f32) -> Input {
    Input { forward: keys.contains('F'), sprint: keys.contains('S'), jump: keys.contains('J'), yaw, ..Input::default() }
}
/// Hold one word until the body lands after being airborne, catches a climbable, or 80 ticks pass.
fn hold(world: &BoxWorld, mut s: PlayerState, keys: &str, yaw: f32) -> (PlayerState, u32, bool) {
    let mut airborne = !s.on_ground;
    for t in 1..=80u32 {
        if law::try_tick(&mut s, world, &input(keys, yaw)).is_err() { return (s, t, false); }
        if s.on_climbable && !airborne { continue; }
        if s.on_climbable { return (s, t, true); }
        if !s.on_ground { airborne = true; } else if airborne { return (s, t, true); }
    }
    (s, 80, false)
}
fn on_wall(s: &PlayerState) -> bool {
    s.on_ground && s.position.y == -31.0 && s.position.x - 0.3 < -84.0 && s.position.x + 0.3 > -85.0
        && s.position.z + 0.3 > -99.0 && s.position.z - 0.3 < -95.0
}
fn on_final_ladder(s: &PlayerState) -> bool {
    s.on_climbable && (-84.0..-82.0).contains(&s.position.x) && (-92.0..-91.0).contains(&s.position.z.floor())
}
const WORDS: [&str; 4] = ["F", "FS", "FSJ", "FJ"];

fn main() {
    let text = std::fs::read_to_string("D:/path-tracing-wt/hard-track-2/jobs/hard-track-2/old.build.txt").unwrap();
    let world = build(&text);
    let still = Still(&world);

    // A. From rest on the southern lane (blue ice, y -29), one held word, does it land on the wall top?
    println!("== A: from rest on the southern lane, one held word -> wall top (x -85, z -99..-96, top y -31)");
    let mut a_hits = Vec::new(); let mut a_starts = 0;
    let xs = [-89.5, -88.5, -87.5, -86.5, -85.5, -84.5, -83.71];
    let zs = [-102.5, -101.5, -101.0, -100.71];
    for &x in &xs { for &z in &zs {
        let Ok(s0) = settle(&still, [x, -28.0, z], 0) else { continue };
        if s0.position.y != -28.0 { continue; }
        a_starts += 1;
        for w in WORDS { for k in 0..YAWS {
            let (s, t, done) = hold(&world, s0.clone(), w, yaw(k));
            if done && on_wall(&s) { a_hits.push((x, z, w, yaw(k), t, s.position.x, s.position.z)); }
        }}
    }}
    println!("starts at rest: {a_starts}; (start, word, yaw) hits: {}", a_hits.len());
    let cc = a_hits.iter().filter(|h| (h.0 - h.0.floor() - 0.5).abs() < 1e-9 && (h.1 - h.1.floor() - 0.5).abs() < 1e-9).count();
    println!("  of which from cell centres (reach.rs style): {cc}");
    for h in a_hits.iter().take(12) { println!("  start ({:.2},{:.2}) {} yaw {:.3} -> {} ticks, lands ({:.4},{:.4})", h.0, h.1, h.2, h.3, h.4, h.5, h.6); }
    if let Some(m) = a_hits.iter().min_by_key(|h| h.4) { println!("  fastest: {} ticks from ({:.2},{:.2}) {} yaw {:.3}", m.4, m.0, m.1, m.2, m.3); }

    // B. From rest on the wall top, one held word, does it catch the final ladder?
    println!("== B: from rest on the wall top, one held word -> final ladder (x -84..-83, z -92)");
    let mut b_hits = Vec::new(); let mut b_starts = 0;
    for &x in &[-84.5, -84.29, -83.71] { for &z in &[-98.5, -97.5, -96.5, -95.5, -95.29, -94.71] {
        let Ok(s0) = settle(&still, [x, -31.0, z], 0) else { continue };
        if s0.position.y != -31.0 { continue; }
        b_starts += 1;
        for w in WORDS { for k in 0..YAWS {
            let (s, t, done) = hold(&world, s0.clone(), w, yaw(k));
            if done && on_final_ladder(&s) { b_hits.push((x, z, w, yaw(k), t, s.position.y)); }
        }}
    }}
    println!("starts at rest: {b_starts}; hits: {}", b_hits.len());
    for h in b_hits.iter().take(8) { println!("  start ({:.2},{:.2}) {} yaw {:.3} -> catch after {} ticks at feet y {:.4}", h.0, h.1, h.2, h.3, h.4, h.5); }
    if let Some(m) = b_hits.iter().max_by(|a, b| a.5.total_cmp(&b.5)) {
        let climb = ((-24.0 - m.5) / 0.1176).ceil();
        println!("  highest catch: y {:.4} after {} ticks; climb to the top about {climb} ticks at 0.1176/tick", m.5, m.4);
    }

    // C. Momentum: from rest on the lane, word 1 = sprint-jump east under the head-hitter for n ticks,
    //    word 2 = one held word to the wall top. Which n admit a transfer?
    println!("== C: rest at (-97.5,-28,-102.5); FSJ yaw 270 for n ticks, then one held word -> wall top");
    let s_rest = settle(&still, [-97.5, -28.0, -102.5], 0).unwrap();
    let mut s = s_rest.clone();
    for n in 0..=40u32 {
        if n > 0 { law::try_tick(&mut s, &world, &input("FSJ", 270.0)).unwrap(); }
        let mut best: Option<(u32, &str, f32)> = None; let mut count = 0;
        for w in WORDS { for k in 0..YAWS {
            let (e, t, done) = hold(&world, s.clone(), w, yaw(k));
            if done && on_wall(&e) { count += 1; if best.is_none_or(|b| t < b.0) { best = Some((t, w, yaw(k))); } }
        }}
        if n % 4 == 0 || count > 0 {
            println!("  n {n:2}: pos ({:.4},{:.4},{:.4}) v ({:.4},{:.4},{:.4}) ground {} -> {count} hits, best {:?}",
                s.position.x, s.position.y, s.position.z, s.velocity.x, s.velocity.y, s.velocity.z, s.on_ground, best.map(|b| (n + b.0, b.1, b.2)));
        }
    }

    // E. T bar: from his ladder-exit point, a 4-word fixed-yaw sentence through the wall top.
    //    w1 = (FS|FSJ, yaw y1) for n1 ticks; w2 held to the wall top; w3 held to the final ladder;
    //    w4 = F yaw 0 held until B is touched. Compare with his 428 - 259 = 169 ticks after the exit.
    println!("== E: from his ladder exit (-105.47,-28,-105.56) at rest: 4 fixed-yaw words via the wall top to B");
    let start = settle(&still, [-105.47, -28.0, -105.56], 0).unwrap();
    println!("  settled start y {:.4} ground {}", start.position.y, start.on_ground);
    let goal = plan::Goal { cell: [-84, -25, -88] };
    let mut best: Option<(u32, String)> = None; let mut wall_hits = 0usize; let mut ladder_hits = 0usize; let mut sentences = 0usize;
    let mut prefixes = Vec::new();
    for k0 in ["FS", "FSJ"] { for y0i in 44..=56usize { let y0 = yaw(y0i);
        let mut s0 = start.clone();
        for n0 in 1..=40u32 {
            if law::try_tick(&mut s0, &world, &input(k0, y0)).is_err() { break; }
            if s0.on_ground && s0.position.y == -28.0 && (-102.7..=-101.3).contains(&s0.position.z) && s0.position.x > -100.0 {
                prefixes.push((format!("{k0} y{y0} x{n0}"), n0, s0.clone()));
            }
        }
    }}
    println!("  prefixes onto the southern lane: {}", prefixes.len());
    for (p0, n0, s0) in &prefixes { for k1 in ["FSJ"] { let y1 = 270.0f32;
        let mut s1 = s0.clone();
        for n1 in 1..=60u32 {
            if law::try_tick(&mut s1, &world, &input(k1, y1)).is_err() { break; }
            if s1.position.x < -93.0 { continue; }
            if s1.position.x > -82.0 { break; }
            for w2 in WORDS { for k2 in 0..YAWS {
                let (s2, t2, d2) = hold(&world, s1.clone(), w2, yaw(k2));
                if !(d2 && on_wall(&s2)) { continue; }
                wall_hits += 1;
                for w3 in WORDS { for k3 in 0..YAWS {
                    let (s3, t3, d3) = hold(&world, s2.clone(), w3, yaw(k3));
                    if !(d3 && on_final_ladder(&s3)) { continue; }
                    ladder_hits += 1;
                    let mut s4 = s3.clone(); let mut prev = s4.clone();
                    for t4 in 1..=200u32 {
                        if law::try_tick(&mut s4, &world, &input("F", 0.0)).is_err() { break; }
                        if goal.touched(&prev, &s4) {
                            sentences += 1;
                            let total = n0 + n1 + t2 + t3 + t4;
                            if best.as_ref().is_none_or(|b| total < b.0) {
                                best = Some((total, format!("{p0} | {k1} y{y1} x{n1} | {w2} y{} x{t2} | {w3} y{} x{t3} | F y0 x{t4}", yaw(k2), yaw(k3))));
                            }
                            break;
                        }
                        prev = s4.clone();
                    }
                }}
            }}
        }
    }}
    println!("  wall hits {wall_hits}, ladder hits {ladder_hits}, sentences to B {sentences}");
    println!("  best after the exit: {:?}  (his: 169; his total 428 = 259 + 169)", best);
    // Cost of coming to rest on the lane, which a from-rest reach relation charges at the lane-end joint.
    let mut s = s_rest.clone();
    for _ in 0..24 { law::try_tick(&mut s, &world, &input("FSJ", 270.0)).unwrap(); }
    let v0 = s.velocity.x; let mut stop = 0;
    while s.velocity.x != 0.0 && stop < 400 { law::try_tick(&mut s, &world, &Input { yaw: 270.0, ..Input::default() }).unwrap(); stop += 1; }
    println!("  braking on blue ice from vx {v0:.4} with no keys: {stop} ticks to vx == 0");

    // D. Same law-comparison class, same position, different horizontal speed: different futures?
    println!("== D: one start point, two speeds with identical velocity comparisons (codec_state.rs:30-55), one word");
    let base = settle(&still, [-86.5, -28.0, -101.5], 0).unwrap();
    for vx in [0.06, 0.12, 0.24, 0.36, 0.48] {
        let mut s0 = base.clone(); s0.velocity.x = vx; s0.velocity.z = 0.06;
        let (e, t, done) = hold(&world, s0, "FS", 0.0);
        println!("  vx {vx:.2}: after {t} ticks done={done} lands ({:.4},{:.4},{:.4}) on_wall={}", e.position.x, e.position.y, e.position.z, on_wall(&e));
    }
}
