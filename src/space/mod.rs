use bevy::prelude::*;

mod asteroid;
mod bg;
mod bounds;
mod camera;
mod collision;
mod enemy;
mod gas;
mod particles;
mod physics;
mod pickup;
mod restart;
mod ship;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        asteroid::plugin,
        bg::plugin,
        camera::plugin,
        collision::plugin,
        enemy::plugin,
        gas::plugin,
        particles::plugin,
        physics::plugin,
        pickup::plugin,
        restart::plugin,
        ship::plugin,
    ));
}
