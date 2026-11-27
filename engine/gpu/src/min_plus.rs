//! Batched 3D `(min, +)` `TopoField` wavefront solver on OpenCL + exact CPU `BinaryHeap` reference.
use std::{cmp::Reverse, collections::BinaryHeap, time::Instant};

use law::{BlockPos, CollisionContext, FluidKind, World};

use crate::opencl::{GpuBuffer, GpuDevice, GpuKernel, GpuProgram};

/// Matches `engine/plan/src/defect.rs` `DIRS` order bit-for-bit.
pub const DIRS: [BlockPos; 10] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 0, 1],
    [0, 0, -1],
    [0, 1, 0],
    [0, -1, 0],
    [1, 0, 1],
    [1, 0, -1],
    [-1, 0, 1],
    [-1, 0, -1],
];

pub const FLAG_STEP_SUP: u16 = 1 << 10;
pub const FLAG_BODY_FITS: u16 = 1 << 11;

#[derive(Clone, Copy, Debug)]
pub struct LegsCost {
    pub h: f32,
    pub up: f32,
    pub down: f32,
    pub reach: u8,
}

impl Default for LegsCost {
    fn default() -> Self {
        Self {
            h: 2.81,
            up: 6.19,
            down: 0.0,
            reach: 4,
        }
    }
}

#[derive(Clone)]
pub struct PackedTopoGrid {
    pub min: BlockPos,
    pub dx: usize,
    pub dy: usize,
    pub dz: usize,
    /// Per cell: bits `0..10` open edge `(n -> n - DIRS[k])`, bit 10 `has_step_support`, bit 11 `body_fits`.
    pub flags: Vec<u16>,
}

impl PackedTopoGrid {
    #[inline]
    pub fn vol(&self) -> usize {
        self.dx * self.dy * self.dz
    }

    #[inline]
    pub fn idx(&self, p: BlockPos) -> Option<usize> {
        let ix = p[0] - self.min[0];
        let iy = p[1] - self.min[1];
        let iz = p[2] - self.min[2];
        if ix < 0
            || iy < 0
            || iz < 0
            || ix >= self.dx as i32
            || iy >= self.dy as i32
            || iz >= self.dz as i32
        {
            return None;
        }
        Some(((iz as usize) * self.dy + (iy as usize)) * self.dx + (ix as usize))
    }

    #[inline]
    pub fn pos(&self, ci: usize) -> BlockPos {
        let ix = ci % self.dx;
        let iy = (ci / self.dx) % self.dy;
        let iz = ci / (self.dx * self.dy);
        [
            self.min[0] + ix as i32,
            self.min[1] + iy as i32,
            self.min[2] + iz as i32,
        ]
    }

