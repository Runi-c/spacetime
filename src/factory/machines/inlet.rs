use bevy::prelude::*;

use crate::{
    factory::grid::{Direction, TileCoords},
    layers::FactoryLayer,
    materials::{DitherMaterial, SOLID_WHITE},
    resources::{ResourceType, Resources},
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    meshes::{INLET_MESH, INLET_MESH_INNER},
    port::{machine_port, FlowDirection, MachinePort},
    Buffer, Machine,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            inlet_fill.in_set(Sets::Physics),
            inlet_update_material.in_set(Sets::PostUpdate),
        ),
    );
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
        Name::new(format!("{} Inlet", resource.to_string())),
        Inlet(resource),
        Buffer(resource, 0.0),
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
                MachinePort::new(dir, FlowDirection::Outlet),
                flow_material.clone(),
            ),
        ],
    )
}

fn inlet_fill(mut inlets: Query<(&mut Buffer, &Inlet)>, mut resources: ResMut<Resources>) {
    for (mut buffer, inlet) in inlets.iter_mut() {
        if resources.get(inlet.0) >= 1.0 && buffer.1 < 10.0 {
            resources.add(inlet.0, -1.0);
            buffer.1 += 1.0;
        }
    }
}

fn inlet_update_material(
    inlets: Query<(&Buffer, &Children), (With<Inlet>, Changed<Buffer>)>,
    material: Query<&MeshMaterial2d<DitherMaterial>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    for (buffer, children) in inlets.iter() {
        for child in children.iter() {
            if let Ok(material) = material.get(child) {
                let mat = materials.get_mut(&material.0).unwrap();
                mat.settings.fill = 0.2 + 0.4 * buffer.1 / 10.0;
            }
        }
    }
}
