use bevy::prelude::*;
use parry2d::{
    na::{Isometry2, Vector2},
    query::{self, Contact},
    shape::{Compound, TriMesh},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, do_collision)
        .add_event::<CollisionEvent>();
}

#[derive(Component)]
pub struct Collider(pub Compound);

impl Collider {
    pub fn from_vertices(vertices: Vec<[f32; 3]>) -> Self {
        Self(
            Compound::decompose_trimesh(
                &TriMesh::from_polygon(
                    vertices
                        .iter()
                        .map(|v| parry2d::na::Point2::new(v[0], v[1]))
                        .collect(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
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
                    info!(
                        "Collision detected between {:?} and {:?}",
                        entity_a, entity_b
                    );
                    info!("Contact: {:?}", contact);
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
