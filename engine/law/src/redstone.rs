//! Redstone scheduling and circuit laws: repeater (`DiodeBlock`/`RepeaterBlock`),
//! observer (`ObserverBlock`), dispenser (`DispenserBlock`), torch
//! (`RedstoneTorchBlock`/`RedstoneWallTorchBlock`), comparator (`ComparatorBlock`),
//! copper bulb (`CopperBulbBlock`), target (`TargetBlock`), and daylight detector
//! (`DaylightDetectorBlock`) ported bit-for-bit from Mojang 26.2 `server-26.2.jar`.
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

/// `RedstoneTorchBlock` / `RedstoneWallTorchBlock` (`2`-tick delay, `8`-in-`60` burnout, `160`-tick restart).
pub mod torch {
    use super::TickPriority;
    use crate::neighbor::Direction;
    use crate::BlockPos;

    pub const TOGGLE_DELAY: i32 = 2;
    pub const RECENT_TOGGLE_TIMER: i64 = 60;
    pub const MAX_RECENT_TOGGLES: i32 = 8;
    pub const RESTART_DELAY: i32 = 160;

    /// Per-torch `RedstoneTorchBlock$Toggle` timestamps; entries are added on turn-offs only.
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Toggles {
        pub off_times: Vec<i64>,
    }

    impl Toggles {
        /// `RedstoneTorchBlock.tick @23-73`: drop leading entries while `game_time - when > 60`.
        pub fn prune(&mut self, game_time: i64) {
            while let Some(&head) = self.off_times.first() {
                if game_time - head > RECENT_TOGGLE_TIMER {
                    self.off_times.remove(0);
                } else {
                    break;
                }
            }
        }

        /// `RedstoneTorchBlock.isToggledTooFrequently @18-105`:
        /// optionally records `game_time`, then returns `count >= 8`.
        pub fn toggled_too_frequently(&mut self, game_time: i64, record_turn_off: bool) -> bool {
            if record_turn_off {
                self.off_times.push(game_time);
            }
            self.off_times.len() as i32 >= MAX_RECENT_TOGGLES
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct TorchTick {
        pub lit: bool,
        pub burnt_out: bool,
        pub restart_after: Option<(i32, TickPriority)>,
    }

    /// `RedstoneTorchBlock.neighborChanged @0-44`:
    /// schedules `+2 NORMAL` iff `lit == has_neighbor_signal && !will_tick_this_tick`.
    #[inline]
    pub const fn neighbor_changed_schedule(
        lit: bool,
        has_neighbor_signal: bool,
        will_tick_this_tick: bool,
    ) -> Option<(i32, TickPriority)> {
        if lit == has_neighbor_signal && !will_tick_this_tick {
            Some((TOGGLE_DELAY, TickPriority::Normal))
        } else {
            None
        }
    }

    /// `RedstoneTorchBlock.tick @0-193`:
    /// prunes entries older than `60` ticks, turns off when `lit && has_neighbor_signal`
    /// (triggering burnout + `+160 NORMAL` restart if 8+ recent turn-offs), or relights
    /// when `!lit && !has_neighbor_signal && !is_toggled_too_frequently(false)`.
    pub fn tick(
        lit: bool,
        has_neighbor_signal: bool,
        toggles: &mut Toggles,
        game_time: i64,
    ) -> TorchTick {
        toggles.prune(game_time);
        if lit {
            if has_neighbor_signal {
                let burnt_out = toggles.toggled_too_frequently(game_time, true);
                let restart_after = if burnt_out {
                    Some((RESTART_DELAY, TickPriority::Normal))
                } else {
                    None
                };
                return TorchTick {
                    lit: false,
                    burnt_out,
                    restart_after,
                };
            }
        } else if !has_neighbor_signal && !toggles.toggled_too_frequently(game_time, false) {
            return TorchTick {
                lit: true,
                burnt_out: false,
                restart_after: None,
            };
        }
        TorchTick {
            lit,
            burnt_out: false,
            restart_after: None,
        }
    }

    /// `RedstoneTorchBlock.ownSignal @0-22`: `15` if `lit`, `0` otherwise.
    #[inline]
    pub const fn own_signal(lit: bool) -> i32 {
        if lit { 15 } else { 0 }
    }

    /// `RedstoneTorchBlock.getSignal @0-19`: `0` when queried with `Direction::Up`, else `own_signal`.
    #[inline]
    pub const fn standing_get_signal(lit: bool, query_dir: Direction) -> i32 {
        if matches!(query_dir, Direction::Up) {
            0
        } else {
            own_signal(lit)
        }
    }

    /// `RedstoneWallTorchBlock.getSignal @0-23`: `0` when queried with `FACING`, else `own_signal`.
    #[inline]
    pub const fn wall_get_signal(
        lit: bool,
        torch_facing: Direction,
        query_dir: Direction,
    ) -> i32 {
        if (torch_facing as u8) == (query_dir as u8) {
            0
        } else {
            own_signal(lit)
        }
    }

    /// `RedstoneTorchBlock.getDirectSignal @0-18`: delegates to `getSignal` for `Direction::Down` only.
    #[inline]
    pub const fn get_direct_signal(
        lit: bool,
        wall_facing: Option<Direction>,
        query_dir: Direction,
    ) -> i32 {
        if !matches!(query_dir, Direction::Down) {
            return 0;
        }
        match wall_facing {
            None => standing_get_signal(lit, query_dir),
            Some(facing) => wall_get_signal(lit, facing, query_dir),
        }
    }

    /// `RedstoneTorchBlock.hasNeighborSignal @0-11`: queries `(pos.below(), Direction::Down)`.
    #[inline]
    pub const fn standing_input_query(pos: BlockPos) -> (BlockPos, Direction) {
        (Direction::Down.relative(pos, 1), Direction::Down)
    }

    /// `RedstoneWallTorchBlock.hasNeighborSignal @0-27`: queries `(pos.relative(FACING.opposite()), FACING.opposite())`.
    #[inline]
    pub const fn wall_input_query(pos: BlockPos, torch_facing: Direction) -> (BlockPos, Direction) {
        let dir = torch_facing.opposite();
        (dir.relative(pos, 1), dir)
    }
}

/// `ComparatorBlock` (extends `DiodeBlock`): `2`-tick delay, `HIGH` when facing a diode,
/// `COMPARE` / `SUBTRACT`, analog container & solid-conductor read-through inputs.
pub mod comparator {
    use super::TickPriority;
    use crate::math;

