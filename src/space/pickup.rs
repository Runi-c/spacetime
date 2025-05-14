use bevy::{math::FloatPow, prelude::*};
use rand::Rng;

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, GassyDither, RockyDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::{ResourceType, Resources},
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    asteroid::generate_asteroid_shape,
    physics::{Spin, Velocity},
    ship::Ship,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, pickup_update.in_set(Sets::Update));
}

#[derive(Component, Clone)]
pub struct Pickup {
    pub resource: ResourceType,
    pub amount: f32,
}

impl Pickup {
    pub fn new(resource: ResourceType, amount: f32) -> Self {
        Self { resource, amount }
    }
}

const MAGNET_RADIUS: f32 = 150.0;
const MAGNET_FORCE: f32 = 100.0;
const PICKUP_RADIUS: f32 = 20.0;
fn pickup_update(
    mut commands: Commands,
    ship_query: Query<&Transform, With<Ship>>,
    pickup_query: Query<(Entity, &Pickup, &Transform)>,
    mut resources: ResMut<Resources>,
) {
    for ship_transform in ship_query.iter() {
        for (pickup_entity, pickup, pickup_transform) in pickup_query.iter() {
            let distance = ship_transform
                .translation
                .truncate()
                .distance(pickup_transform.translation.truncate());
            if distance < PICKUP_RADIUS {
                resources.add(pickup.resource, pickup.amount);
                commands.entity(pickup_entity).despawn();
            } else if distance < MAGNET_RADIUS {
                let magnitude = 2.0 - (distance / MAGNET_RADIUS);
                let direction = (ship_transform.translation - pickup_transform.translation)
                    .normalize()
                    .truncate();
                commands
                    .entity(pickup_entity)
                    .insert(Velocity(direction * magnitude.squared() * MAGNET_FORCE));
            }
        }
    }
}

pub fn mineral_pickup(
    pos: Vec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
) -> impl Bundle {
    let vertices = generate_asteroid_shape(5..=7, 10.0..=20.0);
    (
        Name::new("Mineral Pickup"),
        Pickup::new(ResourceType::Mineral, 2.5),
        SpaceLayer,
        Mesh2d(meshes.add(Mesh::stroke_polygon(&vertices, 2.0))),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(
            pos.x + rand::thread_rng().gen_range(-10.0..=10.0),
            pos.y + rand::thread_rng().gen_range(-10.0..=10.0),
            0.0,
        ),
        ZOrder::PICKUP,
        Velocity::random(25.0..40.0),
        Spin::random(90.0),
        children![(
            Name::new("Mineral Pickup Inner"),
            SpaceLayer,
            Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
            MeshMaterial2d(materials.add(RockyDither {
                fill: 0.5,
                scale: 20.0,
            })),
            Transform::from_xyz(0.0, 0.0, 1.0),
        )],
    )
}

pub fn time_pickup(
    pos: Vec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
) -> impl Bundle {
    let vertices = vec![
        vec2(5.0, 0.0),
        vec2(0.0, 10.0),
        vec2(-5.0, 0.0),
        vec2(0.0, -10.0),
    ];
    (
        Name::new("Time Pickup"),
        Pickup::new(ResourceType::Time, 5.0),
        SpaceLayer,
        Mesh2d(meshes.add(Mesh::stroke_polygon(&vertices, 2.0))),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(
            pos.x + rand::thread_rng().gen_range(-10.0..=10.0),
            pos.y + rand::thread_rng().gen_range(-10.0..=10.0),
            0.0,
        ),
        ZOrder::PICKUP,
        Velocity::random(25.0..40.0),
        Spin::random(45.0),
        children![(
            Name::new("Time Pickup Inner"),
            SpaceLayer,
            Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
            MeshMaterial2d(materials.add(GassyDither {
                fill: 0.5,
                scale: 20.0,
            })),
            Transform::from_xyz(0.0, 0.0, 1.0),
        )],
    )
}
