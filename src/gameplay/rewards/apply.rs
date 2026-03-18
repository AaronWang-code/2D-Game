use crate::gameplay::player::components::{
    AttackCooldown, CritChance, DashCooldown, Health, MoveSpeed, RangedCooldown, RewardModifiers,
};
use crate::gameplay::rewards::data::RewardType;

pub fn apply_reward_to_player_components(
    reward: RewardType,
    value: f32,
    mods: &mut RewardModifiers,
    health: &mut Health,
    move_speed: &mut MoveSpeed,
    dash_cd: &mut DashCooldown,
    ranged_cd: &mut RangedCooldown,
    crit: &mut CritChance,
    atk_cd: &mut AttackCooldown,
) {
    match reward {
        RewardType::IncreaseAttackSpeed => mods.attack_speed_mult += value,
        RewardType::IncreaseMaxHealth => {
            mods.max_hp_add += value;
            health.max += value;
            health.current = (health.current + value).min(health.max);
        }
        RewardType::ReduceDashCooldown => mods.dash_cooldown_mult += value,
        RewardType::LifeStealOnKill => mods.lifesteal_on_kill += value,
        RewardType::IncreaseCritChance => {
            mods.crit_add += value;
            crit.0 = (crit.0 + value).clamp(0.0, 1.0);
        }
        RewardType::IncreaseMoveSpeed => {
            mods.move_speed_mult += value;
            move_speed.0 *= 1.0 + value;
        }
        RewardType::DashDamageTrail => mods.dash_damage_trail = true,
        RewardType::BonusProjectile => mods.bonus_projectile = true,
    }

    dash_cd.apply_reduction(mods.dash_cooldown_mult);
    atk_cd.apply_speed_bonus(mods.attack_speed_mult);
    ranged_cd.apply_speed_bonus(mods.attack_speed_mult);
}
