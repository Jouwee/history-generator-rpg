use std::collections::HashMap;

use opengl_graphics::Texture;

use crate::{chunk_gen::chunk_generator::ChunkGenerator, commons::{resource_map::ResourceMap, rng::Rng}, engine::{assets::{ImageAsset, ImageRotate}, audio::SoundEffect, geometry::{Coord2, Size2D}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, tilemap::{TileMap, TileSet}, Color}, resources::{resources::Resources, tile::{Tile, TileId}}, world::{creature::CreatureId, item::{Item, ItemMaker, ItemQuality}, world::World}, GameContext};

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
        if self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize) == 9 {
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
        let mut rng = Rng::seeded(xy);
        rng.next();
        // TODO: Size from params
        let mut generator = ChunkGenerator::new(resources, player, Size2D(128, 128), rng);
        generator.generate(world, xy, resources);
        return generator.into_chunk();
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

        self.map.object_layer.render(ctx, game_ctx, |ctx, game_ctx, x, y| {
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
            let left = game_ctx.assets.image(&ImageAsset::new("gui/nav_arrow_left.png"));
            for y in 1..self.size.y()-1 {
                ctx.texture_ref(&left.texture, [12., y as f64 * 24. + 12.]);
            }
        }
        {
            let right = game_ctx.assets.image(&ImageAsset::new("gui/nav_arrow_right.png"));
            for y in 1..self.size.y()-1 {
                ctx.texture_ref(&right.texture, [self.size.x() as f64 * 24. - 12., y as f64 * 24. + 12.]);
            }
        }
        {
            let up = game_ctx.assets.image(&ImageAsset::new("gui/nav_arrow_up.png"));
            for x in 1..self.size.x()-1 {
                ctx.texture_ref(&up.texture, [x as f64 * 24. + 12., 12.]);
            }
        }
        {
            let down = game_ctx.assets.image(&ImageAsset::new("gui/nav_arrow_down.png"));
            for x in 1..self.size.x()-1 {
                ctx.texture_ref(&down.texture, [x as f64 * 24. + 12., self.size.y() as f64 * 24. - 12.]);
            }
        }
        {
            let corner = game_ctx.assets.image(&ImageAsset::new("gui/nav_corner.png"));
            ctx.texture_ref(&corner.texture, [12., 12.]);
            let corner = game_ctx.assets.image(&ImageAsset::new("gui/nav_corner.png").rotate(ImageRotate::R90));
            ctx.texture_ref(&corner.texture, [self.size.x() as f64 * 24. - 12., 12.]);
            let corner = game_ctx.assets.image(&ImageAsset::new("gui/nav_corner.png").rotate(ImageRotate::R180));
            ctx.texture_ref(&corner.texture, [self.size.x() as f64 * 24. - 12., self.size.y() as f64 * 24. - 12.]);
            let corner = game_ctx.assets.image(&ImageAsset::new("gui/nav_corner.png").rotate(ImageRotate::R270));
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