use law::{Input, PlayerState, World};
use std::collections::BTreeMap;

fn ordered(x: f64) -> u64 { let b = x.to_bits(); if b >> 63 != 0 { !b } else { b ^ (1 << 63) } }
fn unordered(b: u64) -> f64 { f64::from_bits(if b >> 63 != 0 { b ^ (1 << 63) } else { !b }) }

#[derive(Clone, PartialEq, Eq, Debug)]
struct Out { sig: Vec<u128>, t: usize, end_x: u64, end_y: u64, support: Option<[i32; 3]>, vx: u64 }

fn run(w: &impl World, start: &PlayerState, s: f64) -> Out {
    let mut st = start.clone(); st.position.x = s;
    let mut sig = Vec::new();
    for _ in 0..80 {
        let before = st.clone();
        if let Err(e) = law::try_tick(&mut st, w, &Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() }) {
            sig.push(law::codec::digest(e.0.as_bytes())); break;
        }
        sig.push(words::segment::joint_events(&before, &st).0);
        if st.on_ground && st.position.x > 100.0 { break; }
    }
    Out { t: sig.len(), sig, end_x: st.position.x.to_bits(), end_y: st.position.y.to_bits(), support: st.main_supporting_block, vx: st.velocity.x.to_bits() }
}

// The test's estimator: one cut per differing consecutive pair.
fn refined_count(f: &mut impl FnMut(f64) -> Vec<u128>, lo: f64, hi: f64, d: u32) -> usize {
    let mut last = f(lo); let mut prev = lo; let mut cuts = 0;
    for i in 1..=d {
        let s = lo + (hi - lo) * i as f64 / d as f64; let sg = f(s);
        if sg != last {
            let mut a = ordered(prev); let mut b = ordered(s);
            while b - a > 1 { let m = a + (b - a) / 2; if f(unordered(m)) == last { a = m } else { b = m } }
            cuts += 1;
        }
        last = sg; prev = s;
    }
    cuts + 1
}

// All cuts between differing samples (recursive), keyed on a projection of Out.
fn all_cuts<K: PartialEq + Clone>(f: &mut impl FnMut(u64) -> K, a: u64, ka: K, b: u64, kb: K, out: &mut Vec<u64>) {
    if ka == kb { return; }
    if b - a <= 1 { out.push(b); return; }
    let m = a + (b - a) / 2; let km = f(m);
    all_cuts(f, a, ka, m, km.clone(), out); all_cuts(f, m, km, b, kb, out);
}

