use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::achievements::{AchievementId, Achievements};
use crate::gameplay::enemy::systems::EnemySpawnCount;
use crate::gameplay::player::components::{
    AttackCooldown, AttackPower, CritChance, DashCooldown, ENERGY_SYSTEM_ENABLED, Energy, Gold,
    Health, MoveSpeed, Player, RangedCooldown, RewardModifiers,
};
use crate::gameplay::progression::floor::FloorNumber;
use crate::states::AppState;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingLoad>()
            .add_systems(Update, (save_hotkey_system, load_hotkey_system).chain())
            .add_systems(
                Update,
                apply_pending_load
                    .run_if(in_state(AppState::InGame))
                    .after(load_hotkey_system),
            );
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PendingLoad(pub Option<SaveData>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub floor: u32,
    pub player: PlayerSave,
    pub enemy_spawn_count: Option<u32>,
    pub achievements: Vec<AchievementId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
    pub hp_current: f32,
    pub hp_max: f32,
    pub energy_current: f32,
    pub energy_max: f32,
    pub gold: u32,
    pub move_speed: f32,
    pub attack_power: f32,
    pub crit_chance: f32,
    pub rewards: RewardModifiers,
    pub attack_cd_s: f32,
    pub dash_cd_s: f32,
    pub ranged_cd_s: f32,
}

fn save_path() -> PathBuf {
    PathBuf::from("saves").join("run_save.ron")
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create dir {}", parent.display()))?;
    }
    Ok(())
}

fn write_save_file(save: &SaveData) -> Result<()> {
    let path = save_path();
    ensure_parent_dir(&path)?;
    let pretty = ron::ser::PrettyConfig::new().depth_limit(4);
    let content = ron::ser::to_string_pretty(save, pretty).context("serialize save to ron")?;
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_save_file() -> Result<SaveData> {
    let path = save_path();
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    ron::from_str::<SaveData>(&content).with_context(|| format!("parse ron {}", path.display()))
}

fn save_hotkey_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    floor: Option<Res<FloorNumber>>,
    spawn_count: Option<Res<EnemySpawnCount>>,
    achievements: Option<Res<Achievements>>,
    player_q: Query<
        (
            &Health,
            &Energy,
            &Gold,
            &MoveSpeed,
            &AttackPower,
            &CritChance,
            &RewardModifiers,
            &AttackCooldown,
            &DashCooldown,
            &RangedCooldown,
        ),
        With<Player>,
    >,
) {
    if !keyboard.just_pressed(KeyCode::F5) {
        return;
    }
    let Ok((hp, energy, gold, move_speed, attack_power, crit, rewards, atk_cd, dash_cd, ranged_cd)) =
        player_q.get_single()
    else {
        warn!("未找到玩家实体，无法存档（F5 仅在游戏内可用）");
        return;
    };

    let save = SaveData {
        version: 1,
        floor: floor.as_deref().map(|f| f.0).unwrap_or(1),
        player: PlayerSave {
            hp_current: hp.current,
            hp_max: hp.max,
            energy_current: energy.current,
            energy_max: energy.max,
            gold: gold.0,
            move_speed: move_speed.0,
            attack_power: attack_power.0,
            crit_chance: crit.0,
            rewards: *rewards,
            attack_cd_s: atk_cd.base_duration_s,
            dash_cd_s: dash_cd.base_duration_s,
            ranged_cd_s: ranged_cd.base_duration_s,
        },
        enemy_spawn_count: spawn_count.as_deref().map(|s| s.current),
        achievements: achievements
            .as_deref()
            .map(|a| a.unlocked.iter().copied().collect())
            .unwrap_or_default(),
    };

    if let Err(err) = write_save_file(&save) {
        warn!("存档失败：{err:?}");
    } else {
        info!("已存档到 {:?}", save_path());
    }
}

fn load_hotkey_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingLoad>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !keyboard.just_pressed(KeyCode::F9) {
        return;
    }

    match read_save_file() {
        Ok(save) => {
            pending.0 = Some(save);
            if *state.get() != AppState::InGame {
                next.set(AppState::InGame);
            }
        }
        Err(err) => warn!("读档失败：{err:?}"),
    }
}

fn apply_pending_load(
    mut commands: Commands,
    mut pending: ResMut<PendingLoad>,
    mut floor: Option<ResMut<FloorNumber>>,
    mut spawn_count: Option<ResMut<EnemySpawnCount>>,
    achievements: Option<ResMut<Achievements>>,
    mut player_q: Query<
        (
            &mut Health,
            &mut Energy,
            &mut Gold,
            &mut MoveSpeed,
            &mut AttackPower,
            &mut CritChance,
            &mut RewardModifiers,
            &mut AttackCooldown,
            &mut DashCooldown,
            &mut RangedCooldown,
        ),
        With<Player>,
    >,
) {
    let Some(save) = pending.0.take() else {
        return;
    };

    let floor_value = save.floor.max(1);
    match floor.as_mut() {
        Some(floor) => floor.0 = floor_value,
        None => commands.insert_resource(FloorNumber(floor_value)),
    }
    if let Some(v) = save.enemy_spawn_count {
        match spawn_count.as_mut() {
            Some(spawn_count) => spawn_count.current = v,
            None => commands.insert_resource(EnemySpawnCount { current: v }),
        }
    }
    if let Some(mut achievements) = achievements {
        achievements.unlocked.clear();
        achievements
            .unlocked
            .extend(save.achievements.iter().copied());
    }

    let Ok((
        mut hp,
        mut energy,
        mut gold,
        mut move_speed,
        mut attack_power,
        mut crit,
        mut rewards,
        mut atk_cd,
        mut dash_cd,
        mut ranged_cd,
    )) = player_q.get_single_mut()
    else {
        pending.0 = Some(save);
        return;
    };

    hp.max = save.player.hp_max.max(1.0);
    hp.current = save.player.hp_current.clamp(0.0, hp.max);
    energy.max = save.player.energy_max.max(0.0);
    energy.current = if ENERGY_SYSTEM_ENABLED {
        save.player.energy_current.clamp(0.0, energy.max)
    } else {
        energy.max
    };
    gold.0 = save.player.gold;
    move_speed.0 = save.player.move_speed.max(0.0);
    attack_power.0 = save.player.attack_power.max(0.0);
    crit.0 = save.player.crit_chance.clamp(0.0, 1.0);
    *rewards = save.player.rewards;

    *atk_cd = AttackCooldown::new(save.player.attack_cd_s.max(0.05));
    *dash_cd = DashCooldown::new(save.player.dash_cd_s.max(0.05));
    *ranged_cd = RangedCooldown::new(save.player.ranged_cd_s.max(0.05));
    atk_cd.apply_speed_bonus(rewards.total_melee_speed_bonus());
    dash_cd.apply_reduction(rewards.total_dash_cooldown_reduction());
    ranged_cd.apply_speed_bonus(rewards.total_ranged_speed_bonus());

    info!("读档完成：楼层 {}", floor_value);
}
