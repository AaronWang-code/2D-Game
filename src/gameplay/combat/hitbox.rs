use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::events::DamageEvent;
use crate::gameplay::combat::components::{ArcHitbox, Hitbox, Hurtbox, Projectile, Team};
use crate::gameplay::effects::particles;
use crate::gameplay::player::components::RewardModifiers;
use crate::utils::collision::Aabb2;
use crate::utils::collision::aabb_from_transform_size;
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
    mut damage_ev: EventWriter<DamageEvent>,
    mut rng: ResMut<GameRng>,
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

            damage_ev.send(DamageEvent {
                target,
                source: hb.owner,
                amount,
                knockback: dir * hb.knockback,
                team: hb.team,
                is_crit,
            });

            // Single-hit hitboxes for MVP.
            commands.entity(hb_entity).despawn_recursive();
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
            commands.entity(e).despawn_recursive();
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
