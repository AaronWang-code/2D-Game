use bevy::prelude::*;

use crate::core::events::{DamageEvent, DeathEvent, RoomClearedEvent};
use crate::data::definitions::EnemyStatsConfig;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::{Hitbox, Hurtbox, Knockback, Lifetime, Projectile, Team};
use crate::gameplay::combat::projectiles;
use crate::gameplay::effects::flash::Flash;
use crate::gameplay::effects::particles;
use crate::gameplay::enemy::{ai, boss, spawner};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomType};
use crate::gameplay::player::components::{Gold, Health, InvincibilityTimer, RewardModifiers};
use crate::gameplay::player::components::{Health as PlayerHealth, Player};
use crate::gameplay::progression::difficulty::{
    get_floor_difficulty_multiplier, get_floor_enemy_count,
};
use crate::gameplay::progression::floor::FloorNumber;
use crate::gameplay::puzzle::{
    ActivePuzzle, PuzzleEntity, reset_active_puzzle, spawn_puzzle_for_room,
};
use crate::gameplay::shop::ShopKiosk;
use crate::states::{AppState, RoomState};
use crate::utils::collision::aabb_from_transform_size;
use crate::utils::entity::safe_despawn_recursive;
use crate::utils::math::direction_to;
use crate::utils::rng::GameRng;

use super::components::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EnemyVelocity(pub Vec2);

pub struct EnemySystemsPlugin;

