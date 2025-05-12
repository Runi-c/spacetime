use std::collections::HashSet;

use bevy::{asset::weak_handle, prelude::*};

use crate::{
    layers::FactoryLayer,
    materials::{Dither, DitherMaterial, SOLID_BLACK, SOLID_WHITE},
    resources::ResourceType,
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    camera::CursorPosition,
    grid::{Direction, Grid, Tile, TileCoords, TILE_SIZE},
    machines::{Buffer, FlowDirection, Machine, MachinePort},
    shop::PickedUpItem,
    time::TimeScale,
};

pub const PIPE_MESH: Handle<Mesh> = weak_handle!("2c48ec14-5922-4888-b929-bfcb0d1379a5");
pub const PIPE_MESH_INNER: Handle<Mesh> = weak_handle!("b67e0c8e-a3ad-4477-b865-e30799506a15");
pub const PIPE_BRIDGE: Handle<Mesh> = weak_handle!("19d40193-0720-4cf3-accd-445eb2d6f3f2");
pub const PIPE_BRIDGE_INNER: Handle<Mesh> = weak_handle!("61135ba2-b044-4dc6-bb2e-41c3c1f4aadf");

pub fn plugin(app: &mut App) {
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    meshes.insert(
        PIPE_MESH.id(),
        Rectangle::from_length(PIPE_SIZE + 2.0).into(),
    );
    meshes.insert(
        PIPE_MESH_INNER.id(),
        Rectangle::from_length(PIPE_SIZE).into(),
    );
    meshes.insert(
        PIPE_BRIDGE.id(),
        Rectangle::new(TILE_SIZE * 0.8 + 4.0, PIPE_SIZE + 2.0).into(),
    );
    meshes.insert(
        PIPE_BRIDGE_INNER.id(),
        Rectangle::new(TILE_SIZE * 0.8 + 4.0, PIPE_SIZE).into(),
    );
    app.register_type::<NetworkMembers>();
    app.register_type::<InNetwork>();
    app.register_type::<PipeNetwork>();
    app.register_type::<MachinePort>();
    app.add_event::<InvalidateNetworks>()
        .add_systems(Startup, setup_pipe_materials.in_set(Sets::Init))
        .add_systems(
            Update,
            (
                draw_pipe.in_set(Sets::Input),
                (connect_ports, connect_pipes, rebuild_networks)
                    .chain()
                    .in_set(Sets::Update),
                (update_pipe_material, make_pipe_bridges, log_networks).in_set(Sets::PostUpdate),
            ),
        );
}

#[derive(Resource)]
pub struct PipeFlowMaterial(pub Handle<DitherMaterial>);

fn setup_pipe_materials(mut commands: Commands, mut materials: ResMut<Assets<DitherMaterial>>) {
    let material = materials.add(Dither {
        fill: 0.01,
        scale: 44.0,
        ..default()
    });
    commands.insert_resource(PipeFlowMaterial(material));
}

const PIPE_SIZE: f32 = TILE_SIZE * 0.6;

#[derive(Component, Clone, Debug)]
pub struct Pipe;

#[derive(Component, Clone, Debug)]
#[relationship(relationship_target = PipeFrom)]
pub struct PipeTo(Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = PipeTo)]
pub struct PipeFrom(Entity);

#[derive(Reflect, Component, Debug)]
pub struct PipeNetwork {
    pub source: Entity,
    pub resource: ResourceType,
    pub sink: Option<Entity>,
    pub material: Handle<DitherMaterial>,
}

#[derive(Event)]
pub struct InvalidateNetworks;

#[derive(Reflect, Component, Clone, Debug)]
#[relationship(relationship_target = NetworkMembers)]
pub struct InNetwork(pub Entity);

#[derive(Reflect, Component, Clone, Debug)]
#[relationship_target(relationship = InNetwork)]
pub struct NetworkMembers(Vec<Entity>);

