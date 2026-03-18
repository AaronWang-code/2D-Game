use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::enemy::components::Enemy;
use crate::gameplay::enemy::components::{EnemyKind, EnemyType};
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::player::components::{DashCooldown, Health, Player};
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
    let Some(floor) = floor else { return };
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
        RoomType::Reward => "休整",
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
        (RoomType::Reward, _) => "提示：这是休整房，整理状态后继续向前",
        (RoomType::Boss, RoomState::BossFight) => "提示：保持移动，合理冲刺，抓住首领空档输出",
        (_, RoomState::Locked) => "提示：清掉房间内所有敌人，门才会开启",
        (_, RoomState::Cleared) => "提示：房门已经打开，靠近后按 E 切换房间",
        _ => "提示：靠近房门后按 E 进入下一个房间",
    };

    text.sections[0].value = hint.to_string();
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
        commands.entity(entity).despawn_recursive();
    }
}
