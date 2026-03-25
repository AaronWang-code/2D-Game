use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::events::{RewardChoiceGroup, RewardChosenEvent};
use crate::gameplay::player::components::{Health, Player, RewardModifiers};
use crate::gameplay::progression::floor::FloorNumber;
use crate::gameplay::rewards::apply::heal_amount;
use crate::gameplay::rewards::data::RewardType;
use crate::gameplay::rewards::systems::{RewardChoices, RewardFlow, RewardFlowMode};
use crate::ui::widgets;
use crate::utils::entity::safe_despawn_recursive;

#[derive(Component)]
pub struct RewardUi;

#[derive(Component, Debug, Clone, Copy)]
pub struct RewardButton {
    pub reward: RewardType,
    pub group: RewardChoiceGroup,
}

pub fn setup_reward_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    choices: Res<RewardChoices>,
    flow: Res<RewardFlow>,
    floor: Option<Res<FloorNumber>>,
    player_q: Query<(&RewardModifiers, &Health), With<Player>>,
) {
    let (mods, health) = player_q
        .get_single()
        .map(|(mods, health)| (*mods, *health))
        .unwrap_or((
            RewardModifiers::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));

    let floor_number = floor.as_deref().map(|value| value.0).unwrap_or(1);
    let heal_value = heal_amount(health.max, floor_number);

    commands
        .spawn((widgets::root_node(), RewardUi, Name::new("RewardRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.02, 0.02, 0.03, 0.92)))
                .with_children(|panel| match flow.mode {
                    RewardFlowMode::SingleBuff => {
                        panel.spawn(widgets::title_text(&assets, "选择一项强化", 30.0));
                        panel.spawn(widgets::title_text(
                            &assets,
                            "按 1 / 2 / 3，或直接点击按钮",
                            16.0,
                        ));
                        spawn_reward_column(
                            panel,
                            &assets,
                            None,
                            &choices.primary,
                            RewardChoiceGroup::Primary,
                            1,
                            mods,
                            health,
                        );
                    }
                    RewardFlowMode::HealOrBuff => {
                        panel.spawn(widgets::title_text(&assets, "普通通关奖励", 30.0));
                        panel.spawn(widgets::title_text(
                            &assets,
                            "左侧选择休息，或在右侧 3 个强化中选择 1 个",
                            16.0,
                        ));
                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    column_gap: Val::Px(18.0),
                                    align_items: AlignItems::FlexStart,
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn(widgets::panel_node(Color::srgba(
                                    0.12, 0.16, 0.12, 0.95,
                                )))
                                .with_children(|col| {
                                    col.spawn(widgets::title_text(&assets, "休息", 24.0));
                                    col.spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Px(250.0),
                                                height: Val::Px(250.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(8.0),
                                                ..default()
                                            },
                                            background_color: BackgroundColor(Color::srgb(
                                                0.18, 0.32, 0.18,
                                            )),
                                            ..default()
                                        },
                                        RewardButton {
                                            reward: RewardType::RecoverHealth,
                                            group: RewardChoiceGroup::Heal,
                                        },
                                    ))
                                    .with_children(|button| {
                                        button.spawn(widgets::title_text(&assets, "1. 回血", 24.0));
                                        button.spawn(widgets::title_text(
                                            &assets,
                                            format!("恢复 {:.0} 生命", heal_value),
                                            22.0,
                                        ));
                                        button.spawn(widgets::body_text(
                                            &assets,
                                            "稳住当前状态后继续推进",
                                            15.0,
                                        ));
                                    });
                                });

                                row.spawn(widgets::panel_node(Color::srgba(
                                    0.12, 0.12, 0.18, 0.95,
                                )))
                                .with_children(|col| {
                                    col.spawn(widgets::title_text(&assets, "强化", 24.0));
                                    spawn_reward_column(
                                        col,
                                        &assets,
                                        None,
                                        &choices.primary,
                                        RewardChoiceGroup::Primary,
                                        2,
                                        mods,
                                        health,
                                    );
                                });
                            });
                    }
                    RewardFlowMode::DualBuff => {
                        panel.spawn(widgets::title_text(&assets, "Boss 通关强化", 30.0));
                        panel.spawn(widgets::title_text(
                            &assets,
                            "已自动恢复生命。左侧选择 1 个，右侧再选择 1 个",
                            16.0,
                        ));
                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    column_gap: Val::Px(18.0),
                                    align_items: AlignItems::FlexStart,
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn(widgets::panel_node(Color::srgba(
                                    0.12, 0.14, 0.22, 0.95,
                                )))
                                .with_children(|col| {
                                    col.spawn(widgets::title_text(&assets, "强化 1", 24.0));
                                    spawn_reward_column(
                                        col,
                                        &assets,
                                        flow.selected_primary,
                                        &choices.primary,
                                        RewardChoiceGroup::Primary,
                                        1,
                                        mods,
                                        health,
                                    );
                                });
                                row.spawn(widgets::panel_node(Color::srgba(
                                    0.18, 0.14, 0.22, 0.95,
                                )))
                                .with_children(|col| {
                                    col.spawn(widgets::title_text(&assets, "强化 2", 24.0));
                                    spawn_reward_column(
                                        col,
                                        &assets,
                                        flow.selected_secondary,
                                        &choices.secondary,
                                        RewardChoiceGroup::Secondary,
                                        4,
                                        mods,
                                        health,
                                    );
                                });
                            });
                    }
                });
        });
}

