/// A block at a position. The adapter changes the identity on replacement,
/// including replacement by the same block state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiningTarget {
    pub position: [i32; 3],
    pub identity: u64,
}

/// A single `Tool.Rule` entry evaluated against a target block (`matches` comes
/// from the block's tag/id membership in the block table).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolRule {
    pub matches: bool,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}

/// Evaluates `Tool.getMiningSpeed` (`@0-71`) and `Tool.isCorrectForDrops` (`@0-68`)
/// as two independent first-match scans over `rules`.
pub fn evaluate_tool_rules(rules: &[ToolRule], default_mining_speed: f32) -> (f32, bool) {
    let speed = rules
        .iter()
        .find(|rule| rule.matches && rule.speed.is_some())
        .and_then(|rule| rule.speed)
        .unwrap_or(default_mining_speed);
    let correct_for_drops = rules
        .iter()
        .find(|rule| rule.matches && rule.correct_for_drops.is_some())
        .and_then(|rule| rule.correct_for_drops)
        .unwrap_or(false);
    (speed, correct_for_drops)
}

/// `Player.hasCorrectToolForDrops` (`@0-26`): blocks that do not require a
/// correct tool always drop; otherwise the selected tool's rule scan decides.
pub fn has_correct_tool_for_drops(
    requires_correct_tool_for_drops: bool,
    tool_correct_for_drops: bool,
) -> bool {
    !requires_correct_tool_for_drops || tool_correct_for_drops
}

/// `MobEffectInstance` constructor amplifier clamp (`Mth.clamp(amp, 0, 255)`).
pub fn mob_effect_amplifier(raw_amplifier: i32) -> i32 {
    raw_amplifier.clamp(0, 255)
}

/// `MobEffectUtil.getDigSpeedAmplification` (`@0-51`): max amplifier over present
/// `HASTE` and `CONDUIT_POWER` effects, or `None` when neither is active.
pub fn dig_speed_amplification(haste_amp: Option<i32>, conduit_amp: Option<i32>) -> Option<i32> {
    match (haste_amp, conduit_amp) {
        (None, None) => None,
        (Some(h), None) => Some(mob_effect_amplifier(h).max(0)),
        (None, Some(c)) => Some(mob_effect_amplifier(c).max(0)),
        (Some(h), Some(c)) => Some(mob_effect_amplifier(h).max(mob_effect_amplifier(c)).max(0)),
    }
}

/// Inputs to `Player.getDestroySpeed` (`@0-175`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningModifiers {
    pub tool_base_speed: f32,
    pub mining_efficiency: f64,
    pub haste_or_conduit_amp: Option<i32>,
    pub fatigue_amp: Option<i32>,
    pub block_break_speed: f64,
    pub eye_in_water: bool,
    pub submerged_mining_speed: f64,
    pub on_ground: bool,
}

impl Default for MiningModifiers {
    fn default() -> Self {
        Self {
            tool_base_speed: 1.0,
            mining_efficiency: 0.0,
            haste_or_conduit_amp: None,
            fatigue_amp: None,
            block_break_speed: 1.0,
            eye_in_water: false,
            submerged_mining_speed: 0.2,
            on_ground: true,
        }
    }
}

/// `Player.getDestroySpeed` (`@0-175`) in Mojang's exact `f32` operation order.
pub fn player_destroy_speed(modifiers: MiningModifiers) -> f32 {
    let mut f = modifiers.tool_base_speed;
    if f > 1.0f32 {
        f += modifiers.mining_efficiency as f32;
    }
    if let Some(amp) = modifiers.haste_or_conduit_amp {
        f *= 1.0f32 + ((amp + 1) as f32) * 0.2f32;
    }
    if let Some(amp) = modifiers.fatigue_amp {
        let factor = match amp {
            0 => 0.3f32,
            1 => 0.09f32,
            2 => 0.0027f32,
            _ => 8.1e-4f32,
        };
        f *= factor;
    }
    f *= modifiers.block_break_speed as f32;
    if modifiers.eye_in_water {
        f *= modifiers.submerged_mining_speed as f32;
    }
    if !modifiers.on_ground {
        f /= 5.0f32;
    }
    f
}

impl MiningModifiers {
    pub fn destroy_speed(self) -> f32 {
        player_destroy_speed(self)
    }

    pub fn into_sample(self, target: MiningTarget, hardness: f32, correct_tool: bool) -> MiningSample {
        MiningSample {
            target,
            destroy_speed: self.destroy_speed(),
            hardness,
            correct_tool,
        }
    }
}

/// The current non-air block and evaluated inputs to vanilla's default block
/// destroy-progress law.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningSample {
    pub target: MiningTarget,
    pub destroy_speed: f32,
    pub hardness: f32,
    pub correct_tool: bool,
}

impl MiningSample {
    pub fn progress(self) -> f32 {
        if self.hardness == -1.0 {
            return 0.0;
        }
        let divisor = if self.correct_tool { 30.0 } else { 100.0 };
        self.destroy_speed / self.hardness / divisor
    }

