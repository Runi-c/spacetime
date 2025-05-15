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
    materials::{DitherMaterial, MetalDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::{ResourceType, Resources},
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    meshes::CONSTRUCTOR_MESH,
    port::{machine_port, FlowDirection, MachinePort},
    Buffer, Machine,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, hull_fixer_tick.in_set(Sets::Physics));
}

#[derive(Component, Clone)]
#[require(Machine)]
pub struct HullFixer;

pub fn hull_fixer(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![MachinePort::new(Direction::Up, FlowDirection::Inlet)];
    const BIG: f32 = TILE_SIZE * 0.4;
    const SMALL: f32 = TILE_SIZE * 0.1;
    let vertices = vec![
        vec2(-BIG, -BIG),
        vec2(BIG, -BIG),
        vec2(BIG, BIG),
        vec2(SMALL, BIG),
        vec2(SMALL, -SMALL),
        vec2(-SMALL, -SMALL),
        vec2(-SMALL, BIG),
        vec2(-BIG, BIG),
    ];
    let mesh = meshes.add(Mesh::fill_polygon(&vertices));
    (
        Name::new("Hull Fixer"),
        Machine,
        HullFixer,
        ShopItem::HullFixer,
        Buffer(ResourceType::Mineral, 0.0),
        FactoryLayer,
        Mesh2d(CONSTRUCTOR_MESH),
        MeshMaterial2d(SOLID_WHITE),
        ZOrder::MACHINE,
        Tooltip(
            "Hull Reconstructor".to_string(),
            Some("Consumes minerals to repair ship".to_string()),
        ),
        Children::spawn((
            Spawn((
                Name::new("Hull Fixer Inner"),
                FactoryLayer,
                Mesh2d(mesh),
                MeshMaterial2d(materials.add(MetalDither {
                    fill: 0.3,
                    scale: 40.0,
                })),
                Transform::from_xyz(0.0, 0.0, 0.3),
            )),
            SpawnIter(
                ports
                    .into_iter()
                    .map(move |port| machine_port(port, flow_material.clone())),
            ),
        )),
    )
}

fn hull_fixer_tick(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<&Children, (With<HullFixer>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
    mut resources: ResMut<Resources>,
) -> Result {
    for _ in ticks.read() {
        for children in machines.iter() {
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0)?;
                    if port.flow == FlowDirection::Inlet
                        && network.resource == ResourceType::Mineral
                    {
                        let parent = parents.get(network.source)?;
                        let source = buffers.get(parent.0)?;
                        if source.1 < 1.0 {
                            continue;
                        }
                        if resources.health < 100.0 {
                            let new_health = (resources.health + 20.0).min(100.0);
                            resources.health = new_health;
                            commands
                                .entity(parent.0)
                                .insert(Buffer(source.0, source.1 - 1.0));
                            info!("Repairing hull: {} to {}", resources.health, new_health);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