impl Plugin for EnemySystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameRng>()
            .insert_resource(EnemySpawnCount { current: 0 })
            .insert_resource(SpawnedForRoom::default())
            .insert_resource(ClearGrace::default())
            .add_systems(
                Update,
                room_entry_spawner.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                enemy_buff_decay_system.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                ai::update_enemy_ai.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                enemy_attack_system.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                boss::boss_phase_controller.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                boss::boss_attack_patterns.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                boss_contact_damage_system.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                enemy_death_system.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                clear_enemy_attacks_on_room_clear.run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub(crate) struct SpawnedForRoom(pub Option<u32>);

#[derive(Resource, Debug)]
pub(crate) struct ClearGrace {
    pub(crate) last_room: Option<u32>,
    pub(crate) timer: Timer,
}

impl Default for ClearGrace {
    fn default() -> Self {
        Self {
            last_room: None,
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct EnemySpawnCount {
    pub current: u32,
}

fn enemy_buff_decay_system(
    mut commands: Commands,
    time: Res<Time>,
    mut buffed_enemies: Query<(Entity, &mut EnemyBuffState)>,
) {
    for (entity, mut buff) in &mut buffed_enemies {
        buff.timer.tick(time.delta());
        if buff.timer.finished() {
            commands.entity(entity).remove::<EnemyBuffState>();
        }
    }
}

pub fn room_entry_spawner(
    mut commands: Commands,
    mut spawned: ResMut<SpawnedForRoom>,
    data: Res<GameDataRegistry>,
    assets: Res<crate::core::assets::GameAssets>,
    layout: Res<FloorLayout>,
    current_room: Res<CurrentRoom>,
    mut room_state: ResMut<RoomState>,
    mut cleanup_q: ParamSet<(
        Query<Entity, With<Enemy>>,
        Query<Entity, With<Projectile>>,
        Query<Entity, With<Hitbox>>,
        Query<Entity, With<ShopKiosk>>,
        Query<Entity, With<PuzzleEntity>>,
    )>,
    mut spawn_count: ResMut<EnemySpawnCount>,
    mut active_puzzle: ResMut<ActivePuzzle>,
    mut rng: ResMut<GameRng>,
    floor: Option<Res<FloorNumber>>,
) {
    if spawned.0 == Some(current_room.0.0) {
        return;
    }
    spawned.0 = Some(current_room.0.0);

    for entity in &cleanup_q.p0() {
        safe_despawn_recursive(&mut commands, entity);
    }
    for entity in &cleanup_q.p1() {
        safe_despawn_recursive(&mut commands, entity);
    }
    for entity in &cleanup_q.p2() {
        safe_despawn_recursive(&mut commands, entity);
    }
    for entity in &cleanup_q.p3() {
        safe_despawn_recursive(&mut commands, entity);
    }
    for entity in &cleanup_q.p4() {
        safe_despawn_recursive(&mut commands, entity);
    }
    reset_active_puzzle(&mut active_puzzle);

    let room = layout.room(current_room.0).unwrap();
    let floor_number = floor.as_deref().map(|value| value.0).unwrap_or(1);
    let mut floor_multiplier = get_floor_difficulty_multiplier(&data, floor_number);
    let base_enemy_count = get_floor_enemy_count(&data, floor_number);
    let coop_hp_mult = 1.0;

    match room.room_type {
        RoomType::Start | RoomType::Reward | RoomType::Shop => {
            *room_state = RoomState::Idle;
        }
        RoomType::Normal => {
            *room_state = RoomState::Locked;
            if spawn_count.current == 0 {
                spawn_count.current = base_enemy_count;
            }
            let mut enemy_count = spawn_count.current;
            if floor_number == 1 {
                if current_room.0.0 == 1 {
                    enemy_count = enemy_count.saturating_sub(1).max(3);
                    floor_multiplier *= 0.86;
                } else if current_room.0.0 == 2 {
                    floor_multiplier *= 0.93;
                }
            }
            spawn_room_enemies(
                &mut commands,
                &assets,
                &data,
                enemy_count,
                floor_multiplier,
                floor_number,
                coop_hp_mult,
            );
        }
        RoomType::Boss => {
            *room_state = RoomState::BossFight;
            spawn_boss(
                &mut commands,
                &assets,
                &data,
                floor_number,
                floor_multiplier,
                coop_hp_mult,
            );
        }
        RoomType::Puzzle => {
            *room_state = RoomState::Locked;
            spawn_puzzle_for_room(
                &mut commands,
                &assets,
                &mut rng,
                &mut active_puzzle,
                current_room.0,
            );
        }
    }
}

pub fn spawn_room_enemies(
    commands: &mut Commands,
    assets: &crate::core::assets::GameAssets,
    data: &GameDataRegistry,
    enemy_count: u32,
    floor_multiplier: f32,
    floor_number: u32,
    coop_hp_mult: f32,
) {
    let points = spawner::get_spawn_points_for_room();
    let pool = spawner::choose_enemy_types(data, floor_number);
    let frontline_pool = spawner::frontline_enemy_types(&pool);
    let backline_pool = spawner::backline_enemy_types(&pool);
    let count = enemy_count as usize;
    let mut rng = GameRng::default();
    let spawn_n = count.min(points.len());
    let frontline_in_pool = !frontline_pool.is_empty();
    let backline_in_pool = !backline_pool.is_empty();

    let elite_chance = match floor_number {
        0..=2 => 0.0,
        3 => data.balance.elite_chance,
        _ => 0.32,
    };
    let elite_idx = if spawn_n > 0 && rng.gen_range_f32(0.0, 1.0) < elite_chance {
        Some((rng.gen_range_f32(0.0, spawn_n as f32) as usize).min(spawn_n - 1))
    } else {
        None
    };
    let mut planned_types = vec![None; spawn_n];
    if spawn_n > 0 && frontline_in_pool {
        planned_types[0] = Some(spawner::pick_enemy_type(&mut rng, &frontline_pool));
    }
    if spawn_n > 1 && backline_in_pool {
        planned_types[1] = Some(spawner::pick_enemy_type(&mut rng, &backline_pool));
    }
    if floor_number == 3 && spawn_n > 2 {
        let special_pool = pool
            .iter()
            .copied()
            .filter(|enemy_type| matches!(enemy_type, EnemyType::Flanker | EnemyType::Sniper))
            .collect::<Vec<_>>();
        if !special_pool.is_empty() {
            planned_types[2] = Some(spawner::pick_enemy_type(&mut rng, &special_pool));
        }
    } else if floor_number >= 4 {
        if spawn_n > 2 {
            let aggressive_pool = pool
                .iter()
                .copied()
                .filter(|enemy_type| matches!(enemy_type, EnemyType::Charger | EnemyType::Flanker))
                .collect::<Vec<_>>();
            if !aggressive_pool.is_empty() {
                planned_types[2] = Some(spawner::pick_enemy_type(&mut rng, &aggressive_pool));
            }
        }
        if spawn_n > 3 {
            let pressure_pool = pool
                .iter()
                .copied()
                .filter(|enemy_type| {
                    matches!(enemy_type, EnemyType::Sniper | EnemyType::SupportCaster)
                })
                .collect::<Vec<_>>();
            if !pressure_pool.is_empty() {
                planned_types[3] = Some(spawner::pick_enemy_type(&mut rng, &pressure_pool));
            }
        }
    }

    for i in 0..spawn_n {
        let enemy_type =
            planned_types[i].unwrap_or_else(|| spawner::pick_enemy_type(&mut rng, &pool));
        let is_elite = elite_idx == Some(i) && enemy_type != EnemyType::SupportCaster;
        spawn_enemy(
            commands,
            assets,
            data,
            enemy_type,
            points[i],
            floor_number,
            floor_multiplier,
            coop_hp_mult,
            is_elite,
        );
    }
}

pub fn spawn_enemy(
    commands: &mut Commands,
    assets: &crate::core::assets::GameAssets,
    data: &GameDataRegistry,
    enemy_type: EnemyType,
    pos: Vec2,
    floor_number: u32,
    floor_multiplier: f32,
    coop_hp_mult: f32,
    is_elite: bool,
) -> Entity {
    let stats_cfg = match enemy_type {
        EnemyType::MeleeChaser => &data.enemies.melee_chaser,
        EnemyType::RangedShooter => &data.enemies.ranged_shooter,
        EnemyType::Charger => &data.enemies.charger,
        EnemyType::Flanker => &data.enemies.flanker,
        EnemyType::Sniper => &data.enemies.sniper,
        EnemyType::SupportCaster => &data.enemies.support_caster,
        EnemyType::Boss => &data.enemies.melee_chaser,
    };
    let mut stats = scaled_enemy_stats(stats_cfg, enemy_type, floor_number, floor_multiplier);
    stats.max_hp *= coop_hp_mult.max(1.0);
    if is_elite && enemy_type != EnemyType::Boss {
        stats.max_hp *= data.balance.elite_hp_mult.max(1.0);
        stats.attack_damage *= data.balance.elite_damage_mult.max(1.0);
        stats.move_speed *= 1.12;
    }
    let color = enemy_color(enemy_type, is_elite);
    let sprite_size = if enemy_type == EnemyType::Boss {
        56.0
    } else if is_elite {
        34.0
    } else {
        match enemy_type {
            EnemyType::Charger => 30.0,
            EnemyType::SupportCaster => 30.0,
            EnemyType::Sniper => 27.0,
            EnemyType::Flanker => 24.0,
            _ => 28.0,
        }
    };
    let hurtbox_size = if enemy_type == EnemyType::Boss {
        60.0
    } else if is_elite {
        32.0
    } else {
        match enemy_type {
            EnemyType::Charger => 28.0,
            EnemyType::SupportCaster => 28.0,
            EnemyType::Sniper => 24.0,
            EnemyType::Flanker => 22.0,
            _ => 26.0,
        }
    };

    let mut entity = commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(pos.extend(45.0)),
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(sprite_size)),
                ..default()
            },
            ..default()
        },
        Enemy,
        EnemyKind(enemy_type),
        TeamMarker(Team::Enemy),
        Health {
            current: stats.max_hp,
            max: stats.max_hp,
        },
        stats,
        EnemyAttackCooldown {
            timer: Timer::from_seconds(stats.attack_cooldown_s, TimerMode::Once),
        },
        EnemyVelocity::default(),
        Hurtbox {
            team: Team::Enemy,
            size: Vec2::splat(hurtbox_size),
        },
        Flash::new(0.0),
        Knockback(Vec2::ZERO),
        InGameEntity,
        Name::new("Enemy"),
    ));

    if is_elite && enemy_type != EnemyType::Boss {
        entity.insert(Elite);
    }

    if enemy_type == EnemyType::Charger {
        entity.insert(ChargerState {
            phase: ChargerPhase::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            dir: Vec2::X,
        });
    }
    if enemy_type == EnemyType::Flanker {
        entity.insert(FlankerState {
            phase: FlankerPhase::Stalk,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            dir: Vec2::X,
            strafe_sign: 1.0,
            repath_timer: Timer::from_seconds(0.40, TimerMode::Once),
        });
    }
    if enemy_type == EnemyType::Sniper {
        entity.insert(SniperState {
            phase: SniperPhase::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            aim_dir: Vec2::X,
        });
    }

    entity.id()
}