pub fn pipe_bundle(pos: IVec2) -> impl Bundle {
    (
        Name::new("Pipe"),
        Pipe,
        FactoryLayer,
        TileCoords(pos),
        ZOrder::PIPE,
        Mesh2d(PIPE_MESH),
        MeshMaterial2d(SOLID_WHITE),
        RecalcBridges,
        children![(
            Name::new("Pipe Inner"),
            FactoryLayer,
            Mesh2d(PIPE_MESH_INNER),
            MeshMaterial2d(SOLID_BLACK),
            Transform::from_xyz(0.0, 0.0, 0.01),
        )],
    )
}

fn draw_pipe(
    mut commands: Commands,
    tiles: Query<&TileCoords, With<Tile>>,
    pipes: Query<Entity, With<Pipe>>,
    cursor_pos: CursorPosition,
    mut grid: ResMut<Grid>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut events: EventWriter<InvalidateNetworks>,
    picked_up_item: Option<Res<PickedUpItem>>,
) {
    if !buttons.pressed(MouseButton::Left) && !buttons.pressed(MouseButton::Right) {
        return;
    }
    if picked_up_item.is_some() {
        return;
    }
    if let Some(tile_pos) = cursor_pos.tile_pos() {
        if let Some(tile_entity) = grid.get_tile(tile_pos) {
            let coords = tiles.get(tile_entity).unwrap();
            if buttons.pressed(MouseButton::Left) {
                if grid.get_building(tile_pos).is_none() {
                    info!("Drawing pipe at tile: {:?}", coords);
                    let pipe = commands.spawn((pipe_bundle(tile_pos), ChildOf(grid.entity)));
                    grid.get_building_mut(tile_pos).unwrap().replace(pipe.id());
                    events.write(InvalidateNetworks);
                }
            } else if buttons.pressed(MouseButton::Right) {
                if let Some(building) = grid.get_building(tile_pos) {
                    if pipes.contains(building) {
                        info!("Removing pipe at tile: {:?}", coords);
                        commands.entity(building).despawn();
                        for dir in Direction::iter() {
                            if let Some(neighbor) = grid.get_building(tile_pos + dir.as_ivec2()) {
                                if pipes.contains(neighbor) {
                                    commands.entity(neighbor).insert(RecalcBridges);
                                }
                            }
                        }
                        grid.get_building_mut(tile_pos).unwrap().take();
                        events.write(InvalidateNetworks);
                    }
                }
            }
        }
    }
}

fn connect_pipes(
    mut commands: Commands,
    added_pipes: Query<(Entity, &TileCoords), Added<Pipe>>,
    connections: Query<(Option<&PipeTo>, Option<&PipeFrom>), With<Pipe>>,
    ports: Query<(Entity, &MachinePort)>,
    machines: Query<(&Machine, &Children)>,
    mut grid: ResMut<Grid>,
) {
    let mut iter = added_pipes.iter();
    if let Some((pipe, tile_pos)) = iter.next() {
        let mut connected_to = false;
        let mut connected_from = false;
        for neighbor_dir in Direction::iter() {
            if let Some(neighbor) = grid.get_building(tile_pos.0 + neighbor_dir.as_ivec2()) {
                if let Ok((_, children)) = machines.get(neighbor) {
                    let maybe_port = children
                        .iter()
                        .filter_map(|c| ports.get(c).ok())
                        .find(|(_, port)| port.side == neighbor_dir.flip());
                    if let Some((neighbor, port)) = maybe_port {
                        if !connected_to && port.flow == FlowDirection::Inlet {
                            connected_to = true;
                            commands.entity(pipe).insert(PipeTo(neighbor));
                        } else if !connected_from && port.flow == FlowDirection::Outlet {
                            connected_from = true;
                            commands.entity(neighbor).insert(PipeTo(pipe));
                        }
                    }
                } else if let Ok((pipe_to, pipe_from)) = connections.get(neighbor) {
                    if !connected_to && pipe_from.is_none() {
                        connected_to = true;
                        commands.entity(pipe).insert(PipeTo(neighbor));
                    } else if !connected_from && pipe_to.is_none() {
                        connected_from = true;
                        commands.entity(neighbor).insert(PipeTo(pipe));
                    }
                }
            }
        }
    }
    for (pipe, coords) in iter {
        // respawn other pipes so that only one connection happpens per frame
        commands.entity(pipe).despawn();
        let pipe = commands
            .spawn((pipe_bundle(coords.0), ChildOf(grid.entity)))
            .id();
        grid.get_building_mut(coords.0).unwrap().replace(pipe);
    }
}

