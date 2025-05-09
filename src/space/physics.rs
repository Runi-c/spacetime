use std::{
    f32::consts::{PI, TAU},
    ops::Range,
};

use bevy::prelude::*;

use crate::scheduling::Sets;

use super::camera::SpaceCamera;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (physics_move, physics_spin).in_set(Sets::Physics),
            physics_rotation.in_set(Sets::PostUpdate),
        ),
    );
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn random(speed: Range<f32>) -> Self {
        Self(
            Vec2::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
            )
            .normalize()
                * rand::random_range(speed),
        )
    }
}

#[derive(Component)]
pub struct Rotation(pub f32);

impl Rotation {
    pub fn random() -> Self {
        Self(rand::random::<f32>() * TAU)
    }
}

#[derive(Component)]
#[require(Rotation(0.0))]
pub struct Spin(pub f32);

impl Spin {
    pub fn random() -> Self {
        Self(rand::random::<f32>() * TAU - PI)
    }
}

fn physics_move(
    time: Res<Time>,
    camera: Query<(&Camera, &Projection), With<SpaceCamera>>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    let (camera, projection) = camera.single().unwrap();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        const WRAP_BUFFER: f32 = 40.0;
        let scale = match projection {
            Projection::Orthographic(OrthographicProjection { scale, .. }) => *scale,
            _ => 1.0,
        };
        let viewport_size =
            camera.physical_viewport_size().unwrap().as_vec2() * scale * 0.5 + WRAP_BUFFER;

        if transform.translation.x > viewport_size.x {
            transform.translation.x = -viewport_size.x;
        } else if transform.translation.x < -viewport_size.x {
            transform.translation.x = viewport_size.x;
        }
        if transform.translation.y > viewport_size.y {
            transform.translation.y = -viewport_size.y;
        } else if transform.translation.y < -viewport_size.y {
            transform.translation.y = viewport_size.y;
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
