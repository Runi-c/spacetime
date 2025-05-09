use bevy::prelude::*;

use crate::{scheduling::Sets, SCREEN_SIZE};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_tooltip.in_set(Sets::Spawn))
        .add_systems(Update, observe_tooltips.in_set(Sets::Input));
}

#[derive(Component)]
#[require(Pickable)]
pub struct Tooltip(pub String);

#[derive(Resource)]
struct ActiveTooltip {
    container: Entity,
    text: Entity,
}

fn spawn_tooltip(mut commands: Commands) {
    let container = commands
        .spawn((
            Name::new("Tooltip Container"),
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(Color::BLACK),
            BorderColor(Color::WHITE),
        ))
        .id();
    let text = commands
        .spawn((
            Name::new("Tooltip Text"),
            Node::default(),
            Text("".to_string()),
            Visibility::Inherited,
            ChildOf { parent: container },
        ))
        .id();
    commands.insert_resource(ActiveTooltip { container, text });
}

fn observe_tooltips(mut commands: Commands, tooltip_query: Query<Entity, Added<Tooltip>>) {
    for entity in tooltip_query.iter() {
        info!("Tooltip entity: {:?}", entity);
        commands
            .entity(entity)
            .observe(update_tooltip)
            .observe(remove_tooltip);
    }
}

fn update_tooltip(
    trigger: Trigger<Pointer<Move>>,
    mut commands: Commands,
    tooltips: Query<&Tooltip>,
    active_tooltip: Res<ActiveTooltip>,
) {
    let target = trigger.target();
    if let Ok(entity) = tooltips.get(target) {
        let tooltip = entity.0.clone();
        // Update the tooltip text and position
        let position = trigger.pointer_location.position;
        commands
            .entity(active_tooltip.text)
            .insert(Text(tooltip.clone()));
        commands.entity(active_tooltip.container).insert((
            Visibility::Visible,
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(5.0)),
                left: Val::Px(position.x + 5.0),
                bottom: Val::Px(SCREEN_SIZE.y - position.y + 5.0),
                ..default()
            },
        ));
    } else {
        // Hide the tooltip if no entity has a Tooltip component
        commands.entity(target).insert(Visibility::Hidden);
    }
}

fn remove_tooltip(
    _trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    active_tooltip: Res<ActiveTooltip>,
) {
    commands
        .entity(active_tooltip.container)
        .insert(Visibility::Hidden);
}
