pub mod difficulty;
pub mod floor;
pub mod stats;

use bevy::prelude::*;

use crate::states::AppState;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), floor::setup_floor)
            .add_systems(Update, (floor::complete_floor, stats::update_run_stats).run_if(in_state(AppState::InGame)));
    }
}

