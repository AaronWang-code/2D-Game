use bevy::prelude::*;

use crate::core::events::RoomClearedEvent;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};

#[derive(Resource, Debug, Clone, Copy)]
pub struct FloorNumber(pub u32);

pub fn setup_floor(mut commands: Commands, floor: Option<Res<FloorNumber>>) {
    if floor.is_none() {
        commands.insert_resource(FloorNumber(1));
    }
}

pub fn complete_floor(
    mut room_cleared: EventReader<RoomClearedEvent>,
    layout: Res<FloorLayout>,
    current: Res<CurrentRoom>,
) {
    if room_cleared.read().next().is_none() {
        return;
    }
    let room = layout.room(current.0).unwrap();
    if room.room_type == RoomType::Boss {
        // 通关后的流程由奖励系统接管：Boss 清理 -> 奖励三选一 -> 进入下一关。
        // 这里不再直接切换 Victory，避免与 RewardSelect 冲突。
    }
}

pub fn go_to_next_floor(mut floor: ResMut<FloorNumber>) {
    floor.0 += 1;
}
