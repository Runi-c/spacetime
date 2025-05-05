use bevy::prelude::*;

mod camera;
mod grid;

pub fn plugin(app: &mut App) {
    app.add_plugins((camera::plugin, grid::plugin));
}
