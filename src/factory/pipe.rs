use bevy::{asset::weak_handle, prelude::*};

use crate::{
    layers::FactoryLayer,
    materials::{Dither, DitherMaterial, SOLID_BLACK, SOLID_WHITE},
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    camera::CursorPosition,
    grid::{Direction, Grid, TileCoords, TILE_SIZE},
    machines::{FlowDirection, Machine, MachinePort},
    pipe_network::InvalidateNetworks,
    shop::PickedUpItem,
    time::TimeScale,
};

pub const PIPE_MESH: Handle<Mesh> = weak_handle!("2c48ec14-5922-4888-b929-bfcb0d1379a5");
pub const PIPE_MESH_INNER: Handle<Mesh> = weak_handle!("b67e0c8e-a3ad-4477-b865-e30799506a15");
pub const PIPE_BRIDGE: Handle<Mesh> = weak_handle!("19d40193-0720-4cf3-accd-445eb2d6f3f2");
pub const PIPE_BRIDGE_INNER: Handle<Mesh> = weak_handle!("61135ba2-b044-4dc6-bb2e-41c3c1f4aadf");

pub(super) fn plugin(app: &mut App) {
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

    app.add_systems(Startup, pipe_setup_materials.in_set(Sets::Init))
        .add_systems(
            Update,
            (
                pipe_draw.in_set(Sets::Input),
                pipe_connect.in_set(Sets::Physics),
                (pipe_update_material, pipe_make_bridges).in_set(Sets::PostUpdate),
            ),
        )
        .add_observer(pipe_cleanup_connections);
}

const PIPE_SIZE: f32 = TILE_SIZE * 0.6;

#[derive(Component, Clone, Default, Debug)]
pub struct Pipe {
    pub to: Option<Entity>,
    pub from: Option<Entity>,
}

pub fn pipe_bundle(pos: IVec2) -> impl Bundle {
    (
        Name::new("Pipe"),
        Pipe::default(),
        FactoryLayer,
        TileCoords(pos),
        ZOrder::PIPE,
        Mesh2d(PIPE_MESH),
        MeshMaterial2d(SOLID_WHITE),
        children![(
            Name::new("Pipe Inner"),
            FactoryLayer,
            Mesh2d(PIPE_MESH_INNER),
            MeshMaterial2d(SOLID_BLACK),
            Transform::from_xyz(0.0, 0.0, 0.01),
        )],
    )
}

fn pipe_draw(
    mut commands: Commands,
    pipes: Query<&Pipe>,
    cursor_pos: CursorPosition,
    mut grid: ResMut<Grid>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut invalidate: EventWriter<InvalidateNetworks>,
    picked_up_item: Option<Res<PickedUpItem>>,
) {
    if !buttons.pressed(MouseButton::Left) && !buttons.pressed(MouseButton::Right) {
        return;
    }
    if picked_up_item.is_some() {
        return;
    }
    if let Some(tile_pos) = cursor_pos.tile() {
        if grid.get_tile(tile_pos).is_some() {
            let maybe_building = grid.get_building(tile_pos);
            if buttons.pressed(MouseButton::Left) && maybe_building.is_none() {
                info!("Drawing pipe at tile: {:?}", tile_pos);
                let pipe = commands
                    .spawn((pipe_bundle(tile_pos), ChildOf(grid.entity)))
                    .id();
                grid.insert_building(tile_pos, pipe);
                invalidate.write(InvalidateNetworks);
            } else if buttons.pressed(MouseButton::Right) && maybe_building.is_some() {
                let building = maybe_building.unwrap();
                if pipes.contains(building) {
                    info!("Removing pipe at tile: {:?}", tile_pos);
                    commands.entity(building).despawn();
                    grid.remove_building(tile_pos);
                    invalidate.write(InvalidateNetworks);
                }
            }
        }
    }
}

