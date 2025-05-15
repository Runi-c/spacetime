use bevy::prelude::*;

use crate::game_over::RestartGame;

use super::grid::{grid_spawn, Grid};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(factory_restart);
}

fn factory_restart(_trigger: Trigger<RestartGame>, mut commands: Commands, grid: Res<Grid>) {
    commands.entity(grid.entity).despawn();
    commands.run_system_cached(grid_spawn);
}
