use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.configure_sets(Startup, (Sets::Init, Sets::Spawn).chain())
        .configure_sets(
            Update,
            (Sets::Input, Sets::Physics, Sets::Update, Sets::PostUpdate).chain(),
        );
}

#[derive(SystemSet, Hash, Debug, Clone, PartialEq, Eq)]
pub enum Sets {
    Init,
    Spawn,
    Input,
    Physics,
    Collision,
    Update,
    PostUpdate,
}
