use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::events::DamageEvent;
use crate::gameplay::combat::components::{
    ArcHitbox, Hitbox, Hurtbox, Projectile, RuptureDot, Team,
};
use crate::gameplay::effects::particles;
use crate::gameplay::enemy::components::BossArchetype;
use crate::gameplay::player::components::RewardModifiers;
use crate::gameplay::player::components::{Health, Player};
use crate::utils::collision::Aabb2;
use crate::utils::collision::aabb_from_transform_size;
use crate::utils::entity::safe_despawn_recursive;
use crate::utils::rng::GameRng;

#[derive(Clone, Copy)]
struct MeleeReflector {
    owner: Entity,
    aabb: Aabb2,
    arc: ArcHitbox,
}

pub fn reflect_enemy_projectiles_on_melee(
    mut commands: Commands,
    assets: Res<GameAssets>,
    owner_mods: Query<&RewardModifiers>,
    mut collision_sets: ParamSet<(
        Query<(&Hitbox, &GlobalTransform, Option<&ArcHitbox>)>,
        Query<(
            Entity,
            &mut Projectile,
            &mut Hitbox,
            &GlobalTransform,
            &mut Transform,
            &mut Sprite,
        )>,
    )>,
) {
    let mut reflectors = Vec::new();

    {
        for (hitbox, hitbox_tf, arc) in &collision_sets.p0() {
            if hitbox.team != Team::Player {
                continue;
            }

            let Some(arc) = arc.copied() else {
                continue;
            };
            let Some(owner) = hitbox.owner else {
                continue;
            };
            let Ok(mods) = owner_mods.get(owner) else {
                continue;
            };
            if !mods.melee_projectile_reflect_unlocked() {
                continue;
            }

            reflectors.push(MeleeReflector {
                owner,
                aabb: aabb_from_transform_size(hitbox_tf, hitbox.size),
                arc,
            });
        }
    }

    if reflectors.is_empty() {
        return;
    }

    for (
        _projectile_entity,
        mut projectile,
        mut projectile_hitbox,
        projectile_tf,
        mut tf,
        mut sprite,
    ) in &mut collision_sets.p1()
    {
        if projectile.team != Team::Enemy {
            continue;
        }

        let projectile_proxy = Hurtbox {
            team: Team::Enemy,
            size: projectile_hitbox.size,
        };
        let Some(reflector) = reflectors.iter().find(|reflector| {
            hitbox_intersects_target(
                reflector.aabb,
                Some(reflector.arc),
                &projectile_proxy,
                projectile_tf,
            )
        }) else {
            continue;
        };

        let reflected_dir = blended_reflect_direction(projectile.velocity, reflector.arc.direction);
        let reflected_speed = projectile.velocity.length().max(360.0) * 1.1;
        projectile.velocity = reflected_dir * reflected_speed;
        projectile.team = Team::Player;

        projectile_hitbox.owner = Some(reflector.owner);
        projectile_hitbox.team = Team::Player;
        projectile_hitbox.damage *= 1.1;
        projectile_hitbox.knockback = projectile_hitbox.knockback.max(260.0);
        projectile_hitbox.can_crit = false;
        projectile_hitbox.crit_chance = 0.0;
        projectile_hitbox.crit_multiplier = 1.0;

        tf.translation += (reflected_dir * 12.0).extend(0.0);
        tf.rotation = Quat::from_rotation_z(reflected_dir.y.atan2(reflected_dir.x));
        sprite.color = Color::srgb(0.86, 1.0, 0.52);
        sprite.custom_size = Some(Vec2::new(18.0, 9.0));

        particles::spawn_hit_particles(
            &mut commands,
            &assets,
            projectile_tf.translation().truncate(),
            Color::srgba(0.92, 1.0, 0.72, 0.9),
        );
    }
}

