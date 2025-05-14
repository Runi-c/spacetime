use bevy::{asset::AssetMetaCheck, prelude::*, window::EnabledButtons};

#[cfg(feature = "dev")]
use bevy::remote::{http::RemoteHttpPlugin, RemotePlugin};

mod camera;
mod factory;
mod game_over;
mod layers;
mod materials;
mod mesh;
mod resources;
mod scheduling;
mod sounds;
mod space;
mod z_order;

pub const SCREEN_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

pub fn plugin(app: &mut App) {
    app // Bevy plugins
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space|Time".to_string(),
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
                .set(AssetPlugin {
                    // Required to fix a panic on itch.io
                    // https://github.com/bevyengine/bevy_github_ci_template/issues/48
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            MeshPickingPlugin,
            #[cfg(feature = "dev")]
            RemotePlugin::default(),
            #[cfg(feature = "dev")]
            RemoteHttpPlugin::default(),
        ))
        // Resource config
        .insert_resource(ClearColor(Color::BLACK))
        // Our plugins
        .add_plugins((
            camera::plugin,
            factory::plugin,
            game_over::plugin,
            materials::plugin,
            resources::plugin,
            scheduling::plugin,
            sounds::plugin,
            space::plugin,
            z_order::plugin,
        ));
}
