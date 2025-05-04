use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_camera);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, -100.0)));
}
