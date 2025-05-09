use bevy::prelude::*;

use crate::{
    layers::SpaceLayer,
    materials::{Dither, DitherMaterial},
    scheduling::Sets,
    z_order::ZOrder,
};
pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_space_bg.in_set(Sets::Spawn));
}

fn spawn_space_bg(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    const SIZE: f32 = 2000.0;
    commands.spawn((
        SpaceLayer,
        Mesh2d(meshes.add(Rectangle::new(SIZE, SIZE))),
        MeshMaterial2d(materials.add(Dither {
            fill: 0.001,
            scale: 5.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, ZOrder::BACKGROUND),
    ));
}
