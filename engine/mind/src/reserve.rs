//! 4D space-time voxel reservation table (`(cell, t_bucket)`) for bottleneck-only stagger.
use law::BlockPos;

use crate::{
    pool::{CertifiedStep, JUMP_ARC_Y},
    world_mirror::FxHashMap,
};

pub const TIME_BUCKET_TICKS: u32 = 2;
pub const MAX_RESERVE_DELAY: u16 = 240;

pub struct BotFootprint {
    pub bot_id: u32,
    pub bot_index: u32,
    pub base_delay: u16,
    pub total_ticks: u32,
    pub cells: Vec<(BlockPos, u32)>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReserveStats {
    pub checked_bots: u32,
    pub staggered_bots: u32,
    pub max_extra_delay: u16,
    pub reserved_cells: usize,
    pub residual_conflicts: u32,
}

#[derive(Default)]
pub struct ReservationTable {
    occupied: FxHashMap<(BlockPos, u32), u32>,
}

impl ReservationTable {
    pub fn clear(&mut self) {
        self.occupied.clear();
    }

    pub fn resolve_batch(
        &mut self,
        footprints: &mut [BotFootprint],
    ) -> (Vec<(u32, u16)>, ReserveStats) {
        self.occupied.clear();
        footprints.sort_unstable_by_key(|f| (f.total_ticks, f.bot_index));

        let max_shift_buckets = u32::from(MAX_RESERVE_DELAY) / TIME_BUCKET_TICKS;
        let mut extra_delays = Vec::new();
        let mut stats = ReserveStats {
            checked_bots: footprints.len() as u32,
            ..ReserveStats::default()
        };

        for f in footprints.iter() {
            let base_tb = u32::from(f.base_delay) / TIME_BUCKET_TICKS;
            let mut chosen_shift = 0u32;
            let mut chosen_hits = u32::MAX;
            for shift in 0..=max_shift_buckets {
                let mut hits = 0u32;
                for &(cell, tb) in &f.cells {
                    if self.occupied.contains_key(&(cell, base_tb + tb + shift)) {
                        hits += 1;
                    }
                }
                if hits < chosen_hits {
                    chosen_hits = hits;
                    chosen_shift = shift;
                    if hits == 0 {
                        break;
                    }
                }
            }

            if chosen_hits == u32::MAX {
                chosen_hits = 0;
            }
            for &(cell, tb) in &f.cells {
                self.occupied.insert((cell, base_tb + tb + chosen_shift), f.bot_id);
            }
            if chosen_shift > 0 {
                let extra = (chosen_shift * TIME_BUCKET_TICKS).min(u32::from(u16::MAX)) as u16;
                extra_delays.push((f.bot_id, extra));
                stats.staggered_bots += 1;
                stats.max_extra_delay = stats.max_extra_delay.max(extra);
            }
            stats.residual_conflicts += chosen_hits;
        }
        stats.reserved_cells = self.occupied.len();
        (extra_delays, stats)
    }
}

/// Extract the quantized `(cell, t_bucket)` 4D trajectory for a certified `WordStep` sequence.
pub fn extract_footprint(
    bot_id: u32,
    bot_index: u32,
    base_delay: u16,
    steps: &[CertifiedStep],
) -> BotFootprint {
    let mut cells: Vec<(BlockPos, u32)> = Vec::new();
    let mut t_base = 0u32;

    for step in steps {
        let ticks = step.ticks.max(1);
        for dt in 0..ticks {
            let (x, y, z) = if let Some(curve) = &step.curve
                && let Some(p) = curve.get(dt as usize)
            {
                (
                    step.start_pos.x + f64::from(p.dx),
                    step.start_pos.y + f64::from(p.dy),
                    step.start_pos.z + f64::from(p.dz),
                )
            } else {
                let frac = f64::from(dt + 1) / f64::from(ticks);
                let x = step.start_pos.x + (step.end.position.x - step.start_pos.x) * frac;
                let z = step.start_pos.z + (step.end.position.z - step.start_pos.z) * frac;
                let base_y = step.start_pos.y + (step.end.position.y - step.start_pos.y) * frac;
                let y = if step.program == plan::store::SPRINT_JUMP {
                    let phase = ((dt + 1) as usize) % JUMP_ARC_Y.len();
                    step.start_pos.y.min(step.end.position.y) + JUMP_ARC_Y[phase]
                } else {
                    base_y
                };
                (x, y, z)
            };
            let cell = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
            let tb = (t_base + dt) / TIME_BUCKET_TICKS;
            if cells.last() != Some(&(cell, tb)) {
                cells.push((cell, tb));
            }
        }
        t_base += step.ticks;
    }

    BotFootprint {
        bot_id,
        bot_index,
        base_delay,
        total_ticks: t_base,
        cells,
    }
}
