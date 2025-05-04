use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use parry2d::query;

use crate::{
    asteroid::Asteroid,
    collision::{transform_to_isometry, Collider},
    particles::SpawnParticles,
    physics::{Rotation, Velocity},
    utils::parry2d_vec2,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ship)
        .add_systems(Update, (ship_input, ship_laser));
}

#[derive(Component)]
pub struct Ship;

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut ship = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let vertices = vec![
        [50.0, 0.0, 0.0],
        [-50.0, 25.0, 0.0],
        [-35.0, 0.0, 0.0],
        [-50.0, -25.0, 0.0],
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    ship.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
    ship.insert_indices(Indices::U32(indices));

    let mesh = meshes.add(ship);
    commands.spawn((
        Ship,
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Collider::from_vertices(vertices),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity(Vec2::ZERO),
        Rotation(0.0),
    ));
}

fn ship_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Rotation, &mut Velocity), With<Ship>>,
) {
    const SPEED: f32 = 210.0;
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

    for (transform, mut rotation, mut velocity) in query.iter_mut() {
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
    }
}

pub fn ship_laser(
    time: Res<Time>,
    mut gizmos: Gizmos,
    ship: Query<(&Transform, &Collider), With<Ship>>,
    mut asteroids: Query<(&mut Asteroid, &Transform, &Collider)>,
    mut particles: EventWriter<SpawnParticles>,
) {
    for (ship_transform, ship_collider) in ship.iter() {
        for (mut asteroid, asteroid_transform, asteroid_collider) in asteroids.iter_mut() {
            let distance = ship_transform
                .translation
                .distance(asteroid_transform.translation);
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
                    particles.write(SpawnParticles {
                        position: parry2d_vec2(contact.point2),
                        count: 1,
                    });
                    gizmos.line_2d(
                        ship_transform.translation.truncate(),
                        parry2d_vec2(contact.point2),
                        Color::WHITE,
                    );
                    asteroid.health -= time.delta_secs() * 25.0;
                }
            }
        }
    }
}
