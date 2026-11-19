//! Portal creation bounds and fallback base selection (Mojang 26.2 `PortalForcer.createPortal`).

use crate::math;

pub type BlockPos = [i32; 3];

/// `WorldBorder.clampToBounds` horizontal epsilon (`1.0E-5f` as `double`).
pub const BORDER_CLAMP_EPS: f64 = 9.999999747378752e-6;

/// Nether/Overworld default horizontal border extent for portal clamping.
pub const DEFAULT_WORLD_BORDER_RADIUS: f64 = 29_999_984.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimensionHeightBounds {
    pub min_y: i32,
    pub max_y: i32,
    pub logical_height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBorderBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl WorldBorderBounds {
    pub const fn default_mc() -> Self {
        Self {
            min_x: -DEFAULT_WORLD_BORDER_RADIUS,
            max_x: DEFAULT_WORLD_BORDER_RADIUS,
            min_z: -DEFAULT_WORLD_BORDER_RADIUS,
            max_z: DEFAULT_WORLD_BORDER_RADIUS,
        }
    }

    pub fn from_center_size(center_x: f64, center_z: f64, size: f64) -> Self {
        let half = size / 2.0;
        Self {
            min_x: center_x - half,
            max_x: center_x + half,
            min_z: center_z - half,
            max_z: center_z + half,
        }
    }

    /// `WorldBorder.clampToBounds(BlockPos)` (`BlockPos.containing` via `Mth.floor`).
    /// Note: does not alter Y; runs AFTER fallback Y-clamping in `PortalForcer.createPortal`.
    pub fn clamp_to_bounds(self, pos: BlockPos) -> BlockPos {
        let x = (pos[0] as f64).clamp(self.min_x, self.max_x - BORDER_CLAMP_EPS);
        let z = (pos[2] as f64).clamp(self.min_z, self.max_z - BORDER_CLAMP_EPS);
        [math::floor(x), pos[1], math::floor(z)]
    }
}

/// Top inclusive usable Y in `PortalForcer.createPortal` (`@33-57`):
/// `Math.min(level.getMaxY(), level.getMinY() + level.getLogicalHeight() - 1)`.
pub const fn top_usable(dims: DimensionHeightBounds) -> i32 {
    let logical_top = dims.min_y + dims.logical_height - 1;
    if dims.max_y < logical_top {
        dims.max_y
    } else {
        logical_top
    }
}

/// Fallback minimum Y (`@452-459`): `Math.max(70, level.getMinY() + 1)`.
pub const fn fallback_floor(min_y: i32) -> i32 {
    let min_plus_one = min_y + 1;
    if 70 > min_plus_one {
        70
    } else {
        min_plus_one
    }
}

/// Fallback ceiling Y (`@464-469`): `top_usable - 9`.
pub const fn fallback_ceiling(dims: DimensionHeightBounds) -> i32 {
    top_usable(dims) - 9
}

/// `PortalForcer.createPortal` fallback loop base (`@443-497`).
/// Assumes the primary search found no valid slot (`bestPos == null`).
/// Returns `None` without touching `border` if `ceiling < floor`.
pub fn create_portal_fallback_base(
    entry_pos: BlockPos,
    axis_step: [i32; 2],
    dims: DimensionHeightBounds,
    border: WorldBorderBounds,
) -> Option<BlockPos> {
    let floor_y = fallback_floor(dims.min_y);
    let ceil_y = fallback_ceiling(dims);
    if ceil_y < floor_y {
        return None;
    }
    let clamped_y = entry_pos[1].clamp(floor_y, ceil_y);
    let offset = [
        entry_pos[0] - axis_step[0],
        clamped_y,
        entry_pos[2] - axis_step[1],
    ];
    Some(border.clamp_to_bounds(offset))
}

#[cfg(test)]
mod part12_tests {
    use super::*;

    #[test]
    fn part12_portal_create_ceiling_and_fallback_witnesses() {
        let default_border = WorldBorderBounds::default_mc();
        let axis_x_step = [1, 0];

        // harness-output.txt:1-7 & check-output.txt:1-5
        let cases: &[(&str, DimensionHeightBounds, WorldBorderBounds, BlockPos, Option<BlockPos>)] = &[
            (
                "case1-nether",
                DimensionHeightBounds { min_y: 0, max_y: 255, logical_height: 128 },
                default_border,
                [0, 200, 0],
                Some([-1, 118, 0]),
            ),
            (
                "case2a-overworld-low",
                DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                default_border,
                [0, 40, 0],
                Some([-1, 70, 0]),
            ),
            (
                "case2b-overworld-high",
                DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                default_border,
                [0, 999, 0],
                Some([-1, 310, 0]),
            ),
            (
                "case3-maxY-leg",
                DimensionHeightBounds { min_y: 0, max_y: 256, logical_height: 256 },
                default_border,
                [0, 999, 0],
                Some([-1, 246, 0]),
            ),
            (
                "case4-empty",
                DimensionHeightBounds { min_y: 0, max_y: 78, logical_height: 128 },
                default_border,
                [0, 70, 0],
                None,
            ),
            (
                "case6-empty-gate",
                DimensionHeightBounds { min_y: 0, max_y: 319, logical_height: 78 },
                default_border,
                [0, 999, 0],
                None,
            ),
            (
                "A-ceiling-eq-floor",
                DimensionHeightBounds { min_y: 0, max_y: 79, logical_height: 128 },
                default_border,
                [0, 999, 0],
                Some([-1, 70, 0]),
            ),
            (
                "B-floor-leg-minY100",
                DimensionHeightBounds { min_y: 100, max_y: 319, logical_height: 384 },
                default_border,
                [0, 50, 0],
                Some([-1, 101, 0]),
            ),
            (
                "C-mid-passthrough",
                DimensionHeightBounds { min_y: 0, max_y: 319, logical_height: 384 },
                default_border,
                [0, 150, 0],
                Some([-1, 150, 0]),
            ),
            (
                "D-small-border-far",
                DimensionHeightBounds { min_y: -64, max_y: 319, logical_height: 384 },
                WorldBorderBounds::from_center_size(0.0, 0.0, 10.0),
                [1000, 999, 1000],
                Some([4, 310, 4]),
            ),
            (
                "E-floor-gt-ceiling",
                DimensionHeightBounds { min_y: 90, max_y: 99, logical_height: 128 },
                default_border,
                [0, 999, 0],
                None,
            ),
        ];

        for &(name, dims, border, entry, expected) in cases {
            assert_eq!(
                create_portal_fallback_base(entry, axis_x_step, dims, border),
                expected,
                "{name}"
            );
        }
    }
}
