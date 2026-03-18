pub mod doors;
pub mod generator;
pub mod room;
pub mod tiles;
pub mod transitions;

use bevy::prelude::*;

use crate::gameplay::map::room::{CurrentRoom, FloorLayout};
use crate::gameplay::map::transitions::RoomTransition;
use crate::states::AppState;
use crate::states::RoomState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            transitions::TransitionsPlugin,
            doors::DoorsPlugin,
            tiles::TilesPlugin,
        ))
        .add_systems(
            OnEnter(AppState::InGame),
            generator::generate_and_spawn_floor,
        );
        // 清理逻辑由 GameplayPlugin 统一在真正离开一局时触发（MainMenu/GameOver/Victory）。
    }
}

#[derive(Component)]
pub struct InGameEntity;

pub fn cleanup_ingame_world(mut commands: Commands, q: Query<Entity, With<InGameEntity>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<FloorLayout>();
    commands.remove_resource::<CurrentRoom>();
    commands.remove_resource::<RoomTransition>();
    commands.remove_resource::<RoomState>();
}
