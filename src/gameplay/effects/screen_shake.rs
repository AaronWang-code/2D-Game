use bevy::prelude::*;
use rand::Rng;
use rand::SeedableRng;

#[derive(Event, Debug, Clone, Copy)]
pub struct ScreenShakeRequest {
    pub strength: f32,
    pub duration: f32,
}

#[derive(Resource, Debug, Default)]
pub struct ScreenShake {
    remaining: f32,
    strength: f32,
    seed: u64,
}

impl ScreenShake {
    pub fn trigger(&mut self, strength: f32, duration: f32) {
        self.strength = self.strength.max(strength);
        self.remaining = self.remaining.max(duration);
        self.seed = self.seed.wrapping_add(1);
    }

    pub fn update(&mut self, dt: f32) -> Vec2 {
        if self.remaining <= 0.0 {
            self.strength = 0.0;
            return Vec2::ZERO;
        }
        self.remaining = (self.remaining - dt).max(0.0);
        let t = self.remaining.max(0.001);
        let falloff = (t / (t + dt)).clamp(0.0, 1.0);
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed ^ (t.to_bits() as u64));
        let offset =
            Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)) * self.strength;
        offset * falloff
    }
}
