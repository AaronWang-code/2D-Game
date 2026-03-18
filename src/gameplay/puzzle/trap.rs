use bevy::prelude::*;

use crate::constants::UI_Z;
use crate::core::assets::GameAssets;
use crate::gameplay::combat::components::{Hitbox, Lifetime, Team};
use crate::gameplay::map::room::{CurrentRoom, RoomId};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::player::components::Player;
use crate::gameplay::puzzle::{ActivePuzzle, PuzzleEntity, PuzzleKind};
use crate::states::RoomState;

#[derive(Component, Debug, Clone)]
pub struct Trap {
    pub period: Timer,
    pub active: Timer,
    pub size: Vec2,
    pub damage: f32,
    pub knockback: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct TrapSurvivalState {
    pub remaining_s: f32,
}

impl Default for TrapSurvivalState {
    fn default() -> Self {
        Self { remaining_s: 0.0 }
    }
}

pub fn spawn_traps(commands: &mut Commands, assets: &GameAssets) {
    for pos in [
        Vec2::new(-170.0, -40.0),
        Vec2::new(-40.0, 60.0),
        Vec2::new(90.0, -20.0),
        Vec2::new(200.0, 120.0),
    ] {
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos.extend(UI_Z - 10.0)),
                sprite: Sprite {
                    color: Color::srgb(0.55, 0.15, 0.15),
                    custom_size: Some(Vec2::new(46.0, 18.0)),
                    ..default()
                },
                ..default()
            },
            Trap {
                period: Timer::from_seconds(0.95, TimerMode::Repeating),
                active: Timer::from_seconds(0.18, TimerMode::Once),
                size: Vec2::new(46.0, 18.0),
                damage: 14.0,
                knockback: 380.0,
            },
            PuzzleEntity,
            InGameEntity,
            Name::new("Trap"),
        ));
    }
}

pub fn trap_system(
    mut commands: Commands,
    time: Res<Time>,
    mut room_state: ResMut<RoomState>,
    current_room: Option<Res<CurrentRoom>>,
    mut active: ResMut<ActivePuzzle>,
    mut survival: Local<TrapSurvivalState>,
    assets: Option<Res<GameAssets>>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut traps: Query<(&GlobalTransform, &mut Trap, &mut Sprite)>,
) {
    if !matches!(*room_state, RoomState::Locked) {
        return;
    }
    let Some(current_room) = current_room.as_deref() else { return };
    if active.room != Some(current_room.0) || active.kind != Some(PuzzleKind::TrapSurvival) {
        return;
    }
    let Some(assets) = assets else { return };
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation().truncate();

    if survival.remaining_s <= 0.0 {
        survival.remaining_s = 8.0;
    }

    survival.remaining_s = (survival.remaining_s - time.delta_seconds()).max(0.0);

    for (tf, mut trap, mut sprite) in &mut traps {
        trap.period.tick(time.delta());
        let just_triggered = trap.period.just_finished();
        if just_triggered {
            trap.active.reset();
        }

        trap.active.tick(time.delta());
        let is_active = !trap.active.finished();
        sprite.color = if is_active {
            Color::srgb(0.95, 0.30, 0.25)
        } else {
            Color::srgb(0.55, 0.15, 0.15)
        };

        if just_triggered {
            let trap_pos = tf.translation().truncate();
            if trap_pos.distance(player_pos) <= 60.0 {
                commands.spawn((
                    SpriteBundle {
                        texture: assets.textures.white.clone(),
                        transform: Transform::from_translation(trap_pos.extend(UI_Z - 8.0)),
                        sprite: Sprite {
                            color: Color::srgba(0.95, 0.25, 0.20, 0.25),
                            custom_size: Some(trap.size),
                            ..default()
                        },
                        ..default()
                    },
                    Hitbox {
                        owner: None,
                        team: Team::Enemy,
                        size: trap.size,
                        damage: trap.damage,
                        knockback: trap.knockback,
                        can_crit: false,
                    },
                    Lifetime(Timer::from_seconds(0.06, TimerMode::Once)),
                    InGameEntity,
                    Name::new("TrapHitbox"),
                ));
            }
        }
    }

    if survival.remaining_s <= 0.0 {
        active.completed = true;
        survival.remaining_s = 0.0;
    }
}

pub fn activate_trap_survival_puzzle(active: &mut ActivePuzzle, room: RoomId) {
    active.room = Some(room);
    active.kind = Some(PuzzleKind::TrapSurvival);
}