    pub const DELAY: i32 = 2;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        Compare,
        Subtract,
    }

    /// `ComparatorBlock.checkTickOnNeighbor @91-114`:
    /// `shouldPrioritize ? TickPriority.HIGH : TickPriority.NORMAL`.
    #[inline]
    pub const fn priority(should_prioritize: bool) -> TickPriority {
        if should_prioritize {
            TickPriority::High
        } else {
            TickPriority::Normal
        }
    }

    /// `ComparatorBlock.checkTickOnNeighbor @0-117`:
    /// schedules `+2` (`HIGH` if `should_prioritize`, else `NORMAL`) when `!will_tick_this_tick`
    /// and either `calculated_output != stored_output` or `powered != should_turn_on`.
    #[inline]
    pub const fn check_tick_on_neighbor(
        calculated_output: i32,
        stored_output: i32,
        powered: bool,
        should_turn_on: bool,
        will_tick_this_tick: bool,
        should_prioritize: bool,
    ) -> Option<(i32, TickPriority)> {
        if will_tick_this_tick {
            return None;
        }
        if calculated_output != stored_output || powered != should_turn_on {
            Some((DELAY, priority(should_prioritize)))
        } else {
            None
        }
    }

    /// `ComparatorBlock.calculateOutputSignal @0-55`.
    #[inline]
    pub const fn calculate_output_signal(input: i32, side: i32, mode: Mode) -> i32 {
        if input == 0 || side > input {
            return 0;
        }
        match mode {
            Mode::Subtract => input - side,
            Mode::Compare => input,
        }
    }

    /// `ComparatorBlock.shouldTurnOn @0-59`.
    #[inline]
    pub const fn should_turn_on(input: i32, side: i32, mode: Mode) -> bool {
        if input == 0 {
            return false;
        }
        if input > side {
            return true;
        }
        input == side && matches!(mode, Mode::Compare)
    }

    /// `DiodeBlock.getInputSignal @20-82`:
    /// wire `POWER` behind overrides if `< 15` and larger than `behind_signal`.
    #[inline]
    pub fn diode_input_signal(
        behind_signal: i32,
        behind_is_wire: bool,
        behind_wire_power: i32,
    ) -> i32 {
        if behind_signal >= 15 {
            return behind_signal;
        }
        let wire = if behind_is_wire { behind_wire_power } else { 0 };
        behind_signal.max(wire)
    }

