use std::collections::HashSet;

use bevy::{asset::weak_handle, prelude::*, window::PrimaryWindow};

use crate::{
    layers::FactoryLayer,
    materials::{Dither, DitherMaterial, SOLID_BLACK, SOLID_WHITE},
    resources::ResourceType,
    scheduling::Sets,
};

use super::{
    camera::FactoryCamera,
    grid::{Direction, Grid, Tile, TileCoords, GRID_WIDTH, TILE_SIZE},
    machines::{FlowDirection, Machine, MachinePort},
    time::TimeScale,
};

pub const PIPE_BRIDGE: Handle<Mesh> = weak_handle!("19d40193-0720-4cf3-accd-445eb2d6f3f2");
pub const PIPE_BRIDGE_INNER: Handle<Mesh> = weak_handle!("61135ba2-b044-4dc6-bb2e-41c3c1f4aadf");

pub fn plugin(app: &mut App) {
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    meshes.insert(
        PIPE_BRIDGE.id(),
        Rectangle::new(TILE_SIZE * 0.8 + 4.0, PIPE_SIZE + 2.0).into(),
    );
    meshes.insert(
        PIPE_BRIDGE_INNER.id(),
        Rectangle::new(TILE_SIZE * 0.8 + 4.0, PIPE_SIZE).into(),
    );
    app.add_event::<InvalidateNetworks>()
        .add_systems(Startup, setup_pipe_materials.in_set(Sets::Init))
        .add_systems(
            Update,
            (
                draw_pipe.in_set(Sets::Input),
                (connect_pipes, rebuild_networks).in_set(Sets::Update),
                (update_pipe_material, make_pipe_bridges).in_set(Sets::PostUpdate),
            ),
        );
}

#[derive(Resource)]
pub struct PipeFlowMaterial(pub Handle<DitherMaterial>);

fn setup_pipe_materials(mut commands: Commands, mut materials: ResMut<Assets<DitherMaterial>>) {
    let material = materials.add(Dither {
        fill: 0.01,
        scale: 0.05,
        //flags: DitherFlags::WORLDSPACE.bits(),
        ..default()
    });
    commands.insert_resource(PipeFlowMaterial(material));
}

const PIPE_SIZE: f32 = TILE_SIZE * 0.6;

#[derive(Component, Debug)]
pub struct Pipe;

#[derive(Component, Debug)]
#[relationship(relationship_target = PipeFrom)]
pub struct PipeTo(Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = PipeTo)]
pub struct PipeFrom(Entity);

#[derive(Component, Debug)]
pub struct PipeNetwork {
    pub source: Entity,
    pub resource: ResourceType,
    pub sink: Option<Entity>,
    pub material: Handle<DitherMaterial>,
}

#[derive(Event)]
pub struct InvalidateNetworks;

#[derive(Component, Debug)]
#[relationship(relationship_target = NetworkMembers)]
pub struct InNetwork(Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = InNetwork)]
pub struct NetworkMembers(Vec<Entity>);

