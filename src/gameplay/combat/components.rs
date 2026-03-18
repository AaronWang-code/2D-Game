use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Player,
    Enemy,
    Pvp1,
    Pvp2,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Hurtbox {
    pub team: Team,
    pub size: Vec2,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Hitbox {
    pub owner: Option<Entity>,
    pub team: Team,
    pub size: Vec2,
    pub damage: f32,
    pub knockback: f32,
    pub can_crit: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub team: Team,
    pub velocity: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct Lifetime(pub Timer);

#[derive(Component, Debug, Clone, Copy)]
pub struct Knockback(pub Vec2);