pub fn update_reward_ui(
    flow: Res<RewardFlow>,
    mut button_q: Query<(&RewardButton, &mut BackgroundColor), With<Button>>,
) {
    for (button, mut color) in &mut button_q {
        let selected = match button.group {
            RewardChoiceGroup::Heal => false,
            RewardChoiceGroup::Primary => flow.selected_primary == Some(button.reward),
            RewardChoiceGroup::Secondary => flow.selected_secondary == Some(button.reward),
        };
        let disabled = match button.group {
            RewardChoiceGroup::Heal => false,
            RewardChoiceGroup::Primary => flow.selected_primary.is_some() && !selected,
            RewardChoiceGroup::Secondary => flow.selected_secondary.is_some() && !selected,
        };

        color.0 = if selected {
            Color::srgb(0.25, 0.52, 0.26)
        } else if disabled {
            Color::srgb(0.12, 0.12, 0.14)
        } else {
            base_button_color(button.group)
        };
    }
}

pub fn reward_ui_input_system(
    mut interaction_q: Query<
        (&Interaction, &RewardButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    flow: Res<RewardFlow>,
    mut chosen: EventWriter<RewardChosenEvent>,
) {
    for (interaction, button, mut color) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered => {
                if !group_locked(&flow, button.group) {
                    color.0 = hover_button_color(button.group);
                }
            }
            Interaction::None => {
                if !group_locked(&flow, button.group) {
                    color.0 = base_button_color(button.group);
                }
            }
            Interaction::Pressed => {
                if group_locked(&flow, button.group) {
                    continue;
                }
                chosen.send(RewardChosenEvent {
                    reward: button.reward,
                    group: button.group,
                });
            }
        }
    }
}

pub fn cleanup_reward_ui(mut commands: Commands, q: Query<Entity, With<RewardUi>>) {
    for entity in &q {
        safe_despawn_recursive(&mut commands, entity);
    }
}

