use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(bevy::prelude::Resource, Debug, Clone)]
pub struct GameRng {
    rng: StdRng,
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

impl GameRng {
    pub fn from_entropy() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    pub fn reseed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }

    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        self.rng.gen_range(min..max)
    }

    pub fn gen_bool(&mut self, probability: f32) -> bool {
        self.rng.gen_bool(probability.clamp(0.0, 1.0) as f64)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.rng);
    }
}
