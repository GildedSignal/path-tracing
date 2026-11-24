use crate::calendar::Wheel;
use law::{BlockPos, Vec3};
use std::collections::HashMap;

pub const READ_SHAPE: u8 = 2;
#[allow(dead_code)]
pub const READ_FLUID: u8 = 4;
#[allow(dead_code)]
pub const READ_HOOKS: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CertHandle {
    Stream(usize),
    Arrow(usize),
    WorldGait(usize),
    Bot(u64),
}

#[derive(Clone, Copy, Debug)]
pub struct CertEntry {
    pub t_start: u64,
    pub t_end: u64,
    pub mask: u8,
    pub handle: CertHandle,
}

#[inline]
fn section_key(p: BlockPos) -> [i32; 3] {
    [p[0] >> 4, p[1] >> 4, p[2] >> 4]
}

#[inline]
fn subcell_bit(p: BlockPos) -> u64 {
    let sx = ((p[0] & 15) >> 2) as u32;
    let sy = ((p[1] & 15) >> 2) as u32;
    let sz = ((p[2] & 15) >> 2) as u32;
    1u64 << (sx | (sy << 2) | (sz << 4))
}

#[derive(Default)]
pub struct SpaceTimeIndex {
    sections: HashMap<[i32; 3], u64>,
    cells: HashMap<BlockPos, Vec<CertEntry>>,
}

impl SpaceTimeIndex {
    pub fn register(
        &mut self,
        pos: BlockPos,
        t_start: u64,
        t_end: u64,
        mask: u8,
        handle: CertHandle,
    ) {
        *self.sections.entry(section_key(pos)).or_default() |= subcell_bit(pos);
        self.cells.entry(pos).or_default().push(CertEntry {
            t_start,
            t_end,
            mask,
            handle,
        });
    }

    pub fn notify_edit(
        &self,
        t: u64,
        pos: BlockPos,
        write_mask: u8,
        out: &mut Vec<CertHandle>,
    ) {
        out.clear();
        let Some(&sec_mask) = self.sections.get(&section_key(pos)) else {
            return;
        };
        if (sec_mask & subcell_bit(pos)) == 0 {
            return;
        }
        let Some(entries) = self.cells.get(&pos) else {
            return;
        };
        for e in entries {
            if t >= e.t_start && t <= e.t_end && (e.mask & write_mask) != 0 {
                out.push(e.handle);
            }
        }
    }

    pub fn bytes(&self) -> usize {
        let sec = self.sections.capacity() * (std::mem::size_of::<([i32; 3], u64)>() + 1);
        let cell_vec_bytes: usize = self
            .cells
            .values()
            .map(|v| v.capacity() * std::mem::size_of::<CertEntry>())
            .sum();
        let cell_map = self.cells.capacity() * (std::mem::size_of::<(BlockPos, Vec<CertEntry>)>() + 1);
        sec + cell_map + cell_vec_bytes
    }
}

/// 4-tick repeater clock folded as a periodic WorldGait.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WorldGait {
    pub t0: u64,
    pub period: u32,
    pub pos: BlockPos,
    pub states: [u8; 4],
}

impl WorldGait {
    pub fn clock_4(t0: u64, pos: BlockPos) -> Self {
        Self {
            t0,
            period: 4,
            pos,
            states: [0, 1, 2, 3],
        }
    }

    #[inline]
    pub fn eval(&self, t: u64) -> u8 {
        self.states[((t - self.t0) % self.period as u64) as usize]
    }

    pub fn materialize(&self, t: u64) -> u8 {
        self.eval(t)
    }

    /// Brute-force stepped reference over `[t0..t]`.
    pub fn stepped_oracle(t0: u64, t: u64) -> u8 {
        let mut s = 0u8;
        for _ in t0..t {
            s = (s + 1) % 4;
        }
        s
    }
}

/// Drives a Wheel over `100_000` ticks with an undisturbed WorldGait registered at `t_far > 100_000`.
/// Returns number of wheel wake-ups (must be 0 while undisturbed).
pub fn verify_undisturbed_world_gait(gait: &WorldGait, horizon: u64) -> usize {
    let mut wheel = Wheel::new(4, 0, 10, 6);
    // Undisturbed periodic WorldGait needs no per-tick wakeup; only horizon/end-of-tube check.
    wheel.file(0, horizon as i64 + 1_000);
    let mut out = Vec::new();
    let mut wakeups = 0usize;
    for step in (1_000..=horizon).step_by(1_000) {
        out.clear();
        wheel.drain(step as i64, |_| Some(horizon as i64 + 1_000), &mut out);
        wakeups += out.len();
    }
    assert_eq!(gait.eval(horizon), WorldGait::stepped_oracle(gait.t0, horizon));
    wakeups
}

/// Registers a flying arrow trajectory whose swept cells cross `target_pos` before `t_end`.
pub fn register_arrow_crossing(
    index: &mut SpaceTimeIndex,
    arrow_id: usize,
    start: Vec3,
    vel: Vec3,
    t0: u64,
    ticks: u64,
    target_pos: BlockPos,
) {
    let mut p = start;
    let mut v = vel;
    let mut hit_target = false;
    let mut visited = std::collections::HashSet::new();
    for _dt in 0..ticks {
        let next = p.plus(v);
        // Conservative swept cell AABB along segment (p..next)
        let lo = [
            p.x.min(next.x).floor() as i32,
            p.y.min(next.y).floor() as i32,
            p.z.min(next.z).floor() as i32,
        ];
        let hi = [
            p.x.max(next.x).floor() as i32,
            p.y.max(next.y).floor() as i32,
            p.z.max(next.z).floor() as i32,
        ];
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                for x in lo[0]..=hi[0] {
                    let cell = [x, y, z];
                    if cell == target_pos {
                        hit_target = true;
                    }
                    if visited.insert(cell) {
                        index.register(cell, t0, t0 + ticks, READ_SHAPE, CertHandle::Arrow(arrow_id));
                    }
                }
            }
        }
        p = next;
        v = Vec3::new(v.x * 0.99f64, v.y * 0.99f64 - 0.05f64, v.z * 0.99f64);
    }
    assert!(hit_target, "arrow setup must cross target_pos {target_pos:?}");
}
