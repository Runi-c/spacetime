use bevy::prelude::*;
use parry2d::query;
use rand::Rng;

use crate::{
    camera::UICamera,
    factory::{setup_grid, Grid},
    layers::SpaceLayer,
    materials::{DitherMaterial, MetalDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::Resources,
    scheduling::Sets,
    sounds::Sounds,
    space::physics::DespawnOutOfBounds,
    utils::parry2d_vec2,
    z_order::ZOrder,
};

use super::{
    asteroid::Asteroid,
    collision::{transform_to_isometry, Collider},
    enemy::Enemy,
    particles::SpawnParticles,
    physics::{Rotation, Velocity},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ship.in_set(Sets::Spawn))
        .add_systems(
            Update,
            (
                (ship_input, restart_game).in_set(Sets::Input),
                ship_laser.in_set(Sets::Update),
                (display_ship_health, destroy_ship)
                    .chain()
                    .in_set(Sets::PostUpdate),
            ),
        );
}

#[derive(Component, Clone)]
pub struct Ship {
    pub health: f32,
}

#[derive(Component, Clone)]
pub struct ShipHealthMarker;

#[derive(Component, Clone)]
pub struct LaserSound;

#[derive(Component, Clone)]
pub struct RocketSound;

fn spawn_ship(
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
        Ship { health: 100.0 },
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

#[derive(Component)]
pub struct GameOverScreen;

fn destroy_ship(
    mut commands: Commands,
    query: Query<(Entity, &Ship, &Transform), With<Ship>>,
    mut particles: EventWriter<SpawnParticles>,
    sounds: Res<Sounds>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ui_camera: Query<Entity, With<UICamera>>,
) {
    for (entity, ship, transform) in query.iter() {
        if ship.health <= 0.0 || keyboard_input.just_pressed(KeyCode::KeyK) {
            commands.entity(entity).despawn();
            particles.write(SpawnParticles {
                position: transform.translation.truncate(),
                count: 10,
            });
            commands.spawn((
                Name::new("Death Sound"),
                AudioPlayer::new(sounds.player_death.clone()),
                PlaybackSettings::DESPAWN,
            ));
            let ui_camera = ui_camera.single().unwrap();
            commands.spawn((
                Name::new("Gameover Screen"),
                GameOverScreen,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    right: Val::ZERO,
                    top: Val::ZERO,
                    bottom: Val::ZERO,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                UiTargetCamera(ui_camera),
                children![(
                    Node {
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    BorderColor(Color::WHITE),
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    Text("You died!\nPress R to restart.".to_string()),
                )],
            ));
        }
    }
}

fn restart_game(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_over_screen: Query<Entity, With<GameOverScreen>>,
    grid: Res<Grid>,
    asteroids: Query<Entity, With<Asteroid>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for entity in game_over_screen.iter() {
            commands.entity(entity).despawn();
        }
        commands.entity(grid.entity).despawn();
        for entity in asteroids.iter() {
            commands.entity(entity).despawn();
        }
        for entity in enemies.iter() {
            commands.entity(entity).despawn();
        }
        commands.insert_resource(Resources::default());
        commands.run_system_cached(setup_grid);
        commands.run_system_cached(spawn_ship);
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
    mut rocket_sound: Query<(Entity, &mut AudioSink), With<RocketSound>>,
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
            for (entity, sink) in rocket_sound.iter_mut() {
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
                for (entity, sink) in rocket_sound.iter_mut() {
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
                Velocity(Vec2::from_angle(angle) * 1500.0),
                Rotation(0.0),
                DespawnOutOfBounds,
            ));
            commands.entity(ship).insert(ShootyCooldown(0.5));
            commands.spawn((
                Name::new("Shooty Gun Noise"),
                AudioPlayer::new(sounds.gun.clone()),
                PlaybackSettings::DESPAWN,
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
    mut particles: EventWriter<SpawnParticles>,
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
                let contact = query::contact(
                    &transform_to_isometry(ship_transform),
                    &ship_collider.0,
                    &transform_to_isometry(asteroid_transform),
                    &asteroid_collider.0,
                    distance,
                )
                .unwrap();
                if let Some(contact) = contact {
                    if rand::thread_rng().gen_bool(0.1) {
                        particles.write(SpawnParticles {
                            position: parry2d_vec2(contact.point2),
                            count: 1,
                        });
                    }
                    gizmos.line_2d(
                        ship_transform.translation.truncate(),
                        parry2d_vec2(contact.point2),
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
    ship: Query<(Entity, &Ship), Changed<Ship>>,
    health_marker: Query<(Entity, &ChildOf, &Transform), With<ShipHealthMarker>>,
) {
    for (entity, ship) in ship.iter() {
        for (marker_entity, child, transform) in health_marker.iter() {
            if child.parent() == entity {
                let scale = ship.health / 100.0;
                commands.entity(marker_entity).insert(Transform {
                    scale: Vec3::splat(scale),
                    ..*transform
                });
            }
        }
    }
}
