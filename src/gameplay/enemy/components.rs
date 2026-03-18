use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::combat::components::Team;

#[derive(Component)]
pub struct Enemy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyType {
    MeleeChaser,
    RangedShooter,
    Charger,
    Boss,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyKind(pub EnemyType);

#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyStats {
    pub max_hp: f32,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown_s: f32,
    pub aggro_range: f32,
    pub attack_range: f32,
    pub projectile_speed: f32,
}

#[derive(Component, Debug, Clone)]
pub struct EnemyAttackCooldown {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMarker(pub Team);

#[derive(Component, Debug, Clone, Copy)]
pub struct BossPhase(pub u8);

#[derive(Component, Debug, Clone)]
pub struct BossPatternTimer(pub Timer);

#[derive(Component, Debug, Clone)]
pub struct ChargerState {
    pub phase: ChargerPhase,
    pub timer: Timer,
    pub dir: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargerPhase {
    Idle,
    Windup,
    Charging,
    Stunned,
}
