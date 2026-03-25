use bevy::prelude::*;

use crate::data::definitions::*;

#[derive(Resource, Debug, Clone)]
pub struct GameDataRegistry {
    pub player: PlayerConfig,
    pub enemies: EnemiesConfig,
    pub bosses: BossesConfig,
    pub rewards: RewardsConfig,
    pub rooms: RoomGenConfig,
    pub balance: GameBalanceConfig,
}
