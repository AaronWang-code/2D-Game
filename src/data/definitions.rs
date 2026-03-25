use serde::{Deserialize, Serialize};

use crate::gameplay::enemy::components::EnemyType;
use crate::gameplay::map::room::RoomType;
use crate::gameplay::rewards::data::RewardType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub max_hp: f32,
    pub move_speed: f32,
    pub attack_power: f32,
    pub attack_cooldown_s: f32,
    pub ranged_cooldown_s: f32,
    pub dash_cooldown_s: f32,
    pub dash_speed: f32,
    pub dash_duration_s: f32,
    pub invincibility_s: f32,
    pub crit_chance: f32,
    pub energy_max: f32,
    pub energy_regen_per_s: f32,
    pub dash_energy_cost: f32,
    pub ranged_energy_cost: f32,
    pub skill1_energy_cost: f32,
    pub heal_energy_cost_per_s: f32,
    pub heal_hp_per_s: f32,
    pub skill1_cooldown_s: f32,
    pub ranged_base_cooldown_s: f32,
    pub ranged_min_cooldown_s: f32,
    pub ranged_ramp_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyStatsConfig {
    pub max_hp: f32,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown_s: f32,
    pub aggro_range: f32,
    pub attack_range: f32,
    pub projectile_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemiesConfig {
    pub melee_chaser: EnemyStatsConfig,
    pub ranged_shooter: EnemyStatsConfig,
    pub charger: EnemyStatsConfig,
    pub flanker: EnemyStatsConfig,
    pub sniper: EnemyStatsConfig,
    pub support_caster: EnemyStatsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossFloorConfig {
    pub max_hp: f32,
    pub move_speed: f32,
    pub contact_damage: f32,
    pub phase_thresholds: Vec<f32>,
    pub projectile_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossesConfig {
    pub floor_1: BossFloorConfig,
    pub floor_2: BossFloorConfig,
    pub floor_3: BossFloorConfig,
    pub floor_4: BossFloorConfig,
}

impl BossesConfig {
    pub fn for_floor(&self, floor: u32) -> &BossFloorConfig {
        match floor {
            0 | 1 => &self.floor_1,
            2 => &self.floor_2,
            3 => &self.floor_3,
            _ => &self.floor_4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardConfig {
    pub reward: RewardType,
    pub title: String,
    pub description: String,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsConfig {
    pub rewards: Vec<RewardConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomGenConfig {
    pub room_sequence: Vec<RoomType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBalanceConfig {
    pub difficulty_per_floor: f32,
    pub enemy_count_normal_room: u32,
    pub reward_rooms_give_choice: bool,
    pub boss_room_gives_victory: bool,
    pub total_floors: u32,
    pub floor_rooms: u32,
    pub enemy_types: Vec<EnemyType>,
    pub elite_chance: f32,
    pub elite_hp_mult: f32,
    pub elite_damage_mult: f32,
    pub elite_gold_bonus: u32,
}
