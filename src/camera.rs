use bevy::prelude::*;

use crate::{layers::UILayer, scheduling::Sets};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera.in_set(Sets::Init));
}

#[derive(Component, Clone)]
#[require(Camera2d)]
pub struct UICamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("UI Camera"),
        UICamera,
        UILayer,
        Camera {
            order: 2,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0),
    ));
}
