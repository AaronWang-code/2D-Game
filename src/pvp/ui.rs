use bevy::app::AppExit;
use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::states::AppState;
use crate::ui::widgets;

use super::components::PvpEntity;
use super::net::{start_client_socket, start_host_socket, NetMode, PvpNetConfig, PvpNetState, PVP_PORT};

#[derive(Component)]
pub struct PvpMenuUi;

#[derive(Component)]
pub struct MultiplayerMenuUi;

#[derive(Component, Debug, Clone, Copy)]
pub enum MultiplayerMenuButton {
    Coop,
    Versus,
    Back,
}

#[derive(Component)]
pub struct PvpLobbyUi;

#[derive(Component)]
pub struct PvpResultUi;

#[derive(Component)]
pub struct PvpLobbyText;

#[derive(Component)]
pub struct PvpIpText;

#[derive(Resource, Debug, Default, Clone)]
pub struct PvpJoinIp {
    pub ip: String,
}

pub fn setup_multiplayer_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            widgets::root_node(),
            MultiplayerMenuUi,
            PvpEntity,
            Name::new("MultiplayerMenuRoot"),
        ))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "联机游戏", 52.0));
                    panel.spawn(widgets::title_text(&assets, "请选择模式", 18.0));

                    panel
                        .spawn((widgets::button_bundle(), MultiplayerMenuButton::Coop))
                        .with_children(|b| {
                            b.spawn(widgets::title_text(&assets, "玩家合作（一起闯关）", 22.0));
                        });

                    panel
                        .spawn((widgets::button_bundle(), MultiplayerMenuButton::Versus))
                        .with_children(|b| {
                            b.spawn(widgets::title_text(&assets, "玩家对抗（2P PVP）", 22.0));
                        });

                    panel
                        .spawn((widgets::button_bundle(), MultiplayerMenuButton::Back))
                        .with_children(|b| {
                            b.spawn(widgets::title_text(&assets, "返回", 20.0));
                        });
                });
        });
}

pub fn multiplayer_menu_button_system(
    mut interaction_q: Query<
        (&Interaction, &MultiplayerMenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action, mut color) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered => color.0 = Color::srgb(0.24, 0.28, 0.38),
            Interaction::None => color.0 = Color::srgb(0.18, 0.22, 0.30),
            Interaction::Pressed => match action {
                MultiplayerMenuButton::Coop => next_state.set(AppState::CoopMenu),
                MultiplayerMenuButton::Versus => next_state.set(AppState::PvpMenu),
                MultiplayerMenuButton::Back => next_state.set(AppState::MainMenu),
            },
        }
    }
}

pub fn cleanup_multiplayer_menu(mut commands: Commands, q: Query<Entity, With<MultiplayerMenuUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_pvp_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands.init_resource::<PvpJoinIp>();
    commands
        .spawn((widgets::root_node(), PvpMenuUi, PvpEntity, Name::new("PvpMenuRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "局域网 2P 对战（PVP）", 42.0));
                    panel.spawn(widgets::title_text(
                        &assets,
                        "规则：2人对战；每人3条命；无技能（只保留移动+近战+远程）",
                        18.0,
                    ));
                    panel.spawn(widgets::title_text(&assets, "H=当房主  J=输入房主IP并加入  Esc=返回", 18.0));
                    panel.spawn((widgets::title_text(&assets, "房主IP：", 18.0), PvpIpText));
                });
        });
}

pub fn pvp_menu_input_system(
    mut chars: EventReader<ReceivedCharacter>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ip: ResMut<PvpJoinIp>,
    mut ip_text_q: Query<&mut Text, With<PvpIpText>>,
    mut config: ResMut<PvpNetConfig>,
    mut net: ResMut<PvpNetState>,
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
        next.set(AppState::PvpLobby);
        return;
    }

    // Edit IP.
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
            if let Ok(addr) = format!("{host}:{PVP_PORT}").parse() {
                net.peer = Some(addr);
            }
            next.set(AppState::PvpLobby);
        }
    }

    ip_text.sections[0].value = format!("房主IP：{}", ip.ip);
}

pub fn cleanup_pvp_menu(mut commands: Commands, q: Query<Entity, With<PvpMenuUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_pvp_lobby(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((widgets::root_node(), PvpLobbyUi, PvpEntity, Name::new("PvpLobbyRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "PVP 联机大厅", 46.0));
                    panel.spawn((widgets::title_text(&assets, "连接中...", 18.0), PvpLobbyText));
                    panel.spawn(widgets::title_text(&assets, "Esc=取消并返回菜单", 18.0));
                });
        });
}

pub fn pvp_lobby_ui_system(
    config: Res<PvpNetConfig>,
    net: Res<PvpNetState>,
    mut q: Query<&mut Text, With<PvpLobbyText>>,
) {
    let Ok(mut text) = q.get_single_mut() else { return };
    let status = match config.mode {
        NetMode::Host => {
            if net.connected {
                format!("已连接：客户端 {}", net.peer.map(|p| p.to_string()).unwrap_or_default())
            } else {
                format!("房主已启动，等待客户端连接（端口 {PVP_PORT}）")
            }
        }
        NetMode::Client => {
            let host = config.host_ip.clone();
            if net.connected {
                format!("已连接到房主：{host}:{PVP_PORT}")
            } else {
                format!("正在连接：{host}:{PVP_PORT}（请确认房主已按 H）")
            }
        }
        NetMode::None => "未选择模式".to_string(),
    };
    text.sections[0].value = status;
}

pub fn pvp_lobby_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<PvpNetConfig>,
    mut net: ResMut<PvpNetState>,
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

pub fn cleanup_pvp_lobby(mut commands: Commands, q: Query<Entity, With<PvpLobbyUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_pvp_result(mut commands: Commands, assets: Res<GameAssets>, net: Res<PvpNetState>) {
    let winner = net.winner.unwrap_or(0);
    let title = if winner == 1 || winner == 2 {
        format!("P{winner} 获胜！")
    } else {
        "对局结束".to_string()
    };

    commands
        .spawn((widgets::root_node(), PvpResultUi, PvpEntity, Name::new("PvpResultRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, title, 56.0));
                    panel.spawn(widgets::title_text(&assets, "Enter=返回主菜单", 22.0));
                    panel.spawn(widgets::title_text(&assets, "Esc=退出游戏", 18.0));
                });
        });
}

pub fn pvp_result_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut config: ResMut<PvpNetConfig>,
    mut net: ResMut<PvpNetState>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        config.mode = NetMode::None;
        net.socket = None;
        net.peer = None;
        net.connected = false;
        net.my_id = None;
        net.clear_runtime();
        next.set(AppState::MultiplayerMenu);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        let _ = exit.send(AppExit::Success);
    }
}

pub fn cleanup_pvp_result(mut commands: Commands, q: Query<Entity, With<PvpResultUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
