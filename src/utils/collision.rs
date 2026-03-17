use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Aabb2 {
    pub center: Vec2,
    pub half_size: Vec2,
}

impl Aabb2 {
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self {
            center,
            half_size: size * 0.5,
        }
    }

    pub fn intersects(self, other: Aabb2) -> bool {
        let delta = (self.center - other.center).abs();
        delta.x <= (self.half_size.x + other.half_size.x) && delta.y <= (self.half_size.y + other.half_size.y)
    }
}

pub fn aabb_from_transform_size(transform: &GlobalTransform, size: Vec2) -> Aabb2 {
    let scale = transform.compute_transform().scale.truncate();
    Aabb2::from_center_size(transform.translation().truncate(), size * scale)
}
