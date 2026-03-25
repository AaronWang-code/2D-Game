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
    Flanker,
    Sniper,
    SupportCaster,
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
pub struct Elite;

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMarker(pub Team);

#[derive(Component, Debug, Clone, Copy)]
pub struct BossPhase(pub u8);

#[derive(Component, Debug, Clone)]
pub struct BossPatternTimer(pub Timer);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossArchetype {
    Floor1Guardian,
    MirrorWarden,
    TideHunter,
    CubeCore,
}

impl BossArchetype {
    pub fn from_floor(floor: u32) -> Self {
        match floor {
            0 | 1 => Self::Floor1Guardian,
            2 => Self::MirrorWarden,
            3 => Self::TideHunter,
            _ => Self::CubeCore,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct BossCycleState {
    pub step: u8,
    pub anchor_index: usize,
    pub rotation: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BossSummoned;

#[derive(Component, Debug, Clone)]
pub struct EnemyBuffState {
    pub speed_mult: f32,
    pub cooldown_mult: f32,
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct ChargerState {
    pub phase: ChargerPhase,
    pub timer: Timer,
    pub dir: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct FlankerState {
    pub phase: FlankerPhase,
    pub timer: Timer,
    pub dir: Vec2,
    pub strafe_sign: f32,
    pub repath_timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct SniperState {
    pub phase: SniperPhase,
    pub timer: Timer,
    pub aim_dir: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargerPhase {
    Idle,
    Windup,
    Charging,
    Stunned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlankerPhase {
    Stalk,
    Windup,
    Lunging,
    Recover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniperPhase {
    Idle,
    Aiming,
    Recover,
}
