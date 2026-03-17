use bevy::prelude::*;

#[derive(Resource, Debug, Default, Clone)]
pub struct RunStats {
    pub time_s: f32,
    pub kills: u32,
    pub damage_done: f32,
    pub damage_taken: f32,
}

pub fn update_run_stats(mut stats: Local<RunStats>, time: Res<Time>) {
    stats.time_s += time.delta_seconds();
}

