//! Mojang 26.2 piston law (`PistonBaseBlock`, `PistonStructureResolver`,
//! `PistonMovingBlockEntity`, `PistonMath`, `MovingPistonBlock`).
//!
//! Ported from `lab/results/local-20261118-2327-1-0-port-piston-extend-retract-motion/dossier.md`
//! and refactored for zero heap allocations on `Resolver`, `trigger_event`, and slice-backed push sweeps.

pub use crate::neighbor::{update_flags as update, Direction};
use crate::{Aabb, BlockPos, Vec3};

/// `PushReaction`, in class declaration order (ordinals 0..=4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushReaction {
    Normal = 0,
    Destroy = 1,
    Block = 2,
    Ignore = 3,
    PushOnly = 4,
}

/// Block identities tested by `PistonBaseBlock.isPushable` and `PistonStructureResolver`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Other,
    Obsidian,
    CryingObsidian,
    RespawnAnchor,
    ReinforcedDeepslate,
    Piston { extended: bool },
    StickyPiston { extended: bool },
    PistonHead,
    MovingPiston,
    SlimeBlock,
    HoneyBlock,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PistonBlock {
    pub air: bool,
    pub destroy_speed: f32,
    pub has_block_entity: bool,
    pub push_reaction: PushReaction,
    pub kind: BlockKind,
}

impl PistonBlock {
    #[inline]
    pub const fn air() -> Self {
        Self {
            air: true,
            destroy_speed: 0.0,
            has_block_entity: false,
            push_reaction: PushReaction::Normal,
            kind: BlockKind::Other,
        }
    }

    /// An ordinary pushable full block (stone: destroy speed 1.5).
    #[inline]
    pub const fn pushable() -> Self {
        Self {
            air: false,
            destroy_speed: 1.5,
            has_block_entity: false,
            push_reaction: PushReaction::Normal,
            kind: BlockKind::Other,
        }
    }
}

pub trait PistonWorld {
    fn block(&self, position: BlockPos) -> PistonBlock;
    fn min_y(&self) -> i32 {
        -64
    }
    fn max_y(&self) -> i32 {
        319
    }
    fn within_world_border(&self, _position: BlockPos) -> bool {
        true
    }
}

/// `PistonStructureResolver.MAX_PUSH_DEPTH` and `bipush 12` comparisons:
/// 12 pushed blocks succeed, 13 fail.
pub const MAX_PUSH_DEPTH: usize = 12;

/// Upper bound on destroyed frontier blocks per resolve (bounded by `<= 1 + 4 * MAX_PUSH_DEPTH`).
pub const MAX_DESTROY_DEPTH: usize = 64;

/// `PistonBaseBlock.isPushable(state, level, pos, moveDirection, destroy, pistonDirection)`.
pub fn is_pushable(
    world: &impl PistonWorld,
    position: BlockPos,
    move_direction: Direction,
    destroy: bool,
    piston_direction: Direction,
) -> bool {
    let y = position[1];
    if y < world.min_y() || y > world.max_y() || !world.within_world_border(position) {
        return false;
    }
    let state = world.block(position);
    if state.air {
        return true;
    }
    match state.kind {
        BlockKind::Obsidian
        | BlockKind::CryingObsidian
        | BlockKind::RespawnAnchor
        | BlockKind::ReinforcedDeepslate => return false,
        BlockKind::Piston { extended: true } | BlockKind::StickyPiston { extended: true } => {
            return false
        }
        _ => {}
    }
    if move_direction == Direction::Down && y == world.min_y() {
        return false;
    }
    if move_direction == Direction::Up && y == world.max_y() {
        return false;
    }
    if state.destroy_speed == -1.0 {
        return false;
    }
    match state.push_reaction {
        PushReaction::Block => false,
        PushReaction::Destroy => destroy,
        PushReaction::PushOnly => move_direction == piston_direction,
        PushReaction::Normal | PushReaction::Ignore => !state.has_block_entity,
    }
}

#[inline]
const fn is_sticky(kind: BlockKind) -> bool {
    matches!(kind, BlockKind::SlimeBlock | BlockKind::HoneyBlock)
}

#[inline]
pub const fn can_stick_to_each_other(a: BlockKind, b: BlockKind) -> bool {
    let mixed = (matches!(a, BlockKind::HoneyBlock) && matches!(b, BlockKind::SlimeBlock))
        || (matches!(a, BlockKind::SlimeBlock) && matches!(b, BlockKind::HoneyBlock));
    !mixed && (is_sticky(a) || is_sticky(b))
}

