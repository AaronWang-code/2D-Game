use bevy::prelude::*;

use crate::coop::net::{CoopNetConfig, CoopRemoteInput, NetMode};
use crate::core::assets::GameAssets;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::{Hurtbox, Knockback, Team};
use crate::gameplay::combat::projectiles;
use crate::gameplay::effects::flash::Flash;
use crate::gameplay::effects::particles;
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::player::combat::spawn_player_melee_hitbox;
use crate::gameplay::player::components::*;
use crate::gameplay::shop::{self, ShopOffers};
use crate::utils::rng::GameRng;
use crate::utils::math::{clamp_in_room, clamp_length};
use crate::{constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH}, core::events::DeathEvent};

use super::components::CoopPlayer;

#[derive(Component)]
pub struct CoopMateHeadText;

pub fn ensure_coop_player_spawned_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    data: Option<Res<GameDataRegistry>>,
    config: Res<CoopNetConfig>,
    existing_q: Query<(), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    if existing_q.iter().next().is_some() {
        return;
    }

    let cfg = data.as_deref().map(|d| &d.player);
    let max_hp = cfg.map(|c| c.max_hp).unwrap_or(100.0);
    let move_speed = cfg.map(|c| c.move_speed).unwrap_or(260.0);
    let attack_power = cfg.map(|c| c.attack_power).unwrap_or(18.0);
    let attack_cd = cfg.map(|c| c.attack_cooldown_s).unwrap_or(0.35);
    let dash_cd = cfg.map(|c| c.dash_cooldown_s).unwrap_or(1.2);
    let dash_speed = cfg.map(|c| c.dash_speed).unwrap_or(680.0);
    let dash_duration = cfg.map(|c| c.dash_duration_s).unwrap_or(0.12);
    let inv_s = cfg.map(|c| c.invincibility_s).unwrap_or(0.35);
    let crit = cfg.map(|c| c.crit_chance).unwrap_or(0.05);
    let energy_max = cfg.map(|c| c.energy_max).unwrap_or(100.0);
    let ranged_cd = cfg.map(|c| c.ranged_base_cooldown_s).unwrap_or(0.45);

    let mut e = commands.spawn(SpriteBundle {
        texture: assets.textures.white.clone(),
        transform: Transform::from_translation(Vec3::new(-170.0, 0.0, 50.0)),
        sprite: Sprite {
            color: Color::srgb(0.25, 0.9, 1.0),
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
        ..default()
    });
    e.insert((CoopPlayer, TeamMarker(Team::Player), InGameEntity, Name::new("CoopPlayer2")));
    e.insert((
        Health {
            current: max_hp,
            max: max_hp,
        },
        Energy {
            current: energy_max,
            max: energy_max,
        },
        Gold(0),
        Velocity::default(),
        MoveSpeed(move_speed),
        AttackPower(attack_power),
        FacingDirection(Vec2::X),
        CritChance(crit),
        RewardModifiers::default(),
        Combo::new(1.8),
        Flash::new(0.0),
        Knockback(Vec2::ZERO),
        Hurtbox {
            team: Team::Player,
            size: Vec2::splat(30.0),
        },
    ));
    e.insert((
        AttackCooldown {
            timer: Timer::from_seconds(attack_cd, TimerMode::Once),
        },
        RangedCooldown {
            timer: Timer::from_seconds(ranged_cd, TimerMode::Once),
        },
        RangedRapidFire {
            ramp: 0,
            decay: Timer::from_seconds(0.65, TimerMode::Once),
        },
        DashCooldown {
            timer: Timer::from_seconds(dash_cd, TimerMode::Once),
        },
        Skill1Cooldown {
            timer: Timer::from_seconds(cfg.map(|c| c.skill1_cooldown_s).unwrap_or(1.1), TimerMode::Once),
        },
        InvincibilityTimer {
            timer: Timer::from_seconds(inv_s, TimerMode::Once),
        },
        DashState::inactive(dash_speed, dash_duration),
    ));
}

pub fn coop_player_energy_regen_system(
    time: Res<Time>,
    data: Option<Res<GameDataRegistry>>,
    config: Res<CoopNetConfig>,
    mut q: Query<&mut Energy, With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    let regen = data
        .as_deref()
        .map(|d| d.player.energy_regen_per_s)
        .unwrap_or(12.0);
    for mut energy in &mut q {
        energy.current = (energy.current + regen * time.delta_seconds()).min(energy.max);
    }
}

