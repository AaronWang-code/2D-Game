use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::assets::GameAssets;
use crate::gameplay::map::InGameEntity;

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_room_tiles_if_missing);
    }
}

#[derive(Component)]
pub struct RoomTiles;

pub fn spawn_room_tiles_if_missing(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    existing: Query<(), With<RoomTiles>>,
) {
    if existing.get_single().is_ok() {
        return;
    }
    let Some(assets) = assets else { return };
    spawn_room_tiles(&mut commands, &assets);
}

pub fn spawn_room_tiles(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            sprite: Sprite {
                color: Color::srgb(0.12, 0.14, 0.18),
                custom_size: Some(Vec2::new(ROOM_HALF_WIDTH * 2.0, ROOM_HALF_HEIGHT * 2.0)),
                ..default()
            },
            ..default()
        },
        RoomTiles,
        InGameEntity,
        Name::new("RoomFloor"),
    ));

    // Simple border walls as visuals.
    let wall_color = Color::srgb(0.20, 0.22, 0.28);
    let thickness = 20.0;
    let size = Vec2::new(ROOM_HALF_WIDTH * 2.0 + thickness, thickness);
    for (name, pos, custom_size) in [
        (
            "WallTop",
            Vec3::new(0.0, ROOM_HALF_HEIGHT + thickness * 0.5, 1.0),
            size,
        ),
        (
            "WallBottom",
            Vec3::new(0.0, -(ROOM_HALF_HEIGHT + thickness * 0.5), 1.0),
            size,
        ),
        (
            "WallLeft",
            Vec3::new(-(ROOM_HALF_WIDTH + thickness * 0.5), 0.0, 1.0),
            Vec2::new(thickness, ROOM_HALF_HEIGHT * 2.0 + thickness),
        ),
        (
            "WallRight",
            Vec3::new(ROOM_HALF_WIDTH + thickness * 0.5, 0.0, 1.0),
            Vec2::new(thickness, ROOM_HALF_HEIGHT * 2.0 + thickness),
        ),
    ] {
        commands.spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(pos),
                sprite: Sprite {
                    color: wall_color,
                    custom_size: Some(custom_size),
                    ..default()
                },
                ..default()
            },
            InGameEntity,
            Name::new(name),
        ));
    }
}
