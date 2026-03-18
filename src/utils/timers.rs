use bevy::prelude::*;

pub fn tick_timer(timer: &mut Timer, time: &Time) -> bool {
    timer.tick(time.delta());
    timer.just_finished()
}
