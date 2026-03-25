use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::enemy::components::Enemy;
use crate::gameplay::enemy::components::{EnemyKind, EnemyType};
use crate::gameplay::map::VisitedRooms;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomId, RoomType};
use crate::gameplay::player::components::{
    DashCooldown, ENERGY_SYSTEM_ENABLED, Energy, Gold, Health, Player,
};
use crate::gameplay::progression::floor::FloorNumber;
use crate::states::RoomState;
use crate::ui::widgets;
use crate::utils::entity::safe_despawn_recursive;

#[derive(Component)]
pub struct HudUi;

#[derive(Component)]
pub struct HealthFill;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct GoldText;

#[derive(Component)]
pub struct EnergyText;

#[derive(Component)]
pub struct DashText;

#[derive(Component)]
pub struct DashIconFill;

#[derive(Component)]
pub struct FloorText;

#[derive(Component)]
pub struct BossHealthBar;

#[derive(Component)]
pub struct BossHealthFill;

#[derive(Component)]
pub struct RoomText;

#[derive(Component)]
pub struct EnemyCountText;

#[derive(Component)]
pub struct HintText;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component, Debug, Clone, Copy)]
pub struct MinimapRoomNode(pub RoomId);

#[derive(Component)]
pub struct MinimapDynamic;

#[derive(Component)]
pub struct StageProgressText;

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
                col.spawn(widgets::title_text(&assets, "生命", 16.0));
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
                col.spawn((
                    widgets::title_text(&assets, "HP: 100 / 100", 15.0),
                    HealthText,
                ));
                col.spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(6.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(4.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.22)),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "属性", 15.0));
                    panel.spawn((widgets::title_text(&assets, "金币: 0", 14.0), GoldText));
                    panel.spawn((
                        widgets::title_text(&assets, "能量: 100 / 100（暂未启用）", 14.0),
                        EnergyText,
                    ));
                });

                col.spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(4.0)),
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.10, 0.12, 0.16)),
                        ..default()
                    })
                    .with_children(|icon| {
                        icon.spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    bottom: Val::Px(0.0),
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::srgb(0.16, 0.74, 0.95)),
                                ..default()
                            },
                            DashIconFill,
                        ));
                        icon.spawn(widgets::title_text(&assets, ">>", 18.0));
                    });

                    row.spawn((widgets::title_text(&assets, "冲刺：就绪", 16.0), DashText));
                });

                col.spawn((
                    widgets::title_text(&assets, "楼层：第 1 层", 16.0),
                    FloorText,
                ));
                col.spawn((widgets::title_text(&assets, "房间：起始", 16.0), RoomText));
                col.spawn((
                    widgets::title_text(&assets, "敌人：0", 16.0),
                    EnemyCountText,
                ));
                col.spawn((
                    widgets::title_text(
                        &assets,
                        "提示：长按鼠标左右键持续攻击，靠近门后按 E 交互",
                        16.0,
                    ),
                    HintText,
                ));
            });

            root.spawn((
                NodeBundle {
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
                },
                BossHealthBar,
            ))
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
                mm.spawn(widgets::title_text(&assets, "关卡进度", 16.0));
                mm.spawn((
                    widgets::title_text(&assets, "第1层：1/1", 15.0),
                    StageProgressText,
                ));
            });
        });
}

