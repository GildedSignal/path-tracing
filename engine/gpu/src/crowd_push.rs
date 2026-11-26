//! Dense `Knudsen ≈ 1` `Entity.push` (`push_pair`) crowd separation on OpenCL (`fp64`) + exact CPU `law::entity_contact::push_pair` reference.
use std::{sync::Mutex, time::Instant};

use law::{Vec3, entity_contact::{ContactEntity, push_pair}};

use crate::opencl::{GpuDevice, GpuKernel, GpuProgram};

#[derive(Clone)]
pub struct CrowdState {
    pub pos_x: Vec<f64>,
    pub pos_y: Vec<f64>,
    pub pos_z: Vec<f64>,
    pub vel_x: Vec<f64>,
    pub vel_z: Vec<f64>,
}

impl CrowdState {
    pub fn len(&self) -> usize {
        self.pos_x.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pos_x.is_empty()
    }

    /// Deterministic dense `Knudsen ≈ 1` arena with high AABB overlap density.
    pub fn dense_arena(count: usize, width: f64) -> Self {
        let mut pos_x = Vec::with_capacity(count);
        let mut pos_y = Vec::with_capacity(count);
        let mut pos_z = Vec::with_capacity(count);
        let mut vel_x = Vec::with_capacity(count);
        let mut vel_z = Vec::with_capacity(count);
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut next_u01 = || -> f64 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 11) as f64) / ((1u64 << 53) as f64)
        };
        for _ in 0..count {
            pos_x.push(next_u01() * width);
            pos_y.push(0.0);
            pos_z.push(next_u01() * width);
            vel_x.push((next_u01() - 0.5) * 0.08);
            vel_z.push((next_u01() - 0.5) * 0.08);
        }
        Self {
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_z,
        }
    }
}

