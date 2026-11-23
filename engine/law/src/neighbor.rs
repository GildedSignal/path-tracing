//! Mojang 26.2 synchronous neighbor & shape update cascade (`Level.setBlock` flags,
//! `NeighborUpdater`, `CollectingNeighborUpdater`, `DefaultRedstoneWireEvaluator`
//! / `RedstoneWireEvaluator`, `DoorBlock`/`TrapDoorBlock`/`FenceGateBlock`,
//! `LadderBlock` support-loss pop).
//!
//! Ported from:
//! - `lab/results/local-20261112-1251-1-6-port-update-order/dossier.md`
//! - `lab/results/local-20261114-1158-1-0-port-update-order/dossier.md`
//! - `CollectingNeighborUpdater` bytecode in `server-26.2.jar`

use crate::{Axis, BlockPos};

/// Vanilla 3D block direction (`Direction.get3DDataValue`):
/// Down=0, Up=1, North=2, South=3, West=4, East=5.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl Direction {
    /// `Direction.values()` and `SignalGetter.hasNeighborSignal` scan order.
    pub const ALL: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    /// `Direction.Plane.HORIZONTAL`: North, East, South, West.
    pub const HORIZONTAL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    #[inline]
    pub const fn from_3d_data(v: u8) -> Self {
        match v & 7 {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            // Vanilla `from3DDataValue` indexes `BY_3D_DATA[abs(v % 6)]`; 6 -> 0 (Down), 7 -> 1 (Up).
            6 => Direction::Down,
            _ => Direction::Up,
        }
    }

    #[inline]
    pub const fn step(self) -> BlockPos {
        match self {
            Direction::Down => [0, -1, 0],
            Direction::Up => [0, 1, 0],
            Direction::North => [0, 0, -1],
            Direction::South => [0, 0, 1],
            Direction::West => [-1, 0, 0],
            Direction::East => [1, 0, 0],
        }
    }

    #[inline]
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    #[inline]
    pub const fn axis(self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }

    /// Sign along `self.axis()`: `+1` for Up/South/East, `-1` for Down/North/West.
    #[inline]
    pub const fn axis_step(self) -> i32 {
        match self {
            Direction::Down | Direction::North | Direction::West => -1,
            Direction::Up | Direction::South | Direction::East => 1,
        }
    }

    #[inline]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Direction::North | Direction::South | Direction::West | Direction::East)
    }

    #[inline]
    pub const fn relative_by(self, [x, y, z]: BlockPos, n: i32) -> BlockPos {
        let [sx, sy, sz] = self.step();
        [x + sx * n, y + sy * n, z + sz * n]
    }
}

/// `NeighborUpdater.UPDATE_ORDER` (neighbor block updates): `{WEST, EAST, DOWN, UP, NORTH, SOUTH}`.
/// Differs from both `Direction::ALL` and `UPDATE_SHAPE_ORDER`.
pub const UPDATE_ORDER: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
];

/// `BlockBehaviour.UPDATE_SHAPE_ORDER` (neighbor shape updates): `{WEST, EAST, NORTH, SOUTH, DOWN, UP}`.
pub const UPDATE_SHAPE_ORDER: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::North,
    Direction::South,
    Direction::Down,
    Direction::Up,
];

/// `Block.UPDATE_*` bitflags and `Level.setBlock` propagation helpers.
pub mod update_flags {
    pub const NEIGHBORS: i32 = 1;
    pub const CLIENTS: i32 = 2;
    pub const INVISIBLE: i32 = 4;
    pub const IMMEDIATE: i32 = 8;
    pub const KNOWN_SHAPE: i32 = 16;
    pub const SUPPRESS_DROPS: i32 = 32;
    pub const MOVE_BY_PISTON: i32 = 64;
    pub const SKIP_SHAPE_UPDATE_ON_WIRE: i32 = 128;
    pub const SKIP_BLOCK_ENTITY_SIDEEFFECTS: i32 = 256;
    pub const SKIP_ON_PLACE: i32 = 512;

