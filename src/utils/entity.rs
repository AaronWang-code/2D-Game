use bevy::ecs::world::World;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Commands, Entity};

pub fn safe_despawn_recursive(commands: &mut Commands, entity: Entity) {
    commands.add(move |world: &mut World| {
        if world.get_entity(entity).is_some() {
            world.entity_mut(entity).despawn_recursive();
        }
    });
}
