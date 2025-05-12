use bevy::prelude::*;

use crate::{
    resources::{ResourceType, Resources},
    scheduling::Sets,
    SCREEN_SIZE,
};

use super::machines::{Buffer, Inlet};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ui.in_set(Sets::Spawn))
        .add_systems(
            Update,
            (update_ui, hide_time_warning).in_set(Sets::PostUpdate),
        );
}

#[derive(Component, Clone)]
pub struct ResourceDisplay(pub ResourceType);

#[derive(Component, Clone)]
pub struct OutOfTimeThing;

fn spawn_ui(mut commands: Commands) {
    const DISPLAY_WIDTH: f32 = 120.0;
    commands.spawn((
        Name::new("Resource UI"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(SCREEN_SIZE.x / 2.0),
            top: Val::ZERO,
            bottom: Val::ZERO,
            right: Val::ZERO,
            flex_direction: FlexDirection::Row,
            border: UiRect::left(Val::Px(1.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Start,
            column_gap: Val::Px(10.0),
            ..default()
        },
        Pickable::IGNORE,
        BorderColor(Color::WHITE),
        children![
            (
                Name::new("Minerals Display"),
                ResourceDisplay(ResourceType::Mineral),
                Text::new("Minerals: 0"),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
            (
                Name::new("Gas Display"),
                ResourceDisplay(ResourceType::Gas),
                Text::new("Gas: 0"),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
            (
                Name::new("Time Display"),
                ResourceDisplay(ResourceType::Time),
                Text::new("Time: 0"),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
            (
                Name::new("Ammo Display"),
                ResourceDisplay(ResourceType::Ammo),
                Text::new("Ammo: 0"),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
            (
                Name::new("Rockets Display"),
                ResourceDisplay(ResourceType::Rockets),
                Text::new("Rockets: 0"),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
        ],
    ));
    commands.spawn((
        Name::new("Out of Time Marker"),
        OutOfTimeThing,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(SCREEN_SIZE.x / 2.0),
            top: Val::Px(50.0),
            bottom: Val::ZERO,
            right: Val::ZERO,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Start,
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Name::new("Out of Time Text"),
            Text::new("Factory in Stasis!\nCollect more time."),
            TextLayout::new_with_justify(JustifyText::Center),
        ),],
    ));
}

fn hide_time_warning(
    mut commands: Commands,
    query: Query<Entity, With<OutOfTimeThing>>,
    resources: Res<Resources>,
) {
    if resources.get(ResourceType::Time) > 0.0 {
        for entity in query.iter() {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    } else {
        for entity in query.iter() {
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
}

fn update_ui(
    mut commands: Commands,
    resources: Res<Resources>,
    query: Query<(Entity, &ResourceDisplay)>,
    inlets: Query<&Buffer, With<Inlet>>,
) {
    if !resources.is_changed() {
        return;
    }
    for (entity, resource_display) in query.iter() {
        let mut amount = resources.get(resource_display.0);
        for buffer in inlets.iter() {
            if buffer.0 == resource_display.0 {
                amount += buffer.1;
            }
        }
        commands.entity(entity).insert(Text::new(format!(
            "{}: {}",
            resource_display.0.to_string(),
            amount.ceil()
        )));
    }
}
