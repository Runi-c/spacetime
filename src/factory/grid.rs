use std::f32::consts::PI;

use bevy::prelude::*;
use lyon_tessellation::{geom::Box2D, path::Winding, StrokeOptions};

use crate::{
    layers::FactoryLayer,
    materials::{DitherMaterial, GassyDither, MetalDither, RockyDither, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::ResourceType,
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    machines::{inlet, outlet},
    pipe::PipeFlowMaterial,
    tooltip::Tooltip,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, grid_spawn.in_set(Sets::Spawn))
        .add_systems(Update, apply_coords.in_set(Sets::PostUpdate));
}

pub const GRID_SIZE: usize = 10;
pub const TILE_SIZE: f32 = 50.0;
pub const GRID_WIDTH: f32 = GRID_SIZE as f32 * TILE_SIZE;

#[derive(Resource)]
pub struct Grid {
    pub entity: Entity,
    tiles: [Option<Entity>; GRID_SIZE * GRID_SIZE],
    buildings: [Option<Entity>; GRID_SIZE * GRID_SIZE],
}

impl Grid {
    pub fn get_tile(&self, pos: IVec2) -> Option<Entity> {
        self.try_index(pos).and_then(|index| self.tiles[index])
    }

    pub fn get_building(&self, pos: IVec2) -> Option<Entity> {
        self.try_index(pos).and_then(|index| self.buildings[index])
    }

    pub fn insert_building(&mut self, pos: IVec2, entity: Entity) -> Option<Entity> {
        self.try_index(pos).and_then(|index| {
            let old = self.buildings[index];
            self.buildings[index] = Some(entity);
            old
        })
    }

    pub fn remove_building(&mut self, pos: IVec2) -> Option<Entity> {
        self.try_index(pos).and_then(|index| {
            let old = self.buildings[index];
            self.buildings[index] = None;
            old
        })
    }

    fn try_index(&self, pos: IVec2) -> Option<usize> {
        if pos.x >= 0 && pos.y >= 0 && pos.x < GRID_SIZE as i32 && pos.y < GRID_SIZE as i32 {
            Some((pos.y * GRID_SIZE as i32 + pos.x) as usize)
        } else {
            None
        }
    }
}

pub fn grid_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    flow_material: Res<PipeFlowMaterial>,
) {
    let mut tiles = [None; GRID_SIZE * GRID_SIZE];
    let mut buildings = [None; GRID_SIZE * GRID_SIZE];
    let mesh = meshes.add(Mesh::stroke_with(
        |builder| {
            builder.add_rectangle(
                &Box2D::zero().inflate(TILE_SIZE * 0.5, TILE_SIZE * 0.5),
                Winding::Positive,
            );
        },
        &StrokeOptions::default().with_line_width(1.0),
    ));
    let grid = commands
        .spawn((
            Name::new("Grid"),
            FactoryLayer,
            Transform::from_xyz(GRID_WIDTH * -0.5, GRID_WIDTH * -0.5, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            for y in 1..GRID_SIZE - 1 {
                for x in 1..GRID_SIZE - 1 {
                    let tile = parent.spawn((
                        Name::new("Tile"),
                        Tile,
                        FactoryLayer,
                        TileCoords(ivec2(x as i32, y as i32)),
                        ZOrder::TILE,
                        Mesh2d(mesh.clone()),
                        MeshMaterial2d(SOLID_WHITE),
                    ));
                    tiles[y * GRID_SIZE + x] = Some(tile.id());
                }
            }
            for y in [3, 6] {
                let inlet = parent.spawn((
                    Tooltip(
                        "Mineral Inlet".to_string(),
                        Some("Provides minerals to connected machines".to_string()),
                    ),
                    inlet(
                        ResourceType::Mineral,
                        ivec2(0, y),
                        Direction::Right,
                        materials.add(RockyDither {
                            fill: 0.6,
                            scale: 40.0,
                        }),
                        flow_material.0.clone(),
                    ),
                ));
                buildings[y as usize * GRID_SIZE + 0] = Some(inlet.id());
            }
            for y in [3, 6] {
                let inlet = parent.spawn((
                    Tooltip(
                        "Gas Inlet".to_string(),
                        Some("Provides gas to connected machines".to_string()),
                    ),
                    inlet(
                        ResourceType::Gas,
                        ivec2(GRID_SIZE as i32 - 1, y),
                        Direction::Left,
                        materials.add(GassyDither {
                            fill: 0.8,
                            scale: 40.0,
                        }),
                        flow_material.0.clone(),
                    ),
                ));
                buildings[y as usize * GRID_SIZE + GRID_SIZE - 1] = Some(inlet.id());
            }
            for x in [3, 6] {
                let outlet = parent.spawn((
                    Tooltip(
                        "Ammo Outlet".to_string(),
                        Some("Accepts ammo for the ship's weapons".to_string()),
                    ),
                    outlet(
                        ivec2(x, GRID_SIZE as i32 - 1),
                        Direction::Down,
                        materials.add(MetalDither {
                            fill: 0.2,
                            scale: 40.0,
                        }),
                        flow_material.0.clone(),
                    ),
                ));
                buildings[(GRID_SIZE - 1) * GRID_SIZE + x as usize] = Some(outlet.id());
            }
        })
        .id();
    commands.insert_resource(Grid {
        entity: grid,
        tiles,
        buildings,
    });
}

#[derive(Component, Clone, Debug)]
pub struct Tile;

#[derive(Component, Clone, Debug)]
#[require(Transform)]
pub struct TileCoords(pub IVec2);

impl TileCoords {
    pub fn direction_to(&self, other: &TileCoords) -> Direction {
        if self.0.x == other.0.x {
            if self.0.y < other.0.y {
                Direction::Up
            } else {
                Direction::Down
            }
        } else if self.0.x < other.0.x {
            Direction::Right
        } else {
            Direction::Left
        }
    }
}

fn apply_coords(mut query: Query<(&TileCoords, &mut Transform), Changed<TileCoords>>) {
    for (coords, mut transform) in query.iter_mut() {
        let target = coords.0.as_vec2() * TILE_SIZE + TILE_SIZE * 0.5;
        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Direction {
    pub fn iter() -> impl Iterator<Item = Direction> {
        [
            Direction::Right,
            Direction::Up,
            Direction::Left,
            Direction::Down,
        ]
        .into_iter()
    }
    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Direction::Right => Vec2::new(1.0, 0.0),
            Direction::Up => Vec2::new(0.0, 1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Down => Vec2::new(0.0, -1.0),
        }
    }
    pub fn as_ivec2(&self) -> IVec2 {
        match self {
            Direction::Right => IVec2::new(1, 0),
            Direction::Up => IVec2::new(0, 1),
            Direction::Left => IVec2::new(-1, 0),
            Direction::Down => IVec2::new(0, -1),
        }
    }
    pub fn angle(&self) -> f32 {
        match self {
            Direction::Right => 0.0,
            Direction::Up => PI / 2.0,
            Direction::Left => PI,
            Direction::Down => 3.0 * PI / 2.0,
        }
    }
    pub fn flip(&self) -> Direction {
        match self {
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
        }
    }
}