    pub fn from_world(world: &impl World, min: BlockPos, max: BlockPos) -> Self {
        let dx = (max[0] - min[0] + 1).max(1) as usize;
        let dy = (max[1] - min[1] + 1).max(1) as usize;
        let dz = (max[2] - min[2] + 1).max(1) as usize;
        let vol = dx * dy * dz;
        let mut flags = vec![0u16; vol];

        let way_clear = |a: BlockPos, b: BlockPos| -> bool {
            let lo = [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])];
            let hi = [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])];
            let hull = law::Aabb::new(
                f64::from(lo[0]) + 0.25,
                f64::from(lo[1]) + 0.05,
                f64::from(lo[2]) + 0.25,
                f64::from(hi[0]) + 0.75,
                f64::from(hi[1]) + 1.55,
                f64::from(hi[2]) + 0.75,
            );
            world.no_collision(
                hull,
                CollisionContext {
                    feet_y: f64::from(lo[1]),
                    descending: false,
                    fall_distance: 0.0,
                    leather_boots: false,
                },
            )
        };

        let has_sup = |p: BlockPos| -> bool {
            if !world.loaded(p) {
                return false;
            }
            let b = world.block(p);
            if !b.never_collides() || b.climbable || b.fluid.kind != FluidKind::Empty {
                return true;
            }
            let bp = [p[0], p[1] - 1, p[2]];
            if !world.loaded(bp) {
                return false;
            }
            let bb = world.block(bp);
            !bb.never_collides() || bb.climbable
        };

        let smin = min.map(|v| v.div_euclid(16));
        let smax = max.map(|v| v.div_euclid(16));
        for sz in smin[2]..=smax[2] {
            for sy in smin[1]..=smax[1] {
                for sx in smin[0]..=smax[0] {
                    let sec_lo = [
                        min[0].max(sx * 16),
                        min[1].max(sy * 16),
                        min[2].max(sz * 16),
                    ];
                    let sec_hi = [
                        max[0].min(sx * 16 + 15),
                        max[1].min(sy * 16 + 15),
                        max[2].min(sz * 16 + 15),
                    ];
                    let halo_lo = [sec_lo[0] - 2, sec_lo[1] - 2, sec_lo[2] - 2];
                    let halo_hi = [sec_hi[0] + 2, sec_hi[1] + 3, sec_hi[2] + 2];
                    if let Some(b) = world.uniform(halo_lo, halo_hi) {
                        if b.never_collides() && !b.climbable && b.fluid.kind == FluidKind::Empty {
                            for z in sec_lo[2]..=sec_hi[2] {
                                let iz = (z - min[2]) as usize;
                                let y_base = iz * dy;
                                for y in sec_lo[1]..=sec_hi[1] {
                                    let iy = (y - min[1]) as usize;
                                    let row_base = (y_base + iy) * dx;
                                    for x in sec_lo[0]..=sec_hi[0] {
                                        let ix = (x - min[0]) as usize;
                                        let mut m = FLAG_BODY_FITS;
                                        for (k, d) in DIRS.iter().enumerate() {
                                            let cur = [x - d[0], y - d[1], z - d[2]];
                                            if cur[0] >= min[0]
                                                && cur[0] <= max[0]
                                                && cur[1] >= min[1]
                                                && cur[1] <= max[1]
                                                && cur[2] >= min[2]
                                                && cur[2] <= max[2]
                                            {
                                                m |= 1u16 << k;
                                            }
                                        }
                                        flags[row_base + ix] = m;
                                    }
                                }
                            }
                            continue;
                        }
                        if !b.never_collides()
                            && b.inside != law::Inside::PowderSnow
                            && b.collision_shapes().all(|s| s.is_block())
                        {
                            for z in sec_lo[2]..=sec_hi[2] {
                                let iz = (z - min[2]) as usize;
                                let y_base = iz * dy;
                                for y in sec_lo[1]..=sec_hi[1] {
                                    let iy = (y - min[1]) as usize;
                                    let row_base = (y_base + iy) * dx;
                                    for x in sec_lo[0]..=sec_hi[0] {
                                        let ix = (x - min[0]) as usize;
                                        flags[row_base + ix] = FLAG_STEP_SUP;
                                    }
                                }
                            }
                            continue;
                        }
                    }

                    for z in sec_lo[2]..=sec_hi[2] {
                        let iz = (z - min[2]) as usize;
                        for y in sec_lo[1]..=sec_hi[1] {
                            let iy = (y - min[1]) as usize;
                            for x in sec_lo[0]..=sec_hi[0] {
                                let ix = (x - min[0]) as usize;
                                let n = [x, y, z];
                                let ci = (iz * dy + iy) * dx + ix;
                                if !world.loaded(n) {
                                    continue;
                                }
                                let mut m = 0u16;
                                if has_sup(n) {
                                    m |= FLAG_STEP_SUP;
                                }
                                let fits = way_clear(n, n);
                                if fits {
                                    m |= FLAG_BODY_FITS;
                                }
                                for (k, d) in DIRS.iter().enumerate() {
                                    if !fits && d[1] == 0 {
                                        continue;
                                    }
                                    let cur = [n[0] - d[0], n[1] - d[1], n[2] - d[2]];
                                    if cur[0] >= min[0]
                                        && cur[0] <= max[0]
                                        && cur[1] >= min[1]
                                        && cur[1] <= max[1]
                                        && cur[2] >= min[2]
                                        && cur[2] <= max[2]
                                        && way_clear(n, cur)
                                    {
                                        m |= 1u16 << k;
                                    }
                                }
                                flags[ci] = m;
                            }
                        }
                    }
                }
            }
        }

        Self {
            min,
            dx,
            dy,
            dz,
            flags,
        }
    }

    /// Deterministic synthetic obstacle arena (solid floor at `iy = 0`, air walls/bridges/gaps above)
    /// for offline benchmarking and CPU-vs-GPU parity verification.
    pub fn synthetic_arena(dx: usize, dy: usize, dz: usize) -> Self {
        let mut solid = vec![false; dx * dy * dz];
        let at = |x: usize, y: usize, z: usize| (z * dy + y) * dx + x;
        for z in 0..dz {
            for x in 0..dx {
                solid[at(x, 0, z)] = true;
                // Carve two parallel moat gaps with narrow stone bridges.
                if (x == dx / 3 && !(z == dz / 4 || z == (3 * dz) / 4))
                    || (x == (2 * dx) / 3 && z != dz / 2)
                {
                    solid[at(x, 0, z)] = false;
                }
                // Stepped wall barriers with staggered gates on iy = 1..=2.
                if dy >= 4 {
                    if (x == dx / 2 && z % 11 != 5) || (z == dz / 2 && x % 13 != 6) {
                        solid[at(x, 1, z)] = true;
                        solid[at(x, 2, z)] = true;
                    }
                    // Raised platform near quadrant center.
                    if (x >= dx / 6 && x < dx / 6 + 6) && (z >= dz / 6 && z < dz / 6 + 6) {
                        solid[at(x, 1, z)] = true;
                    }
                }
            }
        }
        let is_solid = |x: i32, y: i32, z: i32| -> bool {
            if x < 0 || z < 0 || x >= dx as i32 || z >= dz as i32 {
                return false;
            }
            if y < 0 {
                return true;
            }
            if y >= dy as i32 {
                return false;
            }
            solid[at(x as usize, y as usize, z as usize)]
        };
        let fits = |x: i32, y: i32, z: i32| !is_solid(x, y, z) && !is_solid(x, y + 1, z);
        let way = |a: [i32; 3], b: [i32; 3]| -> bool {
            for z in a[2].min(b[2])..=a[2].max(b[2]) {
                for y in a[1].min(b[1])..=a[1].max(b[1]) {
                    for x in a[0].min(b[0])..=a[0].max(b[0]) {
                        if !fits(x, y, z) {
                            return false;
                        }
                    }
                }
            }
            true
        };
        let mut flags = vec![0u16; dx * dy * dz];
        for z in 0..dz as i32 {
            for y in 0..dy as i32 {
                for x in 0..dx as i32 {
                    let n = [x, y, z];
                    let ci = at(x as usize, y as usize, z as usize);
                    let mut m = 0u16;
                    if is_solid(x, y - 1, z) {
                        m |= FLAG_STEP_SUP;
                    }
                    if fits(x, y, z) {
                        m |= FLAG_BODY_FITS;
                    }
                    for (k, d) in DIRS.iter().enumerate() {
                        let cur = [x - d[0], y - d[1], z - d[2]];
                        if cur[0] >= 0
                            && cur[0] < dx as i32
                            && cur[1] >= 0
                            && cur[1] < dy as i32
                            && cur[2] >= 0
                            && cur[2] < dz as i32
                            && way(n, cur)
                        {
                            m |= 1u16 << k;
                        }
                    }
                    flags[ci] = m;
                }
            }
        }
        Self {
            min: [0, 0, 0],
            dx,
            dy,
            dz,
            flags,
        }
    }
}