pub fn coop_player_heal_channel_system(
    time: Res<Time>,
    data: Option<Res<GameDataRegistry>>,
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    mut q: Query<(&mut Health, &mut Energy), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    if !remote.0.heal_held {
        return;
    }

    let cfg = data.as_deref().map(|d| &d.player);
    let energy_per_s = cfg.map(|c| c.heal_energy_cost_per_s).unwrap_or(20.0).max(0.0);
    let heal_per_s = cfg.map(|c| c.heal_hp_per_s).unwrap_or(18.0).max(0.0);

    let dt = time.delta_seconds();
    for (mut hp, mut energy) in &mut q {
        if hp.current <= 0.0 || hp.current >= hp.max {
            continue;
        }
        if energy.current <= 0.0 {
            continue;
        }
        let need = energy_per_s * dt;
        let ratio = (energy.current / need).clamp(0.0, 1.0);
        let actual_dt = dt * ratio;
        let actual_need = energy_per_s * actual_dt;

        energy.current = (energy.current - actual_need).max(0.0);
        hp.current = (hp.current + heal_per_s * actual_dt).min(hp.max);
    }
}

pub fn coop_player_move_system(
    time: Res<Time>,
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    mut q: Query<(&DashState, &MoveSpeed, &mut Velocity, &mut Transform), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }

    let move_axis = Vec2::new(remote.0.move_axis.0, remote.0.move_axis.1);
    for (dash, move_speed, mut vel, mut tf) in &mut q {
        if dash.active {
            vel.0 = dash.dir * dash.speed;
        } else {
            vel.0 = move_axis * move_speed.0.max(0.0);
        }
        vel.0 = clamp_length(vel.0, dash.speed.max(move_speed.0));
        tf.translation += (vel.0 * time.delta_seconds()).extend(0.0);

        let clamped = clamp_in_room(
            tf.translation.truncate(),
            Vec2::new(ROOM_HALF_WIDTH, ROOM_HALF_HEIGHT),
            28.0,
        );
        tf.translation.x = clamped.x;
        tf.translation.y = clamped.y;
    }
}

pub fn coop_player_facing_system(
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    mut q: Query<(&GlobalTransform, &mut FacingDirection, &Velocity), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }

    let aim_world = if remote.0.aim_valid {
        Some(Vec2::new(remote.0.aim.0, remote.0.aim.1))
    } else {
        None
    };

    for (tf, mut facing, vel) in &mut q {
        if let Some(world) = aim_world {
            let dir = (world - tf.translation().truncate()).try_normalize();
            if let Some(dir) = dir {
                facing.0 = dir;
                continue;
            }
        }
        if vel.0.length_squared() > 1.0 {
            if let Some(dir) = vel.0.try_normalize() {
                facing.0 = dir;
            }
        }
    }
}

pub fn coop_player_invincibility_system(
    time: Res<Time>,
    config: Res<CoopNetConfig>,
    mut q: Query<&mut InvincibilityTimer, With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    for mut inv in &mut q {
        inv.timer.tick(time.delta());
    }
}

pub fn coop_player_attack_input_system(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    data: Option<Res<GameDataRegistry>>,
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    combo_q: Query<&Combo, With<crate::gameplay::player::components::Player>>,
    mut q: Query<(
        Entity,
        &GlobalTransform,
        &FacingDirection,
        &AttackPower,
        &mut AttackCooldown,
        &CritChance,
        &RewardModifiers,
    ), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    let combo_mult = combo_q
        .get_single()
        .ok()
        .map(|c| 1.0 + (c.count.min(20) as f32) * 0.02)
        .unwrap_or(1.0);

    for (player_e, player_tf, facing, power, mut cd, crit, mods) in &mut q {
        cd.timer.tick(time.delta());
        if !remote.0.attack_held || !cd.timer.finished() {
            continue;
        }

        cd.timer.reset();
        let mut cd_s = cd.timer.duration().as_secs_f32();
        if mods.attack_speed_mult > 0.0 {
            cd_s *= 1.0 / (1.0 + mods.attack_speed_mult);
            cd.timer.set_duration(std::time::Duration::from_secs_f32(cd_s.max(0.08)));
        }

        spawn_player_melee_hitbox(
            &mut commands,
            &assets,
            player_e,
            player_tf,
            facing.0,
            power.0 * combo_mult,
            crit.0,
        );

        if mods.bonus_projectile {
            let proj_speed = data.as_deref().map(|d| d.player.move_speed).unwrap_or(260.0) * 2.0;
            projectiles::spawn_projectile(
                &mut commands,
                &assets,
                Team::Player,
                player_tf.translation().truncate() + facing.0 * 18.0,
                facing.0 * proj_speed,
                power.0 * 0.65,
            );
        }

        particles::spawn_hit_particles(
            &mut commands,
            &assets,
            player_tf.translation().truncate() + facing.0 * 20.0,
            Color::srgba(0.6, 1.0, 0.95, 0.9),
        );
    }
}

