use bevy::prelude::*;

use crate::{camera::UICamera, resources::Resources, scheduling::Sets};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, restart_game.in_set(Sets::PreUpdate))
        .add_observer(on_game_over);
}

#[derive(Event)]
pub struct GameOver;

#[derive(Component)]
pub struct GameOverScreen;

fn on_game_over(
    _trigger: Trigger<GameOver>,
    mut commands: Commands,
    ui_camera: Single<Entity, With<UICamera>>,
) {
    commands.spawn((
        Name::new("Gameover Screen"),
        GameOverScreen,
        UiTargetCamera(*ui_camera),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Node {
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            BorderColor(Color::WHITE),
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
            Text("You died!\nPress R to restart.".to_string()),
        )],
    ));
}

#[derive(Event)]
pub struct RestartGame;

fn restart_game(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_over_screen: Option<Single<Entity, With<GameOverScreen>>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        commands.trigger(GameOver)
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        if let Some(game_over_screen) = game_over_screen {
            commands.entity(*game_over_screen).despawn();
        }
        commands.trigger(RestartGame);
        commands.insert_resource(Resources::default());
    }
}
