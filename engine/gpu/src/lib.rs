//! Zero-dependency OpenCL GPU compute backend for RX 7900 XTX (`gfx1100`):
//! batched `(min, +)` fields, dense `Knudsen ≈ 1` `Entity.push` crowds, and `sin.bin`-exact rollouts.
pub mod crowd_push;
pub mod min_plus;
pub mod opencl;
pub mod rollout;

use std::sync::OnceLock;

use crowd_push::{CrowdPushGpuProgram, CrowdState, step_crowd_cpu};
use min_plus::{LegsCost, MinPlusGpuProgram, PackedTopoGrid, UploadedTopoGrid, solve_min_plus_cpu};
use opencl::{GpuDevice, GpuDeviceInfo};
use rollout::{
    RolloutGpuProgram, RolloutOut, RolloutReq, SolidVoxelBox, UploadedVoxelBox, run_rollouts_cpu,
};

#[derive(Clone, Copy, Debug)]
pub struct CrossoverPolicy {
    pub min_plus_min_state_updates: usize,
    pub crowd_push_min_bots: usize,
    pub rollout_min_bot_ticks: usize,
}

impl Default for CrossoverPolicy {
    fn default() -> Self {
        Self {
            min_plus_min_state_updates: 250_000,
            crowd_push_min_bots: 8_192,
            rollout_min_bot_ticks: 1_000_000,
        }
    }
}

impl CrossoverPolicy {
    pub fn should_use_min_plus(&self, goals: usize, vol: usize) -> bool {
        goals.saturating_mul(vol) >= self.min_plus_min_state_updates
    }

    pub fn should_use_crowd(&self, bots: usize, ticks: usize) -> bool {
        bots >= self.crowd_push_min_bots && ticks >= 1
    }

    pub fn should_use_rollout(&self, candidates: usize, ticks: usize) -> bool {
        self.should_use_rollout_total(candidates.saturating_mul(ticks))
    }

    pub fn should_use_rollout_total(&self, total_bot_ticks: usize) -> bool {
        total_bot_ticks >= self.rollout_min_bot_ticks
    }
}

pub struct GpuContext {
    pub device: GpuDevice,
    pub min_plus: MinPlusGpuProgram,
    pub crowd_push: CrowdPushGpuProgram,
    pub rollout: RolloutGpuProgram,
    pub roundtrip_us: (f64, f64, f64),
    pub policy: CrossoverPolicy,
}

impl GpuContext {
    pub fn init() -> Result<Self, String> {
        let device = GpuDevice::open_best()?;
        let min_plus = MinPlusGpuProgram::compile(&device)?;
        let crowd_push = CrowdPushGpuProgram::compile(&device)?;
        let rollout = RolloutGpuProgram::compile(&device)?;
        let roundtrip_us = device.measure_roundtrip_us()?;
        Ok(Self {
            device,
            min_plus,
            crowd_push,
            rollout,
            roundtrip_us,
            policy: CrossoverPolicy::default(),
        })
    }

    pub fn global() -> Option<&'static Self> {
        static CTX: OnceLock<Option<GpuContext>> = OnceLock::new();
        CTX.get_or_init(|| Self::init().ok()).as_ref()
    }

    pub fn device_info(&self) -> &GpuDeviceInfo {
        self.device.info()
    }
}

/// Crossover-gated batched 3D `(min, +)` `TopoField` wavefront solver: runs on RX 7900 XTX
/// when `goals * vol >= 250_000`, else CPU `BinaryHeap` reference.
/// Returns `Ok((dist, true))` if executed on GPU, `Ok((dist, false))` if executed on CPU.
pub fn min_plus_auto(
    grid: &PackedTopoGrid,
    goal_cells: &[usize],
    legs: LegsCost,
    max_sweeps: usize,
) -> Result<(Vec<f32>, bool), String> {
    min_plus_auto_uploaded(grid, None, goal_cells, legs, max_sweeps)
}

/// Same as `min_plus_auto`, reusing a pre-uploaded VRAM grid buffer when available.
pub fn min_plus_auto_uploaded(
    grid: &PackedTopoGrid,
    uploaded: Option<&UploadedTopoGrid>,
    goal_cells: &[usize],
    legs: LegsCost,
    max_sweeps: usize,
) -> Result<(Vec<f32>, bool), String> {
    if goal_cells.is_empty() || grid.vol() == 0 {
        return Ok((Vec::new(), false));
    }
    if let Some(ctx) = GpuContext::global()
        && ctx.policy.should_use_min_plus(goal_cells.len(), grid.vol())
    {
        let sweeps = if max_sweeps == 0 {
            (grid.dx + grid.dy + grid.dz) * 2
        } else {
            max_sweeps
        };
        let (dist, _, _) = match uploaded {
            Some(up) => ctx
                .min_plus
                .solve_batch_uploaded(up, goal_cells, legs, sweeps)?,
            None => ctx.min_plus.solve_batch(grid, goal_cells, legs, sweeps)?,
        };
        return Ok((dist, true));
    }
    Ok((solve_min_plus_cpu(grid, goal_cells, legs, 8), false))
}

/// Crossover-gated batched `sin.bin`-exact kinematic rollouts: runs on RX 7900 XTX
/// when `candidates * ticks >= 1_000_000`, else CPU reference.
/// Returns `Ok((outs, true))` if executed on GPU, `Ok((outs, false))` if executed on CPU.
pub fn rollout_auto(
    reqs: &[RolloutReq],
    voxels: &SolidVoxelBox,
) -> Result<(Vec<RolloutOut>, bool), String> {
    rollout_auto_uploaded(reqs, voxels, None)
}

/// Same as `rollout_auto`, reusing a pre-uploaded VRAM voxel buffer when available.
pub fn rollout_auto_uploaded(
    reqs: &[RolloutReq],
    voxels: &SolidVoxelBox,
    uploaded: Option<&UploadedVoxelBox>,
) -> Result<(Vec<RolloutOut>, bool), String> {
    if reqs.is_empty() {
        return Ok((Vec::new(), false));
    }
    let total_bot_ticks = reqs
        .iter()
        .fold(0usize, |acc, r| acc.saturating_add(r.ticks as usize));
    if let Some(ctx) = GpuContext::global()
        && ctx.policy.should_use_rollout_total(total_bot_ticks)
    {
        let (outs, _) = match uploaded {
            Some(up) => ctx.rollout.run_batch_uploaded(reqs, up)?,
            None => ctx.rollout.run_batch(reqs, voxels)?,
        };
        return Ok((outs, true));
    }
    Ok((run_rollouts_cpu(reqs, voxels, 8), false))
}

/// Crossover-gated dense `Entity.push` step: runs on RX 7900 XTX above crossover, else CPU reference.
/// Returns `Ok(true)` if executed on GPU, `Ok(false)` if executed on CPU.
pub fn crowd_push_auto(
    state: &mut CrowdState,
    origin_x: f64,
    origin_z: f64,
    grid_w: usize,
    grid_h: usize,
    ticks: usize,
) -> Result<bool, String> {
    if let Some(ctx) = GpuContext::global()
        && ctx.policy.should_use_crowd(state.len(), ticks)
    {
        ctx.crowd_push
            .step_crowd(state, origin_x, origin_z, grid_w, grid_h, ticks)?;
        return Ok(true);
    }
    step_crowd_cpu(state, origin_x, origin_z, grid_w, grid_h, ticks, 8);
    Ok(false)
}
