//! The class: the state the law read before writing it, as words (vocabulary.md,
//! section 3). It is a cache key and nothing else: no bit of it reaches the law.
use law::{BlockPos, PlayerState, Vec3, pattern::Start, state_reads::{StateField as F, StateReads}};

/// The quantizations.
/// - `horizontal`: the edges, inside a block, of the bins for x and z.
/// - `vertical`: bins per block for y and for the fall distance, a length along y.
/// - `velocity_bits`: the velocity is kept to this many bits below the leading bit of
///   its largest component. A segment ends when the body changes block, so it lasts
///   about 1/|v| ticks, and a velocity error dv moves its exit by dv/|v|: what the
///   exit depends on is the relative error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub horizontal: &'static [f64],
    pub vertical: f64,
    pub velocity_bits: i32,
}
/// 1/16 block, the frozen record's choice (vocabulary.md, section 3).
pub const SIXTEENTHS: &[f64] = &[0.0625, 0.125, 0.1875, 0.25, 0.3125, 0.375, 0.4375, 0.5, 0.5625, 0.625, 0.6875, 0.75, 0.8125, 0.875, 0.9375];
/// Where the body's 0.6-wide box starts and stops overlapping the next block.
pub const BODY_EDGES: &[f64] = &[0.3, 0.7];
impl Default for Grid {
    fn default() -> Self { Self { horizontal: BODY_EDGES, vertical: 16.0, velocity_bits: 4 } }
}

pub fn body_block(state: &PlayerState) -> BlockPos {
    [state.position.x, state.position.y, state.position.z].map(law::math::floor)
}

fn velocity(v: Vec3, bits: i32) -> [u64; 4] {
    let largest = v.x.abs().max(v.y.abs()).max(v.z.abs());
    if largest == 0.0 { return [0; 4]; }
    let exponent = (largest.to_bits() >> 52) as i32 - 1023;
    let quantum = 2f64.powi(exponent - bits);
    [exponent as u64, (v.x / quantum).round() as i64 as u64, (v.y / quantum).round() as i64 as u64,
        (v.z / quantum).round() as i64 as u64]
}

/// The class of `state` under the fields in `reads`: the mask, then each field's words.
/// Resources are guards and deltas, never key; `t` is only counted.
pub fn class(state: &PlayerState, reads: StateReads, grid: Grid) -> Box<[u64]> {
    let block = body_block(state);
    let mut words = vec![reads.0 as u64, (reads.0 >> 64) as u64];
    for field in reads.fields() {
        match field {
            F::t => {}
            _ if field.is_resource() => {}
            F::position => {
                let within = |a: usize, p: f64| p - block[a] as f64;
                let bin = |a: usize, p: f64| grid.horizontal.iter().filter(|&&edge| edge <= within(a, p)).count() as u64;
                words.extend([bin(0, state.position.x), (within(1, state.position.y) * grid.vertical).floor() as u64, bin(2, state.position.z)]);
            }
            F::velocity => words.extend(velocity(state.velocity, grid.velocity_bits)),
            F::fall_distance => words.push((state.fall_distance * grid.vertical).floor() as i64 as u64),
            F::main_supporting_block => match state.main_supporting_block {
                Some(support) => words.extend((0..3).map(|a| (support[a] - block[a]) as u64 & 0xffff_ffff)),
                None => words.push(u64::MAX),
            },
            _ => state.push_field_bits(field, &mut words),
        }
    }
    words.into()
}

/// The class as a pattern-table start: the world part of the class is then the
/// pattern key of the cells read, anchored at the body's block.
pub fn start(state: &PlayerState, reads: StateReads, grid: Grid) -> Start {
    Start { origin: body_block(state), words: class(state, reads, grid), context: law::CollisionContext::of(state) }
}
