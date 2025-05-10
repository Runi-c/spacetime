use bevy::{asset::weak_handle, ecs::spawn::SpawnIter, prelude::*};
use lyon_tessellation::{
    geom::{euclid::Point2D, Box2D},
    path::{builder::BorderRadii, Winding},
};

use crate::{
    layers::FactoryLayer,
    materials::{DitherMaterial, MetalDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::ResourceType,
    z_order::ZOrder,
};

use super::{
    grid::{Direction, TileCoords, TILE_SIZE},
    pipe::pipe_bridge,
};

pub const INLET_MESH: Handle<Mesh> = weak_handle!("00197959-e60b-4cad-baff-c1af5262890b");
pub const INLET_MESH_INNER: Handle<Mesh> = weak_handle!("b578feab-2a46-45a5-9478-0c01b2d13f81");
pub const CONSTRUCTOR_MESH: Handle<Mesh> = weak_handle!("4ff209b3-3fcf-46ea-8449-15223fe12b50");
pub const CONSTRUCTOR_MESH_INNER: Handle<Mesh> =
    weak_handle!("7470eec1-4214-45db-a106-ef0bf961d9f2");

pub fn plugin(app: &mut App) {
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    meshes.insert(
        INLET_MESH.id(),
        Mesh::fill_with(|builder| {
            builder.add_circle(Point2D::zero(), TILE_SIZE * 0.4 + 2.0, Winding::Positive);
        }),
    );
    meshes.insert(
        INLET_MESH_INNER.id(),
        Mesh::fill_with(|builder| {
            builder.add_circle(Point2D::zero(), TILE_SIZE * 0.4, Winding::Positive);
        }),
    );
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

#[derive(Component, Clone, Default)]
pub struct Machine;

#[derive(Clone, PartialEq, Eq)]
pub enum FlowDirection {
    Inlet,
    Outlet,
}

#[derive(Component, Clone)]
pub struct MachinePort {
    pub side: Direction,
    pub flow: FlowDirection,
}

#[derive(Component, Clone)]
#[require(Machine)]
pub struct Inlet(pub ResourceType);

pub fn inlet(
    resource: ResourceType,
    coords: IVec2,
    dir: Direction,
    material: Handle<DitherMaterial>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    (
        Inlet(resource),
        FactoryLayer,
        Mesh2d(INLET_MESH),
        MeshMaterial2d(SOLID_WHITE),
        TileCoords(coords),
        ZOrder::MACHINE,
        children![
            (
                Name::new("Inlet Inner"),
                FactoryLayer,
                Mesh2d(INLET_MESH_INNER),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.2),
            ),
            machine_port(
                MachinePort {
                    side: dir,
                    flow: FlowDirection::Outlet,
                },
                flow_material.clone(),
            ),
        ],
    )
}

#[derive(Component, Clone)]
#[require(Machine)]
pub struct Outlet(pub ResourceType);

pub fn outlet(
    resource: ResourceType,
    coords: IVec2,
    dir: Direction,
    material: Handle<DitherMaterial>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    (
        Inlet(resource),
        FactoryLayer,
        Mesh2d(INLET_MESH),
        MeshMaterial2d(SOLID_WHITE),
        TileCoords(coords),
        ZOrder::MACHINE,
        children![
            (
                Name::new("Outlet Inner"),
                FactoryLayer,
                Mesh2d(INLET_MESH_INNER),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.2),
            ),
            machine_port(
                MachinePort {
                    side: dir,
                    flow: FlowDirection::Inlet,
                },
                flow_material.clone(),
            ),
        ],
    )
}

pub fn test_machine(
    mut materials: ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![
        MachinePort {
            side: Direction::Right,
            flow: FlowDirection::Inlet,
        },
        MachinePort {
            side: Direction::Up,
            flow: FlowDirection::Outlet,
        },
    ];
    (
        Name::new("Test Machine"),
        Machine,
        FactoryLayer,
        Mesh2d(CONSTRUCTOR_MESH),
        MeshMaterial2d(SOLID_WHITE),
        ZOrder::MACHINE,
        Children::spawn((
            Spawn((
                Name::new("Test Machine Inner"),
                FactoryLayer,
                Mesh2d(CONSTRUCTOR_MESH_INNER),
                MeshMaterial2d(materials.add(MetalDither {
                    fill: 0.3,
                    scale: 40.0,
                })),
                Transform::from_xyz(0.0, 0.0, 0.2),
            )),
            SpawnIter(
                ports
                    .into_iter()
                    .map(move |port| machine_port(port, flow_material.clone())),
            ),
        )),
    )
}

pub fn machine_port(port: MachinePort, flow_material: Handle<DitherMaterial>) -> impl Bundle {
    (
        Name::new("Machine Port"),
        InheritedVisibility::VISIBLE,
        Transform::IDENTITY,
        children![pipe_bridge(
            port.flow == FlowDirection::Inlet,
            port.side,
            flow_material
        )],
        port,
    )
}
