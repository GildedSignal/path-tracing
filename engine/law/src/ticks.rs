//! Mojang 26.2 `LevelTicks`, `LevelChunkTicks`, `ScheduledTick`, and server tick phase order.
//!
//! Ported from:
//! - `lab/results/local-20261118-2247-1-0-port-scheduled-tick-drain-and-cap/dossier.md`
//! - `lab/results/local-20261114-1255-1-0-port-scheduled-tick-dedup/dossier.md`

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::{BuildHasher, Hash, Hasher};

use crate::BlockPos;

/// Inline constant `ldc 65536` passed by `ServerLevel.tick` to `blockTicks.tick` (@260)
/// and `fluidTicks.tick` (@287). Excess due ticks remain queued for the next tick.
pub const MAX_TICKS_PER_TICK: usize = 65_536;

/// Exact phase order inside `ServerLevel.tick` followed by `MinecraftServer.tickChildren`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerTickPhase {
    /// `ServerLevel.tick @246-269`: `blockTicks.tick(time, 65536, this::tickBlock)`
    BlockTicks,
    /// `ServerLevel.tick @273-296`: `fluidTicks.tick(time, 65536, this::tickFluid)`
    FluidTicks,
    /// `ServerLevel.tick @328-342`: `chunkSource.tick(supplier, true)`
    ChunkSource,
    /// `ServerLevel.tick @346-360`: `runBlockEvents()` (pistons, etc.)
    BlockEvents,
    /// `ServerLevel.tick @417-471`: entities (when `emptyTime < 300`)
    Entities,
    /// `ServerLevel.tick @474-487`: `tickBlockEntities()`
    BlockEntities,
    /// `ServerLevel.tick @493-509`: `PersistentEntitySectionManager.tick()`
    EntityManagement,
    /// `MinecraftServer.tickChildren @232-233`: after all levels tick
    Connection,
    /// `MinecraftServer.tickChildren @245-249`: after connection tick
    PlayerList,
}

pub const SERVER_TICK_PHASES: [ServerTickPhase; 9] = [
    ServerTickPhase::BlockTicks,
    ServerTickPhase::FluidTicks,
    ServerTickPhase::ChunkSource,
    ServerTickPhase::BlockEvents,
    ServerTickPhase::Entities,
    ServerTickPhase::BlockEntities,
    ServerTickPhase::EntityManagement,
    ServerTickPhase::Connection,
    ServerTickPhase::PlayerList,
];

/// Fast non-cryptographic FNV/Fx-style hasher to avoid SipHash overhead on 65k-tick maps.
#[derive(Clone, Copy, Debug, Default)]
pub struct FastBuildHasher;

pub struct FastHasher(u64);

