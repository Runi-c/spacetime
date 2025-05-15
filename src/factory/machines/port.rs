use bevy::prelude::*;

use crate::{
    factory::{
        grid::{Direction, Grid, TileCoords},
        pipe::{pipe_bridge, Pipe},
    },
    materials::DitherMaterial,
    scheduling::Sets,
};

use super::Machine;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MachinePort>()
        .add_systems(Update, machine_port_connect.in_set(Sets::Update))
        .add_observer(machine_port_cleanup);
}

#[derive(Reflect, Component, Clone, Debug)]
pub struct MachinePort {
    pub side: Direction,
    pub flow: FlowDirection,
    pub connected: Option<Entity>,
}

impl MachinePort {
    pub fn new(side: Direction, flow: FlowDirection) -> Self {
        Self {
            side,
            flow,
            connected: None,
        }
    }
}

#[derive(Reflect, Clone, PartialEq, Eq, Debug)]
pub enum FlowDirection {
    Inlet,
    Outlet,
}

pub fn machine_port(port: MachinePort, flow_material: Handle<DitherMaterial>) -> impl Bundle {
    (
        Name::new("Machine Port"),
        Transform::IDENTITY,
        Visibility::Inherited,
        children![pipe_bridge(
            port.flow == FlowDirection::Inlet,
            port.side,
            flow_material
        )],
        port,
    )
}

fn machine_port_connect(
    mut ports: Query<(Entity, &ChildOf, Mut<MachinePort>)>,
    machines: Query<(&TileCoords, &Children), With<Machine>>,
    mut pipes: Query<&mut Pipe>,
    grid: Res<Grid>,
) {
    let added_ports = ports
        .iter()
        .filter(|(_, _, port)| port.is_changed())
        .map(|(entity, child_of, _)| (entity, child_of.clone()))
        .collect::<Vec<_>>();
    for (port_entity, child_of) in added_ports.into_iter() {
        if let Ok((coords, _)) = machines.get(child_of.parent()) {
            let (_, _, port) = ports.get(port_entity).unwrap();
            info!("Connecting port at: {:?}", coords);
            if let Some(neighbor) = grid.get_building(coords.0 + port.side.as_ivec2()) {
                if let Ok((_, children)) = machines.get(neighbor) {
                    // neighbor is a machine
                    let maybe_port = children.iter().filter_map(|c| ports.get(c).ok()).find(
                        |(_, _, nbr_port)| {
                            info!("Neighbor port: {:?}", nbr_port);
                            port.side == nbr_port.side.flip() && port.flow != nbr_port.flow
                        },
                    );
                    if let Some((nbr_port_entity, _, nbr_port)) = maybe_port {
                        info!("Connecting ports: {:?} -> {:?}", nbr_port, port);
                        let [(_, _, mut port), (_, _, mut nbr_port)] =
                            ports.get_many_mut([port_entity, nbr_port_entity]).unwrap();
                        port.connected = Some(nbr_port_entity);
                        nbr_port.connected = Some(port_entity);
                    }
                } else if pipes.contains(neighbor) {
                    // neighbor is a pipe
                    let mut nbr_pipe = pipes.get_mut(neighbor).unwrap();
                    let (_, _, mut port) = ports.get_mut(port_entity).unwrap();
                    if port.flow == FlowDirection::Inlet && nbr_pipe.to.is_none() {
                        port.connected = Some(neighbor);
                        nbr_pipe.to = Some(port_entity);
                    } else if port.flow == FlowDirection::Outlet && nbr_pipe.from.is_none() {
                        port.connected = Some(neighbor);
                        nbr_pipe.from = Some(port_entity);
                    }
                }
            }
        }
    }
}

fn machine_port_cleanup(
    trigger: Trigger<OnRemove, MachinePort>,
    mut pipes: Query<&mut Pipe>,
    mut ports: Query<&mut MachinePort>,
) {
    let target = trigger.target();
    let port = ports.get(target).unwrap();
    if let Some(connected) = port.connected {
        if let Ok(mut pipe) = pipes.get_mut(connected) {
            if pipe.to == Some(target) {
                pipe.to = None;
            } else if pipe.from == Some(target) {
                pipe.from = None;
            }
        } else if let Ok(mut port) = ports.get_mut(connected) {
            port.connected = None;
        }
    }
}
