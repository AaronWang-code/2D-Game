use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    MultiplayerMenu,
    CoopMenu,
    CoopLobby,
    CoopGame,
    PvpMenu,
    PvpLobby,
    PvpGame,
    PvpResult,
    Paused,
    RewardSelect,
    Shop,
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
