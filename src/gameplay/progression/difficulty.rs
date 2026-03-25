use crate::data::registry::GameDataRegistry;

pub fn get_floor_difficulty_multiplier(data: &GameDataRegistry, floor: u32) -> f32 {
    1.0 + (floor.saturating_sub(1) as f32) * data.balance.difficulty_per_floor
}

pub fn get_floor_enemy_count(data: &GameDataRegistry, floor: u32) -> u32 {
    match floor {
        0 | 1 => 4,
        2 => 5,
        3 => 5,
        _ => 6,
    }
}

pub fn is_final_floor(data: &GameDataRegistry, floor: u32) -> bool {
    floor >= data.balance.total_floors.max(1)
}
