pub mod definitions;
pub mod loaders;
pub mod registry;

use bevy::prelude::*;

use crate::states::AppState;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), loaders::load_all_configs);
    }
}

