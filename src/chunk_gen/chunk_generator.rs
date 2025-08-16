use core::panic;
use std::{cmp::Ordering, collections::BTreeSet, time::Instant};

use noise::{NoiseFn, Perlin};

use crate::{chunk_gen::jigsaw_structure_generator::JigsawPieceRequirement, commons::{astar::{AStar, MovementCost}, rng::Rng}, engine::tilemap::Tile, game::chunk::{AiGroups, TileMetadata}, info, warn, world::{creature::Profession, unit::{Unit, UnitType}, world::World}, Actor, Chunk, Coord2, Resources};

use super::{jigsaw_parser::JigsawParser, jigsaw_structure_generator::{JigsawPiece, JigsawPieceTile, JigsawSolver}, structure_filter::{AbandonedStructureFilter, NoopFilter, StructureFilter}};

struct ChunkFeaturePools {
    start_pool: Option<String>,
    detached_housing_pool: Option<String>,
    cemetery_pool: Option<String>,
    artifacts_pool: Option<String>,
}

pub(crate) struct ChunkGenParams {
    pub(crate) layer: ChunkLayer
}

#[derive(Clone, Copy)]
pub(crate) enum ChunkLayer {
    Surface,
    Underground
}

pub(crate) struct ChunkGenerator<'a> {
    rng: Rng,
    chunk: &'a mut Chunk,
    path_endpoints: Vec<Coord2>,
    statue_spots: Vec<Coord2>
}

impl<'a> ChunkGenerator<'a> {

