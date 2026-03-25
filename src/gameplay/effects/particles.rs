use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::gameplay::map::InGameEntity;
use crate::utils::entity::safe_despawn_recursive;

#[derive(Component, Debug, Clone)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: Timer,
}

pub fn spawn_hit_particles(commands: &mut Commands, assets: &GameAssets, pos: Vec2, color: Color) {
    for i in 0..6 {
        let angle = i as f32 / 6.0 * std::f32::consts::TAU;
        let vel = Vec2::new(angle.cos(), angle.sin()) * 160.0;
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos.extend(50.0)),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                ..default()
            },
            Particle {
                velocity: vel,
                lifetime: Timer::from_seconds(0.25, TimerMode::Once),
            },
            InGameEntity,
            Name::new("HitParticle"),
        ));
    }
}

pub fn spawn_dash_particles(commands: &mut Commands, assets: &GameAssets, pos: Vec2) {
    spawn_hit_particles(commands, assets, pos, Color::srgba(0.8, 0.9, 1.0, 0.7));
}

pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    for (e, mut p, mut tf, mut sprite) in &mut q {
        p.lifetime.tick(time.delta());
        tf.translation += (p.velocity * time.delta_seconds()).extend(0.0);
        sprite.color.set_alpha(1.0 - p.lifetime.fraction());
        if p.lifetime.finished() {
            safe_despawn_recursive(&mut commands, e);
        }
    }
}
