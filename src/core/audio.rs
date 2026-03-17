use bevy::prelude::*;

use crate::core::assets::GameAssets;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, _app: &mut App) {}
}

// 先保证编译通过：MVP 阶段音频接口保留，但不实际播放。
pub fn play_sfx_ui_click(_audio: Res<bevy_kira_audio::Audio>, _assets: Res<GameAssets>) {}
pub fn play_sfx_attack(_audio: Res<bevy_kira_audio::Audio>, _assets: Res<GameAssets>) {}
pub fn play_sfx_dash(_audio: Res<bevy_kira_audio::Audio>, _assets: Res<GameAssets>) {}
pub fn play_sfx_hit(_audio: Res<bevy_kira_audio::Audio>, _assets: Res<GameAssets>) {}
