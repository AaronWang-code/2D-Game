use bevy::prelude::*;

use crate::constants::UI_Z;
use crate::core::assets::GameAssets;
use crate::core::events::DamageAppliedEvent;
use crate::gameplay::combat::components::Team;
use crate::gameplay::map::InGameEntity;

#[derive(Component, Debug, Clone)]
pub struct DamageNumber {
    pub timer: Timer,
    pub velocity: Vec2,
}

pub fn update_damage_numbers(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut ev: EventReader<DamageAppliedEvent>,
    mut q: Query<(Entity, &mut DamageNumber, &mut Transform, &mut Text)>,
) {
    let Some(assets) = assets else { return };

    for e in ev.read() {
        let color = match e.attacker_team {
            Team::Player => Color::srgb(0.85, 1.0, 0.85),
            Team::Enemy => Color::srgb(1.0, 0.75, 0.75),
            Team::Pvp1 => Color::srgb(0.85, 1.0, 0.85),
            Team::Pvp2 => Color::srgb(1.0, 0.75, 0.75),
        };
        let text_color = if e.is_crit {
            Color::srgb(1.0, 0.95, 0.35)
        } else {
            color
        };

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("{:.0}", e.amount.max(0.0)),
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: if e.is_crit { 28.0 } else { 22.0 },
                        color: text_color,
                    },
                ),
                transform: Transform::from_translation((e.pos + Vec2::new(0.0, 18.0)).extend(UI_Z - 6.0)),
                ..default()
            },
            DamageNumber {
                timer: Timer::from_seconds(0.75, TimerMode::Once),
                velocity: Vec2::new(0.0, 80.0),
            },
            InGameEntity,
            Name::new("DamageNumber"),
        ));
    }

    for (entity, mut dmg, mut tf, mut text) in &mut q {
        dmg.timer.tick(time.delta());
        tf.translation += (dmg.velocity * time.delta_seconds()).extend(0.0);
        let t = dmg.timer.fraction();
        let alpha = (1.0 - t).clamp(0.0, 1.0);
        if let Some(section) = text.sections.get_mut(0) {
            section.style.color.set_alpha(alpha);
        }
        if dmg.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
