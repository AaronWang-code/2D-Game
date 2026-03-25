use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::events::SpawnEnemyEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::map::room::{
    CurrentRoom, Direction, FloorLayout, RoomBounds, RoomConnections, RoomData, RoomId, RoomType,
};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::map::{InGameEntity, VisitedRooms};
use crate::gameplay::player::components::{DashState, Player, Velocity};
use crate::gameplay::progression::floor::FloorNumber;
use crate::states::RoomState;
use crate::utils::rng::GameRng;

#[derive(Debug, Clone, Copy)]
struct GeneratedRoom {
    room_type: RoomType,
    mystery: bool,
}

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
    mut visited: Option<ResMut<VisitedRooms>>,
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
    let rooms = build_rooms(data.as_deref(), floor_number, &mut rng);

    let layout = FloorLayout {
        rooms,
        current: RoomId(0),
    };
    commands.insert_resource(CurrentRoom(layout.current));
    commands.insert_resource(layout);
    if let Some(mut visited) = visited {
        visited.0.clear();
        visited.0.insert(RoomId(0));
    }
    reset_player_for_floor(&mut player_q);

    spawn_current_room(&mut commands, &spawn_ev);
}

pub fn spawn_current_room(commands: &mut Commands, _spawn_ev: &EventWriter<SpawnEnemyEvent>) {
    commands.spawn((InGameEntity, Name::new("RoomRoot")));
}

pub(crate) fn build_rooms(
    data: Option<&GameDataRegistry>,
    floor: u32,
    rng: &mut GameRng,
) -> Vec<RoomData> {
    if let Some(data) = data {
        if !data.rooms.room_sequence.is_empty() {
            return linear_rooms(
                data.rooms
                    .room_sequence
                    .iter()
                    .copied()
                    .map(|room_type| GeneratedRoom {
                        room_type,
                        mystery: false,
                    })
                    .collect(),
            );
        }
        return branching_rooms(data.balance.floor_rooms.max(4), floor, rng);
    }

    branching_rooms(7, floor, rng)
}

fn linear_rooms(sequence: Vec<GeneratedRoom>) -> Vec<RoomData> {
    let count = sequence.len();
    let mut rooms = Vec::with_capacity(count);
    for (i, room) in sequence.into_iter().enumerate() {
        let id = RoomId(i as u32);
        let mut exits = Vec::new();
        if i > 0 {
            exits.push((Direction::Left, RoomId((i as u32) - 1)));
        }
        if i + 1 < count {
            exits.push((Direction::Right, RoomId((i as u32) + 1)));
        }
        rooms.push(make_room(id, room.room_type, room.mystery, exits));
    }
    rooms
}

