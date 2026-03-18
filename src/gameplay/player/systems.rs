use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::assets::GameAssets;
use crate::core::events::DeathEvent;
use crate::core::input::PlayerInputState;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::{Hurtbox, Knockback, Team};
use crate::gameplay::effects::flash::Flash;
use crate::gameplay::map::InGameEntity;
use crate::states::{AppState, RoomState};
use crate::utils::math::{clamp_in_room, clamp_length};

use super::animation::PlayerAnim;
use super::components::*;

pub fn spawn_player(
    mut commands: Commands,
    assets: Res<GameAssets>,
    data: Option<Res<GameDataRegistry>>,
    existing_player_q: Query<(), With<Player>>,
) {
    // 从 RewardSelect/Paused 返回 InGame 时也会触发 OnEnter(InGame)。
    // 如果这里重复生成 Player，会导致大量系统的 `get_single()` 失败，从而出现“回到游戏后无法移动/无响应”。
    if existing_player_q.iter().next().is_some() {
        return;
    }

    let cfg = data.as_deref().map(|d| &d.player);
    let max_hp = cfg.map(|c| c.max_hp).unwrap_or(100.0);
    let move_speed = cfg.map(|c| c.move_speed).unwrap_or(260.0);
    let attack_power = cfg.map(|c| c.attack_power).unwrap_or(18.0);
    let attack_cd = cfg.map(|c| c.attack_cooldown_s).unwrap_or(0.50);
    let ranged_cd = cfg.map(|c| c.ranged_cooldown_s).unwrap_or(0.70);
    let dash_cd = cfg.map(|c| c.dash_cooldown_s).unwrap_or(1.2);
    let dash_speed = cfg.map(|c| c.dash_speed).unwrap_or(680.0);
    let dash_duration = cfg.map(|c| c.dash_duration_s).unwrap_or(0.12);
    let inv_s = cfg.map(|c| c.invincibility_s).unwrap_or(0.35);
    let crit = cfg.map(|c| c.crit_chance).unwrap_or(0.05);

    let mut entity = commands.spawn((SpriteBundle {
        texture: assets.textures.player.clone(),
        transform: Transform::from_translation(Vec3::new(-220.0, 0.0, 50.0)),
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(74.0, 60.0)),
            ..default()
        },
        ..default()
    },));

    entity.insert((
        Player,
        TeamMarker(Team::Player),
        InGameEntity,
        Name::new("Player"),
    ));
    entity.insert((
        Health {
            current: max_hp,
            max: max_hp,
        },
        Energy {
            current: 100.0,
            max: 100.0,
        },
        Velocity::default(),
        MoveSpeed(move_speed),
        AttackPower(attack_power),
        FacingDirection(Vec2::X),
        CritChance(crit),
        RewardModifiers::default(),
        PlayerAnim {
            state: AnimationState::Idle,
            timer: Timer::from_seconds(0.12, TimerMode::Once),
        },
    ));
    entity.insert((
        AttackCooldown::new(attack_cd),
        RangedCooldown::new(ranged_cd),
        DashCooldown::new(dash_cd),
        InvincibilityTimer {
            timer: Timer::from_seconds(inv_s, TimerMode::Once),
        },
        DashState::inactive(dash_speed, dash_duration),
        Hurtbox {
            team: Team::Player,
            size: Vec2::splat(30.0),
        },
        Flash::new(0.0),
        Knockback(Vec2::ZERO),
    ));
}

pub fn player_move_system(
    time: Res<Time>,
    input: Res<PlayerInputState>,
    room_state: Res<RoomState>,
    mut q: Query<(&DashState, &MoveSpeed, &mut Velocity, &mut Transform), With<Player>>,
) {
    if matches!(*room_state, RoomState::BossFight) {
        // still movable
    }
    let Ok((dash, move_speed, mut vel, mut tf)) = q.get_single_mut() else {
        return;
    };
    if dash.active {
        vel.0 = dash.dir * dash.speed;
    } else {
        vel.0 = input.move_axis * move_speed.0.max(0.0);
    }
    vel.0 = clamp_length(vel.0, dash.speed.max(move_speed.0));
    tf.translation += (vel.0 * time.delta_seconds()).extend(0.0);

    let clamped = clamp_in_room(
        tf.translation.truncate(),
        Vec2::new(ROOM_HALF_WIDTH, ROOM_HALF_HEIGHT),
        28.0,
    );
    tf.translation.x = clamped.x;
    tf.translation.y = clamped.y;
}

pub fn player_facing_system(
    input: Res<PlayerInputState>,
    mut q: Query<(&GlobalTransform, &mut FacingDirection, &Velocity), With<Player>>,
) {
    let Ok((tf, mut facing, vel)) = q.get_single_mut() else {
        return;
    };
    if let Some(world) = input.aim_world {
        let dir = (world - tf.translation().truncate()).try_normalize();
        if let Some(dir) = dir {
            facing.0 = dir;
            return;
        }
    }
    if vel.0.length_squared() > 1.0 {
        if let Some(dir) = vel.0.try_normalize() {
            facing.0 = dir;
        }
    }
}

pub fn player_invincibility_system(
    time: Res<Time>,
    mut q: Query<&mut InvincibilityTimer, With<Player>>,
) {
    let Ok(mut inv) = q.get_single_mut() else {
        return;
    };
    inv.timer.tick(time.delta());
}

pub fn player_death_system(
    mut death_events: EventReader<DeathEvent>,
    player_q: Query<Entity, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(player_e) = player_q.get_single() else {
        return;
    };
    for ev in death_events.read() {
        if ev.entity == player_e {
            next_state.set(AppState::GameOver);
        }
    }
}
