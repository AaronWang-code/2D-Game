pub mod pressure_plate;
pub mod switch_order;
pub mod trap;

use bevy::prelude::*;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, _app: &mut App) {}
}
