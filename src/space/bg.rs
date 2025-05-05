use bevy::prelude::*;

use crate::{dither::DitherMaterial, layers::Layers, z_order::ZOrder};
pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_space_bg);
}

fn spawn_space_bg(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    const SIZE: f32 = 2000.0;
    commands.spawn((
        Layers::SPACE,
        Mesh2d(meshes.add(Rectangle::new(SIZE, SIZE))),
        MeshMaterial2d(materials.add(DitherMaterial {
            fill: 0.001,
            dither_scale: 5.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, ZOrder::BACKGROUND),
    ));
}
