use bevy::prelude::*;

use crate::constants::UI_Z;
use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::gameplay::map::room::{CurrentRoom, RoomId};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::player::components::Player;
use crate::gameplay::puzzle::{ActivePuzzle, PuzzleEntity, PuzzleKind};
use crate::states::RoomState;

#[derive(Component, Debug, Clone, Copy)]
pub struct Switch {
    pub index: u8,
}

#[derive(Resource, Debug, Clone)]
pub struct SwitchOrderState {
    pub expected_next: u8,
    pub reset_flash_s: f32,
}

impl Default for SwitchOrderState {
    fn default() -> Self {
        Self {
            expected_next: 1,
            reset_flash_s: 0.0,
        }
    }
}

pub fn spawn_switch_sequence(commands: &mut Commands, assets: &GameAssets) {
    let positions = [Vec2::new(-140.0, 40.0), Vec2::new(0.0, 130.0), Vec2::new(140.0, 40.0)];
    for (i, pos) in positions.into_iter().enumerate() {
        let idx = (i + 1) as u8;
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos.extend(UI_Z - 10.0)),
                sprite: Sprite {
                    color: Color::srgb(0.85, 0.85, 0.20),
                    custom_size: Some(Vec2::splat(34.0)),
                    ..default()
                },
                ..default()
            },
            Switch { index: idx },
            PuzzleEntity,
            InGameEntity,
            Name::new(format!("Switch{idx}")),
        ));

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("{idx}"),
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 22.0,
                        color: Color::BLACK,
                    },
                ),
                transform: Transform::from_translation((pos + Vec2::new(0.0, -2.0)).extend(UI_Z - 9.0)),
                ..default()
            },
            PuzzleEntity,
            InGameEntity,
            Name::new(format!("SwitchLabel{idx}")),
        ));
    }

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "按顺序触发 1-2-3（靠近按 E）",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 22.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, -150.0, UI_Z - 9.0)),
            ..default()
        },
        PuzzleEntity,
        InGameEntity,
        Name::new("SwitchOrderHint"),
    ));
}

pub fn switch_order_system(
    time: Res<Time>,
    input: Res<PlayerInputState>,
    mut room_state: ResMut<RoomState>,
    current_room: Option<Res<CurrentRoom>>,
    mut active: ResMut<ActivePuzzle>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut state: Local<SwitchOrderState>,
    mut q: Query<(&GlobalTransform, &Switch, &mut Sprite)>,
) {
    if !matches!(*room_state, RoomState::Locked) {
        return;
    }
    let Some(current_room) = current_room.as_deref() else { return };
    if active.room != Some(current_room.0) || active.kind != Some(PuzzleKind::SwitchOrder) {
        return;
    }
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation().truncate();

    state.reset_flash_s = (state.reset_flash_s - time.delta_seconds()).max(0.0);

    let mut interacted = None;
    if input.interact_pressed {
        for (tf, sw, _sprite) in &mut q {
            if tf.translation().truncate().distance(player_pos) <= 62.0 {
                interacted = Some(sw.index);
                break;
            }
        }
    }

    if let Some(idx) = interacted {
        if idx == state.expected_next {
            state.expected_next += 1;
        } else {
            state.expected_next = 1;
            state.reset_flash_s = 0.25;
        }
    }

    for (_tf, sw, mut sprite) in &mut q {
        let base = Color::srgb(0.85, 0.85, 0.20);
        let ok = sw.index < state.expected_next;
        let col = if ok {
            Color::srgb(0.25, 0.9, 0.35)
        } else if state.reset_flash_s > 0.0 {
            Color::srgb(0.9, 0.25, 0.25)
        } else {
            base
        };
        sprite.color = col;
    }

    if state.expected_next >= 4 {
        active.completed = true;
        state.expected_next = 1;
        state.reset_flash_s = 0.0;
    }
}

pub fn activate_switch_order_puzzle(active: &mut ActivePuzzle, room: RoomId) {
    active.room = Some(room);
    active.kind = Some(PuzzleKind::SwitchOrder);
}
