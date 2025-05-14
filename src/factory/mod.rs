use bevy::prelude::*;

mod camera;
mod grid;
mod machines;
mod pipe;
mod restart;
mod shop;
mod time;
mod tooltip;
mod ui;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        camera::plugin,
        grid::plugin,
        machines::plugin,
        pipe::plugin,
        restart::plugin,
        shop::plugin,
        time::plugin,
        tooltip::plugin,
        ui::plugin,
    ));
}
