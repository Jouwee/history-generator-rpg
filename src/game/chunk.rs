use std::collections::HashMap;

use noise::{NoiseFn, Perlin};

use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::{Coord2, Size2D}, World};

use super::{actor::Actor, Renderable};

pub struct Chunk {
    size: Size2D,
    tiles: Vec<Tile>,
    tile_def: HashMap<u8, TileDef>,
    pub npcs: Vec<Actor>,
    pub killed_people: Vec<Id>
}

impl Chunk {
    pub fn new(size: Size2D) -> Chunk {
        let mut tile_def = HashMap::new();
        tile_def.insert(0, TileDef { sprite: (0, 0) }); // Grass
        tile_def.insert(1, TileDef { sprite: (1, 0) }); // Sand
        tile_def.insert(2, TileDef { sprite: (0, 1) }); // Stone
        tile_def.insert(3, TileDef { sprite: (1, 1) }); // Water
        tile_def.insert(4, TileDef { sprite: (2, 0) }); // Brick
        tile_def.insert(5, TileDef { sprite: (2, 1) }); // Wall
        Chunk {
            size,
            tiles: vec![Tile { id: 0 }; size.area()],
            npcs: Vec::new(),
            killed_people: Vec::new(),
            tile_def
        }
    }

    pub fn from_world_tile(world: &World, xy: Coord2) -> Chunk {
        let mut chunk = Self::new(Size2D(64, 64));
        let mut rng = Rng::rand();
        let tile = world.map.tile(xy.x as usize, xy.y as usize);
        let noise = Perlin::new(rng.derive("terrain").seed());
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                let idx = (y * chunk.size.x()) + x;
                let n = noise.get([x as f64 / 10.0, y as f64 / 10.0]);
                match tile.region_id {
                    0 => { // Ocean
                        chunk.tiles[idx].id = 3; // water
                    },
                    1 => { // Coastal
                        if n < -0.5 {
                            chunk.tiles[idx].id = 3; // water
                        } else {
                            chunk.tiles[idx].id = 1; // sand
                        }
                    },
                    2 => { // Forest - Grass
                        if n < 0.5 {
                            chunk.tiles[idx].id = 0; // grass
                        } else {
                            chunk.tiles[idx].id = 2; // stone
                        }
                    },
                    3 => { // Desert - Sand
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
        for (_, settlement) in world.settlements.iter() {
            let settlement = settlement.borrow();
            if settlement.xy.0 as i32 == xy.x && settlement.xy.1 as i32 == xy.y {
                let rect_xy1 = Coord2::xy(rng.randu_range(8, 48) as i32, rng.randu_range(16, 48) as i32);
                let rect_size = Coord2::xy(rng.randu_range(6, 16) as i32, rng.randu_range(6, 16) as i32);
                let rect_xy2 = rect_xy1 + rect_size;
                for x in rect_xy1.x..rect_xy2.x + 1 {
                    for y in rect_xy1.y..rect_xy2.y + 1 {
                        let idx = (y * chunk.size.x() as i32) + x;
                        if x == rect_xy1.x || y == rect_xy1.y || x == rect_xy2.x || y == rect_xy2.y {
                            chunk.tiles[idx as usize].id = 5; // Wall
                        } else {
                            chunk.tiles[idx as usize].id = 4; // Floor
                        }
                    }
                }
            }
        }
        for (id, person) in world.people.iter() {
            if person.borrow().position == xy {
                let point = Coord2::xy(
                    rng.randu_range(0, chunk.size.x()) as i32,
                    rng.randu_range(0, chunk.size.y()) as i32
                );
                chunk.npcs.push(Actor::new(point, super::actor::ActorType::Passive, Some(*id), Some(&person.borrow())));
            }
        }

        if chunk.npcs.len() == 0 {
            for _ in 0..rng.randu_range(3, 7) {
                let point = Coord2::xy(
                    rng.randu_range(0, chunk.size.x()) as i32,
                    rng.randu_range(0, chunk.size.y()) as i32
                );
                let npc = Actor::new(point, super::actor::ActorType::Hostile, None, None);
                chunk.npcs.push(npc);
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

#[derive(Clone)]
pub struct Tile {
    id: u8
}

pub struct TileDef {
    sprite: (u32, u32)
}