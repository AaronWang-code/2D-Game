use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::gameplay::enemy::components::{Enemy, EnemyKind, EnemyType};
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::map::VisitedRooms;
use crate::gameplay::player::components::{DashCooldown, Energy, Gold, Health, Player};
use crate::gameplay::progression::floor::FloorNumber;
use crate::states::RoomState;
use crate::ui::widgets;

#[derive(Component)]
pub struct HudUi;

#[derive(Component)]
pub struct HealthFill;

#[derive(Component)]
pub struct DashText;

#[derive(Component)]
pub struct FloorText;

#[derive(Component)]
pub struct EnergyFill;

#[derive(Component)]
pub struct EnergyText;

#[derive(Component)]
pub struct BossHealthFill;

#[derive(Component)]
pub struct RoomText;

#[derive(Component)]
pub struct EnemyCountText;

#[derive(Component)]
pub struct HintText;

#[derive(Component)]
pub struct GoldText;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct MinimapRoomNode(pub crate::gameplay::map::room::RoomId);

#[derive(Component)]
pub struct MinimapDynamic;

pub fn setup_hud(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            HudUi,
            Name::new("HudRoot"),
        ))
        .with_children(|root| {
            // Top-left HUD.
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
                col.spawn(widgets::title_text(&assets, "生命值", 16.0));
                col.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(240.0),
                        height: Val::Px(18.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.20, 0.85, 0.30)),
                            ..default()
                        },
                        HealthFill,
                    ));
                });

                col.spawn((widgets::title_text(&assets, "能量：0/0", 16.0), EnergyText));
                col.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(240.0),
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
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.25, 0.65, 0.95)),
                            ..default()
                        },
                        EnergyFill,
                    ));
                });

                col.spawn((widgets::title_text(&assets, "冲刺：就绪", 16.0), DashText));
                col.spawn((widgets::title_text(&assets, "金币：0", 16.0), GoldText));
                col.spawn((widgets::title_text(&assets, "连击：0", 16.0), ComboText));
                col.spawn((widgets::title_text(&assets, "楼层：1", 16.0), FloorText));
                col.spawn((widgets::title_text(&assets, "房间：起始", 16.0), RoomText));
                col.spawn((widgets::title_text(&assets, "敌人：0", 16.0), EnemyCountText));
                col.spawn((
                    widgets::title_text(
                        &assets,
                        "提示：左键近战，右键远程连发；Q=散射技能；F=按住回血；B=打开商店；靠近门按 E",
                        16.0,
                    ),
                    HintText,
                ));
            });

            // Top-center boss bar.
            root.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Percent(25.0),
                    width: Val::Percent(50.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                visibility: Visibility::Hidden,
                ..default()
            })
            .with_children(|bar| {
                bar.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.85, 0.20, 0.90)),
                        ..default()
                    },
                    BossHealthFill,
                ));
            });

            // Top-right minimap.
            root.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(16.0),
                        top: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(6.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.28)),
                    ..default()
                },
                MinimapRoot,
                Name::new("MinimapRoot"),
            ))
            .with_children(|mm| {
                mm.spawn(widgets::title_text(&assets, "小地图", 16.0));
            });
        });
}

pub fn update_health_bar(player_q: Query<&Health, With<Player>>, mut fill_q: Query<&mut Style, With<HealthFill>>) {
    let Ok(hp) = player_q.get_single() else { return };
    let Ok(mut style) = fill_q.get_single_mut() else { return };
    let ratio = if hp.max > 0.0 { (hp.current / hp.max).clamp(0.0, 1.0) } else { 0.0 };
    style.width = Val::Percent(ratio * 100.0);
}

