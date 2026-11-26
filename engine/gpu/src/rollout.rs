//! Batched `sin.bin`-exact kinematic rollouts on OpenCL (`fp64`) + CPU reference.
use std::{sync::Mutex, time::Instant};

use law::{BlockPos, World, math};

use crate::opencl::{GpuBuffer, GpuDevice, GpuKernel, GpuProgram};

pub const ROLLOUT_FLAG_FORWARD: u32 = 1 << 0;
pub const ROLLOUT_FLAG_SPRINT: u32 = 1 << 1;
pub const ROLLOUT_FLAG_JUMP: u32 = 1 << 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RolloutReq {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub yaw: f32,
    pub flags: u32,
    pub ticks: u32,
    pub on_ground: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RolloutOut {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub on_ground: u32,
    pub horiz_collided: u32,
}

#[derive(Clone)]
pub struct SolidVoxelBox {
    pub min: BlockPos,
    pub dx: usize,
    pub dy: usize,
    pub dz: usize,
    /// Bitset over `(iz * dy + iy) * dx + ix`, 1 bit per cell (32 cells per u32).
    pub bits: Vec<u32>,
}

impl SolidVoxelBox {
    pub fn flat_floor(min: BlockPos, dx: usize, dy: usize, dz: usize, floor_ly: usize) -> Self {
        let words = (dx * dy * dz).div_ceil(32).max(1);
        let mut bits = vec![0u32; words];
        for iz in 0..dz {
            for ix in 0..dx {
                let ci = (iz * dy + floor_ly) * dx + ix;
                bits[ci >> 5] |= 1u32 << (ci & 31);
            }
        }
        Self { min, dx, dy, dz, bits }
    }

    pub fn from_world(world: &impl World, min: BlockPos, max: BlockPos) -> Self {
        let dx = (max[0] - min[0] + 1).max(1) as usize;
        let dy = (max[1] - min[1] + 1).max(1) as usize;
        let dz = (max[2] - min[2] + 1).max(1) as usize;
        let words = (dx * dy * dz).div_ceil(32).max(1);
        let mut bits = vec![0u32; words];
        for iz in 0..dz {
            let z = min[2] + iz as i32;
            for iy in 0..dy {
                let y = min[1] + iy as i32;
                for ix in 0..dx {
                    let x = min[0] + ix as i32;
                    let p = [x, y, z];
                    if world.loaded(p) && !world.block(p).never_collides() {
                        let ci = (iz * dy + iy) * dx + ix;
                        bits[ci >> 5] |= 1u32 << (ci & 31);
                    }
                }
            }
        }
        Self { min, dx, dy, dz, bits }
    }

    #[inline]
    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        let ix = x - self.min[0];
        let iy = y - self.min[1];
        let iz = z - self.min[2];
        if ix < 0
            || iy < 0
            || iz < 0
            || ix >= self.dx as i32
            || iy >= self.dy as i32
            || iz >= self.dz as i32
        {
            return false;
        }
        let ci = (iz as usize * self.dy + iy as usize) * self.dx + ix as usize;
        ((self.bits[ci >> 5] >> (ci & 31)) & 1) != 0
    }
}

#[inline]
fn sin_from_lut(lut: &[f32], rad: f64) -> f32 {
    lut[((rad * 10430.378350470453) as i64 & 65535) as usize]
}

#[inline]
fn cos_from_lut(lut: &[f32], rad: f64) -> f32 {
    lut[((rad * 10430.378350470453 + 16384.0) as i64 & 65535) as usize]
}

