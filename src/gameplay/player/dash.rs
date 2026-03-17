use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::gameplay::effects::{afterimage, particles};

use super::components::*;

pub fn player_dash_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<(&GlobalTransform, &mut DashCooldown, &mut DashState, &FacingDirection, &mut InvincibilityTimer, &Sprite), With<Player>>,
) {
    let Ok((tf, mut cd, mut dash, facing, mut inv, sprite)) = q.get_single_mut() else { return };
    cd.timer.tick(time.delta());
    if dash.active || !input.dash_pressed || !cd.timer.finished() {
        return;
    }

    cd.timer.reset();
    dash.active = true;
    dash.timer.reset();
    dash.dir = if input.move_axis.length_squared() > 0.0 {
        input.move_axis.normalize()
    } else {
        facing.0
    };

    inv.timer.reset();
    particles::spawn_dash_particles(&mut commands, &assets, tf.translation().truncate());

    afterimage::spawn_afterimage(
        &mut commands,
        &assets,
        tf.translation().truncate(),
        sprite.color.with_alpha(0.45),
        sprite.custom_size.unwrap_or(Vec2::splat(32.0)),
    );
}

pub fn update_dash_state(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<(&GlobalTransform, &mut DashState, &Sprite), With<Player>>,
) {
    let Ok((tf, mut dash, sprite)) = q.get_single_mut() else { return };
    if !dash.active {
        return;
    }

    dash.timer.tick(time.delta());
    if dash.timer.just_finished() {
        dash.active = false;
        return;
    }

    afterimage::spawn_afterimage(
        &mut commands,
        &assets,
        tf.translation().truncate(),
        sprite.color.with_alpha(0.25),
        sprite.custom_size.unwrap_or(Vec2::splat(32.0)),
    );
}
