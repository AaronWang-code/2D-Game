use bevy::app::AppExit;
use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::gameplay::player::components::{
    AttackCooldown, AttackPower, CritChance, DashCooldown, ENERGY_SYSTEM_ENABLED, Energy, Gold,
    Health, MoveSpeed, Player, RangedCooldown, RangedVolleyPattern, RewardModifiers,
};
use crate::states::AppState;
use crate::ui::widgets;

#[derive(Component)]
pub struct PauseUi;

#[derive(Component)]
pub struct PauseCharacterPanel;

#[derive(Component)]
pub struct PauseCharacterText;

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        AppState::InGame => next.set(AppState::Paused),
        AppState::Paused => next.set(AppState::InGame),
        _ => {}
    }
}

pub fn setup_pause_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((widgets::root_node(), PauseUi, Name::new("PauseRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.0, 0.0, 0.0, 0.78)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "游戏暂停", 48.0));
                    panel.spawn(widgets::body_text(&assets, "ESC：继续游戏", 20.0));
                    panel.spawn(widgets::body_text(&assets, "M：回到主菜单", 20.0));
                    panel.spawn(widgets::body_text(&assets, "C：查看角色面板", 20.0));
                    panel.spawn(widgets::body_text(&assets, "Q：退出游戏", 20.0));
                    panel
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(430.0),
                                    margin: UiRect::top(Val::Px(14.0)),
                                    padding: UiRect::all(Val::Px(14.0)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(8.0),
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::srgba(
                                    0.10, 0.12, 0.18, 0.94,
                                )),
                                visibility: Visibility::Hidden,
                                ..default()
                            },
                            PauseCharacterPanel,
                            Name::new("PauseCharacterPanel"),
                        ))
                        .with_children(|stats| {
                            stats.spawn(widgets::title_text(&assets, "角色面板", 24.0));
                            stats.spawn((
                                widgets::body_text(&assets, "加载中...", 18.0),
                                PauseCharacterText,
                            ));
                        });
                });
        });
}

pub fn pause_menu_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut panel_q: Query<&mut Visibility, With<PauseCharacterPanel>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next.set(AppState::InGame);
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        next.set(AppState::MainMenu);
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let _ = exit.send(AppExit::Success);
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        let Ok(mut visibility) = panel_q.get_single_mut() else {
            return;
        };
        *visibility = if *visibility == Visibility::Visible {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

pub fn update_pause_character_panel(
    player_q: Query<
        (
            &Health,
            &Energy,
            &Gold,
            &MoveSpeed,
            &AttackPower,
            &CritChance,
            &AttackCooldown,
            &RangedCooldown,
            &DashCooldown,
            &RewardModifiers,
        ),
        With<Player>,
    >,
    mut text_q: Query<&mut Text, With<PauseCharacterText>>,
) {
    let Ok((hp, energy, gold, move_speed, attack_power, crit, atk_cd, ranged_cd, dash_cd, mods)) =
        player_q.get_single()
    else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };

    let ranged_mode = match mods.ranged_volley_pattern() {
        RangedVolleyPattern::Single => "单发",
        RangedVolleyPattern::Double => "双发",
        RangedVolleyPattern::Triple => "三向",
        RangedVolleyPattern::Nova => "环射",
    };
    let melee_skill = if mods.melee_projectile_reflect_unlocked() {
        "已进入 4 阶以上"
    } else {
        "仍在积累"
    };
    let energy_text = if ENERGY_SYSTEM_ENABLED {
        format!("{:.0} / {:.0}", energy.current, energy.max)
    } else {
        "暂未启用".to_string()
    };

    text.sections[0].value = format!(
        "生命：{:.0} / {:.0}\n金币：{}\n能量：{}\n攻击力：{:.1}\n暴击率：{:.0}%\n移速：{:.0}\n近战冷却：{:.2}s\n远程冷却：{:.2}s\n冲刺冷却：{:.2}s\n近战精通：{} 层（{}，已解锁：{}）\n远程改装：{} 层（当前：{}）\n击杀回血：{:.0}",
        hp.current,
        hp.max,
        gold.0,
        energy_text,
        attack_power.0,
        crit.0 * 100.0,
        move_speed.0,
        atk_cd.timer.duration().as_secs_f32(),
        ranged_cd.timer.duration().as_secs_f32(),
        dash_cd.timer.duration().as_secs_f32(),
        mods.melee_mastery_stacks,
        melee_skill,
        mods.melee_feature_summary(),
        mods.ranged_mastery_stacks,
        ranged_mode,
        mods.lifesteal_on_kill,
    );
}

pub fn cleanup_pause_menu(mut commands: Commands, q: Query<Entity, With<PauseUi>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}
