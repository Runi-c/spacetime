use std::time::Duration;

use bevy::prelude::*;

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, MetalDither},
    mesh::MeshLyonExtensions,
    scheduling::Sets,
    utils::parry2d_vec2,
    z_order::ZOrder,
};

use super::{
    bounds::ScreenBounds,
    collision::{Collider, CollisionEvent},
    particles::SpawnParticles,
    physics::{DespawnOutOfBounds, Rotation, Velocity},
    ship::Ship,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            enemy_move.in_set(Sets::Input),
            (enemy_shoot, enemy_timer, collide_bullet).in_set(Sets::Update),
        ),
    );
}

#[derive(Component, Clone)]
pub struct Enemy {
    pub health: f32,
}

#[derive(Component, Clone)]
pub struct Target(pub Vec2);

#[derive(Component, Clone)]
pub struct AttackCooldown(pub f32);

fn enemy_timer(
    mut commands: Commands,
    mut timer: Local<Option<Timer>>,
    enemies: Query<&Enemy>,
    time: Res<Time>,
) {
    let enemy_count = enemies.iter().count();
    if enemy_count >= 1 {
        return;
    }
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_secs_f32(1.5), TimerMode::Repeating));
    if timer.tick(time.delta()).just_finished() {
        commands.run_system_cached(spawn_enemy);
    }
}

fn spawn_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    bounds: ScreenBounds,
) {
    const OUTER: f32 = 30.0;
    const INNER: f32 = 20.0;
    const MID: f32 = (INNER + OUTER) / 2.0;
    let vertices = vec![
        vec2(INNER, -INNER),        // top right
        vec2(-INNER, -INNER * 0.8), // top left
        vec2(-INNER, INNER * 0.8),  // bottom left
        vec2(INNER, INNER),         // bottom right
        vec2(OUTER, MID),
        vec2(0.0, OUTER),
        vec2(-OUTER, MID),
        vec2(-MID, 0.0),
        vec2(-OUTER, -MID),
        vec2(0.0, -OUTER),
        vec2(OUTER, -MID),
    ];
    commands.spawn((
        Name::new("Enemy"),
        SpaceLayer,
        Enemy { health: 100.0 },
        Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
        MeshMaterial2d(materials.add(MetalDither {
            fill: 1.0,
            scale: 20.0,
        })),
        Transform::from_translation(bounds.random_outside().extend(0.0) * 1.2),
        ZOrder::ENEMY,
        Velocity(Vec2::ZERO),
        Target(bounds.random_outside() * 0.8),
    ));
}

fn enemy_move(
    time: Res<Time>,
    ship: Single<(&Transform, &Velocity), (With<Ship>, Without<Enemy>)>,
    mut query: Query<(&mut Transform, &mut Target, &mut Velocity), With<Enemy>>,
    bounds: ScreenBounds,
) {
    for (mut transform, mut target, mut velocity) in query.iter_mut() {
        let diff = target.0 - transform.translation.truncate();
        velocity.0 = velocity.0.lerp(diff.normalize() * 300.0, time.delta_secs());
        if diff.length() < 10.0 {
            target.0 = bounds.random_outside() * 0.8;
        }
        let (ship_transform, ship_velocity) = *ship;
        let ship_prediction = ship_transform.translation.truncate() + ship_velocity.0 * 0.5;
        let target_rotation = (ship_prediction - transform.translation.truncate()).to_angle();
        transform.rotation = transform.rotation.slerp(
            Quat::from_rotation_z(target_rotation),
            time.delta_secs() * 4.0,
        );
    }
}

fn enemy_shoot(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &Transform, Option<&AttackCooldown>), With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    for (entity, transform, cooldown) in query.iter() {
        if let Some(cooldown) = cooldown {
            if cooldown.0 > 0.0 {
                commands
                    .entity(entity)
                    .insert(AttackCooldown(cooldown.0 - time.delta_secs()));
                continue;
            } else {
                commands.entity(entity).remove::<AttackCooldown>();
            }
        } else {
            commands.entity(entity).insert(AttackCooldown(5.0));
            let rotation = transform.rotation.to_euler(EulerRot::YXZ).2;
            commands.spawn((
                Name::new("Enemy Bullet"),
                SpaceLayer,
                EnemyBullet(10.0),
                Mesh2d(meshes.add(Ellipse::from_size(vec2(15.0, 12.0)))),
                MeshMaterial2d(materials.add(MetalDither {
                    fill: 0.5,
                    scale: 10.0,
                })),
                Collider::from_circle(15.0),
                DespawnOutOfBounds,
                Transform::from_translation(transform.translation),
                ZOrder::BULLET,
                Velocity(Vec2::from_angle(rotation) * 700.0),
                Rotation(rotation.to_degrees()),
            ));
        }
    }
}

#[derive(Component, Clone)]
pub struct EnemyBullet(pub f32);

fn collide_bullet(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut particle_writer: EventWriter<SpawnParticles>,
    ships: Query<&Ship>,
    bullets: Query<&EnemyBullet>,
) {
    for event in collision_events.read() {
        if let Ok(bullet) = bullets.get(event.entity_a) {
            if let Ok(ship) = ships.get(event.entity_b) {
                // Handle collision between bullet and ship
                commands.entity(event.entity_a).despawn();
                commands.entity(event.entity_b).insert(Ship {
                    health: ship.health - bullet.0,
                });
                particle_writer.write(SpawnParticles {
                    position: parry2d_vec2(event.contact.point2),
                    count: 3,
                });
            }
        }
    }
}
