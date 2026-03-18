use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::gameplay::combat::components::{ArcHitbox, Hitbox, Lifetime, Team};
use crate::gameplay::combat::projectiles;
use crate::gameplay::effects::particles;
use crate::gameplay::map::InGameEntity;

use super::components::*;

const BASE_RANGED_PROJECTILE_SPEED: f32 = 720.0;
const DOUBLE_SPREAD_ANGLE: f32 = 0.12;
const TRIPLE_SPREAD_ANGLE: f32 = 0.24;
const NOVA_PROJECTILE_COUNT: usize = 8;
const MELEE_HITBOX_LIFETIME_S: f32 = 0.09;
const MELEE_SLASH_EFFECT_LIFETIME_S: f32 = 0.16;

#[derive(Component, Debug, Clone)]
pub struct MeleeSlashEffect {
    pub timer: Timer,
    pub base_alpha: f32,
    pub base_scale: Vec3,
}

pub fn player_attack_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    assets: Res<GameAssets>,
    mut q: Query<
        (
            Entity,
            &GlobalTransform,
            &FacingDirection,
            &AttackPower,
            &mut AttackCooldown,
            &CritChance,
            &RewardModifiers,
        ),
        With<Player>,
    >,
) {
    let Ok((player_e, player_tf, facing, power, mut cd, crit, mods)) = q.get_single_mut() else {
        return;
    };
    if !input.attack_held || !cd.timer.finished() {
        return;
    }

    cd.timer.reset();
    let melee_reach = melee_arc_radius(*mods);

    spawn_player_melee_hitbox(
        &mut commands,
        &assets,
        player_e,
        player_tf,
        facing.0,
        power.0 * mods.melee_damage_mult(),
        crit.0,
        *mods,
    );

    particles::spawn_hit_particles(
        &mut commands,
        &assets,
        player_tf.translation().truncate() + facing.0 * (melee_reach - 8.0),
        Color::srgba(0.7, 1.0, 0.7, 0.9),
    );
}

pub fn player_ranged_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    assets: Res<GameAssets>,
    mut q: Query<
        (
            Entity,
            &GlobalTransform,
            &FacingDirection,
            &AttackPower,
            &CritChance,
            &mut RangedCooldown,
            &RewardModifiers,
        ),
        With<Player>,
    >,
) {
    let Ok((player_e, tf, facing, power, crit, mut cd, mods)) = q.get_single_mut() else {
        return;
    };
    if !input.ranged_held || !cd.timer.finished() {
        return;
    }
    cd.timer.reset();

    let dir = facing.0;
    let speed = BASE_RANGED_PROJECTILE_SPEED * mods.ranged_projectile_speed_mult();
    let damage = power.0 * 0.64 * mods.ranged_damage_mult();
    spawn_player_ranged_volley(
        &mut commands,
        &assets,
        player_e,
        tf.translation().truncate() + dir * 18.0,
        dir,
        speed,
        damage,
        crit.0,
        *mods,
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
    crit_chance: f32,
    mods: RewardModifiers,
) {
    let owner_pos = owner_tf.translation().truncate();
    let direction = dir.try_normalize().unwrap_or(Vec2::X);
    let radius = melee_arc_radius(mods);
    let half_angle = mods.melee_arc_half_angle_rad();
    let pos = owner_pos + direction * (radius * 0.55);
    let hitbox_size = Vec2::splat((radius + 18.0) * 1.35);
    let slash_size = Vec2::new(
        (radius + 56.0) * 1.45,
        (88.0 + mods.melee_mastery_stacks as f32 * 7.0) * mods.melee_slash_scale(),
    );

    commands.spawn((
        SpriteBundle {
            texture: assets.textures.slash.clone(),
            transform: Transform {
                translation: pos.extend(61.0),
                rotation: Quat::from_rotation_z(direction.y.atan2(direction.x)),
                scale: Vec3::ONE,
            },
            sprite: Sprite {
                color: Color::srgba(0.78, 1.0, 0.93, 0.82),
                custom_size: Some(slash_size),
                ..default()
            },
            ..default()
        },
        MeleeSlashEffect {
            timer: Timer::from_seconds(MELEE_SLASH_EFFECT_LIFETIME_S, TimerMode::Once),
            base_alpha: 0.82,
            base_scale: Vec3::ONE,
        },
        InGameEntity,
        Name::new("MeleeSlashEffect"),
    ));

    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(pos.extend(60.0)),
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                custom_size: Some(hitbox_size),
                ..default()
            },
            ..default()
        },
        Hitbox {
            owner: Some(owner),
            team: Team::Player,
            size: hitbox_size,
            damage,
            knockback: 360.0 + mods.melee_mastery_stacks as f32 * 12.0,
            can_crit: true,
            crit_chance,
            crit_multiplier: 1.75,
        },
        ArcHitbox {
            origin: owner_pos,
            direction,
            radius,
            half_angle_rad: half_angle,
        },
        Lifetime(Timer::from_seconds(
            MELEE_HITBOX_LIFETIME_S,
            TimerMode::Once,
        )),
        InGameEntity,
        Name::new("PlayerHitbox"),
    ));
}