fn connect_ports(
    mut commands: Commands,
    added_ports: Query<(Entity, &MachinePort, &ChildOf), Added<MachinePort>>,
    machines: Query<(&TileCoords, &Children), With<Machine>>,
    ports: Query<&MachinePort>,
    grid: Res<Grid>,
) {
    for (port_entity, port, child_of) in added_ports.iter() {
        info!("Connecting port: {:?}", port);
        if let Ok((coords, _)) = machines.get(child_of.parent()) {
            info!("Port coords: {:?}", coords);
            if let Some(neighbor) = grid.get_building(coords.0 + port.side.as_ivec2()) {
                info!("Neighbor: {:?}", neighbor);
                if let Ok((_, children)) = machines.get(neighbor) {
                    info!("Neighbor children: {:?}", children);
                    // neighbor is a machine
                    let maybe_port =
                        children
                            .iter()
                            .filter_map(|c| ports.get(c).ok())
                            .find(|nbr_port| {
                                info!("Neighbor port: {:?}", nbr_port);
                                port.side == nbr_port.side.flip() && port.flow != nbr_port.flow
                            });
                    if let Some(port) = maybe_port {
                        info!("Connecting ports: {:?} -> {:?}", port_entity, port);
                        if port.flow == FlowDirection::Inlet {
                            commands.entity(port_entity).insert(PipeTo(neighbor));
                        } else if port.flow == FlowDirection::Outlet {
                            commands.entity(neighbor).insert(PipeTo(port_entity));
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct PipeBridge;
#[derive(Component, Clone)]
pub struct RecalcBridges;

fn make_pipe_bridges(
    mut commands: Commands,
    bridges: Query<&PipeBridge>,
    pipes: Query<
        (
            Entity,
            &TileCoords,
            Option<&PipeTo>,
            Option<&PipeFrom>,
            &Children,
        ),
        Or<(Changed<PipeTo>, Changed<PipeFrom>, With<RecalcBridges>)>,
    >,
    ports: Query<&ChildOf, With<MachinePort>>,
    coords_q: Query<&TileCoords>,
    flow_material: Res<PipeFlowMaterial>,
) {
    for (pipe, coords, maybe_to, maybe_from, children) in pipes.iter() {
        for child in children.iter() {
            if bridges.contains(child) {
                commands.entity(child).despawn();
            }
        }
        let mut bridges = vec![];
        if let Some(to) = maybe_to {
            bridges.push((false, to.0));
        }
        if let Some(from) = maybe_from {
            bridges.push((true, from.0));
        }
        for (flip, entity) in bridges {
            let nbr = if let Ok(parent) = ports.get(entity) {
                parent.parent()
            } else {
                entity
            };
            let nbr_coords = coords_q.get(nbr).unwrap();
            let dir = coords.direction_to(nbr_coords);
            commands.spawn((
                pipe_bridge(flip, dir, flow_material.0.clone()),
                ChildOf(pipe),
            ));
        }
        commands.entity(pipe).remove::<RecalcBridges>();
    }
}

pub fn pipe_bridge(
    flip: bool,
    dir: Direction,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let offset_amount = TILE_SIZE * 0.1 + 2.0;
    let offset = dir.as_vec2() * offset_amount;
    let dir = if flip { dir.flip() } else { dir };
    (
        Name::new("Pipe Bridge"),
        PipeBridge,
        FactoryLayer,
        Mesh2d(PIPE_BRIDGE),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(offset.x, offset.y, 0.0)
            .with_rotation(Quat::from_rotation_z(dir.angle())),
        children![(
            Name::new("Pipe Bridge Inner"),
            FactoryLayer,
            Mesh2d(PIPE_BRIDGE_INNER),
            MeshMaterial2d(flow_material),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}

fn update_pipe_material(
    material: Res<PipeFlowMaterial>,
    time: Res<Time>,
    timescale: Res<TimeScale>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    if let Some(material) = materials.get_mut(&material.0) {
        material.settings.offset.x -= time.delta_secs() * timescale.0 * 10.0;
    }
}

fn rebuild_networks(
    mut commands: Commands,
    mut events: EventReader<InvalidateNetworks>,
    networks: Query<Entity, With<PipeNetwork>>,
    machines: Query<(&TileCoords, &Buffer, &Children), With<Machine>>,
    ports: Query<(Entity, &ChildOf, &MachinePort)>,
    pipe_q: Query<&TileCoords, With<Pipe>>,
    pipe_chain: Query<&PipeTo, With<Pipe>>,
    grid: Res<Grid>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    info!("Rebuilding pipe networks");
    for network in networks.iter() {
        commands.entity(network).despawn();
    }
    let mut networks = vec![];
    for (port_entity, child_of, port) in ports.iter() {
        if port.flow == FlowDirection::Inlet {
            continue;
        }
        let parent = child_of.parent();
        if !machines.contains(parent) {
            continue;
        }
        let (coords, buffer, _) = machines.get(child_of.parent()).unwrap();
        let coords = coords.0 + port.side.as_ivec2();
        let mut sink = None;
        let mut members = vec![port_entity];
        if let Some(building) = grid.get_building(coords) {
            if pipe_q.contains(building) {
                // neighbor is pipe
                members.push(building);
                let mut visited = HashSet::new();
                let pipes = pipe_chain
                    .iter_ancestors::<PipeTo>(building)
                    .take_while(|pipe| visited.insert(*pipe))
                    .collect::<Vec<_>>();
                if !pipes.is_empty() {
                    let last = *pipes.last().unwrap();
                    if ports.contains(last) {
                        sink = Some(last);
                    }
                    members.extend(pipes);
                }
            } else if let Ok((_, _, children)) = machines.get(building) {
                // neighbor is machine
                let maybe_port = children
                    .iter()
                    .filter_map(|c| ports.get(c).ok())
                    .filter(|(_, _, p)| p.flow == FlowDirection::Inlet)
                    .find(|(_, _, nbr_port)| nbr_port.side == port.side.flip());
                if let Some((nbr_port_e, _, _)) = maybe_port {
                    sink = Some(nbr_port_e);
                    members.push(nbr_port_e);
                }
            }
        }
        networks.push((port_entity, buffer.0, sink, members));
    }

    for (source, resource, sink, members) in networks {
        let network = commands
            .spawn((
                Name::new("Pipe Network"),
                PipeNetwork {
                    source,
                    resource,
                    sink,
                    material: Handle::default(),
                },
            ))
            .id();
        commands.insert_batch(
            members
                .iter()
                .map(|e| (*e, InNetwork(network)))
                .collect::<Vec<_>>(),
        );
    }
}

fn log_networks(
    networks: Query<&PipeNetwork, Added<PipeNetwork>>,
    child_of: Query<&ChildOf>,
    names: Query<&Name>,
) {
    for network in networks.iter() {
        let source = names
            .get(child_of.get(network.source).unwrap().parent())
            .unwrap();
        let sink = if let Some(sink) = network.sink {
            names.get(child_of.get(sink).unwrap().parent()).unwrap()
        } else {
            &Name::new("None")
        };
        info!("Pipe network: {} -> {}", source.as_str(), sink.as_str());
    }
}