pub fn spawn_boss(
    commands: &mut Commands,
    assets: &crate::core::assets::GameAssets,
    data: &GameDataRegistry,
    floor_number: u32,
    floor_multiplier: f32,
    coop_hp_mult: f32,
) -> Entity {
    let archetype = BossArchetype::from_floor(floor_number);
    let stats = scaled_boss_stats(data, archetype, floor_multiplier, floor_number);
    let stats = EnemyStats {
        max_hp: stats.max_hp * coop_hp_mult.max(1.0),
        ..stats
    };
    let hurtbox_size = if archetype == BossArchetype::CubeCore {
        66.0
    } else {
        60.0
    };
    let id = commands
        .spawn((
            SpriteBundle {
                texture: assets.textures.white.clone(),
                transform: Transform::from_translation(Vec3::new(220.0, 0.0, 45.0)),
                sprite: Sprite {
                    color: boss::boss_color(archetype),
                    custom_size: Some(Vec2::splat(if archetype == BossArchetype::CubeCore {
                        68.0
                    } else {
                        64.0
                    })),
                    ..default()
                },
                ..default()
            },
            Enemy,
            TeamMarker(Team::Enemy),
            Health {
                current: stats.max_hp,
                max: stats.max_hp,
            },
            stats,
            EnemyVelocity::default(),
            Hurtbox {
                team: Team::Enemy,
                size: Vec2::splat(hurtbox_size),
            },
            Flash::new(0.0),
            Knockback(Vec2::ZERO),
            InGameEntity,
            Name::new(boss::boss_name(archetype)),
        ))
        .id();
    let (kind, phase, timer, cycle) = boss::spawn_boss_bundle(data, archetype);
    commands
        .entity(id)
        .insert((kind, archetype, phase, timer, cycle));
    id
}