    pub const ALL: i32 = NEIGHBORS | CLIENTS;
    pub const ALL_IMMEDIATE: i32 = NEIGHBORS | CLIENTS | IMMEDIATE;
    pub const DEFAULT_LIMIT: i32 = 512;

    /// `LevelChunk.setBlockState @525`: server `onPlace` fires unless `SKIP_ON_PLACE` (512) is set.
    #[inline]
    pub const fn triggers_on_place(flags: i32) -> bool {
        (flags & SKIP_ON_PLACE) == 0
    }

    /// `Level.setBlock @140-153`: neighbor block updates fire iff `NEIGHBORS` (1) is set.
    #[inline]
    pub const fn triggers_neighbor_updates(flags: i32) -> bool {
        (flags & NEIGHBORS) != 0
    }

    /// `Level.setBlock @177-186`: shape phase fires iff `(flags & KNOWN_SHAPE) == 0 && limit > 0`.
    #[inline]
    pub const fn triggers_shape_updates(flags: i32, limit: i32) -> bool {
        (flags & KNOWN_SHAPE) == 0 && limit > 0
    }

    /// `Level.setBlock @190`: `flags & -34` (clears `NEIGHBORS` (1) and `SUPPRESS_DROPS` (32)).
    #[inline]
    pub const fn propagated_shape_flags(flags: i32) -> i32 {
        flags & -34
    }

    /// `Block.updateOrDestroy @53`: `flags & -33` (clears `SUPPRESS_DROPS` (32)).
    #[inline]
    pub const fn update_or_destroy_flags(flags: i32) -> i32 {
        flags & -33
    }

    /// `Block.updateOrDestroy @24-39`: drops items on destroy iff `(flags & SUPPRESS_DROPS) == 0`.
    #[inline]
    pub const fn update_or_destroy_drops(flags: i32) -> bool {
        (flags & SUPPRESS_DROPS) == 0
    }

    /// `NeighborUpdater.executeShapeUpdate @9-29`: skips shape update when targeting wire with flag 128.
    #[inline]
    pub const fn skip_shape_update_on_wire(flags: i32, current_is_wire: bool) -> bool {
        (flags & SKIP_SHAPE_UPDATE_ON_WIRE) != 0 && current_is_wire
    }
}

/// What `Block.updateOrDestroy(oldState, newState, level, pos, flags, recursionLeft)` does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateOrDestroyAction {
    /// `@2`: `newState == oldState` -> no-op.
    NoOp,
    /// `@6-42`: `newState.isAir()` on server -> `Level.destroyBlock(pos, drop, null, recursionLeft)`.
    Destroy {
        drop_resources: bool,
        recursion_left: i32,
    },
    /// `@43-58`: non-air change -> `Level.setBlock(pos, newState, flags & -33, recursionLeft)`.
    SetBlock { flags: i32, recursion_left: i32 },
}

#[inline]
pub const fn decide_update_or_destroy(
    changed: bool,
    new_is_air: bool,
    flags: i32,
    recursion_left: i32,
) -> UpdateOrDestroyAction {
    if !changed {
        return UpdateOrDestroyAction::NoOp;
    }
    if new_is_air {
        UpdateOrDestroyAction::Destroy {
            drop_resources: update_flags::update_or_destroy_drops(flags),
            recursion_left,
        }
    } else {
        UpdateOrDestroyAction::SetBlock {
            flags: update_flags::update_or_destroy_flags(flags),
            recursion_left,
        }
    }
}

/// `CollectingNeighborUpdater` internal stack/layer item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborEntry<B: Copy> {
    Simple {
        pos: BlockPos,
        source_block: B,
        moved_by_piston: bool,
    },
    Multi {
        origin: BlockPos,
        source_block: B,
        skip: Option<Direction>,
        idx: u8,
    },
    Shape {
        pos: BlockPos,
        direction: Direction,
        neighbor_pos: BlockPos,
        neighbor_state: B,
        flags: i32,
        recursion_left: i32,
    },
}