#[inline]
fn sweep_box_one(
    voxels: &SolidVoxelBox,
    bmin_x: f64,
    bmin_y: f64,
    bmin_z: f64,
    bmax_x: f64,
    bmax_y: f64,
    bmax_z: f64,
    axis: u8,
    mut d: f64,
) -> f64 {
    if d.abs() < 1.0e-7 {
        return d;
    }
    let eps = 1.0e-7;
    let ex_min_x = (bmin_x + if axis == 0 { d.min(0.0) } else { 0.0 } - eps).floor() as i32;
    let ex_max_x = (bmax_x + if axis == 0 { d.max(0.0) } else { 0.0 } + eps).floor() as i32;
    let ex_min_y = (bmin_y + if axis == 1 { d.min(0.0) } else { 0.0 } - eps).floor() as i32;
    let ex_max_y = (bmax_y + if axis == 1 { d.max(0.0) } else { 0.0 } + eps).floor() as i32;
    let ex_min_z = (bmin_z + if axis == 2 { d.min(0.0) } else { 0.0 } - eps).floor() as i32;
    let ex_max_z = (bmax_z + if axis == 2 { d.max(0.0) } else { 0.0 } + eps).floor() as i32;

    for cz in ex_min_z..=ex_max_z {
        for cy in ex_min_y..=ex_max_y {
            for cx in ex_min_x..=ex_max_x {
                if !voxels.is_solid(cx, cy, cz) {
                    continue;
                }
                let smin_x = f64::from(cx);
                let smax_x = smin_x + 1.0;
                let smin_y = f64::from(cy);
                let smax_y = smin_y + 1.0;
                let smin_z = f64::from(cz);
                let smax_z = smin_z + 1.0;
                match axis {
                    1 => {
                        if bmax_x - eps > smin_x && bmin_x + eps < smax_x && bmax_z - eps > smin_z && bmin_z + eps < smax_z {
                            if d > 0.0 && bmax_y <= smin_y + eps {
                                let gap = smin_y - bmax_y;
                                if gap < d { d = gap; }
                            } else if d < 0.0 && bmin_y >= smax_y - eps {
                                let gap = smax_y - bmin_y;
                                if gap > d { d = gap; }
                            }
                        }
                    }
                    0 => {
                        if bmax_y - eps > smin_y && bmin_y + eps < smax_y && bmax_z - eps > smin_z && bmin_z + eps < smax_z {
                            if d > 0.0 && bmax_x <= smin_x + eps {
                                let gap = smin_x - bmax_x;
                                if gap < d { d = gap; }
                            } else if d < 0.0 && bmin_x >= smax_x - eps {
                                let gap = smax_x - bmin_x;
                                if gap > d { d = gap; }
                            }
                        }
                    }
                    _ => {
                        if bmax_x - eps > smin_x && bmin_x + eps < smax_x && bmax_y - eps > smin_y && bmin_y + eps < smax_y {
                            if d > 0.0 && bmax_z <= smin_z + eps {
                                let gap = smin_z - bmax_z;
                                if gap < d { d = gap; }
                            } else if d < 0.0 && bmin_z >= smax_z - eps {
                                let gap = smax_z - bmin_z;
                                if gap > d { d = gap; }
                            }
                        }
                    }
                }
                if d.abs() < 1.0e-7 {
                    return 0.0;
                }
            }
        }
    }
    d
}

