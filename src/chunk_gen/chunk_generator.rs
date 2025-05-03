use core::panic;
use std::{cmp::Ordering, collections::HashSet, time::Instant};

use noise::{NoiseFn, Perlin};

use crate::{commons::{astar::{AStar, MovementCost}, rng::Rng}, engine::geometry::Size2D, game::chunk::TileMetadata, world::{creature::Profession, unit::Unit, world::World}, Actor, Chunk, Coord2, Resources};

use super::{jigsaw_parser::JigsawParser, jigsaw_structure_generator::{JigsawPiece, JigsawPieceTile, JigsawSolver}, structure_filter::{AbandonedStructureFilter, NoopFilter, StructureFilter}};

pub(crate) struct ChunkGenerator {
    rng: Rng,
    chunk: Chunk,
    path_endpoints: Vec<Coord2>,
    statue_spots: Vec<Coord2>
}

impl ChunkGenerator {

    pub(crate) fn new(resources: &Resources, player: Actor, size: Size2D) -> ChunkGenerator {
        ChunkGenerator {
            // TODO: Determinate
            rng: Rng::rand(),
            chunk: Chunk::new(size, player, resources),
            path_endpoints: Vec::new(),
            statue_spots: Vec::new(),
        }
    }

    pub(crate) fn generate(&mut self, world: &World, xy: Coord2, resources: &Resources) {
        let now = Instant::now();
        self.generate_fixed_terrain_features();
        println!("[Chunk gen] Terrain: {:.2?}", now.elapsed());

        let now = Instant::now();
        let mut found_sett = None;
        for unit in world.units.iter() {
            let unit = unit.borrow();
            if unit.xy.x as i32 == xy.x && unit.xy.y as i32 == xy.y {
                found_sett = Some(unit)
            }
        }
        println!("[Chunk gen] Unit search: {:.2?}", now.elapsed());

        if let Some(unit) = found_sett {
            let mut solver = self.get_jigsaw_solver();
            let now = Instant::now();
            self.generate_large_structures(&unit, &mut solver);
            println!("[Chunk gen] Large structs: {:.2?}", now.elapsed());

            println!("Chunk has {} creatures, {} artifacts, {} graves. Peak was {} in {}", unit.creatures.len(), unit.artifacts.len(), unit.cemetery.len(), unit.population_peak.1, unit.population_peak.0);
            let now = Instant::now();
            self.generate_buildings(&unit, &mut solver, world, resources);
            println!("[Chunk gen] Building gen: {:.2?}", now.elapsed());

            let now = Instant::now();
            self.generate_ruins(&unit, &mut solver, world, resources);
            println!("[Chunk gen] Ruins gen: {:.2?}", now.elapsed());

            if self.statue_spots.len() > 0 {
                let now = Instant::now();
                self.place_statues(&unit, &world, &resources);
                println!("[Chunk gen] Statues: {:.2?}", now.elapsed());
            }

        }

        if self.path_endpoints.len() > 0 {
            let now = Instant::now();
            self.generate_paths();
            println!("[Chunk gen] Streets: {:.2?}", now.elapsed());
        }

        let now = Instant::now();
        self.collapse_decor();
        println!("[Chunk gen] Decor: {:.2?}", now.elapsed());
    }

    pub(crate) fn into_chunk(self) -> Chunk {
        return self.chunk;
    }

    fn generate_fixed_terrain_features(&mut self) {
        // TODO: Based on region
        for x in 0..self.chunk.size.x() {
            for y in 0..self.chunk.size.y() {
                self.chunk.map.ground_layer.set_tile(x, y, 1);
            }
        }
    }

