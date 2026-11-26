//! Zero-dependency OpenCL GPU compute backend for RX 7900 XTX (`gfx1100`):
//! batched `(min, +)` fields, dense `Knudsen ≈ 1` `Entity.push` crowds, and `sin.bin`-exact rollouts.
pub mod crowd_push;
pub mod min_plus;
pub mod opencl;
pub mod rollout;

use std::sync::OnceLock;

use crowd_push::{CrowdPushGpuProgram, CrowdState, step_crowd_cpu};
use min_plus::MinPlusGpuProgram;
use opencl::{GpuDevice, GpuDeviceInfo};
use rollout::RolloutGpuProgram;

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
        candidates.saturating_mul(ticks) >= self.rollout_min_bot_ticks
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
