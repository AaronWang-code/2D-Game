use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;

use crate::core::assets::GameAssets;
use crate::gameplay::player::components::{Energy, Gold, Health, Player};
use crate::ui::widgets;

use super::components::CoopPlayer;
use super::net::{CoopNetConfig, CoopNetState, CoopPlayerStateMsg, NetMode};
use super::ui::{
    CoopClientStatusText, CoopHostMateEnergyFill, CoopHostMateEnergyText, CoopHostMateGoldText, CoopHostMateHealthFill,
    CoopHostMateHealthText, CoopHostOverlayUi, CoopHostStatusText, CoopLocalEnergyFill, CoopLocalEnergyText, CoopLocalGoldText,
    CoopLocalHealthFill, CoopLocalHealthText,
};

pub fn setup_coop_host_overlay(mut commands: Commands, assets: Res<GameAssets>, config: Res<CoopNetConfig>) {
    if config.mode != NetMode::Host {
        return;
    }

    commands
        .spawn((widgets::root_node(), CoopHostOverlayUi, Name::new("CoopHostOverlayRoot")))
        .with_children(|root| {
            root.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    top: Val::Px(12.0),
                    row_gap: Val::Px(8.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .with_children(|col| {
                col.spawn((widgets::title_text(&assets, "联机状态", 18.0), CoopHostStatusText));
                col.spawn(widgets::body_text(&assets, "左上角为房主自身 HUD", 15.0));
            });

            spawn_status_panel(
                root,
                &assets,
                "队友状态",
                UiRect {
                    right: Val::Px(16.0),
                    top: Val::Px(70.0),
                    ..default()
                },
                CoopHostMateHealthText,
                CoopHostMateHealthFill,
                CoopHostMateEnergyText,
                CoopHostMateEnergyFill,
                CoopHostMateGoldText,
                "CoopHostMatePanel",
            );
        });
}

pub fn cleanup_coop_host_overlay(mut commands: Commands, q: Query<Entity, With<CoopHostOverlayUi>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

pub fn update_coop_client_panels(
    net: Res<CoopNetState>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<CoopClientStatusText>>,
        Query<&mut Text, With<CoopLocalHealthText>>,
        Query<&mut Text, With<CoopLocalEnergyText>>,
        Query<&mut Text, With<CoopLocalGoldText>>,
    )>,
    mut style_sets: ParamSet<(
        Query<&mut Style, With<CoopLocalHealthFill>>,
        Query<&mut Style, With<CoopLocalEnergyFill>>,
    )>,
) {
    if let Some(snapshot) = net.last_snapshot.as_ref() {
        let me = if net.my_id == Some(1) {
            (&snapshot.p1, &snapshot.p2)
        } else {
            (&snapshot.p2, &snapshot.p1)
        }
        .0;
        set_text_value(&mut text_sets.p0(), "合作模式");
        set_text_value(&mut text_sets.p1(), &format!("生命：{:.0}/{:.0}", me.hp, me.hp_max));
        set_style_width(&mut style_sets.p0(), ratio(me.hp, me.hp_max));
        set_text_value(&mut text_sets.p2(), &format!("能量：{:.0}/{:.0}", me.energy, me.energy_max));
        set_style_width(&mut style_sets.p1(), ratio(me.energy, me.energy_max));
        set_text_value(&mut text_sets.p3(), &format!("金币：{}", me.gold));
    } else {
        set_text_value(&mut text_sets.p0(), "合作模式：等待同步");
        set_text_value(&mut text_sets.p1(), "生命：0/0");
        set_style_width(&mut style_sets.p0(), 0.0);
        set_text_value(&mut text_sets.p2(), "能量：0/0");
        set_style_width(&mut style_sets.p1(), 0.0);
        set_text_value(&mut text_sets.p3(), "金币：0");
    }
}

pub fn update_coop_host_overlay(
    config: Res<CoopNetConfig>,
    net: Res<CoopNetState>,
    host_q: Query<&Gold, With<Player>>,
    mate_q: Query<(&Health, &Energy, Option<&Gold>), With<CoopPlayer>>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<CoopHostStatusText>>,
        Query<&mut Text, With<CoopHostMateHealthText>>,
        Query<&mut Text, With<CoopHostMateEnergyText>>,
        Query<&mut Text, With<CoopHostMateGoldText>>,
    )>,
    mut style_sets: ParamSet<(
        Query<&mut Style, With<CoopHostMateHealthFill>>,
        Query<&mut Style, With<CoopHostMateEnergyFill>>,
    )>,
) {
    if config.mode != NetMode::Host {
        return;
    }

    let status_text = if net.connected {
        "联机状态：已连接"
    } else {
        "联机状态：等待队友加入"
    };
    set_text_value(&mut text_sets.p0(), status_text);

    if let Ok((hp, energy, gold)) = mate_q.get_single() {
        let fallback_gold = host_q.get_single().ok().map(|g| g.0).unwrap_or(0);
        let mate = CoopPlayerStateMsg {
            id: 2,
            pos: (0.0, 0.0),
            hp: hp.current,
            hp_max: hp.max,
            energy: energy.current,
            energy_max: energy.max,
            gold: gold.map(|g| g.0).unwrap_or(fallback_gold),
        };
        set_text_value(&mut text_sets.p1(), &format!("生命：{:.0}/{:.0}", mate.hp, mate.hp_max));
        set_style_width(&mut style_sets.p0(), ratio(mate.hp, mate.hp_max));
        set_text_value(&mut text_sets.p2(), &format!("能量：{:.0}/{:.0}", mate.energy, mate.energy_max));
        set_style_width(&mut style_sets.p1(), ratio(mate.energy, mate.energy_max));
        set_text_value(&mut text_sets.p3(), &format!("金币：{}", mate.gold));
    } else {
        set_text_value(&mut text_sets.p1(), "生命：0/0");
        set_style_width(&mut style_sets.p0(), 0.0);
        set_text_value(&mut text_sets.p2(), "能量：0/0");
        set_style_width(&mut style_sets.p1(), 0.0);
        set_text_value(&mut text_sets.p3(), "金币：等待加入");
    }
}

