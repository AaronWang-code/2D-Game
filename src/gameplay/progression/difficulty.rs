use crate::data::registry::GameDataRegistry;

pub fn get_floor_difficulty_multiplier(data: &GameDataRegistry, floor: u32) -> f32 {
    1.0 + (floor.saturating_sub(1) as f32) * data.balance.difficulty_per_floor
}

