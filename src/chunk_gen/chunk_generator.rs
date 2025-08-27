use std::{collections::BTreeSet, time::Instant};

use common::error::Error;
use noise::{NoiseFn, Perlin};

use crate::{chunk_gen::jigsaw_structure_generator::JigsawPieceRequirement, commons::{astar::{AStar, MovementCost}, id_vec::Id, rng::Rng}, engine::tilemap::Tile, game::chunk::{Chunk, ChunkLayer, Spawner}, info, resources::resources::resources, warn, world::{site::{Structure, StructureGeneratedData, StructureStatus, StructureType, Site, SiteType}, world::World}, Coord2, Resources};

use super::{jigsaw_parser::JigsawParser, jigsaw_structure_generator::{JigsawPiece, JigsawPieceTile, JigsawSolver}, structure_filter::{AbandonedStructureFilter, NoopFilter, StructureFilter}};

pub(crate) struct ChunkGenerator<'a> {
    rng: Rng,
    chunk: &'a mut Chunk,
    /// Point cloud sorted by distance to the center
    structure_point_cloud: Vec<Coord2>,
    path_endpoints: Vec<Coord2>,
    statue_spots: Vec<Coord2>
}

impl<'a> ChunkGenerator<'a> {

    pub(crate) fn new(chunk: &'a mut Chunk, rng: Rng) -> ChunkGenerator<'a> {
        ChunkGenerator {
            rng,
            chunk,
            structure_point_cloud: Vec::new(),
            path_endpoints: Vec::new(),
            statue_spots: Vec::new(),
        }
    }

    pub(crate) fn generate(&mut self, world: &World, resources: &Resources) {
        let now = Instant::now();
        self.generate_fixed_terrain_features();
        info!("[Chunk gen] Terrain: {:.2?}", now.elapsed());

        self.structure_point_cloud = self.generate_point_cloud(1000);

        let mut solver = self.get_jigsaw_solver();

        let now = Instant::now();
        let mut found_site = None;
        for site in world.sites.iter() {
            let site = site.borrow_mut();
            if site.xy == self.chunk.coord.xy {
                found_site = Some(site)
            }
        }
        info!("[Chunk gen] Site search ({:?} = {}): {:.2?}", self.chunk.coord.xy, found_site.is_some(), now.elapsed());

        // TODO(WCF3fkX3): Bandit camps
        // detached_housing_pool: Some(String::from("camp_start")),

        if let Some(mut site) = found_site {

            dbg!(&site.structures);
            for structure in site.structures.iter_mut() {
                if structure.generated_data.is_none() {
                    match self.generate_structure(structure, &mut solver) {
                        Ok(data) => structure.generated_data = Some(data),
                        Err(err) => warn!("{err}")
                    }
                }
            }

            match &site.site_type {
                SiteType::BanditCamp | SiteType::Village => {

                    if self.path_endpoints.len() > 0 {
                        let now = Instant::now();
                        self.generate_paths(&mut solver);
                        info!("[Chunk gen] Streets: {:.2?}", now.elapsed());
                    }

                },
                SiteType::VarningrLair => {
                    let now = Instant::now();
                    match self.chunk.coord.layer {
                        ChunkLayer::Surface => self.generate_lair_entrance(&mut solver, resources),
                        ChunkLayer::Underground => self.generate_lair(&site, &mut solver, resources),
                    };
                    info!("[Chunk gen] Large structs: {:.2?}", now.elapsed());
                },
                SiteType::WolfPack => {
                    self.generate_wolf_pack(&site);
                }
            }
        }      

        let now = Instant::now();
        self.collapse_decor(resources);
        info!("[Chunk gen] Decor: {:.2?}", now.elapsed());
    }

    pub(crate) fn regenerate(&mut self, world: &World) {

        self.structure_point_cloud = self.generate_point_cloud(1000);

        let mut solver = self.get_jigsaw_solver();

        if let Some(site) = world.get_site_at(&self.chunk.coord.xy) {
            let mut site = world.sites.get_mut(&site);
            for structure in site.structures.iter_mut() {
                dbg!(&structure);
                match &structure.generated_data {
                    None => {
                        match self.generate_structure(structure, &mut solver) {
                            Ok(data) => structure.generated_data = Some(data),
                            Err(err) => warn!("{err}")
                        }
                    },
                    Some(generated_data) => {
                        match &structure.get_status() {
                            StructureStatus::Occupied => {
                                match self.regenerate_structure(structure.get_status().clone(), generated_data, &mut solver) {
                                    Ok(data) => structure.generated_data = Some(data),
                                    Err(err) => warn!("{err}")
                                }
                            },
                            StructureStatus::Abandoned => {
                                if let Err(err) = self.age_structure(structure.get_status().clone(), generated_data, &mut solver) {
                                    warn!("{err}")
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_structure(&mut self, structure: &Structure, solver: &mut JigsawSolver) -> Result<StructureGeneratedData, Error> {
        let pool =  match structure.get_type() {
            StructureType::House => "village_house_start",
            StructureType::TownHall => "village_house_ruler",
        };

        let mut generated_data = StructureGeneratedData::new(structure.get_status().clone());

        loop {
            let pos = self.structure_point_cloud.pop().ok_or("No more possible points for structure")?;

            let mut filter: Box<dyn StructureFilter> = match structure.get_status() {
                // TODO(WCF3fkX3): Age
                StructureStatus::Abandoned => Box::new(AbandonedStructureFilter::new(self.rng.clone(), 30 as u32)),
                StructureStatus::Occupied => Box::new(NoopFilter {}),
            };

            let built_structure = solver.solve_structure(pool, pos, &mut self.rng, Vec::new());
            if let Ok(built_structure) = built_structure {

                for (pos, piece) in built_structure.vec.iter() {
                    self.place_template_filtered(*pos, &piece, &mut filter);
                    
                    let rect = [pos.x as u8, pos.y as u8, piece.size.0 as u8, piece.size.1 as u8];
                    generated_data.add_piece(piece.name.clone(), rect);

                    let center = *pos + Coord2::xy(piece.size.0 as i32 / 2 - 1, piece.size.1 as i32 / 2 - 1);

                    // TODO(WCF3fkX3): Review
                    for i in 0..4 {
                        // self.spawn(Spawner::CreatureId(*creature_id), *pos + Coord2::xy(piece.size.0 as i32 / 2 - 1, piece.size.1 as i32 / 2 - 1), 3);
                        
                        let xy = center + Coord2::xy(i % 2, i / 2);
                        if !self.chunk.blocks_movement(&xy) {
                            generated_data.spawn_points.push(xy.to_vec2i());
                        }

                    }

                }

                return Ok(generated_data)
            }
        }

    }

    fn regenerate_structure(&mut self, structure_status: StructureStatus, generated_data: &StructureGeneratedData, solver: &mut JigsawSolver) -> Result<StructureGeneratedData, Error> {
        let mut new_generated_data = StructureGeneratedData::new(structure_status);

        for (piece_name, rect) in generated_data.pieces() {

            let piece = solver.find_piece(&piece_name)?;
            let pos = Coord2::xy(rect[0] as i32, rect[1] as i32);

            self.place_template_filtered(pos, &piece, &mut Box::new(NoopFilter {}));
                    
            let rect = [pos.x as u8, pos.y as u8, piece.size.0 as u8, piece.size.1 as u8];
            new_generated_data.add_piece(piece.name.clone(), rect);

            let center = pos + Coord2::xy(piece.size.0 as i32 / 2 - 1, piece.size.1 as i32 / 2 - 1);

            // TODO(WCF3fkX3): Review
            for i in 0..4 {
                // self.spawn(Spawner::CreatureId(*creature_id), *pos + Coord2::xy(piece.size.0 as i32 / 2 - 1, piece.size.1 as i32 / 2 - 1), 3);
                
                let xy = center + Coord2::xy(i % 2, i / 2);
                if !self.chunk.blocks_movement(&xy) {
                    new_generated_data.spawn_points.push(xy.to_vec2i());
                }

            }
        }

        Ok(new_generated_data)
    }

    fn age_structure(&mut self, structure_status: StructureStatus, generated_data: &StructureGeneratedData, solver: &mut JigsawSolver) -> Result<(), Error> {
        let resources = resources();

        // TODO(WCF3fkX3): Compute
        let age = 1;

        let mut filter = AbandonedStructureFilter::new(self.rng.clone(), age);

        for (_piece_name, rect) in generated_data.pieces() {
            for x in rect[0]..(rect[0] + rect[2]) {
                for y in rect[1]..(rect[1] + rect[3]) {
                    let position = Coord2::xy(x as i32, y as i32);
                    let ground = self.chunk.ground_layer.tile(x as usize, y as usize).and_then(|t| resources.tiles.validate_id(t)).ok_or("Invalid tile")?;
                    let object = self.chunk.get_object_id(position);
                    let new_tile = filter.filter(position, &ground, object);
                    if let Some(tile) = new_tile {
                        self.chunk.ground_layer.set_tile(x as usize, y as usize, tile.as_usize());
                        self.chunk.set_object_idx(position, 0);
                    }
                }
            }
        }
        return Ok(())
    }

    fn generate_fixed_terrain_features(&mut self) {
        let resources = resources();
        match self.chunk.coord.layer {
            ChunkLayer::Surface => {
                let patchy_grass = resources.tiles.id_of("tile:grass_patchy").as_usize();
                let grass = resources.tiles.id_of("tile:grass").as_usize();
                let dark_grass = resources.tiles.id_of("tile:grass_dark").as_usize();

                let noise = Perlin::new(self.rng.derive("grass").seed());                
                for x in 0..self.chunk.size.x() {
                    for y in 0..self.chunk.size.y() {
                        let n = noise.get([x as f64 / 15.0, y as f64 / 15.0]);
                        if n > 0. {
                            if n > 0.9 {
                                self.chunk.ground_layer.set_tile(x, y, patchy_grass);
                            } else {
                                self.chunk.ground_layer.set_tile(x, y, grass);
                            }
                        } else {
                            self.chunk.ground_layer.set_tile(x, y, dark_grass);
                        }
                    }
                }
            },
            ChunkLayer::Underground => {
                let cave_floor = resources.tiles.id_of("tile:cave_floor").as_usize();
                // SMELL: See smells in chunk
                let cave_wall = resources.object_tiles.id_of("obj:cave_wall").as_usize() + 1;

                for x in 0..self.chunk.size.x() {
                    for y in 0..self.chunk.size.y() {
                        self.chunk.ground_layer.set_tile(x, y, cave_floor);
                        self.chunk.object_layer.set_tile(x, y, cave_wall);
                    }
                }
            }
        }
    }

    fn generate_point_cloud(&mut self, number_of_points: usize) -> Vec<Coord2> {
        let mut point_cloud = BTreeSet::new();
        for _ in 0..number_of_points {
            point_cloud.insert(Coord2::xy(
                self.rng.randu_range(0, self.chunk.size.x()) as i32,
                self.rng.randu_range(0, self.chunk.size.y()) as i32
            ));
        }
        let center = Coord2::xy(self.chunk.size.x() as i32 / 2, self.chunk.size.y() as i32 / 2);
        let mut building_seed_cloud: Vec<Coord2> = point_cloud.into_iter().collect();
        building_seed_cloud.sort_by(|a, b| {
            let a = a.dist_squared(&center);
            let b = b.dist_squared(&center);
            b.total_cmp(&a)
        });
        return building_seed_cloud;
    }

    fn generate_paths(&mut self, solver: &mut JigsawSolver) {
        let resources = resources();
        let path_tile = resources.tiles.id_of("tile:cobblestone").as_usize();
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
                if !self.chunk.size.in_bounds(xy) || self.chunk.blocks_movement(&xy) || solver.is_occupied(xy) {
                    return MovementCost::Impossible;
                } else {
                    return MovementCost::Cost(1.);
                }
            });
            let path = astar.get_path(*closest);
            for step in path {
                self.chunk.ground_layer.set_tile(step.x as usize, step.y as usize, path_tile);
            }


        }
    }

    fn generate_lair_entrance(&mut self, solver: &mut JigsawSolver, resources: &Resources) {
        let structure = solver.solve_structure("varningr_surface", Coord2::xy(self.chunk.size.0 as i32 / 2, self.chunk.size.1 as i32 / 2), &mut self.rng, Vec::new());
        if let Ok(structure) = structure {
            for (pos, piece) in structure.vec.iter() {
                self.place_template(*pos, &piece);
                if piece.size.area() > 7*7 && self.rng.rand_chance(0.3) {
                    let wolf_id = resources.species.id_of("species:wolf");
                    // Minion wolves
                    for _ in 0..self.rng.randi_range(1, 4) {
                        self.spawn(Spawner::Species(wolf_id), *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 7);
                    }
                }
            }
        }
    }

    fn generate_wolf_pack(&mut self, site: &Site) {
        let pos = Coord2::xy(self.chunk.size.x() as i32 / 2, self.chunk.size.x() as i32 / 2);
        for creature_id in site.creatures.iter() {
            self.spawn(Spawner::CreatureId(*creature_id), pos, 7);
        }
    }

    fn generate_lair(&mut self, site: &Site, solver: &mut JigsawSolver, resources: &Resources) {
        let requirements = vec!(
            JigsawPieceRequirement::Exactly("varningr_lair".to_string(), 1),
            JigsawPieceRequirement::Exactly("varningr_entrance".to_string(), 1)
        );
        let structure = solver.solve_structure("varningr_lair", Coord2::xy(self.chunk.size.0 as i32 / 2, self.chunk.size.1 as i32 / 2), &mut self.rng, requirements);
        if let Ok(structure) = structure {
            let wolf_id = resources.species.id_of("species:wolf");
            let mut iter = structure.vec.iter();

            // First piece
            if let Some((pos, piece)) = iter.next() {
                self.place_template(*pos, &piece);

                // Spawn varningr(s)
                for creature_id in site.creatures.iter() {
                    self.spawn(Spawner::CreatureId(*creature_id), *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 5);
                }
                // Minion wolves
                for _ in 0..3 {
                    self.spawn(Spawner::Species(wolf_id), *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 5);
                }
            }
            for (pos, piece) in iter {
                self.place_template(*pos, &piece);
                if piece.size.area() > 7*7 && self.rng.rand_chance(0.3) {
                    // Minion wolves
                    for _ in 0..self.rng.randi_range(1, 4) {
                        self.spawn(Spawner::Species(wolf_id), *pos + Coord2::xy(piece.size.0 as i32 / 2, piece.size.1 as i32 / 2), 5);
                    }
                }
            }
        }
    }

    fn spawn(&mut self, spawner: Spawner, close_to: Coord2, r: i32) {
        for _ in 0..100 {
            let xy = close_to + Coord2::xy(self.rng.randi_range(-r, r), self.rng.randi_range(-r, r));
            if !self.chunk.blocks_movement(&xy) && self.chunk.get_spawner_at(&xy).is_none() {
                self.chunk.add_spawn_point(xy, spawner);
                return;
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

        let pebles = resources.object_tiles.id_of("obj:pebbles").as_usize() + 1;
        let flowers = resources.object_tiles.id_of("obj:flowers").as_usize() + 1;
        let grass = resources.object_tiles.id_of("obj:grass_decal").as_usize() + 1;
        let tree = resources.object_tiles.id_of("obj:tree").as_usize() + 1;

        for x in 1..self.chunk.size.x()-1 {
            for y in 1..self.chunk.size.y()-1 {
                if let Some(ground) = self.chunk.ground_layer.tile(x, y) {
                    if let Tile::Empty = self.chunk.object_layer.get_tile(x, y) {
                        if ground == 1 || ground == 6 || ground == 7 {
                            if tree_noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0. {
                                if self.rng.rand_chance(0.1) {
                                    self.chunk.object_layer.set_tile(x as usize, y as usize, tree);
                                    continue;
                                }
                            }
                            if self.rng.rand_chance(0.02) {
                                self.chunk.object_layer.set_tile(x as usize, y as usize, pebles);
                                continue;
                            }
                            if flower_noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0.6 && self.rng.rand_chance(0.3) {
                                self.chunk.object_layer.set_tile(x as usize, y as usize, flowers);
                                continue;
                            }
                            if self.rng.rand_chance(0.2) {
                                self.chunk.object_layer.set_tile(x as usize, y as usize, grass);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn place_template(&mut self, origin: Coord2, template: &JigsawPiece) {
        self.place_template_filtered(origin, template, &mut Box::new(NoopFilter {}));
    }

    fn place_template_filtered<F>(&mut self, origin: Coord2, template: &JigsawPiece, filter: &mut Box<F>) where F: StructureFilter + ?Sized {
        let resources = resources();
        for i in 0..template.size.area() {
            let x = origin.x as usize + i % template.size.x();
            let y = origin.y as usize + i / template.size.x();
            let tile = template.tiles.get(i).unwrap().clone();

            match tile {
                JigsawPieceTile::Air => (),
                JigsawPieceTile::Empty => (),
                JigsawPieceTile::PathEndpoint => self.path_endpoints.push(Coord2::xy(x as i32, y as i32)),
                JigsawPieceTile::Fixed { ground, object, statue_spot, connection: _ } => {
                    let ground_id = resources.tiles.validate_id(ground).unwrap();
                    let object_id = object.and_then(|object| resources.object_tiles.validate_id(object - 1));
                    let filtered = filter.filter(Coord2::xy(x as i32, y as i32), &ground_id, object_id);
                    if let Some(filtered) = filtered {
                        self.chunk.ground_layer.set_tile(x, y, filtered.as_usize());
                    } else {
                        self.chunk.ground_layer.set_tile(x, y, ground);
                        if let Some(object) = object {
                            self.chunk.set_object_idx(Coord2::xy(x as i32, y as i32), object);
                        }
                        if statue_spot {
                            self.statue_spots.push(Coord2::xy(x as i32, y as i32))
                        }
                    }
                },
            }
        }
    }

}