use bevy::prelude::*;

use crate::gameplay::map::InGameEntity;

#[derive(Component, Debug, Clone)]
pub struct Afterimage {
    timer: Timer,
}

pub fn spawn_afterimage(
    commands: &mut Commands,
    texture: Handle<Image>,
    pos: Vec2,
    color: Color,
    size: Vec2,
    flip_x: bool,
) {
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_translation(pos.extend(5.0)),
            sprite: Sprite {
                color,
                custom_size: Some(size),
                flip_x,
                ..default()
            },
            ..default()
        },
        Afterimage {
            timer: Timer::from_seconds(0.18, TimerMode::Once),
        },
        InGameEntity,
        Name::new("Afterimage"),
    ));
}

pub fn update_afterimages(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Afterimage, &mut Sprite)>,
) {
    for (e, mut a, mut sprite) in &mut q {
        a.timer.tick(time.delta());
        sprite.color.set_alpha(0.45 * (1.0 - a.timer.fraction()));
        if a.timer.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}
