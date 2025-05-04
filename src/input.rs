use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, exit_on_esc);
}

fn exit_on_esc(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.send_event(AppExit::Success);
    }
}
