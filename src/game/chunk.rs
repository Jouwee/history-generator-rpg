use std::collections::HashMap;

use noise::{NoiseFn, Perlin};

use crate::{commons::{history_vec::Id, rng::Rng}, engine::Point2D, WorldGraph};
use super::{Renderable, NPC};

pub struct Chunk {
    size: ChunkSize,
    tiles: Vec<Tile>,
    tile_def: HashMap<u8, TileDef>,
    pub npcs: Vec<NPC>,
    pub killed_people: Vec<Id>
}

impl Chunk {
    pub fn new(size: ChunkSize) -> Chunk {
        let mut tile_def = HashMap::new();
        tile_def.insert(0, TileDef { sprite: (0, 0) }); // Grass
        tile_def.insert(1, TileDef { sprite: (1, 0) }); // Sand
        tile_def.insert(2, TileDef { sprite: (0, 1) }); // Stone
        tile_def.insert(3, TileDef { sprite: (1, 1) }); // Water
        Chunk {
            size,
            tiles: vec![Tile { id: 0 }; size.area()],
            npcs: Vec::new(),
            killed_people: Vec::new(),
            tile_def
        }
    }

    pub fn from_world_tile(world: &WorldGraph, xy: Point2D) -> Chunk {
        let mut chunk = Self::new(ChunkSize(64, 64));
        let mut rng = Rng::rand();
        let tile = world.map.get_world_tile(xy.0, xy.1);
        let noise = Perlin::new(rng.derive("terrain").seed());
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                let idx = (y * chunk.size.x()) + x;
                let n = noise.get([x as f64 / 10.0, y as f64 / 10.0]);
                match tile.region_id {
                    0 => { // Coastal
                        if n < -0.5 {
                            chunk.tiles[idx].id = 3; // water
                        } else {
                            chunk.tiles[idx].id = 1; // sand
                        }
                    },
                    1 => { // Forest - Grass
                        if n < 0.5 {
                            chunk.tiles[idx].id = 0; // grass
                        } else {
                            chunk.tiles[idx].id = 2; // stone
                        }
                    },
                    2 => { // Desert - Sand
                        if n < 0.5 {
                            chunk.tiles[idx].id = 1; // sand
                        } else {
                            chunk.tiles[idx].id = 2; // stone
                        }
                    },
                    _ => ()
                }
            }
        }
        for (id, person) in world.people.iter() {
            if person.borrow().position == xy {
                let point = Point2D(
                    rng.randu_range(0, chunk.size.x()),
                    rng.randu_range(0, chunk.size.y())
                );
                chunk.npcs.push(NPC::new(point, *id, &person.borrow()));
            }
        }
        chunk
    }
}

impl Renderable for Chunk {
    fn render(&self, ctx: &mut crate::engine::render::RenderContext) {
        for x in 0..self.size.x() {
            for y in 0..self.size.y() {
                let idx = (y * self.size.x()) + x;
                let tile = self.tiles.get(idx).unwrap();
                let tile = self.tile_def.get(&tile.id).unwrap();
                ctx.spritesheet("tiles.png", tile.sprite, [x as f64 * 16., y as f64 * 16.]);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ChunkSize(usize, usize);

impl ChunkSize {
    pub fn x(&self) -> usize {
        self.0
    }
    pub fn y(&self) -> usize {
        self.1
    }
    pub fn area(&self) -> usize {
        return self.x() * self.y()
    }
}

#[derive(Clone)]
pub struct Tile {
    id: u8
}

pub struct TileDef {
    sprite: (u32, u32)
}