fn pipe_connect(
    mut pipes: Query<(Entity, Mut<Pipe>)>,
    coords: Query<&TileCoords>,
    mut ports: Query<(Entity, &mut MachinePort)>,
    machines: Query<&Children, With<Machine>>,
    grid: ResMut<Grid>,
) {
    let pipe_entities = pipes
        .iter()
        .filter(|(_, pipe)| pipe.is_changed())
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for pipe_entity in pipe_entities {
        let pipe_coords = coords.get(pipe_entity).unwrap();
        for neighbor_dir in Direction::iter() {
            if let Some(neighbor) = grid.get_building(pipe_coords.0 + neighbor_dir.as_ivec2()) {
                if let Ok(children) = machines.get(neighbor) {
                    let maybe_port = children
                        .iter()
                        .filter_map(|c| ports.get(c).ok())
                        .find(|(_, port)| port.side == neighbor_dir.flip());
                    if let Some((neighbor, _)) = maybe_port {
                        let (_, mut pipe) = pipes.get_mut(pipe_entity).unwrap();
                        let (_, mut port) = ports.get_mut(neighbor).unwrap();
                        if pipe.to.is_none() && port.flow == FlowDirection::Inlet {
                            pipe.to = Some(neighbor);
                            port.connected = Some(pipe_entity);
                        } else if pipe.from.is_none() && port.flow == FlowDirection::Outlet {
                            pipe.from = Some(neighbor);
                            port.connected = Some(pipe_entity);
                        }
                    }
                } else if pipes.contains(neighbor) {
                    let [(_, mut pipe), (_, mut nbr_pipe)] =
                        pipes.get_many_mut([pipe_entity, neighbor]).unwrap();
                    if pipe.to.is_some_and(|e| e == neighbor)
                        || pipe.from.is_some_and(|e| e == neighbor)
                    {
                        // already connected
                        continue;
                    } else if pipe.to.is_none() && nbr_pipe.from.is_none() {
                        pipe.to = Some(neighbor);
                        nbr_pipe.from = Some(pipe_entity);
                    } else if pipe.from.is_none() && nbr_pipe.to.is_none() {
                        pipe.from = Some(neighbor);
                        nbr_pipe.to = Some(pipe_entity);
                    }
                }
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct PipeBridge;

fn pipe_make_bridges(
    mut commands: Commands,
    pipes: Query<(Entity, &Pipe, &TileCoords, &Children), Changed<Pipe>>,
    bridges: Query<&PipeBridge>,
    ports: Query<&ChildOf, With<MachinePort>>,
    coords: Query<&TileCoords>,
    flow_material: Res<PipeFlowMaterial>,
) -> Result {
    for (pipe_entity, pipe, pipe_coords, pipe_children) in pipes.iter() {
        for child in pipe_children.iter() {
            if bridges.contains(child) {
                commands.entity(child).despawn();
            }
        }
        let mut bridges = vec![];
        if let Some(to) = pipe.to {
            bridges.push((false, to));
        }
        if let Some(from) = pipe.from {
            bridges.push((true, from));
        }
        for (flip, entity) in bridges {
            let nbr = if let Ok(child_of) = ports.get(entity) {
                child_of.parent()
            } else {
                entity
            };
            let nbr_coords = coords.get(nbr)?;
            let dir = pipe_coords.direction_to(nbr_coords);
            commands.spawn((
                pipe_bridge(flip, dir, flow_material.0.clone()),
                ChildOf(pipe_entity),
            ));
        }
    }

    Ok(())
}

fn pipe_cleanup_connections(
    trigger: Trigger<OnRemove, Pipe>,
    mut pipes: Query<&mut Pipe>,
    mut ports: Query<&mut MachinePort>,
) {
    let target = trigger.target();
    let pipe = pipes.get(target).unwrap();
    let (to, from) = (pipe.to, pipe.from);
    if let Some(to) = to {
        if let Ok(mut pipe) = pipes.get_mut(to) {
            pipe.from = None;
        } else if let Ok(mut port) = ports.get_mut(to) {
            port.connected = None;
        }
    }
    if let Some(from) = from {
        if let Ok(mut pipe) = pipes.get_mut(from) {
            pipe.to = None;
        } else if let Ok(mut port) = ports.get_mut(from) {
            port.connected = None;
        }
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
        Pickable::IGNORE, // https://github.com/bevyengine/bevy/issues/19181
        Mesh2d(PIPE_BRIDGE),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(offset.x, offset.y, 0.05)
            .with_rotation(Quat::from_rotation_z(dir.angle())),
        children![(
            Name::new("Pipe Bridge Inner"),
            FactoryLayer,
            Pickable::IGNORE, // https://github.com/bevyengine/bevy/issues/19181
            Mesh2d(PIPE_BRIDGE_INNER),
            MeshMaterial2d(flow_material),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}

#[derive(Resource)]
pub struct PipeFlowMaterial(pub Handle<DitherMaterial>);

fn pipe_setup_materials(mut commands: Commands, mut materials: ResMut<Assets<DitherMaterial>>) {
    let material = materials.add(Dither {
        fill: 0.01,
        scale: 44.0,
        ..default()
    });
    commands.insert_resource(PipeFlowMaterial(material));
}

fn pipe_update_material(
    material: Res<PipeFlowMaterial>,
    time: Res<Time>,
    timescale: Res<TimeScale>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    if let Some(material) = materials.get_mut(&material.0) {
        material.settings.offset.x -= time.delta_secs() * timescale.0 * 10.0;
    }
}
