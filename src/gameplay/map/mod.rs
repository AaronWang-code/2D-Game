pub mod doors;
pub mod generator;
pub mod room;
pub mod tiles;
pub mod transitions;

use bevy::prelude::*;

use crate::gameplay::map::room::{CurrentRoom, FloorLayout};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::player::components::Gold;
use crate::gameplay::player::components::Player;
use crate::coop::components::CoopPlayer;
use crate::states::AppState;
use crate::states::RoomState;
use bevy::utils::HashSet;
use crate::gameplay::map::room::RoomId;
use crate::gameplay::map::room::RoomType;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisitedRooms>()
            .init_resource::<RewardRoomGoldBonusSeen>()
            .add_plugins((transitions::TransitionsPlugin, doors::DoorsPlugin, tiles::TilesPlugin))
            .add_systems(OnEnter(AppState::InGame), generator::generate_and_spawn_floor)
            .add_systems(
                Update,
                (track_visited_rooms, reward_room_gold_bonus_on_enter).run_if(in_state(AppState::InGame)),
            );
            // 清理逻辑由 GameplayPlugin 统一在真正离开一局时触发（MainMenu/GameOver/Victory）。
    }
}

#[derive(Component)]
pub struct InGameEntity;

#[derive(Resource, Debug, Default, Clone)]
pub struct VisitedRooms(pub HashSet<RoomId>);

#[derive(Resource, Debug, Default, Clone)]
pub struct RewardRoomGoldBonusSeen(pub HashSet<RoomId>);

pub fn track_visited_rooms(mut visited: ResMut<VisitedRooms>, current: Option<Res<CurrentRoom>>) {
    let Some(current) = current else { return };
    if current.is_changed() {
        visited.0.insert(current.0);
    }
}

pub fn reward_room_gold_bonus_on_enter(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    mut seen: ResMut<RewardRoomGoldBonusSeen>,
    mut gold_q: Query<&mut Gold, Or<(With<Player>, With<CoopPlayer>)>>,
) {
    let (Some(layout), Some(current)) = (layout, current) else { return };
    if layout.is_changed() {
        seen.0.clear();
    }
    if !current.is_changed() && !layout.is_changed() {
        return;
    }
    let Some(room) = layout.room(current.0) else { return };
    if room.room_type != RoomType::Reward {
        return;
    }
    if !seen.0.insert(current.0) {
        return;
    }
    for mut gold in &mut gold_q {
        gold.0 = gold.0.saturating_add(100);
    }
}

pub fn cleanup_ingame_world(mut commands: Commands, q: Query<Entity, With<InGameEntity>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<FloorLayout>();
    commands.remove_resource::<CurrentRoom>();
    commands.remove_resource::<RoomTransition>();
    commands.remove_resource::<RoomState>();
}
