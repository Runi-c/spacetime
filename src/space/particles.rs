use bevy::prelude::*;

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, GassyDither},
    scheduling::Sets,
    z_order::ZOrder,
};

use super::physics::{Rotation, Velocity};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<EmitParticles>().add_systems(
        Update,
        (
            particle_emit.in_set(Sets::Update),
            particle_tick.in_set(Sets::PostUpdate),
        ),
    );
}

#[derive(Component, Clone)]
pub struct Particle {
    pub lifetime: f32,
}
#[derive(Event)]
pub struct EmitParticles {
    pub position: Vec2,
    pub count: usize,
}

pub fn particle_emit(
    mut commands: Commands,
    mut reader: EventReader<EmitParticles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    let mesh = meshes.add(Circle::new(10.0));
    for event in reader.read() {
        for _ in 0..event.count {
            commands.spawn((
                Name::new("Particle"),
                Particle { lifetime: 1.0 },
                SpaceLayer,
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.add(GassyDither {
                    fill: rand::random::<f32>() * 0.3 + 0.4,
                    scale: 20.0,
                })),
                Transform::from_translation(event.position.extend(0.0)),
                ZOrder::BULLET,
                Velocity::random(0.0..100.0),
                Rotation::random(),
            ));
        }
    }
}

pub fn particle_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    for (entity, mut particle, mut transform) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();
        transform.scale = Vec3::splat(particle.lifetime / 1.0);
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
