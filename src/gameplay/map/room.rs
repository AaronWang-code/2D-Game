use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomType {
    Start,
    Normal,
    Shop,
    Reward,
    Puzzle,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn as_vec2(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::Y,
            Direction::Down => -Vec2::Y,
            Direction::Left => -Vec2::X,
            Direction::Right => Vec2::X,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomConnections {
    pub exits: Vec<(Direction, RoomId)>,
}

#[derive(Debug, Clone)]
pub struct RoomBounds {
    pub half_size: Vec2,
}

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: RoomId,
    pub room_type: RoomType,
    pub mystery: bool,
    pub connections: RoomConnections,
    pub bounds: RoomBounds,
}

#[derive(Resource, Debug, Clone)]
pub struct FloorLayout {
    pub rooms: Vec<RoomData>,
    pub current: RoomId,
}

impl FloorLayout {
    pub fn room(&self, id: RoomId) -> Option<&RoomData> {
        self.rooms.iter().find(|r| r.id == id)
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentRoom(pub RoomId);
