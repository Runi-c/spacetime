use bevy::{
    prelude::*,
    render::camera::Viewport,
    window::{PrimaryWindow, WindowResized},
};

use crate::{layers::SpaceLayer, scheduling::Sets};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera.in_set(Sets::Init))
        .add_systems(Update, resize_camera.in_set(Sets::Input));
}

#[derive(Component, Clone)]
#[require(Camera2d)]
pub struct SpaceCamera;

fn setup_camera(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
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

fn resize_camera(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut space_camera: Single<&mut Camera, With<SpaceCamera>>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        space_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });
    }
}