pub fn detect_hitbox_hurtbox_overlap(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut damage_ev: EventWriter<DamageEvent>,
    mut rng: ResMut<GameRng>,
    owner_mods: Query<&RewardModifiers>,
    mut owner_health_q: Query<&mut Health, With<Player>>,
    boss_q: Query<(), With<BossArchetype>>,
    existing_ruptures: Query<&RuptureDot>,
    hitboxes: Query<(Entity, &Hitbox, &GlobalTransform, Option<&ArcHitbox>)>,
    hurtboxes: Query<(Entity, &Hurtbox, &GlobalTransform)>,
) {
    for (hb_entity, hb, hb_tf, arc) in &hitboxes {
        let hb_aabb = aabb_from_transform_size(hb_tf, hb.size);
        for (target, hurtbox, target_tf) in &hurtboxes {
            if hurtbox.team == hb.team {
                continue;
            }
            if !hitbox_intersects_target(hb_aabb, arc.copied(), hurtbox, target_tf) {
                continue;
            }

            let dir = (target_tf.translation().truncate() - hb_tf.translation().truncate())
                .try_normalize()
                .unwrap_or(Vec2::X);
            let is_crit = hb.can_crit
                && hb.crit_chance > 0.0
                && rng.gen_range_f32(0.0, 1.0) < hb.crit_chance.clamp(0.0, 1.0);
            let amount = if is_crit {
                hb.damage * hb.crit_multiplier.max(1.0)
            } else {
                hb.damage
            };
            let is_melee_arc = arc.is_some() && hb.team == Team::Player;
            let target_is_boss = boss_q.get(target).is_ok();

            damage_ev.send(DamageEvent {
                target,
                source: hb.owner,
                amount,
                knockback: dir * hb.knockback,
                team: hb.team,
                is_crit,
            });

            if is_melee_arc {
                if let Some(owner) = hb.owner {
                    if let Ok(mods) = owner_mods.get(owner) {
                        let heal_fraction = mods.melee_on_hit_heal_fraction(target_is_boss);
                        if heal_fraction > 0.0 {
                            if let Ok(mut owner_health) = owner_health_q.get_mut(owner) {
                                let before = owner_health.current;
                                owner_health.current = (owner_health.current
                                    + (amount * heal_fraction).min(2.0))
                                .min(owner_health.max);
                                if owner_health.current > before + f32::EPSILON {
                                    particles::spawn_hit_particles(
                                        &mut commands,
                                        &assets,
                                        target_tf.translation().truncate(),
                                        Color::srgba(0.52, 1.0, 0.60, 0.76),
                                    );
                                }
                            }
                        }

                        let rupture_fraction = mods.melee_rupture_total_fraction();
                        if rupture_fraction > 0.0 {
                            let per_tick = (amount * rupture_fraction / 3.0).max(0.1);
                            let should_refresh = existing_ruptures
                                .get(target)
                                .map(|current| per_tick > current.damage_per_tick)
                                .unwrap_or(true);
                            if should_refresh {
                                let rupture = RuptureDot {
                                    source: Some(owner),
                                    damage_per_tick: per_tick,
                                    ticks_remaining: 3,
                                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                                };
                                commands.entity(target).insert(rupture);
                                particles::spawn_hit_particles(
                                    &mut commands,
                                    &assets,
                                    target_tf.translation().truncate(),
                                    Color::srgba(1.0, 0.42, 0.46, 0.80),
                                );
                            }
                        }
                    }
                }
            }

            // Single-hit hitboxes for MVP.
            safe_despawn_recursive(&mut commands, hb_entity);
            break;
        }
    }
}

fn hitbox_intersects_target(
    hb_aabb: crate::utils::collision::Aabb2,
    arc: Option<ArcHitbox>,
    hurtbox: &Hurtbox,
    target_tf: &GlobalTransform,
) -> bool {
    if let Some(arc) = arc {
        return arc_hitbox_intersects_target(arc, hurtbox, target_tf);
    }

    let target_aabb = aabb_from_transform_size(target_tf, hurtbox.size);
    hb_aabb.intersects(target_aabb)
}

fn arc_hitbox_intersects_target(
    arc: ArcHitbox,
    hurtbox: &Hurtbox,
    target_tf: &GlobalTransform,
) -> bool {
    let to_target = target_tf.translation().truncate() - arc.origin;
    let distance = to_target.length();
    let target_radius = hurtbox.size.length() * 0.35;

    if distance > arc.radius + target_radius {
        return false;
    }
    if distance <= target_radius {
        return true;
    }

    let Some(target_dir) = to_target.try_normalize() else {
        return true;
    };
    let facing = arc.direction.try_normalize().unwrap_or(Vec2::X);
    let dot = facing.dot(target_dir).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let angular_padding = (target_radius / distance.max(target_radius + 0.001))
        .clamp(0.0, 0.95)
        .asin();

    angle <= arc.half_angle_rad + angular_padding
}

pub fn despawn_expired_hitboxes(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut super::components::Lifetime)>,
) {
    for (e, mut lifetime) in &mut q {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            safe_despawn_recursive(&mut commands, e);
        }
    }
}

pub fn tick_rupture_dots(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_ev: EventWriter<DamageEvent>,
    mut q: Query<(Entity, &mut RuptureDot)>,
) {
    for (entity, mut rupture) in &mut q {
        rupture.timer.tick(time.delta());
        if rupture.timer.just_finished() && rupture.ticks_remaining > 0 {
            rupture.ticks_remaining -= 1;
            damage_ev.send(DamageEvent {
                target: entity,
                source: rupture.source,
                amount: rupture.damage_per_tick,
                knockback: Vec2::ZERO,
                team: Team::Player,
                is_crit: false,
            });
        }

        if rupture.ticks_remaining == 0 {
            commands.entity(entity).remove::<RuptureDot>();
        }
    }
}

fn blended_reflect_direction(projectile_velocity: Vec2, slash_direction: Vec2) -> Vec2 {
    let slash = slash_direction.normalize_or_zero();
    let away = (-projectile_velocity).normalize_or_zero();
    let blended = slash * 0.75 + away * 0.25;
    if blended.length_squared() > 0.0 {
        blended.normalize()
    } else if slash.length_squared() > 0.0 {
        slash
    } else if away.length_squared() > 0.0 {
        away
    } else {
        Vec2::X
    }
}
