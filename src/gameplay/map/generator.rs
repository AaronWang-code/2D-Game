use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::events::SpawnEnemyEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{
    CurrentRoom, Direction, FloorLayout, RoomBounds, RoomConnections, RoomData, RoomId, RoomType,
};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::player::components::{DashState, Player, Velocity};
use crate::gameplay::progression::floor::FloorNumber;
use crate::states::RoomState;
use crate::utils::rng::GameRng;

pub fn generate_and_spawn_floor(
    mut commands: Commands,
    spawn_ev: EventWriter<SpawnEnemyEvent>,
    mut rng: ResMut<GameRng>,
    data: Option<Res<GameDataRegistry>>,
    floor: Option<Res<FloorNumber>>,
    existing_layout: Option<Res<FloorLayout>>,
    existing_current: Option<Res<CurrentRoom>>,
    existing_room_state: Option<Res<RoomState>>,
    existing_transition: Option<Res<RoomTransition>>,
    mut player_q: Query<(&mut Transform, &mut Velocity, &mut DashState), With<Player>>,
) {
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

    let floor_number = floor.as_deref().map(|floor| floor.0).unwrap_or(1);
    let sequence = build_room_sequence(data.as_deref(), floor_number, &mut rng);

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
    reset_player_for_floor(&mut player_q);

    spawn_current_room(&mut commands, &spawn_ev);
}

pub fn spawn_current_room(commands: &mut Commands, _spawn_ev: &EventWriter<SpawnEnemyEvent>) {
    commands.spawn((InGameEntity, Name::new("RoomRoot")));
}

fn build_room_sequence(
    data: Option<&GameDataRegistry>,
    floor: u32,
    rng: &mut GameRng,
) -> Vec<RoomType> {
    if let Some(data) = data {
        if !data.rooms.room_sequence.is_empty() {
            return data.rooms.room_sequence.clone();
        }
        return random_room_sequence(data.balance.floor_rooms.max(4), floor, rng);
    }

    random_room_sequence(5, floor, rng)
}

fn random_room_sequence(total_rooms: u32, floor: u32, rng: &mut GameRng) -> Vec<RoomType> {
    let middle_count = total_rooms.saturating_sub(2).max(2) as usize;
    let mut middle_rooms = vec![RoomType::Normal; middle_count];

    if middle_rooms.len() > 1 && (floor == 1 || rng.gen_bool(0.7)) {
        let mut reward_slots: Vec<usize> = (1..middle_rooms.len()).collect();
        rng.shuffle(&mut reward_slots);
        if let Some(slot) = reward_slots.first() {
            middle_rooms[*slot] = RoomType::Reward;
        }
    }

    let mut sequence = Vec::with_capacity(middle_rooms.len() + 2);
    sequence.push(RoomType::Start);
    sequence.extend(middle_rooms);
    sequence.push(RoomType::Boss);
    sequence
}

fn reset_player_for_floor(
    player_q: &mut Query<(&mut Transform, &mut Velocity, &mut DashState), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut dash)) = player_q.get_single_mut() else {
        return;
    };

    transform.translation = Vec3::new(-220.0, 0.0, 50.0);
    velocity.0 = Vec2::ZERO;
    dash.active = false;
    dash.dir = Vec2::X;
    dash.timer.reset();
    dash.trail_timer.reset();
}
