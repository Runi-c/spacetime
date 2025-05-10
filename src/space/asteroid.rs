use std::time::Duration;

use bevy::prelude::*;
use lyon_tessellation::StrokeOptions;
use rand::{distributions::uniform::SampleRange, Rng};

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, RockyDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    scheduling::Sets,
    sounds::Sounds,
    z_order::ZOrder,
};

use super::{
    bounds::ScreenBounds,
    collision::{Collider, CollisionEvent},
    particles::SpawnParticles,
    physics::{Spin, Velocity},
    pickup::{asteroid_piece, time_piece},
    ship::Ship,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            collide_asteroid.in_set(Sets::Update),
            (asteroid_timer, break_asteroid, display_asteroid_health).in_set(Sets::PostUpdate),
        ),
    );
}

#[derive(Component, Clone)]
pub struct Asteroid {
    pub health: f32,
}

fn asteroid_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    asteroids: Query<&Asteroid>,
) {
    let asteroid_count = asteroids.iter().count();
    if asteroid_count >= 10 {
        return;
    }
    if let Some(timer) = timer.as_mut() {
        if timer.tick(time.delta()).just_finished() {
            commands.run_system_cached(spawn_asteroid);
        }
    } else {
        timer.replace(Timer::new(
            Duration::from_secs_f32(5.0),
            TimerMode::Repeating,
        ));
        commands.run_system_cached(spawn_asteroid);
    }
}

pub fn make_asteroid_polygon(
    vertex_range: impl SampleRange<i32>,
    radius_range: impl SampleRange<f32> + Clone,
) -> Vec<Vec2> {
    let mut vertices = vec![];
    let corner_count = rand::thread_rng().gen_range(vertex_range);
    for i in 0..corner_count {
        let radius = rand::thread_rng().gen_range(radius_range.clone());
        let angle = (i as f32 / corner_count as f32) * std::f32::consts::PI * 2.0;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        vertices.push(vec2(x, y));
    }
    vertices
}

fn spawn_asteroid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    bounds: ScreenBounds,
) {
    let pos = bounds.random_outside() * 1.2;
    let vertices = make_asteroid_polygon(9..=15, 25.0..=50.0);
    let target = bounds.random_inside() * 0.8;
    commands.spawn((
        Name::new("Asteroid"),
        Asteroid { health: 100.0 },
        SpaceLayer,
        Mesh2d(meshes.add(Mesh::stroke_polygon(
            &vertices,
            &StrokeOptions::default().with_line_width(5.0),
        ))),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        ZOrder::ASTEROID,
        Collider::from_vertices(&vertices),
        Velocity::random_towards(target - pos, 60.0..150.0),
        Spin::random(),
        children![(
            SpaceLayer,
            Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
            MeshMaterial2d(materials.add(RockyDither {
                fill: 0.8,
                scale: 50.0,
            })),
            Transform::from_xyz(0.0, 0.0, 1.0),
        )],
    ));
}

fn collide_asteroid(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut particle_writer: EventWriter<SpawnParticles>,
    asteroids: Query<&Velocity, With<Asteroid>>,
    ships: Query<(&Ship, &Transform, &Velocity)>,
    sounds: Res<Sounds>,
) {
    for event in collision_events.read() {
        if asteroids.contains(event.entity_a) {
            if ships.contains(event.entity_b) {
                // Handle collision between asteroid and ship
                let asteroid_velocity = asteroids.get(event.entity_a).unwrap();
                let contact = event.contact;
                let (ship, transform, velocity) = ships.get(event.entity_b).unwrap();
                let contact_point = vec2(contact.point2.x, contact.point2.y);
                let contact_normal = vec2(contact.normal1.x, contact.normal1.y);
                commands.entity(event.entity_b).insert((
                    Ship {
                        health: ship.health - 10.0,
                    },
                    Transform::from_translation(
                        transform.translation + contact_normal.extend(0.0) * -(contact.dist - 1.0),
                    ),
                    Velocity(
                        contact_normal * (velocity.0.length() * 0.5).max(100.0)
                            + asteroid_velocity.0 * 0.5,
                    ),
                ));
                particle_writer.write(SpawnParticles {
                    position: contact_point,
                    count: 10,
                });
                info!("Ship collided with asteroid at {:?}", contact_point);
                commands.spawn((
                    Name::new("Asteroid Bump Sound"),
                    AudioPlayer::new(sounds.asteroid_bump.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}

fn break_asteroid(
    mut commands: Commands,
    asteroids: Query<(Entity, &Transform, &Asteroid)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    sounds: Res<Sounds>,
) {
    for (entity, transform, asteroid) in asteroids.iter() {
        if asteroid.health <= 0.0 {
            commands.entity(entity).despawn();
            for _ in 0..4 {
                commands.spawn(asteroid_piece(
                    transform.translation.truncate(),
                    &mut meshes,
                    &mut materials,
                ));
            }
            if rand::thread_rng().gen_bool(0.5) {
                commands.spawn(time_piece(
                    transform.translation.truncate(),
                    &mut meshes,
                    &mut materials,
                ));
            }
            commands.spawn((
                Name::new("Asteroid Break Sound"),
                AudioPlayer::new(sounds.asteroid_break.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn display_asteroid_health(
    asteroids: Query<(&Asteroid, &Children)>,
    material_handles: Query<&MeshMaterial2d<DitherMaterial>, With<ChildOf>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    for (asteroid, children) in asteroids.iter() {
        for child in children.iter() {
            if let Ok(material) = material_handles.get(child) {
                if let Some(custom_material) = materials.get_mut(&material.0) {
                    custom_material.settings.fill = 0.2 + 0.5 * asteroid.health / 100.0;
                }
            }
        }
    }
}
