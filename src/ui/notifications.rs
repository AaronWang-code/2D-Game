use bevy::prelude::*;

use crate::core::achievements::{AchievementId, AchievementUnlockedEvent};
use crate::core::assets::GameAssets;

#[derive(Component)]
pub struct NotificationRoot;

#[derive(Component, Debug, Clone)]
pub struct Toast {
    pub timer: Timer,
}

pub fn ensure_notification_root(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    existing: Query<(), With<NotificationRoot>>,
) {
    if existing.iter().next().is_some() {
        return;
    }
    let Some(_assets) = assets else { return };

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        NotificationRoot,
        Name::new("NotificationRoot"),
    ));
}

pub fn handle_achievement_notifications(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    root_q: Query<Entity, With<NotificationRoot>>,
    mut ev: EventReader<AchievementUnlockedEvent>,
) {
    let Some(assets) = assets else { return };
    let Ok(root) = root_q.get_single() else {
        return;
    };

    for e in ev.read() {
        let msg = achievement_text(e.id);
        commands.entity(root).with_children(|root| {
            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        format!("成就解锁：{msg}"),
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 18.0,
                            color: Color::srgb(1.0, 0.95, 0.55),
                        },
                    ),
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                    ..default()
                },
                Toast {
                    timer: Timer::from_seconds(2.2, TimerMode::Once),
                },
                Name::new("AchievementToast"),
            ));
        });
    }
}

pub fn update_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Toast, &mut Text, &mut BackgroundColor)>,
) {
    for (e, mut toast, mut text, mut bg) in &mut q {
        toast.timer.tick(time.delta());
        let t = toast.timer.fraction();
        let alpha = (1.0 - t).clamp(0.0, 1.0);
        if let Some(section) = text.sections.get_mut(0) {
            section.style.color.set_alpha(alpha);
        }
        bg.0.set_alpha(alpha * 0.65);
        if toast.timer.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn achievement_text(id: AchievementId) -> &'static str {
    match id {
        AchievementId::FirstBlood => "初次击杀",
        AchievementId::EliteSlayer => "精英猎手",
        AchievementId::Combo10 => "连击达人",
        AchievementId::Rich => "腰缠万贯",
        AchievementId::Shopper => "购物达人",
        AchievementId::PuzzleSolver => "解谜专家",
        AchievementId::BossSlayer => "Boss 终结者",
        AchievementId::Untouchable => "无伤清房",
        AchievementId::Victory => "通关！",
    }
}
