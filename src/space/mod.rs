use bevy::prelude::*;

mod asteroid;
mod bg;
mod bounds;
mod camera;
mod collision;
mod enemy;
mod particles;
mod physics;
mod pickup;
mod ship;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        asteroid::plugin,
        bg::plugin,
        camera::plugin,
        collision::plugin,
        enemy::plugin,
        particles::plugin,
        physics::plugin,
        pickup::plugin,
        ship::plugin,
    ));
}
