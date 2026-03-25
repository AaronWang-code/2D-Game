use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::gameplay::enemy::components::{
    ChargerPhase, ChargerState, EnemyBuffState, EnemyKind, EnemyStats, EnemyType, FlankerPhase,
    FlankerState, SniperPhase, SniperState,
};
use crate::gameplay::player::components::Player;
use crate::utils::math::{clamp_in_room, clamp_length, direction_to};
use crate::utils::rng::GameRng;

use super::systems::effective_enemy_move_speed;

#[derive(Clone, Copy)]
struct EnemySnapshot {
    entity: Entity,
    kind: EnemyType,
    pos: Vec2,
}

pub fn update_enemy_ai(
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut enemies: ParamSet<(
        Query<(Entity, &EnemyKind, &Transform)>,
        Query<(
            Entity,
            &EnemyKind,
            &EnemyStats,
            &mut Transform,
            &mut super::systems::EnemyVelocity,
            Option<&mut ChargerState>,
            Option<&mut FlankerState>,
            Option<&SniperState>,
            Option<&EnemyBuffState>,
        )>,
    )>,
) {
    let player_positions: Vec<Vec2> =
        player_q.iter().map(|tf| tf.translation().truncate()).collect();
    if player_positions.is_empty() {
        return;
    }

    let snapshots = enemies
        .p0()
        .iter()
        .map(|(entity, kind, tf)| EnemySnapshot {
            entity,
            kind: kind.0,
            pos: tf.translation.truncate(),
        })
        .collect::<Vec<_>>();

    for (entity, kind, stats, mut tf, mut vel, charger_state, flanker_state, sniper_state, buff) in
        &mut enemies.p1()
    {
        let pos = tf.translation.truncate();
        let (player_pos, dist) = player_positions
            .iter()
            .map(|p| (*p, pos.distance(*p)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        let dir = direction_to(pos, player_pos);
        let separation = separation_force(entity, pos, kind.0, &snapshots);
        let orbit_sign = if entity.index() % 2 == 0 { 1.0 } else { -1.0 };
        let move_speed = effective_enemy_move_speed(stats, buff);

        match kind.0 {
            EnemyType::MeleeChaser => {
                vel.0 = if dist < stats.aggro_range {
                    let close_strafe = if dist < stats.attack_range * 1.6 {
                        perpendicular(dir) * orbit_sign * 0.55
                    } else {
                        Vec2::ZERO
                    };
                    desired_velocity(dir + close_strafe + separation * 1.55, move_speed)
                } else {
                    far_pursuit_velocity(dir, separation, move_speed, 0.62, 1.10)
                };
            }
            EnemyType::RangedShooter => {
                vel.0 = if dist < stats.aggro_range {
                    let target_dist = (stats.attack_range * 0.72).clamp(180.0, 280.0);
                    let radial = if dist < target_dist * 0.88 {
                        -dir * 1.15
                    } else if dist > target_dist * 1.15 {
                        dir * 0.95
                    } else {
                        Vec2::ZERO
                    };
                    let orbit = perpendicular(dir) * orbit_sign * 1.05;
                    desired_velocity(radial + orbit + separation * 1.85, move_speed * 0.92)
                } else {
                    far_pursuit_velocity(dir, separation, move_speed, 0.55, 1.25)
                };
            }
            EnemyType::Charger => {
                let Some(mut st) = charger_state else {
                    vel.0 = far_pursuit_velocity(dir, separation, move_speed, 0.60, 1.00);
                    continue;
                };
                st.timer.tick(time.delta());
                match st.phase {
                    ChargerPhase::Idle => {
                        vel.0 = if dist < stats.aggro_range {
                            let lane_shift = if dist < stats.attack_range * 2.8 {
                                perpendicular(dir) * orbit_sign * 0.42
                            } else {
                                Vec2::ZERO
                            };
                            desired_velocity(dir + lane_shift + separation * 1.35, move_speed)
                        } else {
                            far_pursuit_velocity(dir, separation, move_speed, 0.66, 1.00)
                        };
                        if dist < stats.attack_range * 2.2 {
                            st.phase = ChargerPhase::Windup;
                            st.timer = Timer::from_seconds(
                                if stats.aggro_range >= 620.0 {
                                    0.28
                                } else {
                                    0.35
                                },
                                TimerMode::Once,
                            );
                            st.timer.reset();
                            st.dir = dir;
                        }
                    }
                    ChargerPhase::Windup => {
                        vel.0 = Vec2::ZERO;
                        if st.timer.finished() {
                            st.phase = ChargerPhase::Charging;
                            st.timer = Timer::from_seconds(0.32, TimerMode::Once);
                            st.timer.reset();
                        }
                    }
                    ChargerPhase::Charging => {
                        vel.0 = st.dir * move_speed * 4.0;
                        if st.timer.finished() {
                            st.phase = ChargerPhase::Stunned;
                            st.timer = Timer::from_seconds(0.5, TimerMode::Once);
                            st.timer.reset();
                        }
                    }
                    ChargerPhase::Stunned => {
                        vel.0 = Vec2::ZERO;
                        if st.timer.finished() {
                            st.phase = ChargerPhase::Idle;
                            st.timer = Timer::from_seconds(0.1, TimerMode::Once);
                            st.timer.reset();
                        }
                    }
                }
            }
            EnemyType::Flanker => {
                let Some(mut flanker) = flanker_state else {
                    vel.0 = far_pursuit_velocity(dir, separation, move_speed, 0.72, 1.10);
                    continue;
                };
                flanker.timer.tick(time.delta());
                flanker.repath_timer.tick(time.delta());
                if flanker.repath_timer.finished() {
                    flanker.strafe_sign = if rng.gen_range_f32(0.0, 1.0) < 0.5 {
                        -1.0
                    } else {
                        1.0
                    };
                    flanker.repath_timer =
                        Timer::from_seconds(rng.gen_range_f32(0.35, 0.55), TimerMode::Once);
                    flanker.repath_timer.reset();
                }
                match flanker.phase {
                    FlankerPhase::Stalk => {
                        let flank_orbit = perpendicular(dir) * flanker.strafe_sign * 1.45;
                        vel.0 = if dist < stats.aggro_range {
                            desired_velocity(
                                dir * 0.52 + flank_orbit + separation * 1.15,
                                move_speed * 1.10,
                            )
                        } else {
                            far_pursuit_velocity(dir, separation, move_speed, 0.78, 1.05)
                        };
                        if dist < stats.attack_range * 2.9 {
                            flanker.phase = FlankerPhase::Windup;
                            flanker.timer = Timer::from_seconds(0.14, TimerMode::Once);
                            flanker.timer.reset();
                            flanker.dir = dir;
                        }
                    }
                    FlankerPhase::Windup => {
                        vel.0 = perpendicular(dir) * flanker.strafe_sign * move_speed * 0.20;
                        if flanker.timer.finished() {
                            flanker.phase = FlankerPhase::Lunging;
                            flanker.timer = Timer::from_seconds(0.18, TimerMode::Once);
                            flanker.timer.reset();
                            flanker.dir = dir;
                        }
                    }
                    FlankerPhase::Lunging => {
                        vel.0 = flanker.dir * move_speed * 3.8;
                        if flanker.timer.finished() {
                            flanker.phase = FlankerPhase::Recover;
                            flanker.timer = Timer::from_seconds(0.22, TimerMode::Once);
                            flanker.timer.reset();
                        }
                    }
                    FlankerPhase::Recover => {
                        vel.0 = perpendicular(dir) * flanker.strafe_sign * move_speed * 0.28;
                        if flanker.timer.finished() {
                            flanker.phase = FlankerPhase::Stalk;
                            flanker.timer = Timer::from_seconds(0.1, TimerMode::Once);
                            flanker.timer.reset();
                        }
                    }
                }
            }
            EnemyType::Sniper => {
                let holding_line = sniper_state
                    .as_ref()
                    .map(|value| matches!(value.phase, SniperPhase::Aiming | SniperPhase::Recover))
                    .unwrap_or(false);
                if holding_line {
                    vel.0 = Vec2::ZERO;
                } else if dist < stats.aggro_range {
                    let target_dist = (stats.attack_range * 0.74).clamp(280.0, 430.0);
                    let radial = if dist < target_dist * 0.78 {
                        -dir * 0.94
                    } else if dist > target_dist * 1.08 {
                        dir * 0.68
                    } else {
                        Vec2::ZERO
                    };
                    let orbit = perpendicular(dir) * orbit_sign * 0.54;
                    vel.0 = desired_velocity(radial + orbit + separation * 1.20, move_speed * 0.84);
                } else {
                    vel.0 = far_pursuit_velocity(dir, separation, move_speed, 0.46, 1.10);
                }
            }
            EnemyType::SupportCaster => {
                vel.0 = if dist < stats.aggro_range {
                    let target_dist = (stats.attack_range * 1.15).clamp(220.0, 340.0);
                    let radial = if dist < target_dist * 0.86 {
                        -dir * 1.18
                    } else if dist > target_dist * 1.12 {
                        dir * 0.70
                    } else {
                        Vec2::ZERO
                    };
                    let orbit = perpendicular(dir) * orbit_sign * 0.78;
                    desired_velocity(radial + orbit + separation * 1.65, move_speed * 0.74)
                } else {
                    far_pursuit_velocity(dir, separation, move_speed, 0.42, 1.15)
                };
            }
            EnemyType::Boss => {
                vel.0 = if dist < stats.aggro_range {
                    desired_velocity(dir + separation * 0.45, move_speed)
                } else {
                    far_pursuit_velocity(dir, separation, move_speed, 0.72, 0.55)
                };
            }
        }

        tf.translation += (vel.0 * time.delta_seconds()).extend(0.0);

        let clamped = clamp_in_room(
            tf.translation.truncate(),
            Vec2::new(ROOM_HALF_WIDTH, ROOM_HALF_HEIGHT),
            26.0,
        );
        tf.translation.x = clamped.x;
        tf.translation.y = clamped.y;
    }
}

fn desired_velocity(input: Vec2, speed: f32) -> Vec2 {
    clamp_length(input.normalize_or_zero() * speed, speed)
}

fn far_pursuit_velocity(
    dir: Vec2,
    separation: Vec2,
    speed: f32,
    speed_mult: f32,
    separation_mult: f32,
) -> Vec2 {
    desired_velocity(dir + separation * separation_mult, speed * speed_mult)
}

fn perpendicular(dir: Vec2) -> Vec2 {
    Vec2::new(-dir.y, dir.x)
}

fn separation_force(
    entity: Entity,
    pos: Vec2,
    kind: EnemyType,
    snapshots: &[EnemySnapshot],
) -> Vec2 {
    let personal_space = match kind {
        EnemyType::Boss => 72.0,
        EnemyType::Charger => 44.0,
        EnemyType::SupportCaster => 42.0,
        EnemyType::Sniper => 40.0,
        EnemyType::RangedShooter => 40.0,
        EnemyType::Flanker => 30.0,
        EnemyType::MeleeChaser => 36.0,
    };

    let mut force = Vec2::ZERO;
    let space_sq = personal_space * personal_space;

    for snapshot in snapshots {
        if snapshot.entity == entity {
            continue;
        }

        let offset = pos - snapshot.pos;
        let dist_sq = offset.length_squared();
        if dist_sq > space_sq {
            continue;
        }

        if dist_sq <= 0.0001 {
            let angle = ((entity.index() ^ snapshot.entity.index()) % 360) as f32
                * std::f32::consts::TAU
                / 360.0;
            force += Vec2::new(angle.cos(), angle.sin()) * 0.8;
            continue;
        }

        let dist = dist_sq.sqrt();
        let same_kind_bias = if snapshot.kind == kind { 1.15 } else { 0.95 };
        let strength = ((personal_space - dist) / personal_space) * same_kind_bias;
        force += offset / dist * strength;
    }

    force
}
