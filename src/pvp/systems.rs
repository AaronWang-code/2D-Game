use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH, UI_Z};
use crate::core::assets::GameAssets;
use crate::core::events::DamageAppliedEvent;
use crate::core::input::PlayerInputState;
use crate::gameplay::combat::components::Team;
use crate::gameplay::player::components::{Health, Velocity};
use crate::states::AppState;
use crate::ui::widgets;

use super::components::*;
use super::net::{NetMode, PvpFireMsg, PvpInputMsg, PvpNetConfig, PvpNetState, PvpPlayerStateMsg, PvpStateMsg};

#[derive(Resource, Debug, Default)]
pub struct PvpMatchState {
    pub tick: u32,
    pub state_send_timer: Timer,
}

impl PvpMatchState {
    fn ensure_init(&mut self) {
        if self.state_send_timer.duration().as_secs_f32() <= 0.0 {
            self.state_send_timer = Timer::from_seconds(1.0 / 20.0, TimerMode::Repeating);
        }
    }
}

#[derive(Component)]
pub struct PvpHudUi;

#[derive(Component)]
pub struct PvpHudText;

pub fn setup_pvp_game(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut match_state: ResMut<PvpMatchState>,
    mut net: ResMut<PvpNetState>,
) {
    match_state.tick = 0;
    match_state.state_send_timer = Timer::from_seconds(1.0 / 20.0, TimerMode::Repeating);
    net.clear_runtime();

    // Arena backdrop.
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite {
                color: Color::srgb(0.06, 0.07, 0.10),
                custom_size: Some(Vec2::new(ROOM_HALF_WIDTH * 2.0, ROOM_HALF_HEIGHT * 2.0)),
                ..default()
            },
            ..default()
        },
        PvpEntity,
        Name::new("PvpArena"),
    ));

    // HUD.
    commands
        .spawn((widgets::root_node(), PvpHudUi, PvpEntity, Name::new("PvpHudRoot")))
        .with_children(|root| {
            root.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    top: Val::Px(12.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|col| {
                col.spawn((widgets::title_text(&assets, "PVP", 18.0), PvpHudText));
            });
        });

    spawn_players(&mut commands, &assets, net.my_id);
}

fn spawn_players(commands: &mut Commands, assets: &GameAssets, my_id: Option<u8>) {
    let p1 = spawn_one_player(commands, assets, 1, Vec2::new(-ROOM_HALF_WIDTH * 0.55, 0.0), Color::srgb(0.25, 0.9, 1.0));
    let p2 = spawn_one_player(commands, assets, 2, Vec2::new(ROOM_HALF_WIDTH * 0.55, 0.0), Color::srgb(1.0, 0.45, 0.30));

    if my_id == Some(1) {
        commands.entity(p1).insert(PvpLocalPlayer);
        commands.entity(p2).insert(PvpRemotePlayer);
    } else if my_id == Some(2) {
        commands.entity(p2).insert(PvpLocalPlayer);
        commands.entity(p1).insert(PvpRemotePlayer);
    }
}

fn spawn_one_player(commands: &mut Commands, assets: &GameAssets, id: u8, pos: Vec2, color: Color) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos.extend(50.0)),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(34.0)),
                    ..default()
                },
                ..default()
            },
            PvpEntity,
            PvpPlayerId(id),
            Velocity::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
            PvpLives::default(),
            PvpCooldowns::new(),
            Name::new(format!("PvpPlayer{id}")),
        ))
        .id()
}

