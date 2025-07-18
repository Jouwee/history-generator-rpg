use std::{collections::HashMap, iter};

use opengl_graphics::Texture;

use crate::{chunk_gen::chunk_generator::ChunkGenerator, commons::{astar::MovementCost, id_vec::Id, resource_map::ResourceMap, rng::Rng}, engine::{asset::image::{ImageAsset, ImageRotate}, audio::SoundEffect, geometry::{Coord2, Size2D}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, tilemap::{TileMap, TileSet}, Color}, resources::{resources::Resources, tile::{Tile, TileId}}, world::{creature::CreatureId, item::{Item, ItemId}, world::World}, GameContext};

use super::{actor::actor::Actor, factory::item_factory::ItemFactory, Renderable};

pub(crate) const PLAYER_IDX: usize = usize::MAX;

pub(crate) struct Chunk {
    pub(crate) size: Size2D,
    pub(crate) map: ChunkMap,
    pub(crate) player: Actor,
    pub(crate) actors: Vec<Actor>,
}

#[derive(Clone)]
pub(crate) enum TileMetadata {
    BurialPlace(CreatureId)
}

pub(crate) struct ChunkMap {
    pub(crate) tiles_metadata: HashMap<Coord2, TileMetadata>,
    pub(crate) items_on_ground: Vec<(Coord2, Item, Texture)>,
    pub(crate) tiles_clone: ResourceMap<TileId, Tile>,
    pub(crate) ground_layer: LayeredDualgridTilemap,
    pub(crate) object_layer: TileMap,
}

impl ChunkMap {

    pub(crate) fn blocks_movement(&self, pos: Coord2) -> bool {
        if let crate::engine::tilemap::Tile::Empty = self.object_layer.get_tile(pos.x as usize, pos.y as usize) {
            return false
        }
        // TODO: Resources
        if self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize) == 9 || self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize) == 11 || self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize) == 12 {
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
        for tile in resources.object_tiles.iter() {
            tileset.add(tile.tile.clone());    
        }

        let mut dual_tileset = LayeredDualgridTileset::new();
        for tile in resources.tiles.iter() {
            dual_tileset.add(tile.tile_layer, tile.tileset_image.clone());
        }

        Chunk {
            size,
            map: ChunkMap {
                tiles_clone: resources.tiles.clone(),
                ground_layer: LayeredDualgridTilemap::new(dual_tileset, size.x(), size.y(), 24, 24),
                object_layer: TileMap::new(tileset, size.x(), size.y(), 24, 24),
                items_on_ground: Vec::new(),
                tiles_metadata: HashMap::new(),
            },
            player,
            actors: Vec::new(),
        }
    }

    pub(crate) fn player(&self) -> &Actor {
        return &self.player
    }

    pub(crate) fn player_mut(&mut self) -> &mut Actor {
        return &mut self.player
    }

    pub(crate) fn actor(&self, index: usize) -> Option<&Actor> {
        match index {
            PLAYER_IDX => Some(&self.player),
            i => self.actors.get(i),
        }
    }

    pub(crate) fn actor_mut(&mut self, index: usize) -> Option<&mut Actor> {
        match index {
            PLAYER_IDX => Some(&mut self.player),
            i => self.actors.get_mut(i),
        }
    }

    // TODO(QZ94ei4M): Remove
    pub(crate) fn actors_iter(&self) -> impl Iterator<Item = &Actor> {
        let player = iter::once(&self.player);
        let others = self.actors.iter();
        others.chain(player)
    }

    // TODO(QZ94ei4M): Remove
    pub(crate) fn actors_iter_mut(&mut self) -> impl Iterator<Item = &mut Actor> {
        let player = iter::once(&mut self.player);
        let others = self.actors.iter_mut();
        others.chain(player)
    }

    pub(crate) fn playground(resources: &Resources, player: Actor, world: &World) -> Chunk {
        let mut chunk = Self::new(Size2D(128, 128), player, resources);
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                chunk.map.ground_layer.set_tile(x, y, 1);
            }
        }

        chunk.player_mut().xy = Coord2::xy(64, 64);

        let species_id = &resources.species.id_of("species:varningr");
        let species = resources.species.get(species_id);
        let npc = Actor::from_species(Coord2::xy(64, 50), &resources.species.id_of("species:varningr"), species);
        chunk.actors.push(npc);

        let mut rng = Rng::seeded("items");
        for i in 0..60 {
            let point = Coord2::xy(60 + (i % 10), 68 + (i / 10));
            if i < 10 {
                let item = ItemFactory::weapon(&mut rng, &resources).make();
                let texture = item.make_texture(&resources.materials);
                chunk.map.items_on_ground.push((point, item, texture));
            } else {
                let i = rng.randu_range(0, world.artifacts.len());
                let item = world.artifacts.get(&ItemId::new(i));
                let texture = item.make_texture(&resources.materials);
                chunk.map.items_on_ground.push((point, item.clone(), texture));
            }         
        }

        return chunk
    }

    pub(crate) fn from_world_tile(world: &World, resources: &Resources, xy: Coord2, player: Actor) -> Chunk {
        let mut rng = Rng::seeded(xy);
        rng.next();
        // TODO: Size from params
        let mut generator = ChunkGenerator::new(resources, player, Size2D(128, 128), rng);
        generator.generate(world, xy, resources);
        return generator.into_chunk();
    }


    pub(crate) fn astar_movement_cost(&self, xy: Coord2) -> MovementCost {
        if !self.size.in_bounds(xy) || self.map.blocks_movement(xy) {
            return MovementCost::Impossible;
        } else {
            return MovementCost::Cost(1.);
        }
    }

}

