use std::time::Duration;

use bevy::prelude::*;
use rand::{distributions::uniform::SampleRange, Rng};

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, RockyDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::{ResourceType, Resources},
    scheduling::Sets,
    sounds::Sounds,
    z_order::ZOrder,
};

use super::{
    bounds::ScreenBounds,
    collision::{Collider, CollisionEvent},
    particles::EmitParticles,
    physics::{Spin, Velocity},
    pickup::{mineral_pickup, time_pickup},
    ship::Ship,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            asteroid_spawn_timer.in_set(Sets::PreUpdate),
            collide_asteroid.in_set(Sets::Update),
            (break_asteroid, display_asteroid_health).in_set(Sets::PostUpdate),
        ),
    );
}

pub fn generate_asteroid_shape(
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

#[derive(Component, Clone)]
pub struct Asteroid {
    pub health: f32,
}

fn asteroid_spawn_timer(
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

fn spawn_asteroid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    bounds: ScreenBounds,
) {
    let vertices = generate_asteroid_shape(9..=15, 25.0..=50.0);
    let mesh = Mesh::stroke_polygon(&vertices, 5.0);
    let pos = bounds.random_outside() * 1.2;
    let target = bounds.random_inside() * 0.8;
    commands.spawn((
        Name::new("Asteroid"),
        Asteroid { health: 100.0 },
        SpaceLayer,
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        ZOrder::ASTEROID,
        Collider::from_vertices(&vertices),
        Velocity::random_towards(target - pos, 60.0..150.0),
        Spin::random(90.0),
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
    mut particles: EventWriter<EmitParticles>,
    asteroids: Query<&Velocity, With<Asteroid>>,
    ship: Single<(Entity, &Transform, &Velocity), With<Ship>>,
    sounds: Res<Sounds>,
    mut resources: ResMut<Resources>,
) {
    let (ship, ship_transform, ship_velocity) = *ship;
    for event in collision_events.read() {
        if let Ok(asteroid_velocity) = asteroids.get(event.entity_a) {
            if event.entity_b == ship {
                // Handle collision between asteroid and ship
                let contact = &event.contact;
                resources.add(ResourceType::Health, -10.0);
                commands.entity(event.entity_b).insert((
                    Transform::from_translation(
                        ship_transform.translation
                            + contact.normal.extend(0.0) * -(contact.dist - 1.0),
                    ),
                    Velocity(
                        contact.normal * (ship_velocity.0.length() * 0.5).max(100.0)
                            + asteroid_velocity.0 * 0.5,
                    ),
                ));
                commands.spawn((
                    Name::new("Asteroid Bump Sound"),
                    AudioPlayer::new(sounds.asteroid_bump.clone()),
                    PlaybackSettings::DESPAWN,
                ));
                particles.write(EmitParticles {
                    position: contact.point_a,
                    count: 10,
                });
                info!("Ship collided with asteroid at {:?}", contact.point_a);
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
                commands.spawn(mineral_pickup(
                    transform.translation.truncate(),
                    &mut meshes,
                    &mut materials,
                ));
            }
            if rand::thread_rng().gen_bool(0.5) {
                commands.spawn(time_pickup(
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