pub fn cleanup_pvp_world(mut commands: Commands, q: Query<Entity, With<PvpEntity>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn pvp_send_local_input_system(input: Res<PlayerInputState>, config: Res<PvpNetConfig>, net: Res<PvpNetState>) {
    if config.mode != NetMode::Client || !net.connected {
        return;
    }

    let aim = input.aim_world.unwrap_or(Vec2::ZERO);
    let msg = PvpInputMsg {
        move_axis: (input.move_axis.x, input.move_axis.y),
        melee: input.attack_pressed,
        ranged: input.ranged_pressed,
        aim: (aim.x, aim.y),
    };
    net.send_input(msg);
}

pub fn pvp_host_simulation_system(
    time: Res<Time>,
    input: Res<PlayerInputState>,
    mut match_state: ResMut<PvpMatchState>,
    config: Res<PvpNetConfig>,
    mut net: ResMut<PvpNetState>,
    mut next: ResMut<NextState<AppState>>,
    mut damage_events: EventWriter<DamageAppliedEvent>,
    mut players: Query<(&PvpPlayerId, &mut Transform, &mut Velocity, &mut Health, &mut PvpLives, &mut PvpCooldowns)>,
) {
    if config.mode != NetMode::Host || !net.connected {
        return;
    }
    match_state.ensure_init();
    match_state.tick = match_state.tick.wrapping_add(1);

    let client_input = net.take_client_input().unwrap_or_default();

    // Host input (player 1).
    let host_aim = input.aim_world.unwrap_or(Vec2::ZERO);
    let host_input = PvpInputMsg {
        move_axis: (input.move_axis.x, input.move_axis.y),
        melee: input.attack_pressed,
        ranged: input.ranged_pressed,
        aim: (host_aim.x, host_aim.y),
    };

    // Tick cooldowns & simulate movement.
    for (id, mut tf, mut vel, _hp, _lives, mut cds) in &mut players {
        cds.melee.tick(time.delta());
        cds.ranged.tick(time.delta());
        cds.respawn.tick(time.delta());

        if !cds.respawn.finished() {
            vel.0 = Vec2::ZERO;
            continue;
        }

        let axis = if id.0 == 1 {
            Vec2::new(host_input.move_axis.0, host_input.move_axis.1)
        } else {
            Vec2::new(client_input.move_axis.0, client_input.move_axis.1)
        };
        let speed = 310.0;
        vel.0 = axis * speed;
        tf.translation += (vel.0 * time.delta_seconds()).extend(0.0);
        clamp_to_arena(&mut tf);
    }

    // Resolve attacks.
    resolve_attacks(&mut players, host_input, client_input, &mut net, &mut next, &mut damage_events);

    // Send snapshot at 20hz.
    match_state.state_send_timer.tick(time.delta());
    if match_state.state_send_timer.just_finished() {
        if let Some(st) = build_state_msg(match_state.tick, &mut players) {
            net.send_state(&st);
        }
    }
}

fn resolve_attacks(
    players: &mut Query<(&PvpPlayerId, &mut Transform, &mut Velocity, &mut Health, &mut PvpLives, &mut PvpCooldowns)>,
    host_input: PvpInputMsg,
    client_input: PvpInputMsg,
    net: &mut PvpNetState,
    next: &mut NextState<AppState>,
    damage_events: &mut EventWriter<DamageAppliedEvent>,
) {
    // Gather data.
    let mut p1 = None;
    let mut p2 = None;
    for (id, tf, _vel, hp, lives, cds) in players.iter_mut() {
        if id.0 == 1 {
            p1 = Some((tf.translation.truncate(), hp.current, lives.lives, cds.respawn.finished()));
        } else if id.0 == 2 {
            p2 = Some((tf.translation.truncate(), hp.current, lives.lives, cds.respawn.finished()));
        }
    }
    let (Some((p1_pos, _p1_hp, _p1_l, p1_alive)), Some((p2_pos, _p2_hp, _p2_l, p2_alive))) = (p1, p2) else { return };

    // Melee: short range, higher damage.
    if host_input.melee && p1_alive {
        try_melee(1, 2, p1_pos, p2_pos, players, damage_events);
    }
    if client_input.melee && p2_alive {
        try_melee(2, 1, p2_pos, p1_pos, players, damage_events);
    }

    // Ranged: hitscan + bullet visual.
    if host_input.ranged && p1_alive {
        try_ranged(1, 2, p1_pos, Vec2::new(host_input.aim.0, host_input.aim.1), p2_pos, players, net, damage_events);
    }
    if client_input.ranged && p2_alive {
        try_ranged(2, 1, p2_pos, Vec2::new(client_input.aim.0, client_input.aim.1), p1_pos, players, net, damage_events);
    }

    // Death / respawn / win.
    let mut p1_lives = None;
    let mut p2_lives = None;
    let mut p1_hp = None;
    let mut p2_hp = None;
    for (id, _tf, _vel, hp, lives, _cds) in players.iter_mut() {
        if id.0 == 1 {
            p1_lives = Some(lives.lives);
            p1_hp = Some(hp.current);
        } else if id.0 == 2 {
            p2_lives = Some(lives.lives);
            p2_hp = Some(hp.current);
        }
    }
    let (Some(p1_l), Some(p2_l), Some(p1_h), Some(p2_h)) = (p1_lives, p2_lives, p1_hp, p2_hp) else { return };
    if p1_h <= 0.0 {
        handle_death(1, players);
    }
    if p2_h <= 0.0 {
        handle_death(2, players);
    }

    // Winner check after deaths applied.
    let mut p1_left = 0;
    let mut p2_left = 0;
    for (id, _tf, _vel, _hp, lives, _cds) in players.iter_mut() {
        if id.0 == 1 {
            p1_left = lives.lives;
        } else if id.0 == 2 {
            p2_left = lives.lives;
        }
    }
    if p1_left == 0 || p2_left == 0 {
        let winner = if p1_left > 0 { 1 } else { 2 };
        net.send_result(winner);
        next.set(AppState::PvpResult);
    }
}

fn try_melee(
    attacker: u8,
    target: u8,
    attacker_pos: Vec2,
    target_pos: Vec2,
    players: &mut Query<(&PvpPlayerId, &mut Transform, &mut Velocity, &mut Health, &mut PvpLives, &mut PvpCooldowns)>,
    damage_events: &mut EventWriter<DamageAppliedEvent>,
) {
    let range = 86.0;
    if attacker_pos.distance(target_pos) > range {
        return;
    }

    let mut can = false;
    for (id, _tf, _vel, _hp, _lives, cds) in players.iter_mut() {
        if id.0 == attacker && cds.melee.finished() && cds.respawn.finished() {
            can = true;
            break;
        }
    }
    if !can {
        return;
    }

    for (id, _tf, _vel, _hp, _lives, mut cds) in players.iter_mut() {
        if id.0 == attacker {
            cds.melee.reset();
        }
    }

    let dir = (target_pos - attacker_pos).try_normalize().unwrap_or(Vec2::X);
    for (id, _tf, mut vel, mut hp, _lives, _cds) in players.iter_mut() {
        if id.0 == target {
            hp.current = (hp.current - 18.0).max(0.0);
            vel.0 += dir * 420.0;
            damage_events.send(DamageAppliedEvent {
                target: Entity::PLACEHOLDER,
                amount: 18.0,
                attacker_team: pvp_team(attacker),
                target_team: Some(pvp_team(target)),
                is_crit: false,
                pos: target_pos,
            });
        }
    }
}

fn try_ranged(
    attacker: u8,
    target: u8,
    attacker_pos: Vec2,
    aim_world: Vec2,
    target_pos: Vec2,
    players: &mut Query<(&PvpPlayerId, &mut Transform, &mut Velocity, &mut Health, &mut PvpLives, &mut PvpCooldowns)>,
    net: &mut PvpNetState,
    damage_events: &mut EventWriter<DamageAppliedEvent>,
) {
    let mut can = false;
    for (id, _tf, _vel, _hp, _lives, cds) in players.iter_mut() {
        if id.0 == attacker && cds.ranged.finished() && cds.respawn.finished() {
            can = true;
            break;
        }
    }
    if !can {
        return;
    }
    for (id, _tf, _vel, _hp, _lives, mut cds) in players.iter_mut() {
        if id.0 == attacker {
            cds.ranged.reset();
        }
    }

    let dir = (aim_world - attacker_pos).try_normalize().unwrap_or(Vec2::X);
    let fire = PvpFireMsg {
        shooter_id: attacker,
        origin: (attacker_pos.x, attacker_pos.y),
        dir: (dir.x, dir.y),
    };
    net.fire_events.push(fire);
    net.send_fire(fire);

    // Hitscan: if target is close to ray.
    let max_range = 560.0;
    let to_target = target_pos - attacker_pos;
    if to_target.length() > max_range {
        return;
    }
    let proj = to_target.dot(dir);
    if proj < 0.0 {
        return;
    }
    let closest = attacker_pos + dir * proj;
    let dist = closest.distance(target_pos);
    if dist > 22.0 {
        return;
    }

    for (id, _tf, mut vel, mut hp, _lives, _cds) in players.iter_mut() {
        if id.0 == target {
            hp.current = (hp.current - 10.0).max(0.0);
            vel.0 += dir * 280.0;
            damage_events.send(DamageAppliedEvent {
                target: Entity::PLACEHOLDER,
                amount: 10.0,
                attacker_team: pvp_team(attacker),
                target_team: Some(pvp_team(target)),
                is_crit: false,
                pos: target_pos,
            });
        }
    }
}

fn handle_death(
    who: u8,
    players: &mut Query<(&PvpPlayerId, &mut Transform, &mut Velocity, &mut Health, &mut PvpLives, &mut PvpCooldowns)>,
) {
    let mut lives_left = 0;
    for (id, _tf, _vel, _hp, lives, _cds) in players.iter_mut() {
        if id.0 == who {
            lives_left = lives.lives;
        }
    }
    if lives_left == 0 {
        return;
    }

    for (id, mut tf, mut vel, mut hp, mut lives, mut cds) in players.iter_mut() {
        if id.0 != who {
            continue;
        }
        lives.lives = lives.lives.saturating_sub(1);
        vel.0 = Vec2::ZERO;
        hp.current = hp.max;
        cds.respawn = Timer::from_seconds(1.1, TimerMode::Once);
        cds.respawn.reset();

        let respawn_pos = if who == 1 {
            Vec2::new(-ROOM_HALF_WIDTH * 0.55, 0.0)
        } else {
            Vec2::new(ROOM_HALF_WIDTH * 0.55, 0.0)
        };
        tf.translation = respawn_pos.extend(tf.translation.z);
        clamp_to_arena(&mut tf);
    }
}

fn clamp_to_arena(tf: &mut Transform) {
    let half = Vec2::new(ROOM_HALF_WIDTH - 26.0, ROOM_HALF_HEIGHT - 26.0);
    tf.translation.x = tf.translation.x.clamp(-half.x, half.x);
    tf.translation.y = tf.translation.y.clamp(-half.y, half.y);
}

fn build_state_msg(
    tick: u32,
    players: &mut Query<(&PvpPlayerId, &mut Transform, &mut Velocity, &mut Health, &mut PvpLives, &mut PvpCooldowns)>,
) -> Option<PvpStateMsg> {
    let mut p1 = None;
    let mut p2 = None;
    for (id, tf, _vel, hp, lives, _cds) in players.iter_mut() {
        let msg = PvpPlayerStateMsg {
            id: id.0,
            pos: (tf.translation.x, tf.translation.y),
            hp: hp.current,
            lives: lives.lives,
        };
        if id.0 == 1 {
            p1 = Some(msg);
        } else if id.0 == 2 {
            p2 = Some(msg);
        }
    }
    Some(PvpStateMsg {
        tick,
        p1: p1?,
        p2: p2?,
    })
}

pub fn pvp_client_apply_state_system(
    config: Res<PvpNetConfig>,
    mut net: ResMut<PvpNetState>,
    mut next: ResMut<NextState<AppState>>,
    mut damage_events: EventWriter<DamageAppliedEvent>,
    mut players: Query<(&PvpPlayerId, &mut Transform, &mut Health, &mut PvpLives, &mut Velocity, &mut PvpCooldowns)>,
) {
    if config.mode != NetMode::Client || !net.connected {
        return;
    }
    let Some(st) = net.last_state.take() else { return };

    for (id, mut tf, mut hp, mut lives, mut vel, mut cds) in &mut players {
        let src = if id.0 == 1 { st.p1 } else { st.p2 };
        let old_hp = hp.current;
        tf.translation.x = src.pos.0;
        tf.translation.y = src.pos.1;
        vel.0 = Vec2::ZERO;
        hp.current = src.hp;
        lives.lives = src.lives;
        cds.respawn = Timer::from_seconds(0.0, TimerMode::Once);
        let loss = (old_hp - src.hp).max(0.0);
        if loss > 0.0 {
            damage_events.send(DamageAppliedEvent {
                target: Entity::PLACEHOLDER,
                amount: loss,
                attacker_team: if id.0 == 1 { Team::Pvp2 } else { Team::Pvp1 },
                target_team: Some(pvp_team(id.0)),
                is_crit: false,
                pos: tf.translation.truncate(),
            });
        }
    }

    if let Some(w) = net.winner {
        let _ = w;
        next.set(AppState::PvpResult);
    }
}

pub fn pvp_bullet_visual_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut net: ResMut<PvpNetState>,
) {
    if net.fire_events.is_empty() {
        return;
    }
    let events = std::mem::take(&mut net.fire_events);
    for ev in events {
        let origin = Vec2::new(ev.origin.0, ev.origin.1);
        let dir = Vec2::new(ev.dir.0, ev.dir.1).try_normalize().unwrap_or(Vec2::X);
        spawn_bullet_visual(&mut commands, &assets, origin, dir);
    }
}

