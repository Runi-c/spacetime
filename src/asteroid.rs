use std::time::Duration;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    collision::{Collider, CollisionEvent},
    physics::{Rotation, Velocity},
    SCREEN_SIZE,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (collide_asteroid, asteroid_timer, break_asteroid));
}

#[derive(Component)]
pub struct Asteroid {
    pub health: f32,
}

fn asteroid_timer(mut commands: Commands, time: Res<Time>, mut timer: Local<Option<Timer>>) {
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_secs_f32(3.0), TimerMode::Repeating));
    if timer.tick(time.delta()).just_finished() {
        commands.run_system_cached(spawn_asteroid);
    }
}

fn spawn_asteroid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut asteroid = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let mut vertices = vec![[0.0, 0.0, 0.0]];
    let corner_count = rand::random_range(9..=15);
    for i in 0..corner_count {
        let radius = rand::random_range(50.0..=100.0);
        let angle = (i as f32 / corner_count as f32) * std::f32::consts::PI * 2.0;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        vertices.push([x, y, 0.0]);
    }
    let mut indices = vec![0, 1, corner_count];
    for i in 1..corner_count {
        indices.extend_from_slice(&[0, i, i + 1]);
    }
    asteroid.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
    asteroid.insert_indices(Indices::U32(indices));

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

    commands.spawn((
        Asteroid { health: 100.0 },
        Mesh2d(meshes.add(asteroid)),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Collider::from_vertices(vertices),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        Velocity(Vec2::new(
            rand::random_range(-100.0..=100.0),
            rand::random_range(-100.0..=100.0),
        )),
        Rotation(rand::random_range(0.0..=2.0)),
    ));
}

fn collide_asteroid(
    mut commands: Commands,
    asteroids: Query<(Entity, &Transform, &Collider), With<Asteroid>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for event in collision_events.read() {
        if let Ok((entity, _, _)) = asteroids.get(event.entity_a) {
            commands.entity(entity).despawn();
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
