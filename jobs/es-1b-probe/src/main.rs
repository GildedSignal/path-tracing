pub mod num {
    pub type Tick = i64;
}

#[allow(dead_code)]
#[path = "D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/bits.rs"]
pub mod bits;

#[allow(dead_code)]
#[path = "D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/codec.rs"]
pub mod codec;

#[allow(dead_code)]
pub mod belt;

#[allow(dead_code)]
#[path = "D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/shelf.rs"]
pub mod shelf;

#[allow(dead_code)]
#[path = "D:/path-tracing-deps/emergent-space-engine/crates/shipmem/src/program.rs"]
pub mod program;

#[allow(dead_code)]
#[path = "D:/path-tracing-deps/emergent-space-engine/crates/universe/src/calendar.rs"]
pub mod calendar;

mod dense;
mod gait;
mod index;

use codec::{Enc, MAXC};
use gait::{calibrate_sprint_jump, eval_gait, make_flat_stone_world, step_law_oracle, GaitCycle, GAIT_PERIOD};
use index::{register_arrow_crossing, verify_undisturbed_world_gait, CertHandle, SpaceTimeIndex, WorldGait, READ_SHAPE};
use law::{BoxWorld, Vec3};
use program::{Prog, Program};
use shelf::Ship;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mutant {
    None,
    PerBotIndex,
    NoReabsorb,
    DirtyObserver,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct GpuBotInstance {
    pos: [f32; 3],
    yaw: f32,
    flags: u32,
    ordinal: u64,
}

struct DistStats {
    n: usize,
    min: f64,
    median: f64,
    p90: f64,
    p99: f64,
    max: f64,
}

fn compute_stats(samples: &[f64]) -> DistStats {
    assert!(!samples.is_empty());
    let mut s = samples.to_vec();
    s.sort_by(f64::total_cmp);
    let n = s.len();
    let pct = |num: usize, den: usize| -> f64 {
        let idx = ((n - 1) * num) / den;
        s[idx]
    };
    DistStats {
        n,
        min: s[0],
        median: pct(1, 2),
        p90: pct(9, 10),
        p99: pct(99, 100),
        max: s[n - 1],
    }
}

fn print_stats(label: &str, unit: &str, d: &DistStats) {
    println!(
        "  {label}: n={} min={:.3} {unit}, median={:.3} {unit}, p90={:.3} {unit}, p99={:.3} {unit}, max={:.3} {unit}",
        d.n, d.min, d.median, d.p90, d.p99, d.max
    );
}

#[inline]
fn next_u64(state: &mut u64) -> u64 {
    let mut z = *state;
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    *state = z;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn fnv128_step(mut h: u128, bytes: &[u8]) -> u128 {
    for &b in bytes {
        h = (h ^ b as u128).wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013b);
    }
    h
}

fn universe_digest(streams: &[Program]) -> u128 {
    let mut h = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58du128;
    let mut buf = crate::codec::Buf::default();
    let mut cur_block;
    for p in streams {
        h = fnv128_step(h, &p.n.to_le_bytes());
        h = fnv128_step(h, &(p.ef.k as u64).to_le_bytes());
        for col in 0..p.cols {
            match p.prog[col] {
                Prog::Const(v) => {
                    h = fnv128_step(h, &[1]);
                    h = fnv128_step(h, &v.to_le_bytes());
                }
                Prog::Affine { a, b } => {
                    h = fnv128_step(h, &[2]);
                    h = fnv128_step(h, &a.to_le_bytes());
                    h = fnv128_step(h, &b.to_le_bytes());
                }
                Prog::Belt(_) => h = fnv128_step(h, &[3]),
            }
        }
        cur_block = usize::MAX;
        for (j, o) in p.ef.cursor(0) {
            if j / program::RB != cur_block {
                cur_block = j / program::RB;
                let mut ship = [0u64; MAXC];
                p.eval(o, &mut ship);
                h = fnv128_step(h, &o.to_le_bytes());
                for c in 0..p.cols {
                    h = fnv128_step(h, &ship[c].to_le_bytes());
                }
            } else {
                let mut ship = [0u64; MAXC];
                p.eval(o, &mut ship);
                h = fnv128_step(h, &o.to_le_bytes());
                for c in 0..p.cols {
                    h = fnv128_step(h, &ship[c].to_le_bytes());
                }
            }
            let _ = &mut buf;
        }
        // Include hot overrides in digest so dirty observer updates are caught immediately.
        let mut hot: Vec<_> = p.hot.iter().map(|(&k, &v)| (k, v)).collect();
        hot.sort_unstable_by_key(|x| x.0);
        for (k, v) in hot {
            h = fnv128_step(h, &k.to_le_bytes());
            for c in 0..p.cols {
                h = fnv128_step(h, &v[c].to_le_bytes());
            }
        }
    }
    h
}

fn make_ship(t_dep: u64, x: f64, z0: f64, gait_id: u64, kit_inv: u64) -> Ship {
    [t_dep, x.to_bits(), z0.to_bits(), gait_id, kit_inv, 0, 0, 0]
}

fn build_scarred_stream(
    n: u64,
    t_base: u64,
    cadence: u64,
    x_lane: f64,
    z0: f64,
    scar_count: usize,
    seed: u64,
    enc: &mut Enc,
) -> (Program, Vec<(u64, Ship)>) {
    let prog = [
        Prog::Affine {
            a: t_base,
            b: cadence as i64,
        },
        Prog::Const(x_lane.to_bits()),
        Prog::Const(z0.to_bits()),
        Prog::Const(0),  // gait_id
        Prog::Const(64), // kit_inv
        Prog::Const(0),
        Prog::Const(0),
        Prog::Const(0),
    ];
    if scar_count == 0 {
        return (Program::seal_from(n, 5, prog, &[], enc), Vec::new());
    }
    assert!(scar_count as u64 <= n);
    let stride = (n / scar_count as u64).max(1);
    let mut rng = seed ^ 0xA5A5_5A5A_1234_5678;
    let mut scars = Vec::with_capacity(scar_count);
    for k in 0..scar_count {
        let ord = ((k as u64) * stride).min(n - 1);
        let r = next_u64(&mut rng);
        let dt_delay = 2 + (r % 19); // +2..+20 ticks
        let dx = if (r >> 8) & 1 == 0 { 0.0 } else { 1.0 };
        let s = make_ship(
            t_base + cadence * ord + dt_delay,
            x_lane + dx,
            z0,
            0,
            64 - (r % 5),
        );
        scars.push((ord, s));
    }
    let p = Program::seal_from(n, 5, prog, &scars, enc);
    (p, scars)
}

fn run_check_1(world: &BoxWorld, cycle: &GaitCycle) -> bool {
    println!("\n[Check 1] Bit-Exact Oracle (10^9 bots across 1,000 Form-P streams vs law::try_tick)");
    const STREAMS: usize = 1_000;
    const BOTS_PER_STREAM: u64 = 1_000_000;
    const SCARS_PER_STREAM: usize = 100;

    let mut enc = Enc::new(false);
    let mut streams = Vec::with_capacity(STREAMS);
    let mut sample_scars: Vec<(usize, u64)> = Vec::new();
    for sid in 0..STREAMS {
        let x_lane = 1.5 + (sid as f64) * 2.0;
        let t_base = (sid as u64) * 3;
        let z0 = 0.5 + ((sid % 7) as f64) * 0.125;
        let (p, scars) = build_scarred_stream(
            BOTS_PER_STREAM,
            t_base,
            2,
            x_lane,
            z0,
            SCARS_PER_STREAM,
            sid as u64 + 1,
            &mut enc,
        );
        if sid < 250 {
            sample_scars.push((sid, scars[sid % SCARS_PER_STREAM].0));
        }
        streams.push(p);
    }

    let total_bots: u64 = streams.iter().map(|s| s.n).sum();
    let total_scars: usize = streams.iter().map(|s| s.scars()).sum();
    assert_eq!(total_bots, 1_000_000_000);
    assert_eq!(total_scars, 100_000);

    // Build 1,000 test samples: endpoints (0, N-1), scarred ordinals, and pseudo-random ordinals.
    let mut queries: Vec<(usize, u64, u64)> = Vec::with_capacity(1_000);
    let mut rng = 0xDEAD_BEEF_CAFE_BABEu64;
    for i in 0..1_000usize {
        let elapsed = 1 + (next_u64(&mut rng) % 5_000);
        let (sid, ord) = if i == 0 {
            (0, 0)
        } else if i == 1 {
            (STREAMS - 1, BOTS_PER_STREAM - 1)
        } else if i < 2 + sample_scars.len() {
            sample_scars[i - 2]
        } else {
            let sid = (next_u64(&mut rng) as usize) % STREAMS;
            let ord = next_u64(&mut rng) % BOTS_PER_STREAM;
            (sid, ord)
        };
        queries.push((sid, ord, elapsed));
    }

    let mut mismatches = 0usize;
    let mut ship = [0u64; MAXC];
    for (sid, ord, elapsed) in queries {
        streams[sid].eval(ord, &mut ship);
        let x = f64::from_bits(ship[1]);
        let z0 = f64::from_bits(ship[2]);
        let lazy = eval_gait(cycle, x, z0, elapsed);
        let oracle = step_law_oracle(world, cycle, x, z0, elapsed);
        let eq = lazy.x.to_bits() == oracle.position.x.to_bits()
            && lazy.y.to_bits() == oracle.position.y.to_bits()
            && lazy.z.to_bits() == oracle.position.z.to_bits()
            && lazy.vx.to_bits() == oracle.velocity.x.to_bits()
            && lazy.vy.to_bits() == oracle.velocity.y.to_bits()
            && lazy.vz.to_bits() == oracle.velocity.z.to_bits()
            && lazy.on_ground == oracle.on_ground
            && lazy.sprinting == oracle.sprinting;
        if !eq {
            mismatches += 1;
        }
    }

    let pass = mismatches == 0;
    println!(
        "  streams={STREAMS}, total_bots={total_bots}, sealed_scars={total_scars}, samples=1000, mismatches={mismatches} => {}",
        if pass { "PASS" } else { "FAIL" }
    );
    pass
}

fn run_check_2(cycle: &GaitCycle, mutant: Mutant) -> bool {
    println!("\n[Check 2] O(1) Space-Time Block-Edit Shock + Arrow + WorldGait (10^3 -> 10^9 ladder)");
    const CADENCE: u64 = 6; // ~2.14 blocks/bot spacing > 1.3-block body extent + jump impulse
    const SCARS_IN_STREAM: usize = 1_000;
    const TRIALS: usize = 100;
    let z0 = 0.5f64;
    let x_lane = 4.5f64;
    let z_wall = 200i32;
    let wall_pos = [4, 0, z_wall];

    // Undisturbed 4-tick WorldGait check over 100,000 ticks in ES Wheel.
    let world_gait = WorldGait::clock_4(0, wall_pos);
    let undisturbed_wakeups = verify_undisturbed_world_gait(&world_gait, 100_000);
    println!("  WorldGait (4-tick clock over 100,000 ticks): wheel wake-ups = {undisturbed_wakeups} (expected 0)");

    let dt_past = cycle
        .map_z
        .first_cross(z0, z_wall as f64 + 1.05, 10_000)
        .expect("crosses z_wall + 1.05");

    let ladder: &[u64] = if mutant == Mutant::PerBotIndex {
        &[1_000, 100_000]
    } else {
        &[1_000, 100_000, 10_000_000, 1_000_000_000]
    };

    let mut medians_ns: Vec<(f64, f64)> = Vec::new();
    let mut all_ok = undisturbed_wakeups == 0;
    let mut enc = Enc::new(false);

    for &n in ladder {
        // Build stream with 1,000 scars (varying kit_inv and gait_id, keeping z schedule monotone for clean geometric cut check).
        let prog = [
            Prog::Affine {
                a: 0,
                b: CADENCE as i64,
            },
            Prog::Const(x_lane.to_bits()),
            Prog::Const(z0.to_bits()),
            Prog::Const(0),
            Prog::Const(64),
            Prog::Const(0),
            Prog::Const(0),
            Prog::Const(0),
        ];
        let stride = (n / SCARS_IN_STREAM as u64).max(1);
        let scars: Vec<(u64, Ship)> = (0..SCARS_IN_STREAM)
            .map(|k| {
                let ord = ((k as u64) * stride).min(n - 1);
                (ord, make_ship(ord * CADENCE, x_lane, z0, 1, 64 - (k as u64 % 7)))
            })
            .collect();
        let stream = Program::seal_from(n, 5, prog, &scars, &mut enc);

        // Register Space-Time Index: stream, flying arrow, and WorldGait.
        let mut st_index = SpaceTimeIndex::default();
        let t_end_horizon = n * CADENCE + dt_past + 100;
        if mutant == Mutant::PerBotIndex {
            for i in 0..n {
                let t_hit = i * CADENCE + dt_past;
                st_index.register(wall_pos, 0, t_hit + 12, READ_SHAPE, CertHandle::Bot(i));
            }
        } else {
            st_index.register(wall_pos, 0, t_end_horizon, READ_SHAPE, CertHandle::Stream(0));
        }
        let t_star = (n / 2) * CADENCE + dt_past + 1;
        let i_star = ((t_star - dt_past) / CADENCE + 1).min(n);

        register_arrow_crossing(
            &mut st_index,
            0,
            Vec3::new(x_lane, 0.5, z_wall as f64 - 15.0),
            Vec3::new(0.0, 0.25, 1.5),
            t_star - 5,
            25,
            wall_pos,
        );
        st_index.register(wall_pos, 0, u64::MAX, READ_SHAPE, CertHandle::WorldGait(0));

        let mut samples_us = Vec::with_capacity(TRIALS);
        let mut woken = Vec::new();
        let mut last_stream_woken = 0usize;
        let mut last_arrow_woken = 0usize;
        let mut last_wg_woken = 0usize;

        for trial in 0..TRIALS {
            let mut local_stream = Program::seal_from(n, 5, prog, &scars, &mut enc);
            // Put 1 unsealed hot edit in local_stream to verify seal()-before-split safety.
            local_stream.update(0, make_ship(0, x_lane, z0, 0, 63));

            let t0 = Instant::now();
            st_index.notify_edit(t_star, wall_pos, READ_SHAPE, &mut woken);
            let mut s_cnt = 0usize;
            let mut a_cnt = 0usize;
            let mut wg_cnt = 0usize;
            let mut wg_phase = 0u8;
            for &h in &woken {
                match h {
                    CertHandle::Stream(_) | CertHandle::Bot(_) => s_cnt += 1,
                    CertHandle::Arrow(_) => a_cnt += 1,
                    CertHandle::WorldGait(_) => {
                        wg_cnt += 1;
                        wg_phase = world_gait.materialize(t_star);
                    }
                }
            }
            local_stream.seal(&mut enc);
            let split_ord = {
                let dt_clear = cycle
                    .map_z
                    .first_cross(z0, z_wall as f64 + 1.05, 10_000)
                    .expect("dt_clear");
                ((t_star - dt_clear) / CADENCE + 1).min(local_stream.n)
            };
            let (left, right) = local_stream.split(split_ord, &mut enc);
            let elapsed_us = t0.elapsed().as_secs_f64() * 1_000_000.0;
            samples_us.push(elapsed_us);

            if trial == 0 {
                last_stream_woken = s_cnt;
                last_arrow_woken = a_cnt;
                last_wg_woken = wg_cnt;
                assert_eq!(split_ord, i_star);
                assert_eq!(left.n + right.n, n, "ordinal conservation");
                assert_eq!(
                    left.scars() + right.scars(),
                    SCARS_IN_STREAM,
                    "all scars preserved across split"
                );
                assert_eq!(wg_phase, WorldGait::stepped_oracle(0, t_star));

                // Verify boundary bots around the split against z_wall.
                let mut s_buf = [0u64; MAXC];
                if left.n > 0 {
                    left.eval(left.n - 1, &mut s_buf);
                    let z_left_tail = cycle.map_z.advance(z0, t_star - s_buf[0]);
                    assert!(z_left_tail > z_wall as f64 + 1.0, "left tail z={z_left_tail}");
                }
                if right.n > 0 {
                    right.eval(0, &mut s_buf);
                    let z_right_head = cycle.map_z.advance(z0, t_star - s_buf[0]);
                    assert!(z_right_head < z_wall as f64 - 0.3, "right head z={z_right_head}");
                }
                // Spot-check all scars in left & right match original stream.
                let mut l_s = [0u64; MAXC];
                let mut o_s = [0u64; MAXC];
                for (_, ord) in left.ef.cursor(0) {
                    left.eval(ord, &mut l_s);
                    stream.eval(ord, &mut o_s);
                    if ord != 0 {
                        assert_eq!(l_s[..5], o_s[..5]);
                    }
                }
                for (_, ord) in right.ef.cursor(0) {
                    right.eval(ord, &mut l_s);
                    stream.eval(split_ord + ord, &mut o_s);
                    assert_eq!(l_s[..5], o_s[..5]);
                }
            }
        }

        let ds = compute_stats(&samples_us);
        print_stats(&format!("N={n} (index={} B, wake stream/bot={last_stream_woken}, arrow={last_arrow_woken}, wg={last_wg_woken})", st_index.bytes()), "us", &ds);
        medians_ns.push(((n as f64).log10(), (ds.median * 1000.0).max(1.0).log10()));

        if last_stream_woken != 1 || last_arrow_woken != 1 || last_wg_woken != 1 {
            all_ok = false;
        }
    }

    // Compute log-log slope of median latency vs N.
    let slope = if medians_ns.len() >= 2 {
        let first = medians_ns.first().unwrap();
        let last = medians_ns.last().unwrap();
        (last.1 - first.1) / (last.0 - first.0)
    } else {
        1.0
    };
    let pass = all_ok && slope.abs() < 0.05;
    println!(
        "  log-log latency slope across ladder = {:.4} (gate: |slope| < 0.05, stream wakeups == 1) => {}",
        slope,
        if pass { "PASS" } else { "FAIL" }
    );
    pass
}

fn run_check_3(mutant: Mutant) -> bool {
    println!("\n[Check 3] Scars, seal() Reabsorption, and < 32 MB L3 Residency");
    const STREAMS: usize = 1_000;
    const FIXED_SCARS: usize = 100_000; // 100 per stream
    let mut enc = Enc::new(false);

    let mut fixed_k_bytes = Vec::new();
    for &per_stream in &[1_000u64, 10_000, 100_000, 1_000_000] {
        let mut streams = Vec::with_capacity(STREAMS);
        for sid in 0..STREAMS {
            let (p, _) = build_scarred_stream(
                per_stream,
                (sid as u64) * 3,
                2,
                1.5 + (sid as f64) * 2.0,
                0.5,
                FIXED_SCARS / STREAMS,
                sid as u64 + 99,
                &mut enc,
            );
            streams.push(p);
        }
        let bytes: usize = streams.iter().map(Program::bytes).sum();
        let total_bots = per_stream * STREAMS as u64;
        println!(
            "  fixed k={FIXED_SCARS}, N={total_bots}: {:.3} MB ({:.2} B/scar, {:.6} B/bot)",
            bytes as f64 / (1024.0 * 1024.0),
            bytes as f64 / FIXED_SCARS as f64,
            bytes as f64 / total_bots as f64
        );
        fixed_k_bytes.push(bytes);
    }

    let growth_ratio = *fixed_k_bytes.last().unwrap() as f64 / fixed_k_bytes[0] as f64;

    // 10^9 bots (1,000 streams x 1,000,000) with 1,000,000 scars (1,000 per stream), then 50% rejoin.
    const SCARS_PER_1M: usize = 1_000;
    let mut streams_1b = Vec::with_capacity(STREAMS);
    let mut scar_ords: Vec<Vec<u64>> = Vec::with_capacity(STREAMS);
    for sid in 0..STREAMS {
        let (p, scars) = build_scarred_stream(
            1_000_000,
            (sid as u64) * 3,
            2,
            1.5 + (sid as f64) * 2.0,
            0.5,
            SCARS_PER_1M,
            sid as u64 + 777,
            &mut enc,
        );
        scar_ords.push(scars.into_iter().map(|(o, _)| o).collect());
        streams_1b.push(p);
    }

    let bytes_before: usize = streams_1b.iter().map(Program::bytes).sum();
    let scars_before: usize = streams_1b.iter().map(Program::scars).sum();

    let mut seal_us = Vec::with_capacity(STREAMS);
    for (sid, p) in streams_1b.iter_mut().enumerate() {
        let x_lane = 1.5 + (sid as f64) * 2.0;
        let t_base = (sid as u64) * 3;
        for &ord in scar_ords[sid].iter().take(SCARS_PER_1M / 2) {
            let restored = if mutant == Mutant::NoReabsorb {
                // Planted fault: fails to restore back to exact Prog value (kit_inv stays off by 1), preventing reabsorption.
                make_ship(t_base + 2 * ord, x_lane, 0.5, 0, 63)
            } else {
                make_ship(t_base + 2 * ord, x_lane, 0.5, 0, 64)
            };
            p.update(ord, restored);
        }
        let t0 = Instant::now();
        p.seal(&mut enc);
        seal_us.push(t0.elapsed().as_secs_f64() * 1_000_000.0);
    }

    let bytes_after: usize = streams_1b.iter().map(Program::bytes).sum();
    let scars_after: usize = streams_1b.iter().map(Program::scars).sum();
    let ds = compute_stats(&seal_us);
    println!(
        "  N=10^9, k=10^6 before rejoin: {:.3} MB ({:.2} B/scar, {:.4} B/bot)",
        bytes_before as f64 / (1024.0 * 1024.0),
        bytes_before as f64 / scars_before as f64,
        bytes_before as f64 / 1e9
    );
    println!(
        "  N=10^9 after 50% rejoin seal(): scars {scars_before} -> {scars_after}, bytes {:.3} MB -> {:.3} MB",
        bytes_before as f64 / (1024.0 * 1024.0),
        bytes_after as f64 / (1024.0 * 1024.0)
    );
    print_stats("seal() per stream (1,000 streams)", "us", &ds);

    let pass = bytes_before < 32 * 1024 * 1024
        && growth_ratio < 2.0
        && scars_after == scars_before / 2
        && bytes_after < bytes_before;
    println!(
        "  under_32MB={} ratio_1B_vs_1M={:.3}x reabsorbed={} => {}",
        bytes_before < 32 * 1024 * 1024,
        growth_ratio,
        scars_after == scars_before / 2 && bytes_after < bytes_before,
        if pass { "PASS" } else { "FAIL" }
    );
    pass
}

fn run_check_4(cycle: &GaitCycle, mutant: Mutant) -> bool {
    println!("\n[Check 4] Pure Observer Light-Cone Flyover (10M-bot highway, p99 & max < 1 ms, zero digest mutation)");
    const STREAMS: usize = 10;
    const BOTS_PER_STREAM: u64 = 1_000_000;
    const SCARS_PER_STREAM: usize = 1_000;
    const CADENCE: u64 = 2;
    const FRAMES: usize = 500;
    let z0 = 0.5f64;

    let mut enc = Enc::new(false);
    let mut streams = Vec::with_capacity(STREAMS);
    for sid in 0..STREAMS {
        let (p, _) = build_scarred_stream(
            BOTS_PER_STREAM,
            0,
            CADENCE,
            1.5 + (sid as f64) * 2.0,
            z0,
            SCARS_PER_STREAM,
            sid as u64 + 31415,
            &mut enc,
        );
        streams.push(p);
    }

    let digest_without_observer = universe_digest(&streams);

    let mut instances: Vec<GpuBotInstance> = Vec::with_capacity(16_384);
    let mut vis_scars: Vec<(u64, u64, f64)> = Vec::with_capacity(256); // (ord, scar_t_dep, scar_x)
    let mut frame_us = Vec::with_capacity(FRAMES);
    let mut vis_counts = Vec::with_capacity(FRAMES);
    let mut bit_mismatches = 0usize;

    for frame in 0..FRAMES {
        let t_frame = 10_000u64 + frame as u64;
        let cam_z = 500.0 + (frame as f64) * 5.0;
        let z_lo = cam_z - 192.0;
        let z_hi = cam_z + 192.0;

        let t0 = Instant::now();
        if mutant == Mutant::DirtyObserver && frame == 0 {
            // Planted fault: observer mutates simulation state.
            streams[0].update(0, make_ship(999, 1.5, z0, 0, 64));
        }
        instances.clear();

        for (sid, prog) in streams.iter().enumerate() {
            let x_base = 1.5 + (sid as f64) * 2.0;
            let Some(dt_lo) = cycle.map_z.first_cross(z0, z_lo, t_frame) else {
                continue;
            };
            let dt_hi_excl = cycle.map_z.first_cross(z0, z_hi + 1.0e-9, t_frame);
            if t_frame < dt_lo {
                continue;
            }
            let i_max = ((t_frame - dt_lo) / CADENCE).min(prog.n - 1);
            let i_min = match dt_hi_excl {
                Some(d) if t_frame >= d => ((t_frame - d) / CADENCE + 1).min(prog.n),
                _ => 0,
            };

            // Collect potentially visible scars widened by maxdev[0].
            let pad_ord = prog.maxdev[0] / CADENCE + 2;
            let ord_scan_lo = i_min.saturating_sub(pad_ord);
            let ord_scan_hi = (i_max + pad_ord).min(prog.n - 1);
            vis_scars.clear();
            let r0 = prog.ef.rank(ord_scan_lo);
            let mut ship = [0u64; MAXC];
            for (_, o) in prog.ef.cursor(r0) {
                if o > ord_scan_hi {
                    break;
                }
                prog.eval(o, &mut ship);
                vis_scars.push((o, ship[0], f64::from_bits(ship[1])));
            }

            // Emit unscarred bots in [i_min..=i_max] stepping dz in O(1) per bot.
            if i_min <= i_max {
                let mut elapsed = t_frame - CADENCE * i_max;
                let mut z = cycle.map_z.advance(z0, elapsed);
                let mut scar_idx = vis_scars.len();
                let mut i = i_max;
                loop {
                    while scar_idx > 0 && vis_scars[scar_idx - 1].0 > i {
                        scar_idx -= 1;
                    }
                    let is_scarred = scar_idx > 0 && vis_scars[scar_idx - 1].0 == i;
                    if !is_scarred && z >= z_lo && z <= z_hi {
                        if frame % 50 == 0 && (i == i_max || i == i_min || i % 100 == 0) {
                            let z_exact = cycle.map_z.advance(z0, elapsed);
                            if z.to_bits() != z_exact.to_bits() {
                                bit_mismatches += 1;
                            }
                        }
                        let phase = (elapsed % GAIT_PERIOD as u64) as usize;
                        let y = cycle.phases[phase].y;
                        instances.push(GpuBotInstance {
                            pos: [x_base as f32, y as f32, z as f32],
                            yaw: 0.0,
                            flags: (sid as u32) << 8 | phase as u32,
                            ordinal: i,
                        });
                    }
                    if i == i_min {
                        break;
                    }
                    i -= 1;
                    let p0 = (elapsed % GAIT_PERIOD as u64) as usize;
                    let p1 = (p0 + 1) % GAIT_PERIOD;
                    z += cycle.dz[p0];
                    z += cycle.dz[p1];
                    elapsed += CADENCE;
                }
            }

            // Emit any scarred bots inside [z_lo, z_hi].
            for &(o, s_tdep, s_x) in &vis_scars {
                if t_frame >= s_tdep {
                    let dt = t_frame - s_tdep;
                    let z = cycle.map_z.advance(z0, dt);
                    if z >= z_lo && z <= z_hi {
                        let phase = (dt % GAIT_PERIOD as u64) as usize;
                        let y = cycle.phases[phase].y;
                        instances.push(GpuBotInstance {
                            pos: [s_x as f32, y as f32, z as f32],
                            yaw: 0.0,
                            flags: 0x8000_0000 | ((sid as u32) << 8) | phase as u32,
                            ordinal: o,
                        });
                    }
                }
            }
        }

        frame_us.push(t0.elapsed().as_secs_f64() * 1_000_000.0);
        vis_counts.push(instances.len() as f64);
    }

    let digest_with_observer = universe_digest(&streams);
    let ds_t = compute_stats(&frame_us);
    let ds_c = compute_stats(&vis_counts);
    print_stats("visible bots per frame", "bots", &ds_c);
    print_stats("frame extraction latency (500 frames)", "us", &ds_t);

    let pass = ds_t.p99 < 1_000.0
        && ds_t.max < 1_000.0
        && bit_mismatches == 0
        && digest_with_observer == digest_without_observer;
    println!(
        "  bit_mismatches={bit_mismatches}, digest_equal={}, p99={:.2} us, worst_max={:.2} us => {}",
        digest_with_observer == digest_without_observer,
        ds_t.p99,
        ds_t.max,
        if pass { "PASS" } else { "FAIL" }
    );
    pass
}

fn run_check_5(world: &BoxWorld, cycle: &GaitCycle) -> bool {
    println!("\n[Check 5] Dense Pit Promotion & Dissolution (500 bots with real law::entity_contact::push_pair)");
    let r = dense::run_dense_pit_check(world, cycle);
    let ds = compute_stats(&r.tick_ms);
    println!(
        "  promote_tick={}, dense_ticks={}, bots={}, push_pairs_applied={}",
        r.promote_tick, r.dense_ticks, r.bot_count, r.push_interactions
    );
    print_stats("dense Battle step (500 bots)", "ms/tick", &ds);
    println!(
        "  battle_bytes_during={} B -> after_dissolve={} B, dissolved_FormP={} B",
        r.battle_bytes_during, r.battle_bytes_after_dissolve, r.dissolved_program_bytes
    );
    let pass = r.oracle_mismatches == 0
        && r.dissolved_mismatches == 0
        && r.battle_bytes_after_dissolve == 0
        && r.total_inventory_after == (r.bot_count as u64) * 64;
    println!(
        "  oracle_mismatches={}, dissolved_mismatches={}, inventory={}/{} => {}",
        r.oracle_mismatches,
        r.dissolved_mismatches,
        r.total_inventory_after,
        r.bot_count * 64,
        if pass { "PASS" } else { "FAIL" }
    );
    pass
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut mutant = Mutant::None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => {}
            "--mutant" => {
                i += 1;
                let name = args.get(i).map(String::as_str).unwrap_or("");
                mutant = match name {
                    "none" => Mutant::None,
                    "per-bot-index" => Mutant::PerBotIndex,
                    "no-reabsorb" => Mutant::NoReabsorb,
                    "dirty-observer" => Mutant::DirtyObserver,
                    other => panic!("unknown mutant {other}"),
                };
            }
            other => panic!("unknown arg {other}"),
        }
        i += 1;
    }

    println!("=== ES 1B-Bot Probe (mutant: {mutant:?}) ===");
    let world = make_flat_stone_world();
    let cycle = calibrate_sprint_jump(&world);
    println!(
        "Calibrated 26.2 sprint-jump: settle={} ticks, period=12, total_dz={:.6}, footprint_reads={} cells",
        cycle.settle_ticks,
        cycle.dz.iter().sum::<f64>(),
        cycle.cycle_footprint.reads.len()
    );

    let ok = match mutant {
        Mutant::None => {
            let c1 = run_check_1(&world, &cycle);
            let c2 = run_check_2(&cycle, mutant);
            let c3 = run_check_3(mutant);
            let c4 = run_check_4(&cycle, mutant);
            let c5 = run_check_5(&world, &cycle);
            c1 && c2 && c3 && c4 && c5
        }
        Mutant::PerBotIndex => run_check_2(&cycle, mutant),
        Mutant::NoReabsorb => run_check_3(mutant),
        Mutant::DirtyObserver => run_check_4(&cycle, mutant),
    };

    if !ok {
        std::process::exit(1);
    }
}