    /// First tick count `n >= 1` where `self.progress() * (n as f32) >= 1.0`
    /// (the same expression as `MiningWork::progress`).
    pub fn break_ticks(self) -> Option<u64> {
        let p = self.progress();
        if !(p > 0.0) || p * (u64::MAX as f32) < 1.0 {
            return None;
        }
        let mut lo = 1u64;
        let mut hi = u64::MAX;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if p * (mid as f32) >= 1.0 {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        Some(lo)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiningWork {
    pub target: MiningTarget,
    pub start_tick: i32,
}

impl MiningWork {
    fn progress(self, game_ticks: i32, sample: MiningSample) -> f32 {
        let ticks = game_ticks.wrapping_sub(self.start_tick).wrapping_add(1);
        sample.progress() * ticks as f32
    }
}

#[cfg(test)]
mod destroy_speed_tests {
    use super::*;

    #[test]
    fn jar_destroy_speed_and_progress_witnesses() {
        let target = MiningTarget {
            position: [0, 0, 0],
            identity: 1,
        };
        let eff5 = f32::from_bits(0x41d0_0000) as f64; // probe-output.txt:7 (26.0f)
        let eff3 = f32::from_bits(0x4120_0000) as f64; // checker-output.txt:13 (10.0f)
        let sub_default = f32::from_bits(0x3e4c_cccd) as f64; // probe-output.txt:13 (0.2f)
        let sub_aa = f32::from_bits(0x3f80_0000) as f64; // probe-output.txt:17 (1.0f)
        let stone_hard = f32::from_bits(0x3fc0_0000); // probe-output.txt:2 (1.5f)
        let bedrock_hard = f32::from_bits(0xbf80_0000); // probe-output.txt:3 (-1.0f)
        let dirt_hard = f32::from_bits(0x3f00_0000); // checker-output.txt:14 (0.5f)
        let obsidian_hard = f32::from_bits(0x4248_0000); // checker-output.txt:16 (50.0f)

        // (name, modifiers, hardness, correct_tool, expected_p_bits, expected_k_break)
        let cases: &[(&str, MiningModifiers, f32, bool, u32, Option<u64>)] = &[
            // Author probe-output.txt:13-25
            (
                "hand-ground", // probe-output.txt:13
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x3f80_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                false,
                0x3bda_740e,
                Some(150),
            ),
            (
                "ironpick-ground", // probe-output.txt:14
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e08_8889,
                Some(8),
            ),
            (
                "ironpick-eff5-ground", // probe-output.txt:15
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    mining_efficiency: eff5,
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3f36_0b61,
                Some(2),
            ),
            (
                "ironpick-water-noAA", // probe-output.txt:16
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    eye_in_water: true,
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3cda_740e,
                Some(38),
            ),
            (
                "ironpick-water-AA", // probe-output.txt:17
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    eye_in_water: true,
                    submerged_mining_speed: sub_aa,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e08_8889,
                Some(8),
            ),
            (
                "ironpick-air", // probe-output.txt:18
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    submerged_mining_speed: sub_default,
                    on_ground: false,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3cda_740e,
                Some(38),
            ),
            (
                "ironpick-haste2-ground", // probe-output.txt:19
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    haste_or_conduit_amp: dig_speed_amplification(Some(1), None),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e3f_258c,
                Some(6),
            ),
            (
                "ironpick-fatigue1-ground", // probe-output.txt:20
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    fatigue_amp: Some(mob_effect_amplifier(0)),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3d23_d70b,
                Some(25),
            ),
            (
                "ironpick-conduit2-ground", // probe-output.txt:21
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    haste_or_conduit_amp: dig_speed_amplification(None, Some(1)),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e3f_258c,
                Some(6),
            ),
            (
                "hoe-eff5-ignored-gate", // probe-output.txt:22
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x3f80_0000),
                    mining_efficiency: eff5,
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                false,
                0x3bda_740e,
                Some(150),
            ),
            (
                "bedrock-hand-ground", // probe-output.txt:23
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x3f80_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                bedrock_hard,
                true,
                0x0000_0000,
                None,
            ),
            (
                "ironpick-fatigue4-ground", // probe-output.txt:24
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    fatigue_amp: Some(mob_effect_amplifier(3)),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x38e2_7e0e,
                Some(9260),
            ),
            (
                "ironpick-eff5-haste2-fatigue1-water-air", // probe-output.txt:25
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    mining_efficiency: eff5,
                    haste_or_conduit_amp: dig_speed_amplification(Some(1), None),
                    fatigue_amp: Some(mob_effect_amplifier(0)),
                    block_break_speed: 1.0,
                    eye_in_water: true,
                    submerged_mining_speed: sub_aa,
                    on_ground: false,
                },
                stone_hard,
                true,
                0x3d74_aaf2,
                Some(17),
            ),
            // Checker checker-output.txt:14-23
            (
                "dirt-hand-ground", // checker-output.txt:14
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x3f80_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                dirt_hard,
                true,
                0x3d88_8889,
                Some(15),
            ),
            (
                "dirt-ironshovel-ground", // checker-output.txt:15
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                dirt_hard,
                true,
                0x3ecc_cccd,
                Some(3),
            ),
            (
                "obsidian-hand-ground", // checker-output.txt:16 (probe's ceil(1/p) in f64 prints k=5001, but vanilla breaks at first n with p * n as f32 >= 1.0, which is 5000)
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x3f80_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                obsidian_hard,
                false,
                0x3951_b717,
                Some(5000),
            ),
            (
                "obsidian-ironpick-ground", // checker-output.txt:17
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                obsidian_hard,
                false,
                0x3a9d_4951,
                Some(834),
            ),
            (
                "ironpick-haste1-ground", // checker-output.txt:18
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    haste_or_conduit_amp: dig_speed_amplification(Some(0), None),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e23_d70b,
                Some(7),
            ),
            (
                "ironpick-haste1-conduit3-ground", // checker-output.txt:19
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    haste_or_conduit_amp: dig_speed_amplification(Some(0), Some(2)),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e5a_740e,
                Some(5),
            ),
            (
                "ironpick-fatigue5-ground", // checker-output.txt:20
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    fatigue_amp: Some(mob_effect_amplifier(4)),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x38e2_7e0e,
                Some(9260),
            ),
            (
                "ironpick-fatigue-neg1-ground", // checker-output.txt:21 (MobEffectInstance clamps -1 -> 0)
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    fatigue_amp: Some(mob_effect_amplifier(-1)),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3d23_d70b,
                Some(25),
            ),
            (
                "woodpick-eff3-ground", // checker-output.txt:22
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x4000_0000),
                    mining_efficiency: eff3,
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                stone_hard,
                true,
                0x3e88_8889,
                Some(4),
            ),
            (
                "ironpick-eff5-haste2-fatigue1-bbs3.5-water-air", // checker-output.txt:23
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x40c0_0000),
                    mining_efficiency: eff5,
                    haste_or_conduit_amp: dig_speed_amplification(Some(1), None),
                    fatigue_amp: Some(mob_effect_amplifier(0)),
                    block_break_speed: f32::from_bits(0x4060_0000) as f64,
                    eye_in_water: true,
                    submerged_mining_speed: sub_default,
                    on_ground: false,
                },
                stone_hard,
                true,
                0x3d2b_4475,
                Some(24),
            ),
        ];

        for &(name, modifiers, hardness, correct_tool, expected_bits, expected_k) in cases {
            let sample = modifiers.into_sample(target, hardness, correct_tool);
            let p = sample.progress();
            assert_eq!(
                p.to_bits(),
                expected_bits,
                "{name}: got 0x{:08x}, expected 0x{expected_bits:08x}",
                p.to_bits()
            );
            assert_eq!(sample.break_ticks(), expected_k, "{name} k_break");
        }

        // Why obsidian-hand-ground is 5000 (not 5001): p = 0x3951_b717 < 0.0002 in f64
        // so ceil(1/p) = 5001, whereas MiningWork::progress evaluates p * (n as f32) >= 1.0
        // and rounds p * 5000.0f32 up to 1.0f32.
        let obsidian_hand = MiningModifiers {
            tool_base_speed: f32::from_bits(0x3f80_0000),
            submerged_mining_speed: sub_default,
            ..MiningModifiers::default()
        }
        .into_sample(target, obsidian_hard, false);
        let work = MiningWork { target, start_tick: 0 };
        assert!(work.progress(4998, obsidian_hand) < 1.0);
        assert!(work.progress(4999, obsidian_hand) >= 1.0);
        assert_eq!(obsidian_hand.break_ticks(), Some(5000));
    }

    #[test]
    fn tool_rules_first_match_scans_independently() {
        // Iron pickaxe rules from probe-output.txt:4:
        // rule 0: incorrect_for_iron_tool -> speed=None, correct_for_drops=Some(false)
        // rule 1: mineable/pickaxe -> speed=Some(6.0), correct_for_drops=Some(true)
        let iron_pick_obsidian = [
            ToolRule { matches: true, speed: None, correct_for_drops: Some(false) },
            ToolRule { matches: true, speed: Some(6.0), correct_for_drops: Some(true) },
        ];
        let (speed, drops) = evaluate_tool_rules(&iron_pick_obsidian, 1.0);
        assert_eq!(speed.to_bits(), 0x40c0_0000);
        assert!(!has_correct_tool_for_drops(true, drops));

        let iron_pick_stone = [
            ToolRule { matches: false, speed: None, correct_for_drops: Some(false) },
            ToolRule { matches: true, speed: Some(6.0), correct_for_drops: Some(true) },
        ];
        let (speed, drops) = evaluate_tool_rules(&iron_pick_stone, 1.0);
        assert_eq!(speed.to_bits(), 0x40c0_0000);
        assert!(has_correct_tool_for_drops(true, drops));

        // Dirt with bare hand: requiresCorrectToolForDrops = false -> correct = true
        assert!(has_correct_tool_for_drops(false, false));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameModeKind {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameModeKind {
    pub const fn is_block_placing_restricted(self) -> bool {
        matches!(self, Self::Adventure | Self::Spectator)
    }
}

/// `Player.blockActionRestricted(Level, BlockPos, GameType)` (`player-full.txt:181-216`).
pub fn block_action_restricted(
    mode: GameModeKind,
    may_build: bool,
    main_hand_empty: bool,
    can_break_in_adventure: bool,
) -> bool {
    if !mode.is_block_placing_restricted() {
        return false;
    }
    if mode == GameModeKind::Spectator {
        return true;
    }
    if may_build {
        return false;
    }
    main_hand_empty || !can_break_in_adventure
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakPacketAction {
    Start,
    Stop,
    Abort,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BreakPacketClock {
    pub game_ticks: i32,
    pub is_destroying_block: bool,
    pub destroy_pos: [i32; 3],
    pub destroy_progress_start: i32,
    pub has_delayed_destroy: bool,
    pub delayed_destroy_pos: [i32; 3],
    pub delayed_tick_start: i32,
    pub last_sent_state: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakGateInput {
    pub action: BreakPacketAction,
    pub pos: [i32; 3],
    pub eye_pos: [f64; 3],
    pub block_interaction_range: f64,
    pub max_y: i32,
    pub under_spawn_protection: bool,
    pub may_interact: bool,
    pub instabuild: bool,
    pub action_restricted: bool,
    pub target_is_air: bool,
    /// `BlockBehaviour.getDestroyProgress` when `target_is_air == false`.
    pub destroy_progress: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakPacketOutcome {
    RefusedTooFar,
    RefusedTooHighSendBlockUpdate([i32; 3]),
    RefusedSpawnProtection([i32; 3]),
    RefusedMayNotInteractSendBlockUpdate([i32; 3]),
    RefusedActionRestrictedSendBlockUpdate([i32; 3]),
    CreativeDestroyAndAck([i32; 3]),
    SurvivalStartArmed {
        pos: [i32; 3],
        stage: i32,
        reset_old_pos: Option<[i32; 3]>,
    },
    SurvivalStartInstaDestroy([i32; 3]),
    StopNoopOtherPos,
    StopNoopDelayedPending {
        world_read: bool,
    },
    StopDestroyed([i32; 3]),
    StopHandedOffToDelayed([i32; 3]),
    Aborted {
        pos: [i32; 3],
        mismatched_destroy_pos: Option<[i32; 3]>,
    },
}

/// `ServerPlayerGameMode.handleBlockBreakAction` (`spgm-full.txt` @0-740):
/// reach -> `pos[1] > max_y` -> START/STOP/ABORT dispatch.
pub fn handle_block_break_action(
    clock: &mut BreakPacketClock,
    input: BreakGateInput,
) -> BreakPacketOutcome {
    let [x, y, z] = input.eye_pos;
    if !crate::interact::is_within_block_interaction_range(
        crate::Vec3::new(x, y, z), input.block_interaction_range, input.pos,
        crate::interact::SERVER_INTERACTION_RANGE_TOLERANCE,
    ) {
        return BreakPacketOutcome::RefusedTooFar;
    }
    if input.pos[1] > input.max_y {
        return BreakPacketOutcome::RefusedTooHighSendBlockUpdate(input.pos);
    }
    match input.action {
        BreakPacketAction::Start => {
            if input.under_spawn_protection {
                return BreakPacketOutcome::RefusedSpawnProtection(input.pos);
            }
            if !input.may_interact {
                return BreakPacketOutcome::RefusedMayNotInteractSendBlockUpdate(input.pos);
            }
            if input.instabuild {
                return BreakPacketOutcome::CreativeDestroyAndAck(input.pos);
            }
            if input.action_restricted {
                return BreakPacketOutcome::RefusedActionRestrictedSendBlockUpdate(input.pos);
            }
            let f = if input.target_is_air {
                1.0f32
            } else {
                input.destroy_progress
            };
            if !input.target_is_air && f >= 1.0f32 {
                return BreakPacketOutcome::SurvivalStartInstaDestroy(input.pos);
            }
            let reset_old_pos = if clock.is_destroying_block {
                Some(clock.destroy_pos)
            } else {
                None
            };
            clock.destroy_progress_start = clock.game_ticks;
            clock.is_destroying_block = true;
            clock.destroy_pos = input.pos;
            let stage = (f * 10.0f32) as i32;
            clock.last_sent_state = stage;
            BreakPacketOutcome::SurvivalStartArmed {
                pos: input.pos,
                stage,
                reset_old_pos,
            }
        }
        BreakPacketAction::Stop => {
            if input.pos != clock.destroy_pos {
                return BreakPacketOutcome::StopNoopOtherPos;
            }
            if clock.has_delayed_destroy {
                return BreakPacketOutcome::StopNoopDelayedPending { world_read: true };
            }
            let f = if input.target_is_air {
                0.0f32
            } else {
                let ticks = clock.game_ticks.wrapping_sub(clock.destroy_progress_start).wrapping_add(1);
                input.destroy_progress * (ticks as f32)
            };
            if !input.target_is_air && f >= 0.7f32 {
                clock.is_destroying_block = false;
                BreakPacketOutcome::StopDestroyed(input.pos)
            } else {
                clock.is_destroying_block = false;
                clock.has_delayed_destroy = true;
                clock.delayed_destroy_pos = input.pos;
                clock.delayed_tick_start = clock.destroy_progress_start;
                BreakPacketOutcome::StopHandedOffToDelayed(input.pos)
            }
        }
        BreakPacketAction::Abort => {
            clock.is_destroying_block = false;
            let mismatched = (clock.destroy_pos != input.pos).then_some(clock.destroy_pos);
            BreakPacketOutcome::Aborted {
                pos: input.pos,
                mismatched_destroy_pos: mismatched,
            }
        }
    }
}

/// `Block.playerDestroy` exhaustion literal (`0.005f`).
pub const BLOCK_DESTROY_EXHAUSTION: f32 = 0.005f32;
/// `FoodData.addExhaustion` cap (`40.0f`).
pub const FOOD_EXHAUSTION_CAP: f32 = 40.0f32;

/// `FoodData.addExhaustion(float)` (`@0-15`).
pub fn add_exhaustion(current: f32, delta: f32) -> f32 {
    (current + delta).min(FOOD_EXHAUSTION_CAP)
}

/// `Player.causeFoodExhaustion(float)` (`player-full` @3864-29).
pub fn cause_food_exhaustion(
    current: f32,
    delta: f32,
    invulnerable: bool,
    is_client_side: bool,
) -> f32 {
    if invulnerable || is_client_side {
        current
    } else {
        add_exhaustion(current, delta)
    }
}

/// `Item.canDestroyBlock` (`@0-58`): TOOL with `canDestroyBlocksInCreative == false`
/// refuses when `instabuild == true`.
pub fn can_destroy_block(
    tool_can_destroy_in_creative: Option<bool>,
    instabuild: bool,
) -> bool {
    !(instabuild && tool_can_destroy_in_creative == Some(false))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemovedReplacement {
    Air,
    WaterSource,
}

/// `Level.removeBlock(BlockPos, boolean isMoving)` (`@0-27`):
/// replaces with `fluid.createLegacyBlock()` (`WaterSource` if waterlogged, else `Air`),
/// `flags = 3 | (if is_moving { 64 } else { 0 })`.
pub const fn remove_block_write(
    has_water_fluid: bool,
    is_moving: bool,
) -> (RemovedReplacement, u8) {
    let replacement = if has_water_fluid {
        RemovedReplacement::WaterSource
    } else {
        RemovedReplacement::Air
    };
    let flags = 3 | if is_moving { 64 } else { 0 };
    (replacement, flags)
}

/// `Item.mineBlock` -> `ItemStack.hurtAndBreak` (`@356-385`, `@6-11`).
pub fn item_mine_block_durability_cost(
    has_tool_component: bool,
    damage_per_block: i32,
    block_destroy_speed: f32,
    is_client_side: bool,
    is_server_level: bool,
) -> i32 {
    if has_tool_component
        && !is_client_side
        && block_destroy_speed != 0.0f32
        && damage_per_block > 0
        && is_server_level
    {
        damage_per_block
    } else {
        0
    }
}

/// Data-driven partner removal rule from the block table (`playerWillDestroy` overriders).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartnerRemovalRule {
    None,
    /// Double-height blocks (`DoublePlantBlock.preventDropFromBottomPart`):
    /// acts only when `is_upper` (UPPER) is broken and cell below matches;
    /// `check_incorrect_tool` is `true` for doors (`preventsBlockDrops || !hasCorrectToolForDrops`)
    /// and `false` for tall plants (`preventsBlockDrops` only).
    BelowPartner {
        is_upper: bool,
        partner_matches: bool,
        partner_has_water: bool,
        check_incorrect_tool: bool,
    },
    /// Foot-to-head partner (`BedBlock.playerWillDestroy`): creative (`preventsBlockDrops`) + `is_foot` only.
    OffsetPartner {
        is_foot: bool,
        partner_offset: [i32; 3],
        partner_matches: bool,
    },
    /// Base destruction (`PistonHeadBlock.playerWillDestroy`): creative + fitting base -> `destroyBlock(base, false)`.
    DestroyPartnerNoDrops {
        partner_offset: [i32; 3],
        partner_matches: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestroySideEffect {
    SendBlockUpdate([i32; 3], u8),
    LevelEvent2001([i32; 3]),
    GameEventBlockDestroy([i32; 3]),
    SetBlock([i32; 3], RemovedReplacement, u8),
    DestroyBlockNoDrops([i32; 3]),
    DamageTool(i32),
    DropResources([i32; 3]),
}

#[derive(Clone, Copy, Debug)]
pub struct DestroyBlockInput {
    pub pos: [i32; 3],
    pub is_client_side: bool,
    pub is_server_level: bool,
    pub tool_can_destroy_in_creative: Option<bool>,
    pub has_tool_component: bool,
    pub tool_damage_per_block: i32,
    pub tool_correct_for_drops: bool,
    pub block_requires_correct_tool: bool,
    pub block_destroy_speed: f32,
    pub is_game_master_block: bool,
    pub can_use_game_master_blocks: bool,
    pub action_restricted: bool,
    pub prevents_block_drops: bool,
    pub invulnerable: bool,
    pub has_water_fluid: bool,
    pub partner_rule: PartnerRemovalRule,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DestroyBlockResult {
    pub succeeded: bool,
    pub exhaustion_added: f32,
    pub effects: Vec<DestroySideEffect>,
}

/// `ServerPlayerGameMode.destroyBlock` (`spgm-destroyBlock.txt` @0-273) mode-unified chain.
pub fn destroy_block(input: DestroyBlockInput) -> DestroyBlockResult {
    let mut effects = Vec::new();
    if !can_destroy_block(input.tool_can_destroy_in_creative, input.prevents_block_drops) {
        return DestroyBlockResult { succeeded: false, exhaustion_added: 0.0, effects };
    }
    if input.is_game_master_block && !input.can_use_game_master_blocks {
        effects.push(DestroySideEffect::SendBlockUpdate(input.pos, 3));
        return DestroyBlockResult { succeeded: false, exhaustion_added: 0.0, effects };
    }
    if input.action_restricted {
        return DestroyBlockResult { succeeded: false, exhaustion_added: 0.0, effects };
    }

    let can_harvest = has_correct_tool_for_drops(
        input.block_requires_correct_tool,
        input.tool_correct_for_drops,
    );

    if !input.is_client_side {
        match input.partner_rule {
            PartnerRemovalRule::None => {}
            PartnerRemovalRule::BelowPartner {
                is_upper,
                partner_matches,
                partner_has_water,
                check_incorrect_tool,
            } => {
                let trigger = input.prevents_block_drops || (check_incorrect_tool && !can_harvest);
                if trigger && is_upper && partner_matches {
                    let partner_pos = [input.pos[0], input.pos[1] - 1, input.pos[2]];
                    let repl = if partner_has_water {
                        RemovedReplacement::WaterSource
                    } else {
                        RemovedReplacement::Air
                    };
                    effects.push(DestroySideEffect::SetBlock(partner_pos, repl, 35));
                    effects.push(DestroySideEffect::LevelEvent2001(partner_pos));
                }
            }
            PartnerRemovalRule::OffsetPartner {
                is_foot,
                partner_offset,
                partner_matches,
            } => {
                if input.prevents_block_drops && is_foot && partner_matches {
                    let partner_pos = [
                        input.pos[0] + partner_offset[0],
                        input.pos[1] + partner_offset[1],
                        input.pos[2] + partner_offset[2],
                    ];
                    effects.push(DestroySideEffect::SetBlock(partner_pos, RemovedReplacement::Air, 35));
                    effects.push(DestroySideEffect::LevelEvent2001(partner_pos));
                }
            }
            PartnerRemovalRule::DestroyPartnerNoDrops {
                partner_offset,
                partner_matches,
            } => {
                if input.prevents_block_drops && partner_matches {
                    let partner_pos = [
                        input.pos[0] + partner_offset[0],
                        input.pos[1] + partner_offset[1],
                        input.pos[2] + partner_offset[2],
                    ];
                    effects.push(DestroySideEffect::DestroyBlockNoDrops(partner_pos));
                }
            }
        }
    }

    // super.playerWillDestroy: particles + gameEvent
    effects.push(DestroySideEffect::LevelEvent2001(input.pos));
    effects.push(DestroySideEffect::GameEventBlockDestroy(input.pos));

    // Level.removeBlock(pos, false)
    let (repl, flags) = remove_block_write(input.has_water_fluid, false);
    effects.push(DestroySideEffect::SetBlock(input.pos, repl, flags));

    if input.prevents_block_drops {
        return DestroyBlockResult { succeeded: true, exhaustion_added: 0.0, effects };
    }

    let dur_cost = item_mine_block_durability_cost(
        input.has_tool_component,
        input.tool_damage_per_block,
        input.block_destroy_speed,
        input.is_client_side,
        input.is_server_level,
    );
    if dur_cost > 0 {
        effects.push(DestroySideEffect::DamageTool(dur_cost));
    }

    let mut exhaustion_added = 0.0f32;
    if can_harvest {
        if !input.invulnerable && !input.is_client_side {
            exhaustion_added = BLOCK_DESTROY_EXHAUSTION;
        }
        if input.is_server_level {
            effects.push(DestroySideEffect::DropResources(input.pos));
        }
    }

    DestroyBlockResult {
        succeeded: true,
        exhaustion_added,
        effects,
    }
}

#[cfg(test)]
mod part4_and_part5_tests {
    use super::*;

    #[test]
    fn break_packet_gates_creative_air_and_stop_witnesses() {
        // probe-output.txt:16-105 & check-probe-output.txt:5-50
        let pos = [10, 0, 0];
        let in_range = |[x, y, z]: [f64; 3], reach| crate::interact::is_within_block_interaction_range(
            crate::Vec3::new(x, y, z), reach, pos, crate::interact::SERVER_INTERACTION_RANGE_TOLERANCE,
        );
        assert!(!in_range([16.5, 0.5, 0.5], 4.5)); // probe-output.txt:94
        assert!(in_range([f64::from_bits(0x4030_7fff_ffff_ffff), 0.5, 0.5], 4.5)); // probe-output.txt:95
        assert!(!in_range([17.0, 0.5, 0.5], 5.0)); // probe-output.txt:98
        assert!(in_range([f64::from_bits(0x4030_ffff_ffff_ffff), 0.5, 0.5], 5.0)); // probe-output.txt:99
        // C3 Y axis (check-probe-output.txt:37-39)
        assert!(!in_range([10.5, 6.5, 0.5], 4.5));
        assert!(in_range([10.5, f64::from_bits(0x4019_ffff_ffff_ffff), 0.5], 4.5));

        // C1a/C1b/C1c maxY gate (check-probe-output.txt:5-28)
        let mut clock = BreakPacketClock {
            game_ticks: 100,
            is_destroying_block: true,
            destroy_pos: [40, 0, 0],
            destroy_progress_start: 90,
            ..BreakPacketClock::default()
        };
        let mk_input = |action: BreakPacketAction, p: [i32; 3], instabuild: bool, air: bool, prog: f32| BreakGateInput {
            action,
            pos: p,
            eye_pos: [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5],
            block_interaction_range: 4.5,
            max_y: 319,
            under_spawn_protection: false,
            may_interact: true,
            instabuild,
            action_restricted: false,
            target_is_air: air,
            destroy_progress: prog,
        };
        assert_eq!(
            handle_block_break_action(&mut clock, mk_input(BreakPacketAction::Stop, [10, 320, 0], false, true, 0.0)),
            BreakPacketOutcome::RefusedTooHighSendBlockUpdate([10, 320, 0])
        );
        assert_eq!(clock.destroy_pos, [40, 0, 0]);
        assert_eq!(
            handle_block_break_action(&mut clock, mk_input(BreakPacketAction::Start, [10, 319, 0], false, true, 0.0)),
            BreakPacketOutcome::SurvivalStartArmed { pos: [10, 319, 0], stage: 10, reset_old_pos: Some([40, 0, 0]) }
        );

        // S2 creative START on bedrock (probe-output.txt:30-40)
        let mut c2 = BreakPacketClock { game_ticks: 100, ..BreakPacketClock::default() };
        assert_eq!(
            handle_block_break_action(&mut c2, mk_input(BreakPacketAction::Start, [20, 0, 0], true, false, 0.0)),
            BreakPacketOutcome::CreativeDestroyAndAck([20, 0, 0])
        );
        assert!(!c2.is_destroying_block);

        // S3 survival START on bedrock -> stage 0 (probe-output.txt:42-50)
        assert_eq!(
            handle_block_break_action(&mut c2, mk_input(BreakPacketAction::Start, [20, 0, 0], false, false, 0.0)),
            BreakPacketOutcome::SurvivalStartArmed { pos: [20, 0, 0], stage: 0, reset_old_pos: None }
        );

        // S4b survival START on air while active (probe-output.txt:62-72)
        let mut c4 = BreakPacketClock {
            game_ticks: 100,
            is_destroying_block: true,
            destroy_pos: [40, 0, 0],
            destroy_progress_start: 90,
            last_sent_state: 3,
            ..BreakPacketClock::default()
        };
        assert_eq!(
            handle_block_break_action(&mut c4, mk_input(BreakPacketAction::Start, [30, 0, 0], false, true, 0.0)),
            BreakPacketOutcome::SurvivalStartArmed { pos: [30, 0, 0], stage: 10, reset_old_pos: Some([40, 0, 0]) }
        );
        assert_eq!(c4.last_sent_state, 10);

        // S5/S6/S7/C4 STOP matrix (probe-output.txt:74-91, check-probe-output.txt:45-50)
        let mut c5 = BreakPacketClock {
            game_ticks: 100,
            is_destroying_block: true,
            destroy_pos: [40, 0, 0],
            destroy_progress_start: 90,
            has_delayed_destroy: true,
            delayed_destroy_pos: [60, 0, 0],
            delayed_tick_start: 80,
            last_sent_state: -999,
        };
        let snap = c5;
        assert_eq!(
            handle_block_break_action(&mut c5, mk_input(BreakPacketAction::Stop, [50, 0, 0], false, false, 0.0)),
            BreakPacketOutcome::StopNoopOtherPos
        );
        assert_eq!(c5, snap);
        assert_eq!(
            handle_block_break_action(&mut c5, mk_input(BreakPacketAction::Stop, [40, 0, 0], false, false, 0.0067)),
            BreakPacketOutcome::StopNoopDelayedPending { world_read: true }
        );
        assert_eq!(c5, snap);
        c5.has_delayed_destroy = false;
        assert_eq!(
            handle_block_break_action(&mut c5, mk_input(BreakPacketAction::Stop, [40, 0, 0], false, false, 0.0067)),
            BreakPacketOutcome::StopHandedOffToDelayed([40, 0, 0])
        );
        assert!(!c5.is_destroying_block && c5.has_delayed_destroy && c5.delayed_destroy_pos == [40, 0, 0]);

        // block_action_restricted (dossier.md:78, player-full.txt:181-216)
        assert!(!block_action_restricted(GameModeKind::Survival, true, true, false));
        assert!(!block_action_restricted(GameModeKind::Creative, true, true, false));
        assert!(block_action_restricted(GameModeKind::Spectator, true, false, true));
        assert!(block_action_restricted(GameModeKind::Adventure, false, true, true));
        assert!(!block_action_restricted(GameModeKind::Adventure, false, false, true));

        // ABORT clears is_destroying_block and reports mismatched destroy_pos
        let mut c_abort = BreakPacketClock {
            game_ticks: 100,
            is_destroying_block: true,
            destroy_pos: [40, 0, 0],
            ..BreakPacketClock::default()
        };
        assert_eq!(
            handle_block_break_action(&mut c_abort, mk_input(BreakPacketAction::Abort, [41, 0, 0], false, false, 0.0)),
            BreakPacketOutcome::Aborted { pos: [41, 0, 0], mismatched_destroy_pos: Some([40, 0, 0]) }
        );
        assert!(!c_abort.is_destroying_block);
    }

    #[test]
    fn destroy_block_side_effects_witnesses() {
        // harness-stdout.txt:6-88 & adversary-stdout.txt:1-39
        assert_eq!(remove_block_write(true, false), (RemovedReplacement::WaterSource, 3)); // W1a/W1b
        assert_eq!(remove_block_write(false, false), (RemovedReplacement::Air, 3)); // W1c
        assert_eq!(remove_block_write(true, true), (RemovedReplacement::WaterSource, 67)); // A2a
        assert_eq!(remove_block_write(false, true), (RemovedReplacement::Air, 67)); // A2b

        // W3a/W3b/W3c exhaustion (harness-stdout.txt:58-67)
        let mut exh = 0.0f32;
        for _ in 0..3 {
            exh = cause_food_exhaustion(exh, BLOCK_DESTROY_EXHAUSTION, false, false);
        }
        assert_eq!(exh.to_bits(), 0x3c75_c28f); // 0.015
        assert_eq!(cause_food_exhaustion(exh, BLOCK_DESTROY_EXHAUSTION, true, false).to_bits(), 0x3c75_c28f);
        assert_eq!(add_exhaustion(0.0, 45.0), 40.0);
        assert_eq!(add_exhaustion(0.0, BLOCK_DESTROY_EXHAUSTION).to_bits(), 0x3ba3_d70a);

        // W6 can_destroy_block (harness-stdout.txt:83-88)
        assert!(!can_destroy_block(Some(false), true));
        assert!(can_destroy_block(Some(false), false));
        assert!(can_destroy_block(Some(true), true));
        assert!(can_destroy_block(None, true));

        // W2a/W2b/W2c/A1 door & seagrass partner removal (harness-stdout.txt:21-38, adversary-stdout.txt:1-12)
        let base_input = DestroyBlockInput {
            pos: [2, 65, 2],
            is_client_side: false,
            is_server_level: false,
            tool_can_destroy_in_creative: None,
            has_tool_component: false,
            tool_damage_per_block: 0,
            tool_correct_for_drops: true,
            block_requires_correct_tool: false,
            block_destroy_speed: 3.0,
            is_game_master_block: false,
            can_use_game_master_blocks: false,
            action_restricted: false,
            prevents_block_drops: true,
            invulnerable: false,
            has_water_fluid: false,
            partner_rule: PartnerRemovalRule::BelowPartner {
                is_upper: true,
                partner_matches: true,
                partner_has_water: false,
                check_incorrect_tool: true,
            },
        };
        let w2a = destroy_block(base_input);
        assert!(w2a.succeeded);
        assert_eq!(w2a.exhaustion_added, 0.0); // creative skips playerDestroy & exhaustion
        assert_eq!(
            w2a.effects,
            vec![
                DestroySideEffect::SetBlock([2, 64, 2], RemovedReplacement::Air, 35),
                DestroySideEffect::LevelEvent2001([2, 64, 2]),
                DestroySideEffect::LevelEvent2001([2, 65, 2]),
                DestroySideEffect::GameEventBlockDestroy([2, 65, 2]),
                DestroySideEffect::SetBlock([2, 65, 2], RemovedReplacement::Air, 3),
            ]
        );
        // W2b survival door (harvestable bare-handed): no partner setBlock, 0.005 exhaustion
        let w2b = destroy_block(DestroyBlockInput { prevents_block_drops: false, ..base_input });
        assert_eq!(w2b.exhaustion_added.to_bits(), 0x3ba3_d70a);
        assert!(!w2b.effects.iter().any(|e| matches!(e, DestroySideEffect::SetBlock([2, 64, 2], _, _))));

        // A1 tall seagrass UPPER creative -> water[level=0], flags=35 (adversary-stdout.txt:1-8)
        let a1 = destroy_block(DestroyBlockInput {
            partner_rule: PartnerRemovalRule::BelowPartner {
                is_upper: true,
                partner_matches: true,
                partner_has_water: true,
                check_incorrect_tool: false,
            },
            ..base_input
        });
        assert_eq!(a1.effects[0], DestroySideEffect::SetBlock([2, 64, 2], RemovedReplacement::WaterSource, 35));

        // W4 durability gate: non-ServerLevel stub gives 0; ServerLevel gives damagePerBlock (harness-stdout.txt:68-75)
        assert_eq!(item_mine_block_durability_cost(true, 1, 1.5, false, false), 0);
        assert_eq!(item_mine_block_durability_cost(true, 2, 1.5, false, true), 2);
        assert_eq!(item_mine_block_durability_cost(true, 2, 0.0, false, true), 0);

        // W2d/e/f BedBlock FOOT-only creative partner removal + PistonHead + None
        let bed_foot = destroy_block(DestroyBlockInput {
            pos: [5, 64, 5],
            partner_rule: PartnerRemovalRule::OffsetPartner {
                is_foot: true,
                partner_offset: [0, 0, -1],
                partner_matches: true,
            },
            ..base_input
        });
        assert_eq!(bed_foot.effects[0], DestroySideEffect::SetBlock([5, 64, 4], RemovedReplacement::Air, 35));
        let bed_head = destroy_block(DestroyBlockInput {
            pos: [5, 64, 4],
            partner_rule: PartnerRemovalRule::OffsetPartner {
                is_foot: false,
                partner_offset: [0, 0, 1],
                partner_matches: true,
            },
            ..base_input
        });
        assert!(!bed_head.effects.iter().any(|e| matches!(e, DestroySideEffect::SetBlock([5, 64, 5], _, _))));
        let piston_head = destroy_block(DestroyBlockInput {
            pos: [3, 64, 3],
            partner_rule: PartnerRemovalRule::DestroyPartnerNoDrops {
                partner_offset: [0, 0, -1],
                partner_matches: true,
            },
            ..base_input
        });
        assert_eq!(piston_head.effects[0], DestroySideEffect::DestroyBlockNoDrops([3, 64, 2]));
        let plain = destroy_block(DestroyBlockInput {
            partner_rule: PartnerRemovalRule::None,
            ..base_input
        });
        assert_eq!(plain.effects[2], DestroySideEffect::SetBlock([2, 65, 2], RemovedReplacement::Air, 3));
    }
}