    /// `ComparatorBlock.getInputSignal @0-168`:
    /// - direct analog block behind overrides `behind_diode_signal`;
    /// - else if `behind_diode_signal < 15 && behind_is_conductor`, reads one further back
    ///   (`max(far_item_frame, far_analog)`) and overrides if either is `Some`.
    #[inline]
    pub fn input_signal(
        behind_diode_signal: i32,
        behind_analog: Option<i32>,
        behind_is_conductor: bool,
        far_item_frame: Option<i32>,
        far_analog: Option<i32>,
    ) -> i32 {
        if let Some(analog) = behind_analog {
            return analog;
        }
        if behind_diode_signal < 15 && behind_is_conductor {
            if let Some(value) = far_item_frame.max(far_analog) {
                return value;
            }
        }
        behind_diode_signal
    }

    /// `SignalGetter.getControlInputSignal @0-144` (`diodes_only = false` for comparator sides,
    /// `true` for repeater lock side).
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub const fn control_input_signal(
        is_redstone_block: bool,
        is_wire: bool,
        wire_power: i32,
        is_signal_source: bool,
        direct_signal: i32,
        is_diode: bool,
        diodes_only: bool,
    ) -> i32 {
        if diodes_only {
            return if is_diode { direct_signal } else { 0 };
        }
        if is_redstone_block {
            return 15;
        }
        if is_wire {
            return wire_power;
        }
        if is_signal_source {
            return direct_signal;
        }
        0
    }

    /// `DiodeBlock.getAlternateSignal @32-67`: max of left/right control inputs.
    #[inline]
    pub fn alternate_signal(left: i32, right: i32) -> i32 {
        left.max(right)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ComparatorRefresh {
        pub stored_output: i32,
        pub powered: bool,
        pub set_powered_flag2: Option<bool>,
        pub update_neighbors_in_front: bool,
    }

    /// `ComparatorBlock.refreshOutputState @0-164`:
    /// writes `calculated_output` to the BE, and if `stored_output != calculated_output || mode != Compare`,
    /// updates `POWERED` (with `setBlock` flag `2`) and calls `updateNeighborsInFront`.
    #[inline]
    pub const fn refresh_output_state(
        stored_output: i32,
        calculated_output: i32,
        mode: Mode,
        current_powered: bool,
        should_turn_on: bool,
    ) -> ComparatorRefresh {
        if stored_output == calculated_output && matches!(mode, Mode::Compare) {
            return ComparatorRefresh {
                stored_output: calculated_output,
                powered: current_powered,
                set_powered_flag2: None,
                update_neighbors_in_front: false,
            };
        }
        let (powered, set_powered_flag2) = if current_powered && !should_turn_on {
            (false, Some(false))
        } else if !current_powered && should_turn_on {
            (true, Some(true))
        } else {
            (current_powered, None)
        };
        ComparatorRefresh {
            stored_output: calculated_output,
            powered,
            set_powered_flag2,
            update_neighbors_in_front: true,
        }
    }

    /// `Mth.lerpDiscrete(fill, 0, 15)` (`@0-27`):
    /// `Mth.floor(fill * 14.0f) + (fill > 0.0f ? 1 : 0)`.
    #[inline]
    pub fn container_analog_from_fill(fill: f32) -> i32 {
        math::floor_f32(fill * 14.0f32) + i32::from(fill > 0.0f32)
    }

    /// `AbstractContainerMenu.getRedstoneSignalFromContainer @0-75`:
    /// `slots` is `(count, max_stack_size)` for non-empty slots.
    pub fn container_analog_from_slots(container_size: usize, slots: &[(i32, i32)]) -> i32 {
        if container_size == 0 {
            return 0;
        }
        let mut fill = 0.0f32;
        for &(count, max_stack) in slots {
            if count > 0 && max_stack > 0 {
                fill += (count as f32) / (max_stack as f32);
            }
        }
        fill /= container_size as f32;
        container_analog_from_fill(fill)
    }
}

/// `CopperBulbBlock` (and `WeatheringCopperBulbBlock`): zero-delay rising-edge toggle,
/// synchronous in `neighborChanged` / `onPlace`.
pub mod copper_bulb {
    /// `CopperBulbBlock.checkAndFlip @0-122`:
    /// returns `Some((new_lit, new_powered))` to write with `setBlock(..., 3)`,
    /// or `None` when `signal == powered`. Only the rising edge (`!powered && signal`) flips `LIT`.
    #[inline]
    pub const fn check_and_flip(lit: bool, powered: bool, signal: bool) -> Option<(bool, bool)> {
        if signal == powered {
            return None;
        }
        let new_lit = if !powered { !lit } else { lit };
        Some((new_lit, signal))
    }

