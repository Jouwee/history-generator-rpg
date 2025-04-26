use std::collections::HashMap;

use image::ImageReader;
use noise::{NoiseFn, Perlin};
use opengl_graphics::Texture;

use crate::{commons::{resource_map::ResourceMap, rng::Rng}, engine::{assets::{ImageParams, ImageRotate}, audio::SoundEffect, geometry::{Coord2, Size2D, Vec2}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, tilemap::{Tile16Subset, TileMap, TileSet, TileSingle}, Color}, resources::{resources::Resources, tile::{Tile, TileId}}, world::{creature::CreatureId, item::{Item, ItemMaker, ItemQuality}, world::World}, GameContext};

use super::{actor::Actor, Renderable};

pub(crate) struct Chunk {
    pub(crate) size: Size2D,
    pub(crate) map: ChunkMap,
    pub(crate) player: Actor,
    pub(crate) npcs: Vec<Actor>,
    pub(crate) killed_people: Vec<CreatureId>,
    // TODO: Should probably be on the map
    pub(crate) tiles_metadata: HashMap<Coord2, TileMetadata>,
    pub(crate) items_on_ground: Vec<(Coord2, Item, Texture)>,
}

pub(crate) enum TileMetadata {
    BurialPlace(CreatureId)
}

pub(crate) struct ChunkMap {
    tiles_clone: ResourceMap<TileId, Tile>,
    ground_layer: LayeredDualgridTilemap,
    object_layer: TileMap,
}

impl ChunkMap {

    pub(crate) fn blocks_movement(&self, pos: Coord2) -> bool {
        if let crate::engine::tilemap::Tile::Empty = self.object_layer.get_tile(pos.x as usize, pos.y as usize) {
            return false
        }
        return true
    }

