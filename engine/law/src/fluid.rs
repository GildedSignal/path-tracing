//! Mojang 26.2 fluid spread, stack-memoized slope-find BFS/DFS, source conversion,
//! lava-water contact, wash-away, and tick delays (`FlowingFluid`, `WaterFluid`,
//! `LavaFluid`, `LiquidBlock`, `SimpleWaterloggedBlock`).
//!
//! Ported from:
//! - `lab/results/local-20261118-2302-1-0-port-fluid-spread-distance/dossier.md`
//! - `lab/results/local-20261114-2031-1-0-port-fluid-tick-delay/dossier.md`
//! - `lab/results/local-20261114-2033-1-0-port-water-source-neighbor-count/dossier.md`
//! - `lab/results/local-20261114-1013-1-0-port-fluid-rules/dossier.md`

use crate::neighbor::Direction;

/// Returned by `slope_distance` when no hole lies within `slope_find_distance`.
pub const NO_HOLE: u16 = 1000;

/// Kind of flowing fluid (`FAST_LAVA` is the Nether dimension attribute).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlowKind {
    Water,
    Lava { fast: bool },
}

impl FlowKind {
    /// `WaterFluid.getSlopeFindDistance 0-1` (4); `LavaFluid 0-12` (4 fast, 2 else).
    #[inline]
    pub const fn slope_find_distance(self) -> u8 {
        match self {
            Self::Water => 4,
            Self::Lava { fast } => if fast { 4 } else { 2 },
        }
    }

    /// `WaterFluid.getDropOff 0-1` (1); `LavaFluid 0-11` (1 fast, 2 else).
    #[inline]
    pub const fn drop_off(self) -> u8 {
        match self {
            Self::Water => 1,
            Self::Lava { fast } => if fast { 1 } else { 2 },
        }
    }

    /// `WaterFluid.getTickDelay 0-1` (5); `LavaFluid.getTickDelay 0-14` (10 fast, 30 else).
    #[inline]
    pub const fn tick_delay(self) -> u8 {
        match self {
            Self::Water => 5,
            Self::Lava { fast } => if fast { 10 } else { 30 },
        }
    }

    /// `LavaFluid.getSpreadDelay 55-89`: multiplies base delay by `4` with `P = 3/4`
    /// (`next_int_4 != 0`) when both fluids are non-empty, neither is falling, and
    /// `new_cell.amount > old_cell.amount` (drain/rise case).
    #[inline]
    pub const fn spread_delay(self, old_cell: FluidCell, new_cell: FluidCell, random_next_int_4: i32) -> u8 {
        let base = self.tick_delay();
        match self {
            Self::Water => base,
            Self::Lava { .. } => {
                if old_cell.amount > 0
                    && new_cell.amount > 0
                    && !old_cell.falling
                    && !new_cell.falling
                    && new_cell.amount > old_cell.amount
                    && random_next_int_4 != 0
                {
                    base * 4
                } else {
                    base
                }
            }
        }
    }
}

/// A fluid cell: `amount` in `0..=8` (0 = empty, 8 = source-strength), `falling`, `source`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FluidCell {
    pub amount: u8,
    pub falling: bool,
    pub source: bool,
}

impl FluidCell {
    pub const EMPTY: Self = Self { amount: 0, falling: false, source: false };
    pub const WATER_SOURCE: Self = Self { amount: 8, falling: false, source: true };

    /// `FlowingFluid.getOwnHeight 0-9`: `amount as f32 / 9.0f`.
    #[inline]
    pub fn own_height(self) -> f32 {
        self.amount as f32 / 9.0f32
    }


    /// `FlowingFluid.getLegacyLevel 0-44`: source 0, else `8 - min(8, amount) + 8 * falling`.
    #[inline]
    pub const fn legacy_level(self) -> u8 {
        if self.source {
            0
        } else {
            8 - (if self.amount > 8 { 8 } else { self.amount }) + if self.falling { 8 } else { 0 }
        }
    }
}

