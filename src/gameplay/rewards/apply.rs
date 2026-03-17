use crate::gameplay::player::components::{AttackCooldown, CritChance, DashCooldown, Health, MoveSpeed, RewardModifiers};
use crate::gameplay::rewards::data::RewardType;

pub fn apply_reward_to_player_components(
    reward: RewardType,
    mods: &mut RewardModifiers,
    health: &mut Health,
    move_speed: &mut MoveSpeed,
    dash_cd: &mut DashCooldown,
    crit: &mut CritChance,
    atk_cd: &mut AttackCooldown,
) {
    match reward {
        RewardType::IncreaseAttackSpeed => mods.attack_speed_mult += 0.15,
        RewardType::IncreaseMaxHealth => {
            mods.max_hp_add += 20.0;
            health.max += 20.0;
            health.current += 20.0;
        }
        RewardType::ReduceDashCooldown => mods.dash_cooldown_mult += 0.15,
        RewardType::LifeStealOnKill => mods.lifesteal_on_kill += 3.0,
        RewardType::IncreaseCritChance => {
            mods.crit_add += 0.05;
            crit.0 += 0.05;
        }
        RewardType::IncreaseMoveSpeed => {
            mods.move_speed_mult += 0.10;
            move_speed.0 *= 1.10;
        }
        RewardType::DashDamageTrail => mods.dash_damage_trail = true,
        RewardType::BonusProjectile => mods.bonus_projectile = true,
    }

    if mods.dash_cooldown_mult > 0.0 {
        let base = dash_cd.timer.duration().as_secs_f32();
        dash_cd.timer.set_duration(std::time::Duration::from_secs_f32(
            (base * (1.0 - mods.dash_cooldown_mult)).max(0.25),
        ));
    }

    if mods.attack_speed_mult > 0.0 {
        let base = atk_cd.timer.duration().as_secs_f32();
        atk_cd.timer.set_duration(std::time::Duration::from_secs_f32(
            (base * (1.0 - mods.attack_speed_mult)).max(0.08),
        ));
    }
}
