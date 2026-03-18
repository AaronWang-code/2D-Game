pub mod animation;
pub mod combat;
pub mod components;
pub mod dash;
pub mod systems;

use bevy::prelude::*;

use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::spawn_player)
            .add_systems(
                Update,
                (
                    systems::player_invincibility_system,
                    systems::player_move_system,
                    systems::player_facing_system,
                    combat::player_attack_input_system,
                    combat::player_ranged_input_system,
                    combat::update_attack_cooldowns,
                    combat::update_melee_slash_effects,
                    dash::player_dash_input_system,
                    dash::update_dash_state,
                    animation::update_player_animation_state,
                    animation::animate_player_sprite,
                    systems::player_death_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
