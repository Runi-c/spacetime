use bevy::prelude::*;
use parry2d::{
    na::{Isometry2, Vector2},
    query::{self, Contact},
    shape::{Compound, Cuboid, SharedShape, TriMesh},
};

use crate::scheduling::Sets;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, do_collision.in_set(Sets::Collision))
        .add_event::<CollisionEvent>();
}

#[derive(Component, Clone)]
pub struct Collider(pub Compound);

impl Collider {
    pub fn from_vertices(vertices: &[Vec2]) -> Self {
        Self(
            Compound::decompose_trimesh(
                &TriMesh::from_polygon(
                    vertices
                        .iter()
                        .map(|v| parry2d::na::Point2::new(v.x, v.y))
                        .collect(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    }

    pub fn from_aabb(min: Vec2, max: Vec2) -> Self {
        let shape = Cuboid::new(Vector2::new((max.x - min.x) / 2.0, (max.y - min.y) / 2.0));
        Self(Compound::new(vec![(
            Isometry2::identity(),
            SharedShape::new(shape),
        )]))
    }

    pub fn from_circle(radius: f32) -> Self {
        let shape = parry2d::shape::Ball::new(radius);
        Self(Compound::new(vec![(
            Isometry2::identity(),
            SharedShape::new(shape),
        )]))
    }
}

#[derive(Event)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contact: Contact,
}

fn do_collision(
    colliders: Query<(Entity, &Transform, &Collider)>,
    mut writer: EventWriter<CollisionEvent>,
) {
    for (entity_a, transform_a, collider_a) in colliders.iter() {
        for (entity_b, transform_b, collider_b) in colliders.iter() {
            if entity_a != entity_b {
                let distance = transform_a
                    .translation
                    .truncate()
                    .distance(transform_b.translation.truncate());
                let contact = query::contact(
                    &transform_to_isometry(transform_a),
                    &collider_a.0,
                    &transform_to_isometry(transform_b),
                    &collider_b.0,
                    distance,
                )
                .unwrap();
                if let Some(contact) = contact.filter(|c| c.dist <= 0.0) {
                    writer.write(CollisionEvent {
                        entity_a,
                        entity_b,
                        contact,
                    });
                }
            }
        }
    }
}

pub fn transform_to_isometry(transform: &Transform) -> Isometry2<f32> {
    Isometry2::new(
        Vector2::new(transform.translation.x, transform.translation.y),
        transform.rotation.to_euler(EulerRot::YXZ).2,
    )
}