/// Stack-allocated zero-heap `PistonStructureResolver` collecting `to_push` and `to_destroy`.
pub struct Resolver {
    piston_pos: BlockPos,
    piston_direction: Direction,
    extending: bool,
    push_direction: Direction,
    start_pos: BlockPos,
    to_push_buf: [BlockPos; MAX_PUSH_DEPTH],
    to_push_len: usize,
    to_destroy_buf: [BlockPos; MAX_DESTROY_DEPTH],
    to_destroy_len: usize,
}

impl Resolver {
    pub fn new(piston_pos: BlockPos, piston_direction: Direction, extending: bool) -> Self {
        let (push_direction, start_pos) = if extending {
            (piston_direction, piston_direction.relative_by(piston_pos, 1))
        } else {
            (
                piston_direction.opposite(),
                piston_direction.relative_by(piston_pos, 2),
            )
        };
        Self {
            piston_pos,
            piston_direction,
            extending,
            push_direction,
            start_pos,
            to_push_buf: [[0; 3]; MAX_PUSH_DEPTH],
            to_push_len: 0,
            to_destroy_buf: [[0; 3]; MAX_DESTROY_DEPTH],
            to_destroy_len: 0,
        }
    }

    #[inline]
    pub const fn push_direction(&self) -> Direction {
        self.push_direction
    }

    #[inline]
    pub fn to_push(&self) -> &[BlockPos] {
        &self.to_push_buf[..self.to_push_len]
    }

    #[inline]
    pub fn to_destroy(&self) -> &[BlockPos] {
        &self.to_destroy_buf[..self.to_destroy_len]
    }

    #[inline]
    fn push_block(&mut self, pos: BlockPos) -> bool {
        if self.to_push_len >= MAX_PUSH_DEPTH {
            return false;
        }
        self.to_push_buf[self.to_push_len] = pos;
        self.to_push_len += 1;
        true
    }

    #[inline]
    fn push_destroy(&mut self, pos: BlockPos) {
        if self.to_destroy_len < MAX_DESTROY_DEPTH {
            self.to_destroy_buf[self.to_destroy_len] = pos;
            self.to_destroy_len += 1;
        }
    }

    /// `PistonStructureResolver.resolve`
    pub fn resolve(&mut self, world: &impl PistonWorld) -> bool {
        self.to_push_len = 0;
        self.to_destroy_len = 0;
        let state = world.block(self.start_pos);
        if !is_pushable(
            world,
            self.start_pos,
            self.push_direction,
            false,
            self.piston_direction,
        ) {
            if self.extending && state.push_reaction == PushReaction::Destroy {
                self.push_destroy(self.start_pos);
                return true;
            }
            return false;
        }
        if !self.add_block_line(world, self.start_pos) {
            return false;
        }
        let mut i = 0;
        while i < self.to_push_len {
            let pos = self.to_push_buf[i];
            if is_sticky(world.block(pos).kind) && !self.add_branching_blocks(world, pos) {
                return false;
            }
            i += 1;
        }
        true
    }

    fn add_block_line(&mut self, world: &impl PistonWorld, front: BlockPos) -> bool {
        let mut state = world.block(front);
        if state.air {
            return true;
        }
        if !is_pushable(world, front, self.push_direction, false, self.piston_direction) {
            return true;
        }
        if front == self.piston_pos {
            return true;
        }
        if self.to_push().contains(&front) {
            return true;
        }
        let mut line = 1usize;
        if line + self.to_push_len > MAX_PUSH_DEPTH {
            return false;
        }
        let back = self.push_direction.opposite();
        while is_sticky(state.kind) {
            let behind = back.relative_by(front, line as i32);
            let previous = state;
            state = world.block(behind);
            if state.air {
                break;
            }
            if !can_stick_to_each_other(previous.kind, state.kind) {
                break;
            }
            if !is_pushable(world, behind, self.push_direction, false, back) {
                break;
            }
            if behind == self.piston_pos {
                break;
            }
            line += 1;
            if line + self.to_push_len > MAX_PUSH_DEPTH {
                return false;
            }
        }
        for offset in (0..line).rev() {
            if !self.push_block(back.relative_by(front, offset as i32)) {
                return false;
            }
        }
        let mut frontier = front;
        loop {
            frontier = self.push_direction.relative_by(frontier, 1);
            if let Some(seam) = self.to_push().iter().position(|p| *p == frontier) {
                self.reorder_list_at_collision(line, seam);
                let mut j = 0;
                while j <= seam + line {
                    let pos = self.to_push_buf[j];
                    if is_sticky(world.block(pos).kind) && !self.add_branching_blocks(world, pos) {
                        return false;
                    }
                    j += 1;
                }
                return true;
            }
            let frontier_state = world.block(frontier);
            if frontier_state.air {
                return true;
            }
            if !is_pushable(
                world,
                frontier,
                self.push_direction,
                true,
                self.push_direction,
            ) || frontier == self.piston_pos
            {
                return false;
            }
            if frontier_state.push_reaction == PushReaction::Destroy {
                self.push_destroy(frontier);
                return true;
            }
            if !self.push_block(frontier) {
                return false;
            }
        }
    }