/// `FlowingFluid.spreadToSides 0-10`: amount spread sideways into an open cell.
/// Falling cells spread sideways at forced level 7 (`bipush 7`); `<= 0` stops spread.
#[inline]
pub const fn side_amount(from: FluidCell, kind: FlowKind) -> Option<u8> {
    let mut level = from.amount as i8 - kind.drop_off() as i8;
    if from.falling {
        level = 7;
    }
    if level > 0 {
        Some(level as u8)
    } else {
        None
    }
}

/// `FlowingFluid.getNewLiquid 126-284`:
/// - When `source_neighbors >= 2 && can_convert_to_source && below_solid_or_same_source`: becomes a source!
/// - Else if `fed_from_above`: `(amount: 8, falling: true, source: false)`.
/// - Else: horizontal max minus `drop_off` (`amount: 0` drains to air).
#[inline]
pub fn get_new_liquid(
    kind: FlowKind,
    max_neighbor_amount: u8,
    source_neighbors: u8,
    fed_from_above: bool,
    can_convert_to_source: bool,
    below_solid_or_same_source: bool,
) -> FluidCell {
    if source_neighbors >= 2 && can_convert_to_source && below_solid_or_same_source {
        return FluidCell { amount: 8, falling: false, source: true };
    }
    if fed_from_above {
        return FluidCell { amount: 8, falling: true, source: false };
    }
    FluidCell {
        amount: max_neighbor_amount.saturating_sub(kind.drop_off()),
        falling: false,
        source: false,
    }
}

/// `FlowingFluid.spread 23-149`: falling-below priority over horizontal spread.
/// - When below cell can take fluid: sides spread iff `source_neighbors >= 3`.
/// - When down is blocked: sides spread iff `source_cell || !down_is_hole`.
#[inline]
pub const fn spreads_to_sides(
    source_cell: bool,
    down_open: bool,
    down_is_hole: bool,
    source_neighbors: u8,
) -> bool {
    if down_open {
        source_neighbors >= 3
    } else {
        source_cell || !down_is_hole
    }
}

/// Unmemoized direct `FlowingFluid.getSlopeDistance 0-150`.
pub fn slope_distance<P, H>(
    kind: FlowKind,
    from: (i32, i32),
    came_from_dir: Direction,
    distance: u8,
    passable: &mut P,
    hole: &mut H,
) -> u16
where
    P: FnMut(i32, i32) -> bool,
    H: FnMut(i32, i32) -> bool,
{
    let mut best = NO_HOLE;
    for dir in Direction::HORIZONTAL {
        if dir == came_from_dir {
            continue;
        }
        let [sx, _, sz] = dir.step();
        let next = (from.0 + sx, from.1 + sz);
        if !passable(next.0, next.1) {
            continue;
        }
        if hole(next.0, next.1) {
            return distance as u16;
        }
        if distance < kind.slope_find_distance() {
            let d = slope_distance(kind, next, dir.opposite(), distance + 1, passable, hole);
            if d < best {
                best = d;
            }
        }
    }
    best
}

/// 121-byte zero-alloc `SpreadContext` cache (`(dx, dz) in -5..=5`, exact radius for `slope_find_distance <= 4`)
/// storing memoized `passable` and `is_hole` queries without heap allocation.
struct StackSpreadCache<P, H> {
    origin: (i32, i32),
    // Bit 0: pass_known, bit 1: pass_val, bit 2: hole_known, bit 3: hole_val.
    state: [u8; 121],
    passable: P,
    hole: H,
}

