use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::core::{
    achievements::AchievementsPlugin, audio::AudioPlugin, assets::AssetsPlugin, camera::CameraPlugin, events::EventsPlugin,
    input::InputPlugin, save::SavePlugin,
};
use crate::data::DataPlugin;
use crate::gameplay::GameplayPlugin;
use crate::coop::CoopPlugin;
use crate::pvp::PvpPlugin;
use crate::states::AppState;
use crate::ui::UiPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource({
                let mut cfg = RapierConfiguration::new(100.0);
                cfg.gravity = Vec2::ZERO;
                cfg
            })
            .add_plugins((
                EventsPlugin,
                AssetsPlugin,
                DataPlugin,
                InputPlugin,
                AudioPlugin,
                SavePlugin,
                AchievementsPlugin,
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
                CameraPlugin,
                GameplayPlugin,
                CoopPlugin,
                PvpPlugin,
                UiPlugin,
            ));
    }
}
