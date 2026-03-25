pub mod combat;
pub mod effects;
pub mod enemy;
pub mod map;
pub mod player;
pub mod progression;
pub mod puzzle;
pub mod rewards;
pub mod shop;

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
            shop::ShopPlugin,
        ))
        .add_systems(OnEnter(AppState::MainMenu), map::cleanup_ingame_world)
        .add_systems(OnEnter(AppState::GameOver), map::cleanup_ingame_world)
        .add_systems(OnEnter(AppState::Victory), map::cleanup_ingame_world);
    }
}
