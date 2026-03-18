use bevy::prelude::*;

// Host-side second player entity (driven by remote input).
#[derive(Component)]
pub struct CoopPlayer;

// Client-side marker for the locally-controlled player visual (camera follows this).
#[derive(Component)]
pub struct CoopClientLocalPlayer;

