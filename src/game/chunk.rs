use std::{collections::HashMap, iter};

use graphics::{image, Transformed};
use opengl_graphics::Texture;

use crate::{chunk_gen::chunk_generator::{ChunkGenParams, ChunkGenerator, ChunkLayer}, commons::{astar::MovementCost, id_vec::Id, resource_map::ResourceMap, rng::Rng}, engine::{assets::assets, audio::SoundEffect, geometry::{Coord2, Size2D}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, scene::BusEvent, tilemap::{TileMap, TileSet}, Color}, game::TurnController, resources::{resources::Resources, tile::{Tile, TileId}}, world::{creature::CreatureId, item::{Item, ItemId}, world::World}, GameContext};

use super::{actor::actor::Actor, factory::item_factory::ItemFactory, Renderable};

pub(crate) const PLAYER_IDX: usize = usize::MAX;

pub(crate) struct Chunk {
    pub(crate) world_coord: Coord2,
    pub(crate) size: Size2D,
    layer: ChunkLayer,
    pub(crate) map: ChunkMap,
    pub(crate) player: Actor,
    pub(crate) actors: Vec<Actor>,
    pub(crate) turn_controller: TurnController,
    pub(crate) ai_groups: AiGroups,
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
        let i = self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize);
        if i == 9 || i == 11 || i == 12 || i == 16 || i == 17 {
            return false
        }
        return true
    }

    pub(crate) fn check_line_of_sight(&self, from: &Coord2, to: &Coord2) -> bool {
        let angle_degrees = f64::atan2((to.y - from.y) as f64, (to.x - from.x) as f64);
        let dist = from.dist(to) as f64;
        let mut step = 0.;

        let mut pos = from.clone();
        let mut last = pos.clone();
        while step < dist {
            if pos != last {               
                if self.blocks_movement(pos) {
                    return false;
                }
                last = pos.clone();
            }
            step += 0.1;
            pos = Coord2::xy(
                from.x + (step * angle_degrees.cos()) as i32,
                from.y + (step * angle_degrees.sin()) as i32,
             );
         }
        return true;
    }

    // SMELL: This -1 +1 thing is prone to errors
    pub(crate) fn set_object_key(&mut self, pos: Coord2, tile: &str, resources: &Resources) {
        let id = resources.object_tiles.id_of(tile);
        let shadow = resources.object_tiles.get(&id).casts_shadow;
        self.object_layer.set_tile(pos.x as usize, pos.y as usize, id.as_usize() + 1);
        self.object_layer.set_shadow(pos.x as usize, pos.y as usize, shadow);
    }

    pub(crate) fn set_object_idx(&mut self, pos: Coord2, id: usize, resources: &Resources) {
        // SMELL
        let shadow;
        if id > 0 {
            shadow = resources.object_tiles.try_get(id - 1).unwrap().casts_shadow;
        } else {
            shadow = false;
        }
        self.object_layer.set_tile(pos.x as usize, pos.y as usize, id);
        self.object_layer.set_shadow(pos.x as usize, pos.y as usize, shadow);
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
    pub(crate) fn new(world_coord: Coord2, size: Size2D, layer: ChunkLayer, player: Actor, resources: &Resources) -> Chunk {

        let mut tileset = TileSet::new();
        for tile in resources.object_tiles.iter() {
            tileset.add(tile.tile.clone());    
        }

        let mut dual_tileset = LayeredDualgridTileset::new();
        for tile in resources.tiles.iter() {
            dual_tileset.add(tile.tile_layer, tile.tileset_image.clone());
        }

        Chunk {
            world_coord,
            size,
            layer,
            map: ChunkMap {
                tiles_clone: resources.tiles.clone(),
                ground_layer: LayeredDualgridTilemap::new(dual_tileset, size.x(), size.y(), 24, 24),
                object_layer: TileMap::new(tileset, size.x(), size.y(), 24, 24),
                items_on_ground: Vec::new(),
                tiles_metadata: HashMap::new(),
            },
            ai_groups: AiGroups::new(),
            player,
            actors: Vec::new(),
            turn_controller: TurnController::new()
        }
    }

    pub(crate) fn player(&self) -> &Actor {
        return &self.player
    }

    pub(crate) fn player_mut(&mut self) -> &mut Actor {
        return &mut self.player
    }

    pub(crate) fn is_player(&self, index: usize) -> bool {
        return index == PLAYER_IDX || index >= self.actors.len();
    }

    pub(crate) fn actor(&self, index: usize) -> Option<&Actor> {
        match index {
            PLAYER_IDX => Some(&self.player),
            i => {
                if i == self.actors.len() {
                    Some(&self.player)
                } else  {
                    self.actors.get(i)
                }
            },
        }
    }

    pub(crate) fn actor_mut(&mut self, index: usize) -> Option<&mut Actor> {
        match index {
            PLAYER_IDX => Some(&mut self.player),
            i => {
                if i == self.actors.len() {
                    Some(&mut self.player)
                } else  {
                    self.actors.get_mut(i)
                }
            },
        }
    }

    // TODO(QZ94ei4M): Remove
    pub(crate) fn actors_iter(&self) -> impl Iterator<Item = &Actor> {
        let player = iter::once(&self.player);
        let others = self.actors.iter();
        return others.chain(player);
    }

    // TODO(QZ94ei4M): Remove
    pub(crate) fn actors_iter_mut(&mut self) -> impl Iterator<Item = &mut Actor> {
        let player = iter::once(&mut self.player);
        let others = self.actors.iter_mut();
        others.chain(player)
    }

    pub(crate) fn playground(resources: &Resources, player: Actor, world: &World) -> Chunk {
        let mut chunk = Self::new(Coord2::xy(0,0), Size2D(128, 128), ChunkLayer::Surface, player, resources);
        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                chunk.map.ground_layer.set_tile(x, y, 1);
            }
        }

        let mut i = 0;
        for x in 40..80 {
            for y in 40..80 {
                i = i + 1;
                chunk.map.ground_layer.set_tile(x, y, i % 9);
            }
        }

        chunk.player_mut().xy = Coord2::xy(64, 64);

        let mut rng = Rng::seeded("items");
        for i in 0..60 {
            let point = Coord2::xy(60 + (i % 10), 68 + (i / 10));
            if i < 10 {
                let item = ItemFactory::weapon(&mut rng, &resources).make();
                let texture = item.make_texture(&resources);
                chunk.map.items_on_ground.push((point, item, texture));
            } else {
                let i = rng.randu_range(0, world.artifacts.len());
                let item = world.artifacts.get(&ItemId::new(i));
                let texture = item.make_texture(&resources);
                chunk.map.items_on_ground.push((point, item.clone(), texture));
            }         
        }

        return chunk
    }

    pub(crate) fn spawn(&mut self, actor: Actor) {
        self.actors.push(actor);
        self.turn_controller.initiative.push(self.actors.len());
    }

    pub(crate) fn remove_npc(&mut self, i: usize, ctx: &mut GameContext) {
        if i == PLAYER_IDX || i >= self.actors.len() {
            ctx.event_bus.push(BusEvent::PlayerDied);
            return;
        }

        let npc = self.actors.get_mut(i).unwrap();
        for item in npc.inventory.take_all() {
            let texture = item.make_texture(&ctx.resources);
            self.map.items_on_ground.push((npc.xy, item, texture));
        }

        if let Some(creature_id) = npc.creature_id {
            ctx.event_bus.push(BusEvent::CreatureKilled(creature_id));
        }

        self.actors.remove(i);
        self.turn_controller.remove(i);
    }

    pub(crate) fn from_world_tile(world: &World, resources: &Resources, xy: Coord2, layer: ChunkLayer, player: Actor) -> Chunk {
        let mut rng = Rng::seeded(xy);
        rng.next();
        // TODO: Size from params
        let mut chunk = Chunk::new(xy, Size2D(80, 80), layer, player, resources);
        let mut generator = ChunkGenerator::new(&mut chunk, rng);
        let params = ChunkGenParams {
            layer
        };
        generator.generate(&params, world, xy, resources);
        return chunk;
    }


    pub(crate) fn astar_movement_cost(&self, xy: Coord2) -> MovementCost {
        if !self.size.in_bounds(xy) || !self.can_occupy(&xy) {
            return MovementCost::Impossible;
        } else {
            return MovementCost::Cost(1.);
        }
    }

    pub(crate) fn can_occupy(&self, coord: &Coord2) -> bool {
        if self.map.blocks_movement(*coord) {
            return false;
        }
        return !self.actors_iter().any(|actor| actor.xy == *coord)
    }

}

