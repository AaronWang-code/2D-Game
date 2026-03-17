pub mod afterimage;
pub mod flash;
pub mod particles;
pub mod screen_shake;

use bevy::prelude::*;

use crate::states::AppState;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<screen_shake::ScreenShakeRequest>().add_systems(
            Update,
            (
                flash::update_flash_effect,
                particles::update_particles,
                afterimage::update_afterimages,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

