use bevy::{prelude::*, window::EnabledButtons};

mod asteroid;
mod camera;
mod collision;
mod input;
mod physics;
mod ship;
//mod z_order;

pub const SCREEN_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space/Time".to_string(),
                        resizable: false,
                        resolution: SCREEN_SIZE.into(),
                        enabled_buttons: EnabledButtons {
                            maximize: false,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((
            asteroid::plugin,
            camera::plugin,
            collision::plugin,
            input::plugin,
            physics::plugin,
            ship::plugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .run();
}