pub fn update_health_bar(
    player_q: Query<&Health, With<Player>>,
    mut fill_q: Query<&mut Style, With<HealthFill>>,
) {
    let Ok(hp) = player_q.get_single() else {
        return;
    };
    let Ok(mut style) = fill_q.get_single_mut() else {
        return;
    };
    let ratio = if hp.max > 0.0 {
        (hp.current / hp.max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    style.width = Val::Percent(ratio * 100.0);
}

pub fn update_health_text(
    player_q: Query<&Health, With<Player>>,
    mut text_q: Query<&mut Text, With<HealthText>>,
) {
    let Ok(hp) = player_q.get_single() else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    text.sections[0].value = format!("HP: {:.0} / {:.0}", hp.current, hp.max);
}

pub fn update_gold_text(
    player_q: Query<&Gold, With<Player>>,
    mut text_q: Query<&mut Text, With<GoldText>>,
) {
    let Ok(gold) = player_q.get_single() else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    text.sections[0].value = format!("金币: {}", gold.0);
}

pub fn update_energy_text(
    player_q: Query<&Energy, With<Player>>,
    mut text_q: Query<&mut Text, With<EnergyText>>,
) {
    let Ok(energy) = player_q.get_single() else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    if ENERGY_SYSTEM_ENABLED {
        text.sections[0].value = format!("能量: {:.0} / {:.0}", energy.current, energy.max);
    } else {
        text.sections[0].value = format!(
            "能量: {:.0} / {:.0}（暂未启用）",
            energy.current, energy.max
        );
    }
}

pub fn update_dash_cooldown_ui(
    player_q: Query<&DashCooldown, With<Player>>,
    mut text_q: Query<&mut Text, With<DashText>>,
    mut icon_q: Query<(&mut Style, &mut BackgroundColor), With<DashIconFill>>,
) {
    let Ok(cd) = player_q.get_single() else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    let Ok((mut style, mut color)) = icon_q.get_single_mut() else {
        return;
    };

    let duration = cd.timer.duration().as_secs_f32();
    let remaining = (duration - cd.timer.elapsed_secs()).max(0.0);
    let progress = if duration > 0.0 {
        (cd.timer.elapsed_secs() / duration).clamp(0.0, 1.0)
    } else {
        1.0
    };

    style.height = Val::Percent(progress * 100.0);
    *color = BackgroundColor(if cd.timer.finished() {
        Color::srgb(0.18, 0.82, 0.45)
    } else {
        Color::srgb(0.95, 0.58, 0.24)
    });

    text.sections[0].value = if cd.timer.finished() {
        "冲刺：就绪".to_string()
    } else {
        format!("冲刺：{remaining:.1} 秒")
    };
}

pub fn update_floor_text(
    floor: Option<Res<FloorNumber>>,
    data: Option<Res<GameDataRegistry>>,
    mut text_q: Query<&mut Text, With<FloorText>>,
) {
    let Some(floor) = floor else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    let total_floors = data
        .as_deref()
        .map(|value| value.balance.total_floors.max(1))
        .unwrap_or(4);
    text.sections[0].value = format!("楼层：第 {} 层 / 共 {} 层", floor.0, total_floors);
}

pub fn update_room_text(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    room_state: Option<Res<RoomState>>,
    mut text_q: Query<&mut Text, With<RoomText>>,
) {
    let (Some(layout), Some(current), Some(room_state)) = (layout, current, room_state) else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };

    let room_type = layout
        .room(current.0)
        .map(|room| room.room_type)
        .unwrap_or(RoomType::Start);
    let room_label = match room_type {
        RoomType::Start => "起始",
        RoomType::Normal => "战斗",
        RoomType::Shop => "商店",
        RoomType::Reward => "奖励",
        RoomType::Puzzle => "机关",
        RoomType::Boss => "首领",
    };
    let state_label = match *room_state {
        RoomState::Idle => "可通行",
        RoomState::Locked => "已封锁",
        RoomState::Cleared => "已清理",
        RoomState::BossFight => "首领战",
    };

    text.sections[0].value = format!("房间：{room_label}（{state_label}）");
}

pub fn update_enemy_count_text(
    enemy_q: Query<(), With<Enemy>>,
    mut text_q: Query<&mut Text, With<EnemyCountText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    text.sections[0].value = format!("敌人：{}", enemy_q.iter().count());
}

pub fn update_hint_text(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    room_state: Option<Res<RoomState>>,
    mut text_q: Query<&mut Text, With<HintText>>,
) {
    let (Some(layout), Some(current), Some(room_state)) = (layout, current, room_state) else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };

    let room_type = layout
        .room(current.0)
        .map(|room| room.room_type)
        .unwrap_or(RoomType::Start);
    let hint = match (room_type, *room_state) {
        (RoomType::Start, _) => "提示：长按鼠标左右键持续攻击，靠近门后按 E 前进",
        (RoomType::Reward, _) => "提示：这里会提供奖励，整理状态后继续前进",
        (RoomType::Shop, _) => "提示：商店房可按数字键购买，离开后继续推进",
        (RoomType::Boss, RoomState::BossFight) => "提示：保持移动，合理冲刺，抓住首领空档输出",
        (_, RoomState::Locked) => "提示：清掉房间内所有敌人或完成机关后，门才会开启",
        (_, RoomState::Cleared) => "提示：房门已经打开，靠近后按 E 切换房间",
        _ => "提示：靠近房门后按 E 进入下一个房间",
    };

    text.sections[0].value = hint.to_string();
}