    /// `CopperBulbBlock.getAnalogOutputSignal @0-26`: `15` when `lit`, `0` when dark.
    #[inline]
    pub const fn analog_output(lit: bool) -> i32 {
        if lit { 15 } else { 0 }
    }
}

/// `TargetBlock`: radial hit offset -> `1..=15`, `20`/`8`-tick reset pulse.
pub mod target {
    use super::TickPriority;
    use crate::math;
    use crate::shapes::Axis;

    pub const ACTIVATION_TICKS_ARROWS: i32 = 20;
    pub const ACTIVATION_TICKS_OTHER: i32 = 8;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct TargetHit {
        pub strength: i32,
        /// Present when `!has_scheduled_tick`: `(output_power, reset_delay, priority)`.
        pub set_output: Option<(i32, i32, TickPriority)>,
    }

    /// `Mth.frac(double) @0-7`: `v - (double) Mth.lfloor(v)`.
    #[inline]
    fn frac(v: f64) -> f64 {
        v - (math::lfloor(v) as f64)
    }

    /// `TargetBlock.updateRedstoneOutput @10-24`: `20` for `AbstractArrow`, `8` otherwise.
    #[inline]
    pub const fn activation_delay(is_arrow: bool) -> i32 {
        if is_arrow {
            ACTIVATION_TICKS_ARROWS
        } else {
            ACTIVATION_TICKS_OTHER
        }
    }

    /// `TargetBlock.getRedstoneStrength @0-131`:
    /// `max(1, Mth.ceil(15.0 * Mth.clamp((0.5 - offset) / 0.5, 0.0, 1.0)))`.
    pub fn redstone_strength(hit_x: f64, hit_y: f64, hit_z: f64, face_axis: Axis) -> i32 {
        let d = (frac(hit_x) - 0.5).abs();
        let e = (frac(hit_y) - 0.5).abs();
        let f = (frac(hit_z) - 0.5).abs();
        let offset = match face_axis {
            Axis::Y => d.max(f),
            Axis::Z => d.max(e),
            Axis::X => e.max(f),
        };
        let ratio = ((0.5 - offset) / 0.5).clamp(0.0, 1.0);
        math::ceil(15.0 * ratio).max(1)
    }

    /// `TargetBlock.updateRedstoneOutput @0-63`:
    /// always returns `strength`; sets `OUTPUT_POWER` and schedules `+20`/`+8 NORMAL` iff `!has_scheduled_tick`.
    pub fn on_projectile_hit(
        hit_x: f64,
        hit_y: f64,
        hit_z: f64,
        face_axis: Axis,
        is_arrow: bool,
        has_scheduled_tick: bool,
    ) -> TargetHit {
        let strength = redstone_strength(hit_x, hit_y, hit_z, face_axis);
        let set_output = if has_scheduled_tick {
            None
        } else {
            Some((strength, activation_delay(is_arrow), TickPriority::Normal))
        };
        TargetHit {
            strength,
            set_output,
        }
    }

    /// `TargetBlock.tick @0-37`: resets `OUTPUT_POWER` to `0` (`flag 3`) if non-zero.
    #[inline]
    pub const fn tick_reset(output_power: i32) -> Option<i32> {
        if output_power != 0 {
            Some(0)
        } else {
            None
        }
    }
}

/// `DaylightDetectorBlock`: effective sky brightness, sun angle `0.2` blend + cosine,
/// inverted `15 - i`, and `20`-tick cadence.
pub mod daylight {
    use crate::math;

    pub const UPDATE_INTERVAL_TICKS: i64 = 20;

    /// `DaylightDetectorBlock.updateSignalStrength @0-127`:
    /// `sky_brightness` is `Level.getEffectiveSkyBrightness(pos)` (`skyLight - skyDarken`),
    /// `sun_angle_deg` is `EnvironmentAttributes.SUN_ANGLE` in degrees.
    pub fn update_signal_strength(sky_brightness: i32, sun_angle_deg: f32, inverted: bool) -> i32 {
        let mut i = sky_brightness;
        let mut angle = sun_angle_deg * 0.017453292f32;
        if inverted {
            i = 15 - i;
        } else if i > 0 {
            let target = if angle < 3.1415927f32 {
                0.0f32
            } else {
                6.2831855f32
            };
            angle += (target - angle) * 0.2f32;
            i = math::round((i as f32) * math::cos(angle as f64));
        }
        i.clamp(0, 15)
    }

    /// `DaylightDetectorBlock.tickEntity @0-19`: recomputes on `game_time % 20 == 0`.
    #[inline]
    pub const fn ticks_this_game_tick(game_time: i64) -> bool {
        game_time % UPDATE_INTERVAL_TICKS == 0
    }
}