fn step_one_rollout_cpu(req: &RolloutReq, voxels: &SolidVoxelBox, lut: &[f32]) -> RolloutOut {
    let mut x = req.x;
    let mut y = req.y;
    let mut z = req.z;
    let mut vx = req.vx;
    let mut vy = req.vy;
    let mut vz = req.vz;
    let mut on_ground = req.on_ground != 0;
    let mut no_jump_delay = 0u32;
    let mut horiz_col = false;
    let forward = (req.flags & ROLLOUT_FLAG_FORWARD) != 0;
    let sprint = (req.flags & ROLLOUT_FLAG_SPRINT) != 0;
    let jump = (req.flags & ROLLOUT_FLAG_JUMP) != 0;
    let yaw = req.yaw;
    let rad = (yaw * 0.017453292f32) as f64;
    let sy = sin_from_lut(lut, rad) as f64;
    let cy = cos_from_lut(lut, rad) as f64;
    let steer_z = if forward { 0.98f32 as f64 } else { 0.0 };

    for _ in 0..req.ticks {
        no_jump_delay = no_jump_delay.saturating_sub(1);
        if vx * vx + vz * vz < 9.0e-6 {
            vx = 0.0;
            vz = 0.0;
        }
        if vy.abs() < 0.003 {
            vy = 0.0;
        }

        if jump {
            if on_ground && no_jump_delay == 0 {
                let p = 0.42f32 as f64;
                if p > vy {
                    vy = p;
                }
                if sprint {
                    vx += -sy * 0.2;
                    vz += cy * 0.2;
                }
                no_jump_delay = 10;
            }
        } else {
            no_jump_delay = 0;
        }

        let f = if on_ground { 0.6f32 } else { 1.0f32 };
        let base_spd = if sprint { 0.13f32 } else { 0.1f32 };
        let accel = if !on_ground {
            if sprint { 0.025999999f32 } else { 0.02f32 }
        } else {
            base_spd * (0.21600002f32 / (f * f * f))
        };
        if steer_z != 0.0 {
            let scaled = steer_z * (accel as f64);
            vx += -scaled * sy;
            vz += scaled * cy;
        }

        let wanted_x = vx;
        let wanted_y = vy;
        let wanted_z = vz;
        let bmin_x = x - 0.3;
        let bmin_y = y;
        let bmin_z = z - 0.3;
        let bmax_x = x + 0.3;
        let bmax_y = y + 1.8;
        let bmax_z = z + 0.3;

        let dy = sweep_box_one(voxels, bmin_x, bmin_y, bmin_z, bmax_x, bmax_y, bmax_z, 1, wanted_y);
        let (dx, dz) = if wanted_x.abs() < wanted_z.abs() {
            let dz = sweep_box_one(voxels, bmin_x, bmin_y + dy, bmin_z, bmax_x, bmax_y + dy, bmax_z, 2, wanted_z);
            let dx = sweep_box_one(voxels, bmin_x, bmin_y + dy, bmin_z + dz, bmax_x, bmax_y + dy, bmax_z + dz, 0, wanted_x);
            (dx, dz)
        } else {
            let dx = sweep_box_one(voxels, bmin_x, bmin_y + dy, bmin_z, bmax_x, bmax_y + dy, bmax_z, 0, wanted_x);
            let dz = sweep_box_one(voxels, bmin_x + dx, bmin_y + dy, bmin_z, bmax_x + dx, bmax_y + dy, bmax_z, 2, wanted_z);
            (dx, dz)
        };

        let d2 = dx * dx + dy * dy + dz * dz;
        let w2 = wanted_x * wanted_x + wanted_y * wanted_y + wanted_z * wanted_z;
        if d2 > 1.0e-7 || w2 - d2 < 1.0e-7 {
            x += dx;
            y += dy;
            z += dz;
        }
        let col_x = !math::equal(wanted_x, dx);
        let col_z = !math::equal(wanted_z, dz);
        let col_y = wanted_y != dy;
        on_ground = col_y && wanted_y < 0.0;
        horiz_col |= col_x || col_z;
        if col_x { vx = 0.0; }
        if col_z { vz = 0.0; }
        if wanted_y.abs() > 0.0 && col_y { vy = 0.0; }

        let vy_post = vy - 0.08;
        let h_drag = (f * 0.91f32) as f64;
        vx *= h_drag;
        vz *= h_drag;
        vy = vy_post * (0.98f32 as f64);
    }

    RolloutOut {
        x,
        y,
        z,
        vx,
        vy,
        vz,
        on_ground: u32::from(on_ground),
        horiz_collided: u32::from(horiz_col),
    }
}

pub fn run_rollouts_cpu(
    reqs: &[RolloutReq],
    voxels: &SolidVoxelBox,
    threads: usize,
) -> Vec<RolloutOut> {
    let raw = math::sin_table_bytes();
    let lut: Vec<f32> = raw
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let mut out = vec![RolloutOut::default(); reqs.len()];
    let t_eff = threads.max(1).min(reqs.len().max(1));
    let chunk = reqs.len().div_ceil(t_eff).max(1);
    std::thread::scope(|s| {
        for (req_c, out_c) in reqs.chunks(chunk).zip(out.chunks_mut(chunk)) {
            let lut_ref = &lut;
            s.spawn(move || {
                for (r, o) in req_c.iter().zip(out_c.iter_mut()) {
                    *o = step_one_rollout_cpu(r, voxels, lut_ref);
                }
            });
        }
    });
    out
}

