use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::events::RewardChosenEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::rewards::data::RewardType;
use crate::gameplay::rewards::systems::RewardChoices;
use crate::ui::widgets;

#[derive(Component)]
pub struct RewardUi;

#[derive(Component, Debug, Clone, Copy)]
pub struct RewardButton(pub usize);

pub fn setup_reward_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    choices: Res<RewardChoices>,
    data: Option<Res<GameDataRegistry>>,
) {
    commands
        .spawn((widgets::root_node(), RewardUi, Name::new("RewardRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.02, 0.02, 0.03, 0.92)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "选择一项增益", 30.0));
                    panel.spawn(widgets::title_text(
                        &assets,
                        "按 1 / 2 / 3，或直接点击按钮",
                        16.0,
                    ));
                    for (i, reward) in choices.choices.iter().enumerate() {
                        let (title, description) = reward_copy(data.as_deref(), *reward);
                        panel
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(420.0),
                                        height: Val::Px(96.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(6.0),
                                        ..default()
                                    },
                                    background_color: BackgroundColor(Color::srgb(
                                        0.18, 0.22, 0.30,
                                    )),
                                    ..default()
                                },
                                RewardButton(i),
                            ))
                            .with_children(|button| {
                                button.spawn(widgets::title_text(
                                    &assets,
                                    format!("{}. {}", i + 1, title),
                                    20.0,
                                ));
                                button.spawn(widgets::title_text(&assets, description, 14.0));
                            });
                    }
                });
        });
}

pub fn update_reward_ui() {}

pub fn reward_ui_input_system(
    mut interaction_q: Query<
        (&Interaction, &RewardButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    choices: Res<RewardChoices>,
    mut chosen: EventWriter<RewardChosenEvent>,
) {
    for (interaction, button, mut color) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered => color.0 = Color::srgb(0.24, 0.28, 0.38),
            Interaction::None => color.0 = Color::srgb(0.18, 0.22, 0.30),
            Interaction::Pressed => {
                if let Some(reward) = choices.choices.get(button.0).copied() {
                    chosen.send(RewardChosenEvent { reward });
                }
            }
        }
    }
}

pub fn cleanup_reward_ui(mut commands: Commands, q: Query<Entity, With<RewardUi>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}

fn reward_copy(data: Option<&GameDataRegistry>, reward: RewardType) -> (String, String) {
    if let Some(data) = data {
        if let Some(cfg) = data.rewards.rewards.iter().find(|cfg| cfg.reward == reward) {
            return (cfg.title.clone(), cfg.description.clone());
        }
    }

    match reward {
        RewardType::EnhanceMeleeWeapon => (
            "近战精通".to_string(),
            "强化近战伤害、攻击距离和攻击范围，并略微缩短近战冷却。持续强化后可解锁弹反。"
                .to_string(),
        ),
        RewardType::IncreaseAttackSpeed => (
            "攻速提升 +10%".to_string(),
            "提高近战与远程的攻击速度。".to_string(),
        ),
        RewardType::IncreaseMaxHealth => (
            "最大生命 +20".to_string(),
            "提高生命上限，并立刻回复同等生命。".to_string(),
        ),
        RewardType::ReduceDashCooldown => {
            ("冲刺冷却 -15%".to_string(), "冲刺恢复得更快。".to_string())
        }
        RewardType::LifeStealOnKill => (
            "击杀回血".to_string(),
            "每次击杀回复生命，不会在打首领时按命中吸血。".to_string(),
        ),
        RewardType::IncreaseCritChance => (
            "暴击率 +5%".to_string(),
            "近战与远程攻击都有机会造成暴击。".to_string(),
        ),
        RewardType::IncreaseMoveSpeed => (
            "移速提升 +18%".to_string(),
            "显著提高移动速度，让走位更有手感。".to_string(),
        ),
        RewardType::DashDamageTrail => (
            "冲刺残影".to_string(),
            "冲刺时留下会造成伤害的轨迹。".to_string(),
        ),
        RewardType::EnhanceRangedWeapon => (
            "远程改装".to_string(),
            "强化右键远程的伤害、射速与弹速。".to_string(),
        ),
    }
}
