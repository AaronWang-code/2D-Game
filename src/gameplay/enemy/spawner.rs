use bevy::prelude::*;

use crate::data::registry::GameDataRegistry;
use crate::gameplay::enemy::components::EnemyType;
use crate::utils::rng::GameRng;

pub fn get_spawn_points_for_room() -> Vec<Vec2> {
    vec![
        Vec2::new(200.0, 120.0),
        Vec2::new(260.0, -90.0),
        Vec2::new(-60.0, 160.0),
        Vec2::new(30.0, -140.0),
        Vec2::new(120.0, 30.0),
        Vec2::new(-150.0, -60.0),
    ]
}

pub fn choose_enemy_types(data: &GameDataRegistry, floor: u32) -> Vec<EnemyType> {
    if !data.balance.enemy_types.is_empty() {
        return data.balance.enemy_types.clone();
    }
    let mut pool = vec![
        EnemyType::MeleeChaser,
        EnemyType::RangedShooter,
        EnemyType::Charger,
    ];
    if floor >= 2 {
        pool.push(EnemyType::Flanker);
    }
    if floor >= 3 {
        pool.push(EnemyType::Sniper);
    }
    if floor >= 4 {
        pool.push(EnemyType::SupportCaster);
    }
    pool
}

pub fn frontline_enemy_types(pool: &[EnemyType]) -> Vec<EnemyType> {
    let filtered = pool
        .iter()
        .copied()
        .filter(|enemy_type| {
            matches!(
                enemy_type,
                EnemyType::MeleeChaser | EnemyType::Charger | EnemyType::Flanker
            )
        })
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        pool.to_vec()
    } else {
        filtered
    }
}

pub fn backline_enemy_types(pool: &[EnemyType]) -> Vec<EnemyType> {
    let filtered = pool
        .iter()
        .copied()
        .filter(|enemy_type| {
            matches!(
                enemy_type,
                EnemyType::RangedShooter | EnemyType::Sniper | EnemyType::SupportCaster
            )
        })
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        pool.to_vec()
    } else {
        filtered
    }
}

pub fn pick_enemy_type(rng: &mut GameRng, pool: &[EnemyType]) -> EnemyType {
    if pool.is_empty() {
        return EnemyType::MeleeChaser;
    }
    let i = (rng.gen_range_f32(0.0, pool.len() as f32) as usize).min(pool.len() - 1);
    pool[i]
}
