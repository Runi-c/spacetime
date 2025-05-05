use bevy::prelude::*;

use crate::{dither::DitherMaterial, physics::Velocity};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_particles, tick_particles))
        .add_event::<SpawnParticles>();
}

#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
}
#[derive(Event)]
pub struct SpawnParticles {
    pub position: Vec2,
    pub count: usize,
}

pub fn spawn_particles(
    mut commands: Commands,
    mut reader: EventReader<SpawnParticles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    for event in reader.read() {
        for _ in 0..event.count {
            commands.spawn((
                Particle { lifetime: 1.0 },
                Transform::from_translation(event.position.extend(0.0)),
                Mesh2d(meshes.add(Circle::new(10.0))),
                Velocity(
                    Vec2::new(
                        rand::random::<f32>() * 2.0 - 1.0,
                        rand::random::<f32>() * 2.0 - 1.0,
                    )
                    .normalize()
                        * rand::random::<f32>()
                        * 100.0,
                ),
                MeshMaterial2d(materials.add(DitherMaterial {
                    fill: 0.5,
                    dither_scale: 0.01,
                    ..default()
                })),
            ));
        }
    }
}

pub fn tick_particles(
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
