use bevy::prelude::*;

use crate::core::events::BossPhaseChangeEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::Team;
use crate::gameplay::combat::projectiles;
use crate::gameplay::effects::screen_shake::ScreenShakeRequest;
use crate::gameplay::enemy::components::{BossPatternTimer, BossPhase, EnemyKind, EnemyType};
use crate::gameplay::player::components::{Health, Player};
use crate::coop::components::CoopPlayer;
use crate::utils::math::direction_to;

pub fn boss_phase_controller(
    mut phase_events: EventWriter<BossPhaseChangeEvent>,
    data: Res<GameDataRegistry>,
    mut q: Query<(&Health, &mut BossPhase), (With<EnemyKind>, Without<Player>)>,
) {
    let Ok((health, mut phase)) = q.get_single_mut() else { return };
    let hp_ratio = if health.max > 0.0 { health.current / health.max } else { 0.0 };
    let thresholds = &data.boss.phase_thresholds;
    let new_phase = if thresholds.get(1).is_some_and(|t| hp_ratio <= *t) {
        3
    } else if thresholds.get(0).is_some_and(|t| hp_ratio <= *t) {
        2
    } else {
        1
    };
    if phase.0 != new_phase {
        phase.0 = new_phase;
        phase_events.send(BossPhaseChangeEvent { phase: new_phase });
    }
}

pub fn boss_attack_patterns(
    mut commands: Commands,
    time: Res<Time>,
    data: Res<GameDataRegistry>,
    assets: Res<crate::core::assets::GameAssets>,
    player_q: Query<&GlobalTransform, Or<(With<Player>, With<CoopPlayer>)>>,
    mut q: Query<(&GlobalTransform, &BossPhase, &mut BossPatternTimer), With<EnemyKind>>,
    mut shake_ev: EventWriter<ScreenShakeRequest>,
) {
    let player_positions: Vec<Vec2> = player_q
        .iter()
        .map(|tf| tf.translation().truncate())
        .collect();
    if player_positions.is_empty() {
        return;
    }
    let Ok((boss_tf, phase, mut timer)) = q.get_single_mut() else { return };
    let boss_pos = boss_tf.translation().truncate();
    let player_pos = player_positions
        .iter()
        .copied()
        .min_by(|a, b| boss_pos.distance(*a).total_cmp(&boss_pos.distance(*b)))
        .unwrap();

    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }

    let proj_speed = data.boss.projectile_speed;
    let dir = direction_to(boss_pos, player_pos);

    match phase.0 {
        1 => {
            timer.0 = Timer::from_seconds(1.1, TimerMode::Once);
            timer.0.reset();
            // Small fan.
            for angle in [-0.35, 0.0, 0.35] {
                let rot = Mat2::from_angle(angle);
                projectiles::spawn_projectile(
                    &mut commands,
                    &assets,
                    Team::Enemy,
                    boss_pos + dir * 24.0,
                    rot.mul_vec2(dir) * proj_speed,
                    data.boss.contact_damage * 0.7,
                );
            }
        }
        2 => {
            timer.0 = Timer::from_seconds(1.25, TimerMode::Once);
            timer.0.reset();
            for i in 0..10 {
                let a = i as f32 / 10.0 * std::f32::consts::TAU;
                let d = Vec2::new(a.cos(), a.sin());
                projectiles::spawn_projectile(
                    &mut commands,
                    &assets,
                    Team::Enemy,
                    boss_pos,
                    d * proj_speed * 0.8,
                    data.boss.contact_damage * 0.55,
                );
            }
            shake_ev.send(ScreenShakeRequest {
                strength: 6.0,
                duration: 0.12,
            });
        }
        _ => {
            timer.0 = Timer::from_seconds(0.85, TimerMode::Once);
            timer.0.reset();
            projectiles::spawn_projectile(
                &mut commands,
                &assets,
                Team::Enemy,
                boss_pos + dir * 24.0,
                dir * proj_speed * 1.1,
                data.boss.contact_damage * 0.9,
            );
            shake_ev.send(ScreenShakeRequest {
                strength: 8.0,
                duration: 0.14,
            });
        }
    }
}

pub fn spawn_boss_bundle(data: &GameDataRegistry) -> (EnemyKind, BossPhase, BossPatternTimer) {
    (
        EnemyKind(EnemyType::Boss),
        BossPhase(1),
        BossPatternTimer(Timer::from_seconds(1.2, TimerMode::Once)),
    )
}