pub fn enemy_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<crate::core::assets::GameAssets>,
    player_q: Query<&GlobalTransform, With<Player>>,
    mut enemy_sets: ParamSet<(
        Query<(Entity, &EnemyKind, &GlobalTransform), With<Enemy>>,
        Query<
            (
                Entity,
                &EnemyKind,
                Option<&Elite>,
                &EnemyStats,
                &GlobalTransform,
                &mut EnemyAttackCooldown,
                Option<&EnemyBuffState>,
                Option<&mut SniperState>,
                Option<&mut Sprite>,
            ),
            With<Enemy>,
        >,
    )>,
) {
    let player_positions: Vec<Vec2> =
        player_q.iter().map(|tf| tf.translation().truncate()).collect();
    if player_positions.is_empty() {
        return;
    }
    let enemy_positions = enemy_sets
        .p0()
        .iter()
        .map(|(entity, kind, tf)| (entity, kind.0, tf.translation().truncate()))
        .collect::<Vec<_>>();

    for (entity, kind, elite, stats, tf, mut cd, buff, sniper_state, sprite) in &mut enemy_sets.p1()
    {
        let effective_cooldown = effective_enemy_attack_cooldown(stats.attack_cooldown_s, buff);
        cd.timer.tick(time.delta());
        let pos = tf.translation().truncate();
        let (player_pos, dist) = player_positions
            .iter()
            .map(|p| (*p, pos.distance(*p)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        let dir = direction_to(pos, player_pos);
        let mut sniper_state = sniper_state;
        let mut sprite = sprite;

        if let Some(sniper) = sniper_state.as_mut() {
            sniper.timer.tick(time.delta());
            match sniper.phase {
                SniperPhase::Aiming if sniper.timer.finished() => {
                    if let Some(sprite) = sprite.as_mut() {
                        sprite.color = enemy_color(kind.0, elite.is_some());
                    }
                    for (angle, damage_mult) in [(-0.16_f32, 0.75_f32), (0.0, 1.0), (0.16, 0.75)] {
                        let shot_dir = Mat2::from_angle(angle).mul_vec2(sniper.aim_dir);
                        projectiles::spawn_projectile(
                            &mut commands,
                            &assets,
                            Team::Enemy,
                            pos + shot_dir * 20.0,
                            shot_dir * stats.projectile_speed,
                            stats.attack_damage * damage_mult,
                        );
                    }
                    sniper.phase = SniperPhase::Recover;
                    sniper.timer = Timer::from_seconds(0.24, TimerMode::Once);
                    sniper.timer.reset();
                    cd.timer = Timer::from_seconds(effective_cooldown, TimerMode::Once);
                    cd.timer.reset();
                    continue;
                }
                SniperPhase::Recover if sniper.timer.finished() => {
                    if let Some(sprite) = sprite.as_mut() {
                        sprite.color = enemy_color(kind.0, elite.is_some());
                    }
                    sniper.phase = SniperPhase::Idle;
                }
                SniperPhase::Aiming | SniperPhase::Recover => continue,
                SniperPhase::Idle => {}
            }
        }

        if !cd.timer.finished() || dist > stats.aggro_range {
            continue;
        }

        match kind.0 {
            EnemyType::MeleeChaser | EnemyType::Charger | EnemyType::Flanker => {
                if dist <= stats.attack_range {
                    cd.timer = Timer::from_seconds(effective_cooldown, TimerMode::Once);
                    cd.timer.reset();
                    spawn_enemy_melee_hitbox(
                        &mut commands,
                        &assets,
                        pos,
                        dir,
                        if kind.0 == EnemyType::Flanker {
                            stats.attack_damage * 0.92
                        } else {
                            stats.attack_damage
                        },
                    );
                }
            }
            EnemyType::RangedShooter => {
                if dist <= stats.attack_range {
                    cd.timer = Timer::from_seconds(effective_cooldown, TimerMode::Once);
                    cd.timer.reset();
                    projectiles::spawn_projectile(
                        &mut commands,
                        &assets,
                        Team::Enemy,
                        pos + dir * 18.0,
                        dir * stats.projectile_speed,
                        stats.attack_damage,
                    );
                }
            }
            EnemyType::Sniper => {
                if dist <= stats.attack_range {
                    if let Some(sniper) = sniper_state.as_mut() {
                        sniper.phase = SniperPhase::Aiming;
                        sniper.timer = Timer::from_seconds(0.32, TimerMode::Once);
                        sniper.timer.reset();
                        sniper.aim_dir = dir;
                    }
                    if let Some(sprite) = sprite.as_mut() {
                        sprite.color = Color::srgb(1.0, 0.93, 0.62);
                    }
                }
            }
            EnemyType::SupportCaster => {
                let mut buffed_any = false;
                let buff_kind_toggle = entity.index() % 2 == 0;
                let mut candidates = enemy_positions
                    .iter()
                    .copied()
                    .filter(|(ally_entity, ally_kind, ally_pos)| {
                        *ally_entity != entity
                            && !matches!(ally_kind, EnemyType::Boss | EnemyType::SupportCaster)
                            && pos.distance(*ally_pos) <= stats.attack_range
                    })
                    .collect::<Vec<_>>();
                candidates.sort_by(|a, b| pos.distance(a.2).total_cmp(&pos.distance(b.2)));
                for (ally_entity, _, ally_pos) in candidates.into_iter().take(2) {
                    commands.entity(ally_entity).insert(EnemyBuffState {
                        speed_mult: if buff_kind_toggle { 1.35 } else { 1.0 },
                        cooldown_mult: if buff_kind_toggle { 1.0 } else { 1.40 },
                        timer: Timer::from_seconds(3.4, TimerMode::Once),
                    });
                    particles::spawn_hit_particles(
                        &mut commands,
                        &assets,
                        ally_pos,
                        Color::srgba(0.55, 0.95, 0.88, 0.82),
                    );
                    buffed_any = true;
                }
                cd.timer = Timer::from_seconds(
                    if buffed_any {
                        effective_cooldown
                    } else {
                        (effective_cooldown * 0.42).max(0.55)
                    },
                    TimerMode::Once,
                );
                cd.timer.reset();
            }
            EnemyType::Boss => {}
        }
    }
}

fn spawn_enemy_melee_hitbox(
    commands: &mut Commands,
    assets: &crate::core::assets::GameAssets,
    pos: Vec2,
    dir: Vec2,
    damage: f32,
) {
    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation((pos + dir * 24.0).extend(55.0)),
            sprite: Sprite {
                color: Color::srgba(1.0, 0.3, 0.25, 0.28),
                custom_size: Some(Vec2::new(40.0, 22.0)),
                ..default()
            },
            ..default()
        },
        Hitbox {
            owner: None,
            team: Team::Enemy,
            size: Vec2::new(40.0, 22.0),
            damage,
            knockback: 300.0,
            can_crit: false,
            crit_chance: 0.0,
            crit_multiplier: 1.0,
        },
        Lifetime(Timer::from_seconds(0.10, TimerMode::Once)),
        InGameEntity,
        Name::new("EnemyHitbox"),
    ));
}

