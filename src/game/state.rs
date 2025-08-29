use std::{collections::HashMap, iter};

use graphics::{image, Transformed};
use math::Vec2i;
use serde::{Deserialize, Serialize};

use crate::{chunk_gen::chunk_generator::ChunkGenerator, commons::{astar::MovementCost, id_vec::Id, rng::Rng}, engine::{assets::assets, geometry::{Coord2, Size2D}, scene::BusEvent, Color}, game::{actor::actor::Actor, chunk::{Chunk, ChunkCoord, ChunkLayer, Spawner}, factory::item_factory::ItemFactory, Renderable}, loadsave::SaveFile, resources::resources::{resources, Resources}, world::{item::ItemId, site::SiteType, world::World}, GameContext};

pub(crate) const PLAYER_IDX: usize = usize::MAX;

#[derive(Serialize, Deserialize)]
/// Game state, passed to several functions, and is saved/loaded
pub(crate) struct GameState {
    pub(crate) coord: ChunkCoord,
    /// The chunk will be serialized separately
    #[serde(skip)]
    pub(crate) chunk: Chunk,
    pub(crate) player: Actor,
    pub(crate) actors: Vec<Actor>,
    pub(crate) turn_controller: TurnController,
    pub(crate) ai_groups: AiGroups,
}

