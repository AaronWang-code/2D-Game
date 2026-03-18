pub mod ai;
pub mod boss;
pub mod components;
pub mod spawner;
pub mod systems;

use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((systems::EnemySystemsPlugin,));
    }
}
