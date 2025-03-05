use std::collections::HashMap;

use image::ImageReader;
use noise::{NoiseFn, Perlin};
use opengl_graphics::Texture;

use crate::{commons::{history_vec::Id, resource_map::ResourceMap, rng::Rng}, engine::{audio::SoundEffect, geometry::{Coord2, Size2D, Vec2}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, tilemap::{Tile16Subset, TileMap, TileSet, TileSingle}}, resources::{resources::Resources, tile::{Tile, TileId}}, world::item::{Item, ItemMaker, ItemQuality}, GameContext, World};

use super::{actor::Actor, Renderable};

pub struct Chunk {
    pub size: Size2D,
    pub map: ChunkMap,
    pub player: Actor,
    pub npcs: Vec<Actor>,
    pub killed_people: Vec<Id>,
    pub items_on_ground: Vec<(Coord2, Item, Texture)>,
}

pub struct ChunkMap {
    tiles_clone: ResourceMap<TileId, Tile>,
    ground_layer: LayeredDualgridTilemap,
    object_layer: TileMap,
}

impl ChunkMap {

    pub fn blocks_movement(&self, pos: Coord2) -> bool {
        if let crate::engine::tilemap::Tile::Empty = self.object_layer.get_tile(pos.x as usize, pos.y as usize) {
            return false
        }
        return true
    }

    pub fn get_object_idx(&self, pos: Coord2) -> usize {
        return self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize)
    }

    pub fn get_step_sound(&self, pos: Coord2) -> Option<SoundEffect> {
        if let Some(tile) = self.ground_layer.tile(pos.x as usize, pos.y as usize) {
            let tile = self.tiles_clone.try_get(tile);
            if let Some(tile) = tile {
                return tile.step_sound_effect.clone()
            }
        }
        None
    }

}

impl Chunk {
    pub fn new(size: Size2D, player: Actor, resources: &Resources) -> Chunk {

        let mut tileset = TileSet::new();
        let image = ImageReader::open("assets/sprites/chunk_tiles/stone_walls.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image, 24, 48)));
        let image = ImageReader::open("assets/sprites/tree.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageReader::open("assets/sprites/bed.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageReader::open("assets/sprites/table.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageReader::open("assets/sprites/stool.png").unwrap().decode().unwrap();
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));

        let mut dual_tileset = LayeredDualgridTileset::new();
        for tile in resources.tiles.iter() {
            dual_tileset.add(tile.tile_layer, tile.tileset_image.clone(), 24, 24);
        }

        Chunk {
            size,
            map: ChunkMap {
                tiles_clone: resources.tiles.clone(),
                ground_layer: LayeredDualgridTilemap::new(dual_tileset, size.x(), size.y(), 24, 24),
                object_layer: TileMap::new(tileset, size.x(), size.y(), 24, 24),
            },
            player,
            npcs: Vec::new(),
            killed_people: Vec::new(),
            items_on_ground: Vec::new(),
        }
    }

    pub fn playground(resources: &Resources, player: Actor) -> Chunk {
        let mut chunk = Self::new(Size2D(128, 128), player, resources);
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                chunk.map.ground_layer.set_tile(x, y, 1);
            }
        }

        // Bed
        chunk.map.object_layer.set_tile(36, 34, 3);

        let species_id = &resources.species.id_of("species:spider");
        let species = resources.species.get(species_id);
        let npc = Actor::from_species(Coord2::xy(26, 26), &resources.species.id_of("species:spider"), species);
        chunk.npcs.push(npc);

        let point = Coord2::xy(34, 34);
        let item = ItemMaker::random(&Rng::seeded("a"), &resources.materials, ItemQuality::Normal);
        let texture = item.make_texture(&resources.materials);
        chunk.items_on_ground.push((point, item, texture));

        let point = Coord2::xy(35, 34);
        let item = ItemMaker::random(&Rng::seeded("c"), &resources.materials, ItemQuality::Normal);
        let texture = item.make_texture(&resources.materials);
        chunk.items_on_ground.push((point, item, texture));

        // Puts a spider in a small labyrinth
        let species_id = &resources.species.id_of("species:spider");
        let species = resources.species.get(species_id);
        let npc = Actor::from_species(Coord2::xy(26, 40), &resources.species.id_of("species:spider"), species);
        chunk.npcs.push(npc);

        chunk.map.object_layer.set_tile(24,37, 1);
        chunk.map.object_layer.set_tile(25,37, 1);
        chunk.map.object_layer.set_tile(26,37, 1);
        chunk.map.object_layer.set_tile(28,37, 1);
        chunk.map.object_layer.set_tile(23,38, 1);
        chunk.map.object_layer.set_tile(29,38, 1);
        chunk.map.object_layer.set_tile(23,39, 1);
        chunk.map.object_layer.set_tile(25,39, 1);
        chunk.map.object_layer.set_tile(26,39, 1);
        chunk.map.object_layer.set_tile(27,39, 1);
        chunk.map.object_layer.set_tile(29,39, 1);
        chunk.map.object_layer.set_tile(23,40, 1);
        chunk.map.object_layer.set_tile(25,40, 1);
        chunk.map.object_layer.set_tile(28,40, 1);
        chunk.map.object_layer.set_tile(22,41, 1);
        chunk.map.object_layer.set_tile(26,41, 1);
        chunk.map.object_layer.set_tile(28,41, 1);
        chunk.map.object_layer.set_tile(22,42, 1);
        chunk.map.object_layer.set_tile(24,42, 1);
        chunk.map.object_layer.set_tile(25,43, 1);
        chunk.map.object_layer.set_tile(26,43, 1);
        chunk.map.object_layer.set_tile(27,43, 1);
        chunk.map.object_layer.set_tile(28,43, 1);
        chunk.map.object_layer.set_tile(29,43, 1);