/// Exact CPU spatial-grid `Entity.push` using `law::entity_contact::push_pair`.
pub fn step_crowd_cpu(
    state: &mut CrowdState,
    origin_x: f64,
    origin_z: f64,
    grid_w: usize,
    grid_h: usize,
    ticks: usize,
    threads: usize,
) {
    let n = state.len();
    if n == 0 || ticks == 0 {
        return;
    }
    let num_cells = grid_w * grid_h;
    let mut counts = vec![0usize; num_cells];
    let mut offsets = vec![0usize; num_cells + 1];
    let mut cursors = vec![0usize; num_cells];
    let mut sorted = vec![0usize; n];
    let mut out_vx = vec![0.0f64; n];
    let mut out_vz = vec![0.0f64; n];
    let t_eff = threads.max(1);

    let cell_of = |x: f64, z: f64| -> (i32, i32) {
        let cx = ((x - origin_x).floor() as i32).clamp(0, grid_w as i32 - 1);
        let cz = ((z - origin_z).floor() as i32).clamp(0, grid_h as i32 - 1);
        (cx, cz)
    };

    for _ in 0..ticks {
        counts.fill(0);
        for i in 0..n {
            let (cx, cz) = cell_of(state.pos_x[i], state.pos_z[i]);
            counts[cz as usize * grid_w + cx as usize] += 1;
        }
        offsets[0] = 0;
        for c in 0..num_cells {
            offsets[c + 1] = offsets[c] + counts[c];
        }
        cursors.copy_from_slice(&offsets[..num_cells]);
        for i in 0..n {
            let (cx, cz) = cell_of(state.pos_x[i], state.pos_z[i]);
            let c = cz as usize * grid_w + cx as usize;
            let slot = cursors[c];
            cursors[c] += 1;
            sorted[slot] = i;
        }

        if t_eff == 1 {
            apply_push_range(
                0..n,
                state,
                origin_x,
                origin_z,
                grid_w,
                grid_h,
                &offsets,
                &sorted,
                &mut out_vx,
                &mut out_vz,
            );
        } else {
            let chunk = n.div_ceil(t_eff).max(1);
            std::thread::scope(|s| {
                for (ti, (vx_chunk, vz_chunk)) in out_vx
                    .chunks_mut(chunk)
                    .zip(out_vz.chunks_mut(chunk))
                    .enumerate()
                {
                    let start = ti * chunk;
                    let end = (start + vx_chunk.len()).min(n);
                    let st_ref = &*state;
                    let off_ref = &offsets;
                    let srt_ref = &sorted;
                    s.spawn(move || {
                        for local_i in 0..(end - start) {
                            let i = start + local_i;
                            let (nvx, nvz) = eval_one_push(
                                i,
                                st_ref,
                                origin_x,
                                origin_z,
                                grid_w,
                                grid_h,
                                off_ref,
                                srt_ref,
                            );
                            vx_chunk[local_i] = nvx;
                            vz_chunk[local_i] = nvz;
                        }
                    });
                }
            });
        }

        let drag = (0.91f32 * 0.6f32) as f64;
        let max_x = origin_x + grid_w as f64;
        let max_z = origin_z + grid_h as f64;
        for i in 0..n {
            let vx = out_vx[i];
            let vz = out_vz[i];
            state.pos_x[i] = (state.pos_x[i] + vx).clamp(origin_x, max_x);
            state.pos_z[i] = (state.pos_z[i] + vz).clamp(origin_z, max_z);
            state.vel_x[i] = vx * drag;
            state.vel_z[i] = vz * drag;
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn eval_one_push(
    i: usize,
    state: &CrowdState,
    origin_x: f64,
    origin_z: f64,
    grid_w: usize,
    grid_h: usize,
    offsets: &[usize],
    sorted: &[usize],
) -> (f64, f64) {
    let px = state.pos_x[i];
    let py = state.pos_y[i];
    let pz = state.pos_z[i];
    let mut me = ContactEntity::player(Vec3::new(px, py, pz), Vec3::new(state.vel_x[i], 0.0, state.vel_z[i]));
    let cx = ((px - origin_x).floor() as i32).clamp(0, grid_w as i32 - 1);
    let cz = ((pz - origin_z).floor() as i32).clamp(0, grid_h as i32 - 1);

    for nz in (cz - 1).max(0)..=(cz + 1).min(grid_h as i32 - 1) {
        for nx in (cx - 1).max(0)..=(cx + 1).min(grid_w as i32 - 1) {
            let c = nz as usize * grid_w + nx as usize;
            for &j in &sorted[offsets[c]..offsets[c + 1]] {
                if j == i {
                    continue;
                }
                let ox = state.pos_x[j];
                let oy = state.pos_y[j];
                let oz = state.pos_z[j];
                if (ox - px).abs() < 0.6 && (oz - pz).abs() < 0.6 && oy < py + 1.8 && py < oy + 1.8 {
                    let mut other = ContactEntity::player(Vec3::new(ox, oy, oz), Vec3::default());
                    push_pair(&mut me, &mut other, false);
                }
            }
        }
    }
    (me.velocity.x, me.velocity.z)
}

#[allow(clippy::too_many_arguments)]
fn apply_push_range(
    range: std::ops::Range<usize>,
    state: &CrowdState,
    origin_x: f64,
    origin_z: f64,
    grid_w: usize,
    grid_h: usize,
    offsets: &[usize],
    sorted: &[usize],
    out_vx: &mut [f64],
    out_vz: &mut [f64],
) {
    for i in range {
        let (nvx, nvz) = eval_one_push(i, state, origin_x, origin_z, grid_w, grid_h, offsets, sorted);
        out_vx[i] = nvx;
        out_vz[i] = nvz;
    }
}

const CROWD_PUSH_CL: &str = r#"
#pragma OPENCL EXTENSION cl_khr_fp64 : enable

__kernel void grid_clear(__global uint* counts, const uint num_cells) {
    uint gid = (uint)get_global_id(0);
    if (gid < num_cells) counts[gid] = 0u;
}

__kernel void grid_count(
    __global const double* pos_x,
    __global const double* pos_z,
    __global uint* counts,
    const uint n,
    const double origin_x,
    const double origin_z,
    const uint grid_w,
    const uint grid_h
) {
    uint gid = (uint)get_global_id(0);
    if (gid >= n) return;
    int cx = clamp((int)floor(pos_x[gid] - origin_x), 0, (int)grid_w - 1);
    int cz = clamp((int)floor(pos_z[gid] - origin_z), 0, (int)grid_h - 1);
    atomic_inc(&counts[(uint)cz * grid_w + (uint)cx]);
}

__kernel void grid_prefix_sum(
    __global const uint* counts,
    __global uint* offsets,
    __global uint* cursors,
    const uint num_cells
) {
    if (get_global_id(0) != 0) return;
    uint acc = 0u;
    for (uint i = 0u; i < num_cells; ++i) {
        offsets[i] = acc;
        cursors[i] = acc;
        acc += counts[i];
    }
    offsets[num_cells] = acc;
}

__kernel void grid_scatter(
    __global const double* pos_x,
    __global const double* pos_z,
    __global uint* cursors,
    __global uint* sorted_ids,
    const uint n,
    const double origin_x,
    const double origin_z,
    const uint grid_w,
    const uint grid_h
) {
    uint gid = (uint)get_global_id(0);
    if (gid >= n) return;
    int cx = clamp((int)floor(pos_x[gid] - origin_x), 0, (int)grid_w - 1);
    int cz = clamp((int)floor(pos_z[gid] - origin_z), 0, (int)grid_h - 1);
    uint slot = atomic_inc(&cursors[(uint)cz * grid_w + (uint)cx]);
    sorted_ids[slot] = gid;
}

__kernel void crowd_push_step(
    __global const double* pos_x,
    __global const double* pos_y,
    __global const double* pos_z,
    __global const double* vel_x,
    __global const double* vel_z,
    __global const uint* offsets,
    __global const uint* sorted_ids,
    __global double* out_vx,
    __global double* out_vz,
    const uint n,
    const double origin_x,
    const double origin_z,
    const uint grid_w,
    const uint grid_h
) {
    uint i = (uint)get_global_id(0);
    if (i >= n) return;

    double px = pos_x[i];
    double py = pos_y[i];
    double pz = pos_z[i];
    int cx = clamp((int)floor(px - origin_x), 0, (int)grid_w - 1);
    int cz = clamp((int)floor(pz - origin_z), 0, (int)grid_h - 1);

    double dvx = 0.0;
    double dvz = 0.0;
    const double thresh = (double)0.01f;
    const double force = (double)0.05f;

    int z0 = max(cz - 1, 0);
    int z1 = min(cz + 1, (int)grid_h - 1);
    int x0 = max(cx - 1, 0);
    int x1 = min(cx + 1, (int)grid_w - 1);

    for (int nz = z0; nz <= z1; ++nz) {
        for (int nx = x0; nx <= x1; ++nx) {
            uint cell = (uint)nz * grid_w + (uint)nx;
            uint s0 = offsets[cell];
            uint s1 = offsets[cell + 1u];
            for (uint s = s0; s < s1; ++s) {
                uint j = sorted_ids[s];
                if (j == i) continue;
                double ox = pos_x[j];
                double oy = pos_y[j];
                double oz = pos_z[j];
                if (fabs(ox - px) < 0.6 && fabs(oz - pz) < 0.6 && oy < py + 1.8 && py < oy + 1.8) {
                    double dx = ox - px;
                    double dz = oz - pz;
                    double d = fmax(fabs(dx), fabs(dz));
                    if (d < thresh) continue;
                    d = sqrt(d);
                    dx /= d;
                    dz /= d;
                    double scale = 1.0 / d;
                    if (scale > 1.0) scale = 1.0;
                    dx *= scale * force;
                    dz *= scale * force;
                    if (isfinite(dx) && isfinite(dz)) {
                        dvx -= dx;
                        dvz -= dz;
                    }
                }
            }
        }
    }

    out_vx[i] = vel_x[i] + dvx;
    out_vz[i] = vel_z[i] + dvz;
}

__kernel void crowd_advance(
    __global double* pos_x,
    __global double* pos_z,
    __global double* vel_x,
    __global double* vel_z,
    __global const double* out_vx,
    __global const double* out_vz,
    const uint n,
    const double min_x,
    const double min_z,
    const double max_x,
    const double max_z,
    const double drag
) {
    uint i = (uint)get_global_id(0);
    if (i >= n) return;
    double vx = out_vx[i];
    double vz = out_vz[i];
    double x = pos_x[i] + vx;
    double z = pos_z[i] + vz;
    pos_x[i] = clamp(x, min_x, max_x);
    pos_z[i] = clamp(z, min_z, max_z);
    vel_x[i] = vx * drag;
    vel_z[i] = vz * drag;
}
"#;

struct CrowdKernels {
    grid_clear: GpuKernel,
    grid_count: GpuKernel,
    grid_prefix_sum: GpuKernel,
    grid_scatter: GpuKernel,
    crowd_push_step: GpuKernel,
    crowd_advance: GpuKernel,
}

pub struct CrowdPushGpuProgram {
    dev: GpuDevice,
    _prog: GpuProgram,
    kernels: Mutex<CrowdKernels>,
}

impl CrowdPushGpuProgram {
    pub fn compile(dev: &GpuDevice) -> Result<Self, String> {
        let prog = dev.build_program(CROWD_PUSH_CL)?;
        let kernels = CrowdKernels {
            grid_clear: prog.kernel("grid_clear")?,
            grid_count: prog.kernel("grid_count")?,
            grid_prefix_sum: prog.kernel("grid_prefix_sum")?,
            grid_scatter: prog.kernel("grid_scatter")?,
            crowd_push_step: prog.kernel("crowd_push_step")?,
            crowd_advance: prog.kernel("crowd_advance")?,
        };
        Ok(Self {
            dev: dev.clone(),
            _prog: prog,
            kernels: Mutex::new(kernels),
        })
    }

    pub fn step_crowd(
        &self,
        state: &mut CrowdState,
        origin_x: f64,
        origin_z: f64,
        grid_w: usize,
        grid_h: usize,
        ticks: usize,
    ) -> Result<f64, String> {
        let t0 = Instant::now();
        let n = state.len();
        if n == 0 || ticks == 0 {
            return Ok(0.0);
        }
        let num_cells = grid_w * grid_h;
        let n_u32 = n as u32;
        let gw_u32 = grid_w as u32;
        let gh_u32 = grid_h as u32;
        let num_cells_u32 = num_cells as u32;
        let max_x = origin_x + grid_w as f64;
        let max_z = origin_z + grid_h as f64;
        let drag = (0.91f32 * 0.6f32) as f64;

        let buf_px = self.dev.alloc_rw_slice(&state.pos_x)?;
        let buf_py = self.dev.alloc_ro_slice(&state.pos_y)?;
        let buf_pz = self.dev.alloc_rw_slice(&state.pos_z)?;
        let buf_vx = self.dev.alloc_rw_slice(&state.vel_x)?;
        let buf_vz = self.dev.alloc_rw_slice(&state.vel_z)?;
        let buf_out_vx = self.dev.alloc_rw(n * std::mem::size_of::<f64>())?;
        let buf_out_vz = self.dev.alloc_rw(n * std::mem::size_of::<f64>())?;
        let buf_counts = self.dev.alloc_rw(num_cells * std::mem::size_of::<u32>())?;
        let buf_offsets = self.dev.alloc_rw((num_cells + 1) * std::mem::size_of::<u32>())?;
        let buf_cursors = self.dev.alloc_rw(num_cells * std::mem::size_of::<u32>())?;
        let buf_sorted = self.dev.alloc_rw(n * std::mem::size_of::<u32>())?;

        let mut k = self.kernels.lock().unwrap();
        k.grid_clear.set_buf(0, &buf_counts)?;
        k.grid_clear.set_val(1, &num_cells_u32)?;

        k.grid_count.set_buf(0, &buf_px)?;
        k.grid_count.set_buf(1, &buf_pz)?;
        k.grid_count.set_buf(2, &buf_counts)?;
        k.grid_count.set_val(3, &n_u32)?;
        k.grid_count.set_val(4, &origin_x)?;
        k.grid_count.set_val(5, &origin_z)?;
        k.grid_count.set_val(6, &gw_u32)?;
        k.grid_count.set_val(7, &gh_u32)?;

        k.grid_prefix_sum.set_buf(0, &buf_counts)?;
        k.grid_prefix_sum.set_buf(1, &buf_offsets)?;
        k.grid_prefix_sum.set_buf(2, &buf_cursors)?;
        k.grid_prefix_sum.set_val(3, &num_cells_u32)?;

        k.grid_scatter.set_buf(0, &buf_px)?;
        k.grid_scatter.set_buf(1, &buf_pz)?;
        k.grid_scatter.set_buf(2, &buf_cursors)?;
        k.grid_scatter.set_buf(3, &buf_sorted)?;
        k.grid_scatter.set_val(4, &n_u32)?;
        k.grid_scatter.set_val(5, &origin_x)?;
        k.grid_scatter.set_val(6, &origin_z)?;
        k.grid_scatter.set_val(7, &gw_u32)?;
        k.grid_scatter.set_val(8, &gh_u32)?;

        k.crowd_push_step.set_buf(0, &buf_px)?;
        k.crowd_push_step.set_buf(1, &buf_py)?;
        k.crowd_push_step.set_buf(2, &buf_pz)?;
        k.crowd_push_step.set_buf(3, &buf_vx)?;
        k.crowd_push_step.set_buf(4, &buf_vz)?;
        k.crowd_push_step.set_buf(5, &buf_offsets)?;
        k.crowd_push_step.set_buf(6, &buf_sorted)?;
        k.crowd_push_step.set_buf(7, &buf_out_vx)?;
        k.crowd_push_step.set_buf(8, &buf_out_vz)?;
        k.crowd_push_step.set_val(9, &n_u32)?;
        k.crowd_push_step.set_val(10, &origin_x)?;
        k.crowd_push_step.set_val(11, &origin_z)?;
        k.crowd_push_step.set_val(12, &gw_u32)?;
        k.crowd_push_step.set_val(13, &gh_u32)?;

        k.crowd_advance.set_buf(0, &buf_px)?;
        k.crowd_advance.set_buf(1, &buf_pz)?;
        k.crowd_advance.set_buf(2, &buf_vx)?;
        k.crowd_advance.set_buf(3, &buf_vz)?;
        k.crowd_advance.set_buf(4, &buf_out_vx)?;
        k.crowd_advance.set_buf(5, &buf_out_vz)?;
        k.crowd_advance.set_val(6, &n_u32)?;
        k.crowd_advance.set_val(7, &origin_x)?;
        k.crowd_advance.set_val(8, &origin_z)?;
        k.crowd_advance.set_val(9, &max_x)?;
        k.crowd_advance.set_val(10, &max_z)?;
        k.crowd_advance.set_val(11, &drag)?;

        for _ in 0..ticks {
            k.grid_clear.enqueue_1d(num_cells, Some(64))?;
            k.grid_count.enqueue_1d(n, Some(64))?;
            k.grid_prefix_sum.enqueue_1d(1, Some(1))?;
            k.grid_scatter.enqueue_1d(n, Some(64))?;
            k.crowd_push_step.enqueue_1d(n, Some(64))?;
            k.crowd_advance.enqueue_1d(n, Some(64))?;
        }
        drop(k);

        buf_px.read_slice(&mut state.pos_x)?;
        buf_pz.read_slice(&mut state.pos_z)?;
        buf_vx.read_slice(&mut state.vel_x)?;
        buf_vz.read_slice(&mut state.vel_z)?;
        Ok(t0.elapsed().as_secs_f64() * 1e3)
    }
}