/// Exact CPU `BinaryHeap` Dijkstra reference over `PackedTopoGrid` for all goals.
pub fn solve_min_plus_cpu(
    grid: &PackedTopoGrid,
    goal_cells: &[usize],
    legs: LegsCost,
    threads: usize,
) -> Vec<f32> {
    let runs = usize::from(legs.reach) + 1;
    let states = grid.vol() * runs;
    let total = goal_cells.len() * states;
    let mut out = vec![f32::INFINITY; total];
    let t_eff = threads.max(1).min(goal_cells.len().max(1));

    std::thread::scope(|s| {
        let chunk_len = goal_cells.len().div_ceil(t_eff).max(1);
        for (goal_chunk, out_chunk) in goal_cells
            .chunks(chunk_len)
            .zip(out.chunks_mut(chunk_len * states))
        {
            s.spawn(move || {
                for (gi, &seed_ci) in goal_chunk.iter().enumerate() {
                    let slice = &mut out_chunk[gi * states..(gi + 1) * states];
                    dijkstra_one(grid, seed_ci, legs, slice);
                }
            });
        }
    });
    out
}

fn dijkstra_one(grid: &PackedTopoGrid, seed_ci: usize, legs: LegsCost, dist: &mut [f32]) {
    let runs = usize::from(legs.reach) + 1;
    let mut heap: BinaryHeap<Reverse<(u32, usize, u8)>> = BinaryHeap::new();
    if seed_ci < grid.vol() && (grid.flags[seed_ci] & FLAG_BODY_FITS) != 0 {
        for r in 0..runs {
            dist[seed_ci * runs + r] = 0.0;
            heap.push(Reverse((0, seed_ci, r as u8)));
        }
    }
    let strides = [1isize, grid.dx as isize, (grid.dx * grid.dy) as isize];
    while let Some(Reverse((bits, ci, rc))) = heap.pop() {
        let cur_d = f32::from_bits(bits);
        if cur_d != dist[ci * runs + usize::from(rc)] {
            continue;
        }
        let cx = (ci % grid.dx) as i32;
        let cy = ((ci / grid.dx) % grid.dy) as i32;
        let cz = (ci / (grid.dx * grid.dy)) as i32;
        let cur_sup = (grid.flags[ci] & FLAG_STEP_SUP) != 0;

        for (k, d) in DIRS.iter().enumerate() {
            let nx = cx + d[0];
            let ny = cy + d[1];
            let nz = cz + d[2];
            if nx < 0
                || ny < 0
                || nz < 0
                || nx >= grid.dx as i32
                || ny >= grid.dy as i32
                || nz >= grid.dz as i32
            {
                continue;
            }
            let ni = (ci as isize
                + d[0] as isize * strides[0]
                + d[1] as isize * strides[1]
                + d[2] as isize * strides[2]) as usize;
            let n_flags = grid.flags[ni];
            if (n_flags & (1u16 << k)) == 0 {
                continue;
            }
            let up = d[1] < 0;
            let across = d[1] == 0;
            let diag = k >= 6;
            let leg = cur_d
                + if up {
                    legs.up
                } else if diag {
                    legs.h * std::f32::consts::SQRT_2
                } else if across {
                    legs.h
                } else {
                    legs.down
                };
            let n_sup = (n_flags & FLAG_STEP_SUP) != 0;

            let push_edge = |heap: &mut BinaryHeap<Reverse<(u32, usize, u8)>>,
                             dist: &mut [f32],
                             rn: usize| {
                let ns = ni * runs + rn;
                if leg < dist[ns] {
                    dist[ns] = leg;
                    heap.push(Reverse((leg.to_bits(), ni, rn as u8)));
                }
            };

            if across && !cur_sup {
                if rc >= 1 && (!n_sup || rc == 1) {
                    push_edge(&mut heap, dist, usize::from(rc) - 1);
                }
            } else if !up && !across && !cur_sup {
                if !n_sup {
                    push_edge(&mut heap, dist, usize::from(rc));
                } else if rc == 0 {
                    push_edge(&mut heap, dist, 0);
                }
            } else if rc == 0 && !(up && !n_sup) {
                if n_sup {
                    push_edge(&mut heap, dist, 0);
                } else {
                    for rn in 0..runs {
                        push_edge(&mut heap, dist, rn);
                    }
                }
            }
        }
    }
}