const ROLLOUT_CL: &str = r#"
#pragma OPENCL EXTENSION cl_khr_fp64 : enable

typedef struct __attribute__((packed)) {
    double x;
    double y;
    double z;
    double vx;
    double vy;
    double vz;
    float yaw;
    uint flags;
    uint ticks;
    uint on_ground;
} RolloutReq;

typedef struct __attribute__((packed)) {
    double x;
    double y;
    double z;
    double vx;
    double vy;
    double vz;
    uint on_ground;
    uint horiz_collided;
} RolloutOut;

inline float mc_sin(double rad, __global const float* lut) {
    long idx = (long)(rad * 10430.378350470453) & 65535L;
    return lut[idx];
}

inline float mc_cos(double rad, __global const float* lut) {
    long idx = (long)(rad * 10430.378350470453 + 16384.0) & 65535L;
    return lut[idx];
}

inline int is_solid(
    __global const uint* bits,
    int x, int y, int z,
    int min_x, int min_y, int min_z,
    uint dx, uint dy, uint dz
) {
    int ix = x - min_x;
    int iy = y - min_y;
    int iz = z - min_z;
    if (ix < 0 || iy < 0 || iz < 0 || ix >= (int)dx || iy >= (int)dy || iz >= (int)dz) return 0;
    uint ci = (((uint)iz * dy + (uint)iy) * dx + (uint)ix);
    return (int)((bits[ci >> 5] >> (ci & 31u)) & 1u);
}

inline double sweep_box(
    __global const uint* bits,
    int min_x, int min_y, int min_z,
    uint dx_dim, uint dy_dim, uint dz_dim,
    double bmin_x, double bmin_y, double bmin_z,
    double bmax_x, double bmax_y, double bmax_z,
    int axis, double d
) {
    if (fabs(d) < 1.0e-7) return d;
    const double eps = 1.0e-7;
    int x0 = (int)floor(bmin_x + (axis == 0 ? fmin(d, 0.0) : 0.0) - eps);
    int x1 = (int)floor(bmax_x + (axis == 0 ? fmax(d, 0.0) : 0.0) + eps);
    int y0 = (int)floor(bmin_y + (axis == 1 ? fmin(d, 0.0) : 0.0) - eps);
    int y1 = (int)floor(bmax_y + (axis == 1 ? fmax(d, 0.0) : 0.0) + eps);
    int z0 = (int)floor(bmin_z + (axis == 2 ? fmin(d, 0.0) : 0.0) - eps);
    int z1 = (int)floor(bmax_z + (axis == 2 ? fmax(d, 0.0) : 0.0) + eps);

    for (int cz = z0; cz <= z1; ++cz) {
        for (int cy = y0; cy <= y1; ++cy) {
            for (int cx = x0; cx <= x1; ++cx) {
                if (!is_solid(bits, cx, cy, cz, min_x, min_y, min_z, dx_dim, dy_dim, dz_dim)) continue;
                double smin_x = (double)cx, smax_x = smin_x + 1.0;
                double smin_y = (double)cy, smax_y = smin_y + 1.0;
                double smin_z = (double)cz, smax_z = smin_z + 1.0;
                if (axis == 1) {
                    if (bmax_x - eps > smin_x && bmin_x + eps < smax_x && bmax_z - eps > smin_z && bmin_z + eps < smax_z) {
                        if (d > 0.0 && bmax_y <= smin_y + eps) {
                            double gap = smin_y - bmax_y;
                            if (gap < d) d = gap;
                        } else if (d < 0.0 && bmin_y >= smax_y - eps) {
                            double gap = smax_y - bmin_y;
                            if (gap > d) d = gap;
                        }
                    }
                } else if (axis == 0) {
                    if (bmax_y - eps > smin_y && bmin_y + eps < smax_y && bmax_z - eps > smin_z && bmin_z + eps < smax_z) {
                        if (d > 0.0 && bmax_x <= smin_x + eps) {
                            double gap = smin_x - bmax_x;
                            if (gap < d) d = gap;
                        } else if (d < 0.0 && bmin_x >= smax_x - eps) {
                            double gap = smax_x - bmin_x;
                            if (gap > d) d = gap;
                        }
                    }
                } else {
                    if (bmax_x - eps > smin_x && bmin_x + eps < smax_x && bmax_y - eps > smin_y && bmin_y + eps < smax_y) {
                        if (d > 0.0 && bmax_z <= smin_z + eps) {
                            double gap = smin_z - bmax_z;
                            if (gap < d) d = gap;
                        } else if (d < 0.0 && bmin_z >= smax_z - eps) {
                            double gap = smax_z - bmin_z;
                            if (gap > d) d = gap;
                        }
                    }
                }
                if (fabs(d) < 1.0e-7) return 0.0;
            }
        }
    }
    return d;
}

