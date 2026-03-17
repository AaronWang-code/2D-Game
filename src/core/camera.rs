use bevy::prelude::*;

use crate::gameplay::effects::screen_shake::{ScreenShake, ScreenShakeRequest};
use crate::gameplay::player::components::Player;
use crate::states::AppState;

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (camera_follow_player, apply_screen_shake)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

pub fn camera_follow_player(
    player_q: Query<&GlobalTransform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let Ok(mut camera_tf) = camera_q.get_single_mut() else { return };

    let target = player_tf.translation().truncate();
    let current = camera_tf.translation.truncate();
    let lerp = 1.0 - (-10.0 * time.delta_seconds()).exp();
    let next = current.lerp(target, lerp);
    camera_tf.translation.x = next.x;
    camera_tf.translation.y = next.y;
}

pub fn apply_screen_shake(
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
    mut shake: ResMut<ScreenShake>,
    mut requests: EventReader<ScreenShakeRequest>,
    time: Res<Time>,
) {
    for req in requests.read() {
        shake.trigger(req.strength, req.duration);
    }

    let Ok(mut tf) = camera_q.get_single_mut() else { return };
    let offset = shake.update(time.delta_seconds());
    tf.translation.x += offset.x;
    tf.translation.y += offset.y;
}
