//! The player's cold rules: the fire clock's rows, farmland and the wither interval, game modes
//! and abilities, death and respawn. The tick itself is `core`.

/// `Entity.clearFire()` (`Entity.code.txt:1271-1279`): `min(0, current)`.
pub fn clear_fire_ticks(current: i32) -> i32 {
    current.min(0)
}

/// `BaseFireBlock.fireIgnite` (`@1-69`): skips `fireImmune`, charges `< 0` by `+1`,
/// increments `ServerPlayer` `>= 0` deterministically by `+1` (`nextInt(1, 3)` min),
/// and ignites for `8.0f * 20 = 160` ticks when `next >= 0`.
pub fn base_fire_ignite_ticks(current: i32, fire_immune: bool, is_server_player: bool) -> i32 {
    if fire_immune {
        return current;
    }
    let next = if current < 0 {
        current + 1
    } else if is_server_player {
        current + 1
    } else {
        current
    };
    if next >= 0 { next.max(160) } else { next }
}

/// `Entity.applyEffectsFromBlocks` fire grace tail (`@129-157`): server-only reset to
/// `-Player.getFireImmuneTicks() (-20)` for players (or `0` for non-player entities) when
/// `!isOnFire && !grew`.
pub fn apply_effects_fire_tail(
    after_ticks: i32,
    before_ticks: i32,
    is_server_level: bool,
    fire_immune: bool,
    is_player: bool,
) -> i32 {
    let is_on_fire = !fire_immune && after_ticks > 0;
    let grew = after_ticks > before_ticks;
    if is_server_level && !is_on_fire && !grew {
        if is_player { -20 } else { 0 }
    } else {
        after_ticks
    }
}

/// `FarmlandBlock.fallOn` trample condition (`@0-99`, `LivingEntity.getBbWidth()*width*height > 0.512f`,
/// `random.nextFloat() < fall_distance - 0.5`).
pub fn farmland_trample_threshold(width: f32, height: f32, fall_distance: f64, random_float: f32) -> bool {
    let bb_volume = width * width * height;
    bb_volume > 0.512f32 && (random_float as f64) < (fall_distance - 0.5)
}

/// `WitherMobEffect.DAMAGE_INTERVAL` (`40` ticks at amplifier 0) and `applyEffectTick` damage (`1.0f`).
pub const WITHER_DAMAGE_BASE_INTERVAL: i32 = 40;

/// `WitherMobEffect.shouldApplyEffectTickThisTick` (`@0-43`):
/// `interval = 40 >> amplifier`; true when `interval <= 0 || duration % interval == 0`.
#[inline]
pub const fn wither_should_apply_effect_tick(duration: i32, amplifier: i32) -> bool {
    let interval = WITHER_DAMAGE_BASE_INTERVAL.wrapping_shr(amplifier as u32);
    interval <= 0 || duration % interval == 0
}

// ============================================================================
// Part 9: The game-mode switch's abilities
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GameType {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl GameType {
    pub const ALL: [Self; 4] = [Self::Survival, Self::Creative, Self::Adventure, Self::Spectator];

    pub const fn by_id(id: i32) -> Self {
        match id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }

    pub const fn is_block_placing_restricted(self) -> bool {
        matches!(self, Self::Adventure | Self::Spectator)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Abilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub mayfly: bool,
    pub instabuild: bool,
    pub may_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            mayfly: false,
            instabuild: false,
            may_build: true,
            flying_speed: 0.05f32,
            walking_speed: 0.1f32,
        }
    }
}