const MIN_PLUS_CL: &str = r#"
__constant int DX_OFF[10] = { 1, -1, 0,  0, 0,  0, 1,  1, -1, -1 };
__constant int DY_OFF[10] = { 0,  0, 0,  0, 1, -1, 0,  0,  0,  0 };
__constant int DZ_OFF[10] = { 0,  0, 1, -1, 0,  0, 1, -1,  1, -1 };

__kernel void topo_min_plus_init(
    __global const ushort* flags,
    __global const uint* goal_cells,
    __global float* dist_a,
    const uint num_goals,
    const uint vol,
    const uint runs
) {
    uint gid = (uint)get_global_id(0);
    uint states_per_goal = vol * runs;
    uint total = num_goals * states_per_goal;
    if (gid >= total) return;

    uint g = gid / states_per_goal;
    uint rem = gid - g * states_per_goal;
    uint ci = rem / runs;
    uint seed_ci = goal_cells[g];
    if (ci == seed_ci && ((flags[ci] & (1u << 11)) != 0u)) {
        dist_a[gid] = 0.0f;
    } else {
        dist_a[gid] = INFINITY;
    }
}

__kernel void topo_min_plus_sweep(
    __global const ushort* flags,
    __global const float* dist_in,
    __global float* dist_out,
    __global uint* changed,
    const uint num_goals,
    const uint dx,
    const uint dy,
    const uint dz,
    const uint runs,
    const float leg_h,
    const float leg_up,
    const float leg_down,
    const float leg_diag
) {
    uint gid = (uint)get_global_id(0);
    uint vol = dx * dy * dz;
    uint states_per_goal = vol * runs;
    uint total = num_goals * states_per_goal;
    if (gid >= total) return;

    float prev = dist_in[gid];
    if (prev == 0.0f) {
        dist_out[gid] = 0.0f;
        return;
    }

    uint g = gid / states_per_goal;
    uint rem = gid - g * states_per_goal;
    uint ci = rem / runs;
    uint rn = rem - ci * runs;

    ushort n_flags = flags[ci];
    uint n_sup = (n_flags >> 10) & 1u;
    if (n_sup && rn > 0u) {
        dist_out[gid] = prev;
        return;
    }

    uint nx = ci % dx;
    uint tmp = ci / dx;
    uint ny = tmp % dy;
    uint nz = tmp / dy;
    uint base = g * states_per_goal;
    float best = prev;

    for (uint k = 0u; k < 10u; ++k) {
        if (((n_flags >> k) & 1u) == 0u) continue;
        int ox = DX_OFF[k];
        int oy = DY_OFF[k];
        int oz = DZ_OFF[k];
        int cx = (int)nx - ox;
        int cy = (int)ny - oy;
        int cz = (int)nz - oz;
        if (cx < 0 || cy < 0 || cz < 0 || cx >= (int)dx || cy >= (int)dy || cz >= (int)dz) continue;

        uint cur_ci = (((uint)cz * dy + (uint)cy) * dx + (uint)cx);
        uint cur_sup = (flags[cur_ci] >> 10) & 1u;

        uint up = (oy < 0) ? 1u : 0u;
        uint across = (oy == 0) ? 1u : 0u;

        uint rc;
        if (across && !cur_sup) {
            rc = rn + 1u;
            if (rc >= runs || (n_sup && rc != 1u)) continue;
        } else if (!up && !across && !cur_sup) {
            if (n_sup && rn != 0u) continue;
            rc = rn;
        } else {
            if (up && !n_sup) continue;
            if (n_sup && rn != 0u) continue;
            rc = 0u;
        }

        float step_cost = up ? leg_up : (k >= 6u ? leg_diag : (across ? leg_h : leg_down));
        float cand = dist_in[base + cur_ci * runs + rc] + step_cost;
        if (cand < best) {
            best = cand;
        }
    }

    dist_out[gid] = best;
    if (best < prev) {
        changed[0] = 1u;
    }
}
"#;

