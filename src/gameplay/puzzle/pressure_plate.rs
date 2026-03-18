use bevy::prelude::*;

use crate::constants::UI_Z;
use crate::core::assets::GameAssets;
use crate::gameplay::map::room::{CurrentRoom, RoomId};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::player::components::Player;
use crate::gameplay::puzzle::{ActivePuzzle, PuzzleEntity, PuzzleKind};
use crate::states::RoomState;

#[derive(Component, Debug, Clone)]
pub struct PressurePlate {
    pub required_s: f32,
    pub progress_s: f32,
    pub radius: f32,
}

pub fn spawn_pressure_plate(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -80.0, UI_Z - 10.0)),
            sprite: Sprite {
                color: Color::srgb(0.18, 0.75, 0.85),
                custom_size: Some(Vec2::new(90.0, 22.0)),
                ..default()
            },
            ..default()
        },
        PressurePlate {
            required_s: 1.8,
            progress_s: 0.0,
            radius: 55.0,
        },
        PuzzleEntity,
        InGameEntity,
        Name::new("PressurePlate"),
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "站上压力板完成机关",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 22.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, -132.0, UI_Z - 9.0)),
            ..default()
        },
        PuzzleEntity,
        InGameEntity,
        Name::new("PressurePlateHint"),
    ));
}

pub fn pressure_plate_system(
    time: Res<Time>,
    mut room_state: ResMut<RoomState>,
    current_room: Option<Res<CurrentRoom>>,
    mut active: ResMut<ActivePuzzle>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut q: Query<(&GlobalTransform, &mut PressurePlate, &mut Sprite)>,
) {
    if !matches!(*room_state, RoomState::Locked) {
        return;
    }
    let Some(current_room) = current_room.as_deref() else { return };
    if active.room != Some(current_room.0) || active.kind != Some(PuzzleKind::PressurePlate) {
        return;
    }
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation().truncate();

    for (tf, mut plate, mut sprite) in &mut q {
        let plate_pos = tf.translation().truncate();
        let on_plate = plate_pos.distance(player_pos) <= plate.radius;
        if on_plate {
            plate.progress_s += time.delta_seconds();
        } else {
            plate.progress_s = (plate.progress_s - time.delta_seconds() * 0.8).max(0.0);
        }

        let ratio = (plate.progress_s / plate.required_s).clamp(0.0, 1.0);
        sprite.color = Color::srgb(0.18 + 0.6 * ratio, 0.75, 0.30 + 0.55 * ratio);

        if plate.progress_s >= plate.required_s {
            active.completed = true;
            break;
        }
    }
}

pub fn activate_pressure_plate_puzzle(active: &mut ActivePuzzle, room: RoomId) {
    active.room = Some(room);
    active.kind = Some(PuzzleKind::PressurePlate);
}
