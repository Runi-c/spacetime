use std::f32::consts::PI;

use bevy::prelude::*;
use lyon_tessellation::{
    geom::{
        euclid::{Point2D, Size2D},
        Box2D,
    },
    path::Winding,
    StrokeOptions,
};

use crate::{
    layers::FactoryLayer,
    materials::{Dither, DitherMaterial, SOLID_WHITE},
    mesh::MeshLyonExtensions,
    resources::ResourceType,
    scheduling::Sets,
    z_order::ZOrder,
};

use super::{
    machines::{
        FlowDirection, Inlet, Machine, MachinePort, CONSTRUCTOR_MESH, CONSTRUCTOR_MESH_INNER,
        INLET_MESH, INLET_MESH_INNER,
    },
    pipe::{pipe_bridge, PipeFlowMaterial},
    tooltip::Tooltip,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_grid.in_set(Sets::Spawn));
}

pub const GRID_SIZE: usize = 10;
pub const TILE_SIZE: f32 = 50.0;
pub const GRID_WIDTH: f32 = GRID_SIZE as f32 * TILE_SIZE;

#[derive(Resource)]
pub struct Grid {
    pub tiles: [Option<Entity>; GRID_SIZE * GRID_SIZE],
    pub buildings: [Option<Entity>; GRID_SIZE * GRID_SIZE],
}

impl Grid {
    pub fn get_tile(&self, pos: IVec2) -> Option<Entity> {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x < GRID_SIZE && y < GRID_SIZE {
            self.tiles[y * GRID_SIZE + x]
        } else {
            None
        }
    }

    pub fn get_building(&self, pos: IVec2) -> Option<Entity> {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x < GRID_SIZE && y < GRID_SIZE {
            self.buildings[y * GRID_SIZE + x]
        } else {
            None
        }
    }

    pub fn get_building_mut(&mut self, pos: IVec2) -> Option<&mut Option<Entity>> {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x < GRID_SIZE && y < GRID_SIZE {
            Some(&mut self.buildings[y * GRID_SIZE + x])
        } else {
            None
        }
    }
}

#[derive(Component, Debug)]
pub struct Tile;

#[derive(Component, Debug)]
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

#[derive(PartialEq, Eq)]
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

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DitherMaterial>>,
    flow_material: Res<PipeFlowMaterial>,
) {
    let mut grid = Grid {
        tiles: [None; GRID_SIZE * GRID_SIZE],
        buildings: [None; GRID_SIZE * GRID_SIZE],
    };
    let mesh = meshes.add(Mesh::stroke_with(
        |builder| {
            builder.add_rectangle(
                &Box2D::from_origin_and_size(Point2D::zero(), Size2D::splat(TILE_SIZE)),
                Winding::Positive,
            );
        },
        &StrokeOptions::default().with_line_width(1.0),
    ));
    commands
        .spawn((
            Name::new("Grid"),
            FactoryLayer,
            Transform::from_xyz(GRID_WIDTH * -0.5, GRID_WIDTH * -0.5, ZOrder::TILE),
            InheritedVisibility::VISIBLE,
        ))
        .with_children(|parent| {
            for y in 1..GRID_SIZE - 1 {
                for x in 1..GRID_SIZE - 1 {
                    let tile = parent.spawn((
                        Name::new("Tile"),
                        Tile,
                        FactoryLayer,
                        TileCoords(IVec2::new(x as i32, y as i32)),
                        Mesh2d(mesh.clone()),
                        MeshMaterial2d(SOLID_WHITE),
                        Transform::from_xyz(
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                            ZOrder::TILE,
                        ),
                    ));
                    grid.tiles[y * GRID_SIZE + x] = Some(tile.id());
                }
            }
            for y in [3, 6] {
                let inlet = parent.spawn((
                    Name::new("Mineral Inlet"),
                    Inlet(ResourceType::Mineral),
                    FactoryLayer,
                    TileCoords(IVec2::new(0, y)),
                    Mesh2d(INLET_MESH),
                    MeshMaterial2d(SOLID_WHITE),
                    Tooltip("Mineral Inlet".to_string()),
                    Transform::from_xyz(
                        0.5 * TILE_SIZE,
                        (y as f32 + 0.5) * TILE_SIZE,
                        ZOrder::MACHINE,
                    ),
                    children![
                        (
                            Name::new("Mineral Inlet Inner"),
                            FactoryLayer,
                            Mesh2d(INLET_MESH_INNER),
                            MeshMaterial2d(materials.add(Dither {
                                fill: 0.9,
                                scale: 0.1,
                                ..default()
                            })),
                            Transform::from_xyz(0.0, 0.0, 0.2),
                        ),
                        (
                            Name::new("Mineral Inlet Port"),
                            TileCoords(IVec2::new(0, y)),
                            MachinePort {
                                side: Direction::Right,
                                flow: FlowDirection::Outlet,
                            },
                            Transform::default(),
                            InheritedVisibility::VISIBLE,
                            children![pipe_bridge(
                                false,
                                Direction::Right,
                                flow_material.0.clone()
                            )]
                        ),
                    ],
                ));
                grid.buildings[y as usize * GRID_SIZE + 0] = Some(inlet.id());
            }

            let machine = parent.spawn((
                Name::new("Test Machine"),
                Machine,
                FactoryLayer,
                TileCoords(IVec2::new(5, 5)),
                Mesh2d(CONSTRUCTOR_MESH),
                MeshMaterial2d(SOLID_WHITE),
                Transform::from_xyz(5.5 * TILE_SIZE, 5.5 * TILE_SIZE, ZOrder::MACHINE),
                children![
                    (
                        Name::new("Test Machine Inner"),
                        FactoryLayer,
                        Mesh2d(CONSTRUCTOR_MESH_INNER),
                        MeshMaterial2d(materials.add(Dither {
                            fill: 0.5,
                            scale: 0.1,
                            ..default()
                        })),
                        Transform::from_xyz(0.0, 0.0, 0.2),
                    ),
                    (
                        Name::new("Test Machine Port"),
                        TileCoords(IVec2::new(5, 5)),
                        MachinePort {
                            side: Direction::Right,
                            flow: FlowDirection::Inlet,
                        },
                        Transform::default(),
                        InheritedVisibility::VISIBLE,
                        children![pipe_bridge(
                            false,
                            Direction::Right,
                            flow_material.0.clone()
                        ),]
                    ),
                    (
                        Name::new("Test Machine Port"),
                        TileCoords(IVec2::new(5, 5)),
                        MachinePort {
                            side: Direction::Up,
                            flow: FlowDirection::Outlet,
                        },
                        Transform::default(),
                        InheritedVisibility::VISIBLE,
                        children![pipe_bridge(false, Direction::Up, flow_material.0.clone()),]
                    ),
                ],
            ));
            grid.buildings[5 * GRID_SIZE + 5] = Some(machine.id());
        });
    commands.insert_resource(grid);
}
