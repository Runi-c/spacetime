use bevy::{prelude::*, render::camera::Viewport};

use crate::{layers::SpaceLayer, scheduling::Sets, SCREEN_SIZE};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera.in_set(Sets::Init));
}

#[derive(Component, Clone)]
#[require(Camera2d)]
pub struct SpaceCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Space Camera"),
        SpaceCamera,
        SpaceLayer,
        Camera {
            order: 1,
            viewport: Some(Viewport {
                physical_size: uvec2((SCREEN_SIZE.x / 2.0) as u32, SCREEN_SIZE.y as u32),
                ..default()
            }),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scale: 2.0,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, -100.0),
    ));
}
