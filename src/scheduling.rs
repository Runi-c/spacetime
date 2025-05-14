use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.configure_sets(Startup, (Sets::Init, Sets::Spawn).chain())
        .configure_sets(
            Update,
            (
                Sets::PreUpdate,
                Sets::Input,
                Sets::Physics,
                Sets::Collision,
                Sets::Update,
                Sets::PostUpdate,
            )
                .chain(),
        );
}

#[derive(SystemSet, Hash, Debug, Clone, PartialEq, Eq)]
pub enum Sets {
    /// Used for initializing resources and essential entities
    Init,
    /// Used for spawning entities on startup
    Spawn,

    /// Used for ticking timers and spawning entities that might use a later set
    PreUpdate,
    /// Used for handling input events
    Input,
    /// Used for updating entity positions and physics
    Physics,
    /// Used for handling collision events after physics
    Collision,
    /// Used for miscellaneous entity updates
    Update,
    /// Used mainly for updating display components based on everything else this frame
    PostUpdate,
}