pub fn update_energy_bar(
    player_q: Query<&Energy, With<Player>>,
    mut fill_q: Query<&mut Style, With<EnergyFill>>,
    mut text_q: Query<&mut Text, With<EnergyText>>,
) {
    let Ok(energy) = player_q.get_single() else { return };
    let Ok(mut style) = fill_q.get_single_mut() else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let ratio = if energy.max > 0.0 {
        (energy.current / energy.max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    style.width = Val::Percent(ratio * 100.0);
    text.sections[0].value = format!("能量：{:.0}/{:.0}", energy.current, energy.max);
}

pub fn update_dash_cooldown_ui(player_q: Query<&DashCooldown, With<Player>>, mut text_q: Query<&mut Text, With<DashText>>) {
    let Ok(cd) = player_q.get_single() else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let remaining = (cd.timer.duration().as_secs_f32() - cd.timer.elapsed_secs()).max(0.0);
    text.sections[0].value = if cd.timer.finished() {
        "冲刺：就绪".to_string()
    } else {
        format!("冲刺：{remaining:.1}s")
    };
}

pub fn update_gold_text(player_q: Query<&Gold, With<Player>>, mut text_q: Query<&mut Text, With<GoldText>>) {
    let Ok(gold) = player_q.get_single() else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    text.sections[0].value = format!("金币：{}", gold.0);
}

pub fn update_combo_text(
    player_q: Query<&crate::gameplay::player::components::Combo, With<Player>>,
    mut text_q: Query<&mut Text, With<ComboText>>,
) {
    let Ok(combo) = player_q.get_single() else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    text.sections[0].value = format!("连击：{}", combo.count);
}

pub fn update_floor_text(floor: Option<Res<FloorNumber>>, mut text_q: Query<&mut Text, With<FloorText>>) {
    let Some(floor) = floor else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    text.sections[0].value = format!("楼层：{}", floor.0);
}

pub fn update_room_text(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    room_state: Option<Res<RoomState>>,
    mut text_q: Query<&mut Text, With<RoomText>>,
) {
    let (Some(layout), Some(current), Some(room_state)) = (layout, current, room_state) else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let room_type = layout.room(current.0).map(|r| r.room_type).unwrap_or(RoomType::Start);
    let ty = match room_type {
        RoomType::Start => "起始",
        RoomType::Normal => "战斗",
        RoomType::Shop => "商店",
        RoomType::Reward => "奖励",
        RoomType::Puzzle => "（已取消机关）",
        RoomType::Boss => "Boss",
    };
    let st = match *room_state {
        RoomState::Idle => "可通行",
        RoomState::Locked => "已锁门",
        RoomState::Cleared => "已清理",
        RoomState::BossFight => "Boss战",
    };
    text.sections[0].value = format!("房间：{ty}（{st}）");
}

pub fn update_enemy_count_text(enemy_q: Query<(), With<Enemy>>, mut text_q: Query<&mut Text, With<EnemyCountText>>) {
    let Ok(mut text) = text_q.get_single_mut() else { return };
    text.sections[0].value = format!("敌人：{}", enemy_q.iter().count());
}

pub fn update_hint_text(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    room_state: Option<Res<RoomState>>,
    mut text_q: Query<&mut Text, With<HintText>>,
) {
    let (Some(layout), Some(current), Some(room_state)) = (layout, current, room_state) else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let room_type = layout.room(current.0).map(|r| r.room_type).unwrap_or(RoomType::Start);

    let hint = match (room_type, *room_state) {
        (RoomType::Start, _) => "提示：左键近战，右键远程连发；Q=散射技能；F=按住回血；B=打开商店；靠近门按 E",
        (RoomType::Shop, _) => "提示：商店内按 1/2/3 购买；按 Esc 关闭商店",
        (RoomType::Reward, _) => "提示：选择奖励后，去门口按 E 继续",
        (RoomType::Boss, RoomState::BossFight) => "提示：击败 Boss 后即可胜利",
        (_, RoomState::Locked) => "提示：清空所有敌人才能开门",
        (_, RoomState::Cleared) => "提示：门已解锁，靠近门按 E",
        _ => "提示：靠近门按 E 切换房间",
    };
    text.sections[0].value = hint.to_string();
}

pub fn update_minimap(
    mut commands: Commands,
    assets: Res<GameAssets>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    visited: Option<Res<VisitedRooms>>,
    root_q: Query<Entity, With<MinimapRoot>>,
    mut nodes_q: Query<(Entity, &MinimapRoomNode, &mut BackgroundColor, &mut Style, &mut BorderColor)>,
    dynamic_q: Query<Entity, With<MinimapDynamic>>,
) {
    let (Some(layout), Some(current), Some(visited)) = (layout, current, visited) else { return };
    let Ok(root) = root_q.get_single() else { return };

    let need_rebuild = nodes_q.iter().next().is_none() || layout.is_changed();
    if need_rebuild {
        let existing_nodes: Vec<Entity> = nodes_q.iter().map(|(e, _, _, _, _)| e).collect();
        for e in existing_nodes {
            commands.entity(e).despawn_recursive();
        }
        let existing_dynamic: Vec<Entity> = dynamic_q.iter().collect();
        for e in existing_dynamic {
            commands.entity(e).despawn_recursive();
        }

        commands.entity(root).with_children(|mm| {
            mm.spawn((
                NodeBundle {
                    style: Style {
                        column_gap: Val::Px(6.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                },
                MinimapDynamic,
                Name::new("MinimapRow"),
            ))
            .with_children(|row| {
                for room in &layout.rooms {
                    let (base, size) = room_color(room.room_type);
                    let visited_room = visited.0.contains(&room.id);
                    let alpha = if visited_room { 0.95 } else { 0.25 };
                    row.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(size),
                                height: Val::Px(size),
                                border: UiRect::all(Val::Px(0.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(base.with_alpha(alpha)),
                            border_color: BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                            ..default()
                        },
                        MinimapRoomNode(room.id),
                        MinimapDynamic,
                        Name::new(format!("MinimapRoom{}", room.id.0)),
                    ));
                }
            });

            mm.spawn((
                widgets::body_text(&assets, "白=当前位置 灰=起点 红=战斗 黄=奖励 紫=Boss", 12.0),
                MinimapDynamic,
            ));
        });
    }

    if !need_rebuild && !current.is_changed() && !visited.is_changed() {
        return;
    }

    for (_, node, mut bg, mut style, mut border) in nodes_q.iter_mut() {
        let Some(room) = layout.room(node.0) else { continue };
        let (base, _) = room_color(room.room_type);
        let visited_room = visited.0.contains(&node.0);
        let alpha = if visited_room { 0.95 } else { 0.25 };
        let mut col = base.with_alpha(alpha);
        style.border = UiRect::all(Val::Px(0.0));
        border.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
        if node.0 == current.0 {
            col = Color::srgb(1.0, 1.0, 1.0).with_alpha(0.95);
            style.border = UiRect::all(Val::Px(2.0));
            border.0 = Color::srgba(0.0, 0.0, 0.0, 0.85);
        }
        *bg = BackgroundColor(col);
    }
}

fn room_color(room_type: RoomType) -> (Color, f32) {
    match room_type {
        RoomType::Start => (Color::srgb(0.50, 0.50, 0.55), 12.0),
        RoomType::Normal => (Color::srgb(0.85, 0.35, 0.25), 12.0),
        RoomType::Puzzle => (Color::srgb(0.25, 0.85, 0.85), 12.0),
        RoomType::Reward => (Color::srgb(0.85, 0.85, 0.20), 12.0),
        RoomType::Boss => (Color::srgb(0.85, 0.25, 0.95), 14.0),
        RoomType::Shop => (Color::srgb(0.25, 0.85, 0.35), 12.0),
    }
}

pub fn update_boss_health_bar(
    boss_q: Query<&crate::gameplay::player::components::Health, (With<crate::gameplay::enemy::components::Enemy>, With<EnemyKind>)>,
    mut boss_fill_q: Query<(&mut Style, &mut Visibility), With<BossHealthFill>>,
    kind_q: Query<&EnemyKind>,
) {
    let Ok((mut style, mut vis)) = boss_fill_q.get_single_mut() else { return };
    let boss = boss_q
        .iter()
        .zip(kind_q.iter())
        .find_map(|(hp, kind)| (kind.0 == EnemyType::Boss).then_some(hp));
    let Some(hp) = boss else {
        *vis = Visibility::Hidden;
        return;
    };
    *vis = Visibility::Visible;
    let ratio = if hp.max > 0.0 { (hp.current / hp.max).clamp(0.0, 1.0) } else { 0.0 };
    style.width = Val::Percent(ratio * 100.0);
}

pub fn cleanup_hud(mut commands: Commands, q: Query<Entity, With<HudUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
