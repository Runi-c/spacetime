use std::time::Duration;

use bevy::prelude::*;
use lyon_tessellation::StrokeOptions;
use rand::distr::uniform::SampleRange;

use crate::{
    layers::SpaceLayer,
    materials::{Dither, DitherMaterial, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::ResourceType,
    scheduling::Sets,
    z_order::ZOrder,
    SCREEN_SIZE,
};

use super::{
    collision::Collider,
    physics::{Spin, Velocity},
    pickup::Pickup,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (asteroid_timer, break_asteroid, display_asteroid_health).in_set(Sets::PostUpdate),
    );
}

#[derive(Component)]
pub struct Asteroid {
    pub health: f32,
}

fn asteroid_timer(mut commands: Commands, time: Res<Time>, mut timer: Local<Option<Timer>>) {
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_secs_f32(1.5), TimerMode::Repeating));
    if timer.tick(time.delta()).just_finished() {
        commands.run_system_cached(spawn_asteroid);
    }
}

pub fn make_asteroid_polygon(
    vertex_range: impl SampleRange<i32>,
    radius_range: impl SampleRange<f32> + Clone,
) -> Vec<Vec2> {
    let mut vertices = vec![];
    let corner_count = rand::random_range(vertex_range);
    for i in 0..corner_count {
        let radius = rand::random_range(radius_range.clone());
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
) {
    let pos = match rand::random_range(0..=3) {
        0 => Vec2::new(
            -SCREEN_SIZE.x / 2.0,
            rand::random_range(-SCREEN_SIZE.y / 2.0..=SCREEN_SIZE.y / 2.0),
        ),
        1 => Vec2::new(
            SCREEN_SIZE.x / 2.0,
            rand::random_range(-SCREEN_SIZE.y / 2.0..=SCREEN_SIZE.y / 2.0),
        ),
        2 => Vec2::new(
            rand::random_range(-SCREEN_SIZE.x / 2.0..=SCREEN_SIZE.x / 2.0),
            -SCREEN_SIZE.y / 2.0,
        ),
        _ => Vec2::new(
            rand::random_range(-SCREEN_SIZE.x / 2.0..=SCREEN_SIZE.x / 2.0),
            SCREEN_SIZE.y / 2.0,
        ),
    };
    let vertices = make_asteroid_polygon(9..=15, 25.0..=50.0);
    commands.spawn((
        Name::new("Asteroid"),
        Asteroid { health: 100.0 },
        SpaceLayer,
        Mesh2d(meshes.add(Mesh::stroke_polygon(
            &vertices,
            &StrokeOptions::default().with_line_width(5.0),
        ))),
        MeshMaterial2d(SOLID_WHITE),
        Collider::from_vertices(&vertices),
        Transform::from_xyz(pos.x, pos.y, ZOrder::ASTEROID),
        Velocity::random(60.0..150.0),
        Spin::random(),
        children![(
            SpaceLayer,
            Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
            MeshMaterial2d(materials.add(Dither {
                fill: 0.9,
                scale: 0.1,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 1.0),
        )],
    ));
}

/* fn collide_asteroid(
    mut commands: Commands,
    mut asteroids: Query<(&Transform, &mut Velocity, &mut Spin), With<Asteroid>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for event in collision_events.read() {
        if asteroids.contains(event.entity_a) {
            if asteroids.contains(event.entity_b) {}
        }
    }
} */

fn break_asteroid(
    mut commands: Commands,
    asteroids: Query<(Entity, &Transform, &Asteroid)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    for (entity, transform, asteroid) in asteroids.iter() {
        if asteroid.health <= 0.0 {
            commands.entity(entity).despawn();
            for _ in 0..4 {
                let vertices = make_asteroid_polygon(5..=7, 10.0..=20.0);
                commands.spawn((
                    Name::new("Mineral Pickup"),
                    Pickup {
                        resource: ResourceType::Mineral,
                        amount: 10.0,
                    },
                    SpaceLayer,
                    Collider::from_vertices(&vertices),
                    Mesh2d(meshes.add(Mesh::stroke_polygon(
                        &vertices,
                        &StrokeOptions::default().with_line_width(2.0),
                    ))),
                    MeshMaterial2d(SOLID_WHITE),
                    Transform::from_xyz(
                        transform.translation.x + rand::random_range(-10.0..=10.0),
                        transform.translation.y + rand::random_range(-10.0..=10.0),
                        ZOrder::PICKUP,
                    ),
                    Velocity::random(25.0..40.0),
                    Spin::random(),
                    children![(
                        Name::new("Asteroid Fragment"),
                        SpaceLayer,
                        Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
                        MeshMaterial2d(materials.add(Dither {
                            fill: 0.5,
                            scale: 0.1,
                            ..default()
                        })),
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    )],
                ));
            }
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
                    custom_material.settings.fill = 0.9 * asteroid.health / 100.0;
                }
            }
        }
    }
}
