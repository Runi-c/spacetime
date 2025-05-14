use bevy::prelude::*;
use parry2d::{
    na::{Isometry2, Vector2},
    query,
    shape::{Ball, Compound, SharedShape, TriMesh},
};

use crate::scheduling::Sets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, do_collision.in_set(Sets::Collision))
        .add_event::<CollisionEvent>();
}

#[derive(Component, Clone, Debug)]
pub struct Collider {
    shape: Compound,
    radius: f32,
}

impl Collider {
    pub fn from_shape(shape: Compound) -> Self {
        let radius = shape.local_bounding_sphere().radius;
        Self { shape, radius }
    }

    pub fn from_vertices(vertices: &[Vec2]) -> Self {
        let shape = Compound::decompose_trimesh(
            &TriMesh::from_polygon(
                vertices
                    .iter()
                    .map(|v| parry2d::na::Point2::new(v.x, v.y))
                    .collect(),
            )
            .unwrap(),
        )
        .unwrap();
        Self::from_shape(shape)
    }

    pub fn from_circle(radius: f32) -> Self {
        let shape = Compound::new(vec![(
            Isometry2::identity(),
            SharedShape::new(Ball::new(radius)),
        )]);
        Self { shape, radius }
    }
}

#[derive(Clone, Debug)]
pub struct Contact {
    pub point_a: Vec2,
    pub point_b: Vec2,
    pub normal: Vec2,
    pub dist: f32,
}

impl Contact {
    pub fn query(
        transform_a: &Transform,
        collider_a: &Collider,
        transform_b: &Transform,
        collider_b: &Collider,
        max_distance: f32,
    ) -> Option<Self> {
        let distance = transform_a
            .translation
            .truncate()
            .distance(transform_b.translation.truncate());
        if distance > max_distance {
            return None;
        }
        let isometry_a = Isometry2::new(
            Vector2::new(transform_a.translation.x, transform_a.translation.y),
            transform_a.rotation.to_euler(EulerRot::YXZ).2,
        );
        let isometry_b = Isometry2::new(
            Vector2::new(transform_b.translation.x, transform_b.translation.y),
            transform_b.rotation.to_euler(EulerRot::YXZ).2,
        );
        query::contact(
            &isometry_a,
            &collider_a.shape,
            &isometry_b,
            &collider_b.shape,
            distance,
        )
        .unwrap()
        .map(Self::from)
    }

    pub fn flip(&self) -> Self {
        Self {
            point_a: self.point_b,
            point_b: self.point_a,
            normal: -self.normal,
            dist: -self.dist,
        }
    }
}

impl From<query::Contact> for Contact {
    fn from(contact: query::Contact) -> Self {
        Self {
            point_a: vec2(contact.point1.x, contact.point1.y),
            point_b: vec2(contact.point2.x, contact.point2.y),
            normal: vec2(contact.normal1.x, contact.normal1.y),
            dist: contact.dist,
        }
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
    for [(entity_a, transform_a, collider_a), (entity_b, transform_b, collider_b)] in
        colliders.iter_combinations()
    {
        let max_distance = collider_a.radius + collider_b.radius;
        let contact = Contact::query(
            transform_a,
            collider_a,
            transform_b,
            collider_b,
            max_distance,
        );
        if let Some(contact) = contact.filter(|c| c.dist < 0.0) {
            writer.write(CollisionEvent {
                entity_a: entity_b,
                entity_b: entity_a,
                contact: contact.flip(),
            });
            writer.write(CollisionEvent {
                entity_a,
                entity_b,
                contact,
            });
        }
    }
}
