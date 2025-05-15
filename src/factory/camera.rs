use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

use crate::{layers::FactoryLayer, scheduling::Sets, SCREEN_SIZE};

use super::grid::{GRID_WIDTH, TILE_SIZE};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, camera_setup.in_set(Sets::Init));
}

#[derive(Component, Clone)]
#[require(Camera2d)]
pub struct FactoryCamera;

fn camera_setup(mut commands: Commands) {
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

#[derive(SystemParam)]
pub struct CursorPosition<'w, 's> {
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<FactoryCamera>>,
}

impl<'w, 's> CursorPosition<'w, 's> {
    pub fn world(&self) -> Option<Vec2> {
        if let (Ok(window), Ok((camera, camera_transform))) =
            (self.window.single(), self.camera.single())
        {
            window.cursor_position().and_then(|screen_pos| {
                camera
                    .viewport_to_world_2d(camera_transform, screen_pos)
                    .ok()
            })
        } else {
            None
        }
    }
    pub fn tile(&self) -> Option<IVec2> {
        self.world().map(|pos| {
            ((pos + Vec2::splat(GRID_WIDTH * 0.5)) / TILE_SIZE)
                .floor()
                .as_ivec2()
        })
    }
}
