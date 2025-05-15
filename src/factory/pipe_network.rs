use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    factory::machines::FlowDirection, materials::DitherMaterial, resources::ResourceType,
    scheduling::Sets,
};

use super::{
    grid::TileCoords,
    machines::{Buffer, Machine, MachinePort},
    pipe::Pipe,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<NetworkMembers>()
        .register_type::<InNetwork>()
        .register_type::<PipeNetwork>()
        .add_systems(
            Update,
            (
                network_rebuild.in_set(Sets::Update),
                network_debug.in_set(Sets::PostUpdate),
            ),
        )
        .add_event::<InvalidateNetworks>();
}

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

fn network_rebuild(
    mut commands: Commands,
    mut invalidations: EventReader<InvalidateNetworks>,
    networks: Query<Entity, With<PipeNetwork>>,
    machines: Query<(&Buffer, &Children), (With<Machine>, With<TileCoords>)>,
    ports: Query<(Entity, &ChildOf, &MachinePort)>,
    pipes: Query<(&Pipe, &TileCoords)>,
) {
    if invalidations.is_empty() {
        return;
    }
    invalidations.clear();

    for network in networks.iter() {
        commands.entity(network).despawn();
    }
    info!("Rebuilding pipe networks");
    let mut networks = vec![];
    for (port_entity, child_of, port) in ports.iter() {
        if port.flow == FlowDirection::Inlet {
            continue;
        }
        let parent = child_of.parent();
        if !machines.contains(parent) {
            continue;
        }
        let (buffer, _) = machines.get(child_of.parent()).unwrap();
        let mut sink = None;
        let mut members = vec![];
        if let Some(connected) = port.connected {
            if pipes.contains(connected) {
                // neighbor is pipe
                let mut visited = HashSet::new();
                let mut current = Some(connected);
                while let Some(pipe) = current {
                    if !visited.insert(pipe) {
                        break;
                    }
                    members.push(pipe);
                    current = pipes.get(pipe).ok().and_then(|(pipe, _)| pipe.to);
                }
                if let Some(&last) = members.last() {
                    if ports.contains(last) {
                        sink = Some(last);
                    }
                };
            } else if let Ok((nbr_port, _, _)) = ports.get(connected) {
                // neighbor is machine
                sink = Some(nbr_port);
                members.push(nbr_port);
            }
        }
        members.push(port_entity);

        networks.push((port_entity, buffer.0, sink, members));
    }

    for (source, resource, sink, members) in networks {
        let network = commands
            .spawn((
                Name::new(format!("{} Network", resource.to_string())),
                PipeNetwork {
                    source,
                    resource,
                    sink,
                    material: Handle::default(),
                },
                NetworkMembers(members.clone()),
            ))
            .id();
        commands.entity(source).insert(InNetwork(network));
        if let Some(sink) = sink {
            commands.entity(sink).insert(InNetwork(network));
        }
    }
}

fn network_debug(
    networks: Query<&PipeNetwork, Added<PipeNetwork>>,
    child_of: Query<&ChildOf>,
    names: Query<&Name>,
) -> Result {
    for network in networks.iter() {
        let source = names.get(child_of.get(network.source)?.parent())?;
        let sink = if let Some(sink) = network.sink {
            names.get(child_of.get(sink)?.parent())?
        } else {
            &Name::new("None")
        };
        info!("Pipe network: {} -> {}", source.as_str(), sink.as_str());
    }

    Ok(())
}
