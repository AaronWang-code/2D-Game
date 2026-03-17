use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::events::RewardChosenEvent;
use crate::gameplay::rewards::data::RewardType;
use crate::gameplay::rewards::systems::RewardChoices;
use crate::ui::widgets;

#[derive(Component)]
pub struct RewardUi;

#[derive(Component, Debug, Clone, Copy)]
pub struct RewardButton(pub usize);

pub fn setup_reward_ui(mut commands: Commands, assets: Res<GameAssets>, choices: Res<RewardChoices>) {
    commands
        .spawn((widgets::root_node(), RewardUi, Name::new("RewardRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.02, 0.02, 0.03, 0.92)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "选择一个奖励（可按 1/2/3 或点击）", 30.0));
                    for (i, reward) in choices.choices.iter().enumerate() {
                        panel
                            .spawn((widgets::button_bundle(), RewardButton(i)))
                            .with_children(|b| {
                                b.spawn(widgets::title_text(&assets, reward_title(*reward), 20.0));
                            });
                    }
                });
        });
}

pub fn update_reward_ui() {}

pub fn reward_ui_input_system(
    mut interaction_q: Query<(&Interaction, &RewardButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    choices: Res<RewardChoices>,
    mut chosen: EventWriter<RewardChosenEvent>,
) {
    for (interaction, btn, mut color) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered => color.0 = Color::srgb(0.24, 0.28, 0.38),
            Interaction::None => color.0 = Color::srgb(0.18, 0.22, 0.30),
            Interaction::Pressed => {
                if let Some(reward) = choices.choices.get(btn.0).copied() {
                    chosen.send(RewardChosenEvent { reward });
                }
            }
        }
    }
}

pub fn cleanup_reward_ui(mut commands: Commands, q: Query<Entity, With<RewardUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn reward_title(r: RewardType) -> &'static str {
    match r {
        RewardType::IncreaseAttackSpeed => "攻速提升（攻击更快）",
        RewardType::IncreaseMaxHealth => "生命上限 +20（并恢复）",
        RewardType::ReduceDashCooldown => "冲刺冷却缩短",
        RewardType::LifeStealOnKill => "击杀回复 3 点生命",
        RewardType::IncreaseCritChance => "暴击率 +5%",
        RewardType::IncreaseMoveSpeed => "移速 +10%",
        RewardType::DashDamageTrail => "冲刺留下伤害轨迹",
        RewardType::BonusProjectile => "普通攻击附带飞弹",
    }
}
