/*
[foyer]
any

[foyer_a]
$template=
######
#c   #
#Tc  #
#c   #
#    #
###O##
@o=entrance
@#={tag=wall}
@T={tag=table}
@c={tag=chair} 
 */

// https://www.youtube.com/watch?v=b6eBndQ_jK0&t=433s

use core::panic;
use std::{cmp::Ordering, collections::HashSet, time::Instant};

use noise::{NoiseFn, Perlin};

use crate::{commons::rng::Rng, engine::geometry::Size2D, world::{creature::Profession, unit::Unit, world::World}, Actor, Chunk, Coord2, Resources};

use super::{jigsaw_parser::JigsawParser, jigsaw_structure_generator::{JigsawPiece, JigsawPiecePool, JigsawPieceTile, JigsawSolver}};

pub(crate) struct ChunkGenerator {
    chunk: Chunk
}

impl ChunkGenerator {

    pub(crate) fn new(resources: &Resources, player: Actor, size: Size2D) -> ChunkGenerator {
        ChunkGenerator { chunk: Chunk::new(size, player, resources) }
    }

    pub(crate) fn generate(&mut self, world: &World, xy: Coord2, resources: &Resources) {
        let now = Instant::now();
        self.generate_fixed_terrain_features();
        println!("[Chunk gen] Terrain: {:.2?}", now.elapsed());
        let now = Instant::now();
        self.generate_large_structures();
        println!("[Chunk gen] Large structs: {:.2?}", now.elapsed());

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
            println!("Chunk has {} creatures", unit.creatures.len());
            let now = Instant::now();
            self.generate_buildings(&unit, world, resources);
            println!("[Chunk gen] Building gen: {:.2?}", now.elapsed());
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

    fn generate_large_structures(&mut self) {
        //...
    }

    fn generate_buildings(&mut self, unit: &Unit, world: &World, resources: &Resources) {
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

        let mut solver = Self::get_jigsaw_solver();

        while homeless.len() > 0 {
            // TODO:
            if j > 100 {
                break;
            }
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
                panic!("No position found")
            }
            let collapsed_pos = collapsed_pos.unwrap();


            let mut lx = 0;
            let mut ly = 0;

            for creature_id in family.iter() {

                let creature = world.get_creature(creature_id);
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

    fn get_jigsaw_solver() -> JigsawSolver {
        let mut solver = JigsawSolver::new(Size2D(64, 64));
        let parser = JigsawParser::new("assets/structures/village.toml");

        parser.parse(&mut solver);

        return solver;
    }

    fn collapse_decor(&mut self) {
        // TODO: Deterministic
        let mut rng = Rng::rand();
        let noise = Perlin::new(Rng::rand().derive("trees").seed());
        for x in 1..self.chunk.size.x()-1 {
            for y in 1..self.chunk.size.y()-1 {
                if noise.get([x as f64 / 15.0, y as f64 / 15.0]) > 0. {
                    if let Some(ground) = self.chunk.map.ground_layer.tile(x, y) {
                        if ground == 1 && rng.rand_chance(0.1) {
                            self.chunk.map.object_layer.set_tile(x as usize, y as usize, 2);
                        }
                    }
                }
            }
        }
    }

    fn place_template(&mut self, origin: Coord2, template: &JigsawPiece) {
        for i in 0..template.size.area() {
            let x = origin.x as usize + i % template.size.x();
            let y = origin.y as usize + i / template.size.x();
            let tile = template.tiles.get(i).unwrap();
            match tile {
                JigsawPieceTile::Air => (),
                JigsawPieceTile::Empty => (),
                JigsawPieceTile::Connection(_) => self.chunk.map.ground_layer.set_tile(x, y, 4),
                JigsawPieceTile::Fixed { ground, object } => {
                    self.chunk.map.ground_layer.set_tile(x, y, *ground);
                    if let Some(object) = object {
                        self.chunk.map.object_layer.set_tile(x, y, *object)
                    }
                },
            }
        }
    }

}
