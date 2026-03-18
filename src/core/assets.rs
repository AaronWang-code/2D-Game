use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::states::AppState;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub textures: TextureHandles,
    pub audio: AudioHandles,
}

#[derive(Resource, Clone, Default)]
pub struct TextureHandles {
    pub white: Handle<Image>,
    pub player: Handle<Image>,
    pub cursor: Handle<Image>,
    pub crosshair: Handle<Image>,
    pub slash: Handle<Image>,
}

#[derive(Resource, Clone, Default)]
pub struct AudioHandles {
    pub ui_click: Handle<bevy_kira_audio::AudioSource>,
    pub attack: Handle<bevy_kira_audio::AudioSource>,
    pub dash: Handle<bevy_kira_audio::AudioSource>,
    pub hit: Handle<bevy_kira_audio::AudioSource>,
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), load_game_assets)
            .add_systems(
                Update,
                check_assets_ready.run_if(in_state(AppState::Loading)),
            );
    }
}

pub fn load_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let font = asset_server.load("fonts/main_font.ttf");
    let player = asset_server.load("textures/player_hero.png");
    let cursor = images.add(make_cursor_image());
    let crosshair = images.add(make_crosshair_image());
    let slash = images.add(make_slash_image());

    let white = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));

    commands.insert_resource(GameAssets {
        font,
        textures: TextureHandles {
            white,
            player,
            cursor,
            crosshair,
            slash,
        },
        audio: AudioHandles::default(),
    });
}

pub fn check_assets_ready(
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if asset_server.is_loaded_with_dependencies(&assets.font)
        && asset_server.is_loaded_with_dependencies(&assets.textures.player)
    {
        next_state.set(AppState::MainMenu);
    }
}

fn make_cursor_image() -> Image {
    let size = 28u32;
    let mut data = vec![0; (size * size * 4) as usize];

    let shadow = [0, 0, 0, 120];
    let fill = [34, 31, 42, 245];
    let outline = [246, 230, 210, 255];
    let accent = [196, 66, 72, 255];
    let core = [255, 247, 236, 255];

    let center = 13;
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            let shadow_distance = (x - (center + 1)).abs() + (y - (center + 1)).abs();
            if shadow_distance <= 8 {
                set_icon_pixel(&mut data, size, x, y, shadow);
            }

            let distance = (x - center).abs() + (y - center).abs();
            if distance <= 7 {
                let color = if x <= center && y <= center && distance >= 3 {
                    accent
                } else {
                    fill
                };
                set_icon_pixel(&mut data, size, x, y, color);
            }

            if (7..=8).contains(&distance) {
                set_icon_pixel(&mut data, size, x, y, outline);
            }
            if distance <= 1 {
                set_icon_pixel(&mut data, size, x, y, core);
            }
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn make_crosshair_image() -> Image {
    let size = 36u32;
    let mut data = vec![0; (size * size * 4) as usize];

    let shadow = [0, 0, 0, 120];
    let outline = [255, 255, 255, 255];

    draw_crosshair_layer(&mut data, size, 1, 1, shadow);
    draw_crosshair_layer(&mut data, size, 0, 0, outline);

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn make_slash_image() -> Image {
    let size = 96u32;
    let mut data = vec![0; (size * size * 4) as usize];

    let center_x = 28.0;
    let center_y = size as f32 * 0.5;
    let outer_radius = 41.0;
    let inner_radius = 18.0;
    let start_angle = -1.05f32;
    let end_angle = 1.05f32;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let radius = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            if angle < start_angle || angle > end_angle {
                continue;
            }
            if radius < inner_radius || radius > outer_radius {
                continue;
            }

            let radial = ((radius - inner_radius) / (outer_radius - inner_radius)).clamp(0.0, 1.0);
            let angle_center = 1.0 - (angle.abs() / end_angle.abs()).clamp(0.0, 1.0);
            let band_strength = (1.0 - (radial - 0.45).abs() * 1.9).clamp(0.0, 1.0);
            let alpha = (band_strength * (0.35 + angle_center * 0.65) * 255.0) as u8;
            if alpha == 0 {
                continue;
            }

            let glow = (220.0 + angle_center * 35.0) as u8;
            let color = [glow, 250, 255, alpha];
            set_icon_pixel(&mut data, size, x as i32, y as i32, color);
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn draw_crosshair_layer(data: &mut [u8], size: u32, offset_x: i32, offset_y: i32, color: [u8; 4]) {
    draw_hline(
        data,
        size,
        4 + offset_x,
        11 + offset_x,
        7 + offset_y,
        2,
        color,
    );
    draw_hline(
        data,
        size,
        24 + offset_x,
        31 + offset_x,
        7 + offset_y,
        2,
        color,
    );
    draw_hline(
        data,
        size,
        4 + offset_x,
        11 + offset_x,
        28 + offset_y,
        2,
        color,
    );
    draw_hline(
        data,
        size,
        24 + offset_x,
        31 + offset_x,
        28 + offset_y,
        2,
        color,
    );

    draw_vline(
        data,
        size,
        7 + offset_x,
        4 + offset_y,
        11 + offset_y,
        2,
        color,
    );
    draw_vline(
        data,
        size,
        28 + offset_x,
        4 + offset_y,
        11 + offset_y,
        2,
        color,
    );
    draw_vline(
        data,
        size,
        7 + offset_x,
        24 + offset_y,
        31 + offset_y,
        2,
        color,
    );
    draw_vline(
        data,
        size,
        28 + offset_x,
        24 + offset_y,
        31 + offset_y,
        2,
        color,
    );

    draw_hline(
        data,
        size,
        15 + offset_x,
        20 + offset_x,
        17 + offset_y,
        2,
        color,
    );
    draw_vline(
        data,
        size,
        17 + offset_x,
        15 + offset_y,
        20 + offset_y,
        2,
        color,
    );
}

fn draw_hline(
    data: &mut [u8],
    size: u32,
    start_x: i32,
    end_x: i32,
    y: i32,
    thickness: i32,
    color: [u8; 4],
) {
    for dy in 0..thickness {
        for x in start_x..=end_x {
            set_icon_pixel(data, size, x, y + dy, color);
        }
    }
}

fn draw_vline(
    data: &mut [u8],
    size: u32,
    x: i32,
    start_y: i32,
    end_y: i32,
    thickness: i32,
    color: [u8; 4],
) {
    for dx in 0..thickness {
        for y in start_y..=end_y {
            set_icon_pixel(data, size, x + dx, y, color);
        }
    }
}

fn set_icon_pixel(data: &mut [u8], size: u32, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
        return;
    }

    let index = ((y as u32 * size + x as u32) * 4) as usize;
    data[index..index + 4].copy_from_slice(&color);
}