pub fn enemy_death_system(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    mut room_cleared: EventWriter<RoomClearedEvent>,
    time: Res<Time>,
    assets: Res<crate::core::assets::GameAssets>,
    room_ctx: (Res<FloorLayout>, Res<CurrentRoom>, ResMut<RoomState>),
    mut player_q: Query<
        (
            &RewardModifiers,
            &mut PlayerHealth,
            &mut Gold,
            &GlobalTransform,
        ),
        With<Player>,
    >,
    enemy_info_q: Query<(&EnemyKind, Option<&Elite>)>,
    enemies_left: Query<Entity, With<Enemy>>,
    boss_summoned_q: Query<Entity, With<BossSummoned>>,
    mut grace: ResMut<ClearGrace>,
    mut spawn_count: ResMut<EnemySpawnCount>,
    data: Res<GameDataRegistry>,
    floor: Option<Res<FloorNumber>>,
) {
    let (layout, current_room, mut room_state) = room_ctx;
    for ev in death_events.read() {
        if ev.team != Team::Enemy {
            continue;
        }

        let (kind, is_elite) = enemy_info_q
            .get(ev.entity)
            .ok()
            .map(|(k, e)| (Some(k.0), e.is_some()))
            .unwrap_or((None, false));
        let floor_number = floor.as_deref().map(|f| f.0).unwrap_or(1);
        let base_gold = match kind {
            Some(EnemyType::Boss) => match floor_number {
                1 => 28,
                2 => 30,
                3 => 32,
                _ => 34,
            },
            _ => match floor_number {
                1 | 2 => 9,
                _ => 7,
            },
        };
        let reward_gold = base_gold
            + if is_elite {
                data.balance.elite_gold_bonus
            } else {
                0
            };

        if let Ok((mods, mut hp, mut gold, player_tf)) = player_q.get_single_mut() {
            gold.0 = gold.0.saturating_add(reward_gold);
            if mods.lifesteal_on_kill > 0.0 {
                let previous = hp.current;
                hp.current = (hp.current + mods.lifesteal_on_kill).min(hp.max);
                if hp.current > previous {
                    particles::spawn_hit_particles(
                        &mut commands,
                        &assets,
                        player_tf.translation().truncate(),
                        Color::srgba(0.42, 1.0, 0.52, 0.88),
                    );
                }
            }
        }

        safe_despawn_recursive(&mut commands, ev.entity);
        if matches!(kind, Some(EnemyType::Boss)) {
            for summoned in &boss_summoned_q {
                if summoned != ev.entity {
                    safe_despawn_recursive(&mut commands, summoned);
                }
            }
        }
    }

    if matches!(*room_state, RoomState::Locked | RoomState::BossFight) {
        let room = layout.room(current_room.0).unwrap();
        if room.room_type == RoomType::Puzzle {
            return;
        }
        if grace.last_room != Some(current_room.0.0) {
            grace.last_room = Some(current_room.0.0);
            grace.timer = Timer::from_seconds(0.20, TimerMode::Once);
            grace.timer.reset();
        }
        if !grace.timer.finished() {
            grace.timer.tick(time.delta());
            return;
        }

        let any_enemy_left = enemies_left.iter().next().is_some();
        if !any_enemy_left {
            *room_state = RoomState::Cleared;
            room_cleared.send(RoomClearedEvent {
                room: current_room.0,
            });
            if room.room_type == RoomType::Normal {
                let floor_number = floor.as_deref().map(|value| value.0).unwrap_or(1);
                let minimum = get_floor_enemy_count(&data, floor_number)
                    .saturating_sub(1)
                    .max(2);
                let next = spawn_count.current.saturating_sub(1);
                spawn_count.current = next.max(minimum);
            }
        }
    }
}