    fn generate_large_structures(&mut self, unit: &Unit, solver: &mut JigsawSolver) {
        // TODO: Determinate
        let mut rng = Rng::rand();

        let mut building_seed_cloud = HashSet::new();
        for _ in 0..50 {
            building_seed_cloud.insert(Coord2::xy(
                rng.randu_range(0, self.chunk.size.x()) as i32,
                rng.randu_range(0, self.chunk.size.y()) as i32
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

        if unit.artifacts.len() > 0 {

            while building_seed_cloud.len() > 0 {

                let pos = building_seed_cloud.pop().unwrap();

                let structure = solver.solve_structure("village_plaza", pos, &mut rng);
                if let Some(structure) = structure {
                    for (pos, piece) in structure.vec.iter() {
                        self.place_template(*pos, &piece);
                    }
                    break;
                }


            }

           

        }

        if unit.cemetery.len() > 0 {
            let mut collapsed_pos = None;

            while building_seed_cloud.len() > 0 {

                let pos = building_seed_cloud.pop().unwrap();

                let structure = solver.solve_structure("village_cemetery", pos, &mut rng);
                if let Some(structure) = structure {
                    collapsed_pos = Some(pos);
                    for (pos, piece) in structure.vec.iter() {
                        self.place_template(*pos, &piece);
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
                self.chunk.tiles_metadata.insert(Coord2::xy(x, y), TileMetadata::BurialPlace(*creature));


                x = x + 2;
                if x > collapsed_pos.x + 15 {
                    y = y + 2;
                    x = collapsed_pos.x;
                }
            }

        }
    }

    fn generate_buildings(&mut self, unit: &Unit, solver: &mut JigsawSolver, world: &World, resources: &Resources) {
        let mut homeless = unit.creatures.clone();
        // TODO: Determinate
        let mut rng = Rng::rand();

        let mut building_seed_cloud = HashSet::new();
        for _ in 0..1000 {
            building_seed_cloud.insert(Coord2::xy(
                rng.randu_range(0, self.chunk.size.x()) as i32,
                rng.randu_range(0, self.chunk.size.y()) as i32
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
        

        let mut j = 0;

        while homeless.len() > 0 {
            j = j + 1;
            let creature_id = homeless.pop().unwrap();
            let mut family = vec!(creature_id);
            let creature = world.creatures.get(&creature_id);
            if let Some(spouse) = creature.spouse {
                let in_homeless = homeless.iter().position(|id| *id == spouse);
                if let Some(index) = in_homeless {
                    homeless.remove(index);
                    family.push(spouse);
                }
            }
            for child_id in creature.offspring.iter() {
                let in_homeless = homeless.iter().position(|id| *id == *child_id);
                if let Some(index) = in_homeless {
                    let child = world.creatures.get(child_id);
                    if child.profession != Profession::None {
                        homeless.remove(index);
                        family.push(*child_id);
                    }
                }
            }

            let mut collapsed_pos = None;

            while building_seed_cloud.len() > 0 {

                let pos = building_seed_cloud.pop().unwrap();

                let structure = solver.solve_structure("village_house_start", pos, &mut rng);
                if let Some(structure) = structure {
                    collapsed_pos = Some(pos);
                    for (pos, piece) in structure.vec.iter() {
                        self.place_template(*pos, &piece);
                    }
                    break;
                }


            }

            if collapsed_pos.is_none() {
                // TODO: No panic
                println!("No position found" );
                // panic!("No position found");
                continue;
            }
            let collapsed_pos = collapsed_pos.unwrap();


            let mut lx = 0;
            let mut ly = 0;

            for creature_id in family.iter() {

                let creature = world.creatures.get(creature_id);
                let point = Coord2::xy(collapsed_pos.x + lx + 1, collapsed_pos.y + ly + 1);
                let species = resources.species.get(&creature.species);
                self.chunk.npcs.push(Actor::from_creature(point, *creature_id, &creature, &creature.species, &species, world));
                lx += 1;
                if lx >= 3 {
                    lx = 0;
                    ly += 1;
                }

            }

        }
    }

    fn generate_ruins(&mut self, unit: &Unit, solver: &mut JigsawSolver, world: &World, resources: &Resources) {
        let mut building_seed_cloud = HashSet::new();
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
        

        let pop_diff = unit.population_peak.1 as usize - unit.creatures.len();
        let ruins = pop_diff / 2;

        let age = world.date.year() - unit.population_peak.0;

        let mut j = 0;

        for _ in 0..ruins {
            j = j + 1;

            let mut collapsed_pos = None;

            while building_seed_cloud.len() > 0 {

                let pos = building_seed_cloud.pop().unwrap();

                let structure = solver.solve_structure("village_house_start", pos, &mut self.rng);
                if let Some(structure) = structure {
                    collapsed_pos = Some(pos);
                    for (pos, piece) in structure.vec.iter() {
                        self.place_template_filtered(*pos, &piece, AbandonedStructureFilter::new(self.rng.clone(), age as u32));
                    }
                    break;
                }


            }

            if collapsed_pos.is_none() {
                // TODO: No panic
                println!("No position found" );
                // panic!("No position found");
                continue;
            }

        }
    }

    fn generate_paths(&mut self) {
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
                if !self.chunk.size.in_bounds(xy) || self.chunk.map.blocks_movement(xy) {
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

    fn place_statues(&mut self, unit: &Unit, world: &World, resources: &Resources) {
        let spots = self.statue_spots.clone();
        let mut spots = self.rng.shuffle(spots);
        for item in unit.artifacts.iter() {
            let spot = spots.pop();
            match spot {
                None => {
                    // TODO:
                    println!("Not enough spots for artifacts");
                    break;
                },
                Some(spot) => {
                    let item = world.artifacts.get(item);
                    let texture = item.make_texture(&resources.materials);
                    self.chunk.items_on_ground.push((spot, item.clone(), texture));
                }
            }
        }
    }

    fn get_jigsaw_solver(&self) -> JigsawSolver {
        let mut solver = JigsawSolver::new(self.chunk.size.clone());
        let parser = JigsawParser::new("assets/structures/village.toml");

        let _ = parser.parse(&mut solver);

        return solver;
    }

    fn collapse_decor(&mut self) {
        // TODO: Deterministic
        let mut rng = Rng::rand();
        let noise = Perlin::new(Rng::rand().derive("trees").seed());
        for x in 1..self.chunk.size.x()-1 {
            for y in 1..self.chunk.size.y()-1 {
                if let Some(ground) = self.chunk.map.ground_layer.tile(x, y) {
                    if ground == 1 {
                        if noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0. {
                            if rng.rand_chance(0.1) {
                                self.chunk.map.object_layer.set_tile(x as usize, y as usize, 2);
                                continue;
                            }
                        }
                        if rng.rand_chance(0.2) {
                            self.chunk.map.object_layer.set_tile(x as usize, y as usize, 9);
                        }
                    }
                }
            }
        }
    }

    fn place_template(&mut self, origin: Coord2, template: &JigsawPiece) {
        self.place_template_filtered(origin, template, NoopFilter {});
    }

    fn place_template_filtered<F>(&mut self, origin: Coord2, template: &JigsawPiece, mut filter: F) where F: StructureFilter {
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
                JigsawPieceTile::Connection(_) => self.chunk.map.ground_layer.set_tile(x, y, 4),
                JigsawPieceTile::Fixed { ground, object, statue_spot } => {
                    self.chunk.map.ground_layer.set_tile(x, y, ground);
                    if let Some(object) = object {
                        self.chunk.map.object_layer.set_tile(x, y, object)
                    }
                    if statue_spot {
                        self.statue_spots.push(Coord2::xy(x as i32, y as i32))
                    }
                },
            }
        }
    }

}
