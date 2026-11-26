use std::time::Instant;

use gpu::{
    GpuContext,
    crowd_push::{CrowdState, step_crowd_cpu},
    min_plus::{LegsCost, PackedTopoGrid, solve_min_plus_cpu},
    rollout::{
        ROLLOUT_FLAG_FORWARD, ROLLOUT_FLAG_JUMP, ROLLOUT_FLAG_SPRINT, RolloutReq,
        SolidVoxelBox, run_rollouts_cpu,
    },
};

fn main() {
    let ctx = match GpuContext::init() {
        Ok(c) => c,
        Err(why) => {
            eprintln!("GPU init failed: {why}");
            std::process::exit(1);
        }
    };
    let info = ctx.device_info();
    println!(
        "Device: {} | CUs={} | Clock={}MHz | VRAM={:.2}GB | fp64={} | Roundtrip min={:.1}us med={:.1}us max={:.1}us",
        info.name,
        info.compute_units,
        info.clock_mhz,
        info.global_mem_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        info.has_fp64,
        ctx.roundtrip_us.0,
        ctx.roundtrip_us.1,
        ctx.roundtrip_us.2,
    );

    // 1. Batched 3D (min, +) TopoField wavefronts.
    let grid = PackedTopoGrid::synthetic_arena(64, 8, 64);
    let mut goals = Vec::new();
    for g in 0..32usize {
        let x = 8 + (g * 7) % 48;
        let z = 8 + (g * 11) % 48;
        goals.push((z * grid.dy + 1) * grid.dx + x);
    }
    let legs = LegsCost::default();
    let t_cpu1 = Instant::now();
    let cpu1 = solve_min_plus_cpu(&grid, &goals, legs, 1);
    let ms_cpu1 = t_cpu1.elapsed().as_secs_f64() * 1e3;

    let t_cpu8 = Instant::now();
    let _cpu8 = solve_min_plus_cpu(&grid, &goals, legs, 8);
    let ms_cpu8 = t_cpu8.elapsed().as_secs_f64() * 1e3;

    let (gpu_out, sweeps, ms_gpu) = ctx.min_plus.solve_batch(&grid, &goals, legs, 256).unwrap();
    let max_diff = cpu1
        .iter()
        .zip(gpu_out.iter())
        .filter(|(a, b)| a.is_finite() || b.is_finite())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    println!(
        "[min_plus 32 goals x 64x8x64] cpu_1t={ms_cpu1:.2}ms cpu_8t={ms_cpu8:.2}ms gpu_e2e={ms_gpu:.2}ms ({sweeps} sweeps) | vs_1t={:.2}x vs_8t={:.2}x | max_err={max_diff:.2e}",
        ms_cpu1 / ms_gpu,
        ms_cpu8 / ms_gpu
    );

    // 2. Dense Knudsen ≈ 1 Entity.push crowd (32,768 bots x 20 ticks).
    let base = CrowdState::dense_arena(32_768, 64.0);
    let mut s_cpu1 = base.clone();
    let t0 = Instant::now();
    step_crowd_cpu(&mut s_cpu1, 0.0, 0.0, 64, 64, 20, 1);
    let ms_c1 = t0.elapsed().as_secs_f64() * 1e3;

    let mut s_cpu8 = base.clone();
    let t0 = Instant::now();
    step_crowd_cpu(&mut s_cpu8, 0.0, 0.0, 64, 64, 20, 8);
    let ms_c8 = t0.elapsed().as_secs_f64() * 1e3;

    let mut s_gpu = base;
    let ms_g = ctx.crowd_push.step_crowd(&mut s_gpu, 0.0, 0.0, 64, 64, 20).unwrap();
    let max_pos_err = s_cpu1
        .pos_x
        .iter()
        .zip(s_gpu.pos_x.iter())
        .chain(s_cpu1.pos_z.iter().zip(s_gpu.pos_z.iter()))
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    println!(
        "[crowd_push 32,768 bots x 20 ticks] cpu_1t={ms_c1:.2}ms cpu_8t={ms_c8:.2}ms gpu_e2e={ms_g:.2}ms | vs_1t={:.2}x vs_8t={:.2}x | max_err={max_pos_err:.2e}",
        ms_c1 / ms_g,
        ms_c8 / ms_g
    );

    // 3. Batched sin.bin-exact kinematic rollouts (131,072 candidates x 64 ticks).
    let voxels = SolidVoxelBox::flat_floor([0, -1, 0], 128, 8, 128, 0);
    let mut reqs = Vec::with_capacity(131_072);
    for i in 0..131_072usize {
        let fx = 16.5 + ((i % 64) as f64);
        let fz = 16.5 + (((i / 64) % 64) as f64);
        let yaw = ((i * 13) % 360) as f32;
        reqs.push(RolloutReq {
            x: fx,
            y: 0.0,
            z: fz,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            yaw,
            flags: ROLLOUT_FLAG_FORWARD | ROLLOUT_FLAG_SPRINT | if i % 3 == 0 { ROLLOUT_FLAG_JUMP } else { 0 },
            ticks: 64,
            on_ground: 1,
        });
    }
    let t0 = Instant::now();
    let r_cpu1 = run_rollouts_cpu(&reqs, &voxels, 1);
    let ms_r1 = t0.elapsed().as_secs_f64() * 1e3;

    let t0 = Instant::now();
    let _r_cpu8 = run_rollouts_cpu(&reqs, &voxels, 8);
    let ms_r8 = t0.elapsed().as_secs_f64() * 1e3;

    let (r_gpu, ms_rg) = ctx.rollout.run_batch(&reqs, &voxels).unwrap();
    let max_r_err = r_cpu1
        .iter()
        .zip(r_gpu.iter())
        .map(|(a, b)| (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()))
        .fold(0.0f64, f64::max);
    println!(
        "[rollout 131,072 x 64t] cpu_1t={ms_r1:.2}ms cpu_8t={ms_r8:.2}ms gpu_e2e={ms_rg:.2}ms | vs_1t={:.2}x vs_8t={:.2}x | max_err={max_r_err:.2e}",
        ms_r1 / ms_rg,
        ms_r8 / ms_rg
    );
}
