use bevy::prelude::*;
use lyon_tessellation::{
    geom::{
        euclid::{Point2D, Size2D},
        Box2D,
    },
    path::Winding,
    StrokeOptions,
};

use crate::{layers::Layers, mesh::MeshLyonExtensions, z_order::ZOrder};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_grid);
}

const GRID_SIZE: usize = 8;

#[derive(Resource)]
pub struct Grid {
    pub tiles: [Entity; GRID_SIZE * GRID_SIZE],
}

impl Grid {
    pub fn get_tile(&self, x: usize, y: usize) -> Option<Entity> {
        if x < GRID_SIZE && y < GRID_SIZE {
            Some(self.tiles[y * GRID_SIZE + x])
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct Tile {
    x: usize,
    y: usize,
}

pub const TILE_SIZE: f32 = 50.0;

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut tiles = Vec::with_capacity(GRID_SIZE * GRID_SIZE);
    let mesh = meshes.add(Mesh::stroke_with(
        |builder| {
            builder.add_rectangle(
                &Box2D::from_origin_and_size(Point2D::zero(), Size2D::splat(TILE_SIZE)),
                Winding::Positive,
            );
        },
        &StrokeOptions::default().with_line_width(2.0),
    ));
    commands
        .spawn((
            Layers::FACTORY,
            Transform::from_xyz(
                GRID_SIZE as f32 * -TILE_SIZE / 2.0,
                GRID_SIZE as f32 * -TILE_SIZE / 2.0,
                ZOrder::TILE,
            ),
            InheritedVisibility::VISIBLE,
        ))
        .with_children(|parent| {
            for y in 0..GRID_SIZE {
                for x in 0..GRID_SIZE {
                    let tile = parent
                        .spawn((
                            Tile { x, y },
                            Layers::FACTORY,
                            Mesh2d(mesh.clone()),
                            MeshMaterial2d(materials.add(Color::WHITE)),
                            Transform::from_xyz(
                                x as f32 * TILE_SIZE,
                                y as f32 * TILE_SIZE,
                                ZOrder::TILE,
                            ),
                        ))
                        .id();
                    tiles.push(tile);
                }
            }
        });
    commands.insert_resource(Grid {
        tiles: tiles.try_into().unwrap(),
    });
}
