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

    pub fn break_ticks(self) -> Option<u64> {
        let p = self.progress();
        (p > 0.0).then(|| (1.0f64 / (p as f64)).ceil() as u64)
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MiningAction {
    /// Eligible non-air target, with inputs evaluated at the START hook
    /// boundary. The position must be distinct from delayed work.
    Start(MiningSample),
    /// Requires the current active position. None means the position is now air.
    Stop {
        position: [i32; 3],
        current: Option<MiningSample>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiningRefusal {
    RejectedInput,
    StartWhileActive,
    StartAtDelayedPosition,
    StopWithoutActive,
    StopAtDifferentPosition,
    /// A replacement invalidated the selected work. Its slot was cleared.
    TargetChanged(MiningTarget),
}

/// The adapter must revalidate the identity and apply the intent at the same
/// event. Only the adapter can report whether destruction succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DestroyIntent {
    pub target: MiningTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MiningStep {
    Idle,
    TargetGone(MiningTarget),
    Evaluated {
        target: MiningTarget,
        progress: f32,
        destroy: Option<DestroyIntent>,
        /// Reset the old position before publishing the new progress stage.
        reset: Option<MiningTarget>,
    },
}

impl MiningStep {
    fn evaluated(target: MiningTarget, progress: f32, destroy: bool) -> Self {
        Self::Evaluated {
            target,
            progress,
            destroy: destroy.then_some(DestroyIntent { target }),
            reset: None,
        }
    }
}

/// Survival mining clock for START, same-active-target STOP and normal server
/// ticks. The caller orders packet actions and ticks as on the server and
/// supplies acceptance after external packet and eligibility checks.
/// ABORT and STOP on retained inactive positions
/// require a separate law. The adapter sends the reset using the old position's
/// current world state before publishing the new stage. Creative and air START
/// paths require separate laws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MiningClock {
    game_ticks: i32,
    active: Option<MiningWork>,
    delayed: Option<MiningWork>,
}

impl MiningClock {
    pub fn new(game_ticks: i32) -> Self {
        Self {
            game_ticks,
            ..Self::default()
        }
    }

    pub fn game_ticks(&self) -> i32 {
        self.game_ticks
    }

    pub fn active(&self) -> Option<MiningWork> {
        self.active
    }

    pub fn delayed(&self) -> Option<MiningWork> {
        self.delayed
    }

    pub fn apply(
        &mut self,
        action: MiningAction,
        accepted: bool,
    ) -> Result<MiningStep, MiningRefusal> {
        if !accepted {
            return Err(MiningRefusal::RejectedInput);
        }
        match action {
            MiningAction::Start(sample) => {
                let target = sample.target;
                if self
                    .delayed
                    .is_some_and(|work| work.target.position == target.position)
                {
                    return Err(MiningRefusal::StartAtDelayedPosition);
                }
                let progress = sample.progress();
                let destroy = progress >= 1.0;
                if destroy {
                    return Ok(MiningStep::evaluated(target, progress, true));
                }
                // Vanilla writes the new start tick, sends the old-position update,
                // then installs the new position and broadcasts its progress.
                let new_work = MiningWork { target, start_tick: self.game_ticks };
                let reset = self.active.map(|work| work.target);
                self.active = Some(new_work);
                Ok(MiningStep::Evaluated { target, progress, destroy: None, reset })
            }
            MiningAction::Stop { position, current } => {
                let work = self.active.ok_or(MiningRefusal::StopWithoutActive)?;
                if work.target.position != position {
                    return Err(MiningRefusal::StopAtDifferentPosition);
                }
                let Some(sample) = current else {
                    return Ok(MiningStep::Idle);
                };
                if sample.target != work.target {
                    self.active = None;
                    return Err(MiningRefusal::TargetChanged(work.target));
                }
                let progress = work.progress(self.game_ticks, sample);
                let destroy = progress >= 0.7;
                if destroy {
                    self.active = None;
                } else if self.delayed.is_none() {
                    self.delayed = self.active.take();
                }
                Ok(MiningStep::evaluated(work.target, progress, destroy))
            }
        }
    }

    /// Advance once and sample only the selected target. The callback returns
    /// the current non-air sample or None for known air. Resolve missing world
    /// data before this step. A delayed slot present at entry consumes the tick
    /// even when cleared.
    pub fn tick(
        &mut self,
        current: impl FnOnce(MiningTarget) -> Option<MiningSample>,
    ) -> Result<MiningStep, MiningRefusal> {
        self.game_ticks = self.game_ticks.wrapping_add(1);
        let is_delayed = self.delayed.is_some();
        let slot = if is_delayed {
            &mut self.delayed
        } else {
            &mut self.active
        };
        let Some(work) = *slot else {
            return Ok(MiningStep::Idle);
        };
        let Some(sample) = current(work.target) else {
            *slot = None;
            return Ok(MiningStep::TargetGone(work.target));
        };
        if sample.target != work.target {
            *slot = None;
            return Err(MiningRefusal::TargetChanged(work.target));
        }

        let progress = work.progress(self.game_ticks, sample);
        let destroy = is_delayed && progress >= 1.0;
        if destroy {
            *slot = None;
        }
        Ok(MiningStep::evaluated(work.target, progress, destroy))
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
                "obsidian-hand-ground", // checker-output.txt:16
                MiningModifiers {
                    tool_base_speed: f32::from_bits(0x3f80_0000),
                    submerged_mining_speed: sub_default,
                    ..MiningModifiers::default()
                },
                obsidian_hard,
                false,
                0x3951_b717,
                Some(5001),
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