fn boss_contact_damage_system(
    mut damage_ev: EventWriter<DamageEvent>,
    boss_q: Query<(&EnemyStats, &Hurtbox, &GlobalTransform), With<BossArchetype>>,
    player_q: Query<
        (Entity, &Hurtbox, &GlobalTransform, Option<&InvincibilityTimer>),
        With<Player>,
    >,
) {
    for (boss_stats, boss_hurtbox, boss_tf) in &boss_q {
        let boss_aabb = aabb_from_transform_size(boss_tf, boss_hurtbox.size);
        let boss_pos = boss_tf.translation().truncate();
        for (entity, hurtbox, player_tf, inv) in &player_q {
            if inv.is_some_and(|timer| !timer.timer.finished()) {
                continue;
            }
            let player_aabb = aabb_from_transform_size(player_tf, hurtbox.size);
            if !boss_aabb.intersects(player_aabb) {
                continue;
            }

            let player_pos = player_tf.translation().truncate();
            damage_ev.send(DamageEvent {
                target: entity,
                source: None,
                amount: boss_stats.attack_damage * 0.45,
                knockback: direction_to(boss_pos, player_pos) * 120.0,
                team: Team::Enemy,
                is_crit: false,
            });
        }
    }
}

fn clear_enemy_attacks_on_room_clear(
    mut commands: Commands,
    mut room_cleared: EventReader<RoomClearedEvent>,
    enemy_attack_q: Query<(Entity, &Hitbox), Without<Enemy>>,
) {
    if room_cleared.read().next().is_none() {
        return;
    }

    for (entity, hitbox) in &enemy_attack_q {
        if hitbox.team == Team::Enemy {
            safe_despawn_recursive(&mut commands, entity);
        }
    }
}