impl Renderable for Chunk {
    fn render(&self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &mut GameContext) {
        self.map.ground_layer.render(ctx);

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
            ctx.texture(texture, ctx.at(pos.x as f64 * 24., pos.y as f64 * 24.));
        }

        if let ChunkLayer::Surface = self.layer {
            // Renders the nav borders
            {
                for y in 1..self.size.y()-2 {
                    ctx.image("gui/nav_arrow_left.png", [12, y as i32 * 24 + 12]);
                    ctx.image("gui/nav_arrow_right.png", [self.size.x() as i32 * 24 - 36, y as i32 * 24 + 12]);
                }
            }
            {
                for x in 1..self.size.x()-2 {
                    ctx.image("gui/nav_arrow_up.png", [x as i32 * 24 + 12, 12]);
                    ctx.image("gui/nav_arrow_down.png", [x as i32 * 24 + 12, self.size.y() as i32 * 24 - 36]);
                }
            }
            {
                let img = assets().image("gui/nav_corner.png");
                let transform = ctx.context.transform;
                image(&img.texture, transform.trans(12., 12.), ctx.gl);
                image(&img.texture, transform.trans(self.size.x() as f64 * 24. - 12., 12.).rot_deg(90.), ctx.gl);
                image(&img.texture, transform.trans(self.size.x() as f64 * 24. - 12., self.size.y() as f64 * 24. - 12.).rot_deg(180.), ctx.gl);
                image(&img.texture, transform.trans(12., self.size.y() as f64 * 24. - 12.).rot_deg(270.), ctx.gl);
            }
        }
        // Renders some black bars outside the map to cover large tiles
        {
            let color = Color::from_hex("090714");
            ctx.rectangle_fill([-64., -64., self.size.x() as f64 * 24. + 76., 76.], color);
            ctx.rectangle_fill([-64., self.size.y() as f64 * 24. - 12., self.size.x() as f64 * 24. + 76., 76.], color);
            ctx.rectangle_fill([-64., -64., 76., self.size.y() as f64 * 24. + 76.], color);
            ctx.rectangle_fill([self.size.x() as f64 * 24. - 12., -64., 76., self.size.y() as f64 * 24. + 76.], color);
        }

    }
}