fn main() {
    let source = plan::source::GeneratorSuperflatSource::new(63, law::Block::full(), [80, 60, -10], [400, 100, 10]);
    let mut warm = plan::settle(&plan::Still(&source), [90.5, 64.0, 0.5], 0).unwrap(); let mut phase = warm.clone();
    for _ in 0..600 { law::try_tick(&mut warm, &source, &Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() }).unwrap(); if warm.on_ground { phase = warm.clone(); } }
    let mut w = law::BoxWorld::default(); w.difficulty = 0; w.fill([90, 60, -5], [120, 63, 5], law::Block::full());
    let mode = std::env::args().nth(1).unwrap_or_default();
    if std::env::var("WORLD").as_deref() == Ok("step") { w.fill([100, 64, -5], [100, 64, 5], law::Block::full()); println!("WORLD step: one-high wall at x=100"); }
    else if mode != "nogap" { w.fill([100, 63, -5], [100, 63, 5], law::Block::default()); }
    let mut start = phase.clone(); start.position.x = 95.5; start.position.z = 0.5; start.main_supporting_block = Some([95, 63, 0]);
    println!("phase v=({:e},{:e},{:e}) y={} on_ground={}", start.velocity.x, start.velocity.y, start.velocity.z, start.position.y, start.on_ground);
    let (lo, hi) = (95.5f64, 99.5f64);
    if mode == "pieces" {
        // Events up to the first tick that clamps the free axis (a pin), with the one
        // non-monotone flag masked; after a pin nothing depends on s but that flag.
        let mask = !(1u128 << (law::state_reads::StateField::minor_horizontal_collision as u8));
        let held = Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() };
        let mut f = |p: u64| -> Vec<u128> { let mut st = start.clone(); st.position.x = unordered(p); let mut sig = Vec::new();
            for _ in 0..80 { let b = st.clone(); law::try_tick(&mut st, &w, &held).unwrap(); sig.push(words::segment::joint_events(&b, &st).0 & mask);
                if st.horizontal_collision { sig.push(u128::MAX); break; } if st.on_ground && st.position.x > 100.0 { break; } } sig };
        for k in [6u32, 8, 10, 12, 14, 16, 18] { let n = 1u64 << k;
            let pts: Vec<u64> = (0..=n).map(|i| ordered(lo + (hi - lo) * i as f64 / n as f64)).collect();
            let sg: Vec<Vec<u128>> = pts.iter().map(|&p| f(p)).collect(); let mut cuts = Vec::new();
            for i in 0..n as usize { all_cuts(&mut f, pts[i], sg[i].clone(), pts[i + 1], sg[i + 1].clone(), &mut cuts); }
            let pins = { let mut c = 0; let mut last = None; for (i, s) in sg.iter().enumerate() { let p = s.last() == Some(&u128::MAX); if p && last != Some(true) { c += 1 } last = Some(p); let _ = i; } c };
            println!("PIECES samples=2^{k} pieces={} pinned_runs={pins}", cuts.len() + 1);
        }
        return;
    }
    if mode == "mono" {
        let held = Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() };
        let exit = |s: f64| { let mut st = start.clone(); st.position.x = s; let mut t = 0;
            for _ in 0..80 { law::try_tick(&mut st, &w, &held).unwrap(); t += 1; if st.on_ground && st.position.x > 100.0 { break; } } (t, st.position.x) };
        for (name, a, b) in [("P01 T=12 translating", 95.73f64, 96.33f64), ("P44 T=12 translating", 96.74, 99.49), ("window T=14 pinned", 96.40, 96.70)] {
            let mut steps: BTreeMap<i64, u64> = BTreeMap::new(); let mut nonmono = 0u64; let mut tchg = 0u64;
            for j in 0..400u64 { let s0 = a + (b - a) * j as f64 / 400.0; let mut p = s0; let (t0, mut prev) = exit(p);
                for _ in 0..2000 { let q = p.next_up(); let (t, x) = exit(q); if t != t0 { tchg += 1; break; }
                    if x < prev { nonmono += 1 } let d = ((x.to_bits() as i64) - (prev.to_bits() as i64)); *steps.entry(d).or_default() += 1; prev = x; p = q; } }
            println!("MONO {name}: exit-ulp steps per entry ulp {steps:?} nonmonotone={nonmono} T_changes={tchg}");
        }
        return;
    }
    if mode == "image" {
        use std::collections::{HashMap, HashSet};
        let held = Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() };
        let n = 1u64 << 16;
        let mut exits: HashMap<Vec<u64>, usize> = HashMap::new(); let mut exits_nox: HashMap<(usize, Vec<u64>), usize> = HashMap::new();
        let mut later: Vec<HashSet<Vec<u64>>> = vec![HashSet::new(); 61];
        let mut later_nox: Vec<HashSet<Vec<u64>>> = vec![HashSet::new(); 61];
        let mut window = 0;
        for i in 0..=n { let s = lo + (hi - lo) * i as f64 / n as f64;
            let mut st = start.clone(); st.position.x = s; let mut t = 0;
            for _ in 0..80 { law::try_tick(&mut st, &w, &held).unwrap(); t += 1; if st.on_ground && st.position.x > 100.0 { break; } }
            let bits = st.bits(); *exits.entry(bits.clone()).or_default() += 1;
            let mut nx = st.clone(); nx.position.x = 0.0; *exits_nox.entry((t, nx.bits())).or_default() += 1;
            if t == std::env::var("WT").ok().and_then(|v| v.parse().ok()).unwrap_or(14) { window += 1; let mut c = st.clone(); later[0].insert(c.bits());
                for k in 1..=60 { law::try_tick(&mut c, &w, &held).unwrap(); later[k].insert(c.bits()); let mut q = c.clone(); q.position.x = 0.0; q.exhaustion = 0.0; q.saturation = 0.0; later_nox[k].insert(q.bits()); } }
        }
        println!("IMAGE samples={} distinct_exit_states={} distinct_(T,exit_minus_x)={}", n + 1, exits.len(), exits_nox.len());
        { // minus x and minus the resource accumulators; also which fields separate classes
            let mut m: HashMap<(usize, Vec<u64>), usize> = HashMap::new(); let mut mf: HashMap<(usize, Vec<u64>), usize> = HashMap::new();
            let mut reps: Vec<PlayerState> = Vec::new(); let mut xs: HashMap<(usize, Vec<u64>), HashSet<u64>> = HashMap::new();
            for i in 0..=n { let s = lo + (hi - lo) * i as f64 / n as f64;
                let mut st = start.clone(); st.position.x = s; let mut t = 0;
                for _ in 0..80 { law::try_tick(&mut st, &w, &held).unwrap(); t += 1; if st.on_ground && st.position.x > 100.0 { break; } }
                let mut q = st.clone(); q.position.x = 0.0; q.exhaustion = 0.0; q.saturation = 0.0; q.food_level = 0;
                *m.entry((t, q.bits())).or_default() += 1;
                xs.entry((t, q.bits())).or_insert_with(HashSet::new).insert(st.position.x.to_bits());
                let mut r = q.clone(); r.fall_distance = 0.0; r.main_supporting_block = None; *mf.entry((t, r.bits())).or_default() += 1;
                if reps.len() < 3 && t == 21 && reps.iter().all(|p: &PlayerState| p.exhaustion != st.exhaustion) { reps.push(st.clone()); }
            }
            println!("IMAGE distinct_(T,exit_minus_x_minus_resources)={} also_minus_fall_distance_and_support_address={}", m.len(), mf.len());
            let mut per: Vec<_> = m.iter().map(|(k, c)| (k.0, *c, xs[k].len())).collect(); per.sort();
            println!("  per class (T, samples, distinct exit x): {per:?}");
            for p in &reps { println!("  rep T=21 x={} exhaustion={:e} fall={} support={:?} vx={:e}", p.position.x, p.exhaustion, p.fall_distance, p.main_supporting_block, p.velocity.x); }
        }
        let mut v: Vec<_> = exits_nox.iter().map(|((t, _), c)| (*t, *c)).collect(); v.sort(); println!("  (T,count) per exit-minus-x class: {v:?}");
        println!("WINDOW samples={window} distinct_states_after_k_ticks: {:?}", (0..=60).step_by(2).map(|k| (k, later[k].len())).collect::<Vec<_>>());
        println!("WINDOW distinct_states_minus_x_after_k: {:?}", (1..=60).step_by(3).map(|k| (k, later_nox[k].len())).collect::<Vec<_>>());
        return;
    }
    if mode == "probe" {
        let yaw = (270.0f32 * 0.017453292f32) as f64; let sn = law::math::sin(yaw) as f64; let cs = law::math::cos(yaw) as f64;
        let (ix, iz) = (0.0 * cs - 0.98 * sn, 0.98 * cs + 0.0 * sn);
        println!("intended=({ix:e},{iz:e})");
        let (mut agree, mut n, mut minor_true, mut nan, mut sprint_lost) = (0, 0, 0, 0, 0);
        for i in 0..20000 { let s = 96.34 + 0.39 * i as f64 / 20000.0;
            let mut st = start.clone(); st.position.x = s;
            for _ in 0..20 { let bx = st.position.x; let bz = st.position.z;
                law::try_tick(&mut st, &w, &Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() }).unwrap();
                if st.horizontal_collision { let (mx, mz) = (st.position.x - bx, st.position.z - bz);
                    let a = ix*ix + iz*iz; let b = mx*mx + mz*mz; let c = (ix*mx + iz*mz) / (a*b).sqrt();
                    let isnan = c.acos().is_nan(); n += 1; if st.minor_horizontal_collision { minor_true += 1 } if isnan { nan += 1 }
                    if isnan != st.minor_horizontal_collision { agree += 1 }
                    let mut nx = st.clone(); law::try_tick(&mut nx, &w, &Input { forward: true, sprint: true, jump: true, yaw: yaw_g(), ..Default::default() }).unwrap();
                    if !nx.sprinting { sprint_lost += 1 }
                    if i < 4 { println!("s={s} mx={mx:e} mz={mz:e} cos-1={:e} minor={} sprint_next={}", c - 1.0, st.minor_horizontal_collision, nx.sprinting); }
                    break; } } }
        println!("PROBE hits={n} minor_true={minor_true} acos_nan={nan} nan_iff_not_minor={agree} sprint_lost_next_tick={sprint_lost}");
        return;
    }

    let mut calls = 0u64;
    for d in [64u32, 256, 1024, 4096, 16384, 65536] {
        let mut f = |s: f64| { calls += 1; run(&w, &start, s).sig };
        println!("TEST_ESTIMATOR divisions={d} plateaus={}", refined_count(&mut f, lo, hi, d));
    }
    // All-cut estimator at several densities, for three projections.
    let (olo, ohi) = (ordered(lo), ordered(hi));
    let mut finest: Vec<u64> = Vec::new();
    for k in [6u32, 8, 10, 12, 14, 16, 18] {
        let n = 1u64 << k;
        let pts: Vec<u64> = (0..=n).map(|i| ordered(lo + (hi - lo) * i as f64 / n as f64)).collect();
        let outs: Vec<Out> = pts.iter().map(|&p| run(&w, &start, unordered(p))).collect();
        let mut sigc = Vec::new(); let mut tc = Vec::new(); let mut dec = Vec::new();
        for i in 0..n as usize {
            let mut fs = |p: u64| run(&w, &start, unordered(p)).sig;
            all_cuts(&mut fs, pts[i], outs[i].sig.clone(), pts[i + 1], outs[i + 1].sig.clone(), &mut sigc);
            let mut ft = |p: u64| run(&w, &start, unordered(p)).t;
            all_cuts(&mut ft, pts[i], outs[i].t, pts[i + 1], outs[i + 1].t, &mut tc);
            let mut fd = |p: u64| { let o = run(&w, &start, unordered(p)); (o.t, o.end_y, o.support, o.vx) };
            let a = &outs[i]; let b = &outs[i + 1];
            all_cuts(&mut fd, pts[i], (a.t, a.end_y, a.support, a.vx), pts[i + 1], (b.t, b.end_y, b.support, b.vx), &mut dec);
        }
        println!("ALLCUTS samples=2^{k} signature_plateaus={} T_plateaus={} exitclass_plateaus={}", sigc.len() + 1, tc.len() + 1, dec.len() + 1);
        if k == 18 { finest = sigc; }
    }
    // Describe the finest signature partition.
    let mut bounds = vec![olo]; bounds.extend(finest.iter().copied()); bounds.push(ohi + 1);
    let mut by_t: BTreeMap<usize, usize> = BTreeMap::new();
    let mut prev: Option<Out> = None;
    for j in 0..bounds.len() - 1 {
        let (a, b) = (bounds[j], bounds[j + 1] - 1);
        let oa = run(&w, &start, unordered(a)); let ob = run(&w, &start, unordered(b));
        *by_t.entry(oa.t).or_default() += 1;
        let width = unordered(b) - unordered(a);
        let mut diff = String::new();
        if let Some(p) = &prev {
            if let Some(i) = (0..p.sig.len().max(oa.sig.len())).find(|&i| p.sig.get(i) != oa.sig.get(i)) {
                let x = p.sig.get(i).copied().unwrap_or(0) ^ oa.sig.get(i).copied().unwrap_or(0);
                diff = format!("first_diff_tick={} kinds={:?}", i + 1, words::Exit(x));
            }
        }
        // translation check inside plateau: exit_x - entry in ulps at both ends
        let ulp_a = (f64::from_bits(oa.end_x) - unordered(a)) ; let ulp_b = (f64::from_bits(ob.end_x) - unordered(b));
        println!("P{j:02} lo={:.17} hi={:.17} width={:.3e} ulps={} T={} end_y={} same_sig_ends={} exitminusentry_lo={:.17} hi={:.17} {}",
            unordered(a), unordered(b), width, b - a + 1, oa.t, f64::from_bits(oa.end_y), oa.sig == ob.sig, ulp_a, ulp_b, diff);
        prev = Some(oa);
    }
    println!("plateaus_by_T={by_t:?} calls_first_phase={calls}");
}
fn yaw_g() -> f32 { std::env::var("YAW").ok().and_then(|v| v.parse().ok()).unwrap_or(270.0) }
