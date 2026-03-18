use bevy::prelude::*;

use crate::constants::UI_Z;
use crate::core::{assets::GameAssets, input::PlayerInputState};
use crate::states::AppState;

const CURSOR_SIZE: f32 = 28.0;
const CROSSHAIR_SIZE: f32 = 34.0;
const CURSOR_HOTSPOT: f32 = CURSOR_SIZE * 0.5;

#[derive(Component)]
pub struct GameCursorUi;

#[derive(Component)]
pub struct AimCrosshair;

pub fn ensure_cursor_visuals(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    cursor_q: Query<(), With<GameCursorUi>>,
    crosshair_q: Query<(), With<AimCrosshair>>,
) {
    let Some(assets) = assets else {
        return;
    };

    if cursor_q.iter().next().is_none() {
        commands.spawn((
            ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(CURSOR_SIZE),
                    height: Val::Px(CURSOR_SIZE),
                    left: Val::Px(-200.0),
                    top: Val::Px(-200.0),
                    ..default()
                },
                image: UiImage::new(assets.textures.cursor.clone()),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(60),
                ..default()
            },
            GameCursorUi,
            Name::new("GameCursorUi"),
        ));
    }

    if crosshair_q.iter().next().is_none() {
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.crosshair.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, UI_Z - 5.0)),
                sprite: Sprite {
                    color: Color::srgba(0.85, 0.97, 1.0, 0.92),
                    custom_size: Some(Vec2::splat(CROSSHAIR_SIZE)),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            AimCrosshair,
            Name::new("AimCrosshair"),
        ));
    }
}

pub fn sync_window_cursor_visibility(state: Res<State<AppState>>, mut windows: Query<&mut Window>) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    let should_hide_system_cursor = uses_custom_cursor(*state.get());
    if window.cursor.visible == should_hide_system_cursor {
        window.cursor.visible = !should_hide_system_cursor;
    }
}

pub fn update_custom_cursor(
    state: Res<State<AppState>>,
    windows: Query<&Window>,
    mut cursor_q: Query<(&mut Style, &mut Visibility), With<GameCursorUi>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((mut style, mut visibility)) = cursor_q.get_single_mut() else {
        return;
    };

    if !uses_custom_cursor(*state.get()) {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some(cursor_pos) = window.cursor_position() else {
        *visibility = Visibility::Hidden;
        return;
    };

    style.left = Val::Px(cursor_pos.x - CURSOR_HOTSPOT);
    style.top = Val::Px(cursor_pos.y - CURSOR_HOTSPOT);
    *visibility = Visibility::Visible;
}

pub fn update_crosshair(
    state: Res<State<AppState>>,
    time: Res<Time>,
    input: Res<PlayerInputState>,
    mut crosshair_q: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<AimCrosshair>>,
) {
    let Ok((mut transform, mut sprite, mut visibility)) = crosshair_q.get_single_mut() else {
        return;
    };

    if *state.get() != AppState::InGame {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some(aim_world) = input.aim_world else {
        *visibility = Visibility::Hidden;
        return;
    };

    let pulse = 1.0 + 0.05 * (time.elapsed_seconds() * 6.0).sin();
    let hold_scale = if input.ranged_held { 1.14 } else { 1.0 };
    transform.translation = aim_world.extend(UI_Z - 5.0);
    transform.rotation = Quat::from_rotation_z(time.elapsed_seconds() * 0.4);
    transform.scale = Vec3::splat(pulse * hold_scale);
    sprite.color = if input.ranged_held {
        Color::srgba(0.48, 0.95, 1.0, 1.0)
    } else {
        Color::srgba(0.90, 0.98, 1.0, 0.90)
    };
    *visibility = Visibility::Visible;
}

fn uses_custom_cursor(state: AppState) -> bool {
    matches!(
        state,
        AppState::InGame | AppState::Paused | AppState::RewardSelect
    )
}