pub fn update_stage_progress(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    floor: Option<Res<FloorNumber>>,
    mut text_q: Query<&mut Text, With<StageProgressText>>,
) {
    let (Some(layout), Some(current)) = (layout, current) else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };

    let distances = room_distances_from_start(&layout);
    let current_step = distances.get(&current.0).copied().unwrap_or(0) + 1;
    let total_steps = layout
        .rooms
        .iter()
        .filter(|room| room.room_type == RoomType::Boss)
        .filter_map(|room| distances.get(&room.id).copied())
        .min()
        .unwrap_or_else(|| distances.values().copied().max().unwrap_or(0))
        + 1;
    let floor_number = floor.as_deref().map(|value| value.0).unwrap_or(1);

    text.sections[0].value = format!("第{}层：{}/{}", floor_number, current_step, total_steps);
}

pub fn update_minimap(
    mut commands: Commands,
    assets: Res<GameAssets>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    visited: Option<Res<VisitedRooms>>,
    root_q: Query<Entity, With<MinimapRoot>>,
    mut nodes_q: Query<(
        Entity,
        &MinimapRoomNode,
        &mut BackgroundColor,
        &mut Style,
        &mut BorderColor,
    )>,
    dynamic_q: Query<Entity, With<MinimapDynamic>>,
) {
    let (Some(layout), Some(current), Some(visited)) = (layout, current, visited) else {
        return;
    };
    let Ok(root) = root_q.get_single() else {
        return;
    };

    let need_rebuild = nodes_q.iter().next().is_none() || layout.is_changed();
    if need_rebuild {
        let existing_nodes: Vec<Entity> = nodes_q.iter().map(|(e, _, _, _, _)| e).collect();
        for e in existing_nodes {
            safe_despawn_recursive(&mut commands, e);
        }
        let existing_dynamic: Vec<Entity> = dynamic_q.iter().collect();
        for e in existing_dynamic {
            safe_despawn_recursive(&mut commands, e);
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
                widgets::body_text(
                    &assets,
                    "白：当前位置 灰：起点 红：战斗 绿：商店 黄：奖励 蓝：机关 紫：Boss",
                    12.0,
                ),
                MinimapDynamic,
            ));
        });
    }

    if !need_rebuild && !current.is_changed() && !visited.is_changed() {
        return;
    }

    for (_, node, mut bg, mut style, mut border) in nodes_q.iter_mut() {
        let Some(room) = layout.room(node.0) else {
            continue;
        };
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

fn room_distances_from_start(layout: &FloorLayout) -> HashMap<RoomId, u32> {
    let mut distances = HashMap::new();
    let mut queue = VecDeque::from([(RoomId(0), 0u32)]);

    while let Some((room_id, distance)) = queue.pop_front() {
        if distances.contains_key(&room_id) {
            continue;
        }
        distances.insert(room_id, distance);

        if let Some(room) = layout.room(room_id) {
            for (_, next_room) in &room.connections.exits {
                if !distances.contains_key(next_room) {
                    queue.push_back((*next_room, distance + 1));
                }
            }
        }
    }

    distances
}

fn room_color(room_type: RoomType) -> (Color, f32) {
    match room_type {
        RoomType::Start => (Color::srgb(0.50, 0.50, 0.55), 12.0),
        RoomType::Normal => (Color::srgb(0.85, 0.35, 0.25), 12.0),
        RoomType::Shop => (Color::srgb(0.25, 0.85, 0.35), 12.0),
        RoomType::Reward => (Color::srgb(0.85, 0.85, 0.20), 12.0),
        RoomType::Puzzle => (Color::srgb(0.25, 0.85, 0.85), 12.0),
        RoomType::Boss => (Color::srgb(0.85, 0.25, 0.95), 14.0),
    }
}

pub fn update_boss_health_bar(
    boss_q: Query<
        (&crate::gameplay::player::components::Health, &EnemyKind),
        With<crate::gameplay::enemy::components::Enemy>,
    >,
    mut boss_fill_q: Query<&mut Style, With<BossHealthFill>>,
    mut boss_bar_q: Query<&mut Visibility, With<BossHealthBar>>,
) {
    let Ok(mut style) = boss_fill_q.get_single_mut() else {
        return;
    };
    let Ok(mut visibility) = boss_bar_q.get_single_mut() else {
        return;
    };

    let boss = boss_q
        .iter()
        .find_map(|(hp, kind)| (kind.0 == EnemyType::Boss).then_some(hp));
    let Some(hp) = boss else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Visible;
    let ratio = if hp.max > 0.0 {
        (hp.current / hp.max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    style.width = Val::Percent(ratio * 100.0);
}

pub fn cleanup_hud(mut commands: Commands, q: Query<Entity, With<HudUi>>) {
    for entity in &q {
        safe_despawn_recursive(&mut commands, entity);
    }
}
