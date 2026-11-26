use std::time::Instant;

use gpu::{
    GpuContext,
    crowd_push::{CrowdState, step_crowd_cpu},
    min_plus::{LegsCost, PackedTopoGrid, solve_min_plus_cpu},
    rollout::{
        ROLLOUT_FLAG_FORWARD, ROLLOUT_FLAG_JUMP, ROLLOUT_FLAG_SPRINT, RolloutReq, SolidVoxelBox,
        run_rollouts_cpu,
    },
};
use law::{BoxWorld, Input, PlayerState, Vec3, try_tick};

#[test]
fn gpu_selects_discrete_rx_7900_xtx_and_measures_roundtrip() {
    let ctx = GpuContext::global().expect("RX 7900 XTX OpenCL device should initialize");
    let info = ctx.device_info();
    eprintln!(
        "GPU: {} | CUs={} | Clock={}MHz | VRAM={:.2}GB | fp64={} | rt=({:.1},{:.1},{:.1})us",
        info.name,
        info.compute_units,
        info.clock_mhz,
        info.global_mem_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        info.has_fp64,
        ctx.roundtrip_us.0,
        ctx.roundtrip_us.1,
        ctx.roundtrip_us.2,
    );
    assert!(info.compute_units >= 40, "expected discrete GPU, got {} CUs ({})", info.compute_units, info.name);
    assert!(info.has_fp64, "expected cl_khr_fp64 support");
    assert!(ctx.roundtrip_us.1 > 0.0 && ctx.roundtrip_us.1 < 2_000.0, "median roundtrip {} us", ctx.roundtrip_us.1);
}

#[test]
fn min_plus_gpu_matches_cpu_and_exceeds_10x_speedup() {
    let ctx = GpuContext::global().expect("GPU context");
    let legs = LegsCost::default();

    // Exactness parity check against CPU BinaryHeap Dijkstra on multi-wall/bridge arena.
    let small = PackedTopoGrid::synthetic_arena(32, 6, 32);
    let small_goals = vec![
        (6 * small.dy + 1) * small.dx + 6,
        (24 * small.dy + 1) * small.dx + 24,
        (8 * small.dy + 1) * small.dx + 26,
        (26 * small.dy + 1) * small.dx + 8,
    ];
    let cpu_ref = solve_min_plus_cpu(&small, &small_goals, legs, 1);
    let (gpu_res, sweeps, _) = ctx.min_plus.solve_batch(&small, &small_goals, legs, 256).unwrap();
    let max_diff = cpu_ref
        .iter()
        .zip(gpu_res.iter())
        .filter(|(a, b)| a.is_finite() || b.is_finite())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(max_diff < 1e-3, "min_plus parity failed after {sweeps} sweeps: max_diff={max_diff}");

    // End-to-end 10x speedup benchmark: 32 goals over 64x8x64 obstacle arena.
    let grid = PackedTopoGrid::synthetic_arena(64, 8, 64);
    let mut goals = Vec::new();
    for g in 0..32usize {
        let x = 8 + (g * 7) % 48;
        let z = 8 + (g * 11) % 48;
        goals.push((z * grid.dy + 1) * grid.dx + x);
    }
    let t0 = Instant::now();
    let cpu_big = solve_min_plus_cpu(&grid, &goals, legs, 1);
    let cpu_ms = t0.elapsed().as_secs_f64() * 1e3;

    let (gpu_big, big_sweeps, gpu_ms) = ctx.min_plus.solve_batch(&grid, &goals, legs, 256).unwrap();
    let big_diff = cpu_big
        .iter()
        .zip(gpu_big.iter())
        .filter(|(a, b)| a.is_finite() || b.is_finite())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    let speedup = cpu_ms / gpu_ms;
    eprintln!("[min_plus] cpu_1t={cpu_ms:.2}ms gpu_e2e={gpu_ms:.2}ms ({big_sweeps} sweeps) speedup={speedup:.2}x max_err={big_diff:.2e}");
    assert!(big_diff < 1e-3, "large min_plus parity failed: {big_diff}");
    assert!(speedup >= 10.0, "min_plus speedup {speedup:.2}x < 10.0x (cpu={cpu_ms:.2}ms, gpu={gpu_ms:.2}ms)");
}

