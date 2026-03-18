use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH, UI_Z};
use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::gameplay::map::doors::Door;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomId};
use crate::gameplay::map::VisitedRooms;
use crate::gameplay::player::components::Player;
use crate::coop::components::CoopPlayer;
use crate::states::{AppState, RoomState};
use crate::utils::easing::ease_in_out;

pub struct TransitionsPlugin;

impl Plugin for TransitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (detect_room_exit, fade_transition_system).run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Resource, Debug, Clone)]
pub struct RoomTransition {
    pub active: bool,
    pub to: RoomId,
    pub phase: TransitionPhase,
    pub timer: Timer,
}

impl Default for RoomTransition {
    fn default() -> Self {
        Self {
            active: false,
            to: RoomId(0),
            phase: TransitionPhase::FadeOut,
            timer: Timer::from_seconds(0.25, TimerMode::Once),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    FadeOut,
    FadeIn,
}

#[derive(Component)]
pub struct TransitionOverlay;

pub fn detect_room_exit(
    input: Res<PlayerInputState>,
    player_q: Query<&GlobalTransform, With<Player>>,
    doors_q: Query<(&Door, &GlobalTransform)>,
    room_state: Res<RoomState>,
    layout: Res<FloorLayout>,
    current_room: Res<CurrentRoom>,
    mut transition: ResMut<RoomTransition>,
) {
    if transition.active || !input.interact_pressed {
        return;
    }
    if matches!(*room_state, RoomState::Locked | RoomState::BossFight) {
        return;
    }
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation().truncate();

    let room = layout.room(current_room.0).unwrap();
    for (dir, to) in &room.connections.exits {
        let door_pos = match dir {
            crate::gameplay::map::room::Direction::Right => Vec2::new(ROOM_HALF_WIDTH - 10.0, 0.0),
            crate::gameplay::map::room::Direction::Left => Vec2::new(-(ROOM_HALF_WIDTH - 10.0), 0.0),
            crate::gameplay::map::room::Direction::Up => Vec2::new(0.0, ROOM_HALF_HEIGHT - 10.0),
            crate::gameplay::map::room::Direction::Down => Vec2::new(0.0, -(ROOM_HALF_HEIGHT - 10.0)),
        };
        if player_pos.distance(door_pos) < 70.0 {
            transition.active = true;
            transition.to = *to;
            transition.phase = TransitionPhase::FadeOut;
            transition.timer.reset();
            return;
        }
    }

    // Backward compatibility: if doors were spawned without layout, try any Door entity.
    // 有 FloorLayout 时不使用 Door.to（Door 是占位/视觉用），避免错误跳转。
    if layout.room(current_room.0).is_none() {
        for (door, tf) in &doors_q {
            if player_pos.distance(tf.translation().truncate()) < 70.0 {
                transition.active = true;
                transition.to = door.to;
                transition.phase = TransitionPhase::FadeOut;
                transition.timer.reset();
                return;
            }
        }
    }
}

pub fn fade_transition_system(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut transition: ResMut<RoomTransition>,
    mut current_room: ResMut<CurrentRoom>,
    layout: Res<FloorLayout>,
    mut overlay_q: Query<(&mut Sprite, Entity), With<TransitionOverlay>>,
    mut room_state: ResMut<RoomState>,
    mut player_q: Query<&mut Transform, (With<Player>, Without<CoopPlayer>)>,
    mut coop_player_q: Query<&mut Transform, (With<CoopPlayer>, Without<Player>)>,
    mut visited: Option<ResMut<VisitedRooms>>,
) {
    if !transition.active {
        return;
    }

    let (mut overlay_sprite, overlay_entity) = match overlay_q.get_single_mut() {
        Ok(v) => v,
        Err(_) => {
            let e = commands
                .spawn((
                    SpriteBundle {
                        texture: assets.textures.white.clone(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, UI_Z - 1.0)),
                        sprite: Sprite {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                            custom_size: Some(Vec2::new(4000.0, 4000.0)),
                            ..default()
                        },
                        ..default()
                    },
                    TransitionOverlay,
                    Name::new("TransitionOverlay"),
                ))
                .id();
            return;
        }
    };

    transition.timer.tick(time.delta());
    let t = transition.timer.fraction();
    let eased = ease_in_out(t);

    match transition.phase {
        TransitionPhase::FadeOut => {
            overlay_sprite.color.set_alpha(eased);
            if transition.timer.finished() {
                current_room.0 = transition.to;
                if let Some(mut visited) = visited {
                    visited.0.insert(transition.to);
                }
                if let Ok(mut player_tf) = player_q.get_single_mut() {
                    player_tf.translation = Vec3::new(-ROOM_HALF_WIDTH * 0.6, 0.0, player_tf.translation.z);
                }
                for mut tf in &mut coop_player_q {
                    tf.translation = Vec3::new(-ROOM_HALF_WIDTH * 0.45, -40.0, tf.translation.z);
                }
                let room_type = layout.room(current_room.0).map(|r| r.room_type);
                *room_state = match room_type {
                    Some(crate::gameplay::map::room::RoomType::Boss) => RoomState::BossFight,
                    _ => RoomState::Idle,
                };

                transition.phase = TransitionPhase::FadeIn;
                transition.timer.reset();
            }
        }
        TransitionPhase::FadeIn => {
            overlay_sprite.color.set_alpha(1.0 - eased);
            if transition.timer.finished() {
                transition.active = false;
                commands.entity(overlay_entity).despawn_recursive();
            }
        }
    }
}
