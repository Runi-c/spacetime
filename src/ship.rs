use bevy::prelude::*;
use parry2d::query;

use crate::{
    asteroid::Asteroid,
    collision::{transform_to_isometry, Collider},
    layers::Layers,
    mesh::MeshLyonExtensions,
    particles::SpawnParticles,
    physics::{Rotation, Velocity},
    utils::parry2d_vec2,
    z_order::ZOrder,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ship)
        .add_systems(Update, (ship_input, ship_laser));
}

#[derive(Component)]
pub struct Ship;

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const SIZE: f32 = 30.0;
    let vertices = vec![
        vec2(SIZE, 0.0),
        vec2(-SIZE, SIZE * 0.7),
        vec2(-SIZE * 0.5, 0.0),
        vec2(-SIZE, -SIZE * 0.7),
    ];

    let mesh = meshes.add(Mesh::fill_polygon(&vertices));
    commands.spawn((
        Ship,
        Layers::SPACE,
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Collider::from_vertices(&vertices),
        Transform::from_xyz(0.0, 0.0, ZOrder::SHIP),
        Velocity(Vec2::ZERO),
        Rotation(0.0),
    ));
}

fn ship_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Rotation, &mut Velocity), With<Ship>>,
) {
    const SPEED: f32 = 210.0;
    let mut input = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.x -= 1.0;
    }

    for (transform, mut rotation, mut velocity) in query.iter_mut() {
        let angle = transform.rotation.to_euler(EulerRot::YXZ).2;
        let acceleration = Quat::from_rotation_z(angle)
            * input.yzz().normalize_or_zero()
            * SPEED
            * time.delta_secs();
        velocity.0 += acceleration.xy();
        rotation.0 += input.x;
        if rotation.0 > 360.0 {
            rotation.0 -= 360.0;
        } else if rotation.0 < -360.0 {
            rotation.0 += 360.0;
        }
    }
}

pub fn ship_laser(
    time: Res<Time>,
    mut gizmos: Gizmos,
    ship: Query<(&Transform, &Collider), With<Ship>>,
    mut asteroids: Query<(&mut Asteroid, &Transform, &Collider)>,
    mut particles: EventWriter<SpawnParticles>,
) {
    for (ship_transform, ship_collider) in ship.iter() {
        for (mut asteroid, asteroid_transform, asteroid_collider) in asteroids.iter_mut() {
            let distance = ship_transform
                .translation
                .distance(asteroid_transform.translation);
            if distance < 300.0 {
                let contact = query::contact(
                    &transform_to_isometry(ship_transform),
                    &ship_collider.0,
                    &transform_to_isometry(asteroid_transform),
                    &asteroid_collider.0,
                    distance,
                )
                .unwrap();
                if let Some(contact) = contact {
                    if rand::random_bool(0.1) {
                        particles.write(SpawnParticles {
                            position: parry2d_vec2(contact.point2),
                            count: 1,
                        });
                    }
                    gizmos.line_2d(
                        ship_transform.translation.truncate(),
                        parry2d_vec2(contact.point2),
                        Color::WHITE,
                    );
                    asteroid.health -= time.delta_secs() * 25.0;
                }
            }
        }
    }
}
