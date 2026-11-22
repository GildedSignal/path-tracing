//! Redstone scheduling law: repeater (`DiodeBlock`/`RepeaterBlock`), observer
//! (`ObserverBlock`), and dispenser (`DispenserBlock`) rules ported bit-for-bit
//! from Mojang 26.2 `server-26.2.jar`.
//!
//! Only the decision rules live here; `LevelTicks` scheduling and neighbor-update
//! drain ordering live in `crate::ticks` and `crate::neighbor`.

pub use crate::ticks::TickPriority;

/// `TickPriority.getValue` (`TickPriority` static init `-3..=3`).
#[inline]
pub const fn tick_priority_value(priority: TickPriority) -> i32 {
    priority as i32
}

/// `RepeaterBlock.getDelay` (`@0-15`): `DELAY * 2` game ticks (setting `1..=4` → `2..=8`).
#[inline]
pub const fn repeater_game_delay(delay_setting: i32) -> i32 {
    delay_setting * 2
}

/// `DiodeBlock.shouldPrioritize` (`@0-55`): the cell our arrow points into
/// (`pos.relative(FACING.getOpposite())`; blockstate `FACING` is the input side)
/// holds a diode whose `FACING` is not our output direction — practically, a
/// chained diode we feed gets `EXTREMELY_HIGH`.
#[inline]
pub const fn feeds_into_diode(front_is_diode: bool, front_facing_not_my_output: bool) -> bool {
    front_is_diode && front_facing_not_my_output
}

/// `DiodeBlock.checkTickOnNeighbor` (`@0-102`). Returns `(game_delay, priority)`
/// or `None` when nothing is scheduled.
/// Guards: `locked` (`@7`), `powered == should_turn_on` (`@24..39`),
/// `will_tick_this_tick` (`@53`), priority (`@56..88`).
#[allow(clippy::too_many_arguments)]
#[inline]
pub const fn diode_neighbor_schedule(
    game_delay: i32,
    locked: bool,
    powered: bool,
    should_turn_on: bool,
    will_tick_this_tick: bool,
    prioritize: bool,
) -> Option<(i32, TickPriority)> {
    if locked || powered == should_turn_on || will_tick_this_tick {
        return None;
    }
    let priority = if prioritize {
        TickPriority::ExtremelyHigh
    } else if powered {
        TickPriority::VeryHigh
    } else {
        TickPriority::High
    };
    Some((game_delay, priority))
}

/// `DiodeBlock.tick` (`@0-114`): pulse-hold turns `POWERED` on even when the
/// input is already gone, scheduling turn-off at `+delay` with `VERY_HIGH` (`@95-111`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiodeTick {
    /// `isLocked` (`@7`): scheduled tick is dropped, no state change.
    IgnoredLocked,
    /// `POWERED=true` written with `setBlock(..., flags = 2)` (`@78, 92`);
    /// `held_off_after` is set when input is already gone (`@95-111`).
    TurnOn {
        held_off_after: Option<(i32, TickPriority)>,
    },
    /// `POWERED=false` written with `flags = 2` (`@62`).
    TurnOff,
    /// `powered && should_turn_on` (`@37-42`): no state change.
    NoChange,
}

#[inline]
pub const fn diode_tick(
    locked: bool,
    powered: bool,
    should_turn_on: bool,
    game_delay: i32,
) -> DiodeTick {
    if locked {
        return DiodeTick::IgnoredLocked;
    }
    match (powered, should_turn_on) {
        (true, false) => DiodeTick::TurnOff,
        (true, true) => DiodeTick::NoChange,
        (false, true) => DiodeTick::TurnOn {
            held_off_after: None,
        },
        (false, false) => DiodeTick::TurnOn {
            held_off_after: Some((game_delay, TickPriority::VeryHigh)),
        },
    }
}

/// `ObserverBlock.updateShape` (`@0-36`) + `startSignal` (`@0-34`):
/// a change on the `FACING` (observed) side arms the observer server-side only,
/// once per pending tick, at delay `2`, `NORMAL` priority.
#[inline]
pub const fn observer_arms(
    change_on_observed_side: bool,
    powered: bool,
    has_scheduled_tick: bool,
    client_side: bool,
) -> Option<(i32, TickPriority)> {
    if change_on_observed_side && !powered && !has_scheduled_tick && !client_side {
        Some((2, TickPriority::Normal))
    } else {
        None
    }
}

/// `ObserverBlock.tick` (`@0-75`): `POWERED` `true -> false` (`@32`, `flags=2`);
/// `false -> true` (`@57`, `flags=2`) + reschedules off at `+2 NORMAL` (`@64`).
/// Both branches notify the output cell `pos.relative(FACING.getOpposite())`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverTick {
    PowerOff,
    PowerOn { off_after: i32 },
}

#[inline]
pub const fn observer_tick(powered: bool) -> ObserverTick {
    if powered {
        ObserverTick::PowerOff
    } else {
        ObserverTick::PowerOn { off_after: 2 }
    }
}

/// `DispenserBlock.neighborChanged` (`@0-113`): checks `pos` and `pos.above()`
/// (quasi-connectivity `@10-16`); rising edge schedules dispense at `+4 NORMAL`
/// (`@54-55`) and sets `TRIGGERED=true` (`flags=2`, `@75`); falling edge clears `TRIGGERED` (`@109`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispenserEdge {
    Arm { dispense_after: i32 },
    Disarm,
    NoChange,
}

#[inline]
pub const fn dispenser_neighbor_changed(
    triggered: bool,
    signal_at_dispenser: bool,
    signal_at_cell_above: bool,
) -> DispenserEdge {
    let powered = signal_at_dispenser || signal_at_cell_above;
    match (powered, triggered) {
        (true, false) => DispenserEdge::Arm { dispense_after: 4 },
        (false, true) => DispenserEdge::Disarm,
        _ => DispenserEdge::NoChange,
    }
}

/// `DispenseItemBehavior$3.execute` (`@16-89`, filled buckets): tries `emptyContents`
/// at `pos.relative(FACING)`, replacing slot with `BUCKET` on success or ejecting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilledBucketDispense {
    Emptied,
    Ejected,
}

#[inline]
pub const fn filled_bucket_dispense(empty_contents_ok: bool) -> FilledBucketDispense {
    if empty_contents_ok {
        FilledBucketDispense::Emptied
    } else {
        FilledBucketDispense::Ejected
    }
}

/// `DispenseItemBehavior$4.execute` (`@9-133`, empty bucket): picks up liquid
/// via `BucketPickup.pickupBlock` at `pos.relative(FACING)` or ejects if none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyBucketDispense<'a> {
    PickedUp(&'a str),
    Ejected,
}

#[inline]
pub const fn empty_bucket_dispense(picked_up_item: Option<&str>) -> EmptyBucketDispense<'_> {
    match picked_up_item {
        Some(item) => EmptyBucketDispense::PickedUp(item),
        None => EmptyBucketDispense::Ejected,
    }
}