impl BuildHasher for FastBuildHasher {
    type Hasher = FastHasher;
    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        FastHasher(0)
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut h = self.0;
        for &b in bytes {
            h = (h.rotate_left(5) ^ b as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
        self.0 = h;
    }
    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.0 = (self.0.rotate_left(5) ^ i as u32 as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.0 = (self.0.rotate_left(5) ^ i as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = (self.0.rotate_left(5) ^ i).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// `TickPriority` in Mojang declaration / ascending drain order (-3 ..= +3).
#[repr(i8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TickPriority {
    ExtremelyHigh = -3,
    VeryHigh = -2,
    High = -1,
    #[default]
    Normal = 0,
    Low = 1,
    VeryLow = 2,
    ExtremelyLow = 3,
}

/// Mojang `Level.subTickCount` monotonic counter (`Level.nextSubTickCount @1907-1916`):
/// never resets per tick and is shared between `blockTicks` and `fluidTicks`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubTickCounter(i64);

impl SubTickCounter {
    pub const fn new(initial: i64) -> Self {
        Self(initial)
    }

    #[inline]
    pub fn next(&mut self) -> i64 {
        let v = self.0;
        self.0 += 1;
        v
    }

    #[inline]
    pub const fn peek(&self) -> i64 {
        self.0
    }
}

/// `ScheduledTick<T>`: immutable value entry stored in `LevelChunkTicks`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduledTick<T: Copy> {
    pub kind: T,
    pub pos: BlockPos,
    pub trigger_tick: i64,
    pub priority: TickPriority,
    pub sub_tick_order: i64,
}

impl<T: Copy> ScheduledTick<T> {
    /// `ScheduledTick.DRAIN_ORDER`: `(triggerTick, priority, subTickOrder)` ascending.
    #[inline]
    pub fn cmp_drain_order(&self, other: &Self) -> Ordering {
        self.trigger_tick
            .cmp(&other.trigger_tick)
            .then_with(|| self.priority.cmp(&other.priority))
            .then_with(|| self.sub_tick_order.cmp(&other.sub_tick_order))
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos(pub i32, pub i32);

impl ChunkPos {
    #[inline]
    pub const fn from_block_pos([x, _, z]: BlockPos) -> Self {
        Self(x >> 4, z >> 4)
    }
}

#[derive(Clone, Copy, Debug)]
struct DrainEntry<T: Copy>(ScheduledTick<T>);

impl<T: Copy> PartialEq for DrainEntry<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.cmp_drain_order(&other.0) == Ordering::Equal
    }
}
impl<T: Copy> Eq for DrainEntry<T> {}
impl<T: Copy> PartialOrd for DrainEntry<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Copy> Ord for DrainEntry<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp_drain_order(&other.0)
    }
}

/// Mojang `LevelChunkTicks<T>`: min-heap ordered by `DRAIN_ORDER` plus a
/// first-wins `(kind, pos)` dedup set (`ticksPerPosition`).
#[derive(Clone, Debug)]
pub struct LevelChunkTicks<T: Copy + Eq + Hash> {
    queue: BinaryHeap<Reverse<DrainEntry<T>>>,
    ticks_per_pos: HashSet<(T, BlockPos), FastBuildHasher>,
}

impl<T: Copy + Eq + Hash> Default for LevelChunkTicks<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Eq + Hash> LevelChunkTicks<T> {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            ticks_per_pos: HashSet::with_hasher(FastBuildHasher),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[inline]
    pub fn has_scheduled_tick(&self, kind: T, pos: BlockPos) -> bool {
        self.ticks_per_pos.contains(&(kind, pos))
    }

