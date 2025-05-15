use bevy::prelude::*;

use crate::resources::ResourceType;

mod ammo_factory;
mod hull_fixer;
mod inlet;
mod meshes;
mod outlet;
mod pipe_switch;
mod port;
mod rocket_factory;

pub use ammo_factory::ammo_factory;
pub use hull_fixer::hull_fixer;
pub use inlet::{inlet, Inlet};
pub use outlet::outlet;
pub use pipe_switch::pipe_switch;
pub use port::{FlowDirection, MachinePort};
pub use rocket_factory::rocket_factory;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ammo_factory::plugin,
        hull_fixer::plugin,
        inlet::plugin,
        meshes::plugin,
        outlet::plugin,
        pipe_switch::plugin,
        port::plugin,
        rocket_factory::plugin,
    ));
}

#[derive(Component, Clone, Default)]
pub struct Machine;

#[derive(Component, Clone, Debug)]
pub struct Buffer(pub ResourceType, pub f32);
