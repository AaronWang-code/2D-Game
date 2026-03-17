use bevy::prelude::*;

use crate::core::events::DamageEvent;

use super::components::*;

#[derive(Component, Debug, Clone)]
pub struct PlayerAnim {
    pub state: AnimationState,
    pub timer: Timer,
}

pub fn update_player_animation_state(
    time: Res<Time>,
    mut damage_events: EventReader<DamageEvent>,
    mut q: Query<(&Velocity, &DashState, &Health, &mut PlayerAnim), With<Player>>,
) {
    let Ok((vel, dash, health, mut anim)) = q.get_single_mut() else { return };
    anim.timer.tick(time.delta());

    if health.current <= 0.0 {
        anim.state = AnimationState::Dead;
        return;
    }
    if damage_events.read().next().is_some() {
        anim.state = AnimationState::Hurt;
        anim.timer = Timer::from_seconds(0.12, TimerMode::Once);
        anim.timer.reset();
        return;
    }
    if dash.active {
        anim.state = AnimationState::Dash;
        return;
    }
    if vel.0.length_squared() > 1.0 {
        anim.state = AnimationState::Move;
    } else {
        anim.state = AnimationState::Idle;
    }
}

pub fn animate_player_sprite(mut q: Query<(&PlayerAnim, &mut Sprite), With<Player>>) {
    let Ok((anim, mut sprite)) = q.get_single_mut() else { return };
    sprite.color = match anim.state {
        AnimationState::Idle => Color::srgb(0.35, 0.9, 0.45),
        AnimationState::Move => Color::srgb(0.28, 0.82, 0.4),
        AnimationState::Attack => Color::srgb(0.85, 0.92, 0.35),
        AnimationState::Dash => Color::srgb(0.55, 0.95, 0.95),
        AnimationState::Hurt => Color::srgb(1.0, 0.55, 0.55),
        AnimationState::Dead => Color::srgb(0.25, 0.25, 0.25),
    };
}
