use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::gameplay::combat::components::{Hitbox, Lifetime, Team};
use crate::gameplay::effects::{afterimage, particles};
use crate::gameplay::map::InGameEntity;

use super::components::*;

pub fn player_dash_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<
        (
            &GlobalTransform,
            &mut DashCooldown,
            &mut DashState,
            &FacingDirection,
            &mut InvincibilityTimer,
            &Handle<Image>,
            &Sprite,
        ),
        With<Player>,
    >,
) {
    let Ok((tf, mut cd, mut dash, facing, mut inv, texture, sprite)) = q.get_single_mut() else {
        return;
    };
    cd.timer.tick(time.delta());
    if dash.active || !input.dash_pressed || !cd.timer.finished() {
        return;
    }

    cd.timer.reset();
    dash.active = true;
    dash.timer.reset();
    dash.trail_timer.reset();
    dash.dir = if input.move_axis.length_squared() > 0.0 {
        input.move_axis.normalize()
    } else {
        facing.0
    };

    inv.timer.reset();
    particles::spawn_dash_particles(&mut commands, &assets, tf.translation().truncate());

    afterimage::spawn_afterimage(
        &mut commands,
        texture.clone(),
        tf.translation().truncate(),
        sprite.color.with_alpha(0.45),
        sprite.custom_size.unwrap_or(Vec2::splat(32.0)),
        sprite.flip_x,
    );
}

pub fn update_dash_state(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<
        (
            Entity,
            &GlobalTransform,
            &mut DashState,
            &Handle<Image>,
            &Sprite,
            &AttackPower,
            &RewardModifiers,
        ),
        With<Player>,
    >,
) {
    let Ok((player_e, tf, mut dash, texture, sprite, attack_power, mods)) = q.get_single_mut()
    else {
        return;
    };
    if !dash.active {
        return;
    }

    dash.timer.tick(time.delta());
    if dash.timer.just_finished() {
        dash.active = false;
        return;
    }

    if mods.dash_damage_trail {
        dash.trail_timer.tick(time.delta());
        if dash.trail_timer.just_finished() {
            spawn_dash_trail_hitbox(
                &mut commands,
                &assets,
                player_e,
                tf.translation().truncate() - dash.dir * 10.0,
                attack_power.0 * 0.45,
            );
        }
    }

    afterimage::spawn_afterimage(
        &mut commands,
        texture.clone(),
        tf.translation().truncate(),
        sprite.color.with_alpha(0.25),
        sprite.custom_size.unwrap_or(Vec2::splat(32.0)),
        sprite.flip_x,
    );
}

fn spawn_dash_trail_hitbox(
    commands: &mut Commands,
    assets: &GameAssets,
    owner: Entity,
    pos: Vec2,
    damage: f32,
) {
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(pos.extend(54.0)),
            sprite: Sprite {
                color: Color::srgba(0.40, 0.95, 1.0, 0.25),
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            ..default()
        },
        Hitbox {
            owner: Some(owner),
            team: Team::Player,
            size: Vec2::splat(24.0),
            damage,
            knockback: 220.0,
            can_crit: false,
            crit_chance: 0.0,
            crit_multiplier: 1.0,
        },
        Lifetime(Timer::from_seconds(0.08, TimerMode::Once)),
        InGameEntity,
        Name::new("DashTrailHitbox"),
    ));
}
