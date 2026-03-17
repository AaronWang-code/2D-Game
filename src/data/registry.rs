use bevy::prelude::*;

use crate::data::definitions::*;

#[derive(Resource, Debug, Clone)]
pub struct GameDataRegistry {
    pub player: PlayerConfig,
    pub enemies: EnemiesConfig,
    pub boss: BossConfig,
    pub rewards: RewardsConfig,
    pub rooms: RoomGenConfig,
    pub balance: GameBalanceConfig,
}

