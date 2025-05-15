use bevy::{asset::weak_handle, prelude::*};
use lyon_tessellation::{
    geom::Box2D,
    path::{builder::BorderRadii, Winding},
};

use crate::{factory::grid::TILE_SIZE, mesh::MeshLyonExtensions};

pub const INLET_MESH: Handle<Mesh> = weak_handle!("00197959-e60b-4cad-baff-c1af5262890b");
pub const INLET_MESH_INNER: Handle<Mesh> = weak_handle!("b578feab-2a46-45a5-9478-0c01b2d13f81");
pub const CONSTRUCTOR_MESH: Handle<Mesh> = weak_handle!("4ff209b3-3fcf-46ea-8449-15223fe12b50");
pub const CONSTRUCTOR_MESH_INNER: Handle<Mesh> =
    weak_handle!("7470eec1-4214-45db-a106-ef0bf961d9f2");

pub(super) fn plugin(app: &mut App) {
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    meshes.insert(INLET_MESH.id(), Circle::new(TILE_SIZE * 0.4 + 2.0).into());
    meshes.insert(INLET_MESH_INNER.id(), Circle::new(TILE_SIZE * 0.4).into());
    meshes.insert(
        CONSTRUCTOR_MESH.id(),
        Mesh::fill_with(|builder| {
            builder.add_rounded_rectangle(
                &Box2D::zero().inflate(TILE_SIZE * 0.4 + 2.0, TILE_SIZE * 0.4 + 2.0),
                &BorderRadii::new(5.0),
                Winding::Positive,
            );
        }),
    );
    meshes.insert(
        CONSTRUCTOR_MESH_INNER.id(),
        Mesh::fill_with(|builder| {
            builder.add_rounded_rectangle(
                &Box2D::zero().inflate(TILE_SIZE * 0.4, TILE_SIZE * 0.4),
                &BorderRadii::new(5.0),
                Winding::Positive,
            );
        }),
    );
}