impl GameState {
    pub(crate) fn new(coord: ChunkCoord, size: Size2D, player: Actor, resources: &Resources) -> GameState {
        GameState {
            coord,
            chunk: Chunk::new(coord, size, resources),
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

    pub(crate) fn playground(resources: &Resources, player: Actor, world: &World) -> GameState {
        let mut chunk = Self::new(ChunkCoord::new(Vec2i(0,0), ChunkLayer::Surface), Size2D(128, 128), player, resources);
        for x in 0..chunk.chunk.size.x() {
            for y in 0..chunk.chunk.size.y() {
                chunk.chunk.ground_layer.set_tile(x, y, 1);
            }
        }

        let mut i = 0;
        for x in 40..80 {
            for y in 40..80 {
                i = i + 1;
                chunk.chunk.ground_layer.set_tile(x, y, i % 9);
            }
        }

        chunk.player_mut().xy = Vec2i(64, 64);

        let mut rng = Rng::seeded("items");
        for i in 0..60 {
            let point = Coord2::xy(60 + (i % 10), 68 + (i / 10));
            if i < 10 {
                let item = ItemFactory::weapon(&mut rng, &resources).make();
                let texture = item.make_texture(&resources);
                chunk.chunk.items_on_ground.push((point, item, texture));
            } else {
                let i = rng.randu_range(0, world.artifacts.len());
                let item = world.artifacts.get(&ItemId::new(i));
                let texture = item.make_texture(&resources);
                chunk.chunk.items_on_ground.push((point, item.clone(), texture));
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
            self.chunk.items_on_ground.push((npc.xy.into(), item, texture));
        }

        if let Some(creature_id) = npc.creature_id {
            ctx.event_bus.push(BusEvent::CreatureKilled(creature_id));
        }

        self.actors.remove(i);
        self.turn_controller.remove(i);
    }

    pub(crate) fn from_world_tile(world: &World, save_file: &SaveFile, resources: &Resources, coord: ChunkCoord, player: Actor) -> GameState {
        let mut rng = Rng::seeded(coord.xy);
        rng.next();
        // TODO: Size from params
        let mut state = GameState::new(coord, Size2D(80, 80), player, resources);
        state.load_or_generate_chunk(state.coord, save_file, world);
        save_file.save_game_state(&state).unwrap();
        return state;
    }

    pub(crate) fn switch_chunk(&mut self, coord: ChunkCoord, save_file: &SaveFile, world: &World) {
        let offset = coord.xy - self.coord.xy;
        let change_layer = coord.layer != self.coord.layer;
        // Saves the chunk
        save_file.save_chunk(&self.chunk).unwrap();
        // Resets the state
        self.coord = coord;
        self.actors.clear();
        self.ai_groups = AiGroups::new();
        self.turn_controller = TurnController::new();

        self.load_or_generate_chunk(coord, save_file, world);

        // Reposition player
        if offset.x() < 0 {
            self.player_mut().xy.0 = self.chunk.size.x() as i32 - 3;
        }
        if offset.x() > 0 {
            self.player_mut().xy.0 = 2;
        }
        if offset.y() < 0 {
            self.player_mut().xy.1 = self.chunk.size.y() as i32 - 3;
        }
        if offset.y() > 0 {
            self.player_mut().xy.1 = 2;
        }

        if change_layer && self.coord.layer == ChunkLayer::Underground {
            let resources = resources();
            // Finds the exit
            'outer: for x in 0..self.chunk.size.x() {
                for y in 0..self.chunk.size.y() {
                    let pos = Vec2i(x as i32, y as i32);
                    if self.chunk.get_object_id(pos.into()).map(|id| id == resources.object_tiles.id_of("obj:ladder_up")).unwrap_or(false) {
                        self.player_mut().xy = pos + Vec2i(0, -1);
                        break 'outer;
                    }
                }
            }
        }

        save_file.save_game_state(&self).unwrap();
    }

    fn load_or_generate_chunk(&mut self, coord: ChunkCoord, save_file: &SaveFile, world: &World) {
        let resources = resources();
        // Load or generate new chunk
        let chunk = save_file.load_chunk(&coord, &resources);
        let chunk = match chunk {
            Ok(mut chunk) => {
                // TODO: Dupped code
                let mut rng = Rng::rand();
                rng.next();

                let mut generator = ChunkGenerator::new(&mut chunk, rng);
                generator.regenerate(world);

                chunk
            }
            Err(_) => {
                // TODO: Dupped code
                let mut rng = Rng::rand();
                rng.next();

                // TODO: Size from params
                let mut chunk = Chunk::new(coord, Size2D(80, 80), &resources);
                let mut generator = ChunkGenerator::new(&mut chunk, rng);
                generator.generate(world, &resources);
                
                chunk
            }
        };
        self.chunk = chunk;

        // Spawn actors
        let ai_group = self.ai_groups.next_group();
        let site = world.get_site_at(&self.coord.xy.into());
        if let Some(site) = site {
            let site = world.sites.get(&site);
            match site.site_type {
                SiteType::BanditCamp | SiteType::VarningrLair | SiteType::WolfPack => {
                    self.ai_groups.make_hostile(AiGroups::player(), ai_group);
                },
                SiteType::Village => ()
            };

            // Spawn creatures in structures
            for structure in site.structures.iter() {
                let data = structure.generated_data.as_ref().unwrap();
                let mut spawnpoint_i = 0;
                for creature_id in structure.occupants() {
                    let pos = data.spawn_points[spawnpoint_i % data.spawn_points.len()];
                    let creature = world.creatures.get(creature_id);
                    let species = resources.species.get(&creature.species);
                    let actor = Actor::from_creature(pos.into(), ai_group, *creature_id, &creature, &creature.species, &species, &world, &resources);
                    self.actors.push(actor);
                    spawnpoint_i += 1;
                }
            }

            // Spawn others
            for (pos, spawner) in self.chunk.spawn_points() {
            let actor = match spawner {
                Spawner::CreatureId(creature_id) => {
                    let creature = world.creatures.get(creature_id);
                    if creature.death.is_some() {
                        continue;
                    }
                    let species = resources.species.get(&creature.species);
                    Actor::from_creature((*pos).into(), ai_group, *creature_id, &creature, &creature.species, &species, &world, &resources)
                },
                Spawner::Species(species_id) => {
                    let species = resources.species.get(species_id);
                    Actor::from_species((*pos).into(), &species_id, &species, ai_group)
                },
            };
            self.actors.push(actor);
        }

        }

        self.turn_controller.roll_initiative(self.actors.len());
    }

    pub(crate) fn astar_movement_cost(&self, xy: Coord2) -> MovementCost {
        if !self.chunk.size.in_bounds(xy) || !self.can_occupy(&xy) {
            return MovementCost::Impossible;
        } else {
            return MovementCost::Cost(1.);
        }
    }

    pub(crate) fn can_occupy(&self, coord: &Coord2) -> bool {
        if self.chunk.blocks_movement(coord) {
            return false;
        }
        return !self.actors_iter().any(|actor| actor.xy == coord.to_vec2i())
    }

}

impl Renderable for GameState {
    fn render(&self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &mut GameContext) {
        self.chunk.ground_layer.render(ctx);

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
            if npc.xy.x() < cull_start[0] || npc.xy.y() < cull_start[1] || npc.xy.x() > cull_limit[0] || npc.xy.y() > cull_limit[1] {
                continue
            }
            if !actors_by_position.contains_key(&npc.xy) {
                actors_by_position.insert(&npc.xy, Vec::new());
            }
            actors_by_position.get_mut(&npc.xy).unwrap().push(npc);
        }

        self.chunk.object_layer.render(ctx, game_ctx, |ctx, game_ctx, x, y| {
            if let Some(actors) = actors_by_position.get(&Vec2i(x as i32, y as i32)) {
                for actor in actors {
                    actor.render(ctx, game_ctx);
                }
            }
        });

        for (pos, _item, texture) in self.chunk.items_on_ground.iter() {
            ctx.texture(texture, ctx.at(pos.x as f64 * 24., pos.y as f64 * 24.));
        }

        let size = self.chunk.size;
        if let ChunkLayer::Surface = self.coord.layer {
            // Renders the nav borders
            {
                for y in 1..size.y()-2 {
                    ctx.image("gui/nav_arrow_left.png", [12, y as i32 * 24 + 12]);
                    ctx.image("gui/nav_arrow_right.png", [size.x() as i32 * 24 - 36, y as i32 * 24 + 12]);
                }
            }
            {
                for x in 1..size.x()-2 {
                    ctx.image("gui/nav_arrow_up.png", [x as i32 * 24 + 12, 12]);
                    ctx.image("gui/nav_arrow_down.png", [x as i32 * 24 + 12, size.y() as i32 * 24 - 36]);
                }
            }
            {
                let img = assets().image("gui/nav_corner.png");
                let transform = ctx.context.transform;
                image(&img.texture, transform.trans(12., 12.), ctx.gl);
                image(&img.texture, transform.trans(size.x() as f64 * 24. - 12., 12.).rot_deg(90.), ctx.gl);
                image(&img.texture, transform.trans(size.x() as f64 * 24. - 12., size.y() as f64 * 24. - 12.).rot_deg(180.), ctx.gl);
                image(&img.texture, transform.trans(12., size.y() as f64 * 24. - 12.).rot_deg(270.), ctx.gl);
            }
        }
        // Renders some black bars outside the map to cover large tiles
        {
            let color = Color::from_hex("090714");
            ctx.rectangle_fill([-64., -64., size.x() as f64 * 24. + 76., 76.], &color);
            ctx.rectangle_fill([-64., size.y() as f64 * 24. - 12., size.x() as f64 * 24. + 76., 76.], &color);
            ctx.rectangle_fill([-64., -64., 76., size.y() as f64 * 24. + 76.], &color);
            ctx.rectangle_fill([size.x() as f64 * 24. - 12., -64., 76., size.y() as f64 * 24. + 76.], &color);
        }

    }
}


#[derive(Serialize, Deserialize)]
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

impl Default for AiGroups {
    fn default() -> Self {
        Self::new()
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

#[derive(Serialize, Deserialize)]
pub(crate) struct TurnController {
    turn_idx: usize,
    initiative: Vec<usize>
}

impl TurnController {

    pub(crate) fn new() -> TurnController {
        TurnController {
            initiative: vec!(),
            turn_idx: 0
        }
    }

    pub(crate) fn roll_initiative(&mut self, len: usize) {
        self.initiative = vec![0; len+1];
        for i in 0..len+1 {
            self.initiative[i] = i;
        }
    }

    pub(crate) fn remove(&mut self, index: usize) {
        self.initiative.retain_mut(|i| {
            if *i == index + 1 {
                return false
            }
            if *i > index + 1 {
                *i = *i -1;
            }
            return true
        });
        self.turn_idx = self.turn_idx % self.initiative.len();
    }

    pub(crate) fn is_player_turn(&self) -> bool {
        return self.initiative[self.turn_idx] == 0
    }

    pub(crate) fn npc_idx(&self) -> usize {
        // TODO: Attempt to subtract with overflow on loading a city
        return ((self.initiative[self.turn_idx] as i64 - 1) % self.initiative.len() as i64) as usize
    }

    pub(crate) fn next_turn(&mut self) {
        self.turn_idx = (self.turn_idx + 1) % self.initiative.len();
    }

}