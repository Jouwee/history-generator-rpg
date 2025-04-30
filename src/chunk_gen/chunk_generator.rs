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

use std::time::Instant;

use noise::{NoiseFn, Perlin};

use crate::{commons::rng::Rng, engine::geometry::Size2D, world::{creature::Profession, unit::Unit, world::World}, Actor, Chunk, Coord2, Resources};

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

        let room = Template::parse(Size2D(5, 7), 
        "#####|#___#|#___C|#___#|C___#|#___#|##_##");

        let mut x = 64;
        let mut y = 64;

        let mut dir = 0;
        let mut i = 0;
        let mut inc = 1;
        let step = 9;

        let mut j = 0;

        while homeless.len() > 0 {
            // TODO:
            if j > 100 {
                break;
            }
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



            // println!("{} {}", x, y);
            self.place_template(Coord2::xy(x, y), &room);

            let mut lx = 0;
            let mut ly = 0;

            for creature_id in family.iter() {

                let creature = world.get_creature(creature_id);
                let point = Coord2::xy(x + lx + 1, y + ly + 1);
                let species = resources.species.get(&creature.species);
                self.chunk.npcs.push(Actor::from_creature(point, *creature_id, &creature, &creature.species, &species, world));
                lx += 1;
                if lx >= 3 {
                    lx = 0;
                    ly += 1;
                }

            }

            // TODO: Spiral
            match dir {
                0 => x = x + step,
                1 => y = y - step,
                2 => x = x - step,
                3 => y = y + step,
                _ => ()
            };
            i = i + 1;
            if i == inc {
                i = 0;
                if dir == 1 || dir == 3 {
                    inc = inc + 1;
                }
                dir = (dir + 1) % 4;
            }

        }
        

        // TODO: Use Jigsaw
        // let entrance = Template::parse(Size2D(5, 7), 
        // "#####|#___#|#___C|#___#|C___#|#___#|##_##");
        // let room_a = Template::parse(Size2D(4, 5), 
        // "####|#__#|#__C|#__#|####");
        // let room_b = Template::parse(Size2D(4, 5), 
        // "####|#__#|C__#|#__#|####");

        // // TODO: Several buildings

        // // TODO: Allow for several starts
        // let templates = &vec!(entrance.clone(), room_a, room_b);
        // let start = entrance;

        // let templates = self.solve_jigsaw(Coord2::xy(20, 15), start, &templates, 5);
        // for (origin, template) in templates.vec {
        //     self.place_template(origin, &template);
        // }
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

    fn solve_jigsaw(&self, origin: Coord2, starting_template: Template, templates: &Vec<Template>, max_depth: usize) -> Structure {
        let mut vec = Structure::new();
        vec.add(&starting_template, origin);
        let result = self.recursive_jigsaw(vec.clone(), templates, 1, max_depth);
        if let Some(structure) = result {
            return structure
        }
        return vec;
    }

    fn recursive_jigsaw(&self, vec: Structure, templates: &Vec<Template>, depth: usize, max_depth: usize) -> Option<Structure> {
        if vec.open_connections.len() == 0 {
            return Some(vec)
        }

        if depth >= max_depth {
            return None;
        }

        let mut possibilities = Vec::new();
        for connection in vec.open_connections.iter() {
            for template in templates.iter() {
                let connectors = vec.template_fits(&template, &connection);
                for connector in connectors.iter() {
                    possibilities.push((connection, template, *connector));   
                }
            }
        }
        // TODO: Deterministic
        let mut rng = Rng::rand();
        let possibilities = rng.shuffle(possibilities);
        for possibility in possibilities.iter() {
            let mut state_clone = vec.clone();
            let origin = *possibility.0 - possibility.2;
            state_clone.add(possibility.1, origin);
            state_clone.remove_connection(possibility.0);
            let result = self.recursive_jigsaw(state_clone, &templates, depth + 1, max_depth);
            if result.is_some() {
                return result;
            }
        }
        return None
    }

    fn place_template(&mut self, origin: Coord2, template: &Template) {
        for i in 0..template.size.area() {
            let x = origin.x as usize + i % template.size.x();
            let y = origin.y as usize + i / template.size.x();
            let tile = template.tiles.get(i).unwrap();
            match tile {
                TemplateTile::Air => self.chunk.map.ground_layer.set_tile(x, y, 4),
                TemplateTile::Empty => (),
                TemplateTile::Connection => self.chunk.map.ground_layer.set_tile(x, y, 4),
                TemplateTile::Fixed(tile_id) => {
                    self.chunk.map.ground_layer.set_tile(x, y, 4);
                    self.chunk.map.object_layer.set_tile(x, y, *tile_id)
                },
            }
        }
    }

}

#[derive(Clone)]
pub(crate) struct Structure {
    // TODO: Turn to reference
    pub(crate) vec: Vec<(Coord2, Template)>,
    pub(crate) open_connections: Vec<Coord2>
}

impl Structure {

    pub(crate) fn new() -> Structure {
        Structure { vec: vec!(), open_connections: vec!() }
    }

    pub(crate) fn add(&mut self, template: &Template, origin: Coord2) {
        let mut first_c = self.vec.len() > 0;
        self.vec.push((origin, template.clone()));
        for (i, tile) in template.tiles.iter().enumerate() {
            if let TemplateTile::Connection = tile {
                // TODO: Actually check if open
                if first_c {
                    first_c = false;
                    continue;
                }
                let x = origin.x as usize + i % template.size.x();
                let y = origin.y as usize + i / template.size.x();
                self.open_connections.push(Coord2::xy(x as i32, y as i32));
            }
        }
    }