fn draw_pipe(
    mut commands: Commands,
    tiles: Query<(&TileCoords, Option<&Children>), With<Tile>>,
    pipes: Query<Entity, With<Pipe>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<FactoryCamera>>,
    mut grid: ResMut<Grid>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventWriter<InvalidateNetworks>,
) {
    if !buttons.pressed(MouseButton::Left) && !buttons.pressed(MouseButton::Right) {
        return;
    }
    let window = window.single().unwrap();
    let (camera, camera_transform) = camera.single().unwrap();
    let screen_pos = window.cursor_position().unwrap_or_default();
    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
        let tile_pos = ((world_pos + GRID_WIDTH * 0.5) / TILE_SIZE)
            .floor()
            .as_ivec2();
        if let Some(tile_entity) = grid.get_tile(tile_pos) {
            let (coords, children) = tiles.get(tile_entity).unwrap();
            if buttons.pressed(MouseButton::Left) {
                if grid.get_building(tile_pos).is_none() {
                    info!("Drawing pipe at tile: {:?}", coords);
                    let pipe = commands.spawn((
                        Name::new("Pipe"),
                        Pipe,
                        FactoryLayer,
                        TileCoords(tile_pos),
                        ChildOf {
                            parent: tile_entity,
                        },
                        Mesh2d(meshes.add(Rectangle::new(PIPE_SIZE + 2.0, PIPE_SIZE + 2.0))),
                        MeshMaterial2d(SOLID_WHITE),
                        Transform::from_xyz(TILE_SIZE * 0.5, TILE_SIZE * 0.5, 1.0),
                        children![(
                            Name::new("Pipe Inner"),
                            FactoryLayer,
                            Mesh2d(meshes.add(Rectangle::new(PIPE_SIZE, PIPE_SIZE))),
                            MeshMaterial2d(SOLID_BLACK),
                            Transform::from_xyz(0.0, 0.0, 0.01),
                        )],
                    ));
                    grid.get_building_mut(tile_pos).unwrap().replace(pipe.id());
                    events.write(InvalidateNetworks);
                }
            } else if buttons.pressed(MouseButton::Right) {
                if let Some(children) = children {
                    for child in children.iter() {
                        if pipes.contains(child) {
                            info!("Removing pipe at tile: {:?}", coords);
                            commands.entity(child).despawn();
                            for dir in Direction::iter() {
                                if let Some(neighbor) = grid.get_building(tile_pos + dir.as_ivec2())
                                {
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
}

fn connect_pipes(
    mut commands: Commands,
    added_pipes: Query<(Entity, &ChildOf), Added<Pipe>>,
    connections: Query<(Option<&PipeTo>, Option<&PipeFrom>), With<Pipe>>,
    ports: Query<(Entity, &MachinePort)>,
    machines: Query<(&Machine, &Children)>,
    tiles: Query<&TileCoords, With<Tile>>,
    grid: Res<Grid>,
) {
    for (pipe, relation) in added_pipes.iter() {
        let tile_pos = tiles.get(relation.parent).unwrap().0;
        let mut connected_to = false;
        let mut connected_from = false;
        for neighbor_dir in Direction::iter() {
            if let Some(neighbor) = grid.get_building(tile_pos + neighbor_dir.as_ivec2()) {
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
}

#[derive(Component)]
pub struct PipeBridge;
#[derive(Component)]
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
    coords_q: Query<&TileCoords>,
    flow_material: Res<PipeFlowMaterial>,
) {
    for (pipe, coords, maybe_to, maybe_from, children) in pipes.iter() {
        for child in children.iter() {
            if bridges.contains(child) {
                commands.entity(child).despawn();
            }
        }
        if let Some(to) = maybe_to {
            info!("Making pipe bridge to: {:?}", to.0);
            let nbr_coords = coords_q.get(to.0).unwrap();
            let dir = coords.direction_to(nbr_coords);
            commands.spawn((
                pipe_bridge(false, dir, flow_material.0.clone()),
                ChildOf { parent: pipe },
            ));
        }
        if let Some(from) = maybe_from {
            info!("Making pipe bridge from: {:?}", from.0);
            let nbr_coords = coords_q.get(from.0).unwrap();
            let dir = coords.direction_to(nbr_coords);
            commands.spawn((
                pipe_bridge(true, dir, flow_material.0.clone()),
                ChildOf { parent: pipe },
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
        Transform::from_xyz(offset.x, offset.y, 0.1)
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
    machines: Query<&Children, With<Machine>>,
    ports: Query<(Entity, &MachinePort, &TileCoords)>,
    pipe_q: Query<&TileCoords, With<Pipe>>,
    pipe_chain: Query<&PipeTo, With<Pipe>>,
    grid: Res<Grid>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    for network in networks.iter() {
        commands.entity(network).despawn();
    }
    let mut networks = vec![];
    for (port_entity, port, coords) in ports.iter() {
        if port.flow == FlowDirection::Inlet {
            continue;
        }
        let mut sink = None;
        let coords = coords.0 + port.side.as_ivec2();
        let mut members = vec![port_entity];
        if let Some(building) = grid.get_building(coords) {
            if pipe_q.contains(building) {
                // neighbor is pipe
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
            } else if let Ok(children) = machines.get(building) {
                // neighbor is machine
                let maybe_port = children
                    .iter()
                    .filter_map(|c| ports.get(c).ok())
                    .filter(|(_, p, _)| p.flow == FlowDirection::Outlet)
                    .find(|(_, nbr_port, _)| nbr_port.side == port.side.flip());
                if let Some((nbr_port_e, _, _)) = maybe_port {
                    sink = Some(nbr_port_e);
                    members.push(nbr_port_e);
                }
            }
        }
        networks.push((port_entity, sink, members));
    }

    for (source, sink, members) in networks {
        commands.spawn((
            Name::new("Pipe Network"),
            PipeNetwork {
                source,
                resource: ResourceType::Time,
                sink,
                material: Handle::default(),
            },
            NetworkMembers(members),
        ));
    }
}
