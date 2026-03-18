use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::combat::components::Team;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PvpPlayerId(pub u8);

#[derive(Component)]
pub struct PvpLocalPlayer;

#[derive(Component)]
pub struct PvpRemotePlayer;

#[derive(Component)]
pub struct PvpEntity;

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PvpLives {
    pub lives: u8,
}

impl Default for PvpLives {
    fn default() -> Self {
        Self { lives: 3 }
    }
}

#[derive(Component, Debug, Clone)]
pub struct PvpCooldowns {
    pub melee: Timer,
    pub ranged: Timer,
    pub respawn: Timer,
}

impl PvpCooldowns {
    pub fn new() -> Self {
        Self {
            melee: Timer::from_seconds(0.35, TimerMode::Once),
            ranged: Timer::from_seconds(0.45, TimerMode::Once),
            respawn: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PvpTeam(pub Team);

#[derive(Component, Debug, Clone, Copy)]
pub struct PvpBullet {
    pub velocity: Vec2,
    pub remaining_distance: f32,
}
