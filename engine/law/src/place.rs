//! Block placement state laws (26.2): slabs, stairs, trapdoors, doors, ladders, scaffolding.
//!
//! Block identity is passed as opaque `BlockId` / table properties; no block name
//! branches appear in code.

use crate::interact::{nearest_looking_directions, BlockId, Direction, DoorHalf};
use crate::{BlockPos, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct PlaceContext {
    pub clicked_pos: BlockPos,
    pub clicked_face: Direction,
    pub click_location: Vec3,
    pub replace_clicked: bool,
    pub is_inside: bool,
    pub is_sneaking: bool,
    pub player_look_direction: Option<Direction>,
    pub player_pitch_yaw: (f32, f32),
    pub held_block_id: BlockId,
    pub is_client_side: bool,
}

impl PlaceContext {
    /// `UseOnContext.getHorizontalDirection` (`@0-20`): null player -> `NORTH`.
    pub fn horizontal_direction(&self) -> Direction {
        self.player_look_direction.unwrap_or(Direction::North)
    }

    fn click_y_offset(&self) -> f64 {
        self.click_location.y - self.clicked_pos[1] as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlabType {
    Top,
    Bottom,
    Double,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlabState {
    pub block_id: BlockId,
    pub slab_type: SlabType,
    pub waterlogged: bool,
}

/// `SlabBlock.getStateForPlacement` (`@0-156`): merges to `DOUBLE` with `waterlogged = false`
/// forced when the clicked cell already matches `held_block_id`.
pub fn slab_get_state_for_placement(
    ctx: &PlaceContext,
    existing_block_id: BlockId,
    cell_is_water: bool,
) -> SlabState {
    if existing_block_id == ctx.held_block_id {
        return SlabState {
            block_id: ctx.held_block_id,
            slab_type: SlabType::Double,
            waterlogged: false,
        };
    }
    let is_top = ctx.clicked_face == Direction::Down
        || (ctx.clicked_face != Direction::Up && ctx.click_y_offset() > 0.5);
    SlabState {
        block_id: ctx.held_block_id,
        slab_type: if is_top { SlabType::Top } else { SlabType::Bottom },
        waterlogged: cell_is_water,
    }
}

/// `SlabBlock.canBeReplaced` (`@0-150`).
pub fn slab_can_be_replaced(state: SlabState, ctx: &PlaceContext) -> bool {
    if state.slab_type == SlabType::Double || ctx.held_block_id != state.block_id {
        return false;
    }
    if !ctx.replace_clicked {
        return true;
    }
    let flag = ctx.click_y_offset() > 0.5;
    match state.slab_type {
        SlabType::Bottom => ctx.clicked_face == Direction::Up || (flag && ctx.clicked_face.is_horizontal()),
        SlabType::Top => ctx.clicked_face == Direction::Down || (!flag && ctx.clicked_face.is_horizontal()),
        SlabType::Double => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StairShape {
    Straight,
    InnerLeft,
    InnerRight,
    OuterLeft,
    OuterRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StairState {
    pub facing: Direction,
    pub half: Half,
    pub shape: StairShape,
    pub waterlogged: bool,
}

/// `StairBlock.canTakeShape` (`@0-62`).
pub fn stair_can_take_shape(
    neighbor_stair: Option<StairState>,
    facing: Direction,
    half: Half,
) -> bool {
    neighbor_stair.is_none_or(|s| s.facing != facing || s.half != half)
}

/// `StairBlock.getStairsShape` (`@0-222`).
pub fn get_stairs_shape(
    facing: Direction,
    half: Half,
    pos: BlockPos,
    get_stair: impl Fn(BlockPos) -> Option<StairState>,
) -> StairShape {
    if let Some(outer) = get_stair(facing.relative(pos)) {
        if outer.half == half
            && outer.facing.axis() != facing.axis()
            && stair_can_take_shape(get_stair(outer.facing.opposite().relative(pos)), facing, half)
        {
            return if outer.facing == facing.counter_clock_wise() {
                StairShape::OuterLeft
            } else {
                StairShape::OuterRight
            };
        }
    }
    if let Some(inner) = get_stair(facing.opposite().relative(pos)) {
        if inner.half == half
            && inner.facing.axis() != facing.axis()
            && stair_can_take_shape(get_stair(inner.facing.relative(pos)), facing, half)
        {
            return if inner.facing == facing.counter_clock_wise() {
                StairShape::InnerLeft
            } else {
                StairShape::InnerRight
            };
        }
    }
    StairShape::Straight
}

/// `StairBlock.getStateForPlacement` (`@0-132`).
pub fn stair_get_state_for_placement(
    ctx: &PlaceContext,
    cell_is_water: bool,
    get_stair: impl Fn(BlockPos) -> Option<StairState>,
) -> StairState {
    let facing = ctx.horizontal_direction();
    let half = if ctx.clicked_face == Direction::Down
        || (ctx.clicked_face != Direction::Up && ctx.click_y_offset() > 0.5)
    {
        Half::Top
    } else {
        Half::Bottom
    };
    let shape = get_stairs_shape(facing, half, ctx.clicked_pos, get_stair);
    StairState {
        facing,
        half,
        shape,
        waterlogged: cell_is_water,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrapDoorPlaceState {
    pub facing: Direction,
    pub half: Half,
    pub open: bool,
    pub powered: bool,
    pub waterlogged: bool,
}

/// `TrapDoorBlock.getStateForPlacement` (`@0-204`).
pub fn trapdoor_get_state_for_placement(
    ctx: &PlaceContext,
    cell_is_water: bool,
    has_neighbor_signal: bool,
) -> TrapDoorPlaceState {
    let (facing, half) = if !ctx.replace_clicked && ctx.clicked_face.is_horizontal() {
        let half = if ctx.click_y_offset() > 0.5 {
            Half::Top
        } else {
            Half::Bottom
        };
        (ctx.clicked_face, half)
    } else {
        let facing = ctx.horizontal_direction().opposite();
        let half = if ctx.clicked_face == Direction::Up {
            Half::Bottom
        } else {
            Half::Top
        };
        (facing, half)
    };
    TrapDoorPlaceState {
        facing,
        half,
        open: has_neighbor_signal,
        powered: has_neighbor_signal,
        waterlogged: cell_is_water,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorHinge {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoorPlaceState {
    pub block_id: BlockId,
    pub facing: Direction,
    pub half: DoorHalf,
    pub hinge: DoorHinge,
    pub open: bool,
    pub powered: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DoorCellInfo {
    pub is_full_collision: bool,
    pub lower_door: bool,
}

/// `DoorBlock.getHinge` (`@0-393`).
pub fn door_get_hinge(
    ctx: &PlaceContext,
    dir: Direction,
    get_cell: impl Fn(BlockPos) -> DoorCellInfo,
) -> DoorHinge {
    let pos = ctx.clicked_pos;
    let ccw = dir.counter_clock_wise();
    let left_lower = ccw.relative(pos);
    let left_upper = Direction::Up.relative(left_lower);
    let cw = dir.clock_wise();
    let right_lower = cw.relative(pos);
    let right_upper = Direction::Up.relative(right_lower);

    let ll = get_cell(left_lower);
    let lu = get_cell(left_upper);
    let rl = get_cell(right_lower);
    let ru = get_cell(right_upper);

    let score = -(ll.is_full_collision as i32)
        - (lu.is_full_collision as i32)
        + (rl.is_full_collision as i32)
        + (ru.is_full_collision as i32);

    if (ll.lower_door && !rl.lower_door) || score > 0 {
        DoorHinge::Right
    } else if (rl.lower_door && !ll.lower_door) || score < 0 {
        DoorHinge::Left
    } else {
        let [sx, _, sz] = dir.step();
        let dx = ctx.click_location.x - pos[0] as f64;
        let dz = ctx.click_location.z - pos[2] as f64;
        if (sx < 0 && dz < 0.5) || (sx > 0 && dz > 0.5) || (sz < 0 && dx > 0.5) || (sz > 0 && dx < 0.5) {
            DoorHinge::Right
        } else {
            DoorHinge::Left
        }
    }
}

/// `DoorBlock.getStateForPlacement` (`@0-135`).
pub fn door_get_state_for_placement(
    ctx: &PlaceContext,
    max_y: i32,
    above_replaceable: bool,
    has_neighbor_signal_at: impl Fn(BlockPos) -> bool,
    get_cell: impl Fn(BlockPos) -> DoorCellInfo,
) -> Option<DoorPlaceState> {
    let pos = ctx.clicked_pos;
    if pos[1] >= max_y || !above_replaceable {
        return None;
    }
    let powered = has_neighbor_signal_at(pos) || has_neighbor_signal_at(Direction::Up.relative(pos));
    let facing = ctx.horizontal_direction();
    let hinge = door_get_hinge(ctx, facing, get_cell);
    Some(DoorPlaceState {
        block_id: ctx.held_block_id,
        facing,
        half: DoorHalf::Lower,
        hinge,
        open: powered,
        powered,
    })
}

/// `DoorBlock.setPlacedBy` (`@0-22`): writes `HALF = UPPER` at `pos.above()` with `flags = 3`.
pub fn door_set_placed_by(lower: DoorPlaceState, pos: BlockPos) -> (BlockPos, DoorPlaceState, u8) {
    (
        Direction::Up.relative(pos),
        DoorPlaceState {
            half: DoorHalf::Upper,
            ..lower
        },
        3,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LadderPlaceState {
    pub facing: Direction,
    pub waterlogged: bool,
}

/// `LadderBlock.canAttachTo` (`@0-17`) and `LadderBlock.canSurvive` (`@0-28`).
pub fn ladder_can_survive(
    pos: BlockPos,
    facing: Direction,
    is_face_sturdy_at: impl Fn(BlockPos, Direction) -> bool,
) -> bool {
    facing.is_horizontal() && is_face_sturdy_at(facing.opposite().relative(pos), facing)
}

/// `LadderBlock.getStateForPlacement` (`@0-176`).
pub fn ladder_get_state_for_placement(
    ctx: &PlaceContext,
    behind_ladder_facing: Option<Direction>,
    cell_is_water: bool,
    is_face_sturdy_at: impl Fn(BlockPos, Direction) -> bool,
) -> Option<LadderPlaceState> {
    if !ctx.replace_clicked && behind_ladder_facing == Some(ctx.clicked_face) {
        return None;
    }
    let dirs = nearest_looking_directions(
        ctx.player_pitch_yaw.0,
        ctx.player_pitch_yaw.1,
        ctx.clicked_face,
        ctx.replace_clicked,
    );
    for d in dirs {
        if d.is_horizontal() && ladder_can_survive(ctx.clicked_pos, d.opposite(), &is_face_sturdy_at) {
            return Some(LadderPlaceState {
                facing: d.opposite(),
                waterlogged: cell_is_water,
            });
        }
    }
    None
}

pub const SCAFFOLDING_MAX_DISTANCE: u8 = 7;
pub const SCAFFOLDING_TICK_DELAY: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScaffoldingCell {
    pub is_scaffolding: bool,
    pub distance: u8,
    pub sturdy_up: bool,
    pub replaceable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaffoldingState {
    pub distance: u8,
    pub bottom: bool,
    pub waterlogged: bool,
}

/// `ScaffoldingBlock.getDistance` (`@0-161`).
pub fn scaffolding_get_distance(
    pos: BlockPos,
    get_cell: impl Fn(BlockPos) -> ScaffoldingCell,
) -> u8 {
    let below = get_cell(Direction::Down.relative(pos));
    if below.is_scaffolding {
        return below.distance;
    }
    if below.sturdy_up {
        return 0;
    }
    let mut dist = SCAFFOLDING_MAX_DISTANCE;
    for d in [Direction::North, Direction::South, Direction::West, Direction::East] {
        let neighbor = get_cell(d.relative(pos));
        if neighbor.is_scaffolding {
            dist = dist.min(neighbor.distance.saturating_add(1));
            if dist == 1 {
                break;
            }
        }
    }
    dist
}

/// `ScaffoldingBlock.isBottom` (`@0-26`).
pub fn scaffolding_is_bottom(dist: u8, below_is_scaffolding: bool) -> bool {
    dist > 0 && !below_is_scaffolding
}

/// `ScaffoldingBlock.canSurvive` (`@0-15`).
pub fn scaffolding_can_survive(dist: u8) -> bool {
    dist < SCAFFOLDING_MAX_DISTANCE
}

/// `ScaffoldingBlock.getStateForPlacement` (`@0-78`): never returns `None` and schedules no tick.
pub fn scaffolding_get_state_for_placement(
    pos: BlockPos,
    cell_is_water: bool,
    get_cell: impl Fn(BlockPos) -> ScaffoldingCell,
) -> ScaffoldingState {
    let dist = scaffolding_get_distance(pos, &get_cell);
    let below_is_scaffolding = get_cell(Direction::Down.relative(pos)).is_scaffolding;
    ScaffoldingState {
        distance: dist,
        bottom: scaffolding_is_bottom(dist, below_is_scaffolding),
        waterlogged: cell_is_water,
    }
}

/// `ScaffoldingBlockItem.updatePlacementContext` (`@0-237`).
pub fn scaffolding_update_placement_context(
    ctx: &PlaceContext,
    in_world_bounds: impl Fn(BlockPos) -> bool,
    get_cell: impl Fn(BlockPos) -> ScaffoldingCell,
) -> Option<(BlockPos, Direction)> {
    let clicked = get_cell(ctx.clicked_pos);
    if !clicked.is_scaffolding {
        return (scaffolding_get_distance(ctx.clicked_pos, &get_cell) != SCAFFOLDING_MAX_DISTANCE)
            .then_some((ctx.clicked_pos, ctx.clicked_face));
    }
    let dir = if ctx.is_sneaking {
        if ctx.is_inside {
            ctx.clicked_face.opposite()
        } else {
            ctx.clicked_face
        }
    } else if ctx.clicked_face == Direction::Up {
        ctx.horizontal_direction()
    } else {
        Direction::Up
    };

    let mut counter = 0u8;
    let mut cursor = dir.relative(ctx.clicked_pos);
    loop {
        if !ctx.is_client_side && !in_world_bounds(cursor) {
            return None;
        }
        let c = get_cell(cursor);
        if !c.is_scaffolding {
            return c.replaceable.then_some((cursor, dir));
        }
        cursor = dir.relative(cursor);
        if dir.is_horizontal() {
            counter += 1;
            if counter >= SCAFFOLDING_MAX_DISTANCE {
                return None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldingCollision {
    Empty,
    Stable,
    BelowLip,
}

/// `ScaffoldingBlock.getCollisionShape` (`@0-97`).
pub fn scaffolding_collision_shape(
    state: ScaffoldingState,
    feet_y: f64,
    block_y: i32,
    descending: bool,
    is_placement: bool,
) -> ScaffoldingCollision {
    if is_placement {
        return ScaffoldingCollision::Empty;
    }
    if feet_y > (block_y as f64) + 1.0 - (1.0e-5f32 as f64) && !descending {
        ScaffoldingCollision::Stable
    } else if state.distance != 0
        && state.bottom
        && feet_y > (block_y as f64) - (1.0e-5f32 as f64)
    {
        ScaffoldingCollision::BelowLip
    } else {
        ScaffoldingCollision::Empty
    }
}

#[cfg(test)]
mod part2_tests {
    use super::*;
    use std::collections::HashMap;

    const AIR_ID: BlockId = BlockId(0);
    const OAK_SLAB_ID: BlockId = BlockId(1);
    const STONE_SLAB_ID: BlockId = BlockId(2);
    const OAK_TRAPDOOR_ID: BlockId = BlockId(3);
    const STONE_STAIRS_ID: BlockId = BlockId(4);

    fn ctx_at(
        pos: BlockPos,
        face: Direction,
        click_y: f64,
        replace_clicked: bool,
        yaw: Option<f64>,
        held: BlockId,
    ) -> PlaceContext {
        PlaceContext {
            clicked_pos: pos,
            clicked_face: face,
            click_location: Vec3::new(pos[0] as f64 + 0.5, click_y, pos[2] as f64 + 0.5),
            replace_clicked,
            is_inside: false,
            is_sneaking: false,
            player_look_direction: yaw.map(Direction::from_y_rot),
            player_pitch_yaw: (0.0, yaw.unwrap_or(0.0) as f32),
            held_block_id: held,
            is_client_side: false,
        }
    }

    #[test]
    fn slab_placement_merge_and_can_be_replaced_witnesses() {
        // gold/sweep.txt:6-29, 48-65, 291-434 and gen/adv-out.txt:1-30
        let d_below = 64.0 + f64::from_bits(0x3fdf_ffff_ffff_ff80); // dLaw = 0.5 - 2^-46 (sweep.txt:6)
        let d_exact = 64.5;
        let d_above = 64.0 + f64::from_bits(0x3fe0_0000_0000_0080); // dLaw = 0.5 + 2^-46 (sweep.txt:8)

        // North face strict 0.5 boundary (sweep.txt:6-8)
        assert_eq!(
            slab_get_state_for_placement(&ctx_at([0, 64, -1], Direction::North, d_below, false, None, OAK_SLAB_ID), AIR_ID, false).slab_type,
            SlabType::Bottom
        );
        assert_eq!(
            slab_get_state_for_placement(&ctx_at([0, 64, -1], Direction::North, d_exact, false, None, OAK_SLAB_ID), AIR_ID, false).slab_type,
            SlabType::Bottom
        );
        assert_eq!(
            slab_get_state_for_placement(&ctx_at([0, 64, -1], Direction::North, d_above, false, None, OAK_SLAB_ID), AIR_ID, false).slab_type,
            SlabType::Top
        );

        // E rows: waterlogged bottom oak slab merges to DOUBLE with waterlogged=false forced (sweep.txt:21-29)
        let merged = slab_get_state_for_placement(
            &ctx_at([0, 64, -1], Direction::North, d_above, true, None, OAK_SLAB_ID),
            OAK_SLAB_ID,
            true,
        );
        assert_eq!(merged, SlabState { block_id: OAK_SLAB_ID, slab_type: SlabType::Double, waterlogged: false });

        // F row: cross-item stone_slab does not merge (sweep.txt:48-56)
        let cross = slab_get_state_for_placement(
            &ctx_at([0, 64, -1], Direction::North, d_above, false, None, OAK_SLAB_ID),
            STONE_SLAB_ID,
            false,
        );
        assert_eq!(cross.slab_type, SlabType::Top);

        // SlabBlock.canBeReplaced (sweep.txt:291-365 & gen/adv-out.txt:19-30)
        let oak_bottom = SlabState { block_id: OAK_SLAB_ID, slab_type: SlabType::Bottom, waterlogged: false };
        let oak_top = SlabState { block_id: OAK_SLAB_ID, slab_type: SlabType::Top, waterlogged: false };
        let oak_double = SlabState { block_id: OAK_SLAB_ID, slab_type: SlabType::Double, waterlogged: false };
        assert!(!slab_can_be_replaced(oak_bottom, &ctx_at([0, 64, 0], Direction::East, d_exact, true, None, OAK_SLAB_ID)));
        assert!(slab_can_be_replaced(oak_bottom, &ctx_at([0, 64, 0], Direction::East, d_above, true, None, OAK_SLAB_ID)));
        assert!(slab_can_be_replaced(oak_top, &ctx_at([0, 64, 0], Direction::West, d_exact, true, None, OAK_SLAB_ID)));
        assert!(!slab_can_be_replaced(oak_top, &ctx_at([0, 64, 0], Direction::West, d_above, true, None, OAK_SLAB_ID)));
        assert!(!slab_can_be_replaced(oak_double, &ctx_at([0, 64, 0], Direction::Up, d_exact, false, None, OAK_SLAB_ID)));
        assert!(!slab_can_be_replaced(oak_bottom, &ctx_at([0, 64, 0], Direction::Up, d_exact, true, None, STONE_SLAB_ID)));
        assert!(slab_can_be_replaced(oak_bottom, &ctx_at([0, 64, 0], Direction::Down, d_exact, false, None, OAK_SLAB_ID)));
    }

    #[test]
    fn stair_placement_shape_and_trapdoor_witnesses() {
        // gold/sweep.txt:121-229 & gen/adv-out.txt:31-39
        let pos = [0, 65, 0];
        let bottom_east = StairState { facing: Direction::East, half: Half::Bottom, shape: StairShape::Straight, waterlogged: false };
        let bottom_west = StairState { facing: Direction::West, half: Half::Bottom, shape: StairShape::Straight, waterlogged: false };
        let bottom_north = StairState { facing: Direction::North, half: Half::Bottom, shape: StairShape::Straight, waterlogged: false };
        let top_east = StairState { facing: Direction::East, half: Half::Top, shape: StairShape::Straight, waterlogged: false };

        let run_cfg = |map: &HashMap<BlockPos, StairState>, face: Direction, click_y: f64, yaw: Option<f64>| {
            let c = ctx_at(pos, face, click_y, false, yaw, STONE_STAIRS_ID);
            stair_get_state_for_placement(&c, false, |p| map.get(&p).copied())
        };

        // cfg1: north=east -> outer_right (sweep.txt:124)
        let mut cfg1 = HashMap::new();
        cfg1.insert(Direction::North.relative(pos), bottom_east);
        assert_eq!(run_cfg(&cfg1, Direction::Up, 64.5, None).shape, StairShape::OuterRight);

        // cfg2: north=west -> outer_left (sweep.txt:127)
        let mut cfg2 = HashMap::new();
        cfg2.insert(Direction::North.relative(pos), bottom_west);
        assert_eq!(run_cfg(&cfg2, Direction::Up, 64.5, None).shape, StairShape::OuterLeft);

        // cfg3: south=east -> inner_right (sweep.txt:130)
        let mut cfg3 = HashMap::new();
        cfg3.insert(Direction::South.relative(pos), bottom_east);
        assert_eq!(run_cfg(&cfg3, Direction::Up, 64.5, None).shape, StairShape::InnerRight);

        // cfg4: south=east-top -> straight on bottom placement, inner_right on top placement (sweep.txt:133, 163)
        let mut cfg4 = HashMap::new();
        cfg4.insert(Direction::South.relative(pos), top_east);
        assert_eq!(run_cfg(&cfg4, Direction::Up, 64.5, None).shape, StairShape::Straight);
        assert_eq!(run_cfg(&cfg4, Direction::Down, 64.5, None).shape, StairShape::InnerRight);

        // cfg7: north=east + west=north -> outer blocked -> straight; cfg8 adds south=east -> inner_right (sweep.txt:142, 145)
        let mut cfg7 = HashMap::new();
        cfg7.insert(Direction::North.relative(pos), bottom_east);
        cfg7.insert(Direction::West.relative(pos), bottom_north);
        assert_eq!(run_cfg(&cfg7, Direction::Up, 64.5, None).shape, StairShape::Straight);
        cfg7.insert(Direction::South.relative(pos), bottom_east);
        assert_eq!(run_cfg(&cfg7, Direction::Up, 64.5, None).shape, StairShape::InnerRight);

        // gen/adv-out.txt:37-39 (FACING=east -> yaw=270.0)
        let mut d1 = HashMap::new();
        d1.insert(Direction::East.relative(pos), bottom_north);
        assert_eq!(run_cfg(&d1, Direction::Up, 64.5, Some(270.0)).shape, StairShape::OuterLeft);
        let mut d3 = HashMap::new();
        d3.insert(Direction::West.relative(pos), bottom_north);
        assert_eq!(run_cfg(&d3, Direction::Up, 64.5, Some(270.0)).shape, StairShape::InnerLeft);

        // Trapdoor branch A vs branch B (gold/sweep.txt:267-289, gen/adv-out.txt:31-36)
        assert_eq!(
            trapdoor_get_state_for_placement(&ctx_at([0, 64, 0], Direction::North, 64.5, true, None, OAK_TRAPDOOR_ID), false, true),
            TrapDoorPlaceState { facing: Direction::South, half: Half::Top, open: true, powered: true, waterlogged: false }
        );
        assert_eq!(
            trapdoor_get_state_for_placement(&ctx_at([0, 64, -1], Direction::North, 64.5, false, None, OAK_TRAPDOOR_ID), true, true),
            TrapDoorPlaceState { facing: Direction::North, half: Half::Bottom, open: true, powered: true, waterlogged: true }
        );
        assert_eq!(
            trapdoor_get_state_for_placement(&ctx_at([0, 64, -1], Direction::North, 64.5 + 1.0e-12, false, None, OAK_TRAPDOOR_ID), false, false),
            TrapDoorPlaceState { facing: Direction::North, half: Half::Top, open: false, powered: false, waterlogged: false }
        );
        // gen/adv-out.txt:32 down(yaw=90, signal) -> facing=east (opposite of west), half=top, open=true, powered=true
        assert_eq!(
            trapdoor_get_state_for_placement(&ctx_at([0, 63, 0], Direction::Down, 64.5, false, Some(90.0), OAK_TRAPDOOR_ID), false, true),
            TrapDoorPlaceState { facing: Direction::East, half: Half::Top, open: true, powered: true, waterlogged: false }
        );
        // sweep.txt:286-289 yaw=0,90,180,270 on UP
        for (yaw, expected_facing) in [(0.0, Direction::North), (90.0, Direction::East), (180.0, Direction::South), (270.0, Direction::West)] {
            assert_eq!(
                trapdoor_get_state_for_placement(&ctx_at([0, 65, 0], Direction::Up, 64.5, false, Some(yaw), OAK_TRAPDOOR_ID), false, false).facing,
                expected_facing
            );
        }
    }
}

#[cfg(test)]
mod part3_tests {
    use super::*;
    use std::collections::HashMap;

    const OAK_DOOR_ID: BlockId = BlockId(10);
    const LADDER_ID: BlockId = BlockId(11);
    const SCAFFOLD_ID: BlockId = BlockId(12);

    fn door_ctx(pos: BlockPos, dx: f64, dz: f64, yaw: f64) -> PlaceContext {
        PlaceContext {
            clicked_pos: pos,
            clicked_face: Direction::Up,
            click_location: Vec3::new(pos[0] as f64 + dx, pos[1] as f64, pos[2] as f64 + dz),
            replace_clicked: false,
            is_inside: false,
            is_sneaking: false,
            player_look_direction: Some(Direction::from_y_rot(yaw)),
            player_pitch_yaw: (0.0, yaw as f32),
            held_block_id: OAK_DOOR_ID,
            is_client_side: false,
        }
    }

    #[test]
    fn door_clearance_hinge_and_set_placed_by_witnesses() {
        // evidence.txt:7-33 & adv-evidence.txt:1-9
        let pos = [2, 64, 2];
        let empty = |_| DoorCellInfo::default();
        assert!(door_get_state_for_placement(&door_ctx(pos, 0.5, 0.5, 180.0), 64, true, |_| false, empty).is_none()); // A4a y==maxY
        assert!(door_get_state_for_placement(&door_ctx([2, 63, 2], 0.5, 0.5, 180.0), 64, true, |_| false, empty).is_some()); // A4b
        assert!(door_get_state_for_placement(&door_ctx(pos, 0.5, 0.5, 180.0), 319, false, |_| false, empty).is_none()); // A2 stone above

        // A5 signal@above-only -> open=true, powered=true (evidence.txt:13)
        let d_sig = door_get_state_for_placement(&door_ctx(pos, 0.5, 0.5, 180.0), 319, true, |p| p == [2, 65, 2], empty).unwrap();
        assert!(d_sig.open && d_sig.powered && d_sig.half == DoorHalf::Lower);
        let (upper_pos, upper_state, flags) = door_set_placed_by(d_sig, pos);
        assert_eq!(upper_pos, [2, 65, 2]);
        assert_eq!(flags, 3);
        assert_eq!(upper_state.half, DoorHalf::Upper);
        assert!(upper_state.open && upper_state.powered);

        // Quadrant strict 0.5 boundaries (evidence.txt:16-22, adv-evidence.txt:4-7)
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.5, 0.5, 180.0), Direction::North, empty), DoorHinge::Left);
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.5000000000000004, 0.5, 180.0), Direction::North, empty), DoorHinge::Right);
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.5, 0.5, 90.0), Direction::West, empty), DoorHinge::Left);
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.5, 0.49, 90.0), Direction::West, empty), DoorHinge::Right);
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.5, 0.5, 0.0), Direction::South, empty), DoorHinge::Left);
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.51, 0.5, 0.0), Direction::South, empty), DoorHinge::Left);

        // SCORE & DOOR attach (evidence.txt:24-31, adv-evidence.txt:1-3)
        let mut cells = HashMap::new();
        cells.insert([1, 65, 2], DoorCellInfo { is_full_collision: true, lower_door: false }); // left-UPPER stone
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.6, 0.5, 180.0), Direction::North, |p| cells.get(&p).copied().unwrap_or_default()), DoorHinge::Left);
        cells.clear();
        cells.insert([1, 64, 2], DoorCellInfo { is_full_collision: false, lower_door: true }); // left-LOWER door
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.4, 0.5, 180.0), Direction::North, |p| cells.get(&p).copied().unwrap_or_default()), DoorHinge::Right);
        cells.clear();
        cells.insert([1, 65, 2], DoorCellInfo { is_full_collision: false, lower_door: false }); // UPPER door ignored
        assert_eq!(door_get_hinge(&door_ctx(pos, 0.4, 0.5, 180.0), Direction::North, |p| cells.get(&p).copied().unwrap_or_default()), DoorHinge::Left);
    }

    #[test]
    fn ladder_and_scaffolding_placement_witnesses() {
        // evidence.txt:37-50 & adv-evidence.txt:10-12
        let pos = [2, 64, 2];
        let mk_ladder_ctx = |face: Direction, replace_clicked: bool, pitch: f32, yaw: f32| PlaceContext {
            clicked_pos: pos,
            clicked_face: face,
            click_location: Vec3::new(2.5, 64.5, 2.5),
            replace_clicked,
            is_inside: false,
            is_sneaking: false,
            player_look_direction: Some(Direction::from_y_rot(yaw as f64)),
            player_pitch_yaw: (pitch, yaw),
            held_block_id: LADDER_ID,
            is_client_side: false,
        };
        // B1: stone north wall, look north (yaw=180) -> facing south
        let north_sturdy = |p: BlockPos, dir: Direction| p == [2, 64, 1] && dir == Direction::South;
        assert_eq!(
            ladder_get_state_for_placement(&mk_ladder_ctx(Direction::South, false, 0.0, 180.0), None, false, north_sturdy),
            Some(LadderPlaceState { facing: Direction::South, waterlogged: false })
        );
        // B2: fence ring (nothing sturdy) -> None
        assert_eq!(
            ladder_get_state_for_placement(&mk_ladder_ctx(Direction::South, false, 0.0, 180.0), None, false, |_, _| false),
            None
        );
        // B5b vs B5c & adv-evidence:11-12: anti-stack guard
        assert_eq!(
            ladder_get_state_for_placement(&mk_ladder_ctx(Direction::North, false, 0.0, 180.0), Some(Direction::North), false, north_sturdy),
            None
        );
        assert_eq!(
            ladder_get_state_for_placement(&mk_ladder_ctx(Direction::North, false, 0.0, 180.0), Some(Direction::West), false, north_sturdy),
            Some(LadderPlaceState { facing: Direction::South, waterlogged: false })
        );
        assert_eq!(
            ladder_get_state_for_placement(&mk_ladder_ctx(Direction::North, true, 0.0, 180.0), Some(Direction::North), false, north_sturdy),
            Some(LadderPlaceState { facing: Direction::South, waterlogged: false })
        );

        // Scaffolding (evidence.txt:52-78, adv-evidence.txt:13-19, walkfix-evidence.txt:2-4)
        let mut grid: HashMap<BlockPos, ScaffoldingCell> = HashMap::new();
        let scaf = |d: u8| ScaffoldingCell { is_scaffolding: true, distance: d, sturdy_up: false, replaceable: true };
        let air = || ScaffoldingCell { is_scaffolding: false, distance: 0, sturdy_up: false, replaceable: true };
        let stone = || ScaffoldingCell { is_scaffolding: false, distance: 0, sturdy_up: true, replaceable: false };

        let pos_s = [2, 70, 2];
        grid.insert([2, 69, 2], stone());
        let s1 = scaffolding_get_state_for_placement(pos_s, false, |p| grid.get(&p).copied().unwrap_or_else(air));
        assert_eq!(s1, ScaffoldingState { distance: 0, bottom: false, waterlogged: false });
        assert!(scaffolding_can_survive(s1.distance));

        grid.clear();
        let s2 = scaffolding_get_state_for_placement(pos_s, true, |p| grid.get(&p).copied().unwrap_or_else(air));
        assert_eq!(s2, ScaffoldingState { distance: 7, bottom: true, waterlogged: true });
        assert!(!scaffolding_can_survive(s2.distance));

        // C4b2 / C4b3 / WF: 6-run vs 7-run east walk (walkfix-evidence.txt:2-4)
        grid.clear();
        for x in 2..=8 {
            grid.insert([x, 70, 2], scaf(0));
        }
        let mk_scaf_ctx = |face: Direction, sneaking: bool, inside: bool, yaw: f64| PlaceContext {
            clicked_pos: pos_s,
            clicked_face: face,
            click_location: Vec3::new(2.5, 70.5, 2.5),
            replace_clicked: false,
            is_inside: inside,
            is_sneaking: sneaking,
            player_look_direction: Some(Direction::from_y_rot(yaw)),
            player_pitch_yaw: (0.0, yaw as f32),
            held_block_id: SCAFFOLD_ID,
            is_client_side: false,
        };
        assert_eq!(
            scaffolding_update_placement_context(&mk_scaf_ctx(Direction::Up, false, false, 270.0), |_| true, |p| grid.get(&p).copied().unwrap_or_else(air)),
            Some(([9, 70, 2], Direction::East))
        );
        grid.insert([9, 70, 2], stone());
        assert_eq!(
            scaffolding_update_placement_context(&mk_scaf_ctx(Direction::Up, false, false, 270.0), |_| true, |p| grid.get(&p).copied().unwrap_or_else(air)),
            None
        );
        grid.insert([9, 70, 2], scaf(0));
        assert_eq!(
            scaffolding_update_placement_context(&mk_scaf_ctx(Direction::Up, false, false, 270.0), |_| true, |p| grid.get(&p).copied().unwrap_or_else(air)),
            None
        );

        // C4c/C4d sneak & sneak-inside
        assert_eq!(
            scaffolding_update_placement_context(&mk_scaf_ctx(Direction::North, true, false, 180.0), |_| true, |p| grid.get(&p).copied().unwrap_or_else(air)),
            Some(([2, 70, 1], Direction::North))
        );
        assert_eq!(
            scaffolding_update_placement_context(&mk_scaf_ctx(Direction::North, true, true, 180.0), |_| true, |p| grid.get(&p).copied().unwrap_or_else(air)),
            Some(([2, 70, 3], Direction::South))
        );

        // 10-column vertical climb (adv-evidence.txt:15)
        grid.clear();
        for y in 70..=79 {
            grid.insert([2, y, 2], scaf(0));
        }
        assert_eq!(
            scaffolding_update_placement_context(&mk_scaf_ctx(Direction::North, false, false, 180.0), |_| true, |p| grid.get(&p).copied().unwrap_or_else(air)),
            Some(([2, 80, 2], Direction::Up))
        );

        // Collision shapes (evidence.txt:76-78, adv-evidence.txt:19)
        let dist0 = ScaffoldingState { distance: 0, bottom: false, waterlogged: false };
        let dist4_bottom = ScaffoldingState { distance: 4, bottom: true, waterlogged: false };
        let dist4_nobottom = ScaffoldingState { distance: 4, bottom: false, waterlogged: false };
        assert_eq!(scaffolding_collision_shape(dist0, 71.0, 70, false, false), ScaffoldingCollision::Stable);
        assert_eq!(scaffolding_collision_shape(dist0, 71.0, 70, true, false), ScaffoldingCollision::Empty);
        assert_eq!(scaffolding_collision_shape(dist4_bottom, 71.0, 70, true, false), ScaffoldingCollision::BelowLip);
        assert_eq!(scaffolding_collision_shape(dist4_nobottom, 71.0, 70, true, false), ScaffoldingCollision::Empty);
        assert_eq!(scaffolding_collision_shape(dist0, 71.0, 70, false, true), ScaffoldingCollision::Empty);
    }
}

