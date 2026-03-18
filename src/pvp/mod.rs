pub mod components;
pub mod net;
pub mod systems;
pub mod ui;

use bevy::prelude::*;

use crate::states::AppState;

pub struct PvpPlugin;

impl Plugin for PvpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<net::PvpNetConfig>()
            .init_resource::<net::PvpNetState>()
            .init_resource::<systems::PvpMatchState>()
            .add_systems(OnEnter(AppState::MultiplayerMenu), ui::setup_multiplayer_menu)
            .add_systems(Update, ui::multiplayer_menu_button_system.run_if(in_state(AppState::MultiplayerMenu)))
            .add_systems(OnExit(AppState::MultiplayerMenu), ui::cleanup_multiplayer_menu)
            .add_systems(OnEnter(AppState::PvpMenu), ui::setup_pvp_menu)
            .add_systems(Update, ui::pvp_menu_input_system.run_if(in_state(AppState::PvpMenu)))
            .add_systems(OnExit(AppState::PvpMenu), ui::cleanup_pvp_menu)
            .add_systems(OnEnter(AppState::PvpLobby), ui::setup_pvp_lobby)
            .add_systems(
                Update,
                (
                    net::pvp_net_tick_system,
                    ui::pvp_lobby_ui_system,
                    ui::pvp_lobby_input_system,
                )
                    .run_if(in_state(AppState::PvpLobby)),
            )
            .add_systems(OnExit(AppState::PvpLobby), ui::cleanup_pvp_lobby)
            .add_systems(OnEnter(AppState::PvpGame), systems::setup_pvp_game)
            .add_systems(
                Update,
                (
                    net::pvp_net_tick_system,
                    systems::pvp_host_simulation_system,
                    systems::pvp_client_apply_state_system,
                    systems::pvp_send_local_input_system,
                    systems::pvp_update_hud_system_v2,
                    systems::pvp_bullet_visual_system,
                    systems::pvp_bullet_visual_system_move_and_despawn,
                )
                    .run_if(in_state(AppState::PvpGame)),
            )
            .add_systems(OnExit(AppState::PvpGame), systems::cleanup_pvp_world)
            .add_systems(OnEnter(AppState::PvpResult), ui::setup_pvp_result)
            .add_systems(Update, ui::pvp_result_input_system.run_if(in_state(AppState::PvpResult)))
            .add_systems(OnExit(AppState::PvpResult), ui::cleanup_pvp_result);
    }
}
