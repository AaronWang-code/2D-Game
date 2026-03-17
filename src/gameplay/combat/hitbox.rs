use bevy::prelude::*;

use crate::core::events::DamageEvent;
use crate::gameplay::combat::components::{Hitbox, Hurtbox, Team};
use crate::utils::collision::{aabb_from_transform_size, Aabb2};

pub fn detect_hitbox_hurtbox_overlap(
    mut commands: Commands,
    mut damage_ev: EventWriter<DamageEvent>,
    hitboxes: Query<(Entity, &Hitbox, &GlobalTransform)>,
    hurtboxes: Query<(Entity, &Hurtbox, &GlobalTransform)>,
) {
    for (hb_entity, hb, hb_tf) in &hitboxes {
        let hb_aabb = aabb_from_transform_size(hb_tf, hb.size);
        for (target, hurtbox, target_tf) in &hurtboxes {
            if hurtbox.team == hb.team {
                continue;
            }
            let target_aabb = aabb_from_transform_size(target_tf, hurtbox.size);
            if !hb_aabb.intersects(target_aabb) {
                continue;
            }

            let dir = (target_tf.translation().truncate() - hb_tf.translation().truncate())
                .try_normalize()
                .unwrap_or(Vec2::X);

            damage_ev.send(DamageEvent {
                target,
                source: hb.owner,
                amount: hb.damage,
                knockback: dir * hb.knockback,
                team: hb.team,
                is_crit: false,
            });

            // Single-hit hitboxes for MVP.
            commands.entity(hb_entity).despawn_recursive();
            break;
        }
    }
}

pub fn despawn_expired_hitboxes(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut super::components::Lifetime)>) {
    for (e, mut lifetime) in &mut q {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

