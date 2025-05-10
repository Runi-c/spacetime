use bevy::prelude::*;

use crate::{scheduling::Sets, SCREEN_SIZE};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_tooltip.in_set(Sets::Spawn))
        .add_systems(Update, observe_tooltips.in_set(Sets::Input));
}

#[derive(Component, Clone)]
#[require(Pickable)]
pub struct Tooltip(pub String, pub Option<String>);

#[derive(Resource)]
struct ActiveTooltip {
    container: Entity,
    title: Entity,
    description: Entity,
}

fn spawn_tooltip(mut commands: Commands) {
    let container = commands
        .spawn((
            Name::new("Tooltip Container"),
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(Color::BLACK),
            BorderColor(Color::WHITE),
        ))
        .id();
    let title = commands
        .spawn((
            Name::new("Tooltip Text"),
            Node::default(),
            Text("".to_string()),
            Visibility::Inherited,
            ChildOf { parent: container },
        ))
        .id();
    let description = commands
        .spawn((
            Name::new("Tooltip Description"),
            Node::default(),
            Text("".to_string()),
            TextFont::from_font_size(12.0),
            Visibility::Inherited,
            ChildOf { parent: container },
        ))
        .id();
    commands.insert_resource(ActiveTooltip {
        container,
        title,
        description,
    });
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
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.any_pressed([MouseButton::Left, MouseButton::Right]) {
        commands
            .entity(active_tooltip.container)
            .insert(Visibility::Hidden);
        return;
    }
    let target = trigger.target();
    if let Ok(tooltip) = tooltips.get(target) {
        let title = tooltip.0.clone();
        // Update the tooltip text and position
        let position = trigger.pointer_location.position;
        commands
            .entity(active_tooltip.title)
            .insert(Text(title.clone()));
        commands.entity(active_tooltip.container).insert((
            Visibility::Visible,
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                left: if position.x > (3.0 * SCREEN_SIZE.x / 4.0) {
                    Val::Auto
                } else {
                    Val::Px(position.x + 5.0)
                },
                right: if position.x > (3.0 * SCREEN_SIZE.x / 4.0) {
                    Val::Px(SCREEN_SIZE.x - position.x + 5.0)
                } else {
                    Val::Auto
                },
                bottom: Val::Px(SCREEN_SIZE.y - position.y + 5.0),
                ..default()
            },
        ));
        if let Some(description) = &tooltip.1 {
            commands.entity(active_tooltip.description).insert((
                Text(description.clone()),
                Node {
                    display: Display::Block,
                    ..default()
                },
            ));
        } else {
            commands.entity(active_tooltip.description).insert(Node {
                display: Display::None,
                ..default()
            });
        }
    } else {
        warn!("Tooltip entity not found: {:?}", target);
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