    pub(crate) fn new(chunk: &'a mut Chunk, rng: Rng) -> ChunkGenerator<'a> {
        ChunkGenerator {
            rng,
            chunk,
            path_endpoints: Vec::new(),
            statue_spots: Vec::new(),
        }
    }

    pub(crate) fn generate(&mut self, params: &ChunkGenParams, world: &World, xy: Coord2, resources: &Resources) {
        let now = Instant::now();
        self.generate_fixed_terrain_features(params);
        info!("[Chunk gen] Terrain: {:.2?}", now.elapsed());

        let mut solver = self.get_jigsaw_solver();

        let now = Instant::now();
        let mut found_unit = None;
        for unit in world.units.iter() {
            let unit = unit.borrow();
            if unit.xy == xy {
                found_unit = Some(unit)
            }
        }
        info!("[Chunk gen] Unit search ({:?} = {}): {:.2?}", xy, found_unit.is_some(), now.elapsed());

        if let Some(unit) = found_unit {

            match &unit.unit_type {
                // SMELL: What can be abstracted here, if anything?
                UnitType::BanditCamp | UnitType::Village => {
                    let pools = self.get_pools(&unit);

                    let now = Instant::now();
                    self.generate_large_structures(&unit, &mut solver, &pools, resources);
                    info!("[Chunk gen] Large structs: {:.2?}", now.elapsed());

                    info!("Chunk has {} creatures, {} artifacts, {} graves. Peak was {} in {}", unit.creatures.len(), unit.artifacts.len(), unit.cemetery.len(), unit.population_peak.1, unit.population_peak.0);
                    let now = Instant::now();
                    self.generate_buildings(&unit, &mut solver, &pools, world, resources);
                    info!("[Chunk gen] Building gen: {:.2?}", now.elapsed());

                    let now = Instant::now();
                    self.generate_ruins(&unit, &mut solver, &pools, world, resources);
                    info!("[Chunk gen] Ruins gen: {:.2?}", now.elapsed());

                    if self.statue_spots.len() > 0 {
                        let now = Instant::now();
                        self.place_statues(&unit, &world, &resources);
                        info!("[Chunk gen] Statues: {:.2?}", now.elapsed());
                    }

                    if self.path_endpoints.len() > 0 {
                        let now = Instant::now();
                        self.generate_paths(&mut solver);
                        info!("[Chunk gen] Streets: {:.2?}", now.elapsed());
                    }
                },
                UnitType::VarningrLair => {
                    let now = Instant::now();
                    match params.layer {
                        ChunkLayer::Surface => self.generate_lair_entrance(&mut solver, resources),
                        ChunkLayer::Underground => self.generate_lair(&unit, &mut solver, &world, resources),
                    };
                    // self.generate_lair(&unit, &mut solver, &world, resources);
                    info!("[Chunk gen] Large structs: {:.2?}", now.elapsed());
                },
                UnitType::WolfPack => {
                    self.generate_wolf_pack(&unit, &world, resources);
                }
            }
        }      

        let now = Instant::now();
        self.collapse_decor(resources);
        info!("[Chunk gen] Decor: {:.2?}", now.elapsed());
    }

    fn get_pools(&self, unit: &Unit) -> ChunkFeaturePools {
        match unit.unit_type {
            UnitType::Village => ChunkFeaturePools {
                start_pool: None,
                detached_housing_pool: Some(String::from("village_house_start")),
                artifacts_pool: Some(String::from("village_plaza")),
                cemetery_pool: Some(String::from("village_cemetery"))
            },
            UnitType::BanditCamp => ChunkFeaturePools {
                start_pool: None,
                detached_housing_pool: Some(String::from("camp_start")),
                artifacts_pool: None,
                cemetery_pool: None
            },
            UnitType::VarningrLair => ChunkFeaturePools {
                start_pool: None,
                detached_housing_pool: Some(String::from("varningr_lair")),
                artifacts_pool: None,
                cemetery_pool: None
            },
            UnitType::WolfPack => ChunkFeaturePools {
                start_pool: None,
                detached_housing_pool: None,
                artifacts_pool: None,
                cemetery_pool: None
            },
        }
    }

    fn generate_fixed_terrain_features(&mut self, params: &ChunkGenParams) {
        match params.layer {
            ChunkLayer::Surface => {
                // TODO: Based on region
                let noise = Perlin::new(self.rng.derive("grass").seed());
                
                for x in 0..self.chunk.size.x() {
                    for y in 0..self.chunk.size.y() {
                        let n = noise.get([x as f64 / 15.0, y as f64 / 15.0]);
                        if n > 0. {
                            if n > 0.9 {
                                self.chunk.map.ground_layer.set_tile(x, y, 7);
                            } else {
                                self.chunk.map.ground_layer.set_tile(x, y, 1);
                            }
                        } else {
                            self.chunk.map.ground_layer.set_tile(x, y, 6);
                        }
                    }
                }
            },
            ChunkLayer::Underground => {
                for x in 0..self.chunk.size.x() {
                    for y in 0..self.chunk.size.y() {
                        self.chunk.map.ground_layer.set_tile(x, y, 6);
                        self.chunk.map.object_layer.set_tile(x, y, 15);
                    }
                }
            }
        }
    }

    fn generate_large_structures(&mut self, unit: &Unit, solver: &mut JigsawSolver, pools: &ChunkFeaturePools, resources: &Resources) {
        let mut building_seed_cloud = BTreeSet::new();
        for _ in 0..50 {
            building_seed_cloud.insert(Coord2::xy(
                self.rng.randu_range(0, self.chunk.size.x()) as i32,
                self.rng.randu_range(0, self.chunk.size.y()) as i32
            ));
        }
        let center = Coord2::xy(self.chunk.size.x() as i32 / 2, self.chunk.size.y() as i32 / 2);
        let mut building_seed_cloud: Vec<Coord2> = building_seed_cloud.into_iter().collect();
        building_seed_cloud.sort_by(|a, b| {
            let a = a.dist_squared(&center);
            let b = b.dist_squared(&center);
            if a < b {
                return Ordering::Greater;
            } else {
                return Ordering::Less;
            }
        });

        if let Some(start_pool) = &pools.start_pool {
            let pos = building_seed_cloud.pop().unwrap();
            let structure = solver.solve_structure(start_pool, pos, &mut self.rng, Vec::new());
            if let Ok(structure) = structure {
                for (pos, piece) in structure.vec.iter() {
                    self.place_template(*pos, &piece, resources);
                }
            } else {
                warn!("Failed to spawn structure")
            }
        }

        if unit.artifacts.len() > 0 {
            if let Some(artifacts_pool) = &pools.artifacts_pool {
                while building_seed_cloud.len() > 0 {

                    let pos = building_seed_cloud.pop().unwrap();

                    let structure = solver.solve_structure(artifacts_pool, pos, &mut self.rng, Vec::new());
                    if let Ok(structure) = structure {
                        for (pos, piece) in structure.vec.iter() {
                            self.place_template(*pos, &piece, resources);
                        }
                        break;
                    }


                }
            }
        }

        if unit.cemetery.len() > 0 {
            if let Some(cemetery_pool) = &pools.cemetery_pool {
                let mut collapsed_pos = None;

                while building_seed_cloud.len() > 0 {

                    let pos = building_seed_cloud.pop().unwrap();

                    let structure = solver.solve_structure(cemetery_pool, pos, &mut self.rng, Vec::new());
                    if let Ok(structure) = structure {
                        collapsed_pos = Some(pos);
                        for (pos, piece) in structure.vec.iter() {
                            self.place_template(*pos, &piece, resources);
                        }
                        break;
                    }


                }

                if collapsed_pos.is_none() {
                    // TODO: No panic
                    panic!("No position found")
                }
                let collapsed_pos = collapsed_pos.unwrap();

                let mut x = collapsed_pos.x;
                let mut y = collapsed_pos.y;

                
                let mut slice = &unit.cemetery[..];
                // TODO:
                if slice.len() > 49 {
                    slice = &unit.cemetery[0..49];
                }

                for creature in slice.iter() {
                    self.chunk.map.object_layer.set_tile(x as usize, y as usize, 6);
                    self.chunk.map.tiles_metadata.insert(Coord2::xy(x, y), TileMetadata::BurialPlace(*creature));


                    x = x + 2;
                    if x > collapsed_pos.x + 15 {
                        y = y + 2;
                        x = collapsed_pos.x;
                    }
                }
            }

        }
    }

    fn generate_buildings(&mut self, unit: &Unit, solver: &mut JigsawSolver, pools: &ChunkFeaturePools, world: &World, resources: &Resources) {
        let ai_group = self.chunk.ai_groups.next_group();
        if unit.unit_type == UnitType::BanditCamp {
            self.chunk.ai_groups.make_hostile(AiGroups::player(), ai_group);
        }

        let mut homeless = unit.creatures.clone();

        let mut building_seed_cloud = BTreeSet::new();
        for _ in 0..1000 {
            building_seed_cloud.insert(Coord2::xy(
                self.rng.randu_range(0, self.chunk.size.x()) as i32,
                self.rng.randu_range(0, self.chunk.size.y()) as i32
            ));
        }
        let center = Coord2::xy(self.chunk.size.x() as i32 / 2, self.chunk.size.y() as i32 / 2);
        let mut building_seed_cloud: Vec<Coord2> = building_seed_cloud.into_iter().collect();
        building_seed_cloud.sort_by(|a, b| {
            let a = a.dist_squared(&center);
            let b = b.dist_squared(&center);
            if a < b {
                return Ordering::Greater;
            } else {
                return Ordering::Less;
            }
        });
        
        if let Some(detached_housing_pool) = &pools.detached_housing_pool {

            while homeless.len() > 0 {
                let mut is_ruler = false;

                let creature_id = homeless.pop().unwrap();
                if world.is_played_creature(&creature_id) {
                    continue;
                }

                let mut family = vec!(creature_id);
                let creature = world.creatures.get(&creature_id);
                is_ruler = is_ruler || creature.profession == Profession::Ruler;
                if let Some(spouse) = creature.spouse {
                    let in_homeless = homeless.iter().position(|id| *id == spouse);
                    if let Some(index) = in_homeless {
                        homeless.remove(index);
                        let creature = world.creatures.get(&spouse);
                        is_ruler = is_ruler || creature.profession == Profession::Ruler;
                        family.push(spouse);
                    }
                }
                for child_id in creature.offspring.iter() {
                    let in_homeless = homeless.iter().position(|id| *id == *child_id);
                    if let Some(index) = in_homeless {
                        let child = world.creatures.get(child_id);
                        if child.profession != Profession::None {
                            homeless.remove(index);
                            let creature = world.creatures.get(child_id);
                            is_ruler = is_ruler || creature.profession == Profession::Ruler;
                            family.push(*child_id);
                        }
                    }
                }

                let mut collapsed_pos = None;

                while building_seed_cloud.len() > 0 {

                    let pos = building_seed_cloud.pop().unwrap();

                    let mut pool = detached_housing_pool.as_str();
                    if is_ruler && unit.unit_type == UnitType::Village {
                        pool = "village_house_ruler";
                    }

                    let structure = solver.solve_structure(pool, pos, &mut self.rng, Vec::new());
                    if let Ok(structure) = structure {
                        collapsed_pos = Some(pos);

                        let mut iter = structure.vec.iter();

                        if let Some((pos, piece)) = iter.next() {
                            self.place_template(*pos, &piece, resources);
                            for creature_id in family.iter() {
                                let creature = world.creatures.get(creature_id);
                                let species = resources.species.get(&creature.species);
                                let actor = Actor::from_creature(Coord2::xy(0, 0), ai_group, *creature_id, &creature, &creature.species, &species, world, resources);
                                self.spawn(actor, *pos + Coord2::xy(piece.size.0 as i32 / 2 - 1, piece.size.1 as i32 / 2 - 1), 2);
                            }
                        }
                        
                        for (pos, piece) in iter {
                            self.place_template(*pos, &piece, resources);
                        }
                        break;
                    }


                }

                if collapsed_pos.is_none() {
                    warn!("No position found");
                    continue;
                }

            }
        }
    }

    fn generate_ruins(&mut self, unit: &Unit, solver: &mut JigsawSolver, pools: &ChunkFeaturePools, world: &World, resources: &Resources) {
        let mut building_seed_cloud = BTreeSet::new();
        for _ in 0..1000 {
            building_seed_cloud.insert(Coord2::xy(
                self.rng.randu_range(0, self.chunk.size.x()) as i32,
                self.rng.randu_range(0, self.chunk.size.y()) as i32
            ));
        }
        let center = Coord2::xy(self.chunk.size.x() as i32 / 2, self.chunk.size.y() as i32 / 2);
        let mut building_seed_cloud: Vec<Coord2> = building_seed_cloud.into_iter().collect();
        building_seed_cloud.sort_by(|a, b| {
            let a = a.dist_squared(&center);
            let b = b.dist_squared(&center);
            if a < b {
                return Ordering::Greater;
            } else {
                return Ordering::Less;
            }
        });
        
        if let Some(detached_housing_pool) = &pools.detached_housing_pool {
            // TODO(BUG): attempt to subtract with overflow
            let pop_diff = unit.population_peak.1 as usize - unit.creatures.len();
            let ruins = pop_diff / 2;

            let age = world.date.year() - unit.population_peak.0;

            let mut j = 0;

            for _ in 0..ruins {
                j = j + 1;

                let mut collapsed_pos = None;

                while building_seed_cloud.len() > 0 {

                    let pos = building_seed_cloud.pop().unwrap();

                    let structure = solver.solve_structure(detached_housing_pool, pos, &mut self.rng, Vec::new());
                    if let Ok(structure) = structure {
                        collapsed_pos = Some(pos);
                        for (pos, piece) in structure.vec.iter() {
                            self.place_template_filtered(*pos, &piece, resources, AbandonedStructureFilter::new(self.rng.clone(), age as u32));
                        }
                        break;
                    }


                }

                if collapsed_pos.is_none() {
                    warn!("No position found" );
                    continue;
                }

            }
        }
    }

    fn generate_paths(&mut self, solver: &mut JigsawSolver) {
        while self.path_endpoints.len() > 1 {
            let start = self.path_endpoints.pop().unwrap();

            let mut closest = self.path_endpoints.first().unwrap();
            let mut closes_dst = closest.dist_squared(&start);
            for point in self.path_endpoints.iter() {
                let dst = start.dist_squared(point);
                if dst < closes_dst {
                    closest = point;
                    closes_dst = dst;
                }
            }
            
            let mut astar = AStar::new(self.chunk.size, start);
            astar.find_path(*closest, |xy| {
                if !self.chunk.size.in_bounds(xy) || self.chunk.map.blocks_movement(xy) || solver.is_occupied(xy) {
                    return MovementCost::Impossible;
                } else {
                    return MovementCost::Cost(1.);
                }
            });
            let path = astar.get_path(*closest);
            for step in path {
                self.chunk.map.ground_layer.set_tile(step.x as usize, step.y as usize, 5);
            }


        }
    }

    fn generate_lair_entrance(&mut self, solver: &mut JigsawSolver, resources: &Resources) {
        let structure = solver.solve_structure("varningr_surface", Coord2::xy(self.chunk.size.0 as i32 / 2, self.chunk.size.1 as i32 / 2), &mut self.rng, Vec::new());
        if let Ok(structure) = structure {
            for (pos, piece) in structure.vec.iter() {
                self.place_template(*pos, &piece, resources);
                if piece.size.area() > 7*7 && self.rng.rand_chance(0.3) {
                    let ai_group = self.chunk.ai_groups.next_group();
                    let wolf_id = resources.species.id_of("species:wolf");
                    let wolf = resources.species.get(&wolf_id);
                    // Minion wolves
                    for _ in 0..self.rng.randi_range(1, 4) {
                        let boss = Actor::from_species(Coord2::xy(0, 0), &wolf_id, wolf, ai_group);
                        self.spawn(boss, *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 7);
                    }
                }
            }
        }
    }

    fn generate_wolf_pack(&mut self, unit: &Unit, world: &World, resources: &Resources) {
        let ai_group = self.chunk.ai_groups.next_group();
        self.chunk.ai_groups.make_hostile(AiGroups::player(), ai_group);
        let pos = Coord2::xy(self.chunk.size.x() as i32 / 2, self.chunk.size.x() as i32 / 2);
        for creature_id in unit.creatures.iter() {
            let creature = world.creatures.get(creature_id);
            let species = resources.species.get(&creature.species);
            let boss = Actor::from_creature(Coord2::xy(0, 0), ai_group, *creature_id, &creature, &creature.species, &species, world, resources);
            self.spawn(boss, pos, 7);
        }
    }

    fn generate_lair(&mut self, unit: &Unit, solver: &mut JigsawSolver, world: &World, resources: &Resources) {
        let requirements = vec!(
            JigsawPieceRequirement::Exactly("varningr_lair".to_string(), 1),
            JigsawPieceRequirement::Exactly("varningr_entrance".to_string(), 1)
        );
        let structure = solver.solve_structure("varningr_lair", Coord2::xy(self.chunk.size.0 as i32 / 2, self.chunk.size.1 as i32 / 2), &mut self.rng, requirements);
        let ai_group = self.chunk.ai_groups.next_group();
        self.chunk.ai_groups.make_hostile(ai_group, AiGroups::player());
        if let Ok(structure) = structure {
            let wolf_id = resources.species.id_of("species:wolf");
            let wolf = resources.species.get(&wolf_id);
            let mut iter = structure.vec.iter();

            // First piece
            if let Some((pos, piece)) = iter.next() {
                self.place_template(*pos, &piece, resources);

                // Spawn varningr(s)
                for creature_id in unit.creatures.iter() {
                    let creature = world.creatures.get(creature_id);
                    let species = resources.species.get(&creature.species);
                    let boss = Actor::from_creature(Coord2::xy(0, 0), ai_group, *creature_id, &creature, &creature.species, &species, world, resources);
                    self.spawn(boss, *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 5);
                }
                // Minion wolves
                for _ in 0..3 {
                    let boss = Actor::from_species(Coord2::xy(0, 0), &wolf_id, wolf, ai_group);
                    self.spawn(boss, *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 5);
                }
            }
            for (pos, piece) in iter {
                self.place_template(*pos, &piece, resources);
                if piece.size.area() > 7*7 && self.rng.rand_chance(0.3) {
                    // Minion wolves
                    for _ in 0..self.rng.randi_range(1, 4) {
                        let boss = Actor::from_species(Coord2::xy(0, 0), &wolf_id, wolf, ai_group);
                        self.spawn(boss, *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 5);
                    }
                }
            }
        }
    }

    fn spawn(&mut self, mut actor: Actor, close_to: Coord2, r: i32) {
        for _ in 0..100 {
            let xy = close_to + Coord2::xy(self.rng.randi_range(-r, r), self.rng.randi_range(-r, r));
            if self.chunk.can_occupy(&xy) {
                actor.xy = xy;
                self.chunk.actors.push(actor);
                return;
            }
        }
    }

    fn place_statues(&mut self, unit: &Unit, world: &World, resources: &Resources) {
        let spots = self.statue_spots.clone();
        let mut spots = self.rng.shuffle(spots);
        for item in unit.artifacts.iter() {
            let spot = spots.pop();
            match spot {
                None => {
                    warn!("Not enough spots for artifacts");
                    break;
                },
                Some(spot) => {
                    let item = world.artifacts.get(item);
                    let texture = item.make_texture(&resources.materials);
                    self.chunk.map.items_on_ground.push((spot, item.clone(), texture));
                }
            }
        }
    }

    pub(crate) fn get_jigsaw_solver(&self) -> JigsawSolver {
        let mut solver = JigsawSolver::new(self.chunk.size.clone(), self.rng.clone());
        
        let parser = JigsawParser::new();
        if let Ok(pools) = parser.parse_file("assets/structures/village.toml") {
            for (name, pool) in pools {
                solver.add_pool(&name, pool);
            }
        }

        if let Ok(pools) = parser.parse_file("assets/structures/bandit_camp.toml") {
            for (name, pool) in pools {
                solver.add_pool(&name, pool);
            }
        }

        if let Ok(pools) = parser.parse_file("assets/structures/varningr_lair.toml") {
            for (name, pool) in pools {
                solver.add_pool(&name, pool);
            }
        }

        return solver;
    }

    fn collapse_decor(&mut self, resources: &Resources) {
        let tree_noise = Perlin::new(self.rng.derive("trees").seed());
        let flower_noise = Perlin::new(self.rng.derive("flower").seed());
        for x in 1..self.chunk.size.x()-1 {
            for y in 1..self.chunk.size.y()-1 {
                if let Some(ground) = self.chunk.map.ground_layer.tile(x, y) {
                    if let Tile::Empty = self.chunk.map.object_layer.get_tile(x, y) {
                        if ground == 1 || ground == 6 || ground == 7 {
                            if tree_noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0. {
                                if self.rng.rand_chance(0.1) {
                                    self.chunk.map.set_object_key(Coord2::xy(x as i32, y as i32), "obj:tree", resources);
                                    continue;
                                }
                            }
                            if self.rng.rand_chance(0.02) {
                                self.chunk.map.object_layer.set_tile(x as usize, y as usize, 11);
                                continue;
                            }
                            if flower_noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0.6 && self.rng.rand_chance(0.3) {
                                self.chunk.map.object_layer.set_tile(x as usize, y as usize, 12);
                                continue;
                            }
                            if self.rng.rand_chance(0.2) {
                                self.chunk.map.object_layer.set_tile(x as usize, y as usize, 9);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn place_template(&mut self, origin: Coord2, template: &JigsawPiece, resources: &Resources) {
        self.place_template_filtered(origin, template, resources, NoopFilter {});
    }

    fn place_template_filtered<F>(&mut self, origin: Coord2, template: &JigsawPiece, resources: &Resources, mut filter: F) where F: StructureFilter {
        for i in 0..template.size.area() {
            let x = origin.x as usize + i % template.size.x();
            let y = origin.y as usize + i / template.size.x();
            let mut tile = template.tiles.get(i).unwrap().clone();

            let filtered = filter.filter(Coord2::xy(x as i32, y as i32), &tile);
            if let Some(filtered) = filtered {
                tile = filtered;
            }

            match tile {
                JigsawPieceTile::Air => (),
                JigsawPieceTile::Empty => (),
                JigsawPieceTile::PathEndpoint => self.path_endpoints.push(Coord2::xy(x as i32, y as i32)),
                JigsawPieceTile::Fixed { ground, object, statue_spot, connection: _ } => {
                    self.chunk.map.ground_layer.set_tile(x, y, ground);
                    if let Some(object) = object {
                        self.chunk.map.set_object_idx(Coord2::xy(x as i32, y as i32), object, resources);
                    }
                    if statue_spot {
                        self.statue_spots.push(Coord2::xy(x as i32, y as i32))
                    }
                },
            }
        }
    }

}