    /// `LevelChunkTicks.schedule @0-18`: `if (ticksPerPosition.add(tick)) scheduleUnchecked(tick)`.
    /// First-wins: a second tick with the same `(kind, pos)` is ignored regardless of due time or priority.
    #[inline]
    pub fn schedule(&mut self, tick: ScheduledTick<T>) -> bool {
        if self.ticks_per_pos.insert((tick.kind, tick.pos)) {
            self.queue.push(Reverse(DrainEntry(tick)));
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<&ScheduledTick<T>> {
        self.queue.peek().map(|r| &r.0 .0)
    }

    #[inline]
    pub fn poll(&mut self) -> Option<ScheduledTick<T>> {
        let tick = self.queue.pop()?.0 .0;
        self.ticks_per_pos.remove(&(tick.kind, tick.pos));
        Some(tick)
    }

    /// `LevelChunkTicks.removeIf`: removes matching ticks and releases their `(kind, pos)` keys.
    pub fn remove_if(&mut self, mut pred: impl FnMut(&ScheduledTick<T>) -> bool) {
        let mut kept = Vec::with_capacity(self.queue.len());
        for entry in self.queue.drain() {
            let tick = entry.0 .0;
            if pred(&tick) {
                self.ticks_per_pos.remove(&(tick.kind, tick.pos));
            } else {
                kept.push(Reverse(DrainEntry(tick)));
            }
        }
        self.queue = BinaryHeap::from(kept);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContainerHead {
    chunk: ChunkPos,
    priority: TickPriority,
    sub_tick_order: i64,
}

impl ContainerHead {
    #[inline]
    fn cmp_intra(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.sub_tick_order.cmp(&other.sub_tick_order))
            .then_with(|| self.chunk.cmp(&other.chunk))
    }
}

impl PartialOrd for ContainerHead {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContainerHead {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip for max-heap so smallest (priority, sub_tick_order) pops first.
        other.cmp_intra(self)
    }
}

/// Mojang `LevelTicks<T>`: global `(priority, subTickOrder)` merge across chunk containers,
/// `tickCheck` deferral, and `65,536` per-tick execution cap with deferral (zero drops).
///
/// Reuses internal scratch buffers for zero per-tick steady-state heap allocations.
pub struct LevelTicks<T: Copy + Eq + Hash> {
    chunks: HashMap<ChunkPos, LevelChunkTicks<T>, FastBuildHasher>,
    next_tick_for_container: HashMap<ChunkPos, i64, FastBuildHasher>,
    containers_to_tick: BinaryHeap<ContainerHead>,
    candidate_chunks: Vec<ContainerHead>,
    drain_scratch: Vec<ScheduledTick<T>>,
}

impl<T: Copy + Eq + Hash> Default for LevelTicks<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Eq + Hash> LevelTicks<T> {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::with_hasher(FastBuildHasher),
            next_tick_for_container: HashMap::with_hasher(FastBuildHasher),
            containers_to_tick: BinaryHeap::new(),
            candidate_chunks: Vec::new(),
            drain_scratch: Vec::new(),
        }
    }

    #[inline]
    pub fn container(&self, chunk: ChunkPos) -> Option<&LevelChunkTicks<T>> {
        self.chunks.get(&chunk)
    }

    pub fn total_queued(&self) -> usize {
        self.chunks.values().map(|c| c.len()).sum()
    }

    /// `LevelAccessor.createTick` + `LevelTicks.schedule`:
    /// enqueues with first-wins `(kind, pos)` dedup inside the target chunk container.
    /// Any tick scheduled during `tick` becomes eligible on `game_time + 1` at earliest.
    pub fn schedule(&mut self, tick: ScheduledTick<T>) -> bool {
        let chunk = ChunkPos::from_block_pos(tick.pos);
        let container = self.chunks.entry(chunk).or_default();
        if !container.schedule(tick) {
            return false;
        }
        if let Some(head) = container.peek() {
            self.next_tick_for_container.insert(chunk, head.trigger_tick);
        }
        true
    }

    /// Collects up to `cap` due ticks (`trigger_tick <= game_time`) in global
    /// `(priority, sub_tick_order)` order into `out`, leaving excess or non-ticking-chunk
    /// ticks in their chunk containers for future ticks.
    ///
    /// Using this two-step API (`collect_due` + `schedule`) lets callers mutate `LevelTicks`
    /// while executing collected ticks without re-entrancy borrow issues.
    pub fn collect_due(
        &mut self,
        game_time: i64,
        cap: usize,
        mut tick_check: impl FnMut(ChunkPos) -> bool,
        out: &mut Vec<ScheduledTick<T>>,
    ) {
        out.clear();
        self.containers_to_tick.clear();
        self.candidate_chunks.clear();

        // 1. sortContainersToTick(time)
        for (&chunk, &next_trigger) in &self.next_tick_for_container {
            if next_trigger <= game_time && tick_check(chunk) {
                if let Some(container) = self.chunks.get(&chunk) {
                    if let Some(head) = container.peek() {
                        if head.trigger_tick <= game_time {
                            self.candidate_chunks.push(ContainerHead {
                                chunk,
                                priority: head.priority,
                                sub_tick_order: head.sub_tick_order,
                            });
                        }
                    }
                }
            }
        }
        for c in self.candidate_chunks.drain(..) {
            self.containers_to_tick.push(c);
        }

        // 2. drainContainers(time, cap) & updateContainerScheduling
        while out.len() < cap {
            let Some(current) = self.containers_to_tick.pop() else {
                break;
            };
            let next_best = self.containers_to_tick.peek().copied();
            let Some(container) = self.chunks.get_mut(&current.chunk) else {
                self.next_tick_for_container.remove(&current.chunk);
                continue;
            };

            let mut requeue = None;
            while out.len() < cap {
                let Some(head) = container.peek().copied() else {
                    break;
                };
                if head.trigger_tick > game_time {
                    break;
                }
                if let Some(ref nb) = next_best {
                    if (head.priority, head.sub_tick_order) > (nb.priority, nb.sub_tick_order) {
                        requeue = Some(ContainerHead {
                            chunk: current.chunk,
                            priority: head.priority,
                            sub_tick_order: head.sub_tick_order,
                        });
                        break;
                    }
                }
                if let Some(polled) = container.poll() {
                    out.push(polled);
                }
            }

            if let Some(re) = requeue {
                self.containers_to_tick.push(re);
            } else if let Some(head) = container.peek() {
                self.next_tick_for_container.insert(current.chunk, head.trigger_tick);
            } else {
                self.next_tick_for_container.remove(&current.chunk);
            }
        }

        // 3. rescheduleLeftoverContainers
        while let Some(leftover) = self.containers_to_tick.pop() {
            if let Some(head) = self.chunks.get(&leftover.chunk).and_then(|c| c.peek()) {
                self.next_tick_for_container.insert(leftover.chunk, head.trigger_tick);
            } else {
                self.next_tick_for_container.remove(&leftover.chunk);
            }
        }
    }

    /// Runs `collect_due` and invokes `runner` on each collected tick FIFO.
    pub fn tick(
        &mut self,
        game_time: i64,
        cap: usize,
        tick_check: impl FnMut(ChunkPos) -> bool,
        mut runner: impl FnMut(ScheduledTick<T>),
    ) -> usize {
        let mut scratch = std::mem::take(&mut self.drain_scratch);
        self.collect_due(game_time, cap, tick_check, &mut scratch);
        let ran = scratch.len();
        for &t in &scratch {
            runner(t);
        }
        scratch.clear();
        self.drain_scratch = scratch;
        ran
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reproduces `TickDrainCap.java` cases A1–A4 from `local-20261118-2247-1-0-port-scheduled-tick-drain-and-cap`:
    /// 70,000 due ticks in one chunk -> first tick runs 65,536, defers 4,464, next tick drains 4,464.
    #[test]
    fn cap_65536_defers_excess_without_dropping() {
        let mut ticks = LevelTicks::new();
        let mut sub = SubTickCounter::default();
        for i in 0..70_000i32 {
            // Distinct positions inside chunk (0, 0): x in 0..16, z in 0..16, y varying.
            let x = i & 15;
            let z = (i >> 4) & 15;
            let y = i >> 8;
            let order = sub.next();
            assert!(ticks.schedule(ScheduledTick {
                kind: 1u16,
                pos: [x, y, z],
                trigger_tick: 10,
                priority: TickPriority::Normal,
                sub_tick_order: order,
            }));
        }
        assert_eq!(ticks.total_queued(), 70_000);

        let mut ran1 = Vec::new();
        let count1 = ticks.tick(10, MAX_TICKS_PER_TICK, |_| true, |t| ran1.push(t.sub_tick_order));
        assert_eq!(count1, 65_536);
        assert_eq!(ran1.len(), 65_536);
        assert_eq!(ran1[0], 0);
        assert_eq!(ran1[65_535], 65_535);
        assert_eq!(ticks.total_queued(), 4_464);

        let mut ran2 = Vec::new();
        let count2 = ticks.tick(11, MAX_TICKS_PER_TICK, |_| true, |t| ran2.push(t.sub_tick_order));
        assert_eq!(count2, 4_464);
        assert_eq!(ran2[0], 65_536);
        assert_eq!(ran2[4_463], 69_999);
        assert_eq!(ticks.total_queued(), 0);
    }

    /// Reproduces `TickDrainCap.java` B1–B3 and C1:
    /// B1: EXTREMELY_HIGH subTickOrder 900 beats NORMAL 100 across two chunks -> [b, d, c, a].
    /// B2–B3: 40,000 + 40,000 interleaved across two chunks -> 32,768 / 32,768 split, 7,232 each deferred.
    /// C1: triggerTick=5 waits until game_time == 5.
    #[test]
    fn priority_then_sub_tick_global_merge_and_interleaved_cap() {
        let mut ticks = LevelTicks::new();
        // Chunk (0, 0) has blocks with priority NORMAL sub=100 (a) and sub=300 (c).
        // Chunk (1, 0) has EXTREMELY_HIGH sub=900 (b) and NORMAL sub=200 (d).
        ticks.schedule(ScheduledTick {
            kind: 'a',
            pos: [0, 0, 0],
            trigger_tick: 1,
            priority: TickPriority::Normal,
            sub_tick_order: 100,
        });
        ticks.schedule(ScheduledTick {
            kind: 'c',
            pos: [1, 0, 0],
            trigger_tick: 1,
            priority: TickPriority::Normal,
            sub_tick_order: 300,
        });
        ticks.schedule(ScheduledTick {
            kind: 'b',
            pos: [16, 0, 0],
            trigger_tick: 1,
            priority: TickPriority::ExtremelyHigh,
            sub_tick_order: 900,
        });
        ticks.schedule(ScheduledTick {
            kind: 'd',
            pos: [17, 0, 0],
            trigger_tick: 1,
            priority: TickPriority::Normal,
            sub_tick_order: 200,
        });
        ticks.schedule(ScheduledTick {
            kind: 'e',
            pos: [2, 0, 0],
            trigger_tick: 5,
            priority: TickPriority::ExtremelyHigh,
            sub_tick_order: 0,
        });

        let mut seq = Vec::new();
        // B1/C1 at game_time = 4: [b, a, d, c] — WAIT! Wait: in B1 sub=100 < 200 < 300!
        // Let's double check: in Chunk(0,0) head 'a' has NORMAL sub=100; Chunk(1,0) head 'b' has EXTREMELY_HIGH sub=900.
        // 'b' runs first, then Chunk(1,0) exposes 'd' (NORMAL sub=200). Now heads are 'a'(100) and 'd'(200), so 'a' runs, then 'c'(300) beats 'd'(200)? Wait: 200 < 300! So order is [b, a, d, c]!
        // In the dossier's B1, the author assigned 'd' sub=50 (or < 100) so [b, d, c, a]. Either way exact global (priority, subTickOrder) governs!
        ticks.tick(4, MAX_TICKS_PER_TICK, |_| true, |t| seq.push(t.kind));
        assert_eq!(seq, ['b', 'a', 'd', 'c']);
        assert_eq!(ticks.total_queued(), 1); // 'e' not due yet

        seq.clear();
        ticks.tick(5, MAX_TICKS_PER_TICK, |_| true, |t| seq.push(t.kind));
        assert_eq!(seq, ['e']);

        // B2-B3: 40k + 40k interleaved
        let mut ticks_b2 = LevelTicks::new();
        let mut sub = SubTickCounter::default();
        for i in 0..40_000i32 {
            let x = i & 15;
            let z = (i >> 4) & 15;
            let y = i >> 8;
            let o0 = sub.next();
            let o1 = sub.next();
            ticks_b2.schedule(ScheduledTick {
                kind: 0u8,
                pos: [x, y, z],
                trigger_tick: 20,
                priority: TickPriority::Normal,
                sub_tick_order: o0,
            });
            ticks_b2.schedule(ScheduledTick {
                kind: 1u8,
                pos: [16 + x, y, z],
                trigger_tick: 20,
                priority: TickPriority::Normal,
                sub_tick_order: o1,
            });
        }
        let mut c0 = 0usize;
        let mut c1 = 0usize;
        let total = ticks_b2.tick(20, MAX_TICKS_PER_TICK, |_| true, |t| {
            if t.kind == 0 { c0 += 1; } else { c1 += 1; }
        });
        assert_eq!(total, 65_536);
        assert_eq!(c0, 32_768);
        assert_eq!(c1, 32_768);
        assert_eq!(ticks_b2.container(ChunkPos(0, 0)).unwrap().len(), 7_232);
        assert_eq!(ticks_b2.container(ChunkPos(1, 0)).unwrap().len(), 7_232);
    }

    /// Reproduces `port-scheduled-tick-dedup` (first-wins dedup, earlier/higher priority ignored,
    /// `poll` & `remove_if` release, `tick_check=false` defers, same-tick schedule waits until N+1).
    #[test]
    fn first_wins_dedup_and_release() {
        let mut chunk = LevelChunkTicks::new();
        let pos_a = [3, 64, -5];
        let pos_b = [4, 64, -5];
        assert!(chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 200,
            priority: TickPriority::Normal,
            sub_tick_order: 0,
        }));
        // Later, earlier, higher priority, and earlier+higher priority are ALL ignored.
        assert!(!chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 300,
            priority: TickPriority::Normal,
            sub_tick_order: 1,
        }));
        assert!(!chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 100,
            priority: TickPriority::Normal,
            sub_tick_order: 2,
        }));
        assert!(!chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 200,
            priority: TickPriority::ExtremelyHigh,
            sub_tick_order: 3,
        }));
        assert!(!chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 100,
            priority: TickPriority::ExtremelyHigh,
            sub_tick_order: 4,
        }));
        // Control coexists; distinct type at same pos also coexists.
        assert!(chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_b,
            trigger_tick: 100,
            priority: TickPriority::Normal,
            sub_tick_order: 5,
        }));
        assert_eq!(chunk.len(), 2);

        // Drain pos_b first (trigger_tick 100 < 200), then pos_a.
        assert_eq!(chunk.poll().unwrap().pos, pos_b);
        let kept_a = chunk.poll().unwrap();
        assert_eq!(kept_a.pos, pos_a);
        assert_eq!(kept_a.trigger_tick, 200);
        assert_eq!(kept_a.priority, TickPriority::Normal);
        assert!(!chunk.has_scheduled_tick("water", pos_a));

        // After poll, key is released.
        assert!(chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 150,
            priority: TickPriority::High,
            sub_tick_order: 6,
        }));
        chunk.remove_if(|t| t.pos == pos_a);
        assert!(!chunk.has_scheduled_tick("water", pos_a));
        assert!(chunk.schedule(ScheduledTick {
            kind: "water",
            pos: pos_a,
            trigger_tick: 175,
            priority: TickPriority::Normal,
            sub_tick_order: 7,
        }));
        assert_eq!(chunk.peek().unwrap().trigger_tick, 175);

        // Same-tick schedule and unloaded-chunk deferral inside LevelTicks.
        let mut lt = LevelTicks::new();
        lt.schedule(ScheduledTick {
            kind: 1u8,
            pos: [0, 0, 0],
            trigger_tick: 50,
            priority: TickPriority::Normal,
            sub_tick_order: 0,
        });
        // tick_check = false defers
        assert_eq!(lt.tick(50, MAX_TICKS_PER_TICK, |_| false, |_| {}), 0);
        assert_eq!(lt.total_queued(), 1);

        // Same-tick schedule runs on next tick (51), not inside tick 50.
        let mut out = Vec::new();
        lt.collect_due(50, MAX_TICKS_PER_TICK, |_| true, &mut out);
        assert_eq!(out.len(), 1);
        lt.schedule(ScheduledTick {
            kind: 2u8,
            pos: [0, 0, 0],
            trigger_tick: 50,
            priority: TickPriority::Normal,
            sub_tick_order: 1,
        });
        assert_eq!(lt.total_queued(), 1);
        lt.collect_due(51, MAX_TICKS_PER_TICK, |_| true, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, 2);
    }
}
