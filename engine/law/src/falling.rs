//! Mojang 26.2 falling blocks (sand, gravel, concrete powder, anvil, stalactite):
//! flight recurrence, timeout discard, anvil/stalactite/stalagmite damage, and the
//! grounded landing placement two-conjunct gate.
//!
//! Ported from:
//! - `lab/results/local-20261118-2344-1-0-port-falling-block-flight-and-anvil/dossier.md`
//! - `lab/results/local-20261114-1226-1-0-port-falling-block-landing-place/dossier.md`

use crate::neighbor::update_flags;
use crate::{math, Vec3};

/// `FallingBlockEntity.getDefaultGravity 0-3`: plain double `0.04` subtracted before `move(SELF)`.
pub const GRAVITY: f64 = 0.04;

/// `Entity.getAirDrag 0-3`: float `0.98f` widened to `f64` (`0.9800000190734863`) in `FallingBlockEntity.tick @893`.
pub const AIR_DRAG: f64 = 0.98f32 as f64;

/// `FallingBlockEntity.fall(level, pos, state) 5-32`: center X/Z (`+ 0.5`), raw block Y, `Vec3::ZERO`.
#[inline]
pub fn spawn(x: i32, y: i32, z: i32) -> (f64, f64, f64) {
    (x as f64 + 0.5, y as f64, z as f64 + 0.5)
}

/// One free-air flight tick (vertical): `vy -= 0.04`, `y += vy`, `vy *= (double)0.98f`.
#[inline]
pub fn flight_tick(y: f64, vy: f64) -> (f64, f64) {
    let vy = vy - GRAVITY;
    let y = y + vy;
    (y, vy * AIR_DRAG)
}

/// Full 3D free-air flight tick (`applyGravity`, `move(SELF)`, `scale(AIR_DRAG)`).
#[inline]
pub fn flight_tick_3d(pos: Vec3, vel: Vec3) -> (Vec3, Vec3) {
    let moved_vel = Vec3::new(vel.x, vel.y - GRAVITY, vel.z);
    let next_pos = pos.plus(moved_vel);
    let next_vel = moved_vel.scale(AIR_DRAG);
    (next_pos, next_vel)
}

/// `FallingBlockEntity.tick 800-847`: timeout discard check after `time += 1` and `move`.
#[inline]
pub const fn discards(time: i32, block_y: i32, min_y: i32, max_y: i32) -> bool {
    (time > 100 && (block_y <= min_y || block_y > max_y)) || time > 600
}

/// `FallingBlockEntity.causeFallDamage 9-15`: `Mth.ceil(fall_distance - 1.0)`.
#[inline]
pub fn fall_distance_cells(fall_distance: f64) -> i32 {
    math::ceil(fall_distance - 1.0)
}

/// `FallingBlockEntity.causeFallDamage 82-100`: `min(Mth.floor(cells * per_distance), max)` in `f32`.
#[inline]
pub fn fall_damage(cells: i32, per_distance: f32, max: i32) -> f32 {
    if cells < 0 {
        return 0.0;
    }
    math::floor_f32(cells as f32 * per_distance).min(max) as f32
}

pub const ANVIL_PER_DISTANCE: f32 = 2.0;
pub const ANVIL_MAX_DAMAGE: i32 = 40;

/// `SpeleothemBlock.spawnFallingStalactite 34-65`: `max(1 + break_y - tip_y, 6)` per block, max 40.
#[inline]
pub fn stalactite_hurt(break_y: i32, tip_y: i32) -> (f32, i32) {
    ((1 + break_y - tip_y).max(6) as f32, ANVIL_MAX_DAMAGE)
}

/// `PointedDripstoneBlock.fallOn 26-45`: `fallDistance + 2.5` (f64), multiplier `2.0` (f32).
pub const STALAGMITE_FALL_DISTANCE_OFFSET: f64 = 2.5;
pub const STALAGMITE_DAMAGE_MULTIPLIER: f32 = 2.0;

/// `FallingBlockEntity.causeFallDamage 131-204`: anvil degradation roll when `dmg > 0`.
#[inline]
pub fn anvil_degrades(cells: i32, next_float: f32) -> bool {
    next_float < (0.05f32 + cells as f32 * 0.05f32)
}

/// `AnvilBlock.damage(BlockState)` progression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnvilStage {
    Intact,
    Chipped,
    Damaged,
    Broken,
}

impl AnvilStage {
    #[inline]
    pub const fn damage(self) -> Self {
        match self {
            Self::Intact => Self::Chipped,
            Self::Chipped => Self::Damaged,
            Self::Damaged | Self::Broken => Self::Broken,
        }
    }
}

/// `FallingBlock.isFree(BlockState) 0-36`: `isAir || is(FIRE) || liquid || canBeReplaced`.
#[inline]
pub const fn is_free(is_air: bool, is_fire: bool, is_liquid: bool, can_be_replaced: bool) -> bool {
    is_air || is_fire || is_liquid || can_be_replaced
}

/// `FallingBlockEntity.tick @263-280`: velocity is damped before checking landing predicates.
#[inline]
pub fn damp_landing_velocity(vel: Vec3) -> Vec3 {
    Vec3::new(vel.x * 0.7, vel.y * -0.5, vel.z * 0.7)
}

