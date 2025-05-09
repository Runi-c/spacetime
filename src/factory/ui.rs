use bevy::prelude::*;

use crate::{
    resources::{ResourceType, Resources},
    scheduling::Sets,
    SCREEN_SIZE,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ui.in_set(Sets::Spawn))
        .add_systems(Update, update_ui.in_set(Sets::PostUpdate));
}

#[derive(Component)]
pub struct ResourceDisplay(pub ResourceType);

fn spawn_ui(mut commands: Commands) {
    const DISPLAY_WIDTH: f32 = 200.0;
    commands.spawn((
        Name::new("Resource UI"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(SCREEN_SIZE.x / 2.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        children![
            (
                Name::new("Minerals Display"),
                ResourceDisplay(ResourceType::Mineral),
                Text::new("Minerals: 0"),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
            (
                Name::new("Gas Display"),
                ResourceDisplay(ResourceType::Gas),
                Text::new("Gas: 0"),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
            (
                Name::new("Time Display"),
                ResourceDisplay(ResourceType::Time),
                Text::new("Time: 0"),
                Node {
                    width: Val::Px(DISPLAY_WIDTH),
                    ..default()
                }
            ),
        ],
    ));
}

fn update_ui(
    mut commands: Commands,
    resources: Res<Resources>,
    query: Query<(Entity, &ResourceDisplay)>,
) {
    if !resources.is_changed() {
        return;
    }
    for (entity, resource_display) in query.iter() {
        let amount = resources.get(resource_display.0);
        commands.entity(entity).insert(Text::new(format!(
            "{}: {}",
            resource_display.0.to_string(),
            amount
        )));
    }
}
