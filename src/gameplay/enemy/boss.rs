use bevy::prelude::*;

use crate::core::events::BossPhaseChangeEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::Team;
use crate::gameplay::combat::projectiles;
use crate::gameplay::effects::screen_shake::ScreenShakeRequest;
use crate::gameplay::enemy::components::{
    BossPatternTimer, BossPhase, EnemyKind, EnemyStats, EnemyType,
};
use crate::gameplay::player::components::{Health, Player};
use crate::utils::math::direction_to;

pub fn boss_phase_controller(
    mut phase_events: EventWriter<BossPhaseChangeEvent>,
    data: Res<GameDataRegistry>,
    mut q: Query<(&Health, &mut BossPhase), (With<EnemyKind>, Without<Player>)>,
) {
    let Ok((health, mut phase)) = q.get_single_mut() else {
        return;
    };
    let hp_ratio = if health.max > 0.0 {
        health.current / health.max
    } else {
        0.0
    };
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
    assets: Res<crate::core::assets::GameAssets>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut q: Query<
        (
            &GlobalTransform,
            &BossPhase,
            &EnemyStats,
            &mut BossPatternTimer,
        ),
        With<EnemyKind>,
    >,
    mut shake_ev: EventWriter<ScreenShakeRequest>,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();
    let Ok((boss_tf, phase, stats, mut timer)) = q.get_single_mut() else {
        return;
    };
    let boss_pos = boss_tf.translation().truncate();

    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }

    let proj_speed = stats.projectile_speed;
    let dir = direction_to(boss_pos, player_pos);

    match phase.0 {
        1 => {
            timer.0 = Timer::from_seconds(1.35, TimerMode::Once);
            timer.0.reset();
            for angle in [-0.28, 0.0, 0.28] {
                let rot = Mat2::from_angle(angle);
                projectiles::spawn_projectile(
                    &mut commands,
                    &assets,
                    Team::Enemy,
                    boss_pos + dir * 24.0,
                    rot.mul_vec2(dir) * proj_speed,
                    stats.attack_damage * 0.55,
                );
            }
        }
        2 => {
            timer.0 = Timer::from_seconds(1.50, TimerMode::Once);
            timer.0.reset();
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let d = Vec2::new(a.cos(), a.sin());
                projectiles::spawn_projectile(
                    &mut commands,
                    &assets,
                    Team::Enemy,
                    boss_pos,
                    d * proj_speed * 0.72,
                    stats.attack_damage * 0.42,
                );
            }
            shake_ev.send(ScreenShakeRequest {
                strength: 4.0,
                duration: 0.12,
            });
        }
        _ => {
            timer.0 = Timer::from_seconds(1.05, TimerMode::Once);
            timer.0.reset();
            projectiles::spawn_projectile(
                &mut commands,
                &assets,
                Team::Enemy,
                boss_pos + dir * 24.0,
                dir * proj_speed,
                stats.attack_damage * 0.72,
            );
            shake_ev.send(ScreenShakeRequest {
                strength: 6.0,
                duration: 0.14,
            });
        }
    }
}

pub fn spawn_boss_bundle(_data: &GameDataRegistry) -> (EnemyKind, BossPhase, BossPatternTimer) {
    (
        EnemyKind(EnemyType::Boss),
        BossPhase(1),
        BossPatternTimer(Timer::from_seconds(1.35, TimerMode::Once)),
    )
}
