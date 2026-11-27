use super::Command;
use law::{BlockPos, EffectKind, PlayerState, Refusal, Vec3, player::GameType};

/// Once execute has resolved the source and selected a player, these are the
/// command's writes to the player. The caller runs them in server order.
#[derive(Clone, Copy, Debug)]
pub enum PlayerEffect {
    Status { kind: EffectKind, duration: i32, amplifier: i32 },
    GameMode(GameType),
    Respawn(RespawnPoint),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RespawnPoint {
    pub cell: BlockPos,
    pub yaw: f32,
    pub pitch: f32,
}

/// Position and rotation of the already resolved command source, in the
/// snapshot's overworld. Both can differ from the selected player's.
#[derive(Clone, Copy, Debug)]
pub struct Source {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Command {
    /// Decodes a terminal player write. The selector and its execution
    /// conditions are the caller's responsibility.
    pub fn player_effect(&self, source: Source) -> Result<PlayerEffect, Refusal> {
        let action = self.action();
        let failed = || Refusal(format!("datapack line {}: unsupported {} player write", self.line,
            action.first().map_or("empty", String::as_str)));
        decode(action, source).ok_or_else(failed)
    }
}

impl PlayerEffect {
    /// Status effects enter the server's effect book and run at tickEffects.
    /// Respawn changes the mission's restart point.
    pub fn apply(self, player: &mut PlayerState, respawn: &mut Option<RespawnPoint>) -> Result<(), Refusal> {
        match self {
            Self::Status { kind, duration, amplifier } => player.add_effect(kind, duration, amplifier),
            Self::GameMode(mode) => {
                if player.profile.riding || player.profile.boat || mode == GameType::Spectator {
                    return Err(Refusal("datapack gamemode: server mode change with mount or spectator".into()));
                }
                if mode == GameType::Creative && player.flying {
                    return Err(Refusal("datapack gamemode: creative flight transition needs the ground query".into()));
                }
                if player.profile.game_mode != mode {
                    player.set_game_mode(mode);
                }
            }
            Self::Respawn(point) => *respawn = Some(point),
        }
        Ok(())
    }
}

fn decode(action: &[String], source: Source) -> Option<PlayerEffect> {
    let action: Vec<_> = action.iter().map(String::as_str).collect();
    match action.as_slice() {
        ["effect", "give", _, kind, rest @ ..] => status(kind, rest),
        ["gamemode", mode] | ["gamemode", mode, _] => Some(PlayerEffect::GameMode(match *mode {
            "adventure" => GameType::Adventure,
            "survival" => GameType::Survival,
            "creative" => GameType::Creative,
            "spectator" => GameType::Spectator,
            _ => return None,
        })),
        ["spawnpoint", _, x, y, z, rest @ ..] => {
            let cell = law::block_pos(Vec3::new(
                block_coordinate(x, source.position.x)?,
                block_coordinate(y, source.position.y)?,
                block_coordinate(z, source.position.z)?,
            ));
            let (yaw, pitch) = match *rest {
                [] => (0.0, 0.0),
                [yaw, pitch] if law::basis::cell::DATA_VERSION == 5023 =>
                    (rotation(yaw, source.yaw)?, rotation(pitch, source.pitch)?),
                _ => return None,
            };
            Some(PlayerEffect::Respawn(RespawnPoint {
                cell,
                yaw: law::math::wrap_degrees(yaw),
                pitch: pitch.clamp(-90.0, 90.0),
            }))
        }
        _ => None,
    }
}

fn status(name: &str, parameters: &[&str]) -> Option<PlayerEffect> {
    let (kind, instantaneous) = match name.strip_prefix("minecraft:").unwrap_or(name) {
        "saturation" => (EffectKind::Saturation, true),
        "instant_health" => (EffectKind::InstantHealth, true),
        "instant_damage" => (EffectKind::InstantDamage, true),
        "regeneration" => (EffectKind::Regeneration, false),
        "hunger" => (EffectKind::Hunger, false),
        "poison" => (EffectKind::Poison, false),
        "wither" => (EffectKind::Wither, false),
        _ => return None,
    };
    let duration = match parameters.first().copied() {
        None => if instantaneous { 1 } else { 600 },
        Some("infinite") => -1,
        Some(value) => {
            let value = value.parse::<i32>().ok().filter(|value| (1..=1_000_000).contains(value))?;
            if instantaneous { value } else { value * 20 }
        }
    };
    let amplifier = parameters.get(1).map_or(Some(0), |s| s.parse::<u8>().ok())? as i32;
    if parameters.len() > 3 || parameters.get(2).is_some_and(|&value| !matches!(value, "true" | "false")) {
        return None;
    }
    Some(PlayerEffect::Status { kind, duration, amplifier })
}

pub(super) fn block_coordinate(word: &str, source: f64) -> Option<f64> {
    if let Some(offset) = word.strip_prefix('~') {
        let offset = if offset.is_empty() { 0.0 } else { offset.parse::<f64>().ok()? };
        return (source + offset).is_finite().then_some(source + offset);
    }
    word.parse::<i32>().ok().map(f64::from)
}

fn rotation(word: &str, source: f32) -> Option<f32> {
    let angle = if let Some(offset) = word.strip_prefix('~') {
        source + if offset.is_empty() { 0.0 } else { offset.parse::<f32>().ok()? }
    } else {
        word.parse::<f32>().ok()?
    };
    angle.is_finite().then_some(angle)
}
