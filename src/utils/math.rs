use bevy::prelude::*;

pub fn clamp_length(vec: Vec2, max: f32) -> Vec2 {
    let len = vec.length();
    if len > max && len > 0.0 {
        vec / len * max
    } else {
        vec
    }
}

pub fn direction_to(from: Vec2, to: Vec2) -> Vec2 {
    let delta = to - from;
    if delta.length_squared() <= f32::EPSILON {
        Vec2::ZERO
    } else {
        delta.normalize()
    }
}

pub fn clamp_in_room(pos: Vec2, room_half_size: Vec2, margin: f32) -> Vec2 {
    let min = -room_half_size + Vec2::splat(margin);
    let max = room_half_size - Vec2::splat(margin);
    Vec2::new(pos.x.clamp(min.x, max.x), pos.y.clamp(min.y, max.y))
}
