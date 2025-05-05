use std::time::Duration;

use bevy::prelude::*;
use lyon_tessellation::StrokeOptions;

use crate::{
    collision::{Collider, CollisionEvent},
    dither::DitherMaterial,
    layers::Layers,
    mesh::MeshLyonExtensions,
    physics::{Spin, Velocity},
    z_order::ZOrder,
    SCREEN_SIZE,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            collide_asteroid,
            asteroid_timer,
            break_asteroid,
            display_asteroid_health,
        ),
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

fn spawn_asteroid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    let mut vertices = vec![];
    let corner_count = rand::random_range(9..=15);
    for i in 0..corner_count {
        let radius = rand::random_range(25.0..=50.0);
        let angle = (i as f32 / corner_count as f32) * std::f32::consts::PI * 2.0;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        vertices.push(vec2(x, y));
    }
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

    commands
        .spawn((
            Asteroid { health: 100.0 },
            Layers::SPACE,
            Mesh2d(meshes.add(Mesh::stroke_polygon(
                &vertices,
                &StrokeOptions::default().with_line_width(5.0),
            ))),
            MeshMaterial2d(materials.add(DitherMaterial {
                fill: 1.0,
                ..default()
            })),
            Collider::from_vertices(&vertices),
            Transform::from_xyz(pos.x, pos.y, ZOrder::ASTEROID),
            Velocity(Vec2::new(
                rand::random_range(-100.0..=100.0),
                rand::random_range(-100.0..=100.0),
            )),
            Spin(rand::random_range(0.0..=20.0)),
        ))
        .with_child((
            Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
            MeshMaterial2d(materials.add(DitherMaterial {
                fill: 0.9,
                dither_type: 0,
                dither_scale: 0.1,
            })),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
}

fn collide_asteroid(
    mut commands: Commands,
    mut asteroids: Query<(&Transform, &mut Velocity, &mut Spin), With<Asteroid>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for event in collision_events.read() {
        if asteroids.contains(event.entity_a) {
            if asteroids.contains(event.entity_b) {}
        }
    }
}

fn break_asteroid(mut commands: Commands, asteroids: Query<(Entity, &Asteroid)>) {
    for (entity, asteroid) in asteroids.iter() {
        if asteroid.health <= 0.0 {
            commands.entity(entity).despawn();
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
                    custom_material.fill = 0.9 * asteroid.health / 100.0;
                }
            }
        }
    }
}
