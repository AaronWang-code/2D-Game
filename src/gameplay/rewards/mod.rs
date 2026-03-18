pub mod apply;
pub mod data;
pub mod systems;

use bevy::prelude::*;

pub struct RewardsPlugin;

impl Plugin for RewardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((systems::RewardsSystemsPlugin,));
    }
}
