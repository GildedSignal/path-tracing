//! `clock`: a counter stepped per tick or per action, a threshold that fires, a reset.
//! A clock is a row; its owner names what firing does.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock { pub step: i32, pub at: i32, pub reset: i32 }

impl Clock {
    /// One step of a cooldown: it steps only while above `at`, so it rests there and a
    /// value already past it (a negative grace) stays.
    #[inline]
    pub fn cool(self, n: &mut i32) { if *n > self.at { *n += self.step; } }
    /// One step of `n`: it fires, and resets, once it reaches `at` in the step's direction
    /// (a zero step only checks, as a counting-down clock).
    #[inline]
    pub fn tick(self, n: &mut i32) -> bool {
        *n += self.step;
        let fired = if self.step > 0 { *n >= self.at } else { *n <= self.at };
        if fired { *n = self.reset; }
        fired
    }
}

/// The cooldowns: `LivingEntity.noJumpDelay` and `invulnerableTime` (baseTick), `Player.jumpTriggerTime`
/// (aiStep @0-14), `LocalPlayer.sprintTriggerTime` (aiStep @0-14): -1 a tick while positive.
pub const COOLDOWN: Clock = Clock { step: -1, at: 0, reset: 0 };
/// `FoodData.tick`'s timer while saturated at full food: heal by the saturation every 10.
pub const FOOD_SATURATED: Clock = Clock { step: 1, at: 10, reset: 0 };
/// `FoodData.tick`'s timer at food >= 18 (heal 1) or at food 0 (starve 1): every 80.
pub const FOOD_FED: Clock = Clock { step: 1, at: 80, reset: 0 };
/// `LivingEntity.decreaseAirSupply` / `shouldTakeDrowningDamage`: -1 a tick, at -20 a drown
/// hit and the air back to 0 (`baseTick @306`).
pub const AIR: Clock = Clock { step: -1, at: -20, reset: 0 };
