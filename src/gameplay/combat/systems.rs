use bevy::prelude::*;

use crate::states::AppState;

use super::{damage, hitbox, projectiles};

pub struct CombatSystemsPlugin;

impl Plugin for CombatSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                hitbox::reflect_enemy_projectiles_on_melee,
                hitbox::detect_hitbox_hurtbox_overlap,
                hitbox::tick_rupture_dots,
                projectiles::move_projectiles,
                damage::apply_damage_events,
                damage::apply_knockback_decay,
                projectiles::despawn_out_of_room_projectiles,
                projectiles::despawn_expired_projectiles,
                hitbox::despawn_expired_hitboxes,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}
