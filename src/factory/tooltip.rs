use bevy::prelude::*;

use crate::{scheduling::Sets, SCREEN_SIZE};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, tooltip_spawn.in_set(Sets::Spawn))
        .add_systems(Update, tooltip_observers.in_set(Sets::Input));
}

#[derive(Component, Clone)]
pub struct Tooltip(pub String, pub Option<String>);

#[derive(Resource)]
struct ActiveTooltip {
    container: Entity,
    title: Entity,
    description: Entity,
}

fn tooltip_spawn(mut commands: Commands) {
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
            ChildOf(container),
        ))
        .id();
    let description = commands
        .spawn((
            Name::new("Tooltip Description"),
            Node::default(),
            Text("".to_string()),
            TextFont::from_font_size(12.0),
            Visibility::Inherited,
            ChildOf(container),
        ))
        .id();
    commands.insert_resource(ActiveTooltip {
        container,
        title,
        description,
    });
}

fn tooltip_observers(
    mut commands: Commands,
    tooltip_query: Query<Entity, Added<Tooltip>>,
    names: Query<&Name>,
) {
    for entity in tooltip_query.iter() {
        info!("Tooltip entity: {:?}", names.get(entity));
        commands
            .entity(entity)
            .observe(tooltip_update)
            .observe(tooltip_hide);
    }
}

fn tooltip_update(
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

fn tooltip_hide(
    _trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    active_tooltip: Res<ActiveTooltip>,
) {
    commands
        .entity(active_tooltip.container)
        .insert(Visibility::Hidden);
}
