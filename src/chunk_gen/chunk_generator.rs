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

use crate::{engine::geometry::Size2D, Actor, Chunk, Coord2, Resources};

pub(crate) struct ChunkGenerator {

}

impl ChunkGenerator {

    pub(crate) fn generate(&self, resources: &Resources, player: Actor) -> Chunk {
        let mut chunk = Chunk::new(Size2D(128, 128), player, resources);

        for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                chunk.map.ground_layer.set_tile(x, y, 1);
            }
        }

        let t_box = Template::parse(Size2D(5, 5), 
        "#####|#___#|#___#|#___#|#####");
        let t_connect_left = Template::parse(Size2D(5, 5), 
        "#####|#___#|C___#|#___#|#####");
        let t_connect_right = Template::parse(Size2D(5, 5), 
        "#####|#___#|#___C|#___#|#####");
        let t_l_down = Template::parse(Size2D(5, 5), 
        ".....|####.|C__#.|##_#.|.#C#.");
        let t_c_up = Template::parse(Size2D(5, 5), 
        "##C##|#___#|#___#|#___#|#####");

        // let t_box = Template::parse(Size2D(3, 3), 
        // "###|#_#|###");
        // let t_connect_left = Template::parse(Size2D(3, 3), 
        // "###|C_#|###");
        // let t_connect_right = Template::parse(Size2D(3, 3), 
        // "###|#_C|###");
        // let t_l_down = Template::parse(Size2D(3, 3), 
        // "###|C_#|#C#");
        // let t_c_up = Template::parse(Size2D(3, 3), 
        // "#C#|#_#|###");

        let templates = &vec!(t_connect_right.clone(), t_c_up, t_l_down);
        let start = t_connect_right;

        let templates = self.solve_jigsaw(Coord2::xy(20, 15), start, &templates, 5);
        for (origin, template) in templates.vec {
            self.place_template(&mut chunk, origin, &template);
        }
        
        chunk
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

        println!("b {}", vec.open_connections.len());

        if vec.open_connections.len() == 0 {
            return Some(vec)
        }


        if depth >= max_depth {
            return None;
        }

        let mut possibilities = Vec::new();
        for connection in vec.open_connections.iter() {
            for template in templates.iter() {
                let connector = vec.template_fits(&template, &connection);
                if let Some(connector) = connector {
                    possibilities.push((connection, template, connector));   
                }
            }
        }

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

    fn place_template(&self, chunk: &mut Chunk, origin: Coord2, template: &Template) {
        for i in 0..template.size.area() {
            let x = origin.x as usize + i % template.size.x();
            let y = origin.y as usize + i / template.size.x();
            let tile = template.tiles.get(i).unwrap();
            match tile {
                TemplateTile::Air => (),
                TemplateTile::Empty => (),
                TemplateTile::Connection => (),
                TemplateTile::Fixed(tile_id) => chunk.map.object_layer.set_tile(x, y, *tile_id),
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

    pub(crate) fn template_fits(&self, template: &Template, connection: &Coord2) -> Option<Coord2> {
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
                // TODO: Could have more
                return Some(Coord2::xy(cx as i32, cy as i32));

            }
        }
        return None;
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

        let generator = ChunkGenerator {};

        let t_box = Template::parse(Size2D(3, 3), 
        "###|#_#|###");
        let t_connect_left = Template::parse(Size2D(3, 3), 
        "###|#_C|###");
        let t_connect_right = Template::parse(Size2D(3, 3), 
        "###|C_#|###");
        let t_l_down = Template::parse(Size2D(3, 3), 
        "###|C_#|#C#");
        let t_c_up = Template::parse(Size2D(3, 3), 
        "#C#|#_#|###");

        let templates = vec!(
            t_box.clone(),
            t_connect_left.clone(),
            t_connect_right.clone()
        );


        let mut start = Structure::new();
        start.add(&t_box, Coord2::xy(10, 10));
        let structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        assert_eq!(structure.is_some(), true);
        let structure = structure.unwrap();
        assert_eq!(structure.vec.len(), 1);

        let mut start = Structure::new();
        start.add(&t_connect_left, Coord2::xy(10, 10));
        let structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        assert_eq!(structure.is_some(), true);
        let structure = structure.unwrap();
        assert_eq!(structure.vec.len(), 2);
        assert_eq!(structure.vec[1].0, Coord2::xy(12, 10));

        let templates = vec!(
            t_connect_left.clone(),
            t_l_down.clone(),
            t_c_up
        );

        let mut start = Structure::new();
        start.add(&t_connect_left, Coord2::xy(10, 10));
        let structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        assert_eq!(structure.is_some(), true);
        let structure = structure.unwrap();
        assert_eq!(structure.vec.len(), 3);
        assert_eq!(structure.vec[1].0, Coord2::xy(12, 10));
        assert_eq!(structure.vec[2].0, Coord2::xy(12, 12));
    }

    #[test]
    fn benchmark() {
        let generator = ChunkGenerator {};

        let t_connect_left = Template::parse(Size2D(3, 3), 
        "###|#_C|###");
        let t_l_down = Template::parse(Size2D(3, 3), 
        "###|C_#|#C#");
        let t_c_up = Template::parse(Size2D(3, 3), 
        "#C#|#_#|###");

        let templates = vec!(
            t_connect_left.clone(),
            t_l_down.clone(),
            t_c_up
        );

        let now = Instant::now();
        for _ in 0..1000 {
            let mut start = Structure::new();
            start.add(&t_connect_left, Coord2::xy(10, 10));
            let _structure = generator.recursive_jigsaw(start, &templates, 1, 3);
        }
        assert_eq!(false, false);
        println!("Bench jigsaw: {:.2?}", now.elapsed());
    }

}
