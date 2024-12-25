use std::collections::HashMap;

use image::ImageReader;
use noise::{NoiseFn, Perlin};
use opengl_graphics::Texture;

use crate::{commons::{history_vec::Id, rng::Rng}, engine::{geometry::{Coord2, Size2D}, tilemap::{Tile16Subset, TileMap, TileSet}}, world::item::{Item, ItemMaker}, World};

use super::{actor::Actor, Renderable};

pub struct Chunk {
    size: Size2D,
    pub map: ChunkMap,
    tile_def: HashMap<u8, TileDef>,
    pub player: Actor,
    pub npcs: Vec<Actor>,
    pub killed_people: Vec<Id>,
    pub items_on_ground: Vec<(Coord2, Item, Texture)>
}

pub struct ChunkMap {
    tiles: Vec<Tile>,
    object_layer: TileMap,
}

impl ChunkMap {

    pub fn blocks_movement(&self, pos: Coord2) -> bool {
        if let crate::engine::tilemap::Tile::Empty = self.object_layer.get_tile(pos.x as usize, pos.y as usize) {
            return false
        }
        return true
    }

}

impl Chunk {
    pub fn new(size: Size2D, player: Actor) -> Chunk {
        let mut tile_def = HashMap::new();
        tile_def.insert(0, TileDef { sprite: (0, 0) }); // Grass
        tile_def.insert(1, TileDef { sprite: (1, 0) }); // Sand
        tile_def.insert(2, TileDef { sprite: (0, 1) }); // Stone
        tile_def.insert(3, TileDef { sprite: (1, 1) }); // Water
        tile_def.insert(4, TileDef { sprite: (2, 0) }); // Brick
        tile_def.insert(5, TileDef { sprite: (2, 1) }); // Wall

        let mut tileset = TileSet::new();
        let image = ImageReader::open("assets/sprites/stone_walls.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image, 16, 32)));

        Chunk {
            size,
            map: ChunkMap {
                tiles: vec![Tile { id: 0 }; size.area()],
                object_layer: TileMap::new(tileset, size.x(), size.y(), 16, 16),
            },
            player,
            npcs: Vec::new(),
            killed_people: Vec::new(),
            tile_def,
            items_on_ground: Vec::new()
        }
    }

    pub fn from_world_tile(world: &World, xy: Coord2, player: Actor) -> Chunk {
        let mut chunk = Self::new(Size2D(64, 64), player);
        let mut rng = Rng::rand();
        let tile = world.map.tile(xy.x as usize, xy.y as usize);
        let noise = Perlin::new(rng.derive("terrain").seed());
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                let idx = (y * chunk.size.x()) + x;
                let n = noise.get([x as f64 / 10.0, y as f64 / 10.0]);
                match tile.region_id {
                    0 => { // Ocean
                        chunk.map.tiles[idx].id = 3; // water
                    },
                    1 => { // Coastal
                        if n < -0.5 {
                            chunk.map.tiles[idx].id = 3; // water
                        } else {
                            chunk.map.tiles[idx].id = 1; // sand
                        }
                    },
                    2 => { // Forest - Grass
                        if n < 0.5 {
                            chunk.map.tiles[idx].id = 0; // grass
                        } else {
                            chunk.map.tiles[idx].id = 2; // stone
                        }
                    },
                    3 => { // Desert - Sand
                        if n < 0.5 {
                            chunk.map.tiles[idx].id = 1; // sand
                        } else {
                            chunk.map.tiles[idx].id = 2; // stone
                        }
                    },
                    _ => ()
                }
            }
        }
        let mut found_sett = None;
        for (_, settlement) in world.settlements.iter() {
            let settlement = settlement.borrow();
            if settlement.xy.0 as i32 == xy.x && settlement.xy.1 as i32 == xy.y {
                found_sett = Some(settlement);
                let rect_xy1 = Coord2::xy(rng.randu_range(8, 48) as i32, rng.randu_range(16, 48) as i32);
                let rect_size = Coord2::xy(rng.randu_range(6, 16) as i32, rng.randu_range(6, 16) as i32);
                let rect_xy2 = rect_xy1 + rect_size;
                for x in rect_xy1.x..rect_xy2.x + 1 {
                    for y in rect_xy1.y..rect_xy2.y + 1 {
                        let idx = (y * chunk.size.x() as i32) + x;
                        if x == rect_xy1.x || y == rect_xy1.y || x == rect_xy2.x || y == rect_xy2.y {
                            chunk.map.tiles[idx as usize].id = 5; // Wall
                            chunk.map.object_layer.set_tile(x as usize, y as usize, 1);
                        } else {
                            chunk.map.tiles[idx as usize].id = 4; // Floor
                        }
                    }
                }
            }
        }
        for (id, person) in world.people.iter() {
            let person = person.borrow();
            if person.position == xy {
                let point = chunk.get_spawn_pos(&mut rng);
                let species = world.species.get(&person.species).unwrap();
                chunk.npcs.push(Actor::from_person(point, *id, &person, &species, world));
            }
        }

        if let Some(_settlement) = found_sett {
            if chunk.npcs.len() == 0 {
                for _ in 0..rng.randu_range(3, 7) {
                    let point = chunk.get_spawn_pos(&mut rng);
                    let species = world.species.get(&Id(3) /* spider */).unwrap();
                    let npc = Actor::from_species(point, species);
                    chunk.npcs.push(npc);
                }

                let point = Coord2::xy(
                    rng.randu_range(0, chunk.size.x()) as i32,
                    rng.randu_range(0, chunk.size.y()) as i32
                );
                let item = ItemMaker::random(&rng, world);
                let texture = item.make_texture(world);
                chunk.items_on_ground.push((point, item, texture));
            }
        }

        chunk
    }

    pub fn get_spawn_pos(&self, rng: &mut Rng) -> Coord2 {
        let mut point = Coord2::xy(
                rng.randu_range(0, self.size.x()) as i32,
                rng.randu_range(0, self.size.y()) as i32
            );
        for _ in 0..10 {
            if !self.map.blocks_movement(point) {
                return point
            }
            point = Coord2::xy(
                rng.randu_range(0, self.size.x()) as i32,
                rng.randu_range(0, self.size.y()) as i32
            );
        }
        return point
    }

}

impl Renderable for Chunk {
    fn render(&self, ctx: &mut crate::engine::render::RenderContext) {
        for x in 0..self.size.x() {
            for y in 0..self.size.y() {
                let idx = (y * self.size.x()) + x;
                let tile = self.map.tiles.get(idx).unwrap();
                let tile = self.tile_def.get(&tile.id).unwrap();
                ctx.spritesheet("tiles.png", tile.sprite, [x as f64 * 16., y as f64 * 16.]);
            }
        }

        let mut actors_by_position = HashMap::new();
        actors_by_position.insert(&self.player.xy, vec!(&self.player));
        for npc in self.npcs.iter() {
            if !actors_by_position.contains_key(&npc.xy) {
                actors_by_position.insert(&npc.xy, Vec::new());
            }
            actors_by_position.get_mut(&npc.xy).unwrap().push(npc);
        }

        self.map.object_layer.render(ctx, |ctx, x, y| {
            if let Some(actors) = actors_by_position.get(&Coord2::xy(x as i32, y as i32)) {
                for actor in actors {
                    actor.render(ctx);
                }
            }
        });

        for (pos, _item, texture) in self.items_on_ground.iter() {
            ctx.texture_ref(texture, [pos.x as f64 * 16., pos.y as f64 * 16.]);
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