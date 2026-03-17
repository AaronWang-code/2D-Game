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
            .add_systems(Update, check_assets_ready.run_if(in_state(AppState::Loading)));
    }
}

pub fn load_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let font = asset_server.load("fonts/main_font.ttf");

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
        textures: TextureHandles { white },
        audio: AudioHandles::default(),
    });
}

pub fn check_assets_ready(
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if asset_server.is_loaded_with_dependencies(&assets.font) {
        next_state.set(AppState::MainMenu);
    }
}
