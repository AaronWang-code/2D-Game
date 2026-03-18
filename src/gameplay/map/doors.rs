use bevy::prelude::*;

use crate::constants::ROOM_HALF_WIDTH;
use crate::core::assets::GameAssets;
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{Direction, RoomId};
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
                transform: Transform::from_translation(pos + Vec3::new(0.0, -74.0, 11.0)),
                ..default()
            },
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
    mut q: Query<&mut Sprite, With<DoorVisual>>,
) {
    let Some(room_state) = room_state else { return };
    for mut sprite in &mut q {
        sprite.color = match *room_state {
            RoomState::Locked | RoomState::BossFight => Color::srgb(0.65, 0.18, 0.12),
            RoomState::Cleared | RoomState::Idle => Color::srgb(0.65, 0.50, 0.20),
        };
    }
}
