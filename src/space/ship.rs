use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use rand::Rng;

use crate::{
    game_over::GameOver,
    layers::SpaceLayer,
    materials::{DitherMaterial, MetalDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::Resources,
    scheduling::Sets,
    sounds::Sounds,
    space::physics::DespawnOutOfBounds,
    z_order::ZOrder,
};

use super::{
    asteroid::Asteroid,
    collision::{Collider, CollisionEvent, Contact},
    enemy::Enemy,
    particles::EmitParticles,
    physics::{Rotation, Velocity},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ship.in_set(Sets::Spawn))
        .add_systems(
            Update,
            (
                ship_rocket_update.in_set(Sets::Physics),
                ship_input.in_set(Sets::Input),
                (
                    ship_laser,
                    ship_rocket_fire,
                    ship_rocket_cooldown,
                    collide_rocket,
                )
                    .in_set(Sets::Update),
                (display_ship_health, destroy_ship)
                    .chain()
                    .in_set(Sets::PostUpdate),
            ),
        );
}

#[derive(Component, Clone)]
pub struct Ship;

#[derive(Component, Clone)]
pub struct ShipHealthMarker;

#[derive(Component, Clone)]
pub struct LaserSound;

#[derive(Component, Clone)]
pub struct RocketSound;

pub(super) fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
) {
    const SIZE: f32 = 30.0;
    let vertices = vec![
        vec2(SIZE, 0.0),
        vec2(-SIZE, SIZE * 0.7),
        vec2(-SIZE * 0.5, 0.0),
        vec2(-SIZE, -SIZE * 0.7),
    ];

    let mesh = meshes.add(Mesh::fill_polygon(&vertices));
    commands.spawn((
        Name::new("Ship"),
        Ship,
        SpaceLayer,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(MetalDither {
            fill: 0.2,
            scale: SIZE,
        })),
        Collider::from_vertices(&vertices),
        ZOrder::SHIP,
        Velocity(Vec2::ZERO),
        Rotation(0.0),
        Visibility::Visible,
        children![(
            Name::new("Ship Health"),
            ShipHealthMarker,
            SpaceLayer,
            Mesh2d(mesh),
            MeshMaterial2d(SOLID_WHITE),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    ));
}

fn destroy_ship(
    mut commands: Commands,
    ship: Single<(Entity, &Transform), With<Ship>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    resources: Res<Resources>,
    sounds: Res<Sounds>,
    mut particles: EventWriter<EmitParticles>,
) {
    if resources.health <= 0.0 || keyboard_input.just_pressed(KeyCode::KeyK) {
        let (entity, transform) = *ship;
        commands.entity(entity).despawn();
        particles.write(EmitParticles {
            position: transform.translation.truncate(),
            count: 10,
        });
        commands.spawn((
            Name::new("Death Sound"),
            AudioPlayer::new(sounds.player_death.clone()),
            PlaybackSettings::DESPAWN,
        ));
        commands.trigger(GameOver);
    }
}

#[derive(Component)]
pub struct ShipBullet;

#[derive(Component)]
pub struct ShootyCooldown(pub f32);

fn ship_input(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &mut Rotation,
            &mut Velocity,
            Option<&ShootyCooldown>,
        ),
        With<Ship>,
    >,
    mut resources: ResMut<Resources>,
    mut meshes: ResMut<Assets<Mesh>>,
    rocket_sound: Query<&AudioSink, With<RocketSound>>,
    sounds: Res<Sounds>,
) {
    const SPEED: f32 = 300.0;
    let mut input = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.x -= 1.0;
    }

    for (ship, transform, mut rotation, mut velocity, cooldown) in query.iter_mut() {
        if input == Vec3::ZERO {
            for sink in rocket_sound.iter() {
                sink.pause();
            }
        } else {
            if rocket_sound.is_empty() {
                commands.spawn((
                    Name::new("Rocket Sound"),
                    RocketSound,
                    AudioPlayer::new(sounds.rocket.clone()),
                    PlaybackSettings::LOOP,
                    ChildOf(ship),
                ));
            } else {
                for sink in rocket_sound.iter() {
                    sink.play();
                }
            }
        }

        let angle = transform.rotation.to_euler(EulerRot::YXZ).2;
        let acceleration = Quat::from_rotation_z(angle)
            * input.yzz().normalize_or_zero()
            * SPEED
            * time.delta_secs();
        velocity.0 += acceleration.xy();
        rotation.0 += input.x;
        if rotation.0 > 360.0 {
            rotation.0 -= 360.0;
        } else if rotation.0 < -360.0 {
            rotation.0 += 360.0;
        }

        if let Some(cooldown) = cooldown {
            if cooldown.0 > 0.0 {
                commands
                    .entity(ship)
                    .insert(ShootyCooldown(cooldown.0 - time.delta_secs()));
                return;
            } else {
                commands.entity(ship).remove::<ShootyCooldown>();
            }
        } else if keyboard_input.pressed(KeyCode::Space) && resources.ammo >= 1.0 {
            resources.ammo -= 1.0;
            commands.spawn((
                Name::new("Ship Bullet"),
                SpaceLayer,
                ShipBullet,
                Mesh2d(meshes.add(Circle::new(5.0))),
                MeshMaterial2d(SOLID_WHITE),
                Collider::from_circle(5.0),
                Transform::from_translation(transform.translation),
                ZOrder::BULLET,
                Velocity(Vec2::from_angle(angle + rand::random::<f32>() * 0.1 - 0.05) * 1500.0),
                Rotation(0.0),
                DespawnOutOfBounds,
            ));
            commands.entity(ship).insert(ShootyCooldown(0.15));
            commands.spawn((
                Name::new("Shooty Gun Noise"),
                AudioPlayer::new(sounds.gun.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::Linear(0.4),
                    ..default()
                },
            ));
        }
    }
}