/// One concrete dispatch popped by `CollectingNeighborUpdater` for execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateEvent<B: Copy> {
    Simple {
        pos: BlockPos,
        source_block: B,
        moved_by_piston: bool,
    },
    Shape {
        pos: BlockPos,
        direction: Direction,
        neighbor_pos: BlockPos,
        neighbor_state: B,
        flags: i32,
        recursion_left: i32,
    },
}

/// Summary returned after `run_updates` finishes a cascade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrainStats {
    pub dispatched: usize,
    pub overflowed: bool,
    pub first_skipped_pos: Option<BlockPos>,
}

/// Split-borrowable queue storage so handlers can enqueue nested updates re-entrantly
/// without allocating per cascade (`stack` and `added_this_layer` keep capacity across runs).
#[derive(Clone, Debug)]
pub struct NeighborQueue<B: Copy> {
    stack: Vec<NeighborEntry<B>>,
    added_this_layer: Vec<NeighborEntry<B>>,
    count: i32,
    max_chained_updates: i32,
    overflowed: bool,
    first_skipped_pos: Option<BlockPos>,
}

impl<B: Copy> NeighborQueue<B> {
    pub fn new(max_chained_updates: i32) -> Self {
        Self {
            stack: Vec::with_capacity(64),
            added_this_layer: Vec::with_capacity(16),
            count: 0,
            max_chained_updates,
            overflowed: false,
            first_skipped_pos: None,
        }
    }


    /// Enqueues a single-cell `neighborChanged` update. Returns `true` if the caller
    /// is at the root (not nested) and should invoke `run_updates`.
    pub fn neighbor_changed(&mut self, pos: BlockPos, source_block: B, moved_by_piston: bool) -> bool {
        self.add(pos, NeighborEntry::Simple { pos, source_block, moved_by_piston })
    }

    /// Enqueues `MultiNeighborUpdate` around `origin` in `UPDATE_ORDER`, skipping `skip`.
    pub fn update_neighbors_except(
        &mut self,
        origin: BlockPos,
        source_block: B,
        skip: Option<Direction>,
    ) -> bool {
        let mut idx = 0u8;
        if Some(UPDATE_ORDER[0]) == skip {
            idx = 1;
        }
        self.add(
            origin,
            NeighborEntry::Multi {
                origin,
                source_block,
                skip,
                idx,
            },
        )
    }

    /// Enqueues a shape update on `pos` caused by neighbor `neighbor_pos` along `direction`.
    pub fn shape_update(
        &mut self,
        direction: Direction,
        neighbor_state: B,
        pos: BlockPos,
        neighbor_pos: BlockPos,
        flags: i32,
        recursion_left: i32,
    ) -> bool {
        self.add(
            pos,
            NeighborEntry::Shape {
                pos,
                direction,
                neighbor_pos,
                neighbor_state,
                flags,
                recursion_left,
            },
        )
    }

    /// Exact `CollectingNeighborUpdater.addAndRun @0-174` queueing rule.
    /// Pushing to `added_this_layer` guarantees FIFO order within any layer (root or nested)
    /// and LIFO nesting across layers when flushed in reverse onto `stack`.
    fn add(&mut self, pos: BlockPos, entry: NeighborEntry<B>) -> bool {
        let already_running = self.count > 0;
        let exceeded = self.max_chained_updates >= 0 && self.count >= self.max_chained_updates;
        self.count += 1;
        if !exceeded {
            self.added_this_layer.push(entry);
        } else if self.count - 1 == self.max_chained_updates {
            self.overflowed = true;
            self.first_skipped_pos = Some(pos);
        }
        !already_running
    }

