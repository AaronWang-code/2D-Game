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

pub fn choose_enemy_types(data: &GameDataRegistry) -> Vec<EnemyType> {
    if !data.balance.enemy_types.is_empty() {
        return data.balance.enemy_types.clone();
    }
    vec![EnemyType::MeleeChaser, EnemyType::RangedShooter, EnemyType::Charger]
}

pub fn pick_enemy_type(rng: &mut GameRng, pool: &[EnemyType]) -> EnemyType {
    let i = (rng.gen_range_f32(0.0, pool.len() as f32) as usize).min(pool.len() - 1);
    pool[i]
}

