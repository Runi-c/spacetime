use bevy::prelude::*;

use crate::{
    factory::{
        grid::{Direction, TileCoords},
        pipe_network::{InNetwork, PipeNetwork},
    },
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
    app.add_systems(Update, outlet_fill.in_set(Sets::Physics));
}

#[derive(Component, Clone)]
#[require(Machine)]
pub struct Outlet;

pub fn outlet(
    coords: IVec2,
    dir: Direction,
    material: Handle<DitherMaterial>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    (
        Name::new("Outlet"),
        Outlet,
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
                MachinePort::new(dir, FlowDirection::Inlet),
                flow_material.clone(),
            ),
        ],
    )
}

fn outlet_fill(
    outlets: Query<&Children, With<Outlet>>,
    ports: Query<&InNetwork>,
    mut resources: ResMut<Resources>,
    networks: Query<&PipeNetwork>,
    mut buffers: Query<&mut Buffer>,
    parents: Query<&ChildOf>,
) -> Result {
    for children in outlets.iter() {
        let in_network = children.iter().find_map(|child| ports.get(child).ok());
        if let Some(in_network) = in_network {
            let network = networks.get(in_network.0)?;
            if network.resource == ResourceType::Rockets || network.resource == ResourceType::Ammo {
                if let Ok(parent) = parents.get(network.source).map(|p| p.parent()) {
                    if let Ok(mut buffer) = buffers.get_mut(parent) {
                        if buffer.1 > 0.0 && resources.get(network.resource) < 10.0 {
                            info!(
                                "Filling outlet: {:?} to {}",
                                network.resource,
                                resources.get(network.resource) + 1.0
                            );
                            resources.add(network.resource, 1.0);
                            buffer.1 -= 1.0;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
