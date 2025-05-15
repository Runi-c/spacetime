use bevy::{
    prelude::*,
    render::camera::Viewport,
    window::{PrimaryWindow, WindowResized},
};

use crate::{layers::SpaceLayer, scheduling::Sets};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, camera_setup.in_set(Sets::Init))
        .add_systems(Update, camera_resize.in_set(Sets::PreUpdate));
}

#[derive(Component, Clone)]
#[require(Camera2d)]
pub struct SpaceCamera;

fn camera_setup(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let resolution = window.physical_size();

    commands.spawn((
        Name::new("Space Camera"),
        SpaceCamera,
        SpaceLayer,
        Camera {
            order: 1,
            viewport: Some(Viewport {
                physical_size: uvec2(resolution.x / 2, resolution.y),
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

fn camera_resize(
    mut resize_events: EventReader<WindowResized>,
    window: Single<&Window>,
    mut space_camera: Single<&mut Camera, With<SpaceCamera>>,
) {
    for _ in resize_events.read() {
        space_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });
    }
}
