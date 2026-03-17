use bevy::prelude::*;

use crate::core::assets::GameAssets;
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
pub struct FloorText;

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
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ..default()
        }, HudUi, Name::new("HudRoot")))
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

                col.spawn((
                    widgets::title_text(&assets, "冲刺：就绪", 16.0),
                    DashText,
                ));
                col.spawn((widgets::title_text(&assets, "楼层：1", 16.0), FloorText));
                col.spawn((widgets::title_text(&assets, "房间：起始", 16.0), RoomText));
                col.spawn((widgets::title_text(&assets, "敌人：0", 16.0), EnemyCountText));
                col.spawn((
                    widgets::title_text(&assets, "提示：右键远程，左键近战；靠近黄色门按 E", 16.0),
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
        });
}

pub fn update_health_bar(player_q: Query<&Health, With<Player>>, mut fill_q: Query<&mut Style, With<HealthFill>>) {
    let Ok(hp) = player_q.get_single() else { return };
    let Ok(mut style) = fill_q.get_single_mut() else { return };
    let ratio = if hp.max > 0.0 { (hp.current / hp.max).clamp(0.0, 1.0) } else { 0.0 };
    style.width = Val::Percent(ratio * 100.0);
}

pub fn update_dash_cooldown_ui(
    player_q: Query<&DashCooldown, With<Player>>,
    mut text_q: Query<&mut Text, With<DashText>>,
) {
    let Ok(cd) = player_q.get_single() else { return };
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let remaining = (cd.timer.duration().as_secs_f32() - cd.timer.elapsed_secs()).max(0.0);
    text.sections[0].value = if cd.timer.finished() {
        "冲刺：就绪".to_string()
    } else {
        format!("冲刺：{:.1}s", remaining)
    };
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
        RoomType::Reward => "奖励",
        RoomType::Puzzle => "解谜",
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
        (RoomType::Start, _) => "提示：右键远程，左键近战；靠近黄色门按 E",
        (RoomType::Reward, _) => "提示：选择奖励后，去门口按 E 继续",
        (RoomType::Boss, RoomState::BossFight) => "提示：击败 Boss 后可选奖励并进入下一关",
        (_, RoomState::Locked) => "提示：清空所有敌人才能开门",
        (_, RoomState::Cleared) => "提示：门已解锁，靠近门按 E",
        _ => "提示：靠近门按 E 切换房间",
    };
    text.sections[0].value = hint.to_string();
}

pub fn update_boss_health_bar(
    boss_q: Query<&crate::gameplay::player::components::Health, (With<crate::gameplay::enemy::components::Enemy>, With<EnemyKind>)>,
    mut boss_fill_q: Query<(&mut Style, &mut Visibility), With<BossHealthFill>>,
    kind_q: Query<&EnemyKind>,
) {
    let Ok((mut style, mut vis)) = boss_fill_q.get_single_mut() else { return };
    let boss = boss_q.iter().zip(kind_q.iter()).find_map(|(hp, kind)| (kind.0 == EnemyType::Boss).then_some(hp));
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