fn ship_laser(
    mut commands: Commands,
    time: Res<Time>,
    mut gizmos: Gizmos,
    ship: Query<(Entity, &Transform, &Collider), With<Ship>>,
    mut asteroids: Query<(Entity, &mut Asteroid, &Transform, &Collider)>,
    mut particles: EventWriter<EmitParticles>,
    mut laser_sound: Query<(Entity, &mut AudioSink), With<LaserSound>>,
    sounds: Res<Sounds>,
) {
    for (ship_entity, ship_transform, ship_collider) in ship.iter() {
        let mut is_lasering = false;
        let closest = asteroids.iter().fold(
            None,
            |closest: Option<(f32, Entity)>, (entity, _, transform, _)| {
                let distance = ship_transform.translation.distance(transform.translation);
                if closest.is_none() || distance < closest.unwrap().0 {
                    Some((distance, entity))
                } else {
                    closest
                }
            },
        );
        if let Some((distance, asteroid_entity)) = closest {
            let (_, mut asteroid, asteroid_transform, asteroid_collider) =
                asteroids.get_mut(asteroid_entity).unwrap();
            if distance < 300.0 {
                let contact = Contact::query(
                    ship_transform,
                    ship_collider,
                    asteroid_transform,
                    asteroid_collider,
                    distance,
                );
                if let Some(contact) = contact {
                    if rand::thread_rng().gen_bool(0.1) {
                        particles.write(EmitParticles {
                            position: contact.point_b,
                            count: 1,
                        });
                    }
                    gizmos.line_2d(
                        ship_transform.translation.truncate(),
                        contact.point_b,
                        Color::WHITE,
                    );
                    asteroid.health -= time.delta_secs() * 25.0;
                    is_lasering = true;
                    if laser_sound.is_empty() {
                        commands.spawn((
                            Name::new("Laser Sound"),
                            LaserSound,
                            AudioPlayer::new(sounds.laser.clone()),
                            PlaybackSettings::LOOP,
                            ChildOf(ship_entity),
                        ));
                    }
                }
            }
        }
        if !is_lasering {
            for (entity, sink) in laser_sound.iter_mut() {
                sink.stop();
                commands.entity(entity).despawn();
            }
        }
    }
}

fn display_ship_health(
    mut commands: Commands,
    ship: Single<Entity, With<Ship>>,
    health_marker: Query<(Entity, &ChildOf, &Transform), With<ShipHealthMarker>>,
    resources: Res<Resources>,
) {
    if !resources.is_changed() {
        return;
    }
    for (marker_entity, child, transform) in health_marker.iter() {
        if child.parent() == *ship {
            let scale = resources.health / 100.0;
            commands.entity(marker_entity).insert(Transform {
                scale: Vec3::splat(scale),
                ..*transform
            });
        }
    }
}

