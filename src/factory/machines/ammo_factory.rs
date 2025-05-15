use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::{
    factory::{
        grid::{Direction, TileCoords},
        pipe_network::{InNetwork, PipeNetwork},
        shop::ShopItem,
        time::FactoryTick,
        tooltip::Tooltip,
    },
    layers::FactoryLayer,
    materials::{DitherMaterial, MetalDither, SOLID_WHITE},
    resources::ResourceType,
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    meshes::{CONSTRUCTOR_MESH, CONSTRUCTOR_MESH_INNER},
    port::{machine_port, FlowDirection, MachinePort},
    Buffer, Machine,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, ammo_factory_tick.in_set(Sets::Physics));
}

#[derive(Component, Clone)]
#[require(Machine)]
pub struct AmmoFactory;

pub fn ammo_factory(
    materials: &mut ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![
        MachinePort::new(Direction::Left, FlowDirection::Inlet),
        MachinePort::new(Direction::Right, FlowDirection::Outlet),
    ];
    (
        Name::new("Ammo Factory"),
        Machine,
        AmmoFactory,
        ShopItem::AmmoFactory,
        Buffer(ResourceType::Ammo, 0.0),
        FactoryLayer,
        Mesh2d(CONSTRUCTOR_MESH),
        MeshMaterial2d(SOLID_WHITE),
        ZOrder::MACHINE,
        Tooltip(
            "Ammo Manufactory".to_string(),
            Some("Consumes minerals from the left\nand produces ammo to the right".to_string()),
        ),
        Children::spawn((
            Spawn((
                Name::new("Ammo Factory Inner"),
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

fn ammo_factory_tick(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Buffer, &Children), (With<AmmoFactory>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
) -> Result {
    for _ in ticks.read() {
        for (entity, buffer, children) in machines.iter() {
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0)?;
                    if port.flow == FlowDirection::Inlet
                        && network.resource == ResourceType::Mineral
                    {
                        let parent = parents.get(network.source)?;
                        let source = buffers.get(parent.0)?;
                        if source.1 < 1.0 || buffer.1 >= 10.0 {
                            continue;
                        }
                        commands
                            .entity(parent.0)
                            .insert(Buffer(source.0, source.1 - 1.0));
                        commands
                            .entity(entity)
                            .insert(Buffer(ResourceType::Ammo, buffer.1 + 3.0));
                        info!(
                            "Filling ammo factory: {:?} to {}",
                            ResourceType::Ammo,
                            buffer.1 + 1.0
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
