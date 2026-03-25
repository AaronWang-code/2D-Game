use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::assets::GameAssets;
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{CurrentRoom, Direction, FloorLayout, RoomId, RoomType};
use crate::states::RoomState;

pub struct DoorsPlugin;

impl Plugin for DoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_room_doors_if_missing, update_door_visuals));
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Door {
    pub to: RoomId,
    pub dir: Direction,
}

#[derive(Component)]
pub struct DoorVisual;

#[derive(Component, Debug, Clone, Copy)]
pub struct DoorLabel(pub Direction);

pub fn spawn_room_doors_if_missing(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    existing: Query<(), With<Door>>,
) {
    if existing.iter().next().is_some() {
        return;
    }
    let Some(assets) = assets else { return };
    spawn_room_doors(&mut commands, &assets);
}

pub fn spawn_room_doors(commands: &mut Commands, assets: &GameAssets) {
    let door_size = Vec2::new(46.0, 96.0);
    for (dir, pos) in [
        (
            Direction::Right,
            Vec3::new(ROOM_HALF_WIDTH - 10.0, 0.0, 10.0),
        ),
        (
            Direction::Left,
            Vec3::new(-(ROOM_HALF_WIDTH - 10.0), 0.0, 10.0),
        ),
        (Direction::Up, Vec3::new(0.0, ROOM_HALF_HEIGHT - 10.0, 10.0)),
        (
            Direction::Down,
            Vec3::new(0.0, -(ROOM_HALF_HEIGHT - 10.0), 10.0),
        ),
    ] {
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos),
                sprite: Sprite {
                    color: Color::srgb(0.65, 0.50, 0.20),
                    custom_size: Some(door_size),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            Door { to: RoomId(0), dir },
            DoorVisual,
            InGameEntity,
            Name::new("Door"),
        ));

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "交互(E)",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 22.0,
                        color: Color::WHITE,
                    },
                ),
                transform: Transform::from_translation(pos + door_label_offset(dir)),
                visibility: Visibility::Hidden,
                ..default()
            },
            DoorLabel(dir),
            InGameEntity,
            Name::new("DoorLabel"),
        ));
    }
}

pub fn lock_room_doors(mut room_state: ResMut<RoomState>) {
    *room_state = RoomState::Locked;
}

pub fn unlock_room_doors(mut room_state: ResMut<RoomState>) {
    *room_state = RoomState::Cleared;
}

pub fn update_door_visuals(
    room_state: Option<Res<RoomState>>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    mut doors: Query<(&mut Door, &mut Sprite, &mut Visibility), With<DoorVisual>>,
    mut labels: Query<(&DoorLabel, &mut Text, &mut Visibility), Without<DoorVisual>>,
) {
    let Some(room_state) = room_state else { return };
    let room = layout
        .as_deref()
        .zip(current.as_deref())
        .and_then(|(layout, current)| layout.room(current.0));

    for (mut door, mut sprite, mut visibility) in &mut doors {
        let Some(room) = room else {
            *visibility = Visibility::Visible;
            continue;
        };
        if let Some((_, to)) = room
            .connections
            .exits
            .iter()
            .find(|(dir, _)| *dir == door.dir)
        {
            door.to = *to;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
            continue;
        }
        sprite.color = match *room_state {
            RoomState::Locked | RoomState::BossFight => Color::srgb(0.65, 0.18, 0.12),
            RoomState::Cleared | RoomState::Idle => Color::srgb(0.65, 0.50, 0.20),
        };
    }

    for (label, mut text, mut visibility) in &mut labels {
        if let Some((layout, room)) = layout.as_deref().zip(room) {
            if let Some((_, to)) = room
                .connections
                .exits
                .iter()
                .find(|(dir, _)| *dir == label.0)
            {
                let destination = layout.room(*to).map(|room| room.room_type);
                text.sections[0].value = format!(
                    "{} (E)",
                    room_type_label(destination.unwrap_or(RoomType::Normal))
                );
                *visibility = Visibility::Visible;
                continue;
            }
        }
        *visibility = Visibility::Hidden;
    }
}

fn door_label_offset(dir: Direction) -> Vec3 {
    match dir {
        Direction::Up => Vec3::new(0.0, -54.0, 11.0),
        Direction::Down => Vec3::new(0.0, 54.0, 11.0),
        Direction::Left | Direction::Right => Vec3::new(0.0, -74.0, 11.0),
    }
}

fn room_type_label(room_type: RoomType) -> &'static str {
    match room_type {
        RoomType::Start => "起点",
        RoomType::Normal => "战斗",
        RoomType::Shop => "商店",
        RoomType::Reward => "奖励",
        RoomType::Puzzle => "事件",
        RoomType::Boss => "首领",
    }
}
