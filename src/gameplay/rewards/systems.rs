use bevy::prelude::*;

use crate::core::events::{RewardChosenEvent, RoomClearedEvent};
use crate::data::registry::GameDataRegistry;
use crate::gameplay::enemy::systems::{ClearGrace, SpawnedForRoom};
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::map::transitions::RoomTransition;
use crate::gameplay::map::InGameEntity;
use crate::gameplay::player::components::{
    AttackCooldown, CritChance, DashCooldown, Health, MoveSpeed, Player, RewardModifiers,
};
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
            .add_systems(OnEnter(AppState::RewardSelect), crate::ui::reward_select::setup_reward_ui)
            .add_systems(
                Update,
                (
                    handle_reward_choice_input,
                    crate::ui::reward_select::reward_ui_input_system,
                    crate::ui::reward_select::update_reward_ui,
                )
                    .run_if(in_state(AppState::RewardSelect)),
            )
            .add_systems(OnExit(AppState::RewardSelect), crate::ui::reward_select::cleanup_reward_ui)
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
) {
    let Some(ev) = room_cleared.read().next() else { return };

    flow.go_next_floor = false;
    flow.go_victory = false;
    if let (Some(layout), Some(current)) = (layout.as_deref(), current.as_deref()) {
        if ev.room == current.0 {
            if let Some(room) = layout.room(current.0) {
                if room.room_type == RoomType::Boss {
                    let boss_gives_victory = data
                        .as_deref()
                        .map(|d| d.balance.boss_room_gives_victory)
                        .unwrap_or(true);
                    if boss_gives_victory {
                        // Boss 通关：选完奖励进入胜利结算。
                        flow.go_victory = true;
                    } else {
                        // Boss 通关：选完奖励进入下一关（下一楼层）。
                        flow.go_next_floor = true;
                    }
                }
            }
        }
    }

    choices.choices = generate_reward_choices(&mut *rng);
    next_state.set(AppState::RewardSelect);
}

fn generate_reward_choices(rng: &mut GameRng) -> Vec<RewardType> {
    let mut pool = vec![
        RewardType::IncreaseAttackSpeed,
        RewardType::IncreaseMaxHealth,
        RewardType::ReduceDashCooldown,
        RewardType::LifeStealOnKill,
        RewardType::IncreaseCritChance,
        RewardType::IncreaseMoveSpeed,
        RewardType::DashDamageTrail,
        RewardType::BonusProjectile,
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
    let Some(reward) = choices.choices.get(i).copied() else { return };
    events.send(RewardChosenEvent { reward });
}

fn apply_reward_choice(
    mut chosen: EventReader<RewardChosenEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    mut flow: ResMut<RewardFlow>,
    mut player_q: Query<(
        &mut RewardModifiers,
        &mut Health,
        &mut MoveSpeed,
        &mut DashCooldown,
        &mut CritChance,
        &mut AttackCooldown,
    ), With<Player>>,
    mut commands: Commands,
    ingame_entities: Query<Entity, With<InGameEntity>>,
    mut floor: Option<ResMut<FloorNumber>>,
    mut spawned_for_room: ResMut<SpawnedForRoom>,
    mut grace: ResMut<ClearGrace>,
) {
    for ev in chosen.read() {
        if let Ok((mut mods, mut health, mut move_speed, mut dash_cd, mut crit, mut atk_cd)) =
            player_q.get_single_mut()
        {
            apply_reward_to_player_components(
                ev.reward,
                &mut mods,
                &mut health,
                &mut move_speed,
                &mut dash_cd,
                &mut crit,
                &mut atk_cd,
            );
        } else {
            warn!("奖励已选择，但未找到玩家实体；仍然返回游戏以避免卡住");
        }

        if flow.go_next_floor {
            // 清理场景实体，并强制下一次进入 InGame 时重新生成楼层布局。
            for e in &ingame_entities {
                commands.entity(e).despawn_recursive();
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