pub fn coop_player_ranged_input_system(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    data: Option<Res<GameDataRegistry>>,
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    combo_q: Query<&Combo, With<crate::gameplay::player::components::Player>>,
    mut q: Query<(
        &GlobalTransform,
        &FacingDirection,
        &AttackPower,
        &mut RangedCooldown,
        &mut Energy,
        &mut RangedRapidFire,
    ), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }

    let combo_mult = combo_q
        .get_single()
        .ok()
        .map(|c| 1.0 + (c.count.min(20) as f32) * 0.015)
        .unwrap_or(1.0);

    for (tf, facing, power, mut cd, mut energy, mut rapid) in &mut q {
        cd.timer.tick(time.delta());

        if remote.0.ranged_held {
            rapid.decay.reset();
        } else {
            rapid.decay.tick(time.delta());
            if rapid.decay.finished() {
                rapid.ramp = 0;
            }
        }

        if !remote.0.ranged_held || !cd.timer.finished() {
            continue;
        }

        let cost = data
            .as_deref()
            .map(|d| d.player.ranged_energy_cost)
            .unwrap_or(12.0);
        if energy.current < cost {
            continue;
        }
        energy.current = (energy.current - cost).max(0.0);

        let cfg = data.as_deref().map(|d| &d.player);
        let base_cd = cfg.map(|c| c.ranged_base_cooldown_s).unwrap_or(0.45);
        let min_cd = cfg.map(|c| c.ranged_min_cooldown_s).unwrap_or(0.18);
        let max_ramp = cfg.map(|c| c.ranged_ramp_max).unwrap_or(8).max(1);
        rapid.ramp = (rapid.ramp + 1).min(max_ramp);
        let ramp_t = (rapid.ramp.saturating_sub(1) as f32) / (max_ramp.saturating_sub(1) as f32).max(1.0);
        let cd_s = (base_cd + (min_cd - base_cd) * ramp_t).max(min_cd);
        cd.timer.set_duration(std::time::Duration::from_secs_f32(cd_s));
        cd.timer.reset();

        let dir = facing.0;
        let speed = 720.0;
        projectiles::spawn_projectile(
            &mut commands,
            &assets,
            Team::Player,
            tf.translation().truncate() + dir * 18.0,
            dir * speed,
            power.0 * 0.6 * combo_mult,
        );
        particles::spawn_hit_particles(
            &mut commands,
            &assets,
            tf.translation().truncate() + dir * 20.0,
            Color::srgba(0.4, 0.95, 1.0, 0.9),
        );
    }
}

pub fn coop_player_dash_input_system(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    data: Option<Res<GameDataRegistry>>,
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    mut q: Query<(
        &GlobalTransform,
        &mut DashCooldown,
        &mut DashState,
        &FacingDirection,
        &mut InvincibilityTimer,
        &mut Energy,
        &Sprite,
    ), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }

    let move_axis = Vec2::new(remote.0.move_axis.0, remote.0.move_axis.1);

    for (tf, mut cd, mut dash, facing, mut inv, mut energy, sprite) in &mut q {
        cd.timer.tick(time.delta());
        if dash.active || !remote.0.dash_pressed || !cd.timer.finished() {
            continue;
        }

        let cost = data
            .as_deref()
            .map(|d| d.player.dash_energy_cost)
            .unwrap_or(25.0);
        if energy.current < cost {
            continue;
        }
        energy.current = (energy.current - cost).max(0.0);

        cd.timer.reset();
        dash.active = true;
        dash.timer.reset();
        dash.dir = if move_axis.length_squared() > 0.0 {
            move_axis.normalize()
        } else {
            facing.0
        };

        inv.timer.reset();
        particles::spawn_dash_particles(&mut commands, &assets, tf.translation().truncate());

        crate::gameplay::effects::afterimage::spawn_afterimage(
            &mut commands,
            &assets,
            tf.translation().truncate(),
            sprite.color.with_alpha(0.45),
            sprite.custom_size.unwrap_or(Vec2::splat(32.0)),
        );
    }
}

pub fn coop_update_dash_state(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    config: Res<CoopNetConfig>,
    mut q: Query<(&GlobalTransform, &mut DashState, &Sprite), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    for (tf, mut dash, sprite) in &mut q {
        if !dash.active {
            continue;
        }

        dash.timer.tick(time.delta());
        if dash.timer.just_finished() {
            dash.active = false;
            continue;
        }

        crate::gameplay::effects::afterimage::spawn_afterimage(
            &mut commands,
            &assets,
            tf.translation().truncate(),
            sprite.color.with_alpha(0.25),
            sprite.custom_size.unwrap_or(Vec2::splat(32.0)),
        );
    }
}

