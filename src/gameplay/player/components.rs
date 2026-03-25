use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::combat::components::Team;
use crate::gameplay::rewards::data::RewardType;

pub const ENERGY_SYSTEM_ENABLED: bool = false;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangedVolleyPattern {
    Single,
    Double,
    Triple,
    Nova,
}

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct RewardModifiers {
    // Legacy percentage fields kept for save compatibility.
    pub attack_speed_mult: f32,
    pub max_hp_add: f32,
    pub dash_cooldown_mult: f32,
    pub lifesteal_on_kill: f32,
    pub crit_add: f32,
    pub move_speed_mult: f32,
    pub attack_speed_add: f32,
    pub dash_damage_trail: bool,
    pub bonus_projectile: bool,
    pub melee_mastery_stacks: u32,
    pub ranged_mastery_stacks: u32,
    pub attack_speed_level: u8,
    pub attack_speed_reduction_s: f32,
    pub shop_attack_speed_reduction_s: f32,
    pub max_health_level: u8,
    pub dash_cooldown_level: u8,
    pub dash_cooldown_reduction_s: f32,
    pub shop_dash_cooldown_reduction_s: f32,
    pub lifesteal_level: u8,
    pub crit_level: u8,
    pub move_speed_level: u8,
    pub move_speed_add: f32,
    pub attack_power_level: u8,
    pub shop_max_health_purchases: u8,
    pub shop_attack_power_purchases: u8,
    pub shop_dash_purchases: u8,
    pub shop_move_speed_purchases: u8,
    pub shop_crit_purchases: u8,
    pub shop_attack_speed_purchases: u8,
}

impl RewardModifiers {
    pub const COMMON_REWARD_MAX_LEVEL: u8 = 3;
    pub const WEAPON_MASTERY_MAX_LEVEL: u8 = 6;
    pub const ATTACK_SPEED_CAP_S: f32 = 0.20;
    pub const DASH_COOLDOWN_CAP_S: f32 = 0.30;

    pub fn melee_projectile_reflect_unlocked(self) -> bool {
        self.melee_mastery_stacks >= 4
    }

    pub fn melee_lifesteal_unlocked(self) -> bool {
        self.melee_mastery_stacks >= 2
    }

    pub fn melee_rupture_unlocked(self) -> bool {
        self.melee_mastery_stacks >= 4
    }

    pub fn melee_sword_wave_unlocked(self) -> bool {
        self.melee_mastery_stacks >= 6
    }

    pub fn melee_on_hit_heal_fraction(self, target_is_boss: bool) -> f32 {
        if !self.melee_lifesteal_unlocked() {
            return 0.0;
        }
        if target_is_boss { 0.02 } else { 0.04 }
    }

    pub fn melee_rupture_total_fraction(self) -> f32 {
        if self.melee_rupture_unlocked() {
            0.30
        } else {
            0.0
        }
    }

    pub fn melee_sword_wave_damage_fraction(self) -> f32 {
        if self.melee_sword_wave_unlocked() {
            0.35
        } else {
            0.0
        }
    }

    pub fn shared_attack_speed_bonus(self) -> f32 {
        (self.attack_speed_reduction_s
            + self.shop_attack_speed_reduction_s
            + self.legacy_attack_speed_reduction_s())
        .clamp(0.0, Self::ATTACK_SPEED_CAP_S + 0.18)
    }

    pub fn total_dash_cooldown_reduction(self) -> f32 {
        (self.dash_cooldown_reduction_s
            + self.shop_dash_cooldown_reduction_s
            + self.legacy_dash_cooldown_reduction_s())
        .clamp(0.0, Self::DASH_COOLDOWN_CAP_S + 0.20)
    }

    pub fn melee_damage_mult(self) -> f32 {
        1.0 + self.melee_mastery_stacks as f32 * 0.10
    }

    pub fn melee_range_bonus(self) -> f32 {
        self.melee_mastery_stacks as f32 * 9.0
    }

    pub fn melee_arc_half_angle_rad(self) -> f32 {
        (0.52 + self.melee_mastery_stacks as f32 * 0.04).min(1.00)
    }

    pub fn melee_speed_bonus(self) -> f32 {
        (self.melee_mastery_stacks as f32 * 0.01).clamp(0.0, 0.06)
    }