fn scaled_enemy_stats(
    stats_cfg: &EnemyStatsConfig,
    enemy_type: EnemyType,
    floor_number: u32,
    floor_multiplier: f32,
) -> EnemyStats {
    let (floor_hp, floor_damage, floor_cooldown, floor_projectile) =
        floor_growth_curve(floor_number, floor_multiplier);
    let (type_hp, type_damage, type_cooldown, type_projectile, aggro_bonus) =
        enemy_type_curve(enemy_type, floor_number);
    EnemyStats {
        max_hp: stats_cfg.max_hp * floor_hp * type_hp,
        move_speed: stats_cfg.move_speed,
        attack_damage: stats_cfg.attack_damage * floor_damage * type_damage,
        attack_cooldown_s: (stats_cfg.attack_cooldown_s * floor_cooldown * type_cooldown).max(0.40),
        aggro_range: stats_cfg.aggro_range + aggro_bonus,
        attack_range: stats_cfg.attack_range,
        projectile_speed: stats_cfg.projectile_speed * floor_projectile * type_projectile,
    }
}

fn floor_growth_curve(floor_number: u32, floor_multiplier: f32) -> (f32, f32, f32, f32) {
    let base_step = if floor_number > 1 {
        (floor_multiplier - 1.0) / floor_number.saturating_sub(1) as f32
    } else {
        0.16
    };
    match floor_number {
        0 | 1 => (1.0, 1.0, 1.0, 1.0),
        2 => (
            1.0 + base_step * 0.625,
            1.0 + base_step * 0.50,
            (1.0 - base_step * 0.1875).max(0.5),
            1.0 + base_step * 0.3125,
        ),
        3 => (
            1.0 + base_step * 3.4375,
            1.0 + base_step * 1.25,
            (1.0 - base_step * 0.625).max(0.5),
            1.0 + base_step * 0.75,
        ),
        _ => (
            1.0 + base_step * 6.5625,
            1.0 + base_step * 2.375,
            (1.0 - base_step * 1.125).max(0.5),
            1.0 + base_step * 1.25,
        ),
    }
}