__kernel void batched_rollout_kernel(
    __global const RolloutReq* reqs,
    __global RolloutOut* outs,
    __global const float* sin_lut,
    __global const uint* solid_bits,
    const uint count,
    const int min_x,
    const int min_y,
    const int min_z,
    const uint dx_dim,
    const uint dy_dim,
    const uint dz_dim
) {
    uint gid = (uint)get_global_id(0);
    if (gid >= count) return;

    RolloutReq r = reqs[gid];
    double x = r.x;
    double y = r.y;
    double z = r.z;
    double vx = r.vx;
    double vy = r.vy;
    double vz = r.vz;
    uint on_ground = r.on_ground;
    uint no_jump_delay = 0u;
    uint horiz_col = 0u;

    uint forward = r.flags & 1u;
    uint sprint = (r.flags >> 1) & 1u;
    uint jump = (r.flags >> 2) & 1u;

    double rad = (double)(r.yaw * 0.017453292f);
    double sy = (double)mc_sin(rad, sin_lut);
    double cy = (double)mc_cos(rad, sin_lut);
    double steer_z = forward ? (double)0.98f : 0.0;
    const double eq_eps = (double)1.0e-5f;

    for (uint t = 0u; t < r.ticks; ++t) {
        if (no_jump_delay > 0u) no_jump_delay -= 1u;
        if (vx * vx + vz * vz < 9.0e-6) { vx = 0.0; vz = 0.0; }
        if (fabs(vy) < 0.003) vy = 0.0;

        if (jump) {
            if (on_ground && no_jump_delay == 0u) {
                double p = (double)0.42f;
                if (p > vy) vy = p;
                if (sprint) {
                    vx += -sy * 0.2;
                    vz += cy * 0.2;
                }
                no_jump_delay = 10u;
            }
        } else {
            no_jump_delay = 0u;
        }

        float f = on_ground ? 0.6f : 1.0f;
        float base_spd = sprint ? 0.13f : 0.1f;
        float accel = !on_ground ? (sprint ? 0.025999999f : 0.02f) : (base_spd * (0.21600002f / (f * f * f)));
        if (steer_z != 0.0) {
            double scaled = steer_z * (double)accel;
            vx += -scaled * sy;
            vz += scaled * cy;
        }

        double wx = vx, wy = vy, wz = vz;
        double bmin_x = x - 0.3, bmin_y = y, bmin_z = z - 0.3;
        double bmax_x = x + 0.3, bmax_y = y + 1.8, bmax_z = z + 0.3;

        double dy = sweep_box(solid_bits, min_x, min_y, min_z, dx_dim, dy_dim, dz_dim, bmin_x, bmin_y, bmin_z, bmax_x, bmax_y, bmax_z, 1, wy);
        double dx, dz;
        if (fabs(wx) < fabs(wz)) {
            dz = sweep_box(solid_bits, min_x, min_y, min_z, dx_dim, dy_dim, dz_dim, bmin_x, bmin_y + dy, bmin_z, bmax_x, bmax_y + dy, bmax_z, 2, wz);
            dx = sweep_box(solid_bits, min_x, min_y, min_z, dx_dim, dy_dim, dz_dim, bmin_x, bmin_y + dy, bmin_z + dz, bmax_x, bmax_y + dy, bmax_z + dz, 0, wx);
        } else {
            dx = sweep_box(solid_bits, min_x, min_y, min_z, dx_dim, dy_dim, dz_dim, bmin_x, bmin_y + dy, bmin_z, bmax_x, bmax_y + dy, bmax_z, 0, wx);
            dz = sweep_box(solid_bits, min_x, min_y, min_z, dx_dim, dy_dim, dz_dim, bmin_x + dx, bmin_y + dy, bmin_z, bmax_x + dx, bmax_y + dy, bmax_z, 2, wz);
        }

        double d2 = dx * dx + dy * dy + dz * dz;
        double w2 = wx * wx + wy * wy + wz * wz;
        if (d2 > 1.0e-7 || w2 - d2 < 1.0e-7) {
            x += dx;
            y += dy;
            z += dz;
        }
        uint col_x = fabs(dx - wx) >= eq_eps ? 1u : 0u;
        uint col_z = fabs(dz - wz) >= eq_eps ? 1u : 0u;
        uint col_y = (wy != dy) ? 1u : 0u;
        on_ground = (col_y && wy < 0.0) ? 1u : 0u;
        horiz_col |= (col_x | col_z);
        if (col_x) vx = 0.0;
        if (col_z) vz = 0.0;
        if (fabs(wy) > 0.0 && col_y) vy = 0.0;

        double vy_post = vy - 0.08;
        double h_drag = (double)(f * 0.91f);
        vx *= h_drag;
        vz *= h_drag;
        vy = vy_post * (double)0.98f;
    }

    RolloutOut o;
    o.x = x; o.y = y; o.z = z;
    o.vx = vx; o.vy = vy; o.vz = vz;
    o.on_ground = on_ground;
    o.horiz_collided = horiz_col;
    outs[gid] = o;
}
"#;

