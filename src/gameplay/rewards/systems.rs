use bevy::prelude::*;

use crate::core::events::{RewardChosenEvent, RoomClearedEvent};
use crate::data::registry::GameDataRegistry;
use crate::gameplay::enemy::systems::{ClearGrace, EnemySpawnCount, SpawnedForRoom};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::player::components::{
    AttackCooldown, CritChance, DashCooldown, Health, MoveSpeed, Player, RangedCooldown,
    RewardModifiers,
};
use crate::gameplay::progression::difficulty::is_final_floor;
use crate::gameplay::progression::floor::FloorNumber;
use crate::gameplay::rewards::apply::apply_reward_to_player_components;
use crate::gameplay::rewards::data::RewardType;
use crate::states::{AppState, RoomState};
use crate::utils::rng::GameRng;

#[derive(Resource, Debug, Default, Clone)]
pub struct RewardChoices {
    pub choices: Vec<RewardType>,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
struct RewardFlow {
    go_next_floor: bool,
    go_victory: bool,
}

pub struct RewardsSystemsPlugin;

impl Plugin for RewardsSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RewardChoices>()
            .init_resource::<RewardFlow>()
            .init_resource::<GameRng>()
            .add_systems(Update, enter_reward_selection)
            .add_systems(
                OnEnter(AppState::RewardSelect),
                crate::ui::reward_select::setup_reward_ui,
            )
            .add_systems(
                Update,
                (
                    handle_reward_choice_input,
                    crate::ui::reward_select::reward_ui_input_system,
                    crate::ui::reward_select::update_reward_ui,
                )
                    .run_if(in_state(AppState::RewardSelect)),
            )
            .add_systems(
                OnExit(AppState::RewardSelect),
                crate::ui::reward_select::cleanup_reward_ui,
            )
            .add_systems(
                Update,
                apply_reward_choice
                    .run_if(in_state(AppState::RewardSelect))
                    .after(handle_reward_choice_input)
                    .after(crate::ui::reward_select::reward_ui_input_system),
            );
    }
}

fn enter_reward_selection(
    mut room_cleared: EventReader<RoomClearedEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut choices: ResMut<RewardChoices>,
    mut rng: ResMut<GameRng>,
    mut flow: ResMut<RewardFlow>,
    data: Option<Res<GameDataRegistry>>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    floor: Option<Res<FloorNumber>>,
) {
    let Some(ev) = room_cleared.read().next() else {
        return;
    };

    flow.go_next_floor = false;
    flow.go_victory = false;

    if let (Some(layout), Some(current)) = (layout.as_deref(), current.as_deref()) {
        if ev.room == current.0 {
            if let Some(room) = layout.room(current.0) {
                if room.room_type == RoomType::Boss {
                    let boss_gives_victory = data
                        .as_deref()
                        .map(|d| d.balance.boss_room_gives_victory)
                        .unwrap_or(false);
                    let current_floor = floor.as_deref().map(|value| value.0).unwrap_or(1);
                    let reached_final_floor = data
                        .as_deref()
                        .map(|registry| is_final_floor(registry, current_floor))
                        .unwrap_or(current_floor >= 4);

                    if boss_gives_victory || reached_final_floor {
                        flow.go_victory = true;
                    } else {
                        flow.go_next_floor = true;
                    }
                }
            }
        }
    }

    choices.choices = generate_reward_choices(&mut rng);
    next_state.set(AppState::RewardSelect);
}

fn generate_reward_choices(rng: &mut GameRng) -> Vec<RewardType> {
    let mut pool = vec![
        RewardType::EnhanceMeleeWeapon,
        RewardType::IncreaseAttackSpeed,
        RewardType::IncreaseMaxHealth,
        RewardType::ReduceDashCooldown,
        RewardType::LifeStealOnKill,
        RewardType::IncreaseCritChance,
        RewardType::IncreaseMoveSpeed,
        RewardType::DashDamageTrail,
        RewardType::EnhanceRangedWeapon,
    ];
    rng.shuffle(&mut pool);
    pool.truncate(3);
    pool
}

fn handle_reward_choice_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<RewardChosenEvent>,
    choices: Res<RewardChoices>,
) {
    let idx = if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        Some(2)
    } else {
        None
    };

    let Some(i) = idx else { return };
    let Some(reward) = choices.choices.get(i).copied() else {
        return;
    };
    events.send(RewardChosenEvent { reward });
}

fn apply_reward_choice(
    mut chosen: EventReader<RewardChosenEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut flow: ResMut<RewardFlow>,
    data: Option<Res<GameDataRegistry>>,
    mut player_q: Query<
        (
            &mut RewardModifiers,
            &mut Health,
            &mut MoveSpeed,
            &mut DashCooldown,
            &mut RangedCooldown,
            &mut CritChance,
            &mut AttackCooldown,
        ),
        With<Player>,
    >,
    mut commands: Commands,
    ingame_entities: Query<(Entity, Option<&Player>), With<InGameEntity>>,
    mut floor: Option<ResMut<FloorNumber>>,
    mut spawned_for_room: ResMut<SpawnedForRoom>,
    mut grace: ResMut<ClearGrace>,
    mut spawn_count: ResMut<EnemySpawnCount>,
) {
    for ev in chosen.read() {
        let value = reward_value_for(data.as_deref(), ev.reward);
        if let Ok((
            mut mods,
            mut health,
            mut move_speed,
            mut dash_cd,
            mut ranged_cd,
            mut crit,
            mut atk_cd,
        )) = player_q.get_single_mut()
        {
            apply_reward_to_player_components(
                ev.reward,
                value,
                &mut mods,
                &mut health,
                &mut move_speed,
                &mut dash_cd,
                &mut ranged_cd,
                &mut crit,
                &mut atk_cd,
            );
        } else {
            warn!("奖励已选择，但没有找到玩家实体。");
        }

        if flow.go_next_floor {
            for (entity, player) in &ingame_entities {
                if player.is_none() {
                    commands.entity(entity).despawn_recursive();
                }
            }

            commands.remove_resource::<FloorLayout>();
            commands.remove_resource::<CurrentRoom>();
            commands.remove_resource::<RoomTransition>();
            commands.remove_resource::<RoomState>();

            if let Some(floor) = floor.as_mut() {
                floor.0 += 1;
            }

            spawned_for_room.0 = None;
            grace.last_room = None;
            grace.timer = Timer::from_seconds(0.0, TimerMode::Once);
            spawn_count.current = 0;
            flow.go_next_floor = false;
        }

        if flow.go_victory {
            flow.go_victory = false;
            flow.go_next_floor = false;
            next_state.set(AppState::Victory);
        } else {
            next_state.set(AppState::InGame);
        }
    }
}

fn reward_value_for(data: Option<&GameDataRegistry>, reward: RewardType) -> f32 {
    data.and_then(|d| {
        d.rewards
            .rewards
            .iter()
            .find(|cfg| cfg.reward == reward)
            .map(|cfg| cfg.value)
    })
    .unwrap_or_else(|| match reward {
        RewardType::EnhanceMeleeWeapon => 1.0,
        RewardType::IncreaseAttackSpeed => 0.10,
        RewardType::IncreaseMaxHealth => 20.0,
        RewardType::ReduceDashCooldown => 0.15,
        RewardType::LifeStealOnKill => 3.0,
        RewardType::IncreaseCritChance => 0.05,
        RewardType::IncreaseMoveSpeed => 0.18,
        RewardType::DashDamageTrail => 1.0,
        RewardType::EnhanceRangedWeapon => 1.0,
    })
}
