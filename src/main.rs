use bevy::{prelude::*, sprite::Material2dPlugin, window::EnabledButtons};
use dither::DitherMaterial;

mod asteroid;
mod cameras;
mod collision;
mod dither;
mod grid;
mod input;
mod layers;
mod mesh;
mod particles;
mod physics;
mod ship;
mod space;
mod utils;
mod z_order;

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
        .add_plugins(Material2dPlugin::<DitherMaterial>::default())
        .add_plugins((
            asteroid::plugin,
            cameras::plugin,
            collision::plugin,
            grid::plugin,
            input::plugin,
            particles::plugin,
            physics::plugin,
            ship::plugin,
            space::plugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .run();
}