    /// In-place `PistonStructureResolver.reorderListAtCollision(added, seam)`:
    /// `[0, seam) + [len-added, len) + [seam, len-added)` via `rotate_right(added)`.
    #[inline]
    fn reorder_list_at_collision(&mut self, added: usize, seam: usize) {
        self.to_push_buf[seam..self.to_push_len].rotate_right(added);
    }

    fn add_branching_blocks(&mut self, world: &impl PistonWorld, origin: BlockPos) -> bool {
        let state = world.block(origin);
        for direction in Direction::ALL {
            if direction.axis() == self.push_direction.axis() {
                continue;
            }
            let side = direction.relative_by(origin, 1);
            let side_state = world.block(side);
            if !can_stick_to_each_other(side_state.kind, state.kind) {
                continue;
            }
            if !self.add_block_line(world, side) {
                return false;
            }
        }
        true
    }
}

/// Exact `setBlock` flags at each piston call site in Mojang 26.2.
pub mod set_block_flags {
    use super::update;
    pub const EXTEND_BODY: i32 = update::MOVE_BY_PISTON | update::CLIENTS | update::NEIGHBORS; // 67
    pub const RETRACT_BODY_TO_MOVING: i32 =
        update::SKIP_BLOCK_ENTITY_SIDEEFFECTS | update::KNOWN_SHAPE | update::INVISIBLE; // 276
    pub const RETRACT_HEAD_TO_AIR: i32 =
        update::SKIP_BLOCK_ENTITY_SIDEEFFECTS | update::KNOWN_SHAPE | update::INVISIBLE; // 276
    pub const DESTROYED_TO_AIR: i32 = update::KNOWN_SHAPE | update::CLIENTS; // 18
    pub const PUSHED_TO_MOVING: i32 =
        update::SKIP_BLOCK_ENTITY_SIDEEFFECTS | update::MOVE_BY_PISTON | update::INVISIBLE; // 324
    pub const LEFTOVER_TO_AIR: i32 =
        update::MOVE_BY_PISTON | update::KNOWN_SHAPE | update::CLIENTS; // 82
    pub const FINAL_RESTORE: i32 = update::MOVE_BY_PISTON | update::CLIENTS | update::NEIGHBORS; // 67
    pub const FINAL_RESTORE_ORPHAN: i32 = update::SKIP_BLOCK_ENTITY_SIDEEFFECTS
        | update::MOVE_BY_PISTON
        | update::KNOWN_SHAPE
        | update::INVISIBLE; // 340
    pub const EVENT_CANCEL: i32 = update::CLIENTS; // 2
}

pub mod trigger {
    pub const EXTEND: i32 = 0;
    pub const CONTRACT: i32 = 1;
    pub const DROP: i32 = 2;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerView {
    pub neighbor_signal: bool,
    pub front_moving_piston: bool,
    pub front2_moving_piston_extending_same_direction: bool,
    pub front2_block: PistonBlock,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TriggerAction {
    MoveBlocks { extending: bool },
    SetBlock { flags: i32 },
    StickyPull,
    RemoveHead,
    FinalTick { position: BlockPos },
}

/// Inline zero-alloc sequence of `TriggerAction`s returned by `trigger_event`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerActions {
    items: [TriggerAction; 4],
    len: u8,
}

impl TriggerActions {
    #[inline]
    pub fn as_slice(&self) -> &[TriggerAction] {
        &self.items[..self.len as usize]
    }

