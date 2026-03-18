use bevy::prelude::*;

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
    pub base_duration_s: f32,
}

#[derive(Component, Debug, Clone)]
pub struct DashCooldown {
    pub timer: Timer,
    pub base_duration_s: f32,
}

#[derive(Component, Debug, Clone)]
pub struct RangedCooldown {
    pub timer: Timer,
    pub base_duration_s: f32,
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

#[derive(Component, Debug, Default, Clone, Copy)]
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
    pub trail_timer: Timer,
    pub speed: f32,
}

impl DashState {
    pub fn inactive(speed: f32, duration_s: f32) -> Self {
        Self {
            active: false,
            dir: Vec2::X,
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            trail_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            speed,
        }
    }
}

impl AttackCooldown {
    pub fn new(duration_s: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            base_duration_s: duration_s,
        }
    }

    pub fn apply_speed_bonus(&mut self, speed_bonus: f32) {
        let duration_s = (self.base_duration_s * (1.0 - speed_bonus.clamp(0.0, 0.8))).max(0.08);
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(duration_s));
    }
}

impl DashCooldown {
    pub fn new(duration_s: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            base_duration_s: duration_s,
        }
    }

    pub fn apply_reduction(&mut self, reduction: f32) {
        let duration_s = (self.base_duration_s * (1.0 - reduction.clamp(0.0, 0.8))).max(0.25);
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(duration_s));
    }
}

impl RangedCooldown {
    pub fn new(duration_s: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            base_duration_s: duration_s,
        }
    }

    pub fn apply_speed_bonus(&mut self, speed_bonus: f32) {
        let duration_s = (self.base_duration_s * (1.0 - speed_bonus.clamp(0.0, 0.8))).max(0.12);
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(duration_s));
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMarker(pub Team);
