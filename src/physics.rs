use bevy::prelude::*;

use crate::SCREEN_SIZE;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (physics_move, physics_spin));
}

#[derive(Component)]
pub struct Velocity(pub Vec2);
#[derive(Component)]
pub struct Rotation(pub f32);

fn physics_move(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        const WRAP_BUFFER: f32 = 40.0;
        const HALF_SCREEN_SIZE: Vec2 = Vec2::new(
            SCREEN_SIZE.x / 2.0 + WRAP_BUFFER,
            SCREEN_SIZE.y / 2.0 + WRAP_BUFFER,
        );
        if transform.translation.x > HALF_SCREEN_SIZE.x {
            transform.translation.x = -HALF_SCREEN_SIZE.x;
        } else if transform.translation.x < -HALF_SCREEN_SIZE.x {
            transform.translation.x = HALF_SCREEN_SIZE.x;
        }
        if transform.translation.y > HALF_SCREEN_SIZE.y {
            transform.translation.y = -HALF_SCREEN_SIZE.y;
        } else if transform.translation.y < -HALF_SCREEN_SIZE.y {
            transform.translation.y = HALF_SCREEN_SIZE.y;
        }
    }
}

fn physics_spin(mut query: Query<(&mut Transform, &Rotation)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(rotation.0.to_radians());
    }
}
