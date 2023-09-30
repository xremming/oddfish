use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use bevy::{ecs::system::Command, prelude::*};

pub struct Region {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Terrain {
    City,
    Town,
    Forest,
    Mountain,
    Water,
    Plains,
    Road,
}

impl Terrain {
    pub fn debug_color(&self) -> Color {
        use Terrain::*;
        match self {
            City => Color::GRAY,
            Town => Color::DARK_GRAY,
            Forest => Color::GREEN,
            Mountain => Color::BLACK,
            Water => Color::BLUE,
            Plains => Color::YELLOW,
            Road => Color::WHITE,
        }
    }

    pub fn as_display(self, sprite: impl Into<String>) -> TerrainDisplay {
        TerrainDisplay {
            terrain: self,
            sprite: sprite.into(),
        }
    }
}

pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Component, Default, Debug)]
pub struct Neighbors {
    pub north: Option<Entity>,
    pub east: Option<Entity>,
    pub south: Option<Entity>,
    pub west: Option<Entity>,
}

impl Neighbors {
    /// Returns the direction in which the given entity is a neighbor, if any.
    pub fn is_neighbor(&self, other: Entity) -> Option<Direction> {
        if self.north == Some(other) {
            Some(Direction::North)
        } else if self.east == Some(other) {
            Some(Direction::East)
        } else if self.south == Some(other) {
            Some(Direction::South)
        } else if self.west == Some(other) {
            Some(Direction::West)
        } else {
            None
        }
    }

    pub fn update_neighbors(
        &mut self,
        (x, y): (usize, usize),
        map: &HashMap<(usize, usize), Entity>,
    ) {
        self.north = map.get(&(x, y + 1)).copied();
        self.east = map.get(&(x + 1, y)).copied();

        if y == 0 {
            self.south = None;
        } else {
            self.south = map.get(&(x, y - 1)).copied();
        }

        if x == 0 {
            self.west = None;
        } else {
            self.west = map.get(&(x - 1, y)).copied();
        }
    }
}

#[derive(Component)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub terrain: Terrain,
}

#[derive(Bundle)]
pub struct TileBundle {
    pub tile: Tile,
    pub neighbors: Neighbors,
    // pub transform: Transform,
    // pub global_transform: GlobalTransform,
}

#[derive(Clone)]
pub struct TerrainDisplay {
    pub terrain: Terrain,
    pub sprite: String,
}

pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TerrainDisplay>>,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "TileMap must have non-zero dimensions"
        );

        Self {
            width,
            height,
            tiles: vec![
                vec![
                    TerrainDisplay {
                        terrain: Terrain::Water,
                        sprite: "water.png".into()
                    };
                    width
                ];
                height
            ],
        }
    }
}

pub const TILE_SIZE: f32 = 16.0;
const SCALE_FACTOR: f32 = 2.0;

impl Command for TileMap {
    fn apply(self, world: &mut World) {
        let asset_server = world.resource::<AssetServer>();

        let mut bundles = Vec::with_capacity(self.width * self.height);
        for (y, column) in self.tiles.iter().enumerate() {
            for (x, terrain) in column.iter().enumerate() {
                bundles.push((
                    SpriteBundle {
                        texture: asset_server.load(terrain.sprite.clone()),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 * TILE_SIZE * SCALE_FACTOR,
                            y as f32 * TILE_SIZE * SCALE_FACTOR,
                            0.0,
                        ))
                        .with_scale(Vec3::splat(SCALE_FACTOR)),
                        ..default()
                    },
                    TileBundle {
                        tile: Tile {
                            x,
                            y,
                            terrain: terrain.terrain,
                        },
                        neighbors: default(),
                    },
                ));
            }
        }

        world.spawn_batch(bundles);
    }
}

impl Index<(usize, usize)> for TileMap {
    type Output = TerrainDisplay;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.tiles[y][x]
    }
}

impl IndexMut<(usize, usize)> for TileMap {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.tiles[y][x]
    }
}
