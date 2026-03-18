use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::states::AppState;
use crate::ui::widgets;

use super::components::CoopClientLocalPlayer;
use super::net::{
    build_client_input, start_client_socket, start_host_socket, CoopMsg, CoopNetConfig, CoopNetState, NetMode, COOP_PORT,
};

#[derive(Component)]
pub struct CoopMenuUi;

#[derive(Component)]
pub struct CoopLobbyUi;

#[derive(Component)]
pub struct CoopClientUi;

#[derive(Component)]
pub struct CoopLobbyText;

#[derive(Component)]
pub struct CoopIpText;

#[derive(Component)]
pub struct CoopHudText;

#[derive(Component)]
pub struct CoopClientEntity;

#[derive(Resource, Debug, Default, Clone)]
pub struct CoopJoinIp {
    pub ip: String,
}

pub fn setup_coop_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands.init_resource::<CoopJoinIp>();
    commands
        .spawn((widgets::root_node(), CoopMenuUi, Name::new("CoopMenuRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "玩家合作（局域网）", 48.0));
                    panel.spawn(widgets::title_text(
                        &assets,
                        "H=当房主（进入单人模式并等待连接）  J=输入房主IP并加入  Esc=返回",
                        18.0,
                    ));
                    panel.spawn((widgets::title_text(&assets, "房主IP：", 18.0), CoopIpText));
                });
        });
}

pub fn coop_menu_input_system(
    mut chars: EventReader<ReceivedCharacter>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ip: ResMut<CoopJoinIp>,
    mut ip_text_q: Query<&mut Text, With<CoopIpText>>,
    mut config: ResMut<CoopNetConfig>,
    mut net: ResMut<CoopNetState>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Ok(mut ip_text) = ip_text_q.get_single_mut() else { return };

    if keyboard.just_pressed(KeyCode::Escape) {
        config.mode = NetMode::None;
        net.socket = None;
        net.peer = None;
        net.connected = false;
        next.set(AppState::MultiplayerMenu);
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyH) {
        config.mode = NetMode::Host;
        let _ = start_host_socket(&mut net);
        // Host plays in normal single-player state.
        next.set(AppState::InGame);
        return;
    }

    for ev in chars.read() {
        for c in ev.char.chars() {
            if c.is_ascii_digit() || c == '.' || c == ':' {
                if ip.ip.len() < 64 {
                    ip.ip.push(c);
                }
            }
        }
    }
    if keyboard.just_pressed(KeyCode::Backspace) {
        ip.ip.pop();
    }

    if keyboard.just_pressed(KeyCode::KeyJ) || keyboard.just_pressed(KeyCode::Enter) {
        let host = ip.ip.trim();
        if !host.is_empty() {
            config.mode = NetMode::Client;
            config.host_ip = host.to_string();
            let _ = start_client_socket(&mut net);
            if let Ok(addr) = format!("{host}:{COOP_PORT}").parse() {
                net.peer = Some(addr);
            }
            next.set(AppState::CoopLobby);
        }
    }

    ip_text.sections[0].value = format!("房主IP：{}", ip.ip);
}

pub fn cleanup_coop_menu(mut commands: Commands, q: Query<Entity, With<CoopMenuUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_coop_lobby(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((widgets::root_node(), CoopLobbyUi, Name::new("CoopLobbyRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "合作模式：连接中", 46.0));
                    panel.spawn((widgets::title_text(&assets, "连接中...", 18.0), CoopLobbyText));
                    panel.spawn(widgets::title_text(&assets, "Esc=取消并返回", 18.0));
                });
        });
}

pub fn coop_lobby_ui_system(config: Res<CoopNetConfig>, net: Res<CoopNetState>, mut q: Query<&mut Text, With<CoopLobbyText>>) {
    let Ok(mut text) = q.get_single_mut() else { return };
    let status = match config.mode {
        NetMode::Client => {
            let host = config.host_ip.clone();
            if net.connected {
                format!("已连接到房主：{host}:{COOP_PORT}")
            } else {
                format!("正在连接：{host}:{COOP_PORT}（请确认房主已按 H 并进入游戏）")
            }
        }
        NetMode::Host => "房主模式无需在此等待".to_string(),
        NetMode::None => "未选择模式".to_string(),
    };
    text.sections[0].value = status;
}

pub fn coop_lobby_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<CoopNetConfig>,
    mut net: ResMut<CoopNetState>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    config.mode = NetMode::None;
    net.socket = None;
    net.peer = None;
    net.connected = false;
    net.my_id = None;
    next.set(AppState::MultiplayerMenu);
}

