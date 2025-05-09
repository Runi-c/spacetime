use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<TimeScale>();
}

#[derive(Resource)]
pub struct TimeScale(pub f32);
impl Default for TimeScale {
    fn default() -> Self {
        Self(1.0)
    }
}
