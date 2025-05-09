use bevy::{prelude::*, window::EnabledButtons};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod factory;
mod layers;
mod materials;
mod mesh;
mod resources;
mod scheduling;
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
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::new(),
        ))
        .add_plugins((
            materials::plugin,
            factory::plugin,
            resources::plugin,
            scheduling::plugin,
            space::plugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Update, exit_on_esc)
        .run();
}

fn exit_on_esc(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.send_event(AppExit::Success);
    }
}
