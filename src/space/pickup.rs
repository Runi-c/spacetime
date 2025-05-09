use bevy::{math::FloatPow, prelude::*};

use crate::{
    resources::{ResourceType, Resources},
    scheduling::Sets,
};

use super::{collision::CollisionEvent, physics::Velocity, ship::Ship};

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

#[derive(Component)]
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
                info!(
                    "Collected {}: {}",
                    pickup.resource.to_string(),
                    pickup.amount
                );
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
