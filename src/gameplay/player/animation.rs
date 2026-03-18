use bevy::prelude::*;

use crate::core::events::DamageEvent;
use crate::core::input::PlayerInputState;

use super::components::*;

#[derive(Component, Debug, Clone)]
pub struct PlayerAnim {
    pub state: AnimationState,
    pub timer: Timer,
}

pub fn update_player_animation_state(
    time: Res<Time>,
    input: Res<PlayerInputState>,
    mut damage_events: EventReader<DamageEvent>,
    mut q: Query<(Entity, &Velocity, &DashState, &Health, &mut PlayerAnim), With<Player>>,
) {
    let Ok((player_e, vel, dash, health, mut anim)) = q.get_single_mut() else {
        return;
    };
    anim.timer.tick(time.delta());

    if health.current <= 0.0 {
        anim.state = AnimationState::Dead;
        return;
    }
    if anim.state == AnimationState::Hurt && !anim.timer.finished() {
        return;
    }

    let took_damage = damage_events.read().any(|event| event.target == player_e);
    if took_damage {
        anim.state = AnimationState::Hurt;
        anim.timer = Timer::from_seconds(0.12, TimerMode::Once);
        anim.timer.reset();
        return;
    }
    if dash.active {
        anim.state = AnimationState::Dash;
        return;
    }
    if input.attack_held || input.ranged_held {
        anim.state = AnimationState::Attack;
        return;
    }
    if vel.0.length_squared() > 1.0 {
        anim.state = AnimationState::Move;
    } else {
        anim.state = AnimationState::Idle;
    }
}

pub fn animate_player_sprite(
    mut q: Query<(&PlayerAnim, &FacingDirection, &mut Sprite), With<Player>>,
) {
    let Ok((anim, facing, mut sprite)) = q.get_single_mut() else {
        return;
    };
    sprite.flip_x = facing.0.x < -0.15;
    sprite.color = match anim.state {
        AnimationState::Idle => Color::WHITE,
        AnimationState::Move => Color::srgb(0.98, 0.99, 1.0),
        AnimationState::Attack => Color::srgb(1.0, 0.95, 0.92),
        AnimationState::Dash => Color::srgb(0.84, 0.94, 1.0),
        AnimationState::Hurt => Color::srgb(1.0, 0.82, 0.82),
        AnimationState::Dead => Color::srgb(0.40, 0.40, 0.40),
    };
}
