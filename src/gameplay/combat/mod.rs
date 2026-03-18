pub mod components;
pub mod damage;
pub mod hitbox;
pub mod projectiles;
pub mod systems;

use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(systems::CombatSystemsPlugin);
    }
}
