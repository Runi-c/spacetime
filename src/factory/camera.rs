use bevy::prelude::*;

use crate::{layers::FactoryLayer, scheduling::Sets, SCREEN_SIZE};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera.in_set(Sets::Spawn));
}

#[derive(Component)]
#[require(Camera2d)]
pub struct FactoryCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Factory Camera"),
        FactoryCamera,
        FactoryLayer,
        Camera {
            order: 0,
            ..default()
        },
        IsDefaultUiCamera,
        Transform::from_xyz(-SCREEN_SIZE.x / 4.0, 0.0, -100.0),
    ));
}
