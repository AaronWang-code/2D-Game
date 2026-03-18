use bevy::prelude::*;

use crate::gameplay::combat::components::Team;
use crate::gameplay::map::room::RoomId;
use crate::gameplay::rewards::data::RewardType;

#[derive(Event, Debug, Clone, Copy)]
pub struct DamageEvent {
    pub target: Entity,
    pub source: Option<Entity>,
    pub amount: f32,
    pub knockback: Vec2,
    pub team: Team,
    pub is_crit: bool,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct DeathEvent {
    pub entity: Entity,
    pub team: Team,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct RoomClearedEvent {
    pub room: RoomId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct RewardChosenEvent {
    pub reward: RewardType,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct DoorOpenEvent {
    pub room: RoomId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnEnemyEvent {
    pub room: RoomId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct BossPhaseChangeEvent {
    pub phase: u8,
}

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_event::<RoomClearedEvent>()
            .add_event::<RewardChosenEvent>()
            .add_event::<DoorOpenEvent>()
            .add_event::<SpawnEnemyEvent>()
            .add_event::<BossPhaseChangeEvent>();
    }
}