impl<P, H> StackSpreadCache<P, H>
where
    P: FnMut(i32, i32) -> bool,
    H: FnMut(i32, i32) -> bool,
{
    #[inline]
    fn idx(&self, x: i32, z: i32) -> Option<usize> {
        let dx = x - self.origin.0;
        let dz = z - self.origin.1;
        if (-5..=5).contains(&dx) && (-5..=5).contains(&dz) {
            Some(((dx + 5) * 11 + (dz + 5)) as usize)
        } else {
            None
        }
    }

    #[inline]
    fn is_passable(&mut self, x: i32, z: i32) -> bool {
        if let Some(i) = self.idx(x, z) {
            let s = self.state[i];
            if (s & 1) != 0 {
                return (s & 2) != 0;
            }
            let v = (self.passable)(x, z);
            self.state[i] = s | 1 | if v { 2 } else { 0 };
            v
        } else {
            (self.passable)(x, z)
        }
    }

    #[inline]
    fn is_hole(&mut self, x: i32, z: i32) -> bool {
        if let Some(i) = self.idx(x, z) {
            let s = self.state[i];
            if (s & 4) != 0 {
                return (s & 8) != 0;
            }
            let v = (self.hole)(x, z);
            self.state[i] = s | 4 | if v { 8 } else { 0 };
            v
        } else {
            (self.hole)(x, z)
        }
    }

    fn recurse(&mut self, max_dist: u8, from: (i32, i32), came_from: Direction, distance: u8) -> u16 {
        let mut best = NO_HOLE;
        for dir in Direction::HORIZONTAL {
            if dir == came_from {
                continue;
            }
            let [sx, _, sz] = dir.step();
            let next = (from.0 + sx, from.1 + sz);
            if !self.is_passable(next.0, next.1) {
                continue;
            }
            if self.is_hole(next.0, next.1) {
                return distance as u16;
            }
            if distance < max_dist {
                let d = self.recurse(max_dist, next, dir.opposite(), distance + 1);
                if d < best {
                    best = d;
                }
            }
        }
        best
    }
}

/// Zero-alloc memoized `FlowingFluid.getSpread 0-231`: returns `(dir_bitmask, min_distance)`
/// where bit `i` corresponds to `Direction::HORIZONTAL[i]` (`N=0, E=1, S=2, W=3`).
/// Checks `can_replace(dir)` only after minimum distance selection (`@192-207`).
pub fn compute_spread_directions<P, H, R>(
    kind: FlowKind,
    origin: (i32, i32),
    passable: P,
    hole: H,
    mut can_replace: R,
) -> (u8, u16)
where
    P: FnMut(i32, i32) -> bool,
    H: FnMut(i32, i32) -> bool,
    R: FnMut(Direction) -> bool,
{
    let mut cache = StackSpreadCache {
        origin,
        state: [0; 121],
        passable,
        hole,
    };
    let max_dist = kind.slope_find_distance();
    let mut min_dist = NO_HOLE;
    let mut raw_mask = 0u8;

    for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
        let [sx, _, sz] = dir.step();
        let nx = origin.0 + sx;
        let nz = origin.1 + sz;
        if !cache.is_passable(nx, nz) {
            continue;
        }
        let d = if cache.is_hole(nx, nz) {
            0
        } else {
            cache.recurse(max_dist, (nx, nz), dir.opposite(), 1)
        };
        if d < min_dist {
            min_dist = d;
            raw_mask = 1 << i;
        } else if d == min_dist {
            raw_mask |= 1 << i;
        }
    }

    let mut final_mask = 0u8;
    for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
        if (raw_mask & (1 << i)) != 0 && can_replace(dir) {
            final_mask |= 1 << i;
        }
    }
    (final_mask, min_dist)
}

/// `LavaFluid.canBeReplacedWith 0-28`: water can enter lava only when `lava_height >= 4.0f / 9.0f`.
#[inline]
pub fn water_replaces_lava(lava_height: f32) -> bool {
    lava_height >= 4.0f32 / 9.0f32
}

/// `WaterFluid.canBeReplacedWith 0-24`: water is replaced only by a non-water fluid arriving from `Direction::Down`.
#[inline]
pub const fn water_replaced_by(direction: Direction, incoming_is_water: bool) -> bool {
    matches!(direction, Direction::Down) && !incoming_is_water
}