    /// Pops the next single dispatch from `CollectingNeighborUpdater.runUpdates`,
    /// flushing any nested `added_this_layer` onto `stack` in reverse first.
    pub fn pop_next(&mut self) -> Option<UpdateEvent<B>> {
        if !self.added_this_layer.is_empty() {
            for entry in self.added_this_layer.drain(..).rev() {
                self.stack.push(entry);
            }
        }
        let top = self.stack.last_mut()?;
        match top {
            NeighborEntry::Multi {
                origin,
                source_block,
                skip,
                idx,
            } => {
                let dir = UPDATE_ORDER[*idx as usize];
                let target = dir.relative_by(*origin, 1);
                let src = *source_block;
                let skip_dir = *skip;
                *idx += 1;
                if (*idx as usize) < UPDATE_ORDER.len() && Some(UPDATE_ORDER[*idx as usize]) == skip_dir {
                    *idx += 1;
                }
                if (*idx as usize) >= UPDATE_ORDER.len() {
                    self.stack.pop();
                }
                Some(UpdateEvent::Simple {
                    pos: target,
                    source_block: src,
                    moved_by_piston: false,
                })
            }
            NeighborEntry::Simple { .. } => match self.stack.pop() {
                Some(NeighborEntry::Simple {
                    pos,
                    source_block,
                    moved_by_piston,
                }) => Some(UpdateEvent::Simple {
                    pos,
                    source_block,
                    moved_by_piston,
                }),
                _ => unreachable!(),
            },
            NeighborEntry::Shape { .. } => match self.stack.pop() {
                Some(NeighborEntry::Shape {
                    pos,
                    direction,
                    neighbor_pos,
                    neighbor_state,
                    flags,
                    recursion_left,
                }) => Some(UpdateEvent::Shape {
                    pos,
                    direction,
                    neighbor_pos,
                    neighbor_state,
                    flags,
                    recursion_left,
                }),
                _ => unreachable!(),
            },
        }
    }

    pub fn finish_run(&mut self) -> DrainStats {
        let stats = DrainStats {
            dispatched: 0,
            overflowed: self.overflowed,
            first_skipped_pos: self.first_skipped_pos,
        };
        self.stack.clear();
        self.added_this_layer.clear();
        self.count = 0;
        self.overflowed = false;
        self.first_skipped_pos = None;
        stats
    }
}

/// Reusable synchronous Mojang `CollectingNeighborUpdater` runner.
/// Zero heap allocations after warmup when reused across edits.
#[derive(Clone, Debug)]
pub struct CollectingNeighborUpdater<B: Copy> {
    queue: NeighborQueue<B>,
}

impl<B: Copy> CollectingNeighborUpdater<B> {
    pub fn new(max_chained_updates: i32) -> Self {
        Self {
            queue: NeighborQueue::new(max_chained_updates),
        }
    }

    #[inline]
    pub fn queue_mut(&mut self) -> &mut NeighborQueue<B> {
        &mut self.queue
    }

    /// Runs a root update cascade to completion, passing `&mut NeighborQueue<B>`
    /// so `handler` can enqueue nested neighbor/shape updates re-entrantly.
    pub fn drain_with(
        &mut self,
        seed: impl FnOnce(&mut NeighborQueue<B>),
        mut handler: impl FnMut(&mut NeighborQueue<B>, UpdateEvent<B>),
    ) -> DrainStats {
        seed(&mut self.queue);
        let mut dispatched = 0usize;
        while let Some(event) = self.queue.pop_next() {
            dispatched += 1;
            handler(&mut self.queue, event);
        }
        let mut stats = self.queue.finish_run();
        stats.dispatched = dispatched;
        stats
    }
}

/// `DefaultRedstoneWireEvaluator.calculateTargetStrength` + `RedstoneWireEvaluator.getIncomingWireSignal`:
/// `block_signal` = direct/strong power from non-wire neighbors (with short-circuit at 15),
/// `incoming_wire_signals` = wire `POWER` along the 4 horizontal directions (including conductor up-step/down-step).
/// Returns `max(block_signal, max_incoming.saturating_sub(1)).min(15)` (wire writes with `update_flags::CLIENTS` = 2).
#[inline]
pub fn redstone_wire_target_strength(block_signal: u8, incoming_wire_signals: [u8; 4]) -> u8 {
    if block_signal >= 15 {
        return 15;
    }
    let mut max_wire = 0u8;
    for s in incoming_wire_signals {
        if s > max_wire {
            max_wire = s;
        }
    }
    block_signal.max(max_wire.saturating_sub(1)).min(15)
}