    fn remove_connection(&mut self, connection: &Coord2) {
        let index = self.open_connections.iter().position(|c| c == connection);
        if let Some(index) = index {
            self.open_connections.remove(index);
        }
    }

    pub(crate) fn template_fits(&self, template: &Template, connection: &Coord2) -> Vec<Coord2> {
        let mut vec = Vec::new();
        'candidate: for (i, tile) in template.tiles.iter().enumerate() {
            if let TemplateTile::Connection = tile {
                let cx = i % template.size.x();
                let cy = i / template.size.x();
                for (j, tile) in template.tiles.iter().enumerate() {
                    let x = connection.x - cx as i32 + (j % template.size.x()) as i32;
                    let y = connection.y - cy as i32 + (j / template.size.x()) as i32;

                    for (origin, template) in self.vec.iter() {
                        for (k, tile_b) in template.tiles.iter().enumerate() {
                            let x2 = origin.x + (k % template.size.x()) as i32;
                            let y2 = origin.y + (k / template.size.x()) as i32;
                            if x == x2 && y == y2 && !tile.can_override(tile_b) {
                                continue 'candidate;
                            }
                        }
                    }

                }
                vec.push(Coord2::xy(cx as i32, cy as i32));
            }
        }
        return vec;
    }

}


#[cfg(test)]
mod tests_structure {
    use super::*;

    #[test]
    fn structure_add() {

        let mut structure = Structure::new();
        let t_box = Template::parse(Size2D(3, 3),   "###|#_#|###");

        structure.add(&t_box, Coord2::xy(0, 0));

        assert_eq!(structure.vec.len(), 1);
        assert_eq!(structure.open_connections.len(), 0);

    }
}

#[derive(Clone)]
pub(crate) struct Template {
    pub(crate) size: Size2D,
    pub(crate) tiles: Vec<TemplateTile>
}

impl Template {

    fn parse(size: Size2D, string: &str) -> Template {
        let mut tiles = Vec::new();
        for char in string.chars() {
            match char {
                '#' => tiles.push(TemplateTile::Fixed(1)),
                'C' => tiles.push(TemplateTile::Connection),
                '_' => tiles.push(TemplateTile::Air),
                '.' => tiles.push(TemplateTile::Empty),
                _ => ()
            }
        }
        Template {
            size,
            tiles
        }
    }

}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum TemplateTile {
    Air,
    Empty,
    Fixed(usize),
    Connection
}

impl TemplateTile {
    fn can_override(&self, another: &TemplateTile) -> bool {
        if *self == TemplateTile::Air || *another == TemplateTile::Air {
            return false
        }
        if *self == TemplateTile::Empty || *another == TemplateTile::Empty {
            return true
        }
        return *another == *self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn recursive_jigsaw_1_room() {
        // TODO:
        // let generator = ChunkGenerator {};

        // let t_box = Template::parse(Size2D(3, 3), 
        // "###|#_#|###");
        // let t_connect_left = Template::parse(Size2D(3, 3), 
        // "###|#_C|###");
        // let t_connect_right = Template::parse(Size2D(3, 3), 
        // "###|C_#|###");
        // let t_l_down = Template::parse(Size2D(3, 3), 
        // "###|C_#|#C#");
        // let t_c_up = Template::parse(Size2D(3, 3), 
        // "#C#|#_#|###");

        // let templates = vec!(
        //     t_box.clone(),
        //     t_connect_left.clone(),
        //     t_connect_right.clone()
        // );


        // let mut start = Structure::new();
        // start.add(&t_box, Coord2::xy(10, 10));
        // let structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        // assert_eq!(structure.is_some(), true);
        // let structure = structure.unwrap();
        // assert_eq!(structure.vec.len(), 1);

        // let mut start = Structure::new();
        // start.add(&t_connect_left, Coord2::xy(10, 10));
        // let structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        // assert_eq!(structure.is_some(), true);
        // let structure = structure.unwrap();
        // assert_eq!(structure.vec.len(), 2);
        // assert_eq!(structure.vec[1].0, Coord2::xy(12, 10));

        // let templates = vec!(
        //     t_connect_left.clone(),
        //     t_l_down.clone(),
        //     t_c_up
        // );

        // let mut start = Structure::new();
        // start.add(&t_connect_left, Coord2::xy(10, 10));
        // let structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        // assert_eq!(structure.is_some(), true);
        // let structure = structure.unwrap();
        // assert_eq!(structure.vec.len(), 3);
        // assert_eq!(structure.vec[1].0, Coord2::xy(12, 10));
        // assert_eq!(structure.vec[2].0, Coord2::xy(12, 12));
    }

    #[test]
    fn benchmark() {
        // TODO:
        // let generator = ChunkGenerator {};

        // let t_connect_left = Template::parse(Size2D(3, 3), 
        // "###|#_C|###");
        // let t_l_down = Template::parse(Size2D(3, 3), 
        // "###|C_#|#C#");
        // let t_c_up = Template::parse(Size2D(3, 3), 
        // "#C#|#_#|###");

        // let templates = vec!(
        //     t_connect_left.clone(),
        //     t_l_down.clone(),
        //     t_c_up
        // );

        // let now = Instant::now();
        // for _ in 0..1000 {
        //     let mut start = Structure::new();
        //     start.add(&t_connect_left, Coord2::xy(10, 10));
        //     let _structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        // }
        // assert_eq!(false, false);
        // println!("Bench jigsaw: {:.2?}", now.elapsed());
    }

}
