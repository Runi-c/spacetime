use std::ops::Range;

use bevy::prelude::*;
use rand::Rng;

use crate::scheduling::Sets;

use super::camera::SpaceCamera;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (physics_move, physics_spin).in_set(Sets::Physics),
            physics_rotation.in_set(Sets::PostUpdate),
        ),
    );
}

#[derive(Component, Clone)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn random(speed: Range<f32>) -> Self {
        Self(
            Vec2::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
            )
            .normalize()
                * rand::thread_rng().gen_range(speed),
        )
    }

    pub fn random_towards(dir: Vec2, speed: Range<f32>) -> Self {
        Self(dir.normalize() * rand::thread_rng().gen_range(speed))
    }
}

#[derive(Component, Clone)]
pub struct Rotation(pub f32);

impl Rotation {
    pub fn random() -> Self {
        Self(rand::random::<f32>() * 360.0)
    }
}

#[derive(Component, Clone)]
#[require(Rotation(0.0))]
pub struct Spin(pub f32);

impl Spin {
    pub fn random(range: f32) -> Self {
        Self(rand::random::<f32>() * range * 2.0 - range)
    }
}

#[derive(Component, Clone)]
pub struct DespawnOutOfBounds;

fn physics_move(
    mut commands: Commands,
    time: Res<Time>,
    camera: Single<(&Camera, &Projection), With<SpaceCamera>>,
    mut query: Query<(Entity, &mut Transform, &Velocity, Has<DespawnOutOfBounds>)>,
) {
    let (camera, projection) = *camera;
    for (entity, mut transform, velocity, despawn) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        const WRAP_BUFFER: f32 = 40.0;
        let scale = match projection {
            Projection::Orthographic(OrthographicProjection { scale, .. }) => *scale,
            _ => 1.0,
        };
        let viewport_size =
            camera.physical_viewport_size().unwrap().as_vec2() * scale * 0.5 + WRAP_BUFFER;

        let mut moved = false;
        if transform.translation.x > viewport_size.x {
            transform.translation.x = -viewport_size.x;
            moved = true;
        } else if transform.translation.x < -viewport_size.x {
            transform.translation.x = viewport_size.x;
            moved = true;
        }
        if transform.translation.y > viewport_size.y {
            transform.translation.y = -viewport_size.y;
            moved = true;
        } else if transform.translation.y < -viewport_size.y {
            transform.translation.y = viewport_size.y;
            moved = true;
        }
        if moved && despawn {
            commands.entity(entity).despawn();
        }
    }
}

fn physics_spin(time: Res<Time>, mut query: Query<(&mut Rotation, &Spin)>) {
    for (mut rotation, spin) in query.iter_mut() {
        rotation.0 += spin.0 * time.delta_secs();
        if rotation.0 > 360.0 {
            rotation.0 -= 360.0;
        } else if rotation.0 < 0.0 {
            rotation.0 += 360.0;
        }
    }
}

fn physics_rotation(mut query: Query<(&mut Transform, &Rotation)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(rotation.0.to_radians());
    }
}
