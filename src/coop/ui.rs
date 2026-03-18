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
pub struct CoopClientEntity;

#[derive(Component)]
pub struct CoopClientStatusText;

#[derive(Component)]
pub struct CoopLocalHealthText;

#[derive(Component)]
pub struct CoopLocalHealthFill;

#[derive(Component)]
pub struct CoopLocalEnergyText;

#[derive(Component)]
pub struct CoopLocalEnergyFill;

#[derive(Component)]
pub struct CoopLocalGoldText;

#[derive(Component)]
pub struct CoopMateHealthText;

#[derive(Component)]
pub struct CoopMateHealthFill;

#[derive(Component)]
pub struct CoopMateEnergyText;

#[derive(Component)]
pub struct CoopMateEnergyFill;

#[derive(Component)]
pub struct CoopMateGoldText;

#[derive(Component)]
pub struct CoopHostOverlayUi;

#[derive(Component)]
pub struct CoopHostStatusText;

#[derive(Component)]
pub struct CoopHostMateHealthText;

#[derive(Component)]
pub struct CoopHostMateHealthFill;

#[derive(Component)]
pub struct CoopHostMateEnergyText;

#[derive(Component)]
pub struct CoopHostMateEnergyFill;

#[derive(Component)]
pub struct CoopHostMateGoldText;

#[derive(Component)]
pub struct CoopClientShopUi;

#[derive(Resource, Debug, Default, Clone)]
pub struct CoopJoinIp {
    pub ip: String,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct CoopClientShopState {
    pub open: bool,
    pub pending_purchase: Option<u8>,
    pub rendered_tick: u32,
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
                        "H=当房主（进入游戏等待连接）  J/回车=输入房主IP加入  Esc=返回",
                        18.0,
                    ));
                    panel.spawn((widgets::title_text(&assets, "房主 IP：", 18.0), CoopIpText));
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

    ip_text.sections[0].value = format!("房主 IP：{}", ip.ip);
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
                    panel.spawn((widgets::title_text(&assets, "正在连接...", 18.0), CoopLobbyText));
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
                format!("正在连接：{host}:{COOP_PORT}（请确认房主已开始游戏）")
            }
        }
        NetMode::Host => "房主模式无需在此等待".to_string(),
        NetMode::None => "尚未选择模式".to_string(),
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
    commands.init_resource::<CoopClientShopState>();
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
                    row_gap: Val::Px(8.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .with_children(|col| {
                col.spawn((widgets::title_text(&assets, "合作模式", 20.0), CoopClientStatusText));
                col.spawn(widgets::body_text(&assets, "Esc=断开并返回", 16.0));
            });

            spawn_status_panel(
                root,
                &assets,
                "我的状态",
                UiRect {
                    left: Val::Px(16.0),
                    top: Val::Px(70.0),
                    ..default()
                },
                CoopLocalHealthText,
                CoopLocalHealthFill,
                CoopLocalEnergyText,
                CoopLocalEnergyFill,
                CoopLocalGoldText,
                "CoopClientLocalPanel",
            );
        });
}

pub fn coop_client_shop_input_system(
    input: Res<PlayerInputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    net: Res<CoopNetState>,
    mut shop: ResMut<CoopClientShopState>,
) {
    let in_shop_room = net.last_snapshot.as_ref().map(|s| s.in_shop_room).unwrap_or(false);
    if !in_shop_room {
        shop.open = false;
        shop.pending_purchase = None;
        return;
    }

    if input.interact_pressed || input.shop_pressed {
        shop.open = true;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        shop.open = false;
    }

    shop.pending_purchase = if !shop.open {
        None
    } else if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        shop.open = false;
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        shop.open = false;
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        shop.open = false;
        Some(2)
    } else {
        None
    };
}

pub fn coop_client_send_input_system(
    input: Res<PlayerInputState>,
    config: Res<CoopNetConfig>,
    net: Res<CoopNetState>,
    mut shop: ResMut<CoopClientShopState>,
) {
    if config.mode != NetMode::Client || !net.connected {
        return;
    }
    let mut payload = build_client_input(&input);
    payload.shop_purchase_index = shop.pending_purchase.take();
    let msg = CoopMsg::Input(payload);
    super::net::coop_send_raw(&net, &msg);
}