fn enemy_type_curve(enemy_type: EnemyType, floor_number: u32) -> (f32, f32, f32, f32, f32) {
    if floor_number < 3 {
        return (1.0, 1.0, 1.0, 1.0, 0.0);
    }

    match enemy_type {
        EnemyType::MeleeChaser => (1.08, 1.0, 1.0, 1.0, 0.0),
        EnemyType::RangedShooter => (1.15, 1.0, 0.96, 1.05, 0.0),
        EnemyType::Charger => (1.20, 1.08, 1.0, 1.0, 80.0),
        EnemyType::Flanker => (1.12, 1.10, 1.0, 1.0, 0.0),
        EnemyType::Sniper => (1.18, 1.12, 1.0, 1.0, 0.0),
        EnemyType::SupportCaster => {
            if floor_number >= 4 {
                (1.20, 1.0, 1.0, 1.0, 0.0)
            } else {
                (1.0, 1.0, 1.0, 1.0, 0.0)
            }
        }
        EnemyType::Boss => (1.0, 1.0, 1.0, 1.0, 0.0),
    }
}

pub fn effective_enemy_move_speed(stats: &EnemyStats, buff: Option<&EnemyBuffState>) -> f32 {
    stats.move_speed * buff.map(|value| value.speed_mult).unwrap_or(1.0)
}

fn effective_enemy_attack_cooldown(base_cooldown_s: f32, buff: Option<&EnemyBuffState>) -> f32 {
    let cooldown_mult = buff.map(|value| value.cooldown_mult).unwrap_or(1.0);
    (base_cooldown_s / cooldown_mult.max(0.2)).max(0.28)
}

fn enemy_color(enemy_type: EnemyType, is_elite: bool) -> Color {
    let base = match enemy_type {
        EnemyType::MeleeChaser => Color::srgb(0.95, 0.45, 0.45),
        EnemyType::RangedShooter => Color::srgb(0.55, 0.65, 0.95),
        EnemyType::Charger => Color::srgb(0.95, 0.75, 0.25),
        EnemyType::Flanker => Color::srgb(0.96, 0.56, 0.78),
        EnemyType::Sniper => Color::srgb(0.70, 0.82, 1.0),
        EnemyType::SupportCaster => Color::srgb(0.55, 0.95, 0.80),
        EnemyType::Boss => Color::srgb(0.85, 0.25, 0.95),
    };
    if !is_elite || matches!(enemy_type, EnemyType::Boss | EnemyType::SupportCaster) {
        return base;
    }
    match enemy_type {
        EnemyType::MeleeChaser => Color::srgb(1.0, 0.65, 0.65),
        EnemyType::RangedShooter => Color::srgb(0.75, 0.82, 1.0),
        EnemyType::Charger => Color::srgb(1.0, 0.88, 0.45),
        EnemyType::Flanker => Color::srgb(1.0, 0.68, 0.86),
        EnemyType::Sniper => Color::srgb(0.84, 0.90, 1.0),
        EnemyType::SupportCaster | EnemyType::Boss => base,
    }
}

fn scaled_boss_stats(
    data: &GameDataRegistry,
    archetype: BossArchetype,
    floor_multiplier: f32,
    floor_number: u32,
) -> EnemyStats {
    let scaling = (floor_multiplier - 1.0).max(0.0);
    let config = data.bosses.for_floor(floor_number);
    let hp_scaling = match archetype {
        BossArchetype::CubeCore => 0.72,
        _ => 0.38,
    };
    let base_range = match archetype {
        BossArchetype::Floor1Guardian => 42.0,
        BossArchetype::MirrorWarden => 44.0,
        BossArchetype::TideHunter => 52.0,
        BossArchetype::CubeCore => 48.0,
    };
    EnemyStats {
        max_hp: config.max_hp * (1.0 + scaling * hp_scaling),
        move_speed: config.move_speed * (1.0 + scaling * 0.08),
        attack_damage: config.contact_damage * (1.0 + scaling * 0.30),
        attack_cooldown_s: (0.95 / (1.0 + scaling * 0.12)).max(0.40),
        aggro_range: 900.0,
        attack_range: base_range,
        projectile_speed: config.projectile_speed * (1.0 + scaling * 0.12),
    }
}