pub fn cleanup_coop_lobby(mut commands: Commands, q: Query<Entity, With<CoopLobbyUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_coop_client_game(mut commands: Commands, assets: Res<GameAssets>) {
    // Background and HUD only (client doesn't run full gameplay).
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite {
                color: Color::srgb(0.06, 0.07, 0.10),
                custom_size: Some(Vec2::new(1600.0, 900.0)),
                ..default()
            },
            ..default()
        },
        CoopClientUi,
        Name::new("CoopClientBackground"),
    ));

    commands
        .spawn((widgets::root_node(), CoopClientUi, Name::new("CoopClientHudRoot")))
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
                col.spawn((widgets::title_text(&assets, "合作模式", 18.0), CoopHudText));
                col.spawn(widgets::body_text(&assets, "Esc=断开并返回", 16.0));
            });
        });
}

pub fn coop_client_send_input_system(input: Res<PlayerInputState>, config: Res<CoopNetConfig>, net: Res<CoopNetState>) {
    if config.mode != NetMode::Client || !net.connected {
        return;
    }
    let msg = CoopMsg::Input(build_client_input(&input));
    super::net::coop_send_raw(&net, &msg);
}

pub fn coop_client_apply_snapshot_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    config: Res<CoopNetConfig>,
    mut net: ResMut<CoopNetState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    existing: Query<Entity, With<CoopClientEntity>>,
    mut rendered_tick: Local<u32>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        net.socket = None;
        net.peer = None;
        net.connected = false;
        net.my_id = None;
        next.set(AppState::MultiplayerMenu);
        return;
    }
    if config.mode != NetMode::Client {
        return;
    }
    let Some(snapshot) = net.last_snapshot.clone() else { return };
    if *rendered_tick == snapshot.tick {
        return;
    }
    *rendered_tick = snapshot.tick;

    // Simple approach: despawn all replicated entities then respawn.
    for e in &existing {
        commands.entity(e).despawn_recursive();
    }

    // Spawn players.
    let my_id = net.my_id.unwrap_or(2);
    spawn_player_visual(&mut commands, &assets, snapshot.p1.pos, Color::srgb(0.35, 0.9, 0.45), "CoopP1", my_id == 1);
    spawn_player_visual(&mut commands, &assets, snapshot.p2.pos, Color::srgb(0.25, 0.9, 1.0), "CoopP2", my_id == 2);

    for e in snapshot.enemies {
        let color = match e.kind {
            1 => Color::srgb(1.0, 0.35, 0.25),
            2 => Color::srgb(1.0, 0.55, 0.25),
            3 => Color::srgb(0.95, 0.25, 0.85),
            4 => Color::srgb(0.85, 0.20, 0.90),
            _ => Color::srgb(1.0, 0.35, 0.25),
        };
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(Vec3::new(e.pos.0, e.pos.1, 20.0)),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(30.0)),
                    ..default()
                },
                ..default()
            },
            CoopClientEntity,
            Name::new("CoopEnemy"),
        ));
    }

    for p in snapshot.projectiles {
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(Vec3::new(p.pos.0, p.pos.1, 20.0)),
                sprite: Sprite {
                    color: Color::srgba(0.8, 0.9, 1.0, 0.9),
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                ..default()
            },
            CoopClientEntity,
            Name::new("CoopProjectile"),
        ));
    }
}

fn spawn_player_visual(
    commands: &mut Commands,
    assets: &GameAssets,
    pos: (f32, f32),
    color: Color,
    name: &str,
    is_local: bool,
) {
    let mut e = commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(Vec3::new(pos.0, pos.1, 50.0)),
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            },
            ..default()
        },
        CoopClientEntity,
        Name::new(name.to_string()),
    ));
    if is_local {
        e.insert(CoopClientLocalPlayer);
    }
}

pub fn coop_client_hud_system(
    net: Res<CoopNetState>,
    mut q: Query<&mut Text, With<CoopHudText>>,
) {
    let Ok(mut text) = q.get_single_mut() else { return };
    if let Some(s) = net.last_snapshot.as_ref() {
        text.sections[0].value = format!(
            "合作模式  P1 HP {:.0} Gold {}   P2 HP {:.0} Gold {}",
            s.p1.hp, s.p1.gold, s.p2.hp, s.p2.gold
        );
    } else {
        text.sections[0].value = "合作模式：等待同步...".to_string();
    }
}

pub fn coop_client_hud_system_v2(net: Res<CoopNetState>, mut q: Query<&mut Text, With<CoopHudText>>) {
    let Ok(mut text) = q.get_single_mut() else { return };
    if let Some(s) = net.last_snapshot.as_ref() {
        let (me, mate) = if net.my_id == Some(1) {
            (&s.p1, &s.p2)
        } else {
            (&s.p2, &s.p1)
        };
        text.sections[0].value = format!(
            "合作模式  你 HP {:.0} EN {:.0} Gold {}   队友 HP {:.0} EN {:.0} Gold {}",
            me.hp, me.energy, me.gold, mate.hp, mate.energy, mate.gold
        );
    } else {
        text.sections[0].value = "合作模式：等待同步...".to_string();
    }
}

pub fn cleanup_coop_client_game(mut commands: Commands, q: Query<Entity, With<CoopClientUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn cleanup_coop_client_dynamic(mut commands: Commands, q: Query<Entity, With<CoopClientEntity>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