    #[inline]
    fn push(&mut self, action: TriggerAction) {
        let i = self.len as usize;
        self.items[i] = action;
        self.len += 1;
    }
}

/// Server-side `PistonBaseBlock.triggerEvent`.
pub fn trigger_event(
    sticky: bool,
    facing: Direction,
    piston_pos: BlockPos,
    event: i32,
    view: TriggerView,
) -> Option<TriggerActions> {
    if view.neighbor_signal {
        if event == trigger::CONTRACT || event == trigger::DROP {
            let mut out = TriggerActions {
                items: [TriggerAction::SetBlock { flags: set_block_flags::EVENT_CANCEL }; 4],
                len: 1,
            };
            let _ = &mut out;
            return Some(out);
        }
    } else if event == trigger::EXTEND {
        return None;
    }
    let dummy = TriggerAction::RemoveHead;
    let mut actions = TriggerActions { items: [dummy; 4], len: 0 };
    match event {
        trigger::EXTEND => {
            actions.push(TriggerAction::MoveBlocks { extending: true });
            actions.push(TriggerAction::SetBlock {
                flags: set_block_flags::EXTEND_BODY,
            });
            Some(actions)
        }
        trigger::CONTRACT | trigger::DROP => {
            let head = facing.relative_by(piston_pos, 1);
            if view.front_moving_piston {
                actions.push(TriggerAction::FinalTick { position: head });
            }
            actions.push(TriggerAction::SetBlock {
                flags: set_block_flags::RETRACT_BODY_TO_MOVING,
            });
            if sticky {
                let front2 = facing.relative_by(head, 1);
                if view.front2_moving_piston_extending_same_direction {
                    actions.push(TriggerAction::FinalTick { position: front2 });
                } else if event == trigger::CONTRACT
                    && !view.front2_block.air
                    && is_pushable(&TriggerWorld(&view), front2, facing.opposite(), false, facing)
                    && (view.front2_block.push_reaction == PushReaction::Normal
                        || matches!(
                            view.front2_block.kind,
                            BlockKind::Piston { .. } | BlockKind::StickyPiston { .. }
                        ))
                {
                    actions.push(TriggerAction::StickyPull);
                } else {
                    actions.push(TriggerAction::RemoveHead);
                }
            } else {
                actions.push(TriggerAction::RemoveHead);
            }
            Some(actions)
        }
        _ => None,
    }
}

struct TriggerWorld<'a>(&'a TriggerView);
impl PistonWorld for TriggerWorld<'_> {
    fn block(&self, _position: BlockPos) -> PistonBlock {
        self.0.front2_block
    }
}

pub const TICKS_TO_EXTEND: i32 = 2;
pub const PUSH_OFFSET: f64 = crate::shapes::ENTITY_PUSH_PADDING;
pub const TICK_MOVEMENT: f64 = 0.51;
pub const PROGRESS_STEP: f32 = 0.5;
pub const STUCK_BOX_HEIGHT: f64 = 1.500001;

#[inline]
pub const fn extended_progress(extending: bool, progress: f32) -> f32 {
    if extending {
        progress - 1.0
    } else {
        1.0 - progress
    }
}

#[inline]
pub const fn movement_direction(extending: bool, direction: Direction) -> Direction {
    if extending {
        direction
    } else {
        direction.opposite()
    }
}

#[inline]
pub fn movement_area(box_: Aabb, direction: Direction, distance: f64) -> Aabb {
    box_.expand_axis(direction.axis(), distance * direction.axis_step() as f64)
}

#[inline]
pub fn get_movement(area: Aabb, direction: Direction, entity: Aabb) -> f64 {
    let axis = direction.axis();
    if direction.axis_step() > 0 {
        area.max.get(axis) - entity.min.get(axis)
    } else {
        entity.max.get(axis) - area.min.get(axis)
    }
}

#[inline]
pub fn box_at(
    position: BlockPos,
    box_: Aabb,
    extending: bool,
    direction: Direction,
    progress: f32,
) -> Aabb {
    let offset = extended_progress(extending, progress) as f64;
    box_.moved(Vec3::block_corner(position).plus(direction.vector(offset)))
}

#[derive(Clone, Copy, Debug)]
pub struct PistonEntity {
    pub aabb: Aabb,
    pub position: Vec3,
    pub delta_movement: Vec3,
    pub push_reaction: PushReaction,
    pub on_ground: bool,
    pub server_player: bool,
    pub supported_by_block: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push {
    pub entity: usize,
    pub direction: Direction,
    pub distance: f64,
    pub velocity_snap: Option<Vec3>,
}

pub(crate) struct Stroke {
    pub position: BlockPos,
    pub direction: Direction,
    pub extending: bool,
    pub progress: f32,
    pub distance: f64,
}

impl Stroke {
    pub fn movement(&self) -> Direction {
        movement_direction(self.extending, self.direction)
    }

    pub fn bounds(&self, local: Aabb) -> Aabb {
        box_at(self.position, local, self.extending, self.direction, self.progress)
    }

    pub fn swept(&self, bounds: Aabb) -> Aabb {
        movement_area(bounds, self.movement(), self.distance)
    }

