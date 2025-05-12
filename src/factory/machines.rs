use bevy::{asset::weak_handle, ecs::spawn::SpawnIter, prelude::*};
use lyon_tessellation::{
    geom::{euclid::Point2D, Box2D},
    path::{builder::BorderRadii, Winding},
};

use crate::{
    layers::FactoryLayer,
    materials::{DitherMaterial, MetalDither, SOLID_BLACK, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::{ResourceType, Resources},
    scheduling::Sets,
    sounds::Sounds,
    space::Ship,
    z_order::ZOrder,
};

use super::{
    grid::{Direction, Grid, TileCoords, TILE_SIZE},
    pipe::{
        pipe_bridge, pipe_bundle, InNetwork, InvalidateNetworks, Pipe, PipeFlowMaterial,
        PipeNetwork,
    },
    shop::ShopItem,
    time::FactoryTick,
    tooltip::Tooltip,
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

    app.add_systems(
        Update,
        (
            (
                fill_inlet,
                fill_outlet,
                tick_ammo_factory,
                tick_hull_fixer,
                tick_pipe_switch,
                tick_rocket_factory,
            )
                .in_set(Sets::Physics),
            (color_inlet, observe_pipe_switches).in_set(Sets::PostUpdate),
        ),
    );
}

#[derive(Component, Clone, Default)]
pub struct Machine;

#[derive(Reflect, Clone, PartialEq, Eq, Debug)]
pub enum FlowDirection {
    Inlet,
    Outlet,
}

#[derive(Reflect, Component, Clone, Debug)]
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
        Outlet(resource),
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

fn fill_inlet(mut inlets: Query<(&mut Buffer, &Inlet)>, mut resources: ResMut<Resources>) {
    for (mut buffer, inlet) in inlets.iter_mut() {
        if resources.get(inlet.0) >= 1.0 && buffer.1 < 10.0 {
            resources.add(inlet.0, -1.0);
            buffer.1 += 1.0;
        }
    }
}

fn fill_outlet(
    outlets: Query<(&Outlet, &Children)>,
    ports: Query<&InNetwork>,
    mut resources: ResMut<Resources>,
    networks: Query<&PipeNetwork>,
    mut buffers: Query<&mut Buffer>,
    parents: Query<&ChildOf>,
) {
    for (outlet, children) in outlets.iter() {
        let in_network = children.iter().find_map(|child| ports.get(child).ok());
        if let Some(in_network) = in_network {
            let network = networks.get(in_network.0).unwrap();
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
}

fn color_inlet(
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

pub fn ammo_factory(
    materials: &mut ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![
        MachinePort {
            side: Direction::Left,
            flow: FlowDirection::Inlet,
        },
        MachinePort {
            side: Direction::Right,
            flow: FlowDirection::Outlet,
        },
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

#[derive(Component, Clone)]
#[require(Machine)]
pub struct PipeSwitch;

#[derive(Component, Clone)]
pub struct PipeSwitchHandle;

pub fn pipe_switch(
    meshes: &mut ResMut<Assets<Mesh>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![
        MachinePort {
            side: Direction::Left,
            flow: FlowDirection::Inlet,
        },
        MachinePort {
            side: Direction::Right,
            flow: FlowDirection::Outlet,
        },
    ];
    const SMALL: f32 = TILE_SIZE * 0.1;
    const BIG: f32 = TILE_SIZE * 0.2;
    let vertices = vec![
        vec2(0.0, SMALL),
        vec2(0.0, BIG),
        vec2(BIG, 0.0),
        vec2(0.0, -BIG),
        vec2(0.0, -SMALL),
        vec2(-BIG, -SMALL),
        vec2(-BIG, SMALL),
    ];
    (
        Name::new("Pipe Switch"),
        Machine,
        PipeSwitch,
        ShopItem::PipeSwitch,
        Buffer(ResourceType::Mineral, 0.0),
        FactoryLayer,
        Mesh2d(CONSTRUCTOR_MESH),
        MeshMaterial2d(SOLID_WHITE),
        ZOrder::MACHINE,
        Tooltip(
            "Pipe Switch".to_string(),
            Some("Switches output direction when clicked".to_string()),
        ),
        Children::spawn((
            Spawn((
                Name::new("Pipe Switch Inner"),
                FactoryLayer,
                Mesh2d(CONSTRUCTOR_MESH_INNER),
                MeshMaterial2d(SOLID_BLACK),
                Transform::from_xyz(0.0, 0.0, 0.2),
            )),
            Spawn((
                Name::new("Pipe Switch Handle"),
                FactoryLayer,
                PipeSwitchHandle,
                Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
                MeshMaterial2d(SOLID_WHITE),
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

#[derive(Component, Clone)]
#[require(Machine)]
pub struct HullFixer;

pub fn hull_fixer(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![MachinePort {
        side: Direction::Up,
        flow: FlowDirection::Inlet,
    }];
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
        Mesh2d(mesh.clone()),
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

#[derive(Component, Clone)]
#[require(Machine)]
pub struct RocketFactory;

pub fn rocket_factory(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
    flow_material: Handle<DitherMaterial>,
) -> impl Bundle {
    let ports = vec![
        MachinePort {
            side: Direction::Right,
            flow: FlowDirection::Inlet,
        },
        MachinePort {
            side: Direction::Down,
            flow: FlowDirection::Inlet,
        },
        MachinePort {
            side: Direction::Left,
            flow: FlowDirection::Outlet,
        },
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
                Mesh2d(CONSTRUCTOR_MESH),
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

#[derive(Component, Clone, Debug)]
pub struct Buffer(pub ResourceType, pub f32);

#[derive(Component, Clone)]
#[require(Machine)]
pub struct AmmoFactory;

fn tick_ammo_factory(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Buffer, &Children), (With<AmmoFactory>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
) {
    for _ in ticks.read() {
        for (entity, buffer, children) in machines.iter() {
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0).unwrap();
                    if port.flow == FlowDirection::Inlet
                        && network.resource == ResourceType::Mineral
                    {
                        let parent = parents.get(network.source).unwrap();
                        let source = buffers.get(parent.0).unwrap();
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
}

fn tick_rocket_factory(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Buffer, &Children), (With<RocketFactory>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
) {
    for _ in ticks.read() {
        for (entity, buffer, children) in machines.iter() {
            let mut mineral_buffer = None;
            let mut gas_buffer = None;
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0).unwrap();
                    if port.flow == FlowDirection::Inlet {
                        let parent = parents.get(network.source).unwrap();
                        let source = buffers.get(parent.0).unwrap();
                        if source.1 < 3.0 || buffer.1 >= 5.0 {
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
}

fn tick_hull_fixer(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Children), (With<HullFixer>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
    ship: Query<(Entity, &Ship)>,
) {
    for _ in ticks.read() {
        for (entity, children) in machines.iter() {
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0).unwrap();
                    if port.flow == FlowDirection::Inlet
                        && network.resource == ResourceType::Mineral
                    {
                        let parent = parents.get(network.source).unwrap();
                        let source = buffers.get(parent.0).unwrap();
                        if source.1 < 1.0 {
                            continue;
                        }
                        commands
                            .entity(parent.0)
                            .insert(Buffer(source.0, source.1 - 1.0));
                        for (ship_entity, ship) in ship.iter() {
                            if ship.health < 100.0 {
                                commands
                                    .entity(entity)
                                    .insert(Buffer(ResourceType::Mineral, 1.0));
                                commands.entity(ship_entity).insert(Ship {
                                    health: ship.health + 20.0,
                                    ..*ship
                                });
                                info!("Repairing ship: {} to {}", ship.health, ship.health + 20.0);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tick_pipe_switch(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Buffer, &Children), (With<PipeSwitch>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
) {
    for _ in ticks.read() {
        for (entity, buffer, children) in machines.iter() {
            for child in children.iter() {
                if let Ok((port, in_network)) = ports.get(child) {
                    let network = networks.get(in_network.0).unwrap();
                    if port.flow == FlowDirection::Inlet
                        && network.resource == ResourceType::Mineral
                    {
                        let parent = parents.get(network.source).unwrap();
                        let source = buffers.get(parent.0).unwrap();
                        if source.1 < 1.0 || buffer.1 >= 5.0 {
                            continue;
                        }
                        commands
                            .entity(parent.0)
                            .insert(Buffer(source.0, source.1 - 1.0));
                        commands
                            .entity(entity)
                            .insert(Buffer(source.0, buffer.1 + 1.0));
                        info!("Filling pipe switch: {:?} to {}", buffer.1, buffer.1 + 1.0);
                    }
                }
            }
        }
    }
}

fn observe_pipe_switches(mut commands: Commands, switches: Query<Entity, Added<PipeSwitch>>) {
    for entity in switches.iter() {
        commands.entity(entity).observe(pipe_switch_click);
    }
}

fn pipe_switch_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    flow_material: Res<PipeFlowMaterial>,
    pipe_switches: Query<(Entity, &Children, &TileCoords), With<PipeSwitch>>,
    ports: Query<&MachinePort>,
    mut grid: ResMut<Grid>,
    pipes: Query<&Pipe>,
    mut invalidate_networks: EventWriter<InvalidateNetworks>,
    sounds: Res<Sounds>,
    handles: Query<&PipeSwitchHandle>,
) {
    let target = trigger.target();
    if let Ok((switch, children, coords)) = pipe_switches.get(target) {
        for child in children.iter() {
            if let Ok(port) = ports.get(child) {
                if port.flow == FlowDirection::Outlet {
                    let new_dir = match port.side {
                        Direction::Right => Direction::Down,
                        Direction::Down => Direction::Up,
                        Direction::Up => Direction::Right,
                        _ => unreachable!(),
                    };
                    let handle = children.iter().find(|child| handles.contains(*child));
                    if let Some(handle) = handle {
                        commands.entity(handle).insert(
                            Transform::from_xyz(0.0, 0.0, 0.3)
                                .with_rotation(Quat::from_rotation_z(new_dir.angle())),
                        );
                    }
                    let old_coords = coords.0 + port.side.as_ivec2();
                    let new_coords = coords.0 + new_dir.as_ivec2();
                    for coords in [old_coords, new_coords] {
                        if let Some(pipe) = grid.get_building(coords).filter(|e| pipes.contains(*e))
                        {
                            commands.entity(pipe).despawn();
                            let pipe = commands
                                .spawn((pipe_bundle(coords), ChildOf(grid.entity)))
                                .id();
                            grid.get_building_mut(coords).unwrap().replace(pipe);
                        }
                    }
                    commands.entity(child).despawn();
                    commands.entity(switch).with_child(machine_port(
                        MachinePort {
                            side: new_dir,
                            flow: FlowDirection::Outlet,
                        },
                        flow_material.0.clone(),
                    ));
                    invalidate_networks.write(InvalidateNetworks);
                    commands.spawn((
                        Name::new("Switch Toggle Sound"),
                        AudioPlayer::new(sounds.switch.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }
        }
    }
}