pub struct UploadedTopoGrid {
    pub dx: usize,
    pub dy: usize,
    pub dz: usize,
    pub flags_buf: GpuBuffer,
}

impl UploadedTopoGrid {
    #[inline]
    pub fn vol(&self) -> usize {
        self.dx * self.dy * self.dz
    }
}

pub struct MinPlusGpuProgram {
    dev: GpuDevice,
    _prog: GpuProgram,
    init: std::sync::Mutex<GpuKernel>,
    sweep: std::sync::Mutex<GpuKernel>,
}

impl MinPlusGpuProgram {
    pub fn compile(dev: &GpuDevice) -> Result<Self, String> {
        let prog = dev.build_program(MIN_PLUS_CL)?;
        let init = prog.kernel("topo_min_plus_init")?;
        let sweep = prog.kernel("topo_min_plus_sweep")?;
        Ok(Self {
            dev: dev.clone(),
            _prog: prog,
            init: std::sync::Mutex::new(init),
            sweep: std::sync::Mutex::new(sweep),
        })
    }

    pub fn upload_grid(&self, grid: &PackedTopoGrid) -> Result<UploadedTopoGrid, String> {
        let flags_buf = self.dev.alloc_ro_slice(&grid.flags)?;
        Ok(UploadedTopoGrid {
            dx: grid.dx,
            dy: grid.dy,
            dz: grid.dz,
            flags_buf,
        })
    }