#[test]
fn crowd_push_gpu_matches_law_push_pair_and_exceeds_10x_speedup() {
    let ctx = GpuContext::global().expect("GPU context");

    // 1-tick exact fp64 parity against law::entity_contact::push_pair on 2,048 overlapping bots.
    let small = CrowdState::dense_arena(2_048, 16.0);
    let mut s_cpu = small.clone();
    let mut s_gpu = small;
    step_crowd_cpu(&mut s_cpu, 0.0, 0.0, 16, 16, 1, 1);
    ctx.crowd_push.step_crowd(&mut s_gpu, 0.0, 0.0, 16, 16, 1).unwrap();
    let vel_err = s_cpu
        .vel_x
        .iter()
        .zip(s_gpu.vel_x.iter())
        .chain(s_cpu.vel_z.iter().zip(s_gpu.vel_z.iter()))
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    assert!(vel_err < 1e-12, "crowd_push velocity mismatch vs law::push_pair: {vel_err}");

    // End-to-end 10x speedup benchmark: 32,768 dense bots x 20 ticks.
    let dense = CrowdState::dense_arena(32_768, 64.0);
    let mut c_state = dense.clone();
    let t0 = Instant::now();
    step_crowd_cpu(&mut c_state, 0.0, 0.0, 64, 64, 20, 1);
    let cpu_ms = t0.elapsed().as_secs_f64() * 1e3;

    let mut g_state = dense;
    let gpu_ms = ctx.crowd_push.step_crowd(&mut g_state, 0.0, 0.0, 64, 64, 20).unwrap();
    let pos_err = c_state
        .pos_x
        .iter()
        .zip(g_state.pos_x.iter())
        .chain(c_state.pos_z.iter().zip(g_state.pos_z.iter()))
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    let speedup = cpu_ms / gpu_ms;
    eprintln!("[crowd_push] cpu_1t={cpu_ms:.2}ms gpu_e2e={gpu_ms:.2}ms speedup={speedup:.2}x max_err={pos_err:.2e}");
    assert!(pos_err < 1e-6, "32k 20t crowd pos mismatch: {pos_err}");
    assert!(speedup >= 10.0, "crowd_push speedup {speedup:.2}x < 10.0x (cpu={cpu_ms:.2}ms, gpu={gpu_ms:.2}ms)");
}

#[test]
fn batched_rollouts_gpu_match_cpu_and_exceed_10x_speedup() {
    let ctx = GpuContext::global().expect("GPU context");
    let voxels = SolidVoxelBox::flat_floor([0, -1, 0], 128, 8, 128, 0);

    // Cross-check GPU rollout against real law::try_tick for sprint & sprint-jump on flat stone.
    let mut law_world = BoxWorld::default();
    law_world.fill([0, -1, 0], [127, -1, 127], law::Block::full());
    let mut law_s = PlayerState {
        position: Vec3::new(16.5, 0.0, 16.5),
        on_ground: true,
        main_supporting_block: Some([16, -1, 16]),
        food_level: 20,
        ..PlayerState::default()
    };
    let yaw = 37.5f32;
    let inp = Input {
        forward: true,
        sprint: true,
        jump: true,
        yaw,
        ..Input::default()
    };
    for _ in 0..32 {
        try_tick(&mut law_s, &law_world, &inp).unwrap();
    }
    let check_req = vec![RolloutReq {
        x: 16.5,
        y: 0.0,
        z: 16.5,
        vx: 0.0,
        vy: 0.0,
        vz: 0.0,
        yaw,
        flags: ROLLOUT_FLAG_FORWARD | ROLLOUT_FLAG_SPRINT | ROLLOUT_FLAG_JUMP,
        ticks: 32,
        on_ground: 1,
    }];
    let (law_gpu, _) = ctx.rollout.run_batch(&check_req, &voxels).unwrap();
    let law_err = (law_gpu[0].x - law_s.position.x)
        .abs()
        .max((law_gpu[0].y - law_s.position.y).abs())
        .max((law_gpu[0].z - law_s.position.z).abs());
    assert!(law_err < 1e-6, "GPU rollout vs law::try_tick error={law_err} (gpu={:?} vs law={:?})", law_gpu[0], law_s.position);

    // End-to-end 10x speedup benchmark: 131,072 candidates x 64 ticks.
    let mut reqs = Vec::with_capacity(131_072);
    for i in 0..131_072usize {
        let fx = 16.5 + ((i % 64) as f64);
        let fz = 16.5 + (((i / 64) % 64) as f64);
        let ryaw = ((i * 13) % 360) as f32;
        reqs.push(RolloutReq {
            x: fx,
            y: 0.0,
            z: fz,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            yaw: ryaw,
            flags: ROLLOUT_FLAG_FORWARD | ROLLOUT_FLAG_SPRINT | if i % 3 == 0 { ROLLOUT_FLAG_JUMP } else { 0 },
            ticks: 64,
            on_ground: 1,
        });
    }
    let t0 = Instant::now();
    let cpu_out = run_rollouts_cpu(&reqs, &voxels, 1);
    let cpu_ms = t0.elapsed().as_secs_f64() * 1e3;

    let (gpu_out, gpu_ms) = ctx.rollout.run_batch(&reqs, &voxels).unwrap();
    let max_err = cpu_out
        .iter()
        .zip(gpu_out.iter())
        .map(|(a, b)| (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()))
        .fold(0.0f64, f64::max);
    let speedup = cpu_ms / gpu_ms;
    eprintln!("[rollout] cpu_1t={cpu_ms:.2}ms gpu_e2e={gpu_ms:.2}ms speedup={speedup:.2}x max_err={max_err:.2e}");
    assert!(max_err < 1e-10, "batched rollout GPU vs CPU error: {max_err}");
    assert!(speedup >= 10.0, "rollout speedup {speedup:.2}x < 10.0x (cpu={cpu_ms:.2}ms, gpu={gpu_ms:.2}ms)");
}