/// `SimpleWaterloggedBlock.canPlaceLiquid 0-13`: checks `fluid == Fluids.WATER` (source waterlogged only;
/// flowing water NEVER waterlogs dry stairs/slabs/containers).
#[inline]
pub const fn can_waterlog_container(fluid_is_water_source: bool) -> bool {
    fluid_is_water_source
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetBlock {
    Air,
    Container,
    Solid,
    Destroyable,
}

#[derive(Clone, Copy, Debug)]
pub struct TargetCell {
    pub block: TargetBlock,
    pub fluid_is_water: bool,
    pub block_is_liquid: bool,
    pub new_legacy_level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpreadWrite {
    PlaceLiquid,
    ReplaceFluid { legacy_level: u8 },
    Stone,
    FizzOnly,
}

/// `WaterFluid.spreadTo` (`FlowingFluid.spreadTo 6-65`) / `LavaFluid.spreadTo 0-81`.
pub fn spread_write(lava: bool, direction: Direction, target: TargetCell) -> SpreadWrite {
    if lava && direction == Direction::Down && target.fluid_is_water {
        return if target.block_is_liquid {
            SpreadWrite::Stone
        } else {
            SpreadWrite::FizzOnly
        };
    }
    match target.block {
        TargetBlock::Container => SpreadWrite::PlaceLiquid,
        TargetBlock::Air | TargetBlock::Solid | TargetBlock::Destroyable => SpreadWrite::ReplaceFluid {
            legacy_level: target.new_legacy_level,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LavaContact {
    None,
    Cobblestone,
    Obsidian,
    Basalt,
}

/// `LiquidBlock.shouldSpreadLiquid 0-170`: returns `(conversion, should_schedule_tick)`.
/// Checks water in `{UP, NORTH, SOUTH, EAST, WEST}` (NEVER below).
#[inline]
pub const fn should_spread_liquid(
    lava: bool,
    lava_is_source: bool,
    water_above_or_beside: bool,
    soul_soil_below: bool,
    blue_ice_above_or_beside: bool,
) -> (LavaContact, bool) {
    if !lava {
        return (LavaContact::None, true);
    }
    if water_above_or_beside {
        return (
            if lava_is_source {
                LavaContact::Obsidian
            } else {
                LavaContact::Cobblestone
            },
            false,
        );
    }
    if soul_soil_below && blue_ice_above_or_beside {
        return (LavaContact::Basalt, false);
    }
    (LavaContact::None, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reproduces `Harness.java` A–F witnesses from `local-20261118-2302-1-0-port-fluid-spread-distance`.
    #[test]
    fn constants_channel_slope_wash_and_contact() {
        // Constants (A)
        assert_eq!(FlowKind::Water.slope_find_distance(), 4);
        assert_eq!(FlowKind::Water.drop_off(), 1);
        assert_eq!(FlowKind::Water.tick_delay(), 5);
        assert_eq!(FlowKind::Lava { fast: false }.slope_find_distance(), 2);
        assert_eq!(FlowKind::Lava { fast: false }.drop_off(), 2);
        assert_eq!(FlowKind::Lava { fast: false }.tick_delay(), 30);
        assert_eq!(FlowKind::Lava { fast: true }.slope_find_distance(), 4);
        assert_eq!(FlowKind::Lava { fast: true }.drop_off(), 1);
        assert_eq!(FlowKind::Lava { fast: true }.tick_delay(), 10);

        // Flat channel length 7 flowing + 1 source (B)
        let mut cell = FluidCell::WATER_SOURCE;
        let mut levels = vec![cell.legacy_level()];
        while let Some(amt) = side_amount(cell, FlowKind::Water) {
            cell = FluidCell { amount: amt, falling: false, source: false };
            levels.push(cell.legacy_level());
        }
        assert_eq!(levels, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(FluidCell { amount: 8, falling: true, source: false }.legacy_level(), 8);

        // Slope distance 1..4 pulls, 5 = NO_HOLE (1000) (C)
        for hole_x in 1..=5i32 {
            let (mask, min_d) = compute_spread_directions(
                FlowKind::Water,
                (0, 0),
                |x, z| z == 0 && (1..=6).contains(&x),
                |x, z| z == 0 && x == hole_x,
                |_| true,
            );
            if hole_x <= 5 {
                // Note: hole at x=1 is 0 steps from neighbor (1,0); hole at x=5 is 4 steps from neighbor (1,0)!
                assert_eq!(min_d, (hole_x - 1) as u16);
                assert_eq!(mask, 1 << 1); // East
            }
        }
        let (mask5, min_d5) = compute_spread_directions(
            FlowKind::Water,
            (0, 0),
            |x, z| z == 0 && (1..=7).contains(&x),
            |x, z| z == 0 && x == 6,
            |_| true,
        );
        assert_eq!(min_d5, NO_HOLE);
        assert_eq!(mask5, 0b10); // still East (only passable neighbor), at min_dist=1000

        // Falling priority + >= 3 source corner
        assert!(!spreads_to_sides(true, true, true, 2));
        assert!(spreads_to_sides(true, true, true, 3));
        assert!(spreads_to_sides(false, false, false, 0));
        assert!(!spreads_to_sides(false, false, true, 0));

        // Source conversion >= 2
        assert!(get_new_liquid(FlowKind::Water, 7, 2, false, true, true).source);
        assert!(!get_new_liquid(FlowKind::Water, 7, 1, false, true, true).source);
        assert!(!can_waterlog_container(false));
        assert!(can_waterlog_container(true));

        // Water-lava height cutoff 4/9 & Lava-into-water stone vs fizz (E/F)
        for amt in 1..=8u8 {
            let h = FluidCell { amount: amt, falling: false, source: false }.own_height();
            assert_eq!(water_replaces_lava(h), amt >= 4, "amt={amt}");
        }
        assert_eq!(
            spread_write(true, Direction::Down, TargetCell {
                block: TargetBlock::Air, fluid_is_water: true, block_is_liquid: true, new_legacy_level: 8
            }),
            SpreadWrite::Stone
        );
        assert_eq!(
            spread_write(true, Direction::Down, TargetCell {
                block: TargetBlock::Container, fluid_is_water: true, block_is_liquid: false, new_legacy_level: 8
            }),
            SpreadWrite::FizzOnly
        );
        assert_eq!(should_spread_liquid(true, true, true, false, false), (LavaContact::Obsidian, false));
        assert_eq!(should_spread_liquid(true, false, true, false, false), (LavaContact::Cobblestone, false));
        assert_eq!(should_spread_liquid(true, false, false, true, true), (LavaContact::Basalt, false));
        assert_eq!(should_spread_liquid(true, true, false, false, false), (LavaContact::None, true));
        assert_eq!(
            FlowKind::Lava { fast: false }.spread_delay(
                FluidCell { amount: 2, falling: false, source: false },
                FluidCell { amount: 4, falling: false, source: false },
                1
            ),
            120
        );
    }

    #[test]
    fn stack_memoized_spread_matches_unmemoized_dfs_on_mazes() {
        let mut rng = 0x517c_c1b7_2722_0a95u64;
        for trial in 0..40 {
            let mut blocked = [[false; 13]; 13];
            let mut holes = [[false; 13]; 13];
            for x in 0..13 {
                for z in 0..13 {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if (rng >> 60) < 4 {
                        blocked[x][z] = true;
                    } else if (rng >> 56) & 15 == 0 {
                        holes[x][z] = true;
                    }
                }
            }
            blocked[6][6] = false;
            holes[6][6] = false;

            let kind = if trial % 2 == 0 { FlowKind::Water } else { FlowKind::Lava { fast: false } };
            let passable = |x: i32, z: i32| {
                let ix = (x + 6) as usize;
                let iz = (z + 6) as usize;
                ix < 13 && iz < 13 && !blocked[ix][iz]
            };
            let hole = |x: i32, z: i32| {
                let ix = (x + 6) as usize;
                let iz = (z + 6) as usize;
                ix < 13 && iz < 13 && holes[ix][iz]
            };

            let (memo_mask, memo_min) = compute_spread_directions(kind, (0, 0), passable, hole, |_| true);

            let mut ref_min = NO_HOLE;
            let mut ref_mask = 0u8;
            let mut p = passable;
            let mut h = hole;
            for (i, dir) in Direction::HORIZONTAL.into_iter().enumerate() {
                let [sx, _, sz] = dir.step();
                if !p(sx, sz) { continue; }
                let d = if h(sx, sz) { 0 } else { slope_distance(kind, (sx, sz), dir.opposite(), 1, &mut p, &mut h) };
                if d < ref_min {
                    ref_min = d;
                    ref_mask = 1 << i;
                } else if d == ref_min {
                    ref_mask |= 1 << i;
                }
            }
            assert_eq!((memo_mask, memo_min), (ref_mask, ref_min), "trial {trial}");
        }
    }
}
