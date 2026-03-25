pub mod pressure_plate;
pub mod switch_order;
pub mod trap;

use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::events::RoomClearedEvent;
use crate::gameplay::map::room::CurrentRoom;
use crate::gameplay::map::room::RoomId;
use crate::states::AppState;
use crate::states::RoomState;
use crate::utils::rng::GameRng;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePuzzle>().add_systems(
            Update,
            (
                pressure_plate::pressure_plate_system,
                switch_order::switch_order_system,
                trap::trap_system,
                complete_active_puzzle_system,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct PuzzleEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleKind {
    PressurePlate,
    SwitchOrder,
    TrapSurvival,
}

#[derive(Resource, Debug, Clone)]
pub struct ActivePuzzle {
    pub room: Option<RoomId>,
    pub kind: Option<PuzzleKind>,
    pub completed: bool,
}

impl Default for ActivePuzzle {
    fn default() -> Self {
        Self {
            room: None,
            kind: None,
            completed: false,
        }
    }
}

pub fn clear_active_puzzle(mut active: ResMut<ActivePuzzle>) {
    reset_active_puzzle(&mut active);
}

pub fn reset_active_puzzle(active: &mut ActivePuzzle) {
    active.room = None;
    active.kind = None;
    active.completed = false;
}

pub fn spawn_puzzle_for_room(
    commands: &mut Commands,
    assets: &GameAssets,
    rng: &mut GameRng,
    active: &mut ActivePuzzle,
    room: RoomId,
) {
    active.completed = false;
    let kinds = [
        PuzzleKind::PressurePlate,
        PuzzleKind::SwitchOrder,
        PuzzleKind::TrapSurvival,
    ];
    let idx = (rng.gen_range_f32(0.0, kinds.len() as f32) as usize).min(kinds.len() - 1);
    let kind = kinds[idx];

    match kind {
        PuzzleKind::PressurePlate => {
            pressure_plate::spawn_pressure_plate(commands, assets);
            pressure_plate::activate_pressure_plate_puzzle(active, room);
        }
        PuzzleKind::SwitchOrder => {
            switch_order::spawn_switch_sequence(commands, assets);
            switch_order::activate_switch_order_puzzle(active, room);
        }
        PuzzleKind::TrapSurvival => {
            trap::spawn_traps(commands, assets);
            trap::activate_trap_survival_puzzle(active, room);
        }
    }
}

fn complete_active_puzzle_system(
    current_room: Option<Res<CurrentRoom>>,
    mut room_state: ResMut<RoomState>,
    mut active: ResMut<ActivePuzzle>,
    mut cleared: EventWriter<RoomClearedEvent>,
) {
    let Some(current_room) = current_room else {
        return;
    };
    if !active.completed || active.room != Some(current_room.0) {
        return;
    }
    if !matches!(*room_state, RoomState::Locked) {
        return;
    }

    *room_state = RoomState::Cleared;
    active.completed = false;
    cleared.send(RoomClearedEvent {
        room: current_room.0,
    });
}
