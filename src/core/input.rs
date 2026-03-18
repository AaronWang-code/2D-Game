use bevy::prelude::*;

use crate::states::AppState;

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PlayerInputState {
    pub move_axis: Vec2,
    pub attack_pressed: bool,
    pub ranged_pressed: bool,
    pub attack_held: bool,
    pub ranged_held: bool,
    pub dash_pressed: bool,
    pub interact_pressed: bool,
    pub pause_pressed: bool,
    pub aim_world: Option<Vec2>,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputState>().add_systems(
            Update,
            collect_player_input.run_if(in_state(AppState::InGame)),
        );
    }
}

pub fn collect_player_input(
    mut input: ResMut<PlayerInputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    *input = PlayerInputState::default();
    input.move_axis = map_keyboard_to_movement(&keyboard);

    input.attack_pressed =
        mouse.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::KeyJ);
    input.ranged_pressed = mouse.just_pressed(MouseButton::Right);
    input.attack_held = mouse.pressed(MouseButton::Left) || keyboard.pressed(KeyCode::KeyJ);
    input.ranged_held = mouse.pressed(MouseButton::Right);
    input.dash_pressed = keyboard.just_pressed(KeyCode::Space);
    input.interact_pressed = keyboard.just_pressed(KeyCode::KeyE);
    input.pause_pressed = keyboard.just_pressed(KeyCode::Escape);

    input.aim_world = map_mouse_to_aim_world(windows, camera_q);
}

pub fn map_keyboard_to_movement(keyboard: &ButtonInput<KeyCode>) -> Vec2 {
    let mut axis = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        axis.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        axis.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        axis.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        axis.x += 1.0;
    }
    if axis.length_squared() > 1.0 {
        axis = axis.normalize();
    }
    axis
}

pub fn map_mouse_to_aim_world(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = camera_q.get_single().ok()?;
    let ray = camera.viewport_to_world(camera_transform, cursor)?;
    let world_cursor = ray.origin.truncate();
    Some(world_cursor)
}
