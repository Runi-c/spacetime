use bevy::prelude::*;
use parry2d::query;
use rand::Rng;

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, MetalDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    scheduling::Sets,
    utils::parry2d_vec2,
    z_order::ZOrder,
};

use super::{
    asteroid::Asteroid,
    collision::{transform_to_isometry, Collider},
    particles::SpawnParticles,
    physics::{Rotation, Velocity},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ship.in_set(Sets::Spawn))
        .add_systems(
            Update,
            (
                ship_input.in_set(Sets::Input),
                ship_laser.in_set(Sets::Update),
                (display_ship_health, destroy_ship)
                    .chain()
                    .in_set(Sets::PostUpdate),
            ),
        );
}

#[derive(Component, Clone)]
pub struct Ship {
    pub health: f32,
}

#[derive(Component, Clone)]
pub struct ShipHealthMarker;

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
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
        Name::new("Ship"),
        Ship { health: 100.0 },
        SpaceLayer,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(MetalDither {
            fill: 0.2,
            scale: SIZE,
        })),
        Collider::from_vertices(&vertices),
        ZOrder::SHIP,
        Velocity(Vec2::ZERO),
        Rotation(0.0),
        Visibility::Visible,
        children![(
            Name::new("Ship Health"),
            ShipHealthMarker,
            SpaceLayer,
            Mesh2d(mesh),
            MeshMaterial2d(SOLID_WHITE),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    ));
}

fn destroy_ship(
    mut commands: Commands,
    query: Query<(Entity, &Ship, &Transform), With<Ship>>,
    mut particles: EventWriter<SpawnParticles>,
) {
    for (entity, ship, transform) in query.iter() {
        if ship.health <= 0.0 {
            commands.entity(entity).despawn();
            particles.write(SpawnParticles {
                position: transform.translation.truncate(),
                count: 10,
            });
        }
    }
}

fn ship_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Rotation, &mut Velocity), With<Ship>>,
) {
    const SPEED: f32 = 300.0;
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
    mut asteroids: Query<(Entity, &mut Asteroid, &Transform, &Collider)>,
    mut particles: EventWriter<SpawnParticles>,
) {
    for (ship_transform, ship_collider) in ship.iter() {
        let closest = asteroids.iter().fold(
            None,
            |closest: Option<(f32, Entity)>, (entity, _, transform, _)| {
                let distance = ship_transform.translation.distance(transform.translation);
                if closest.is_none() || distance < closest.unwrap().0 {
                    Some((distance, entity))
                } else {
                    closest
                }
            },
        );
        if let Some((distance, asteroid_entity)) = closest {
            let (_, mut asteroid, asteroid_transform, asteroid_collider) =
                asteroids.get_mut(asteroid_entity).unwrap();
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
                    if rand::thread_rng().gen_bool(0.1) {
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

fn display_ship_health(
    mut commands: Commands,
    ship: Query<(Entity, &Ship), Changed<Ship>>,
    health_marker: Query<(Entity, &ChildOf, &Transform), With<ShipHealthMarker>>,
) {
    for (entity, ship) in ship.iter() {
        for (marker_entity, child, transform) in health_marker.iter() {
            if child.parent == entity {
                let scale = ship.health / 100.0;
                commands.entity(marker_entity).insert(Transform {
                    scale: Vec3::splat(scale),
                    ..*transform
                });
            }
        }
    }
}
