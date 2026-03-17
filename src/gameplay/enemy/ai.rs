use bevy::prelude::*;

use crate::gameplay::enemy::components::{ChargerPhase, ChargerState, EnemyKind, EnemyStats, EnemyType};
use crate::gameplay::player::components::Player;
use crate::utils::math::{clamp_in_room, direction_to};
use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};

pub fn update_enemy_ai(
    time: Res<Time>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut enemies: Query<(
        &EnemyKind,
        &EnemyStats,
        &mut Transform,
        &mut super::systems::EnemyVelocity,
        Option<&mut ChargerState>,
    )>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let player_pos = player_tf.translation().truncate();

    for (kind, stats, mut tf, mut vel, charger_state) in &mut enemies {
        let pos = tf.translation.truncate();
        let dist = pos.distance(player_pos);
        let dir = direction_to(pos, player_pos);

        match kind.0 {
            EnemyType::MeleeChaser => {
                vel.0 = if dist < stats.aggro_range { dir * stats.move_speed } else { Vec2::ZERO };
            }
            EnemyType::RangedShooter => {
                // Keep mid distance.
                let target = 260.0;
                let delta = dist - target;
                vel.0 = if dist < stats.aggro_range { -dir * delta.signum() * stats.move_speed * 0.7 } else { Vec2::ZERO };
            }
            EnemyType::Charger => {
                let Some(mut st) = charger_state else {
                    vel.0 = Vec2::ZERO;
                    continue;
                };
                st.timer.tick(time.delta());
                match st.phase {
                    ChargerPhase::Idle => {
                        vel.0 = if dist < stats.aggro_range { dir * stats.move_speed } else { Vec2::ZERO };
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
                vel.0 = if dist < stats.aggro_range { dir * stats.move_speed } else { Vec2::ZERO };
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
