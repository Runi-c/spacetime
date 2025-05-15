use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::{
    factory::{
        grid::{Direction, TileCoords, TILE_SIZE},
        pipe_network::{InNetwork, PipeNetwork},
        shop::ShopItem,
        time::FactoryTick,
        tooltip::Tooltip,
    },
    layers::FactoryLayer,
    materials::{DitherMaterial, MetalDither, SOLID_BLACK, SOLID_WHITE},
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
    app.add_systems(Update, rocket_factory_tick.in_set(Sets::Physics));
}

#[derive(Component, Clone)]
#[require(Machine)]
pub struct RocketFactory;

pub fn rocket_factory(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![
        MachinePort::new(Direction::Right, FlowDirection::Inlet),
        MachinePort::new(Direction::Down, FlowDirection::Inlet),
        MachinePort::new(Direction::Left, FlowDirection::Outlet),
    ];
    let mesh = meshes.add(Triangle2d::new(
        vec2(TILE_SIZE * 0.25, -TILE_SIZE * 0.4),
        vec2(-TILE_SIZE * 0.25, -TILE_SIZE * 0.4),
        vec2(0.0, TILE_SIZE * 0.4),
    ));
    (
        Name::new("Rocket Factory"),
        Machine,
        RocketFactory,
        ShopItem::RocketFactory,
        Buffer(ResourceType::Rockets, 0.0),
        FactoryLayer,
        Mesh2d(CONSTRUCTOR_MESH),
        MeshMaterial2d(SOLID_WHITE),
        ZOrder::MACHINE,
        Tooltip(
            "Rocket Factory".to_string(),
            Some("Consumes minerals from below\nand gas from the right\nto produce rockets to the left".to_string()),
        ),
        Children::spawn((
            Spawn((
                Name::new("Rocket Factory Inner"),
                FactoryLayer,
                Mesh2d(CONSTRUCTOR_MESH_INNER),
                MeshMaterial2d(materials.add(MetalDither {
                    fill: 0.5,
                    scale: 40.0,
                })),
                Transform::from_xyz(0.0, 0.0, 0.3),
            )),
            Spawn((
                Name::new("Rocket Factory Inner 2"),
                FactoryLayer,
                Mesh2d(mesh),
                MeshMaterial2d(SOLID_BLACK),
                Transform::from_xyz(0.0, 0.0, 0.4),
            )),
            SpawnIter(
                ports
                    .into_iter()
                    .map(move |port| machine_port(port, flow_material.clone())),
            ),
        )),
    )
}

fn rocket_factory_tick(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Buffer, &Children), (With<RocketFactory>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
) -> Result {
    for _ in ticks.read() {
        for (entity, buffer, children) in machines.iter() {
            if buffer.1 >= 5.0 {
                continue;
            }
            let mut mineral_buffer = None;
            let mut gas_buffer = None;
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0)?;
                    if port.flow == FlowDirection::Inlet {
                        let parent = parents.get(network.source)?;
                        let source = buffers.get(parent.0)?;
                        if source.1 < 3.0 {
                            continue;
                        }
                        info!("Found inlet: {:?}", network.resource);
                        if network.resource == ResourceType::Mineral {
                            mineral_buffer = Some((parent.0, source));
                        } else if network.resource == ResourceType::Gas {
                            gas_buffer = Some((parent.0, source));
                        }
                    }
                }
            }
            if let (Some((mineral_source, mineral_buffer)), Some((gas_source, gas_buffer))) =
                (mineral_buffer, gas_buffer)
            {
                commands
                    .entity(mineral_source)
                    .insert(Buffer(ResourceType::Mineral, mineral_buffer.1 - 3.0));
                commands
                    .entity(gas_source)
                    .insert(Buffer(ResourceType::Gas, gas_buffer.1 - 2.0));
                commands
                    .entity(entity)
                    .insert(Buffer(ResourceType::Rockets, buffer.1 + 1.0));
                info!("Made a rocket :)",);
            }
        }
    }

    Ok(())
}