pub(crate) struct AiGroups {
    next_group: u8,
    ai_group_mask: [u8; 8]   
}

impl AiGroups {

    pub(crate) fn new() -> Self {
        AiGroups { next_group: 1, ai_group_mask: [0;8] }
    }

    pub(crate) fn player() -> u8 {
        return 0;
    }

    pub(crate) fn next_group(&mut self) -> u8 {
        let r = self.next_group;
        self.next_group = self.next_group + 1;
        return r;
    }

    pub(crate) fn make_hostile(&mut self, group_a: u8, group_b: u8) {
        let ia = 0b0000_0001 << group_a;
        let ib = 0b0000_0001 << group_b;
        self.ai_group_mask[ia as usize] = self.ai_group_mask[ia as usize] | ib;
        self.ai_group_mask[ib as usize] = self.ai_group_mask[ib as usize] | ia;
    }

    pub(crate) fn is_hostile(&self, group_a: u8, group_b: u8) -> bool {
        let ia = 0b0000_0001 << group_a;
        let ib = 0b0000_0001 << group_b;
        return self.ai_group_mask[ia as usize] & ib > 0;
    }

}

#[cfg(test)]
mod tests_ai_groups {
    use super::*;

    #[test]
    fn test() {

        let mut ai = AiGroups::new();

        let group = ai.next_group();
        assert_eq!(ai.is_hostile(AiGroups::player(), group), false);
        ai.make_hostile(AiGroups::player(), group);
        assert_eq!(ai.is_hostile(AiGroups::player(), group), true);

        let group_2 = ai.next_group();
        assert_eq!(ai.is_hostile(AiGroups::player(), group_2), false);
        assert_eq!(ai.is_hostile(group, group_2), false);
        ai.make_hostile(AiGroups::player(), group_2);
        assert_eq!(ai.is_hostile(AiGroups::player(), group_2), true);
        assert_eq!(ai.is_hostile(group, group_2), false);

    }

}

