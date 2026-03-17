use bevy::app::AppExit;
use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::gameplay::enemy::systems::EnemySpawnCount;
use crate::gameplay::progression::floor::FloorNumber;
use crate::states::AppState;
use crate::ui::widgets;

#[derive(Component)]
pub struct MainMenuUi;

#[derive(Component)]
pub(crate) enum MenuButton {
    Start,
    Quit,
}

pub fn setup_main_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn((widgets::root_node(), MainMenuUi, Name::new("MainMenuRoot")))
        .with_children(|root| {
            root.spawn(widgets::panel_node(Color::srgba(0.05, 0.06, 0.10, 0.9)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "迷雾回响", 52.0));
                    panel.spawn(widgets::title_text(
                        &assets,
                        "鼠标左键：近战   鼠标右键：远程   Space：冲刺   E：开门   ESC：暂停",
                        18.0,
                    ));

                    panel
                        .spawn((widgets::button_bundle(), MenuButton::Start))
                        .with_children(|b| {
                            b.spawn(widgets::title_text(&assets, "开始游戏", 22.0));
                        });
                    panel
                        .spawn((widgets::button_bundle(), MenuButton::Quit))
                        .with_children(|b| {
                            b.spawn(widgets::title_text(&assets, "退出", 22.0));
                        });
                });
        });
}

pub fn menu_button_system(
    mut interaction_q: Query<(&Interaction, &MenuButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
) {
    for (interaction, action, mut color) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered => color.0 = Color::srgb(0.24, 0.28, 0.38),
            Interaction::None => color.0 = Color::srgb(0.18, 0.22, 0.30),
            Interaction::Pressed => match action {
                MenuButton::Start => {
                    commands.insert_resource(FloorNumber(1));
                    commands.insert_resource(EnemySpawnCount { current: 0 });
                    next_state.set(AppState::InGame);
                }
                MenuButton::Quit => {
                    let _ = exit.send(AppExit::Success);
                }
            },
        }
    }
}

pub fn cleanup_main_menu(mut commands: Commands, q: Query<Entity, With<MainMenuUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
