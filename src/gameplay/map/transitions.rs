use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH, UI_Z};
use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::gameplay::map::doors::Door;
use crate::gameplay::map::room::{CurrentRoom, Direction, FloorLayout, RoomId};
use crate::gameplay::map::VisitedRooms;
use crate::gameplay::player::components::Player;
use crate::states::{AppState, RoomState};
use crate::utils::easing::ease_in_out;
use crate::utils::entity::safe_despawn_recursive;

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
    pub entry_from: Direction,
    pub phase: TransitionPhase,
    pub timer: Timer,
}

impl Default for RoomTransition {
    fn default() -> Self {
        Self {
            active: false,
            to: RoomId(0),
            entry_from: Direction::Left,
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
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();

    let room = layout.room(current_room.0).unwrap();
    for (dir, to) in &room.connections.exits {
        let door_pos = match dir {
            Direction::Right => Vec2::new(ROOM_HALF_WIDTH - 10.0, 0.0),
            Direction::Left => Vec2::new(-(ROOM_HALF_WIDTH - 10.0), 0.0),
            Direction::Up => Vec2::new(0.0, ROOM_HALF_HEIGHT - 10.0),
            Direction::Down => Vec2::new(0.0, -(ROOM_HALF_HEIGHT - 10.0)),
        };
        if player_pos.distance(door_pos) < 70.0 {
            transition.active = true;
            transition.to = *to;
            transition.entry_from = opposite_direction(*dir);
            transition.phase = TransitionPhase::FadeOut;
            transition.timer.reset();
            return;
        }
    }

    // Backward compatibility for legacy door entities without layout data.
    if layout.room(current_room.0).is_none() {
        for (door, tf) in &doors_q {
            if player_pos.distance(tf.translation().truncate()) < 70.0 {
                transition.active = true;
                transition.to = door.to;
                transition.entry_from = opposite_direction(door.dir);
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
    mut player_q: Query<&mut Transform, With<Player>>,
    visited: Option<ResMut<VisitedRooms>>,
) {
    if !transition.active {
        return;
    }

    let (mut overlay_sprite, overlay_entity) = match overlay_q.get_single_mut() {
        Ok(value) => value,
        Err(_) => {
            commands.spawn((
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
            ));
            return;
        }
    };

    transition.timer.tick(time.delta());
    let eased = ease_in_out(transition.timer.fraction());

    match transition.phase {
        TransitionPhase::FadeOut => {
            overlay_sprite.color.set_alpha(eased);
            if transition.timer.finished() {
                current_room.0 = transition.to;
                if let Some(mut visited) = visited {
                    visited.0.insert(transition.to);
                }
                if let Ok(mut player_tf) = player_q.get_single_mut() {
                    player_tf.translation =
                        player_spawn_position(transition.entry_from, player_tf.translation.z);
                }
                let room_type = layout.room(current_room.0).map(|room| room.room_type);
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
                safe_despawn_recursive(&mut commands, overlay_entity);
            }
        }
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

fn player_spawn_position(entry_from: Direction, z: f32) -> Vec3 {
    match entry_from {
        Direction::Left => Vec3::new(-ROOM_HALF_WIDTH * 0.6, 0.0, z),
        Direction::Right => Vec3::new(ROOM_HALF_WIDTH * 0.6, 0.0, z),
        Direction::Up => Vec3::new(0.0, ROOM_HALF_HEIGHT * 0.55, z),
        Direction::Down => Vec3::new(0.0, -ROOM_HALF_HEIGHT * 0.55, z),
    }
}
