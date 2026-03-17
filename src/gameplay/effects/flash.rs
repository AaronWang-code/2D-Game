use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Flash {
    timer: Timer,
    original: Option<Color>,
}

impl Flash {
    pub fn new(duration_s: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            original: None,
        }
    }

    pub fn trigger(&mut self, duration_s: f32) {
        self.timer = Timer::from_seconds(duration_s, TimerMode::Once);
        self.timer.reset();
    }
}

pub fn update_flash_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Flash, &mut Sprite)>,
) {
    for (e, mut flash, mut sprite) in &mut q {
        if flash.original.is_none() {
            flash.original = Some(sprite.color);
        }
        flash.timer.tick(time.delta());
        if !flash.timer.finished() {
            sprite.color = Color::WHITE;
        } else if let Some(original) = flash.original.take() {
            sprite.color = original;
            commands.entity(e).remove::<Flash>();
        }
    }
}