pub fn update_coop_client_shop_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    net: Res<CoopNetState>,
    mut shop: ResMut<CoopClientShopState>,
    existing: Query<Entity, With<CoopClientShopUi>>,
) {
    let open = shop.open && net.last_snapshot.as_ref().map(|s| s.in_shop_room).unwrap_or(false);
    if !open {
        for e in &existing {
            commands.entity(e).despawn_recursive();
        }
        return;
    }

    let Some(snapshot) = net.last_snapshot.as_ref() else { return };
    if snapshot.shop_offers.is_empty() {
        return;
    }
    if existing.iter().next().is_some() && shop.rendered_tick == snapshot.tick {
        return;
    }

    for e in &existing {
        commands.entity(e).despawn_recursive();
    }
    shop.rendered_tick = snapshot.tick;

    commands
        .spawn((widgets::root_node(), CoopClientUi, CoopClientShopUi, Name::new("CoopClientShopOverlay")))
        .with_children(|root| {
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                    ..default()
                },
                CoopClientShopUi,
            ))
            .with_children(|panel_root| {
                panel_root.spawn(widgets::panel_node(Color::srgba(0.08, 0.08, 0.12, 0.95))).with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "商店", 28.0));
                    panel.spawn(widgets::body_text(&assets, "按 1/2/3 购买，Esc 关闭", 18.0));
                    for (i, offer) in snapshot.shop_offers.iter().enumerate() {
                        panel.spawn(widgets::body_text(
                            &assets,
                            format!("{}. {}（价格：{}）", i + 1, offer.title, offer.cost),
                            20.0,
                        ));
                        panel.spawn(widgets::body_text(&assets, offer.description.clone(), 16.0));
                    }
                });
            });
        });
}

pub fn coop_client_apply_snapshot_system(
    mut commands: Commands,
    assets: Res<GameAssets>,
    config: Res<CoopNetConfig>,
    mut net: ResMut<CoopNetState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    shop: Res<CoopClientShopState>,
    mut next: ResMut<NextState<AppState>>,
    existing: Query<Entity, With<CoopClientEntity>>,
    mut rendered_tick: Local<u32>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !shop.open {
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

    for e in &existing {
        commands.entity(e).despawn_recursive();
    }

    let my_id = net.my_id.unwrap_or(2);
    let (me, mate) = if my_id == 1 {
        (snapshot.p1, snapshot.p2)
    } else {
        (snapshot.p2, snapshot.p1)
    };

    spawn_player_visual(
        &mut commands,
        &assets,
        me.pos,
        Color::srgb(0.35, 0.9, 0.45),
        "CoopLocalPlayer",
        true,
    );
    spawn_player_visual(
        &mut commands,
        &assets,
        mate.pos,
        Color::srgb(0.25, 0.9, 1.0),
        "CoopMatePlayer",
        false,
    );
    spawn_mate_head_text(&mut commands, &assets, mate.pos, mate.hp, mate.hp_max);

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

fn spawn_mate_head_text(commands: &mut Commands, assets: &GameAssets, pos: (f32, f32), hp: f32, hp_max: f32) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("{:.0}/{:.0}", hp, hp_max),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(pos.0, pos.1 + 34.0, 90.0)),
            ..default()
        },
        CoopClientEntity,
        Name::new("CoopMateHeadText"),
    ));
}

fn spawn_status_panel<HealthTextMarker, HealthFillMarker, EnergyTextMarker, EnergyFillMarker, GoldTextMarker>(
    root: &mut ChildBuilder,
    assets: &GameAssets,
    title: &str,
    position: UiRect,
    health_text_marker: HealthTextMarker,
    health_fill_marker: HealthFillMarker,
    energy_text_marker: EnergyTextMarker,
    energy_fill_marker: EnergyFillMarker,
    gold_text_marker: GoldTextMarker,
    name: &str,
) where
    HealthTextMarker: Bundle,
    HealthFillMarker: Bundle,
    EnergyTextMarker: Bundle,
    EnergyFillMarker: Bundle,
    GoldTextMarker: Bundle,
{
    root.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: position.left,
                right: position.right,
                top: position.top,
                bottom: position.bottom,
                width: Val::Px(260.0),
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.72)),
            ..default()
        },
        Name::new(name.to_string()),
    ))
    .with_children(|panel| {
        panel.spawn(widgets::title_text(assets, title, 18.0));
        panel.spawn((widgets::body_text(assets, "生命：0/0", 16.0), health_text_marker));
        panel.spawn(NodeBundle {
            style: Style {
                width: Val::Px(236.0),
                height: Val::Px(16.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.20, 0.85, 0.30)),
                    ..default()
                },
                health_fill_marker,
            ));
        });

        panel.spawn((widgets::body_text(assets, "能量：0/0", 16.0), energy_text_marker));
        panel.spawn(NodeBundle {
            style: Style {
                width: Val::Px(236.0),
                height: Val::Px(14.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.25, 0.65, 0.95)),
                    ..default()
                },
                energy_fill_marker,
            ));
        });

        panel.spawn((widgets::body_text(assets, "金币：0", 16.0), gold_text_marker));
    });
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
