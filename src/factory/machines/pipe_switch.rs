use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::{
    factory::{
        grid::{Direction, Grid, TileCoords, TILE_SIZE},
        machines::{
            meshes::{CONSTRUCTOR_MESH, CONSTRUCTOR_MESH_INNER},
            port::machine_port,
            Buffer, FlowDirection, MachinePort,
        },
        pipe::{pipe_bundle, Pipe, PipeFlowMaterial},
        pipe_network::{InNetwork, InvalidateNetworks, PipeNetwork},
        shop::ShopItem,
        time::FactoryTick,
        tooltip::Tooltip,
    },
    layers::FactoryLayer,
    materials::{DitherMaterial, SOLID_BLACK, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::ResourceType,
    scheduling::Sets,
    sounds::Sounds,
    z_order::ZOrder,
};

use super::Machine;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            pipe_switch_tick.in_set(Sets::Physics),
            pipe_switch_observers.in_set(Sets::PostUpdate),
        ),
    );
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
        MachinePort::new(Direction::Left, FlowDirection::Inlet),
        MachinePort::new(Direction::Right, FlowDirection::Outlet),
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

fn pipe_switch_tick(
    mut commands: Commands,
    mut ticks: EventReader<FactoryTick>,
    machines: Query<(Entity, &Buffer, &Children), (With<PipeSwitch>, With<TileCoords>)>,
    buffers: Query<&Buffer>,
    parents: Query<&ChildOf>,
    networks: Query<&PipeNetwork>,
    ports: Query<(&MachinePort, &InNetwork)>,
) -> Result {
    for _ in ticks.read() {
        for (entity, buffer, children) in machines.iter() {
            for (port, in_network) in children.iter().filter_map(|child| ports.get(child).ok()) {
                let network = networks.get(in_network.0)?;
                if port.flow == FlowDirection::Inlet && network.resource == ResourceType::Mineral {
                    let parent = parents.get(network.source)?;
                    let source = buffers.get(parent.0)?;
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

    Ok(())
}

fn pipe_switch_observers(mut commands: Commands, switches: Query<Entity, Added<PipeSwitch>>) {
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
                            grid.insert_building(coords, pipe);
                        }
                    }
                    commands.entity(child).despawn();
                    commands.entity(switch).with_child(machine_port(
                        MachinePort::new(new_dir, FlowDirection::Outlet),
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
