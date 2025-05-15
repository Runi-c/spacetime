use std::time::Duration;

use bevy::prelude::*;

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, MetalDither},
    mesh::MeshLyonExtensions,
    resources::{ResourceType, Resources},
    scheduling::Sets,
    sounds::Sounds,
    z_order::ZOrder,
};

use super::{
    bounds::ScreenBounds,
    collision::{Collider, CollisionEvent},
    particles::EmitParticles,
    physics::{DespawnOutOfBounds, Rotation, Velocity},
    pickup::time_pickup,
    ship::Ship,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            enemy_move.in_set(Sets::Input),
            (enemy_shoot, enemy_spawn_timer, enemy_bullet_collide).in_set(Sets::Update),
            enemy_die.in_set(Sets::PostUpdate),
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

#[derive(Component, Clone)]
pub struct EnemyLivingSound;

fn enemy_spawn_timer(
    mut commands: Commands,
    enemies: Query<&Enemy>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let enemy_count = enemies.iter().count();
    if enemy_count >= 1 {
        return;
    }
    let timer = timer
        .get_or_insert_with(|| Timer::new(Duration::from_secs_f32(15.0), TimerMode::Repeating));
    if timer.tick(time.delta()).just_finished() {
        commands.run_system_cached(enemy_spawn);
    }
}

fn enemy_spawn(
    mut commands: Commands,
    enemy_living_sound: Option<Single<&AudioSink, With<EnemyLivingSound>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    sounds: Res<Sounds>,
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
    let enemy = commands
        .spawn((
            Name::new("Enemy"),
            SpaceLayer,
            Enemy { health: 100.0 },
            Mesh2d(meshes.add(Mesh::fill_polygon(&vertices))),
            MeshMaterial2d(materials.add(MetalDither {
                fill: 1.0,
                scale: 20.0,
            })),
            Collider::from_vertices(&vertices),
            Transform::from_translation(bounds.random_outside().extend(0.0) * 1.2),
            ZOrder::ENEMY,
            Velocity(Vec2::ZERO),
            Target(bounds.random_outside() * 0.8),
        ))
        .id();

    if let Some(sound) = enemy_living_sound {
        sound.play();
    } else {
        commands.spawn((
            Name::new("Enemy Living Sound"),
            EnemyLivingSound,
            AudioPlayer::new(sounds.enemy_gamer.clone()),
            PlaybackSettings::LOOP,
            ChildOf(enemy),
        ));
    }
}

fn enemy_move(
    mut enemies: Query<(&mut Transform, &mut Target, &mut Velocity), With<Enemy>>,
    ship: Single<(&Transform, &Velocity), (With<Ship>, Without<Enemy>)>,
    bounds: ScreenBounds,
    time: Res<Time>,
) {
    for (mut transform, mut target, mut velocity) in enemies.iter_mut() {
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

#[derive(Component, Clone)]
pub struct EnemyBullet(pub f32);

fn enemy_shoot(
    mut commands: Commands,
    enemies: Query<(Entity, &Transform, Option<&AttackCooldown>), With<Enemy>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    sounds: Res<Sounds>,
) {
    for (entity, transform, cooldown) in enemies.iter() {
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
            commands.spawn((
                Name::new("Enemy Shoot Sound"),
                AudioPlayer::new(sounds.enemy_gun.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

fn enemy_bullet_collide(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    bullets: Query<&EnemyBullet>,
    ship: Single<Entity, With<Ship>>,
    mut resources: ResMut<Resources>,
    mut particle_writer: EventWriter<EmitParticles>,
) {
    for event in collision_events.read() {
        if let Ok(bullet) = bullets.get(event.entity_a) {
            if event.entity_b == *ship {
                // Handle collision between bullet and ship
                commands.entity(event.entity_a).despawn();
                resources.add(ResourceType::Health, -bullet.0);
                particle_writer.write(EmitParticles {
                    position: event.contact.point_b,
                    count: 3,
                });
            }
        }
    }
}

fn enemy_die(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy, &Transform)>,
    enemy_living_sound: Query<&AudioSink, With<EnemyLivingSound>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    sounds: Res<Sounds>,
    mut particle_writer: EventWriter<EmitParticles>,
) {
    for (entity, enemy, transform) in enemies.iter() {
        if enemy.health <= 0.0 {
            commands.entity(entity).despawn();
            particle_writer.write(EmitParticles {
                position: transform.translation.truncate(),
                count: 10,
            });
            for _ in 0..4 {
                commands.spawn(time_pickup(
                    transform.translation.truncate(),
                    &mut meshes,
                    &mut materials,
                ));
            }
            for sink in enemy_living_sound.iter() {
                sink.stop();
            }
            commands.spawn((
                Name::new("Enemy Die Sound"),
                AudioPlayer::new(sounds.enemy_die.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}
