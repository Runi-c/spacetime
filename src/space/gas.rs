use std::time::Duration;

use bevy::prelude::*;
use lyon_tessellation::{geom::euclid::Point2D, path::Winding};
use rand::Rng;

use crate::{
    layers::SpaceLayer,
    materials::{DitherMaterial, RockyDither},
    mesh::MeshLyonExtensions,
    resources::{ResourceType, Resources},
    scheduling::Sets,
    sounds::Sounds,
    z_order::ZOrder,
};

use super::{
    bounds::ScreenBounds,
    collision::{Collider, CollisionEvent},
    physics::{Spin, Velocity},
    ship::Ship,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            gas_spawn_timer.in_set(Sets::PreUpdate),
            gas_succ.in_set(Sets::Update),
            gas_dissipate.in_set(Sets::PostUpdate),
        ),
    );
}

#[derive(Component, Clone)]
pub struct GasCloud {
    pub remaining: f32,
}

fn gas_spawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    clouds: Query<&GasCloud>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let count = clouds.iter().count();
    if count >= 2 {
        return;
    }
    if let Some(timer) = timer.as_mut() {
        if timer.tick(time.delta()).just_finished() || keys.just_pressed(KeyCode::KeyG) {
            commands.run_system_cached(spawn_cloud);
        }
    } else {
        timer.replace(Timer::new(
            Duration::from_secs_f32(20.0),
            TimerMode::Repeating,
        ));
    }
}

fn spawn_cloud(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    bounds: ScreenBounds,
) {
    let pos = bounds.random_outside() * 1.2;
    let target = bounds.random_inside() * 0.8;
    let mesh = Mesh::fill_with(|builder| {
        let mut rng = rand::thread_rng();
        builder.add_circle(
            Point2D::new(rng.gen_range(-5.0..5.0), rng.gen_range(-5.0..5.0)),
            40.0,
            Winding::Positive,
        );
        for _ in 0..5 {
            builder.add_circle(
                Point2D::new(rng.gen_range(-40.0..40.0), rng.gen_range(-40.0..40.0)),
                rng.gen_range(15.0..30.0),
                Winding::Positive,
            );
        }
    });
    commands.spawn((
        Name::new("Gas Cloud"),
        GasCloud { remaining: 10.0 },
        SpaceLayer,
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(RockyDither {
            fill: 0.5,
            scale: 100.0,
        })),
        Transform::from_xyz(pos.x, pos.y, 0.0),
        ZOrder::ASTEROID,
        Collider::from_circle(60.0),
        Velocity::random_towards(target - pos, 30.0..70.0),
        Spin::random(30.0),
    ));
}

#[derive(Component, Clone)]
pub struct SuccSound;

fn gas_succ(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    clouds: Query<&GasCloud>,
    ships: Query<Entity, With<Ship>>,
    mut resources: ResMut<Resources>,
    succ_sound: Query<&AudioSink, With<SuccSound>>,
    sounds: Res<Sounds>,
    time: Res<Time>,
) {
    let mut is_succ = false;
    for event in collision_events.read() {
        if let Ok(cloud) = clouds.get(event.entity_a) {
            if let Ok(ship) = ships.get(event.entity_b) {
                let succ_amount = (time.delta_secs() * 2.5).min(cloud.remaining);
                commands.entity(event.entity_a).insert(GasCloud {
                    remaining: cloud.remaining - succ_amount,
                });
                resources.add(ResourceType::Gas, succ_amount);
                // play succ sound
                is_succ = true;
                if succ_sound.is_empty() {
                    commands.spawn((
                        Name::new("Succ Sound"),
                        SuccSound,
                        AudioPlayer::new(sounds.succ.clone()),
                        PlaybackSettings::LOOP,
                        ChildOf(ship),
                    ));
                }
                for sink in succ_sound.iter() {
                    sink.play();
                }
            }
        }
    }

    if !is_succ {
        for sink in succ_sound.iter() {
            sink.pause();
        }
    }
}

fn gas_dissipate(
    mut commands: Commands,
    mut materials: ResMut<Assets<DitherMaterial>>,
    query: Query<(Entity, &GasCloud, &MeshMaterial2d<DitherMaterial>)>,
) {
    for (entity, cloud, material) in query.iter() {
        if cloud.remaining <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            let material = materials.get_mut(material.0.id()).unwrap();
            material.settings.fill = 0.2 + 0.3 * cloud.remaining / 10.0;
        }
    }
}
