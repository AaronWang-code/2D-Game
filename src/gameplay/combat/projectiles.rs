use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::assets::GameAssets;
use crate::gameplay::combat::components::{Hitbox, Lifetime, Projectile, Team};
use crate::gameplay::map::InGameEntity;
use crate::utils::entity::safe_despawn_recursive;

pub fn spawn_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    team: Team,
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
) -> Entity {
    spawn_projectile_with_hitbox(
        commands, assets, None, team, pos, velocity, damage, false, 0.0, 1.0,
    )
}

pub fn spawn_player_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    owner: Entity,
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
    crit_chance: f32,
) -> Entity {
    spawn_projectile_with_hitbox(
        commands,
        assets,
        Some(owner),
        Team::Player,
        pos,
        velocity,
        damage,
        true,
        crit_chance,
        1.75,
    )
}

fn spawn_projectile_with_hitbox(
    commands: &mut Commands,
    assets: &GameAssets,
    owner: Option<Entity>,
    team: Team,
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
    can_crit: bool,
    crit_chance: f32,
    crit_multiplier: f32,
) -> Entity {
    let is_player_ranged = owner.is_some() && team == Team::Player;
    let (color, size, hitbox_size, lifetime_s, rotation, name) = if is_player_ranged {
        (
            Color::srgb(0.18, 0.92, 1.0),
            Vec2::new(18.0, 8.0),
            Vec2::new(18.0, 10.0),
            1.6,
            Quat::from_rotation_z(velocity.y.atan2(velocity.x)),
            "PlayerProjectile",
        )
    } else {
        (
            match team {
                Team::Player => Color::srgb(0.2, 0.85, 1.0),
                Team::Enemy => Color::srgb(1.0, 0.35, 0.25),
                Team::Pvp1 => Color::srgb(0.25, 0.9, 0.35),
                Team::Pvp2 => Color::srgb(0.95, 0.85, 0.25),
            },
            Vec2::splat(12.0),
            Vec2::splat(14.0),
            2.0,
            Quat::IDENTITY,
            "Projectile",
        )
    };

    commands
        .spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform {
                    translation: pos.extend(20.0),
                    rotation,
                    ..default()
                },
                sprite: Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                ..default()
            },
            Projectile { team, velocity },
            Hitbox {
                owner,
                team,
                size: hitbox_size,
                damage,
                knockback: 240.0,
                can_crit,
                crit_chance,
                crit_multiplier,
            },
            Lifetime(Timer::from_seconds(lifetime_s, TimerMode::Once)),
            InGameEntity,
            Name::new(name),
        ))
        .id()
}

pub fn move_projectiles(time: Res<Time>, mut q: Query<(&Projectile, &mut Transform)>) {
    for (proj, mut tf) in &mut q {
        tf.translation += (proj.velocity * time.delta_seconds()).extend(0.0);
    }
}

pub fn despawn_expired_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Lifetime), With<Projectile>>,
) {
    for (e, mut lifetime) in &mut q {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            safe_despawn_recursive(&mut commands, e);
        }
    }
}

pub fn despawn_out_of_room_projectiles(
    mut commands: Commands,
    q: Query<(Entity, &Transform), With<Projectile>>,
) {
    let half = Vec2::new(ROOM_HALF_WIDTH + 160.0, ROOM_HALF_HEIGHT + 120.0);
    for (e, tf) in &q {
        let p = tf.translation.truncate();
        if p.x.abs() > half.x || p.y.abs() > half.y {
            safe_despawn_recursive(&mut commands, e);
        }
    }
}
