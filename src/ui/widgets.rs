use bevy::prelude::*;

use crate::core::assets::GameAssets;

pub fn root_node() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..default()
    }
}

pub fn panel_node(color: Color) -> NodeBundle {
    NodeBundle {
        style: Style {
            padding: UiRect::all(Val::Px(18.0)),
            row_gap: Val::Px(10.0),
            column_gap: Val::Px(10.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: BackgroundColor(color),
        ..default()
    }
}

pub fn title_text(assets: &GameAssets, text: impl Into<String>, size: f32) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font: assets.font.clone(),
            font_size: size,
            color: Color::WHITE,
        },
    )
}

pub fn button_bundle() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(260.0),
            height: Val::Px(48.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::srgb(0.18, 0.22, 0.30)),
        ..default()
    }
}

