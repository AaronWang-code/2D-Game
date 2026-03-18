use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::combat::components::Team;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Clone, Copy)]
pub struct MoveSpeed(pub f32);

#[derive(Component, Debug, Clone, Copy)]
pub struct AttackPower(pub f32);

#[derive(Component, Debug, Clone)]
pub struct AttackCooldown {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct DashCooldown {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct RangedCooldown {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct InvincibilityTimer {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct FacingDirection(pub Vec2);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Move,
    Attack,
    Dash,
    Hurt,
    Dead,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CritChance(pub f32);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Gold(pub u32);

#[derive(Component, Debug, Clone)]
pub struct Combo {
    pub count: u32,
    pub timer: Timer,
}

impl Combo {
    pub fn new(window_s: f32) -> Self {
        Self {
            count: 0,
            timer: Timer::from_seconds(window_s, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Skill1Cooldown {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone)]
pub struct RangedRapidFire {
    pub ramp: u32,
    pub decay: Timer,
}

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct RewardModifiers {
    pub attack_speed_mult: f32,
    pub max_hp_add: f32,
    pub dash_cooldown_mult: f32,
    pub lifesteal_on_kill: f32,
    pub crit_add: f32,
    pub move_speed_mult: f32,
    pub dash_damage_trail: bool,
    pub bonus_projectile: bool,
}

#[derive(Component, Debug, Clone)]
pub struct DashState {
    pub active: bool,
    pub dir: Vec2,
    pub timer: Timer,
    pub speed: f32,
}

impl DashState {
    pub fn inactive(speed: f32, duration_s: f32) -> Self {
        Self {
            active: false,
            dir: Vec2::X,
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            speed,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMarker(pub Team);
