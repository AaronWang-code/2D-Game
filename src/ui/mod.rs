pub mod game_over;
pub mod hud;
pub mod menu;
pub mod pause;
pub mod notifications;
pub mod reward_select;
pub mod shop;
pub mod widgets;

use bevy::prelude::*;

use crate::states::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), menu::setup_main_menu)
            .add_systems(Update, menu::menu_button_system.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnExit(AppState::MainMenu), menu::cleanup_main_menu)
            .add_systems(Update, notifications::ensure_notification_root)
            .add_systems(Update, notifications::handle_achievement_notifications)
            .add_systems(Update, notifications::update_notifications)
            .add_systems(OnEnter(AppState::InGame), hud::setup_hud)
            .add_systems(
                Update,
                (
                    hud::update_health_bar,
                    hud::update_energy_bar,
                    hud::update_dash_cooldown_ui,
                    hud::update_floor_text,
                    hud::update_room_text,
                    hud::update_enemy_count_text,
                    hud::update_hint_text,
                    hud::update_boss_health_bar,
                    hud::update_gold_text,
                    hud::update_combo_text,
                    hud::update_minimap,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), hud::cleanup_hud)
            .add_systems(Update, pause::toggle_pause)
            .add_systems(OnEnter(AppState::Paused), pause::setup_pause_menu)
            .add_systems(Update, pause::pause_menu_keyboard_system.run_if(in_state(AppState::Paused)))
            .add_systems(OnExit(AppState::Paused), pause::cleanup_pause_menu)
            .add_systems(OnEnter(AppState::Shop), shop::setup_shop_ui)
            .add_systems(Update, shop::shop_ui_input_system.run_if(in_state(AppState::Shop)))
            .add_systems(Update, shop::update_shop_ui.run_if(in_state(AppState::Shop)))
            .add_systems(OnExit(AppState::Shop), shop::cleanup_shop_ui)
            .add_systems(OnEnter(AppState::GameOver), game_over::setup_game_over_screen)
            .add_systems(OnEnter(AppState::Victory), game_over::setup_victory_screen)
            .add_systems(
                Update,
                game_over::end_screen_input_system.run_if(in_state(AppState::GameOver).or_else(in_state(AppState::Victory))),
            );
    }
}