    pub fn push(
        &self,
        index: usize,
        entity: &mut PistonEntity,
        areas: impl IntoIterator<Item = Aabb>,
        slime: bool,
        source: bool,
        out: &mut Vec<Push>,
    ) {
        if entity.push_reaction == PushReaction::Ignore || (slime && entity.server_player) {
            return;
        }
        let movement = self.movement();
        let velocity_snap = slime.then(|| {
            entity.delta_movement = entity.delta_movement.with(movement.axis(), movement.axis_step() as f64);
            entity.delta_movement
        });
        let mut overlap: f64 = 0.0;
        for area in areas {
            if entity.aabb.intersects(area) {
                overlap = overlap.max(get_movement(area, movement, entity.aabb));
                if overlap >= self.distance {
                    break;
                }
            }
        }
        if overlap <= 0.0 && velocity_snap.is_none() {
            return;
        }
        let distance = if overlap <= 0.0 { 0.0 } else { overlap.min(self.distance) + PUSH_OFFSET };
        out.push(Push { entity: index, direction: movement, distance, velocity_snap });
        if overlap > 0.0 && !self.extending && source {
            if let Some(distance) = fix_entity_within_piston_base(self.position, &entity.aabb, movement, self.distance) {
                out.push(Push { entity: index, direction: movement.opposite(), distance, velocity_snap: None });
            }
        }
    }
}

/// Zero-alloc variant writing into caller's reusable buffer `out`.
pub fn move_collided_entities_into(
    position: BlockPos,
    boxes: &[Aabb],
    entities: &mut [PistonEntity],
    extending: bool,
    direction: Direction,
    is_source_piston: bool,
    old_progress: f32,
    new_progress: f32,
    slime_moved_state: bool,
    out: &mut Vec<Push>,
) {
    out.clear();
    if boxes.is_empty() {
        return;
    }
    let stroke = Stroke { position, direction, extending, progress: old_progress, distance: (new_progress - old_progress) as f64 };
    for (index, entity) in entities.iter_mut().enumerate() {
        let areas = boxes.iter().map(|&local| stroke.swept(stroke.bounds(local)));
        stroke.push(index, entity, areas, slime_moved_state, is_source_piston, out);
    }
}

pub fn move_collided_entities(
    position: BlockPos,
    boxes: &[Aabb],
    entities: &mut [PistonEntity],
    extending: bool,
    direction: Direction,
    is_source_piston: bool,
    old_progress: f32,
    new_progress: f32,
    slime_moved_state: bool,
) -> Vec<Push> {
    let mut out = Vec::new();
    move_collided_entities_into(
        position,
        boxes,
        entities,
        extending,
        direction,
        is_source_piston,
        old_progress,
        new_progress,
        slime_moved_state,
        &mut out,
    );
    out
}

pub fn fix_entity_within_piston_base(
    position: BlockPos,
    entity_aabb: &Aabb,
    movement: Direction,
    delta: f64,
) -> Option<f64> {
    let base = Aabb::UNIT.moved(Vec3::block_corner(position));
    if !entity_aabb.intersects(base) {
        return None;
    }
    let back = movement.opposite();
    let intersection = entity_aabb.overlap(base);
    let to_face = get_movement(base, back, *entity_aabb) + PUSH_OFFSET;
    let through = get_movement(base, back, intersection) + PUSH_OFFSET;
    if (to_face - through).abs() < PUSH_OFFSET {
        Some(to_face.min(delta) + PUSH_OFFSET)
    } else {
        None
    }
}

#[inline]
pub fn stuck_entity(
    honey_moved_state: bool,
    horizontal: bool,
    stuck_box: Aabb,
    entity: &PistonEntity,
) -> bool {
    if !honey_moved_state || !horizontal {
        return false;
    }
    entity.push_reaction == PushReaction::Normal
        && entity.on_ground
        && (entity.supported_by_block
            || (entity.position.x >= stuck_box.min.x
                && entity.position.x <= stuck_box.max.x
                && entity.position.z >= stuck_box.min.z
                && entity.position.z <= stuck_box.max.z))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovingPiston {
    pub direction: Direction,
    pub extending: bool,
    pub is_source_piston: bool,
    pub progress: f32,
    pub progress_o: f32,
    pub death_ticks: i32,
}

impl MovingPiston {
    pub const fn new(direction: Direction, extending: bool, is_source_piston: bool) -> Self {
        Self {
            direction,
            extending,
            is_source_piston,
            progress: 0.0,
            progress_o: 0.0,
            death_ticks: 0,
        }
    }

    #[inline]
    pub fn next_progress(&self) -> f32 {
        crate::basis::clock::advance_progress(self.progress, PROGRESS_STEP, 1.0).0.min(1.0)
    }

    pub fn advance(&mut self, client_side: bool) -> bool {
        self.progress_o = self.progress;
        if self.progress_o >= 1.0 {
            if client_side && self.death_ticks < 5 {
                self.death_ticks += 1;
                return false;
            }
            return true;
        }
        self.progress = self.next_progress();
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct PlainWorld {
        blocks: HashMap<BlockPos, PistonBlock>,
    }
    impl PlainWorld {
        fn line(dir: Direction, from: BlockPos, n: i32, block: PistonBlock) -> Self {
            let mut blocks = HashMap::new();
            for i in 1..=n {
                blocks.insert(dir.relative_by(from, i), block);
            }
            Self { blocks }
        }
    }
    impl PistonWorld for PlainWorld {
        fn block(&self, position: BlockPos) -> PistonBlock {
            *self.blocks.get(&position).unwrap_or(&PistonBlock::air())
        }
    }

    fn stone() -> PistonBlock {
        PistonBlock::pushable()
    }
    fn slime() -> PistonBlock {
        PistonBlock { kind: BlockKind::SlimeBlock, ..stone() }
    }

    #[test]
    fn push_limit_twelve_succeeds_thirteen_fails() {
        let piston = [0, 64, 0];
        let world12 = PlainWorld::line(Direction::East, piston, 12, stone());
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(resolver.resolve(&world12));
        assert_eq!(resolver.to_push().len(), 12);
        assert!(resolver.to_destroy().is_empty());

        let world13 = PlainWorld::line(Direction::East, piston, 13, stone());
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(!resolver.resolve(&world13), "13 blocks must fail");
    }

    #[test]
    fn destroy_reaction_only_at_frontier() {
        let piston = [0, 64, 0];
        let torch = PistonBlock {
            air: false,
            destroy_speed: 0.0,
            has_block_entity: false,
            push_reaction: PushReaction::Destroy,
            kind: BlockKind::Other,
        };
        let mut world = PlainWorld::line(Direction::East, piston, 1, stone());
        world.blocks.insert([2, 64, 0], torch);
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(resolver.resolve(&world));
        assert_eq!(resolver.to_push(), &[[1, 64, 0]]);
        assert_eq!(resolver.to_destroy(), &[[2, 64, 0]]);
    }

    #[test]
    fn slime_honey_do_not_stick() {
        assert!(!can_stick_to_each_other(BlockKind::SlimeBlock, BlockKind::HoneyBlock));
        assert!(!can_stick_to_each_other(BlockKind::HoneyBlock, BlockKind::SlimeBlock));
        assert!(can_stick_to_each_other(BlockKind::SlimeBlock, BlockKind::SlimeBlock));
        assert!(can_stick_to_each_other(BlockKind::HoneyBlock, BlockKind::Other));

        let piston = [0, 64, 0];
        let mut world = PlainWorld::line(Direction::East, piston, 1, slime());
        world.blocks.insert([2, 64, 0], PistonBlock { kind: BlockKind::HoneyBlock, ..stone() });
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(resolver.resolve(&world));
        assert_eq!(resolver.to_push(), &[[1, 64, 0], [2, 64, 0]]);

        let mut side = PlainWorld::line(Direction::East, piston, 1, slime());
        side.blocks.insert([1, 64, 1], PistonBlock { kind: BlockKind::HoneyBlock, ..stone() });
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(resolver.resolve(&side));
        assert_eq!(resolver.to_push(), &[[1, 64, 0]]);
    }

    #[test]
    fn sticky_chain_pulls_behind() {
        let piston = [0, 64, 0];
        let mut world = PlainWorld::line(Direction::East, piston, 1, slime());
        world.blocks.insert([1, 64, -1], slime());
        world.blocks.insert([0, 64, -1], stone());
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(resolver.resolve(&world));
        assert_eq!(resolver.to_push(), &[[1, 64, 0], [0, 64, -1], [1, 64, -1]]);
    }

    #[test]
    fn sticky_branches_drag_sides() {
        let piston = [0, 64, 0];
        let mut world = PlainWorld::line(Direction::East, piston, 1, slime());
        world.blocks.insert([1, 65, 0], stone());
        let mut resolver = Resolver::new(piston, Direction::East, true);
        assert!(resolver.resolve(&world));
        assert_eq!(resolver.to_push().len(), 2);
        assert!(resolver.to_push().contains(&[1, 64, 0]));
        assert!(resolver.to_push().contains(&[1, 65, 0]));
    }

    #[test]
    fn collision_reorder_moves_added_line_before_middle() {
        let mut resolver = Resolver::new([0, 64, 0], Direction::East, true);
        let (a, b, c, d) = ([1, 64, 0], [2, 64, 0], [3, 64, 0], [4, 64, 0]);
        resolver.push_block(a);
        resolver.push_block(b);
        resolver.push_block(c);
        resolver.push_block(d);
        resolver.reorder_list_at_collision(2, 1);
        assert_eq!(resolver.to_push(), &[a, c, d, b]);
    }

    #[test]
    fn constants_match_the_jar() {
        assert_eq!(set_block_flags::EXTEND_BODY, 67);
        assert_eq!(set_block_flags::RETRACT_BODY_TO_MOVING, 276);
        assert_eq!(set_block_flags::RETRACT_HEAD_TO_AIR, 276);
        assert_eq!(set_block_flags::DESTROYED_TO_AIR, 18);
        assert_eq!(set_block_flags::PUSHED_TO_MOVING, 324);
        assert_eq!(set_block_flags::LEFTOVER_TO_AIR, 82);
        assert_eq!(set_block_flags::FINAL_RESTORE, 67);
        assert_eq!(set_block_flags::FINAL_RESTORE_ORPHAN, 340);
        assert_eq!(set_block_flags::EVENT_CANCEL, 2);
        assert_eq!(MAX_PUSH_DEPTH, 12);
        assert_eq!(TICKS_TO_EXTEND, 2);
        assert_eq!(PUSH_OFFSET, 0.01);
        assert_eq!(TICK_MOVEMENT, 0.51);
        assert_eq!(STUCK_BOX_HEIGHT, 1.500001);
        assert_eq!(PROGRESS_STEP, 0.5);
    }

    #[test]
    fn trigger_event_contract_and_drop() {
        let view = TriggerView {
            neighbor_signal: false,
            front_moving_piston: false,
            front2_moving_piston_extending_same_direction: false,
            front2_block: PistonBlock::air(),
        };
        let actions = trigger_event(true, Direction::East, [0, 64, 0], trigger::CONTRACT, view).unwrap();
        assert_eq!(actions.as_slice()[0], TriggerAction::SetBlock { flags: 276 });
        assert!(actions.as_slice().contains(&TriggerAction::RemoveHead));

        let pull_view = TriggerView { front2_block: stone(), ..view };
        let actions = trigger_event(true, Direction::East, [0, 64, 0], trigger::CONTRACT, pull_view).unwrap();
        assert!(actions.as_slice().contains(&TriggerAction::StickyPull));

        let drop_actions = trigger_event(true, Direction::East, [0, 64, 0], trigger::DROP, pull_view).unwrap();
        assert!(!drop_actions.as_slice().contains(&TriggerAction::StickyPull));
        assert!(drop_actions.as_slice().contains(&TriggerAction::RemoveHead));

        assert_eq!(trigger_event(true, Direction::East, [0, 64, 0], trigger::EXTEND, view), None);
        let back = trigger_event(
            true,
            Direction::East,
            [0, 64, 0],
            trigger::CONTRACT,
            TriggerView { neighbor_signal: true, ..view },
        )
        .unwrap();
        assert_eq!(back.as_slice(), &[TriggerAction::SetBlock { flags: 2 }]);

        let non_sticky = trigger_event(false, Direction::East, [0, 64, 0], trigger::CONTRACT, pull_view).unwrap();
        assert!(!non_sticky.as_slice().contains(&TriggerAction::StickyPull));
    }

    #[test]
    fn progress_steps_half_half() {
        let mut be = MovingPiston::new(Direction::East, true, true);
        assert_eq!(be.progress, 0.0);
        assert_eq!(be.next_progress(), 0.5);
        assert!(!be.advance(true));
        assert_eq!(be.progress, 0.5);
        assert!(!be.advance(true));
        assert_eq!(be.progress, 1.0);
        for i in 0..5 {
            assert!(!be.advance(true), "client keeps entity, tick {i}");
            assert_eq!(be.death_ticks, i + 1);
        }
        assert!(be.advance(true));

        let mut server = MovingPiston::new(Direction::East, true, true);
        server.advance(false);
        server.advance(false);
        assert!(server.advance(false));
    }

    fn entity(box_: Aabb, delta: Vec3, server_player: bool) -> PistonEntity {
        let position = Vec3::new(
            (box_.min.x + box_.max.x) / 2.0,
            box_.min.y,
            (box_.min.z + box_.max.z) / 2.0,
        );
        PistonEntity {
            aabb: box_,
            position,
            delta_movement: delta,
            push_reaction: PushReaction::Normal,
            on_ground: true,
            server_player,
            supported_by_block: true,
        }
    }

    #[test]
    fn slime_snap_and_push_distance() {
        let position = [5, 65, 5];
        let full = Aabb { min: Vec3::new(0.0, 0.0, 0.0), max: Vec3::new(1.0, 1.0, 1.0) };
        let entity_box = Aabb { min: Vec3::new(5.2, 65.0, 5.2), max: Vec3::new(5.8, 66.8, 5.8) };
        let mut entities = vec![entity(entity_box, Vec3::new(0.3, -0.08, 0.2), false)];
        let pushes = move_collided_entities(
            position,
            &[full],
            &mut entities,
            true,
            Direction::Up,
            false,
            0.0,
            0.5,
            true,
        );
        assert_eq!(pushes.len(), 1);
        assert_eq!(pushes[0].direction, Direction::Up);
        assert_eq!(pushes[0].distance, 0.5 + PUSH_OFFSET);
        let snapped = pushes[0].velocity_snap.unwrap();
        assert_eq!(snapped.y, 1.0);
        assert_eq!(snapped.x, 0.3);
        assert_eq!(snapped.z, 0.2);

        let mut player = vec![entity(entity_box, Vec3::default(), true)];
        let pushes = move_collided_entities(
            position, &[full], &mut player, true, Direction::Up, false, 0.0, 0.5, true,
        );
        assert!(pushes.is_empty());

        let mut plain = vec![entity(entity_box, Vec3::default(), true)];
        let pushes = move_collided_entities(
            position, &[full], &mut plain, true, Direction::Up, false, 0.0, 0.5, false,
        );
        assert_eq!(pushes.len(), 1);
        assert_eq!(pushes[0].distance, 0.5 + PUSH_OFFSET);
        assert!(pushes[0].velocity_snap.is_none());
    }

    #[test]
    fn sweep_arithmetic_and_extended_progress() {
        let box_ = Aabb { min: Vec3::new(0.0, 0.0, 0.0), max: Vec3::new(1.0, 1.0, 1.0) };
        let area = movement_area(box_, Direction::East, 0.5);
        assert_eq!(area.min.x, 0.0);
        assert_eq!(area.max.x, 1.5);
        let entity_box = Aabb { min: Vec3::new(1.2, 0.0, 0.0), max: Vec3::new(1.7, 1.0, 1.0) };
        assert_eq!(get_movement(area, Direction::East, entity_box), 1.5 - 1.2);
        let west = movement_area(box_, Direction::West, 0.5);
        assert_eq!(west.min.x, -0.5);
        assert_eq!(west.max.x, 1.0);

        assert_eq!(extended_progress(true, 0.0), -1.0);
        assert_eq!(extended_progress(true, 0.5), -0.5);
        assert_eq!(extended_progress(false, 0.0), 1.0);
        assert_eq!(extended_progress(false, 1.0), 0.0);
    }

    #[test]
    fn fix_within_base_squeeze_and_honey_stuck() {
        let position = [5, 64, 5];
        let tight = Aabb { min: Vec3::new(5.0, 64.0, 5.0), max: Vec3::new(6.0, 65.0, 6.0) };
        assert_eq!(
            fix_entity_within_piston_base(position, &tight, Direction::West, 0.5),
            Some(0.5 + PUSH_OFFSET)
        );
        let edge = Aabb { min: Vec3::new(6.0, 64.0, 5.0), max: Vec3::new(7.0, 65.0, 6.0) };
        assert_eq!(fix_entity_within_piston_base(position, &edge, Direction::West, 0.5), None);

        let box_ = Aabb { min: Vec3::new(5.0, 65.0, 5.0), max: Vec3::new(6.0, 66.500001, 6.0) };
        let on_it = entity(
            Aabb { min: Vec3::new(5.2, 66.5, 5.2), max: Vec3::new(5.8, 68.3, 5.8) },
            Vec3::default(),
            false,
        );
        assert!(stuck_entity(true, true, box_, &on_it));
        let mut aside = entity(
            Aabb { min: Vec3::new(7.2, 66.5, 5.2), max: Vec3::new(7.8, 68.3, 5.8) },
            Vec3::default(),
            false,
        );
        aside.supported_by_block = false;
        assert!(!stuck_entity(true, true, box_, &aside));
        assert!(!stuck_entity(false, true, box_, &on_it));
        assert!(!stuck_entity(true, false, box_, &on_it));
    }
}
