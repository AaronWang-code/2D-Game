use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Paused,
    RewardSelect,
    GameOver,
    Victory,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RoomState {
    #[default]
    Idle,
    Locked,
    Cleared,
    BossFight,
}