pub fn coop_player_skill1_input_system(
    mut commands: Commands,
    time: Res<Time>,
    data: Option<Res<GameDataRegistry>>,
    assets: Res<GameAssets>,
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    mut q: Query<(&GlobalTransform, &AttackPower, &mut Energy, &mut Skill1Cooldown), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    for (tf, power, mut energy, mut cd) in &mut q {
        cd.timer.tick(time.delta());
        if !remote.0.skill1_pressed {
            continue;
        }
        if !cd.timer.finished() {
            continue;
        }

        let cfg = data.as_deref().map(|d| &d.player);
        let cost = cfg.map(|c| c.skill1_energy_cost).unwrap_or(45.0);
        if energy.current < cost {
            continue;
        }
        energy.current = (energy.current - cost).max(0.0);
        cd.timer.reset();

        let pos = tf.translation().truncate();
        let speed = 820.0;
        let damage = power.0 * 1.35;

        for i in 0..8 {
            let a = i as f32 / 8.0 * std::f32::consts::TAU;
            let dir = Vec2::new(a.cos(), a.sin());
            projectiles::spawn_projectile(
                &mut commands,
                &assets,
                Team::Player,
                pos + dir * 18.0,
                dir * speed,
                damage,
            );
        }
    }
}

pub fn coop_remote_shop_system(
    config: Res<CoopNetConfig>,
    remote: Res<CoopRemoteInput>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    mut offers: ResMut<ShopOffers>,
    data: Option<Res<GameDataRegistry>>,
    mut rng: ResMut<GameRng>,
    mut coop_player_q: Query<(
        &mut Gold,
        &mut Health,
        &mut Energy,
        &mut MoveSpeed,
        &mut AttackPower,
        &mut CritChance,
        &mut DashCooldown,
        &mut AttackCooldown,
        &mut RewardModifiers,
    ), With<CoopPlayer>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    let (Some(layout), Some(current)) = (layout, current) else { return };
    let Some(room) = layout.room(current.0) else { return };
    if room.room_type != RoomType::Shop {
        return;
    }

    if remote.0.interact_pressed {
        shop::ensure_shop_offers_for_room(&mut offers, data.as_deref(), &mut rng, current.0);
    }

    let Some(index) = remote.0.shop_purchase_index.map(|i| i as usize) else { return };
    shop::ensure_shop_offers_for_room(&mut offers, data.as_deref(), &mut rng, current.0);
    let Some(line) = offers.lines.get(index) else { return };

    let Ok((mut gold, mut hp, mut energy, mut move_speed, mut power, mut crit, mut dash_cd, mut atk_cd, mut mods)) =
        coop_player_q.get_single_mut()
    else {
        return;
    };
    if gold.0 < line.cost {
        return;
    }
    gold.0 -= line.cost;
    shop::apply_item(
        line.item,
        &mut hp,
        &mut energy,
        &mut move_speed,
        &mut power,
        &mut crit,
        &mut dash_cd,
        &mut atk_cd,
        &mut mods,
    );
}

pub fn coop_player_death_system(
    mut death_events: EventReader<DeathEvent>,
    config: Res<CoopNetConfig>,
    coop_q: Query<(), With<CoopPlayer>>,
    mut next_state: ResMut<NextState<crate::states::AppState>>,
) {
    if config.mode != NetMode::Host {
        return;
    }
    if coop_q.iter().next().is_none() {
        return;
    }
    for ev in death_events.read() {
        // If any player dies in coop, end the run (simple rule for MVP).
        if ev.team == Team::Player {
            next_state.set(crate::states::AppState::GameOver);
            return;
        }
    }
}

pub fn update_coop_player_head_text(
    mut commands: Commands,
    assets: Res<GameAssets>,
    coop_q: Query<(&Transform, &Health), With<CoopPlayer>>,
    mut text_q: Query<(Entity, &mut Transform, &mut Text), With<CoopMateHeadText>>,
) {
    let Ok((player_tf, hp)) = coop_q.get_single() else {
        for (entity, _, _) in &mut text_q {
            commands.entity(entity).despawn_recursive();
        }
        return;
    };

    if let Ok((_, mut tf, mut text)) = text_q.get_single_mut() {
        tf.translation = Vec3::new(player_tf.translation.x, player_tf.translation.y + 34.0, 90.0);
        text.sections[0].value = format!("{:.0}/{:.0}", hp.current, hp.max);
        return;
    }

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("{:.0}/{:.0}", hp.current, hp.max),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(
                player_tf.translation.x,
                player_tf.translation.y + 34.0,
                90.0,
            )),
            ..default()
        },
        CoopMateHeadText,
        InGameEntity,
        Name::new("CoopMateHeadText"),
    ));
}
