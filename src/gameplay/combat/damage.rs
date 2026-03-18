use bevy::prelude::*;

use crate::core::events::{DamageAppliedEvent, DamageEvent, DeathEvent};
use crate::gameplay::combat::components::{Knockback, Hurtbox, Team};
use crate::gameplay::effects::flash::Flash;
use crate::gameplay::player::components::{Health, InvincibilityTimer};

pub fn apply_damage_events(
    mut damage_events: EventReader<DamageEvent>,
    mut applied_events: EventWriter<DamageAppliedEvent>,
    mut death_events: EventWriter<DeathEvent>,
    mut q: Query<(
        Entity,
        &mut Health,
        Option<&mut InvincibilityTimer>,
        Option<&Hurtbox>,
        Option<&mut Flash>,
        &mut Knockback,
        &GlobalTransform,
    )>,
) {
    for ev in damage_events.read() {
        let Ok((entity, mut health, inv_opt, hurtbox, flash_opt, mut knockback, tf)) = q.get_mut(ev.target) else {
            continue;
        };

        if let Some(mut inv) = inv_opt {
            if !inv.timer.finished() {
                continue;
            }
            inv.timer.reset();
        }

        health.current = (health.current - ev.amount).max(0.0);
        applied_events.send(DamageAppliedEvent {
            target: entity,
            amount: ev.amount,
            attacker_team: ev.team,
            target_team: hurtbox.map(|h| h.team),
            is_crit: ev.is_crit,
            pos: tf.translation().truncate(),
        });

        if let Some(mut flash) = flash_opt {
            flash.trigger(0.12);
        }

        knockback.0 = ev.knockback;

        if health.current <= 0.0 {
            let team = hurtbox.map(|h| h.team).unwrap_or(Team::Enemy);
            death_events.send(DeathEvent { entity, team });
        }
    }
}

pub fn apply_knockback_decay(
    time: Res<Time>,
    mut q: Query<(&mut Knockback, &mut Transform)>,
) {
    for (mut kb, mut tf) in &mut q {
        tf.translation += (kb.0 * time.delta_seconds()).extend(0.0);
        kb.0 *= 0.0;
    }
}
