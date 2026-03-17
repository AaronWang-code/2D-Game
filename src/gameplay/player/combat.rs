use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::{Hitbox, Lifetime, Team};
use crate::gameplay::effects::particles;
use crate::gameplay::map::InGameEntity;

use super::components::*;

pub fn player_attack_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    data: Option<Res<GameDataRegistry>>,
    mut q: Query<(
        Entity,
        &GlobalTransform,
        &FacingDirection,
        &AttackPower,
        &mut AttackCooldown,
        &CritChance,
        &RewardModifiers,
    ), With<Player>>,
) {
    let Ok((player_e, player_tf, facing, power, mut cd, crit, mods)) = q.get_single_mut() else { return };
    cd.timer.tick(time.delta());
    if !input.attack_pressed || !cd.timer.finished() {
        return;
    }

    cd.timer.reset();
    let mut cd_s = cd.timer.duration().as_secs_f32();
    if mods.attack_speed_mult > 0.0 {
        cd_s *= 1.0 / (1.0 + mods.attack_speed_mult);
        cd.timer.set_duration(std::time::Duration::from_secs_f32(cd_s.max(0.08)));
    }

    spawn_player_melee_hitbox(&mut commands, &assets, player_e, player_tf, facing.0, power.0, crit.0);

    if mods.bonus_projectile {
        let proj_speed = data.as_deref().map(|d| d.player.move_speed).unwrap_or(260.0) * 2.0;
        crate::gameplay::combat::projectiles::spawn_projectile(
            &mut commands,
            &assets,
            Team::Player,
            player_tf.translation().truncate() + facing.0 * 18.0,
            facing.0 * proj_speed,
            power.0 * 0.65,
        );
    }

    particles::spawn_hit_particles(
        &mut commands,
        &assets,
        player_tf.translation().truncate() + facing.0 * 20.0,
        Color::srgba(0.7, 1.0, 0.7, 0.9),
    );
}

pub fn player_ranged_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<(
        &GlobalTransform,
        &FacingDirection,
        &AttackPower,
        &mut RangedCooldown,
    ), With<Player>>,
) {
    let Ok((tf, facing, power, mut cd)) = q.get_single_mut() else { return };
    cd.timer.tick(time.delta());
    if !input.ranged_pressed || !cd.timer.finished() {
        return;
    }
    cd.timer.reset();

    let dir = facing.0;
    let speed = 720.0;
    crate::gameplay::combat::projectiles::spawn_projectile(
        &mut commands,
        &assets,
        Team::Player,
        tf.translation().truncate() + dir * 18.0,
        dir * speed,
        power.0 * 0.6,
    );
    particles::spawn_hit_particles(
        &mut commands,
        &assets,
        tf.translation().truncate() + dir * 20.0,
        Color::srgba(0.4, 0.85, 1.0, 0.9),
    );
}

pub fn spawn_player_melee_hitbox(
    commands: &mut Commands,
    assets: &GameAssets,
    owner: Entity,
    owner_tf: &GlobalTransform,
    dir: Vec2,
    damage: f32,
    _crit: f32,
) {
    let pos = owner_tf.translation().truncate() + dir * 32.0;
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(pos.extend(60.0)),
            sprite: Sprite {
                color: Color::srgba(0.95, 0.98, 0.85, 0.35),
                custom_size: Some(Vec2::new(46.0, 26.0)),
                ..default()
            },
            ..default()
        },
        Hitbox {
            owner: Some(owner),
            team: Team::Player,
            size: Vec2::new(46.0, 26.0),
            damage,
            knockback: 360.0,
            can_crit: true,
        },
        Lifetime(Timer::from_seconds(0.08, TimerMode::Once)),
        InGameEntity,
        Name::new("PlayerHitbox"),
    ));
}

pub fn update_attack_cooldowns(time: Res<Time>, mut q: Query<&mut AttackCooldown, With<Player>>) {
    let Ok(mut cd) = q.get_single_mut() else { return };
    cd.timer.tick(time.delta());
}
