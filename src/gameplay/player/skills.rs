use bevy::prelude::*;

use crate::core::input::PlayerInputState;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::combat::components::Team;
use crate::gameplay::combat::projectiles;
use crate::gameplay::player::components::{AttackPower, Energy, Player, Skill1Cooldown};

pub fn player_skill1_input_system(
    mut commands: Commands,
    input: Res<PlayerInputState>,
    time: Res<Time>,
    data: Option<Res<GameDataRegistry>>,
    assets: Res<crate::core::assets::GameAssets>,
    mut q: Query<(&GlobalTransform, &AttackPower, &mut Energy, &mut Skill1Cooldown), With<Player>>,
) {
    let Ok((tf, power, mut energy, mut cd)) = q.get_single_mut() else { return };

    cd.timer.tick(time.delta());
    if !input.skill1_pressed || !cd.timer.finished() {
        return;
    }

    let cfg = data.as_deref().map(|d| &d.player);
    let cost = cfg.map(|c| c.skill1_energy_cost).unwrap_or(45.0);
    if energy.current < cost {
        return;
    }
    energy.current = (energy.current - cost).max(0.0);
    cd.timer.reset();

    let pos = tf.translation().truncate();
    let speed = 820.0;
    let damage = power.0 * 1.35;

    // 8-way high damage burst.
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        let dir = Vec2::new(a.cos(), a.sin());
        projectiles::spawn_projectile(
            &mut commands,
            &assets,
            Team::Player,
            pos + dir * 18.0,
            dir * speed,
            damage,
        );
    }
}

