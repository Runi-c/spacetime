use bevy::prelude::*;

use crate::game_over::RestartGame;

use super::grid::{setup_grid, Grid};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_restart_factory);
}

fn on_restart_factory(_trigger: Trigger<RestartGame>, mut commands: Commands, grid: Res<Grid>) {
    commands.entity(grid.entity).despawn();
    commands.run_system_cached(setup_grid);
}