pub struct RolloutGpuProgram {
    dev: GpuDevice,
    _prog: GpuProgram,
    sin_lut: GpuBuffer,
    kern: Mutex<GpuKernel>,
}

impl RolloutGpuProgram {
    pub fn compile(dev: &GpuDevice) -> Result<Self, String> {
        let prog = dev.build_program(ROLLOUT_CL)?;
        let kern = prog.kernel("batched_rollout_kernel")?;
        let sin_bytes = math::sin_table_bytes();
        let sin_lut = dev.alloc_ro_slice(sin_bytes)?;
        Ok(Self {
            dev: dev.clone(),
            _prog: prog,
            sin_lut,
            kern: Mutex::new(kern),
        })
    }

    pub fn run_batch(
        &self,
        reqs: &[RolloutReq],
        voxels: &SolidVoxelBox,
    ) -> Result<(Vec<RolloutOut>, f64), String> {
        let t0 = Instant::now();
        if reqs.is_empty() {
            return Ok((Vec::new(), 0.0));
        }
        let buf_reqs = self.dev.alloc_ro_slice(reqs)?;
        let buf_outs = self.dev.alloc_rw(reqs.len() * std::mem::size_of::<RolloutOut>())?;
        let buf_bits = self.dev.alloc_ro_slice(&voxels.bits)?;

        let count_u32 = reqs.len() as u32;
        let dx_u32 = voxels.dx as u32;
        let dy_u32 = voxels.dy as u32;
        let dz_u32 = voxels.dz as u32;

        {
            let mut k = self.kern.lock().unwrap();
            k.set_buf(0, &buf_reqs)?;
            k.set_buf(1, &buf_outs)?;
            k.set_buf(2, &self.sin_lut)?;
            k.set_buf(3, &buf_bits)?;
            k.set_val(4, &count_u32)?;
            k.set_val(5, &voxels.min[0])?;
            k.set_val(6, &voxels.min[1])?;
            k.set_val(7, &voxels.min[2])?;
            k.set_val(8, &dx_u32)?;
            k.set_val(9, &dy_u32)?;
            k.set_val(10, &dz_u32)?;
            k.enqueue_1d(reqs.len(), Some(64))?;
        }

        let mut outs = vec![RolloutOut::default(); reqs.len()];
        buf_outs.read_slice(&mut outs)?;
        Ok((outs, t0.elapsed().as_secs_f64() * 1e3))
    }
}
