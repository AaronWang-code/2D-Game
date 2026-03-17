use bevy::prelude::*;

use crate::core::{audio::AudioPlugin, assets::AssetsPlugin, camera::CameraPlugin, events::EventsPlugin, input::InputPlugin};
use crate::data::DataPlugin;
use crate::gameplay::GameplayPlugin;
use crate::states::AppState;
use crate::ui::UiPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_plugins((
                EventsPlugin,
                AssetsPlugin,
                DataPlugin,
                InputPlugin,
                AudioPlugin,
                CameraPlugin,
                GameplayPlugin,
                UiPlugin,
            ));
    }
}

