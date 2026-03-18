use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::gameplay::enemy::components::{
    ChargerPhase, ChargerState, EnemyKind, EnemyStats, EnemyType,
};
use crate::gameplay::player::components::Player;
use crate::utils::math::{clamp_in_room, clamp_length, direction_to};

#[derive(Clone, Copy)]
struct EnemySnapshot {
    entity: Entity,
    kind: EnemyType,
    pos: Vec2,
}

pub fn update_enemy_ai(
    time: Res<Time>,
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
        )>,
    )>,
) {
    let Ok(player_tf) = player_q.get_single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();
    let snapshots = enemies
        .p0()
        .iter()
        .map(|(entity, kind, tf)| EnemySnapshot {
            entity,
            kind: kind.0,
            pos: tf.translation.truncate(),
        })
        .collect::<Vec<_>>();

    for (entity, kind, stats, mut tf, mut vel, charger_state) in &mut enemies.p1() {
        let pos = tf.translation.truncate();
        let dist = pos.distance(player_pos);
        let dir = direction_to(pos, player_pos);
        let separation = separation_force(entity, pos, kind.0, &snapshots);
        let orbit_sign = if entity.index() % 2 == 0 { 1.0 } else { -1.0 };

        match kind.0 {
            EnemyType::MeleeChaser => {
                vel.0 = if dist < stats.aggro_range {
                    let close_strafe = if dist < stats.attack_range * 1.6 {
                        perpendicular(dir) * orbit_sign * 0.55
                    } else {
                        Vec2::ZERO
                    };
                    desired_velocity(dir + close_strafe + separation * 1.55, stats.move_speed)
                } else {
                    Vec2::ZERO
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
                    desired_velocity(radial + orbit + separation * 1.85, stats.move_speed * 0.92)
                } else {
                    Vec2::ZERO
                };
            }
            EnemyType::Charger => {
                let Some(mut st) = charger_state else {
                    vel.0 = Vec2::ZERO;
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
                            desired_velocity(dir + lane_shift + separation * 1.35, stats.move_speed)
                        } else {
                            Vec2::ZERO
                        };
                        if dist < stats.attack_range * 2.2 {
                            st.phase = ChargerPhase::Windup;
                            st.timer = Timer::from_seconds(0.35, TimerMode::Once);
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
                        vel.0 = st.dir * stats.move_speed * 4.0;
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
            EnemyType::Boss => {
                vel.0 = if dist < stats.aggro_range {
                    desired_velocity(dir + separation * 0.45, stats.move_speed)
                } else {
                    Vec2::ZERO
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
        EnemyType::RangedShooter => 40.0,
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
