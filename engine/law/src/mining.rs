/// A block at a position. The adapter changes the identity on replacement,
/// including replacement by the same block state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiningTarget {
    pub position: [i32; 3],
    pub identity: u64,
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
    fn progress(self) -> f32 {
        if self.hardness == -1.0 {
            return 0.0;
        }
        let divisor = if self.correct_tool { 30.0 } else { 100.0 };
        self.destroy_speed / self.hardness / divisor
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