fn reward_copy(
    reward: RewardType,
    mods: RewardModifiers,
    health: Health,
) -> (String, String, Option<String>) {
    match reward {
        RewardType::RecoverHealth => (
            "恢复生命".to_string(),
            if health.current + 1.0 < health.max {
                "立即恢复一截生命，稳住当前节奏。".to_string()
            } else {
                "生命状态稳定时，可优先选择成长型增益。".to_string()
            },
            None,
        ),
        RewardType::EnhanceMeleeWeapon => (
            "近战精通".to_string(),
            "强化近战伤害与范围，2/4/6 解锁吸血 / 裂伤 / 剑风。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::IncreaseAttackSpeed => (
            "攻速强化".to_string(),
            "缩短近战和远程的出手间隔，但有明确上限。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::IncreaseAttackPower => (
            "攻击强化".to_string(),
            "稳定提高近战与远程的基础伤害。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::IncreaseMaxHealth => (
            "生命强化".to_string(),
            "提高生命上限，并顺带回一截血。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::ReduceDashCooldown => (
            "冲刺强化".to_string(),
            "让冲刺恢复更快，走位容错更高。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::LifeStealOnKill => (
            "击杀回血".to_string(),
            "击杀敌人时恢复生命，适合续航推进。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::IncreaseCritChance => (
            "暴击强化".to_string(),
            "提高爆发能力，让输出更有上限。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::IncreaseMoveSpeed => (
            "移速强化".to_string(),
            "提高走位速度，拉扯和躲弹都会更轻松。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::DashDamageTrail => (
            "冲刺残影".to_string(),
            "冲刺时留下伤害轨迹，补足贴身时的压制力。".to_string(),
            reward_progress(mods, reward),
        ),
        RewardType::EnhanceRangedWeapon => (
            "远程改装".to_string(),
            "强化远程伤害、节奏与弹道表现。".to_string(),
            reward_progress(mods, reward),
        ),
    }
}

fn spawn_reward_column(
    parent: &mut ChildBuilder,
    assets: &GameAssets,
    selected: Option<RewardType>,
    choices: &[RewardType],
    group: RewardChoiceGroup,
    start_index: usize,
    mods: RewardModifiers,
    health: Health,
) {
    for (i, reward) in choices.iter().enumerate() {
        let (title, description, progress) = reward_copy(*reward, mods, health);
        let label_index = start_index + i;
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(360.0),
                        height: Val::Px(104.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(if selected == Some(*reward) {
                        Color::srgb(0.25, 0.52, 0.26)
                    } else {
                        base_button_color(group)
                    }),
                    ..default()
                },
                RewardButton {
                    reward: *reward,
                    group,
                },
            ))
            .with_children(|button| {
                button.spawn(widgets::title_text(
                    assets,
                    format!("{}. {}", label_index, title),
                    20.0,
                ));
                button.spawn(widgets::title_text(assets, description, 14.0));
                if let Some(progress) = progress {
                    button.spawn(widgets::body_text(assets, progress, 13.0));
                }
            });
    }
}

fn group_locked(flow: &RewardFlow, group: RewardChoiceGroup) -> bool {
    match group {
        RewardChoiceGroup::Heal => false,
        RewardChoiceGroup::Primary => flow.selected_primary.is_some(),
        RewardChoiceGroup::Secondary => flow.selected_secondary.is_some(),
    }
}

fn base_button_color(group: RewardChoiceGroup) -> Color {
    match group {
        RewardChoiceGroup::Heal => Color::srgb(0.18, 0.32, 0.18),
        RewardChoiceGroup::Primary => Color::srgb(0.18, 0.22, 0.30),
        RewardChoiceGroup::Secondary => Color::srgb(0.26, 0.20, 0.30),
    }
}

fn hover_button_color(group: RewardChoiceGroup) -> Color {
    match group {
        RewardChoiceGroup::Heal => Color::srgb(0.24, 0.40, 0.24),
        RewardChoiceGroup::Primary => Color::srgb(0.24, 0.28, 0.38),
        RewardChoiceGroup::Secondary => Color::srgb(0.34, 0.26, 0.38),
    }
}

fn reward_progress(mods: RewardModifiers, reward: RewardType) -> Option<String> {
    let (current, max) = mods.reward_level(reward)?;
    let filled = "■".repeat(current as usize);
    let empty = "□".repeat(max.saturating_sub(current) as usize);
    Some(format!("进度：{}{} {}/{}", filled, empty, current, max))
}
