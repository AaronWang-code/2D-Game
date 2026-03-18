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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangedVolleyPattern {
    Single,
    Double,
    Triple,
    Nova,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RewardModifiers {
    pub max_hp_add: f32,
    pub dash_cooldown_mult: f32,
    pub lifesteal_on_kill: f32,
    pub crit_add: f32,
    pub move_speed_mult: f32,
    pub attack_speed_add: f32,
    pub dash_damage_trail: bool,
    pub melee_mastery_stacks: u32,
    pub ranged_mastery_stacks: u32,
}

impl RewardModifiers {
    pub fn melee_projectile_reflect_unlocked(self) -> bool {
        self.melee_mastery_stacks >= 3
    }

    pub fn shared_attack_speed_bonus(self) -> f32 {
        self.attack_speed_add.clamp(0.0, 0.45)
    }

    pub fn melee_damage_mult(self) -> f32 {
        1.0 + self.melee_mastery_stacks as f32 * 0.12
    }

    pub fn melee_range_bonus(self) -> f32 {
        self.melee_mastery_stacks as f32 * 10.0
    }

    pub fn melee_arc_half_angle_rad(self) -> f32 {
        (0.48 + self.melee_mastery_stacks as f32 * 0.05).min(1.05)
    }

    pub fn melee_speed_bonus(self) -> f32 {
        (self.melee_mastery_stacks as f32 * 0.03).clamp(0.0, 0.24)
    }

    pub fn total_melee_speed_bonus(self) -> f32 {
        (self.shared_attack_speed_bonus() + self.melee_speed_bonus()).clamp(0.0, 0.60)
    }

    pub fn melee_slash_scale(self) -> f32 {
        1.0 + self.melee_mastery_stacks as f32 * 0.05
    }

    pub fn ranged_damage_mult(self) -> f32 {
        1.0 + self.ranged_mastery_stacks as f32 * 0.10
    }

    pub fn ranged_speed_bonus(self) -> f32 {
        (self.ranged_mastery_stacks as f32 * 0.05).clamp(0.0, 0.42)
    }

    pub fn total_ranged_speed_bonus(self) -> f32 {
        (self.shared_attack_speed_bonus() + self.ranged_speed_bonus()).clamp(0.0, 0.65)
    }

    pub fn ranged_projectile_speed_mult(self) -> f32 {
        1.0 + self.ranged_mastery_stacks as f32 * 0.06
    }

    pub fn ranged_volley_pattern(self) -> RangedVolleyPattern {
        match self.ranged_mastery_stacks {
            10.. => RangedVolleyPattern::Nova,
            6..=9 => RangedVolleyPattern::Triple,
            3..=5 => RangedVolleyPattern::Double,
            _ => RangedVolleyPattern::Single,
        }
    }
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