fn spawn_bullet_visual(commands: &mut Commands, assets: &GameAssets, origin: Vec2, dir: Vec2) {
    let speed = 860.0;
    let max_range = 560.0;
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation((origin + dir * 22.0).extend(UI_Z - 20.0)),
            sprite: Sprite {
                color: Color::srgba(0.92, 0.92, 1.0, 0.85),
                custom_size: Some(Vec2::new(10.0, 4.0)),
                ..default()
            },
            ..default()
        },
        PvpEntity,
        PvpBullet {
            velocity: dir * speed,
            remaining_distance: max_range,
        },
        crate::gameplay::combat::components::Lifetime(Timer::from_seconds(max_range / speed, TimerMode::Once)),
        Name::new("PvpBullet"),
    ));
}

pub fn pvp_bullet_visual_system_move_and_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut PvpBullet, &mut Transform, &mut crate::gameplay::combat::components::Lifetime)>,
) {
    for (e, mut bullet, mut tf, mut life) in &mut q {
        let step = bullet.velocity * time.delta_seconds();
        tf.translation += step.extend(0.0);
        bullet.remaining_distance -= step.length();
        life.0.tick(time.delta());
        if life.0.finished() || bullet.remaining_distance <= 0.0 {
            commands.entity(e).despawn_recursive();
        }
    }
}