/// Server-side inputs for `FallingBlockEntity.tick @241-797`.
#[derive(Clone, Copy, Debug)]
pub struct LandingContext {
    pub on_ground: bool,
    pub in_water: bool,
    pub landing_is_moving_piston: bool,
    pub cancel_drop: bool,
    pub landing_can_be_replaced: bool,
    pub carried_can_survive: bool,
    pub below_is_free: bool,
    pub is_concrete_powder: bool,
    pub carried_has_waterlogged: bool,
    pub landing_fluid_is_water: bool,
    pub drop_item: bool,
    pub entity_drops_gamerule: bool,
    pub set_block_succeeded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingOutcome {
    /// Neither `onGround` nor `inWater` (`@250 -> 800`): stays airborne to timeout check / drag.
    Airborne,
    /// Landing cell is `MOVING_PISTON` (`@285 -> 797`): survives to drag without placement or drop.
    SurviveMovingPiston,
    /// `cancelDrop` (`@295 -> 787`): discarded + `callOnBrokenAfterFall`, no item drop.
    DiscardCancelledNoDrop,
    /// Both conjuncts hold & `setBlock(..., 3)` succeeds (`@451-530`): placed, discarded, `onLand`.
    Placed { flags: i32, waterlogged: bool },
    /// Precondition refused (`@390/395 -> 741`): discards first, drops item if `drop_item && entity_drops`.
    PreconditionRefuse { spawn_drop: bool },
    /// `setBlock` returned `false` with `drop_item == false` (`@699 -> 784`): retries next tick.
    SetBlockRejectedRetry,
    /// `setBlock` returned `false` with `drop_item == true` (`@722-734`): discards, spawns item if gamerule.
    SetBlockRejectedDiscard { spawn_drop: bool },
}

/// Exact `FallingBlockEntity.tick @241-797` grounded placement decision gate (claims C2–C7).
pub const fn resolve_landing(ctx: LandingContext) -> LandingOutcome {
    if !ctx.on_ground && !ctx.in_water {
        return LandingOutcome::Airborne;
    }
    if ctx.landing_is_moving_piston {
        return LandingOutcome::SurviveMovingPiston;
    }
    if ctx.cancel_drop {
        return LandingOutcome::DiscardCancelledNoDrop;
    }
    let can_replace = ctx.landing_can_be_replaced;
    let keep_falling = ctx.below_is_free && (!ctx.is_concrete_powder || !ctx.in_water);
    let survives_and_supported = ctx.carried_can_survive && !keep_falling;
    if !can_replace || !survives_and_supported {
        return LandingOutcome::PreconditionRefuse {
            spawn_drop: ctx.drop_item && ctx.entity_drops_gamerule,
        };
    }
    if ctx.set_block_succeeded {
        LandingOutcome::Placed {
            flags: update_flags::ALL, // 3
            waterlogged: ctx.carried_has_waterlogged && ctx.landing_fluid_is_water,
        }
    } else if !ctx.drop_item {
        LandingOutcome::SetBlockRejectedRetry
    } else {
        LandingOutcome::SetBlockRejectedDiscard {
            spawn_drop: ctx.entity_drops_gamerule,
        }
    }
}

/// `ConcretePowderBlock.touchesLiquid @20-113` + `shouldSolidify @0-20`:
/// `neighbor_water_and_non_sturdy` is indexed by `Direction::ALL` `[D, U, N, S, W, E]`.
/// `Down` (index 0) only counts when the cell itself is water (`cell_is_water`).
#[inline]
pub const fn concrete_powder_should_solidify(
    cell_is_water: bool,
    neighbor_water_and_non_sturdy: [bool; 6],
) -> bool {
    if cell_is_water {
        return true;
    }
    let mut i = 1usize; // skip Down (index 0) when cell itself is not water
    while i < 6 {
        if neighbor_water_and_non_sturdy[i] {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reproduces `Witness.java` cases A, B, C and `patch.out` C3/C7 from
    /// `local-20261114-1226-1-0-port-falling-block-landing-place`.
    #[test]
    fn landing_two_conjunct_gate_and_concrete_powder() {
        let eligible = LandingContext {
            on_ground: true,
            in_water: false,
            landing_is_moving_piston: false,
            cancel_drop: false,
            landing_can_be_replaced: true,
            carried_can_survive: true,
            below_is_free: false,
            is_concrete_powder: false,
            carried_has_waterlogged: false,
            landing_fluid_is_water: false,
            drop_item: true,
            entity_drops_gamerule: true,
            set_block_succeeded: true,
        };
        assert_eq!(
            resolve_landing(eligible),
            LandingOutcome::Placed { flags: 3, waterlogged: false }
        );
        // Case B: blocked cell (e.g. stone) -> precondition refuse
        assert_eq!(
            resolve_landing(LandingContext { landing_can_be_replaced: false, ..eligible }),
            LandingOutcome::PreconditionRefuse { spawn_drop: true }
        );
        // Case C: replaceable air over free air -> precondition refuse
        assert_eq!(
            resolve_landing(LandingContext { below_is_free: true, ..eligible }),
            LandingOutcome::PreconditionRefuse { spawn_drop: true }
        );
        // Concrete powder in water ignores below_is_free!
        assert_eq!(
            resolve_landing(LandingContext {
                on_ground: false,
                in_water: true,
                below_is_free: true,
                is_concrete_powder: true,
                ..eligible
            }),
            LandingOutcome::Placed { flags: 3, waterlogged: false }
        );
        // Moving piston survival + setBlock rejection retry vs discard
        assert_eq!(
            resolve_landing(LandingContext { landing_is_moving_piston: true, ..eligible }),
            LandingOutcome::SurviveMovingPiston
        );
        assert_eq!(
            resolve_landing(LandingContext { set_block_succeeded: false, drop_item: false, ..eligible }),
            LandingOutcome::SetBlockRejectedRetry
        );
        // Concrete powder ignores water below unless cell itself is water
        assert!(!concrete_powder_should_solidify(false, [true, false, false, false, false, false]));
        assert!(concrete_powder_should_solidify(false, [false, false, true, false, false, false]));
        assert_eq!(AnvilStage::Intact.damage().damage().damage(), AnvilStage::Broken);
    }
}
