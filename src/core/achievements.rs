use bevy::prelude::*;
use bevy::utils::HashSet;
use serde::{Deserialize, Serialize};

use crate::core::events::{DamageAppliedEvent, DeathEvent, RoomClearedEvent};
use crate::gameplay::combat::components::Team;
use crate::gameplay::enemy::components::Elite;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::player::components::{Combo, Gold, Player};
use crate::states::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    FirstBlood,
    EliteSlayer,
    Combo10,
    Rich,
    Shopper,
    PuzzleSolver,
    BossSlayer,
    Untouchable,
    Victory,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct AchievementUnlockedEvent {
    pub id: AchievementId,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ShopPurchaseEvent;

#[derive(Resource, Debug, Default, Clone)]
pub struct Achievements {
    pub unlocked: HashSet<AchievementId>,
    pub kills: u32,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
struct NoHitRoom {
    room: Option<u32>,
    took_damage: bool,
}

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Achievements>()
            .init_resource::<NoHitRoom>()
            .add_event::<AchievementUnlockedEvent>()
            .add_event::<ShopPurchaseEvent>()
            .add_systems(
                Update,
                (
                    track_room_entry_no_hit,
                    track_damage_taken_no_hit,
                    track_kills,
                    track_combo_and_gold,
                    track_shop_purchase,
                    track_room_clear,
                ),
            )
            .add_systems(OnEnter(AppState::Victory), unlock_victory);
    }
}

fn unlock_once(ach: &mut Achievements, out: &mut EventWriter<AchievementUnlockedEvent>, id: AchievementId) {
    if ach.unlocked.insert(id) {
        out.send(AchievementUnlockedEvent { id });
        info!("Achievement unlocked: {id:?}");
    }
}

fn track_kills(
    mut death: EventReader<DeathEvent>,
    elite_q: Query<(), With<Elite>>,
    mut ach: ResMut<Achievements>,
    mut unlocked: EventWriter<AchievementUnlockedEvent>,
) {
    for ev in death.read() {
        if ev.team != Team::Enemy {
            continue;
        }
        ach.kills = ach.kills.saturating_add(1);
        if ach.kills >= 1 {
            unlock_once(&mut ach, &mut unlocked, AchievementId::FirstBlood);
        }
        if elite_q.get(ev.entity).is_ok() {
            unlock_once(&mut ach, &mut unlocked, AchievementId::EliteSlayer);
        }
    }
}

fn track_combo_and_gold(
    player_q: Query<(&Combo, &Gold), With<Player>>,
    mut ach: ResMut<Achievements>,
    mut unlocked: EventWriter<AchievementUnlockedEvent>,
) {
    let Ok((combo, gold)) = player_q.get_single() else { return };
    if combo.count >= 10 {
        unlock_once(&mut ach, &mut unlocked, AchievementId::Combo10);
    }
    if gold.0 >= 100 {
        unlock_once(&mut ach, &mut unlocked, AchievementId::Rich);
    }
}

fn track_shop_purchase(
    mut shop: EventReader<ShopPurchaseEvent>,
    mut ach: ResMut<Achievements>,
    mut unlocked: EventWriter<AchievementUnlockedEvent>,
) {
    if shop.read().next().is_some() {
        unlock_once(&mut ach, &mut unlocked, AchievementId::Shopper);
    }
}

fn track_room_entry_no_hit(current: Option<Res<CurrentRoom>>, mut tracker: ResMut<NoHitRoom>) {
    let Some(current) = current else { return };
    if current.is_changed() {
        tracker.room = Some(current.0 .0);
        tracker.took_damage = false;
    }
}

fn track_damage_taken_no_hit(
    player_q: Query<Entity, With<Player>>,
    mut damage: EventReader<DamageAppliedEvent>,
    mut tracker: ResMut<NoHitRoom>,
) {
    let Ok(player_e) = player_q.get_single() else { return };
    for ev in damage.read() {
        if ev.attacker_team == Team::Enemy && ev.target == player_e {
            tracker.took_damage = true;
        }
    }
}

fn track_room_clear(
    mut cleared: EventReader<RoomClearedEvent>,
    layout: Option<Res<FloorLayout>>,
    mut ach: ResMut<Achievements>,
    mut unlocked: EventWriter<AchievementUnlockedEvent>,
    tracker: Res<NoHitRoom>,
) {
    let Some(layout) = layout else { return };
    for ev in cleared.read() {
        let room_type = layout.room(ev.room).map(|r| r.room_type);
        match room_type {
            Some(RoomType::Puzzle) => unlock_once(&mut ach, &mut unlocked, AchievementId::PuzzleSolver),
            Some(RoomType::Boss) => unlock_once(&mut ach, &mut unlocked, AchievementId::BossSlayer),
            _ => {}
        }

        if tracker.room == Some(ev.room.0) && !tracker.took_damage {
            unlock_once(&mut ach, &mut unlocked, AchievementId::Untouchable);
        }
    }
}

fn unlock_victory(mut ach: ResMut<Achievements>, mut unlocked: EventWriter<AchievementUnlockedEvent>) {
    unlock_once(&mut ach, &mut unlocked, AchievementId::Victory);
}

