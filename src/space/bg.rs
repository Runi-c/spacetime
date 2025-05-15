use bevy::prelude::*;

use crate::{
    layers::SpaceLayer,
    materials::{Dither, DitherMaterial},
    scheduling::Sets,
    z_order::ZOrder,
    SCREEN_SIZE,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, space_bg_spawn.in_set(Sets::Spawn));
}

fn space_bg_spawn(
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
            scale: SCREEN_SIZE.x,
            ..default()
        })),
        ZOrder::BACKGROUND,
    ));
}
