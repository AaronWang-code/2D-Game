mod app;
mod constants;
mod core;
mod data;
mod gameplay;
mod prelude;
mod states;
mod ui;
mod utils;

use bevy::prelude::*;

use crate::app::GamePlugin;
use crate::constants::WINDOW_CLEAR_COLOR;

fn main() {
    App::new()
        .insert_resource(ClearColor(WINDOW_CLEAR_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "迷雾回响 / Echoes in the Fog".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
