pub mod components;
pub mod net;
pub mod ui;
pub mod host;

use bevy::prelude::*;

use crate::states::AppState;

pub struct CoopPlugin;

impl Plugin for CoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<net::CoopNetConfig>()
            .init_resource::<net::CoopNetState>()
            .init_resource::<net::CoopRemoteInput>()
            .add_systems(OnEnter(AppState::CoopMenu), ui::setup_coop_menu)
            .add_systems(Update, ui::coop_menu_input_system.run_if(in_state(AppState::CoopMenu)))
            .add_systems(OnExit(AppState::CoopMenu), ui::cleanup_coop_menu)
            .add_systems(OnEnter(AppState::CoopLobby), ui::setup_coop_lobby)
            .add_systems(
                Update,
                (net::coop_net_tick_system, ui::coop_lobby_ui_system, ui::coop_lobby_input_system)
                    .run_if(in_state(AppState::CoopLobby)),
            )
            .add_systems(OnExit(AppState::CoopLobby), ui::cleanup_coop_lobby)
            .add_systems(OnEnter(AppState::CoopGame), ui::setup_coop_client_game)
            .add_systems(
                Update,
                (
                    net::coop_net_tick_system,
                    ui::coop_client_apply_snapshot_system,
                    ui::coop_client_send_input_system,
                    ui::coop_client_hud_system_v2,
                )
                    .chain()
                    .run_if(in_state(AppState::CoopGame)),
            )
            .add_systems(
                OnExit(AppState::CoopGame),
                (ui::cleanup_coop_client_dynamic, ui::cleanup_coop_client_game),
            )
            // Host-side networking runs inside single-player `InGame`.
            .add_systems(
                Update,
                (
                    net::coop_net_tick_system,
                    host::ensure_coop_player_spawned_system,
                    host::coop_player_invincibility_system,
                    host::coop_player_energy_regen_system,
                    host::coop_player_heal_channel_system,
                    host::coop_player_move_system,
                    host::coop_player_facing_system,
                    host::coop_player_attack_input_system,
                    host::coop_player_ranged_input_system,
                    host::coop_player_dash_input_system,
                    host::coop_update_dash_state,
                    host::coop_player_skill1_input_system,
                    host::coop_player_death_system,
                    net::coop_host_snapshot_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
