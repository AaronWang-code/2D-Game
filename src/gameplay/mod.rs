pub mod combat;
pub mod effects;
pub mod enemy;
pub mod map;
pub mod player;
pub mod progression;
pub mod puzzle;
pub mod rewards;

use bevy::prelude::*;

use crate::states::AppState;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            map::MapPlugin,
            progression::ProgressionPlugin,
            combat::CombatPlugin,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            rewards::RewardsPlugin,
            effects::EffectsPlugin,
            puzzle::PuzzlePlugin,
        ))
        // 注意：RewardSelect / Paused 也是从 InGame 切换出去的状态，
        // 不能在 OnExit(InGame) 直接清理世界，否则奖励选择时玩家会被 despawn，导致“选奖励没反应”。
        .add_systems(OnEnter(AppState::MainMenu), map::cleanup_ingame_world)
        .add_systems(OnEnter(AppState::GameOver), map::cleanup_ingame_world)
        .add_systems(OnEnter(AppState::Victory), map::cleanup_ingame_world);
    }
}