    pub fn total_melee_speed_bonus(self) -> f32 {
        (self.shared_attack_speed_bonus() + self.melee_speed_bonus()).clamp(0.0, 0.32)
    }

    pub fn melee_slash_scale(self) -> f32 {
        match self.melee_mastery_stacks {
            0..=1 => 1.0,
            2..=3 => 1.08,
            4..=5 => 1.16,
            _ => 1.22,
        }
    }

    pub fn melee_feature_summary(self) -> &'static str {
        match self.melee_mastery_stacks {
            0..=1 => "未解锁",
            2..=3 => "吸血",
            4..=5 => "吸血、裂伤、弹反",
            _ => "吸血、裂伤、弹反、剑风",
        }
    }

    pub fn ranged_damage_mult(self) -> f32 {
        1.0 + self.ranged_mastery_stacks as f32 * 0.07
    }

    pub fn ranged_speed_bonus(self) -> f32 {
        (self.ranged_mastery_stacks as f32 * 0.02).clamp(0.0, 0.12)
    }

    pub fn total_ranged_speed_bonus(self) -> f32 {
        (self.shared_attack_speed_bonus() + self.ranged_speed_bonus()).clamp(0.0, 0.34)
    }

    pub fn ranged_projectile_speed_mult(self) -> f32 {
        1.0 + self.ranged_mastery_stacks as f32 * 0.05
    }

    pub fn ranged_volley_pattern(self) -> RangedVolleyPattern {
        match self.ranged_mastery_stacks {
            6.. => RangedVolleyPattern::Nova,
            4..=5 => RangedVolleyPattern::Triple,
            2..=3 => RangedVolleyPattern::Double,
            _ => RangedVolleyPattern::Single,
        }
    }

    pub fn reward_level(self, reward: RewardType) -> Option<(u8, u8)> {
        match reward {
            RewardType::RecoverHealth => None,
            RewardType::EnhanceMeleeWeapon => Some((
                self.melee_mastery_stacks
                    .min(Self::WEAPON_MASTERY_MAX_LEVEL as u32) as u8,
                Self::WEAPON_MASTERY_MAX_LEVEL,
            )),
            RewardType::EnhanceRangedWeapon => Some((
                self.ranged_mastery_stacks
                    .min(Self::WEAPON_MASTERY_MAX_LEVEL as u32) as u8,
                Self::WEAPON_MASTERY_MAX_LEVEL,
            )),
            RewardType::DashDamageTrail => Some((u8::from(self.dash_damage_trail), 1)),
            RewardType::IncreaseAttackSpeed => {
                Some((self.attack_speed_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
            RewardType::IncreaseAttackPower => {
                Some((self.attack_power_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
            RewardType::IncreaseMaxHealth => {
                Some((self.max_health_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
            RewardType::ReduceDashCooldown => {
                Some((self.dash_cooldown_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
            RewardType::LifeStealOnKill => {
                Some((self.lifesteal_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
            RewardType::IncreaseCritChance => {
                Some((self.crit_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
            RewardType::IncreaseMoveSpeed => {
                Some((self.move_speed_level, Self::COMMON_REWARD_MAX_LEVEL))
            }
        }
    }

    pub fn reward_at_max(self, reward: RewardType) -> bool {
        self.reward_level(reward)
            .map(|(current, max)| current >= max)
            .unwrap_or(false)
    }

    fn legacy_attack_speed_reduction_s(self) -> f32 {
        (self.attack_speed_add.max(self.attack_speed_mult) * 0.34).clamp(0.0, 0.10)
    }

    fn legacy_dash_cooldown_reduction_s(self) -> f32 {
        (self.dash_cooldown_mult * 1.20).clamp(0.0, 0.18)
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
        let duration_s =
            (self.base_duration_s - speed_bonus.clamp(0.0, self.base_duration_s)).max(0.08);
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
        let duration_s =
            (self.base_duration_s - reduction.clamp(0.0, self.base_duration_s)).max(0.25);
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
        let duration_s =
            (self.base_duration_s - speed_bonus.clamp(0.0, self.base_duration_s)).max(0.12);
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(duration_s));
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TeamMarker(pub Team);