fn spawn_status_panel<HealthTextMarker, HealthFillMarker, EnergyTextMarker, EnergyFillMarker, GoldTextMarker>(
    root: &mut ChildBuilder,
    assets: &GameAssets,
    title: &str,
    position: UiRect,
    health_text_marker: HealthTextMarker,
    health_fill_marker: HealthFillMarker,
    energy_text_marker: EnergyTextMarker,
    energy_fill_marker: EnergyFillMarker,
    gold_text_marker: GoldTextMarker,
    name: &str,
) where
    HealthTextMarker: Bundle,
    HealthFillMarker: Bundle,
    EnergyTextMarker: Bundle,
    EnergyFillMarker: Bundle,
    GoldTextMarker: Bundle,
{
    root.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: position.left,
                right: position.right,
                top: position.top,
                bottom: position.bottom,
                width: Val::Px(260.0),
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.72)),
            ..default()
        },
        Name::new(name.to_string()),
    ))
    .with_children(|panel| {
        panel.spawn(widgets::title_text(assets, title, 18.0));
        panel.spawn((widgets::body_text(assets, "生命：0/0", 16.0), health_text_marker));
        panel.spawn(NodeBundle {
            style: Style {
                width: Val::Px(236.0),
                height: Val::Px(16.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.20, 0.85, 0.30)),
                    ..default()
                },
                health_fill_marker,
            ));
        });

        panel.spawn((widgets::body_text(assets, "能量：0/0", 16.0), energy_text_marker));
        panel.spawn(NodeBundle {
            style: Style {
                width: Val::Px(236.0),
                height: Val::Px(14.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.25, 0.65, 0.95)),
                    ..default()
                },
                energy_fill_marker,
            ));
        });

        panel.spawn((widgets::body_text(assets, "金币：0", 16.0), gold_text_marker));
    });
}

fn apply_player_panel(
    player: &CoopPlayerStateMsg,
    hp_text: &mut Text,
    hp_fill: &mut Style,
    energy_text: &mut Text,
    energy_fill: &mut Style,
    gold_text: &mut Text,
) {
    hp_text.sections[0].value = format!("生命：{:.0}/{:.0}", player.hp, player.hp_max);
    hp_fill.width = Val::Percent(ratio(player.hp, player.hp_max) * 100.0);
    energy_text.sections[0].value = format!("能量：{:.0}/{:.0}", player.energy, player.energy_max);
    energy_fill.width = Val::Percent(ratio(player.energy, player.energy_max) * 100.0);
    gold_text.sections[0].value = format!("金币：{}", player.gold);
}

fn reset_player_panel(
    hp_text: &mut Text,
    hp_fill: &mut Style,
    energy_text: &mut Text,
    energy_fill: &mut Style,
    gold_text: &mut Text,
) {
    hp_text.sections[0].value = "生命：0/0".to_string();
    hp_fill.width = Val::Percent(0.0);
    energy_text.sections[0].value = "能量：0/0".to_string();
    energy_fill.width = Val::Percent(0.0);
    gold_text.sections[0].value = "金币：0".to_string();
}

fn set_text_value<Filter: QueryFilter>(query: &mut Query<&mut Text, Filter>, value: &str) {
    let Ok(mut text) = query.get_single_mut() else { return };
    text.sections[0].value = value.to_string();
}

fn set_style_width<Filter: QueryFilter>(query: &mut Query<&mut Style, Filter>, value: f32) {
    let Ok(mut style) = query.get_single_mut() else { return };
    style.width = Val::Percent(value.clamp(0.0, 1.0) * 100.0);
}

fn ratio(current: f32, max: f32) -> f32 {
    if max > 0.0 {
        (current / max).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
