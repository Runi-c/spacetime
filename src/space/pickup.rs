use bevy::{math::FloatPow, prelude::*};
use lyon_tessellation::StrokeOptions;
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
    asteroid::make_asteroid_polygon,
    collision::{Collider, CollisionEvent},
    physics::{Spin, Velocity},
    ship::Ship,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            pickup_magnet.in_set(Sets::Physics),
            collect_pickup.in_set(Sets::Update),
        )
            .chain(),
    );
}

#[derive(Component, Clone)]
pub struct Pickup {
    pub resource: ResourceType,
    pub amount: f32,
}

fn collect_pickup(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    mut resources: ResMut<Resources>,
    ship_query: Query<&Ship>,
    pickup_query: Query<&Pickup>,
) {
    for event in collisions.read() {
        if ship_query.contains(event.entity_a) {
            if let Ok(pickup) = pickup_query.get(event.entity_b) {
                match pickup.resource {
                    ResourceType::Mineral => resources.minerals += pickup.amount,
                    ResourceType::Gas => resources.gas += pickup.amount,
                    ResourceType::Time => resources.time += pickup.amount,
                    _ => panic!("Unknown resource type"),
                }
                commands.entity(event.entity_b).despawn();
            }
        }
    }
}

const MAGNETIC_PULL: f32 = 150.0;
fn pickup_magnet(
    mut commands: Commands,
    ship_query: Query<&Transform, With<Ship>>,
    pickup_query: Query<(Entity, &Transform), With<Pickup>>,
) {
    for ship_transform in ship_query.iter() {
        for (pickup, pickup_transform) in pickup_query.iter() {
            let distance = ship_transform
                .translation
                .truncate()
                .distance(pickup_transform.translation.truncate());
            if distance < MAGNETIC_PULL {
                let magnitude = 2.0 - (distance / MAGNETIC_PULL);
                let direction = (ship_transform.translation - pickup_transform.translation)
                    .normalize()
                    .truncate();
                commands
                    .entity(pickup)
                    .insert(Velocity(direction * magnitude.squared() * 100.0));
            }
        }
    }
}

pub fn asteroid_piece(
    pos: Vec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<DitherMaterial>>,
) -> impl Bundle {
    let vertices = make_asteroid_polygon(5..=7, 10.0..=20.0);
    (
        Name::new("Mineral Pickup"),
        Pickup {
            resource: ResourceType::Mineral,
            amount: 2.5,
        },
        SpaceLayer,
        Mesh2d(meshes.add(Mesh::stroke_polygon(
            &vertices,
            &StrokeOptions::default().with_line_width(2.0),
        ))),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(
            pos.x + rand::thread_rng().gen_range(-10.0..=10.0),
            pos.y + rand::thread_rng().gen_range(-10.0..=10.0),
            0.0,
        ),
        ZOrder::PICKUP,
        Collider::from_vertices(&vertices),
        Velocity::random(25.0..40.0),
        Spin::random(),
        children![(
            Name::new("Asteroid Fragment"),
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

pub fn time_piece(
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
        Name::new("Time Piece"),
        Pickup {
            resource: ResourceType::Time,
            amount: 5.0,
        },
        SpaceLayer,
        Mesh2d(meshes.add(Mesh::stroke_polygon(
            &vertices,
            &StrokeOptions::default().with_line_width(2.0),
        ))),
        MeshMaterial2d(SOLID_WHITE),
        Transform::from_xyz(
            pos.x + rand::thread_rng().gen_range(-10.0..=10.0),
            pos.y + rand::thread_rng().gen_range(-10.0..=10.0),
            0.0,
        ),
        ZOrder::PICKUP,
        Collider::from_vertices(&vertices),
        Velocity::random(25.0..40.0),
        Spin::random(),
        children![(
            Name::new("Time Piece"),
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