#[derive(Component)]
pub struct ShipRocket;

#[derive(Component)]
pub struct ShipRocketCooldown(pub f32);

#[derive(Component, Clone)]
pub struct MissileFlightSound;

fn ship_rocket_fire(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ship: Single<(Entity, &Transform, Has<ShipRocketCooldown>), With<Ship>>,
    enemies: Query<Entity, With<Enemy>>,
    sounds: Res<Sounds>,
    mut resources: ResMut<Resources>,
) {
    let (ship, ship_transform, cooldown) = *ship;
    if cooldown || resources.rockets < 1.0 {
        return;
    }
    if !enemies.is_empty() {
        let mesh = meshes.add(Triangle2d::new(
            vec2(5.0, 0.0),
            vec2(-10.0, -5.0),
            vec2(-10.0, 5.0),
        ));
        commands.spawn((
            Name::new("Ship Rocket"),
            SpaceLayer,
            ShipRocket,
            Mesh2d(mesh),
            MeshMaterial2d(SOLID_WHITE),
            Collider::from_circle(5.0),
            Transform::from_translation(ship_transform.translation),
            ZOrder::BULLET,
            Velocity::random(500.0..1000.0),
            Rotation(0.0),
        ));
        commands.entity(ship).insert(ShipRocketCooldown(5.0));
        commands.spawn((
            Name::new("Missile Launch Noise"),
            AudioPlayer::new(sounds.missile_launch.clone()),
            PlaybackSettings::DESPAWN,
        ));
        resources.rockets -= 1.0;
    }
}

fn ship_rocket_cooldown(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShipRocketCooldown)>,
) {
    for (entity, mut cooldown) in query.iter_mut() {
        cooldown.0 -= time.delta_secs();
        if cooldown.0 <= 0.0 {
            commands.entity(entity).remove::<ShipRocketCooldown>();
        }
    }
}

fn ship_rocket_update(
    mut commands: Commands,
    time: Res<Time>,
    mut rockets: Query<(&Transform, &mut Velocity, &mut Rotation), With<ShipRocket>>,
    enemies: Query<&Transform, With<Enemy>>,
    mut particles: EventWriter<EmitParticles>,
    missile_flight_sound: Query<&AudioSink, With<MissileFlightSound>>,
    sounds: Res<Sounds>,
) {
    for (transform, mut velocity, mut rotation) in rockets.iter_mut() {
        if let Some(enemy_transform) = enemies.iter().next() {
            let pos = transform.translation.truncate();
            let target = enemy_transform.translation.truncate();
            let angle = (target - pos).to_angle();
            rotation.0 = angle.to_degrees();
            velocity.0 = velocity
                .0
                .lerp((target - pos).normalize() * 1000.0, time.delta_secs() * 2.0);
            particles.write(EmitParticles {
                position: pos,
                count: 1,
            });
        }
    }
    if rockets.is_empty() {
        for sink in missile_flight_sound.iter() {
            sink.pause();
        }
    } else {
        if missile_flight_sound.is_empty() {
            commands.spawn((
                Name::new("Missile Flying Sound"),
                MissileFlightSound,
                AudioPlayer::new(sounds.missile_flight.clone()),
                PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::Linear(0.3),
                    ..default()
                },
            ));
        } else {
            for sink in missile_flight_sound.iter() {
                sink.play();
            }
        }
    }
}

fn collide_rocket(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut particle_writer: EventWriter<EmitParticles>,
    rockets: Query<Entity, With<ShipRocket>>,
    enemies: Query<(&Enemy, &Velocity)>,
    sounds: Res<Sounds>,
) {
    for event in collision_events.read() {
        if rockets.contains(event.entity_a) {
            if let Ok((enemy, velocity)) = enemies.get(event.entity_b) {
                // Handle collision between rocket and enemy
                let contact = &event.contact;
                commands.entity(event.entity_a).despawn();
                commands.entity(event.entity_b).insert((
                    Enemy {
                        health: enemy.health - 50.0,
                    },
                    Velocity(velocity.0 + contact.normal * 500.0),
                ));
                particle_writer.write(EmitParticles {
                    position: contact.point_a,
                    count: 30,
                });
                info!("Rocket collided with enemy at {:?}", contact.point_b);
                commands.spawn((
                    Name::new("Boom Sound"),
                    AudioPlayer::new(sounds.enemy_die.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}