pub fn pvp_update_hud_system(
    net: Res<PvpNetState>,
    players: Query<(&PvpPlayerId, &Health, &PvpLives)>,
    mut text_q: Query<&mut Text, With<PvpHudText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let mut p1 = None;
    let mut p2 = None;
    for (id, hp, lives) in &players {
        if id.0 == 1 {
            p1 = Some((hp.current, lives.lives));
        } else if id.0 == 2 {
            p2 = Some((hp.current, lives.lives));
        }
    }
    let (p1, p2) = match (p1, p2) {
        (Some(a), Some(b)) => (a, b),
        _ => return,
    };
    let me = net.my_id.unwrap_or(0);
    text.sections[0].value = format!(
        "PVP（你是P{me}）  P1: HP {:.0} / Lives {}    P2: HP {:.0} / Lives {}",
        p1.0, p1.1, p2.0, p2.1
    );
}
pub fn pvp_update_hud_system_v2(
    net: Res<PvpNetState>,
    players: Query<(&PvpPlayerId, &Health, &PvpLives)>,
    mut text_q: Query<&mut Text, With<PvpHudText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let mut p1 = None;
    let mut p2 = None;
    for (id, hp, lives) in &players {
        if id.0 == 1 {
            p1 = Some((hp.current, lives.lives));
        } else if id.0 == 2 {
            p2 = Some((hp.current, lives.lives));
        }
    }
    let (p1, p2) = match (p1, p2) {
        (Some(a), Some(b)) => (a, b),
        _ => return,
    };
    let me = net.my_id.unwrap_or(0);
    text.sections[0].value = format!(
        "PVP | 你是 P{me} | P1 HP {:.0} LIFE {} | P2 HP {:.0} LIFE {}",
        p1.0, p1.1, p2.0, p2.1
    );
}

fn pvp_team(id: u8) -> Team {
    if id == 1 {
        Team::Pvp1
    } else {
        Team::Pvp2
    }
}