fn spawn_player_ranged_volley(
    commands: &mut Commands,
    assets: &GameAssets,
    owner: Entity,
    pos: Vec2,
    dir: Vec2,
    projectile_speed: f32,
    damage: f32,
    crit_chance: f32,
    mods: RewardModifiers,
) {
    match mods.ranged_volley_pattern() {
        RangedVolleyPattern::Single => {
            spawn_ranged_projectile(
                commands,
                assets,
                owner,
                pos,
                dir,
                projectile_speed,
                damage,
                crit_chance,
            );
        }
        RangedVolleyPattern::Double => {
            for angle in [-DOUBLE_SPREAD_ANGLE, DOUBLE_SPREAD_ANGLE] {
                let shot_dir = Mat2::from_angle(angle).mul_vec2(dir);
                spawn_ranged_projectile(
                    commands,
                    assets,
                    owner,
                    pos,
                    shot_dir,
                    projectile_speed,
                    damage * 0.70,
                    crit_chance,
                );
            }
        }
        RangedVolleyPattern::Triple => {
            for angle in [-TRIPLE_SPREAD_ANGLE, 0.0, TRIPLE_SPREAD_ANGLE] {
                let shot_dir = Mat2::from_angle(angle).mul_vec2(dir);
                spawn_ranged_projectile(
                    commands,
                    assets,
                    owner,
                    pos,
                    shot_dir,
                    projectile_speed,
                    damage * 0.56,
                    crit_chance,
                );
            }
        }
        RangedVolleyPattern::Nova => {
            let base_angle = dir.y.atan2(dir.x);
            for i in 0..NOVA_PROJECTILE_COUNT {
                let angle =
                    base_angle + i as f32 / NOVA_PROJECTILE_COUNT as f32 * std::f32::consts::TAU;
                let shot_dir = Vec2::new(angle.cos(), angle.sin());
                spawn_ranged_projectile(
                    commands,
                    assets,
                    owner,
                    pos,
                    shot_dir,
                    projectile_speed,
                    damage * 0.20,
                    crit_chance,
                );
            }
        }
    }
}

fn spawn_ranged_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    owner: Entity,
    pos: Vec2,
    dir: Vec2,
    projectile_speed: f32,
    damage: f32,
    crit_chance: f32,
) {
    projectiles::spawn_player_projectile(
        commands,
        assets,
        owner,
        pos,
        dir * projectile_speed,
        damage,
        crit_chance,
    );
}

pub fn update_attack_cooldowns(
    time: Res<Time>,
    mut q: Query<(&mut AttackCooldown, &mut RangedCooldown), With<Player>>,
) {
    let Ok((mut attack_cd, mut ranged_cd)) = q.get_single_mut() else {
        return;
    };
    attack_cd.timer.tick(time.delta());
    ranged_cd.timer.tick(time.delta());
}

pub fn update_melee_slash_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut MeleeSlashEffect, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut effect, mut sprite, mut transform) in &mut q {
        effect.timer.tick(time.delta());
        let progress = effect.timer.fraction();
        sprite
            .color
            .set_alpha(effect.base_alpha * (1.0 - progress).clamp(0.0, 1.0));
        transform.scale = effect.base_scale * (1.0 + progress * 0.18);

        if effect.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn melee_arc_radius(mods: RewardModifiers) -> f32 {
    36.0 + mods.melee_range_bonus()
}