impl Renderable for Chunk {
    fn render(&self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &mut GameContext) {
        self.map.ground_layer.render(ctx, game_ctx);

        let mut actors_by_position = HashMap::new();
        actors_by_position.insert(&self.player().xy, vec!(self.player()));
        let cull_start = [
            (ctx.camera_rect[0] / 24. as f64 - 1.).max(0.) as i32,
            (ctx.camera_rect[1] / 24. as f64 - 1.).max(0.) as i32
        ];
        let cull_limit = [
            1 + cull_start[0] + ctx.camera_rect[2] as i32 / 24,
            1 + cull_start[1] + ctx.camera_rect[3] as i32 / 24
        ];
        for npc in self.actors.iter() {
            if npc.xy.x < cull_start[0] || npc.xy.y < cull_start[1] || npc.xy.x > cull_limit[0] || npc.xy.y > cull_limit[1] {
                continue
            }
            if !actors_by_position.contains_key(&npc.xy) {
                actors_by_position.insert(&npc.xy, Vec::new());
            }
            actors_by_position.get_mut(&npc.xy).unwrap().push(npc);
        }

        self.map.object_layer.render(ctx, game_ctx, |ctx, game_ctx, x, y| {
            if let Some(actors) = actors_by_position.get(&Coord2::xy(x as i32, y as i32)) {
                for actor in actors {
                    actor.render(ctx, game_ctx);
                }
            }
        });

        for (pos, _item, texture) in self.map.items_on_ground.iter() {
            ctx.texture_ref(texture, [pos.x as f64 * 24., pos.y as f64 * 24.]);
        }
        // Renders the nav borders
        {
            for y in 1..self.size.y()-1 {
                ctx.image(&ImageAsset::new("gui/nav_arrow_left.png"), [12, y as i32 * 24 + 12], &mut game_ctx.assets);
            }
        }
        {
            for y in 1..self.size.y()-1 {
                ctx.image(&ImageAsset::new("gui/nav_arrow_right.png"), [self.size.x() as i32 * 24 - 12, y as i32 * 24 + 12], &mut game_ctx.assets);
            }
        }
        {
            for x in 1..self.size.x()-1 {
                ctx.image(&ImageAsset::new("gui/nav_arrow_up.png"), [x as i32 * 24 - 12, 12], &mut game_ctx.assets);
            }
        }
        {
            for x in 1..self.size.x()-1 {
                ctx.image(&ImageAsset::new("gui/nav_arrow_down.png"), [x as i32 * 24 - 12, self.size.y() as i32 * 24 - 12], &mut game_ctx.assets);
            }
        }
        {
            ctx.image(&ImageAsset::new("gui/nav_corner.png"), [12, 12], &mut game_ctx.assets);
            ctx.image(&ImageAsset::new("gui/nav_corner.png").rotate(ImageRotate::R90), [self.size.x() as i32 * 24 - 12, 12], &mut game_ctx.assets);
            ctx.image(&ImageAsset::new("gui/nav_corner.png").rotate(ImageRotate::R180), [self.size.x() as i32 * 24 - 12, self.size.y() as i32 * 24 - 12], &mut game_ctx.assets);
            ctx.image(&ImageAsset::new("gui/nav_corner.png").rotate(ImageRotate::R270), [12, self.size.y() as i32 * 24 - 12], &mut game_ctx.assets);
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