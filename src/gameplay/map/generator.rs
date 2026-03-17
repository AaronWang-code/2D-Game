use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::events::SpawnEnemyEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::map::room::{
    CurrentRoom, Direction, FloorLayout, RoomBounds, RoomConnections, RoomData, RoomId, RoomType,
};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::map::InGameEntity;
use crate::states::RoomState;

pub fn generate_and_spawn_floor(
    mut commands: Commands,
    spawn_ev: EventWriter<SpawnEnemyEvent>,
    data: Option<Res<GameDataRegistry>>,
    existing_layout: Option<Res<FloorLayout>>,
    existing_current: Option<Res<CurrentRoom>>,
    existing_room_state: Option<Res<RoomState>>,
    existing_transition: Option<Res<RoomTransition>>,
) {
    // 从 RewardSelect/Paused 返回 InGame 时也会触发 OnEnter(InGame)。
    // 这时不应重置本层布局，否则会出现“回到起始房/状态被重置”等问题。
    if let Some(layout) = existing_layout.as_deref() {
        if existing_current.is_none() {
            commands.insert_resource(CurrentRoom(layout.current));
        }
        if existing_room_state.is_none() {
            commands.insert_resource(RoomState::Idle);
        }
        if existing_transition.is_none() {
            commands.insert_resource(RoomTransition::default());
        }
        return;
    }

    commands.insert_resource(RoomState::Idle);
    commands.insert_resource(RoomTransition::default());

    let sequence = data
        .as_deref()
        .and_then(|d| (!d.rooms.room_sequence.is_empty()).then_some(d.rooms.room_sequence.clone()))
        .unwrap_or_else(|| vec![RoomType::Start, RoomType::Normal, RoomType::Reward, RoomType::Boss]);

    let count = sequence.len();
    let mut rooms = Vec::with_capacity(count);
    for (i, room_type) in sequence.into_iter().enumerate() {
        let id = RoomId(i as u32);
        let mut exits = Vec::new();
        if i > 0 {
            exits.push((Direction::Left, RoomId((i as u32) - 1)));
        }
        if i + 1 < count {
            exits.push((Direction::Right, RoomId((i as u32) + 1)));
        }
        rooms.push(RoomData {
            id,
            room_type,
            connections: RoomConnections { exits },
            bounds: RoomBounds {
                half_size: Vec2::new(ROOM_HALF_WIDTH, ROOM_HALF_HEIGHT),
            },
        });
    }

    let layout = FloorLayout {
        rooms,
        current: RoomId(0),
    };
    commands.insert_resource(CurrentRoom(layout.current));
    commands.insert_resource(layout);

    spawn_current_room(&mut commands, &spawn_ev);
}

pub fn spawn_current_room(commands: &mut Commands, _spawn_ev: &EventWriter<SpawnEnemyEvent>) {
    commands.spawn((InGameEntity, Name::new("RoomRoot")));
    // Room content is spawned by other systems after resources exist; see tiles/doors plugins.
}