/// `RedstoneWireEvaluator.getIncomingWireSignal` for one horizontal direction:
/// - flat neighbor wire -> `flat_wire_power`
/// - step up (`neighbor_is_redstone_conductor && !above_is_redstone_conductor`) -> `up_wire_power`
/// - step down (`!neighbor_is_redstone_conductor`) -> `down_wire_power`
/// (takes max over applicable paths, since flat/below can both exist if non-conductor).
#[inline]
pub fn redstone_wire_side_signal(
    flat_wire_power: Option<u8>,
    neighbor_is_redstone_conductor: bool,
    above_is_redstone_conductor: bool,
    up_wire_power: Option<u8>,
    down_wire_power: Option<u8>,
) -> u8 {
    if let Some(p) = flat_wire_power {
        return p;
    }
    if neighbor_is_redstone_conductor {
        if !above_is_redstone_conductor {
            return up_wire_power.unwrap_or(0);
        }
        return 0;
    }
    down_wire_power.unwrap_or(0)
}

/// `DoorBlock` / `TrapDoorBlock` / `FenceGateBlock` `neighborChanged`:
/// toggles `(powered, open)` to `has_neighbor_signal` via `setBlock(..., update_flags::CLIENTS)` (flag 2)
/// whenever `has_neighbor_signal != current_powered`.
#[inline]
pub const fn powered_toggle_on_signal(
    current_powered: bool,
    has_neighbor_signal: bool,
) -> Option<(bool, i32)> {
    if has_neighbor_signal != current_powered {
        Some((has_neighbor_signal, update_flags::CLIENTS))
    } else {
        None
    }
}