        return chunk
    }

    pub fn from_world_tile(world: &World, resources: &Resources, xy: Coord2, player: Actor) -> Chunk {
        let mut chunk = Self::new(Size2D(128, 128), player, resources);
        let mut rng = Rng::rand();
        let tile = world.map.tile(xy.x as usize, xy.y as usize);
        let noise = Perlin::new(rng.derive("terrain").seed());
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                let n = noise.get([x as f64 / 10.0, y as f64 / 10.0]);
                match tile.region_id {
                    0 => { // Ocean
                        chunk.map.ground_layer.set_tile(x, y, 3);
                        // water
                    },
                    1 => { // Coastal
                        if n < -0.5 {
                            chunk.map.ground_layer.set_tile(x, y, 3);
                            // water
                        } else {
                            chunk.map.ground_layer.set_tile(x, y, 2);
                            // sand
                        }
                    },
                    2 => { // Grassland
                        if n < 0.5 {
                            chunk.map.ground_layer.set_tile(x, y, 1);
                            // grass
                        } else {
                            chunk.map.ground_layer.set_tile(x, y, 0);
                            // stone
                        }
                    },
                    3 => { // Forest
                        if n < 0.5 {
                            chunk.map.ground_layer.set_tile(x, y, 1);
                            // grass
                        } else {
                            chunk.map.ground_layer.set_tile(x, y, 0);
                            // stone
                        }
                    },
                    4 => { // Desert - Sand
                        if n < 0.5 {
                            chunk.map.ground_layer.set_tile(x, y, 2);
                            // sand
                        } else {
                            chunk.map.ground_layer.set_tile(x, y, 0);
                            // stone
                        }
                    },
                    _ => ()
                }
            }
        }

        let noise = Perlin::new(rng.derive("trees").seed());
        for x in 1..chunk.size.x()-1 {
            for y in 1..chunk.size.y()-1 {
                if noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0. {
                    if let Some(ground) = chunk.map.ground_layer.tile(x, y) {
                        if ground == 1 && rng.rand_chance(0.1) {
                            chunk.map.object_layer.set_tile(x as usize, y as usize, 2);
                        }
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
                let species = resources.species.get(&person.species);
                chunk.npcs.push(Actor::from_person(point, *id, &person, &person.species, &species, world));
            }
        }

        if let Some(_settlement) = found_sett {
            if chunk.npcs.len() == 0 {
                for _ in 0..rng.randu_range(3, 7) {
                    let point = chunk.get_spawn_pos(&mut rng);
                    let species_id = resources.species.id_of("species:spider");
                    let species = resources.species.get(&species_id);
                    let npc = Actor::from_species(point, &species_id, species);
                    chunk.npcs.push(npc);
                }

                let point = Coord2::xy(
                    rng.randu_range(0, chunk.size.x()) as i32,
                    rng.randu_range(0, chunk.size.y()) as i32
                );
                let item = ItemMaker::random(&rng, &resources.materials, ItemQuality::Normal);
                let texture = item.make_texture(&resources.materials);
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
                // Floor
                self.map.ground_layer.set_tile(x as usize, y as usize, 4);
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
    fn render(&self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &mut GameContext) {
        self.map.ground_layer.render(ctx);

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
                    actor.render(ctx, game_ctx);
                }
            }
        });

        for (pos, _item, texture) in self.items_on_ground.iter() {
            ctx.texture_ref(texture, [pos.x as f64 * 24., pos.y as f64 * 24.]);
        }
        // Renders the nav borders
        {
            let left = game_ctx.assets.image("gui/nav_arrow_left.png");
            for y in 0..self.size.y() {
                ctx.texture_ref(&left.texture, [12., y as f64 * 24. + 12.]);
            }
        }
        {
            let right = game_ctx.assets.image("gui/nav_arrow_right.png");
            for y in 0..self.size.y() {
                ctx.texture_ref(&right.texture, [self.size.x() as f64 * 24. - 12., y as f64 * 24. + 12.]);
            }
        }
        {
            let up = game_ctx.assets.image("gui/nav_arrow_up.png");
            for x in 1..self.size.x()-1 {
                ctx.texture_ref(&up.texture, [x as f64 * 24. + 12., 12.]);
            }
        }
        {
            let down = game_ctx.assets.image("gui/nav_arrow_down.png");
            for x in 1..self.size.x()-1 {
                ctx.texture_ref(&down.texture, [x as f64 * 24. + 12., self.size.y() as f64 * 24. - 12.]);
            }
        }
    }
}