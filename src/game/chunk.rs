use std::collections::HashMap;

use image::ImageReader;
use noise::{NoiseFn, Perlin};
use opengl_graphics::Texture;

use crate::{commons::{history_vec::Id, rng::Rng}, engine::{geometry::{Coord2, Size2D, Vec2}, tilemap::{Tile16Subset, TileMap, TileSet, TileSingle}}, world::item::{Item, ItemMaker}, World};

use super::{actor::Actor, Renderable};

pub struct Chunk {
    size: Size2D,
    pub map: ChunkMap,
    tile_def: HashMap<u8, TileDef>,
    pub player: Actor,
    pub npcs: Vec<Actor>,
    pub killed_people: Vec<Id>,
    pub items_on_ground: Vec<(Coord2, Item, Texture)>,
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
        let image = ImageReader::open("assets/sprites/tree.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageReader::open("assets/sprites/bed.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageReader::open("assets/sprites/table.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageReader::open("assets/sprites/stool.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));

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
            items_on_ground: Vec::new(),
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

        let noise = Perlin::new(rng.derive("trees").seed());
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                if noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0. {
                    if rng.rand_chance(0.1) {
                        chunk.map.object_layer.set_tile(x as usize, y as usize, 2);
                    }
                }
            }
        }

        let mut found_sett = None;
        for (_, settlement) in world.settlements.iter() {
            let settlement = settlement.borrow();
            if settlement.xy.0 as i32 == xy.x && settlement.xy.1 as i32 == xy.y {
                let num_builds = (settlement.demographics.population / 20).clamp(1, 9) as usize;
                let buildings = chunk.prepare_buildings(&mut rng, num_builds, 100);
                for building in buildings {
                    chunk.make_building(&mut rng, building);
                }
                found_sett = Some(settlement);
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

    pub fn prepare_buildings(&self, rng: &mut Rng, num_buildings: usize, tries: u32) -> Vec<(Coord2, Coord2)> {
        let mut buildings = Vec::new();
        for _ in 0..num_buildings {
            let center = Vec2::xy(rng.randf_range(8., 56.), rng.randf_range(16., 56.));
            let radius = rng.randf_range(3., 8.);
            buildings.push((center, radius));           
        }
        // Separation
        for i in 0..tries {
            let mut v = Vec2::xy(0., 0.);
            let mut neighbor_count = 0;
            let mut overlapping = false;
            let mut new_buildings = buildings.clone();
            for building in new_buildings.iter_mut() {
                for building_2 in buildings.iter() {
                    if building == building_2 {
                        continue
                    }
                    if building.0.dist(&building_2.0) < (building.1 + building_2.1) * 1.42 + 1. {
                        v.x += building_2.0.x - building.0.x;
                        v.y += building_2.0.y - building.0.y;
                        neighbor_count += 1;
                        overlapping = true;
                    }

                }
                if neighbor_count == 0 {
                    continue;
                }
                v.x = -v.x / neighbor_count as f32;
                v.y = -v.y / neighbor_count as f32;
                v = v.normalize(1.);
                building.0.x = (building.0.x + v.x).clamp(building.1, self.size.x() as f32 - building.1 - 1.);
                building.0.y = (building.0.y + v.y).clamp(building.1, self.size.y() as f32 - building.1 - 1.);
            }
            if !overlapping {
                break;
            }
            if i == tries - 1 {
            }
            buildings = new_buildings;
        }
        return buildings.iter().map(|building| {
            let size = Coord2::xy(
                rng.randu_range(6, building.1 as usize * 2) as i32,
                rng.randu_range(6, building.1 as usize * 2) as i32
            );
            let mut xy1 = Coord2::xy(building.0.x as i32, building.0.y as i32) - Coord2::xy(building.1 as i32, building.1 as i32);
            let mut xy2 = xy1 + size;
            xy1.x = xy1.x.clamp(0, self.size.x() as i32 - 1);
            xy1.y = xy1.y.clamp(0, self.size.y() as i32 - 1);
            xy2.x = xy2.x.clamp(0, self.size.x() as i32 - 1);
            xy2.y = xy2.y.clamp(0, self.size.y() as i32 - 1);
            return (xy1, xy2)
        }).collect()
    }

    pub fn make_building(&mut self, rng: &mut Rng, building: (Coord2, Coord2)) {
        for x in building.0.x..building.1.x + 1 {
            for y in building.0.y..building.1.y + 1 {
                let idx = (y * self.size.x() as i32) + x;
                self.map.tiles[idx as usize].id = 4; // Floor
                if x == building.0.x || y == building.0.y || x == building.1.x || y == building.1.y {
                    // Leaves 1 block out for doors
                    if y == building.1.y && x == building.0.x + (building.1.x-building.0.x) / 2 {
                        self.map.object_layer.set_tile(x as usize, y as usize, 0);
                        continue;
                    }
                    self.map.object_layer.set_tile(x as usize, y as usize, 1);
                } else {
                    self.map.object_layer.set_tile(x as usize, y as usize, 0);
                }
            }
        }

        // Bed
        if rng.rand_chance(0.7) {
            let x = rng.randu_range(building.0.x as usize + 1, building.1.x as usize - 2);
            self.map.object_layer.set_tile(x, building.0.y as usize + 1, 3);
        }

        // Table and stools
        if rng.rand_chance(0.6) {
            let x = rng.randu_range(building.0.x as usize + 2, building.1.x as usize - 3);
            let y = rng.randu_range(building.0.y as usize + 2, building.1.y as usize - 2);
            self.map.object_layer.set_tile(x, y, 4);
            self.map.object_layer.set_tile(x + 1, y, 5);
            self.map.object_layer.set_tile(x - 1, y, 5);
        }

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