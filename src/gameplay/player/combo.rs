use bevy::prelude::*;

use crate::core::events::DamageAppliedEvent;
use crate::gameplay::combat::components::Team;
use crate::gameplay::player::components::{Combo, Player};

pub fn update_combo_state(
    time: Res<Time>,
    mut damage_applied: EventReader<DamageAppliedEvent>,
    mut player_q: Query<(Entity, &mut Combo), With<Player>>,
) {
    let Ok((player_e, mut combo)) = player_q.get_single_mut() else {
        return;
    };

    combo.timer.tick(time.delta());
    if combo.timer.finished() {
        combo.count = 0;
    }

    for ev in damage_applied.read() {
        if ev.attacker_team == Team::Player {
            combo.count = combo.count.saturating_add(1);
            combo.timer.reset();
        }
        if ev.attacker_team == Team::Enemy && ev.target == player_e {
            combo.count = 0;
            combo.timer.reset();
        }
    }
}
