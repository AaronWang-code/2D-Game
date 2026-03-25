use bevy::prelude::*;

use crate::constants::{ROOM_HALF_HEIGHT, ROOM_HALF_WIDTH};
use crate::core::assets::GameAssets;
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::utils::entity::safe_despawn_recursive;

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_room_tiles_if_missing, refresh_room_decor));
    }
}

#[derive(Component)]
pub struct RoomTiles;

#[derive(Component)]
pub struct RoomDecor;

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

pub fn refresh_room_decor(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    existing: Query<Entity, With<RoomDecor>>,
) {
    let (Some(assets), Some(layout), Some(current)) = (assets, layout, current) else {
        return;
    };
    if !layout.is_changed() && !current.is_changed() && existing.iter().next().is_some() {
        return;
    }

    for entity in &existing {
        safe_despawn_recursive(&mut commands, entity);
    }

    let Some(room) = layout.room(current.0) else {
        return;
    };
    spawn_room_decor(&mut commands, &assets, room.room_type);
}

fn spawn_room_decor(commands: &mut Commands, assets: &GameAssets, room_type: RoomType) {
    let (color, size, label, label_pos) = match room_type {
        RoomType::Start => (
            Color::srgba(0.30, 0.62, 0.92, 0.22),
            Vec2::new(180.0, 180.0),
            Some("起点"),
            Vec3::new(0.0, -118.0, 2.0),
        ),
        RoomType::Normal => (
            Color::srgba(0.35, 0.40, 0.52, 0.12),
            Vec2::new(132.0, 132.0),
            None,
            Vec3::ZERO,
        ),
        RoomType::Shop => (
            Color::srgba(0.20, 0.58, 0.24, 0.18),
            Vec2::new(180.0, 118.0),
            Some("商店"),
            Vec3::new(0.0, -118.0, 2.0),
        ),
        RoomType::Reward => (
            Color::srgba(0.88, 0.76, 0.20, 0.22),
            Vec2::new(128.0, 128.0),
            Some("奖励圣坛"),
            Vec3::new(0.0, -118.0, 2.0),
        ),
        RoomType::Puzzle => (
            Color::srgba(0.62, 0.36, 0.86, 0.20),
            Vec2::new(148.0, 148.0),
            Some("机关房"),
            Vec3::new(0.0, -118.0, 2.0),
        ),
        RoomType::Boss => (
            Color::srgba(0.88, 0.24, 0.28, 0.18),
            Vec2::new(220.0, 180.0),
            Some("首领房"),
            Vec3::new(0.0, -118.0, 2.0),
        ),
    };

    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            ..default()
        },
        RoomDecor,
        InGameEntity,
        Name::new("RoomDecor"),
    ));

    if let Some(label) = label {
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 24.0,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.92),
                    },
                ),
                transform: Transform::from_translation(label_pos),
                ..default()
            },
            RoomDecor,
            InGameEntity,
            Name::new("RoomDecorLabel"),
        ));
    }
}
