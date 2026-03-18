use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::states::AppState;
use crate::ui::widgets;

#[derive(Component)]
pub struct PauseUi;

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
            root.spawn(widgets::panel_node(Color::srgba(0.0, 0.0, 0.0, 0.75)))
                .with_children(|panel| {
                    panel.spawn(widgets::title_text(&assets, "游戏已暂停", 48.0));
                    panel.spawn(widgets::title_text(&assets, "按 ESC 继续游戏", 18.0));
                });
        });
}

pub fn pause_menu_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next.set(AppState::InGame);
    }
}

pub fn cleanup_pause_menu(mut commands: Commands, q: Query<Entity, With<PauseUi>>) {
    for entity in &q {
        commands.entity(entity).despawn_recursive();
    }
}
