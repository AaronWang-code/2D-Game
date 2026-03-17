use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::gameplay::combat::components::{Hitbox, Lifetime, Projectile, Team};
use crate::gameplay::map::InGameEntity;

pub fn spawn_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    team: Team,
    pos: Vec2,
    velocity: Vec2,
    damage: f32,
) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos.extend(20.0)),
                sprite: Sprite {
                    color: match team {
                        Team::Player => Color::srgb(0.2, 0.85, 1.0),
                        Team::Enemy => Color::srgb(1.0, 0.35, 0.25),
                    },
                    custom_size: Some(Vec2::splat(12.0)),
                    ..default()
                },
                ..default()
            },
            Projectile { team, velocity },
            Hitbox {
                owner: None,
                team,
                size: Vec2::splat(14.0),
                damage,
                knockback: 240.0,
                can_crit: false,
            },
            Lifetime(Timer::from_seconds(2.0, TimerMode::Once)),
            InGameEntity,
            Name::new("Projectile"),
        ))
        .id()
}

pub fn move_projectiles(time: Res<Time>, mut q: Query<(&Projectile, &mut Transform)>) {
    for (proj, mut tf) in &mut q {
        tf.translation += (proj.velocity * time.delta_seconds()).extend(0.0);
    }
}

pub fn despawn_expired_projectiles(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Lifetime), With<Projectile>>) {
    for (e, mut lifetime) in &mut q {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

pub fn despawn_out_of_room_projectiles(mut commands: Commands, q: Query<(Entity, &Transform), With<Projectile>>) {
    let half = Vec2::new(ROOM_HALF_WIDTH + 160.0, ROOM_HALF_HEIGHT + 120.0);
    for (e, tf) in &q {
        let p = tf.translation.truncate();
        if p.x.abs() > half.x || p.y.abs() > half.y {
            commands.entity(e).despawn_recursive();
        }
    }
}
