use bevy::prelude::*;
use bevy_asset_loader::asset_collection::{AssetCollection, AssetCollectionApp};

use crate::scheduling::Sets;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_music.in_set(Sets::Spawn))
        .init_collection::<Sounds>();
}

fn spawn_music(mut commands: Commands, sounds: Res<Sounds>) {
    commands.spawn((
        Name::new("Background Music"),
        AudioPlayer::new(sounds.music.clone()),
        PlaybackSettings::LOOP,
    ));
}

#[derive(AssetCollection, Resource)]
pub struct Sounds {
    #[asset(path = "sounds/laser.ogg")]
    pub laser: Handle<AudioSource>,

    #[asset(path = "sounds/ambient.ogg")]
    pub music: Handle<AudioSource>,

    #[asset(path = "sounds/asteroid_break.ogg")]
    pub asteroid_break: Handle<AudioSource>,

    #[asset(path = "sounds/asteroid_bump.ogg")]
    pub asteroid_bump: Handle<AudioSource>,

    #[asset(path = "sounds/succ.ogg")]
    pub succ: Handle<AudioSource>,

    #[asset(path = "sounds/switch.ogg")]
    pub switch: Handle<AudioSource>,

    #[asset(path = "sounds/enemy_gamer.ogg")]
    pub enemy_gamer: Handle<AudioSource>,

    #[asset(path = "sounds/enemy_gun.ogg")]
    pub enemy_gun: Handle<AudioSource>,

    #[asset(path = "sounds/gun.ogg")]
    pub gun: Handle<AudioSource>,

    #[asset(path = "sounds/pickup_machine.ogg")]
    pub pickup_machine: Handle<AudioSource>,

    #[asset(path = "sounds/place_machine.ogg")]
    pub place_machine: Handle<AudioSource>,

    #[asset(path = "sounds/rocket.ogg")]
    pub rocket: Handle<AudioSource>,

    #[asset(path = "sounds/missile_launch.ogg")]
    pub missile_launch: Handle<AudioSource>,

    #[asset(path = "sounds/missile_flight.ogg")]
    pub missile_flight: Handle<AudioSource>,

    #[asset(path = "sounds/player_death.ogg")]
    pub player_death: Handle<AudioSource>,

    #[asset(path = "sounds/enemy_die.ogg")]
    pub enemy_die: Handle<AudioSource>,
}