    pub fn solve_batch(
        &self,
        grid: &PackedTopoGrid,
        goal_cells: &[usize],
        legs: LegsCost,
        max_sweeps: usize,
    ) -> Result<(Vec<f32>, usize, f64), String> {
        let t0 = Instant::now();
        let total = goal_cells.len() * grid.vol() * (usize::from(legs.reach) + 1);
        if total == 0 {
            return Ok((Vec::new(), 0, 0.0));
        }
        let uploaded = self.upload_grid(grid)?;
        let (out, sweeps, _) = self.solve_batch_uploaded(&uploaded, goal_cells, legs, max_sweeps)?;
        Ok((out, sweeps, t0.elapsed().as_secs_f64() * 1e3))
    }

    pub fn solve_batch_uploaded(
        &self,
        grid: &UploadedTopoGrid,
        goal_cells: &[usize],
        legs: LegsCost,
        max_sweeps: usize,
    ) -> Result<(Vec<f32>, usize, f64), String> {
        let t0 = Instant::now();
        let runs = usize::from(legs.reach) + 1;
        let vol = grid.vol();
        let states = vol * runs;
        let total = goal_cells.len() * states;
        if total == 0 {
            return Ok((Vec::new(), 0, 0.0));
        }

        let goal_u32: Vec<u32> = goal_cells.iter().map(|&c| c as u32).collect();
        let buf_flags = &grid.flags_buf;
        let buf_goals = self.dev.alloc_ro_slice(&goal_u32)?;
        let bytes = total * std::mem::size_of::<f32>();
        let buf_a = self.dev.alloc_rw(bytes)?;
        let buf_b = self.dev.alloc_rw(bytes)?;
        let buf_changed = self.dev.alloc_rw_slice(&[0u32; 1])?;

        let num_goals_u32 = goal_cells.len() as u32;
        let vol_u32 = vol as u32;
        let dx_u32 = grid.dx as u32;
        let dy_u32 = grid.dy as u32;
        let dz_u32 = grid.dz as u32;
        let runs_u32 = runs as u32;
        let leg_diag = legs.h * std::f32::consts::SQRT_2;

        {
            let mut ik = self.init.lock().unwrap();
            ik.set_buf(0, buf_flags)?;
            ik.set_buf(1, &buf_goals)?;
            ik.set_buf(2, &buf_a)?;
            ik.set_val(3, &num_goals_u32)?;
            ik.set_val(4, &vol_u32)?;
            ik.set_val(5, &runs_u32)?;
            ik.enqueue_1d(total, Some(64))?;
        }

        let mut kern = self.sweep.lock().unwrap();
        kern.set_buf(0, buf_flags)?;
        kern.set_buf(3, &buf_changed)?;
        kern.set_val(4, &num_goals_u32)?;
        kern.set_val(5, &dx_u32)?;
        kern.set_val(6, &dy_u32)?;
        kern.set_val(7, &dz_u32)?;
        kern.set_val(8, &runs_u32)?;
        kern.set_val(9, &legs.h)?;
        kern.set_val(10, &legs.up)?;
        kern.set_val(11, &legs.down)?;
        kern.set_val(12, &leg_diag)?;

        let batch = 16usize;
        let mut a_is_in = true;
        let mut sweeps_done = 0usize;
        let mut changed_host = [0u32; 1];

        while sweeps_done < max_sweeps {
            let chunk = batch.min(max_sweeps - sweeps_done);
            buf_changed.write_slice_async(&[0u32; 1])?;
            for _ in 0..chunk {
                if a_is_in {
                    kern.set_buf(1, &buf_a)?;
                    kern.set_buf(2, &buf_b)?;
                } else {
                    kern.set_buf(1, &buf_b)?;
                    kern.set_buf(2, &buf_a)?;
                }
                kern.enqueue_1d(total, Some(64))?;
                a_is_in = !a_is_in;
                sweeps_done += 1;
            }
            buf_changed.read_slice(&mut changed_host)?;
            if changed_host[0] == 0 {
                break;
            }
        }
        drop(kern);

        let mut out = vec![0.0f32; total];
        if a_is_in {
            buf_a.read_slice(&mut out)?;
        } else {
            buf_b.read_slice(&mut out)?;
        }
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1e3;
        Ok((out, sweeps_done, elapsed_ms))
    }
}