fn branching_rooms(total_rooms: u32, floor: u32, rng: &mut GameRng) -> Vec<RoomData> {
    let layer_count = total_rooms.max(7).saturating_sub(1) as usize;
    let mut layers = Vec::with_capacity(layer_count);
    layers.push(vec![GeneratedRoom {
        room_type: RoomType::Normal,
        mystery: false,
    }]);

    for layer_index in 1..layer_count {
        let width = branch_width(rng);
        let is_boss_layer = layer_index == layer_count - 1;
        if is_boss_layer {
            layers.push(
                (0..width)
                    .map(|_| GeneratedRoom {
                        room_type: RoomType::Boss,
                        mystery: false,
                    })
                    .collect(),
            );
        } else {
            layers.push(build_layer_rooms(layer_index, width, floor, rng));
        }
    }

    let mut next_id = 1u32;
    let mut layer_ids = Vec::with_capacity(layers.len());
    for layer in &layers {
        let ids = (0..layer.len())
            .map(|_| {
                let id = RoomId(next_id);
                next_id += 1;
                id
            })
            .collect::<Vec<_>>();
        layer_ids.push(ids);
    }

    let mut rooms = Vec::with_capacity(next_id as usize);
    rooms.push(make_room(
        RoomId(0),
        RoomType::Start,
        false,
        directions_for_width(layer_ids[0].len())
            .into_iter()
            .zip(layer_ids[0].iter().copied())
            .collect(),
    ));

    for (layer_index, layer) in layers.into_iter().enumerate() {
        let exits = layer_ids
            .get(layer_index + 1)
            .map(|next_ids| {
                directions_for_width(next_ids.len())
                    .into_iter()
                    .zip(next_ids.iter().copied())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for (room_index, generated) in layer.into_iter().enumerate() {
            rooms.push(make_room(
                layer_ids[layer_index][room_index],
                generated.room_type,
                generated.mystery,
                exits.clone(),
            ));
        }
    }

    rooms
}

fn branch_width(rng: &mut GameRng) -> usize {
    if rng.gen_range_f32(0.0, 1.0) < 0.58 {
        2
    } else {
        3
    }
}

fn build_layer_rooms(
    layer_index: usize,
    width: usize,
    floor: u32,
    rng: &mut GameRng,
) -> Vec<GeneratedRoom> {
    if layer_index == 1 {
        return (0..width)
            .map(|_| GeneratedRoom {
                room_type: RoomType::Normal,
                mystery: false,
            })
            .collect();
    }

    pick_weighted_unique_room_types(width, layer_index, floor, rng)
        .into_iter()
        .map(|room_type| GeneratedRoom {
            mystery: room_type == RoomType::Puzzle,
            room_type,
        })
        .collect()
}

fn pick_weighted_unique_room_types(
    width: usize,
    layer_index: usize,
    floor: u32,
    rng: &mut GameRng,
) -> Vec<RoomType> {
    let mut selected = vec![RoomType::Normal];
    while selected.len() < width {
        let total_weight = [
            RoomType::Normal,
            RoomType::Shop,
            RoomType::Puzzle,
            RoomType::Reward,
        ]
        .into_iter()
        .filter(|room_type| !selected.contains(room_type))
        .map(|room_type| room_weight(room_type, layer_index, floor))
        .sum::<u32>()
        .max(1);
        let mut pick = rng.gen_range_f32(0.0, total_weight as f32).floor() as u32;
        let mut chosen = RoomType::Normal;

        for room_type in [
            RoomType::Normal,
            RoomType::Shop,
            RoomType::Puzzle,
            RoomType::Reward,
        ] {
            if selected.contains(&room_type) {
                continue;
            }
            let weight = room_weight(room_type, layer_index, floor);
            if pick < weight {
                chosen = room_type;
                break;
            }
            pick = pick.saturating_sub(weight);
        }

        selected.push(chosen);
    }
    rng.shuffle(&mut selected);
    selected
}

fn room_weight(room_type: RoomType, layer_index: usize, floor: u32) -> u32 {
    match room_type {
        RoomType::Normal => {
            if layer_index <= 2 {
                6
            } else if floor >= 4 {
                4
            } else {
                5
            }
        }
        RoomType::Shop => {
            if layer_index == 2 {
                1
            } else {
                2
            }
        }
        RoomType::Puzzle => 2,
        RoomType::Reward => 1,
        RoomType::Start | RoomType::Boss => 0,
    }
}

fn directions_for_width(width: usize) -> Vec<Direction> {
    match width {
        0 => Vec::new(),
        1 => vec![Direction::Right],
        2 => vec![Direction::Left, Direction::Right],
        _ => vec![Direction::Left, Direction::Up, Direction::Right],
    }
}

fn opposite_direction(dir: Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

fn make_room(
    id: RoomId,
    room_type: RoomType,
    mystery: bool,
    exits: Vec<(Direction, RoomId)>,
) -> RoomData {
    RoomData {
        id,
        room_type,
        mystery,
        connections: RoomConnections { exits },
        bounds: RoomBounds {
            half_size: Vec2::new(ROOM_HALF_WIDTH, ROOM_HALF_HEIGHT),
        },
    }
}

pub(crate) fn reset_player_for_floor(
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
