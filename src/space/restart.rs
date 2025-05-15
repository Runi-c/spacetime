use bevy::prelude::*;

use crate::game_over::RestartGame;

use super::{
    asteroid::Asteroid,
    enemy::{Enemy, EnemyBullet},
    gas::GasCloud,
    particles::Particle,
    pickup::Pickup,
    ship::{ship_spawn, Ship, ShipBullet, ShipRocket},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(space_restart);
}

fn space_restart(
    _trigger: Trigger<RestartGame>,
    mut commands: Commands,
    asteroids: Query<Entity, With<Asteroid>>,
    enemies: Query<Entity, With<Enemy>>,
    ships: Query<Entity, With<Ship>>,
    gas_clouds: Query<Entity, With<GasCloud>>,
    pickups: Query<Entity, With<Pickup>>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    ship_bullets: Query<Entity, With<ShipBullet>>,
    rockets: Query<Entity, With<ShipRocket>>,
    particles: Query<Entity, With<Particle>>,
) {
    for entity in asteroids.iter() {
        commands.entity(entity).despawn();
    }
    for entity in enemies.iter() {
        commands.entity(entity).despawn();
    }
    for entity in ships.iter() {
        commands.entity(entity).despawn();
    }
    for entity in gas_clouds.iter() {
        commands.entity(entity).despawn();
    }
    for entity in pickups.iter() {
        commands.entity(entity).despawn();
    }
    for entity in enemy_bullets.iter() {
        commands.entity(entity).despawn();
    }
    for entity in ship_bullets.iter() {
        commands.entity(entity).despawn();
    }
    for entity in rockets.iter() {
        commands.entity(entity).despawn();
    }
    for entity in particles.iter() {
        commands.entity(entity).despawn();
    }
    commands.run_system_cached(ship_spawn);
}
