use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::core::events::{RewardChoiceGroup, RewardChosenEvent, RoomClearedEvent};
use crate::data::registry::GameDataRegistry;
use crate::gameplay::enemy::systems::{ClearGrace, EnemySpawnCount, SpawnedForRoom};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomId, RoomType};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::player::components::{
    AttackCooldown, AttackPower, CritChance, DashCooldown, Health, MoveSpeed, Player,
    RangedCooldown, RewardModifiers,
};
use crate::gameplay::progression::difficulty::is_final_floor;
use crate::gameplay::progression::floor::FloorNumber;
use crate::gameplay::rewards::apply::apply_reward_to_player_components;
use crate::gameplay::rewards::data::RewardType;
use crate::states::{AppState, RoomState};
use crate::utils::entity::safe_despawn_recursive;
use crate::utils::rng::GameRng;

#[derive(Resource, Debug, Default, Clone)]
pub struct RewardChoices {
    pub primary: Vec<RewardType>,
    pub secondary: Vec<RewardType>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct RewardRoomClaims {
    pub rooms: HashSet<RoomId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RewardFlowMode {
    #[default]
    SingleBuff,
    HealOrBuff,
    DualBuff,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct RewardFlow {
    pub mode: RewardFlowMode,
    pub go_next_floor: bool,
    pub go_victory: bool,
    pub reward_scale: f32,
    pub selected_primary: Option<RewardType>,
    pub selected_secondary: Option<RewardType>,
}

pub struct RewardsSystemsPlugin;

impl Plugin for RewardsSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RewardChoices>()
            .init_resource::<RewardRoomClaims>()
            .init_resource::<RewardFlow>()
            .init_resource::<GameRng>()
            .add_systems(Update, enter_reward_selection)
            .add_systems(Update, offer_reward_in_reward_room)
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

fn offer_reward_in_reward_room(
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    data: Option<Res<GameDataRegistry>>,
    transition: Option<Res<RoomTransition>>,
    mut claims: ResMut<RewardRoomClaims>,
    mut choices: ResMut<RewardChoices>,
    mut rng: ResMut<GameRng>,
    mut flow: ResMut<RewardFlow>,
    mut next_state: ResMut<NextState<AppState>>,
    player_q: Query<(&RewardModifiers, &Health), With<Player>>,
) {
    if transition
        .as_deref()
        .map(|value| value.active)
        .unwrap_or(false)
    {
        return;
    }
    let (Some(layout), Some(current)) = (layout, current) else {
        return;
    };
    if layout.is_changed() {
        claims.rooms.clear();
    }

    let Some(room) = layout.room(current.0) else {
        return;
    };
    let reward_room_enabled = data
        .as_deref()
        .map(|registry| registry.balance.reward_rooms_give_choice)
        .unwrap_or(true);
    if !reward_room_enabled || room.room_type != RoomType::Reward {
        return;
    }
    if !claims.rooms.insert(current.0) {
        return;
    }

    flow.go_next_floor = false;
    flow.go_victory = false;
    flow.mode = RewardFlowMode::SingleBuff;
    flow.reward_scale = 1.0;
    flow.selected_primary = None;
    flow.selected_secondary = None;
    let (mods, _health) = player_q
        .get_single()
        .map(|(mods, health)| (*mods, *health))
        .unwrap_or((
            RewardModifiers::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));
    choices.primary = generate_reward_choices(&mut rng, mods, &[]);
    choices.secondary.clear();
    next_state.set(AppState::RewardSelect);
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
    mut player_q: Query<(&RewardModifiers, &mut Health), With<Player>>,
) {
    let Some(ev) = room_cleared.read().next() else {
        return;
    };

    flow.go_next_floor = false;
    flow.go_victory = false;
    flow.mode = RewardFlowMode::SingleBuff;
    flow.reward_scale = 1.0;
    flow.selected_primary = None;
    flow.selected_secondary = None;

    if let (Some(layout), Some(current)) = (layout.as_deref(), current.as_deref()) {
        if ev.room == current.0 {
            if let Some(room) = layout.room(current.0) {
                if room.room_type == RoomType::Reward {
                    return;
                }
                if room.room_type != RoomType::Normal
                    && room.room_type != RoomType::Boss
                    && room.room_type != RoomType::Puzzle
                {
                    return;
                }
                if room.room_type == RoomType::Boss {
                    flow.mode = RewardFlowMode::DualBuff;
                    flow.reward_scale = 1.50;
                    let boss_gives_victory = data
                        .as_deref()
                        .map(|d| d.balance.boss_room_gives_victory)
                        .unwrap_or(false);
                    let current_floor = floor.as_deref().map(|value| value.0).unwrap_or(1);
                    let reached_final_floor = data
                        .as_deref()
                        .map(|registry| is_final_floor(registry, current_floor))
                        .unwrap_or(current_floor >= 4);

                    if let Ok((_, mut health)) = player_q.get_single_mut() {
                        let heal = health.max * 0.80;
                        health.current = (health.current + heal).min(health.max);
                    }

                    if boss_gives_victory || reached_final_floor {
                        flow.go_victory = true;
                    } else {
                        flow.go_next_floor = true;
                    }
                } else {
                    flow.mode = RewardFlowMode::HealOrBuff;
                }
            }
        }
    }

    let (mods, _) = player_q
        .get_single()
        .map(|(mods, health)| (*mods, *health))
        .unwrap_or((
            RewardModifiers::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));
    match flow.mode {
        RewardFlowMode::DualBuff => {
            let (primary, secondary) = generate_dual_reward_choices(&mut rng, mods);
            choices.primary = primary;
            choices.secondary = secondary;
        }
        RewardFlowMode::HealOrBuff | RewardFlowMode::SingleBuff => {
            choices.primary = generate_reward_choices(&mut rng, mods, &[]);
            choices.secondary.clear();
        }
    }
    next_state.set(AppState::RewardSelect);
}

fn generate_reward_choices(
    rng: &mut GameRng,
    mods: RewardModifiers,
    excluded: &[RewardType],
) -> Vec<RewardType> {
    let mut pool = reward_pool()
        .into_iter()
        .filter(|reward| !mods.reward_at_max(*reward) && !excluded.contains(reward))
        .collect::<Vec<_>>();
    if pool.len() < 3 {
        pool = reward_pool()
            .into_iter()
            .filter(|reward| !excluded.contains(reward))
            .collect::<Vec<_>>();
    }
    rng.shuffle(&mut pool);
    pool.truncate(3);
    pool
}

fn generate_dual_reward_choices(
    rng: &mut GameRng,
    mods: RewardModifiers,
) -> (Vec<RewardType>, Vec<RewardType>) {
    let primary = generate_reward_choices(rng, mods, &[]);
    let mut secondary = generate_reward_choices(rng, mods, &primary);
    if secondary.len() < 3 {
        secondary = generate_reward_choices(rng, mods, &[]);
    }
    (primary, secondary)
}

fn reward_pool() -> Vec<RewardType> {
    vec![
        RewardType::EnhanceMeleeWeapon,
        RewardType::IncreaseAttackSpeed,
        RewardType::IncreaseAttackPower,
        RewardType::IncreaseMaxHealth,
        RewardType::ReduceDashCooldown,
        RewardType::LifeStealOnKill,
        RewardType::IncreaseCritChance,
        RewardType::IncreaseMoveSpeed,
        RewardType::DashDamageTrail,
        RewardType::EnhanceRangedWeapon,
    ]
}

fn handle_reward_choice_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<RewardChosenEvent>,
    choices: Res<RewardChoices>,
    flow: Res<RewardFlow>,
) {
    let mapped = match flow.mode {
        RewardFlowMode::SingleBuff => map_reward_key(
            &keyboard,
            RewardChoiceGroup::Primary,
            &choices.primary,
            [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3],
            [KeyCode::Numpad1, KeyCode::Numpad2, KeyCode::Numpad3],
        ),
        RewardFlowMode::HealOrBuff => {
            if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
                Some((RewardChoiceGroup::Heal, RewardType::RecoverHealth))
            } else {
                map_reward_key(
                    &keyboard,
                    RewardChoiceGroup::Primary,
                    &choices.primary,
                    [KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4],
                    [KeyCode::Numpad2, KeyCode::Numpad3, KeyCode::Numpad4],
                )
            }
        }
        RewardFlowMode::DualBuff => map_reward_key(
            &keyboard,
            RewardChoiceGroup::Primary,
            &choices.primary,
            [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3],
            [KeyCode::Numpad1, KeyCode::Numpad2, KeyCode::Numpad3],
        )
        .or_else(|| {
            map_reward_key(
                &keyboard,
                RewardChoiceGroup::Secondary,
                &choices.secondary,
                [KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6],
                [KeyCode::Numpad4, KeyCode::Numpad5, KeyCode::Numpad6],
            )
        }),
    };

    if let Some((group, reward)) = mapped {
        events.send(RewardChosenEvent { reward, group });
    }
}

fn apply_reward_choice(
    mut chosen: EventReader<RewardChosenEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut flow: ResMut<RewardFlow>,
    mut choices: ResMut<RewardChoices>,
    mut player_q: Query<
        (
            &mut RewardModifiers,
            &mut Health,
            &mut MoveSpeed,
            &mut DashCooldown,
            &mut RangedCooldown,
            &mut CritChance,
            &mut AttackCooldown,
            &mut AttackPower,
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
        let choice_valid = match flow.mode {
            RewardFlowMode::SingleBuff => ev.group == RewardChoiceGroup::Primary,
            RewardFlowMode::HealOrBuff => {
                ev.group == RewardChoiceGroup::Heal || ev.group == RewardChoiceGroup::Primary
            }
            RewardFlowMode::DualBuff => match ev.group {
                RewardChoiceGroup::Primary => flow.selected_primary.is_none(),
                RewardChoiceGroup::Secondary => flow.selected_secondary.is_none(),
                RewardChoiceGroup::Heal => false,
            },
        };
        if !choice_valid {
            continue;
        }

        let floor_number = floor.as_deref().map(|value| value.0).unwrap_or(1);
        if let Ok((
            mut mods,
            mut health,
            mut move_speed,
            mut dash_cd,
            mut ranged_cd,
            mut crit,
            mut atk_cd,
            mut attack_power,
        )) = player_q.get_single_mut()
        {
            apply_reward_to_player_components(
                ev.reward,
                floor_number,
                if ev.reward == RewardType::RecoverHealth {
                    1.0
                } else {
                    flow.reward_scale
                },
                &mut mods,
                &mut health,
                &mut move_speed,
                &mut dash_cd,
                &mut ranged_cd,
                &mut crit,
                &mut atk_cd,
                &mut attack_power,
            );
        } else {
            warn!("奖励已选择，但没有找到玩家实体。");
        }

        match flow.mode {
            RewardFlowMode::SingleBuff | RewardFlowMode::HealOrBuff => {}
            RewardFlowMode::DualBuff => match ev.group {
                RewardChoiceGroup::Primary => flow.selected_primary = Some(ev.reward),
                RewardChoiceGroup::Secondary => flow.selected_secondary = Some(ev.reward),
                RewardChoiceGroup::Heal => {}
            },
        }

        if flow.mode == RewardFlowMode::DualBuff
            && (flow.selected_primary.is_none() || flow.selected_secondary.is_none())
        {
            continue;
        }

        flow.mode = RewardFlowMode::SingleBuff;
        flow.reward_scale = 1.0;
        flow.selected_primary = None;
        flow.selected_secondary = None;
        choices.primary.clear();
        choices.secondary.clear();

        if flow.go_next_floor {
            for (entity, player) in &ingame_entities {
                if player.is_none() {
                    safe_despawn_recursive(&mut commands, entity);
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

fn map_reward_key(
    keyboard: &ButtonInput<KeyCode>,
    group: RewardChoiceGroup,
    choices: &[RewardType],
    digits: [KeyCode; 3],
    numpads: [KeyCode; 3],
) -> Option<(RewardChoiceGroup, RewardType)> {
    for (idx, reward) in choices.iter().copied().enumerate() {
        if keyboard.just_pressed(digits[idx]) || keyboard.just_pressed(numpads[idx]) {
            return Some((group, reward));
        }
    }
    None
}