/// `LadderBlock.updateShape @0-31` & `LadderBlock.canSurvive @0-28`:
/// returns `true` (pops to air) iff the changed neighbor is on the attachment side
/// (`update_dir.opposite() == ladder_facing`) and the support face is no longer sturdy.
#[inline]
pub const fn ladder_update_shape_pops(
    ladder_facing: Direction,
    update_dir: Direction,
    support_face_sturdy: bool,
) -> bool {
    update_dir.opposite() as u8 == ladder_facing as u8 && !support_face_sturdy
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn direction_orders_match_26_2_bytecode() {
        // UPDATE_ORDER = {W, E, D, U, N, S} (NeighborUpdater.<clinit>)
        assert_eq!(
            UPDATE_ORDER,
            [
                Direction::West,
                Direction::East,
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::South,
            ]
        );
        // UPDATE_SHAPE_ORDER = {W, E, N, S, D, U} (BlockBehaviour.<clinit>)
        assert_eq!(
            UPDATE_SHAPE_ORDER,
            [
                Direction::West,
                Direction::East,
                Direction::North,
                Direction::South,
                Direction::Down,
                Direction::Up,
            ]
        );
        // Direction.values() = {D, U, N, S, W, E}
        assert_eq!(
            Direction::ALL,
            [
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ]
        );
        // Mutant assertion: UPDATE_ORDER != UPDATE_SHAPE_ORDER
        assert_ne!(UPDATE_ORDER, UPDATE_SHAPE_ORDER);
    }

    #[test]
    fn set_block_flags_match_mojang_26_2() {
        assert!(update_flags::triggers_on_place(update_flags::ALL));
        assert!(!update_flags::triggers_on_place(update_flags::ALL | update_flags::SKIP_ON_PLACE));
        assert!(update_flags::triggers_neighbor_updates(update_flags::ALL));
        assert!(!update_flags::triggers_neighbor_updates(update_flags::CLIENTS));
        assert!(update_flags::triggers_shape_updates(update_flags::ALL, 512));
        assert!(!update_flags::triggers_shape_updates(update_flags::ALL | update_flags::KNOWN_SHAPE, 512));
        assert!(!update_flags::triggers_shape_updates(update_flags::ALL, 0));
        // flags & -34 clears bit 1 (NEIGHBORS) and bit 32 (SUPPRESS_DROPS).
        assert_eq!(update_flags::propagated_shape_flags(3 | 32), 2);
        assert_eq!(update_flags::propagated_shape_flags(11), 10);
        // updateOrDestroy clears bit 32 only.
        assert_eq!(
            decide_update_or_destroy(true, true, 3, 511),
            UpdateOrDestroyAction::Destroy {
                drop_resources: true,
                recursion_left: 511,
            }
        );
        assert_eq!(
            decide_update_or_destroy(true, true, 3 | update_flags::SUPPRESS_DROPS, 511),
            UpdateOrDestroyAction::Destroy {
                drop_resources: false,
                recursion_left: 511,
            }
        );
        assert!(update_flags::skip_shape_update_on_wire(
            update_flags::SKIP_SHAPE_UPDATE_ON_WIRE | update_flags::CLIENTS,
            true
        ));
    }

    /// Reproduces `local-20261112-1251-1-6-port-update-order` Witness:
    /// Lever -> 8 x RedstoneWire -> IronDoor (both halves), verifying depth-first UPDATE_ORDER
    /// fan-out, flag-2 writes, and final powers [15, 14, ..., 8] opening the door synchronously.
    #[test]
    fn redstone_lever_wire_iron_door_cascade() {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Cell {
            Air,
            Stone,
            Lever { powered: bool },
            Wire { power: u8 },
            Door { open: bool },
        }

        let mut world: HashMap<BlockPos, Cell> = HashMap::new();
        for x in 0..=10 {
            world.insert([x, 0, 0], Cell::Stone);
        }
        world.insert([0, 1, 0], Cell::Lever { powered: false });
        for x in 1..=8 {
            world.insert([x, 1, 0], Cell::Wire { power: 0 });
        }
        world.insert([9, 1, 0], Cell::Door { open: false });
        world.insert([9, 2, 0], Cell::Door { open: false });

        let mut updater = CollectingNeighborUpdater::new(100_000);
        let mut visited_order = Vec::new();
        let mut door_write_flags = Vec::new();
        let mut wire_write_flags = Vec::new();

        // Lever.pull: setBlock(lever, powered=true, 3) + updateNeighborsAt
        world.insert([0, 1, 0], Cell::Lever { powered: true });
        let stats = updater.drain_with(
            |q| {
                q.update_neighbors_except([0, 1, 0], 1u8, None);
            },
            |q, event| match event {
                UpdateEvent::Simple { pos, .. } => {
                    visited_order.push(pos);
                    match world.get(&pos).copied().unwrap_or(Cell::Air) {
                        Cell::Wire { power } => {
                            let lever_sig = if matches!(
                                world.get(&[pos[0] - 1, pos[1], pos[2]]),
                                Some(Cell::Lever { powered: true })
                            ) {
                                15
                            } else {
                                0
                            };
                            let mut incoming = [0u8; 4];
                            for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
                                let npos = dir.relative_by(pos, 1);
                                if let Some(Cell::Wire { power: np }) = world.get(&npos) {
                                    incoming[i] = *np;
                                }
                            }
                            let target = redstone_wire_target_strength(lever_sig, incoming);
                            if target != power {
                                world.insert(pos, Cell::Wire { power: target });
                                wire_write_flags.push(update_flags::CLIENTS);
                                // Wire HashSet fan-out over {pos} U {pos.relative(d)}
                                q.update_neighbors_except(pos, 2u8, None);
                                q.neighbor_changed(pos, 2u8, false);
                            }
                        }
                        Cell::Door { open } => {
                            let mut sig = false;
                            for check_pos in [[pos[0], 1, 0], [pos[0], 2, 0]] {
                                for dir in Direction::ALL {
                                    let npos = dir.relative_by(check_pos, 1);
                                    if matches!(world.get(&npos), Some(Cell::Wire { power }) if *power > 0) {
                                        sig = true;
                                    }
                                }
                            }
                            if let Some((new_state, flags)) = powered_toggle_on_signal(open, sig) {
                                world.insert([pos[0], 1, 0], Cell::Door { open: new_state });
                                world.insert([pos[0], 2, 0], Cell::Door { open: new_state });
                                door_write_flags.push(flags);
                            }
                        }
                        _ => {}
                    }
                }
                UpdateEvent::Shape { .. } => {}
            },
        );

        assert!(!stats.overflowed);
        // First 6 neighbors from lever follow UPDATE_ORDER = W, E (wire at [1,1,0]) before depth-first fanout
        assert_eq!(visited_order[0], [-1, 1, 0]); // West first
        assert_eq!(visited_order[1], [1, 1, 0]);  // East hits wire(1) -> triggers wire(1) fan-out before DOWN of lever!
        // Final wire powers [15, 14, 13, 12, 11, 10, 9, 8]
        for x in 1..=8 {
            assert_eq!(world.get(&[x, 1, 0]), Some(&Cell::Wire { power: (16 - x) as u8 }));
        }
        assert_eq!(world.get(&[9, 1, 0]), Some(&Cell::Door { open: true }));
        assert_eq!(world.get(&[9, 2, 0]), Some(&Cell::Door { open: true }));
        assert!(wire_write_flags.iter().all(|&f| f == update_flags::CLIENTS));
        assert!(door_write_flags.iter().all(|&f| f == update_flags::CLIENTS));
    }

    /// Reproduces `local-20261114-1158-1-0-port-update-order` support-loss pop + chain cap.
    #[test]
    fn ladder_support_loss_and_chain_cap() {
        let mut updater = CollectingNeighborUpdater::new(1_000);
        let mut shape_order_seen = Vec::new();
        let mut ladder_popped = false;

        let stats = updater.drain_with(
            |q| {
                // Breaking stone wall at [0,64,0] with ladder on East face [1,64,0] facing East.
                let prop_flags = update_flags::propagated_shape_flags(update_flags::ALL);
                assert_eq!(prop_flags, update_flags::CLIENTS);
                for dir in UPDATE_SHAPE_ORDER {
                    let target = dir.relative_by([0, 64, 0], 1);
                    q.shape_update(dir.opposite(), 0u8, target, [0, 64, 0], prop_flags, 511);
                }
            },
            |_q, ev| {
                if let UpdateEvent::Shape { pos, direction, flags, recursion_left, .. } = ev {
                    shape_order_seen.push(pos);
                    if pos == [1, 64, 0] {
                        assert!(!ladder_update_shape_pops(Direction::East, Direction::Up, false));
                        assert!(ladder_update_shape_pops(Direction::East, direction, false));
                        let action = decide_update_or_destroy(true, true, flags, recursion_left);
                        assert_eq!(
                            action,
                            UpdateOrDestroyAction::Destroy {
                                drop_resources: true,
                                recursion_left: 511,
                            }
                        );
                        ladder_popped = true;
                    }
                }
            },
        );
        assert!(ladder_popped);
        assert!(!stats.overflowed);
        assert_eq!(
            shape_order_seen,
            UPDATE_SHAPE_ORDER.map(|d| d.relative_by([0, 64, 0], 1))
        );

        // Chained cap enforcement: max 3 drops the 4th update and records first_skipped_pos.
        let mut capped = CollectingNeighborUpdater::new(3);
        let cap_stats = capped.drain_with(
            |q| {
                q.update_neighbors_except([0, 0, 0], 1u8, None);
            },
            |q, _| {
                q.neighbor_changed([9, 9, 9], 1u8, false);
                q.neighbor_changed([10, 10, 10], 1u8, false);
                q.neighbor_changed([11, 11, 11], 1u8, false);
            },
        );
        assert!(cap_stats.overflowed);
        assert_eq!(cap_stats.first_skipped_pos, Some([11, 11, 11]));
    }
}
