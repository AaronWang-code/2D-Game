use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bevy::prelude::*;

use crate::data::definitions::*;
use crate::data::registry::GameDataRegistry;

pub fn load_all_configs(mut commands: Commands) {
    let registry = match try_load_all() {
        Ok(r) => r,
        Err(err) => {
            warn!("Failed to load configs from assets/configs/*.ron, using defaults: {err:?}");
            default_registry()
        }
    };
    commands.insert_resource(registry);
}

fn try_load_all() -> Result<GameDataRegistry> {
    Ok(GameDataRegistry {
        player: load_ron("assets/configs/player.ron")?,
        enemies: load_ron("assets/configs/enemies.ron")?,
        boss: load_ron("assets/configs/boss.ron")?,
        rewards: load_ron("assets/configs/rewards.ron")?,
        rooms: load_ron("assets/configs/rooms.ron")?,
        balance: load_ron("assets/configs/game_balance.ron")?,
    })
}

pub fn load_ron<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value =
        ron::from_str::<T>(&content).with_context(|| format!("parse ron {}", path.display()))?;
    Ok(value)
}

fn default_registry() -> GameDataRegistry {
    GameDataRegistry {
        player: PlayerConfig {
            max_hp: 100.0,
            move_speed: 260.0,
            attack_power: 18.0,
            attack_cooldown_s: 0.50,
            ranged_cooldown_s: 0.70,
            dash_cooldown_s: 1.2,
            dash_speed: 680.0,
            dash_duration_s: 0.12,
            invincibility_s: 0.35,
            crit_chance: 0.05,
        },
        enemies: EnemiesConfig {
            melee_chaser: EnemyStatsConfig {
                max_hp: 35.0,
                move_speed: 160.0,
                attack_damage: 12.0,
                attack_cooldown_s: 0.9,
                aggro_range: 420.0,
                attack_range: 38.0,
                projectile_speed: 0.0,
            },
            ranged_shooter: EnemyStatsConfig {
                max_hp: 28.0,
                move_speed: 120.0,
                attack_damage: 10.0,
                attack_cooldown_s: 1.2,
                aggro_range: 520.0,
                attack_range: 360.0,
                projectile_speed: 420.0,
            },
            charger: EnemyStatsConfig {
                max_hp: 45.0,
                move_speed: 110.0,
                attack_damage: 16.0,
                attack_cooldown_s: 1.6,
                aggro_range: 520.0,
                attack_range: 340.0,
                projectile_speed: 0.0,
            },
        },
        boss: BossConfig {
            max_hp: 245.0,
            move_speed: 115.0,
            contact_damage: 13.0,
            phase_thresholds: vec![0.60, 0.30],
            projectile_speed: 430.0,
        },
        rewards: RewardsConfig { rewards: vec![] },
        rooms: RoomGenConfig {
            room_sequence: vec![],
        },
        balance: GameBalanceConfig {
            difficulty_per_floor: 0.15,
            enemy_count_normal_room: 4,
            reward_rooms_give_choice: true,
            boss_room_gives_victory: false,
            total_floors: 4,
            floor_rooms: 5,
            enemy_types: vec![],
        },
    }
}