/// `GameType.updatePlayerAbilities(Abilities)` (`GameType.javap:102-153`).
/// Never writes `flying_speed` or `walking_speed`.
pub fn update_player_abilities(mode: GameType, abilities: &mut Abilities) {
    match mode {
        GameType::Creative => {
            abilities.mayfly = true;
            abilities.instabuild = true;
            abilities.invulnerable = true;
        }
        GameType::Spectator => {
            abilities.mayfly = true;
            abilities.instabuild = false;
            abilities.invulnerable = true;
            abilities.flying = true;
        }
        GameType::Survival | GameType::Adventure => {
            abilities.mayfly = false;
            abilities.instabuild = false;
            abilities.invulnerable = false;
            abilities.flying = false;
        }
    }
    abilities.may_build = !mode.is_block_placing_restricted();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameModeSwitchPacket {
    PlayerAbilities,
    PlayerInfoUpdateGameMode(GameType),
    GameEventChangeGameMode(GameType),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerGameModeSwitchResult {
    pub changed: bool,
    pub previous_mode: GameType,
    pub current_mode: GameType,
    pub packets: Vec<GameModeSwitchPacket>,
    pub cleared_shoulder_entities: bool,
    pub stopped_riding: bool,
    pub stopped_using_item: bool,
    pub reset_impulse_context: bool,
}

/// `ServerPlayer.setGameMode(GameType)` + `ServerPlayerGameMode.changeGameModeForPlayer` (`@0-109`).
/// `is_in_range_of_ground` = `colliders.isEmpty() && available_space_below(1.0) < 1.0`.
pub fn server_set_game_mode(
    current_mode: &mut GameType,
    previous_mode: &mut GameType,
    abilities: &mut Abilities,
    new_mode: GameType,
    is_in_range_of_ground: bool,
) -> ServerGameModeSwitchResult {
    if *current_mode == new_mode {
        return ServerGameModeSwitchResult {
            changed: false,
            previous_mode: *previous_mode,
            current_mode: *current_mode,
            packets: Vec::new(),
            cleared_shoulder_entities: false,
            stopped_riding: false,
            stopped_using_item: false,
            reset_impulse_context: false,
        };
    }
    let was_spectator = *current_mode == GameType::Spectator;
    let old = *current_mode;
    *previous_mode = old;
    *current_mode = new_mode;
    update_player_abilities(new_mode, abilities);

    // ServerPlayerGameMode.changeGameModeForPlayer @27-50
    if abilities.flying && new_mode != GameType::Spectator && is_in_range_of_ground {
        abilities.flying = false;
    }

    let mut packets = Vec::with_capacity(4);
    packets.push(GameModeSwitchPacket::PlayerAbilities); // @53-57
    packets.push(GameModeSwitchPacket::PlayerInfoUpdateGameMode(new_mode)); // @60-84
    packets.push(GameModeSwitchPacket::GameEventChangeGameMode(new_mode)); // setGameMode @18-37
    let entering_spectator = new_mode == GameType::Spectator;
    let _exiting_spectator = !entering_spectator && was_spectator;
    packets.push(GameModeSwitchPacket::PlayerAbilities); // setGameMode @83-84

    ServerGameModeSwitchResult {
        changed: true,
        previous_mode: *previous_mode,
        current_mode: *current_mode,
        packets,
        cleared_shoulder_entities: entering_spectator,
        stopped_riding: entering_spectator,
        stopped_using_item: entering_spectator,
        reset_impulse_context: new_mode == GameType::Creative,
    }
}

#[cfg(test)]
mod game_mode_witnesses {
    use super::*;

    #[test]
    fn part9_gamemode_ability_switch_witnesses() {
        // out/driver-transitions.txt:4-35 (all 16 ordered transitions)
        let expected_post = |mode: GameType, pre_flying: bool| -> (bool, bool, bool, bool, bool) {
            match mode {
                GameType::Survival => (false, false, false, false, true),
                GameType::Creative => (true, pre_flying, true, true, true),
                GameType::Adventure => (false, false, false, false, false),
                GameType::Spectator => (true, true, true, false, false),
            }
        };
        for from in GameType::ALL {
            for to in GameType::ALL {
                let mut ab = Abilities {
                    flying_speed: 0.42,
                    walking_speed: 0.24,
                    ..Abilities::default()
                };
                update_player_abilities(from, &mut ab);
                if matches!(from, GameType::Creative | GameType::Spectator) {
                    ab.flying = true;
                }
                let pre_fly = ab.flying;
                update_player_abilities(to, &mut ab);
                let (inv, fly, may, insta, build) = expected_post(to, pre_fly);
                assert_eq!((ab.invulnerable, ab.flying, ab.mayfly, ab.instabuild, ab.may_build), (inv, fly, may, insta, build), "{from:?} -> {to:?}");
                assert_eq!((ab.flying_speed, ab.walking_speed), (0.42, 0.24));
            }
        }

        // ServerPlayer.setGameMode SPECTATOR->CREATIVE landing clear + double Abilities packet
        let mut cur = GameType::Spectator;
        let mut prev = GameType::Survival;
        let mut ab = Abilities::default();
        update_player_abilities(cur, &mut ab);
        let res = server_set_game_mode(&mut cur, &mut prev, &mut ab, GameType::Creative, true);
        assert!(res.changed);
        assert!(!ab.flying);
        assert_eq!(
            res.packets,
            vec![
                GameModeSwitchPacket::PlayerAbilities,
                GameModeSwitchPacket::PlayerInfoUpdateGameMode(GameType::Creative),
                GameModeSwitchPacket::GameEventChangeGameMode(GameType::Creative),
                GameModeSwitchPacket::PlayerAbilities,
            ]
        );
        assert!(!res.stopped_using_item && res.reset_impulse_context);
        // Same-mode refusal
        assert!(!server_set_game_mode(&mut cur, &mut prev, &mut ab, GameType::Creative, false).changed);
    }

}

/// Void damage and respawn reset (26.2 Entity.checkBelowWorld, ServerPlayer.restoreFrom).
pub mod death {
    //! The death side of the player law: the void's threshold and toll, and the reset
    //! a respawn performs. Anchors are the 26.2 server jar: `Entity.checkBelowWorld`,
    //! `LivingEntity.onBelowWorld`, `Player.hurtServer`, `PlayerList.respawn`,
    //! `ServerPlayer.restoreFrom`; the offsets sit in the run dossier.

    use crate::{PlayerState, Vec3, World};

    /// LivingEntity.onBelowWorld: hurt(damageSources().fellOutOfWorld(), 4.0f).
    /// out_of_world bypasses armor, resistance, invulnerability and knockback, and
    /// carries 0.0 exhaustion, so the raw hit is the debit for any player without
    /// protective enchantments (data/minecraft/damage_type/out_of_world.json and the
    /// tags bypasses_armor, bypasses_invulnerability, bypasses_resistance, no_knockback).
    pub const FELL_OUT_OF_WORLD: f32 = 4.0;

    /// Entity.checkBelowWorld: `getY() < (double)(level.getMinY() - 64)`, asked every
    /// Entity.baseTick. Strict: the overworld (minY -64) reads -128.0 and a player at
    /// exactly -128.0 is still in the world.
    pub fn below_world(y: f64, min_y: i32) -> bool {
        y < (min_y - 64) as f64
    }

    /// PlayerList.respawn into a fresh ServerPlayer with restoreFrom(keepEverything=false):
    /// health to the maximum (20.0 for the stock player), FoodData to its constructor
    /// (foodLevel 20, saturationLevel 5.0f, exhaustion 0.0f), and a new entity's zero for
    /// the hurt cooldown, fire, freezing, fall distance, velocity and effects. The
    /// position is the respawn teleport the caller performs: death is a teleport with a
    /// toll (S189), and this reset is the toll's ledger, not the teleport.
    pub fn respawn(state: &mut PlayerState, world: &impl World) {
        use crate::state_reads::{StateField as F, StateReads};
        world.state_footprint(StateReads(0), StateReads::of(&[F::health, F::food_level, F::saturation, F::exhaustion,
            F::food_tick_timer, F::velocity, F::fall_distance, F::remaining_fire_ticks, F::ticks_frozen, F::effects, F::effect_book, F::max_health, F::hurt_cooldown]));
        state.health = 20.0;
        state.food_level = 20;
        state.saturation = 5.0;
        state.exhaustion = 0.0;
        state.food_tick_timer = 0;
        state.velocity = Vec3::default();
        state.fall_distance = 0.0;
        state.remaining_fire_ticks = 0;
        state.ticks_frozen = 0;
        state.effects = false;
        state.effect_book.clear();
        state.max_health = 20.0;
        state.hurt_cooldown.remaining_ticks = 0;
        state.hurt_cooldown.last_raw_hit = 0.0;
    }

    /// `Blocks.lambda$static$39` (26.2): bed `bounceRestitution = 0.75f`.
    pub const BED_BOUNCE_RESTITUTION: f32 = 0.75;
    /// `BedBlock.fallOn`: `fallDistance * 0.5d`.
    pub const BED_FALL_DISTANCE_MULTIPLIER: f64 = 0.5;
    /// `BedBlock.useWithoutItem` / `RespawnAnchorBlock.explode`: explosion power `5.0f` with fire.
    pub const BAD_RESPAWN_EXPLOSION_RADIUS: f32 = 5.0;
    pub const RESPAWN_ANCHOR_MIN_CHARGES: u8 = 0;
    pub const RESPAWN_ANCHOR_MAX_CHARGES: u8 = 4;

    /// `BedRule.Rule` (`net.minecraft.world.attribute.BedRule`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BedRuleGate {
        Always,
        WhenDark,
        Never,
    }

    impl BedRuleGate {
        pub const fn test(self, is_dark_outside: bool) -> bool {
            match self {
                Self::Always => true,
                Self::WhenDark => is_dark_outside,
                Self::Never => false,
            }
        }
    }

    /// `BedRule` environment attribute (`CAN_SLEEP_WHEN_DARK` vs `EXPLODES`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BedRule {
        pub can_sleep: BedRuleGate,
        pub can_set_spawn: BedRuleGate,
        pub explodes: bool,
    }

    impl BedRule {
        pub const CAN_SLEEP_WHEN_DARK: Self = Self {
            can_sleep: BedRuleGate::WhenDark,
            can_set_spawn: BedRuleGate::Always,
            explodes: false,
        };
        pub const EXPLODES: Self = Self {
            can_sleep: BedRuleGate::Never,
            can_set_spawn: BedRuleGate::Never,
            explodes: true,
        };
    }

    /// `RespawnAnchorBlock.useWithoutItem` outcome.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum RespawnAnchorUse {
        Pass,
        SetSpawn,
        Explode { radius: f32, fire: bool },
    }

    /// `RespawnAnchorBlock.useItemOn` + `charge`: glowstone charges up to `MAX_CHARGES = 4`.
    pub const fn respawn_anchor_charge(charges: u8) -> Option<u8> {
        if charges < RESPAWN_ANCHOR_MAX_CHARGES {
            Some(charges + 1)
        } else {
            None
        }
    }

    /// `RespawnAnchorBlock.useWithoutItem`: `charge == 0` passes; else `works` sets spawn or explodes (`5.0f`, fire).
    pub const fn respawn_anchor_use(charges: u8, works: bool) -> RespawnAnchorUse {
        if charges == RESPAWN_ANCHOR_MIN_CHARGES {
            RespawnAnchorUse::Pass
        } else if works {
            RespawnAnchorUse::SetSpawn
        } else {
            RespawnAnchorUse::Explode { radius: BAD_RESPAWN_EXPLOSION_RADIUS, fire: true }
        }
    }

    /// `ServerPlayer.findRespawnAndUseSpawnBlock` for a respawn anchor:
    /// accepted when `(forced || charges > 0) && works`; consumes 1 charge when `!keep_everything && charges > 0`.
    pub const fn respawn_anchor_find_and_use(
        charges: u8,
        forced: bool,
        works: bool,
        keep_everything: bool,
    ) -> Option<u8> {
        if !(forced || charges > 0) || !works {
            return None;
        }
        if !keep_everything && charges > 0 {
            Some(charges - 1)
        } else {
            Some(charges)
        }
    }
}