    pub(crate) fn get_object_idx(&self, pos: Coord2) -> usize {
        return self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize)
    }

    pub(crate) fn remove_object(&mut self, pos: Coord2) {
        self.object_layer.set_tile(pos.x as usize, pos.y as usize, 0);
    }

    pub(crate) fn get_step_sound(&self, pos: Coord2) -> Option<SoundEffect> {
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
    pub(crate) fn new(size: Size2D, player: Actor, resources: &Resources) -> Chunk {

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
        let image = ImageReader::open("assets/sprites/chunk_tiles/tombstone.png").unwrap().decode().unwrap();
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
            tiles_metadata: HashMap::new(),
        }
    }

    pub(crate) fn playground(resources: &Resources, player: Actor) -> Chunk {
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

    pub(crate) fn from_world_tile(world: &World, resources: &Resources, xy: Coord2, player: Actor) -> Chunk {
        let mut chunk = Self::new(Size2D(128, 128), player, resources);
        let mut rng = Rng::seeded(world.generation_params.seed).derive("chunk").derive(xy);
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



        // Roads
        if world.map_features.has_road(xy) {
            let road_north = world.map_features.has_road(xy + Coord2::xy(0, -1));
            let road_south = world.map_features.has_road(xy + Coord2::xy(0, 1));
            let road_east = world.map_features.has_road(xy + Coord2::xy(1, 0));
            let road_west = world.map_features.has_road(xy + Coord2::xy(-1, 0));

            let start_north = (Coord2::xy(chunk.size.x() as i32 / 2, 0), Vec2::xy(0., 1.));
            let start_south = (Coord2::xy(chunk.size.x() as i32 / 2, chunk.size.y() as i32 - 1), Vec2::xy(0., -1.));
            let start_east = (Coord2::xy(chunk.size.x() as i32 - 1, chunk.size.y() as i32 / 2), Vec2::xy(-1., 0.));
            let start_west = (Coord2::xy(0, chunk.size.y() as i32 / 2), Vec2::xy(1., 0.));
            let center = Coord2::xy((chunk.size.x() as f64 / 2.) as i32, (chunk.size.y() as f64 / 2.) as i32);

            let vec = match (road_north, road_south, road_east, road_west) {
                // 2 conections
                (true, true, false, false) => vec!((start_north.0, start_south.0, start_north.1)),
                (false, false, true, true) => vec!((start_east.0, start_west.0, start_east.1)),
                (true, false, true, false) => vec!((start_north.0, start_east.0, start_north.1)),
                (true, false, false, true) => vec!((start_north.0, start_west.0, start_north.1)),
                (false, true, true, false) => vec!((start_south.0, start_east.0, start_south.1)),
                (false, true, false, true) => vec!((start_south.0, start_west.0, start_south.1)),
                // 3 or 4 connections
                _ => {
                    let mut vec = Vec::new();
                    if road_north {
                        vec.push((start_north.0, center, start_north.1));
                    }
                    if road_south {
                        vec.push((start_south.0, center, start_south.1));
                    }
                    if road_east {
                        vec.push((start_east.0, center, start_east.1));
                    }
                    if road_west {
                        vec.push((start_west.0, center, start_west.1));
                    }
                    vec
                }
            };

            for (start, target, velocity) in vec.iter() {
                let mut position = start.to_vec2();
                let target = target.to_vec2();
                let mut velocity = velocity.clone();

                let mut points = Vec::new();
                points.push(position);

                let max_speed: f32 = 1.;
                let max_force = 0.1;

                while position.dist_squared(&target) > (max_speed).powi(2) {
                    position = position + velocity;

                    let desired_velocity = (target - position).normalize(max_speed);
                    let random_velocity = Vec2::xy(rng.randf_range(-0.3, 0.3), rng.randf_range(-0.3, 0.3));
                    let steering = (desired_velocity - velocity) + random_velocity;
                    let steering = steering.truncate(max_force);
                    // steering = steering / mass
                    velocity = (velocity + steering).truncate(max_speed);

                    // Cobblestone
                    let coord = Coord2::xy(position.x.round() as i32, position.y.round() as i32);
                    for c in coord.neighbours_circle(chunk.size, 3).iter() {
                        if rng.rand_chance(0.5) {
                            chunk.map.ground_layer.set_tile(c.x as usize, c.y as usize, 5);
                        }
                        chunk.map.object_layer.set_tile(c.x as usize, c.y as usize, 0);
                    }
                }


            }
            
        }

        let mut found_sett = None;
        for unit in world.units.iter() {
            let unit = unit.borrow();
            if unit.xy.x as i32 == xy.x && unit.xy.y as i32 == xy.y {
                // TODO:
                // let num_builds = (unit.demographics.population / 20).clamp(1, 9) as usize;
                // let buildings = chunk.prepare_buildings(&mut rng, num_builds, 100);
                // for building in buildings {
                //     chunk.make_building(&mut rng, building);
                // }

                let mut x = 20;
                let mut y = 20;

                // TODO: can't handle more
                let mut slice = &unit.creatures[..];
                if slice.len() > 1000 {
                    slice = &unit.creatures[0..1000];
                }
                for creature_id in slice.iter() {
                    let creature = world.get_creature(creature_id);
                    // TODO:
                    // let point = chunk.get_spawn_pos(&mut rng);
                    let point = Coord2::xy(x, y);
                    let species = resources.species.get(&creature.species);
                    chunk.npcs.push(Actor::from_creature(point, *creature_id, &creature, &creature.species, &species, world));
                    x = x + 3;
                    if x > 100 {
                        y = y + 3;
                        x = 20;
                    }
                }

                let mut x = 10;
                let mut y = 20;

                for item in unit.artifacts.iter() {


                    let item = world.artifacts.get(item);
                    let texture = item.make_texture(&resources.materials);
                    chunk.items_on_ground.push((Coord2::xy(x, y), item.clone(), texture));

                    x = x + 2;
                    if x > 18 {
                        y = y + 2;
                        x = 10;
                    }
                }

                let mut x = 3;
                let mut y = 30;

                let mut slice = &unit.cemetery[..];
                if slice.len() > 700 {
                    slice = &unit.cemetery[0..700];
                }

                for creature in slice.iter() {
                    chunk.map.object_layer.set_tile(x as usize, y as usize, 6);
                    chunk.tiles_metadata.insert(Coord2::xy(x, y), TileMetadata::BurialPlace(*creature));

                    x = x + 1;
                    if x > 18 {
                        y = y + 2;
                        x = 3;
                    }
                }


                
                found_sett = Some(unit);

            }
        }


        // for creature in world.creatures.iter() {
        //     let creature = creature.borrow();
            // TODO:
            // if creature.position == xy {
            //     let point = chunk.get_spawn_pos(&mut rng);
            //     let species = resources.species.get(&creature.species);
            //     chunk.npcs.push(Actor::from_creature(point, *id, &creature, &creature.species, &species, world));
            // }
        // }

        if let Some(_unit) = found_sett {
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

    pub(crate) fn get_spawn_pos(&self, rng: &mut Rng) -> Coord2 {
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

    pub(crate) fn prepare_buildings(&self, rng: &mut Rng, num_buildings: usize, tries: u32) -> Vec<(Coord2, Coord2)> {
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

    pub(crate) fn make_building(&mut self, rng: &mut Rng, building: (Coord2, Coord2)) {
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
        let cull_start = [
            (ctx.camera_rect[0] / 24. as f64 - 1.).max(0.) as i32,
            (ctx.camera_rect[1] / 24. as f64 - 1.).max(0.) as i32
        ];
        let cull_limit = [
            1 + cull_start[0] + ctx.camera_rect[2] as i32 / 24,
            1 + cull_start[1] + ctx.camera_rect[3] as i32 / 24
        ];
        for npc in self.npcs.iter() {
            if npc.xy.x < cull_start[0] || npc.xy.y < cull_start[1] || npc.xy.x > cull_limit[0] || npc.xy.y > cull_limit[1] {
                continue
            }
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
            let left = game_ctx.assets.image(ImageParams::new("gui/nav_arrow_left.png"));
            for y in 1..self.size.y()-1 {
                ctx.texture_ref(&left.texture, [12., y as f64 * 24. + 12.]);
            }
        }
        {
            let right = game_ctx.assets.image(ImageParams::new("gui/nav_arrow_right.png"));
            for y in 1..self.size.y()-1 {
                ctx.texture_ref(&right.texture, [self.size.x() as f64 * 24. - 12., y as f64 * 24. + 12.]);
            }
        }
        {
            let up = game_ctx.assets.image(ImageParams::new("gui/nav_arrow_up.png"));
            for x in 1..self.size.x()-1 {
                ctx.texture_ref(&up.texture, [x as f64 * 24. + 12., 12.]);
            }
        }
        {
            let down = game_ctx.assets.image(ImageParams::new("gui/nav_arrow_down.png"));
            for x in 1..self.size.x()-1 {
                ctx.texture_ref(&down.texture, [x as f64 * 24. + 12., self.size.y() as f64 * 24. - 12.]);
            }
        }
        {
            let corner = game_ctx.assets.image(ImageParams::new("gui/nav_corner.png"));
            ctx.texture_ref(&corner.texture, [12., 12.]);
            let corner = game_ctx.assets.image(ImageParams::new("gui/nav_corner.png").rotate(ImageRotate::R90));
            ctx.texture_ref(&corner.texture, [self.size.x() as f64 * 24. - 12., 12.]);
            let corner = game_ctx.assets.image(ImageParams::new("gui/nav_corner.png").rotate(ImageRotate::R180));
            ctx.texture_ref(&corner.texture, [self.size.x() as f64 * 24. - 12., self.size.y() as f64 * 24. - 12.]);
            let corner = game_ctx.assets.image(ImageParams::new("gui/nav_corner.png").rotate(ImageRotate::R270));
            ctx.texture_ref(&corner.texture, [12., self.size.y() as f64 * 24. - 12.]);
        }
        // Renders some black bars outside the map to cover large tiles
        {
            ctx.rectangle_fill([-64., -64., self.size.x() as f64 * 24. + 76., 76.], Color::from_hex("090714"));
            ctx.rectangle_fill([-64., self.size.y() as f64 * 24., self.size.x() as f64 * 24. + 76., 76.], Color::from_hex("090714"));
            ctx.rectangle_fill([-64., -64., 64., self.size.y() as f64 * 24. + 76.], Color::from_hex("090714"));
            ctx.rectangle_fill([self.size.x() as f64 * 24., -64., 64., self.size.y() as f64 * 24. + 76.], Color::from_hex("090714"));
        }
    }
}