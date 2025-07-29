use std::collections::HashMap;

use crate::{commons::rng::Rng, engine::geometry::Size2D, Coord2};


pub(crate) struct JigsawSolver {
    rng: Rng,
    size: Size2D,
    available_pools: HashMap<String, JigsawPiecePool>,
    structures: Vec<Structure>
}

impl JigsawSolver {

    pub(crate) fn new(size: Size2D, rng: Rng) -> Self {
        Self { 
            rng,
            size,
            available_pools: HashMap::new(),
            structures: Vec::new()
        }
    }

    pub(crate) fn add_pool(&mut self, name: &str, pool: JigsawPiecePool) {
        self.available_pools.insert(String::from(name), pool);
    }

    pub(crate) fn solve_structure(&mut self, starter_pool: &str, position: Coord2, rng: &mut Rng) -> Option<&Structure> {
        let pool = self.available_pools.get(starter_pool).expect("Invalid pool");
        let options = pool.pieces.values().collect();
        let options = rng.shuffle(options);
        
        for selected in options.iter() {

            if !self.can_place(position, &selected.size) {
                continue;
            }

            let mut structure = Structure::new();
            structure.add(&selected, position);

            // TODO: Param
            let result = self.recursive_jigsaw(structure, 1, 5, self.rng.clone());

            if result.is_none() {
                continue;
            }


            self.structures.push(result.unwrap());
            
            return Some(self.structures.last().unwrap());
        }
        return None;
    }

    fn recursive_jigsaw(&self, vec: Structure, depth: usize, max_depth: usize, mut rng: Rng) -> Option<Structure> {
        if vec.open_connections.len() == 0 {
            return Some(vec)
        }

        if depth >= max_depth {
            return None;
        }

        let mut possibilities = Vec::new();
        for connection in vec.open_connections.iter() {
            // TODO: Unwrap
            let pool = self.available_pools.get(&connection.1).unwrap();
            for template in pool.pieces.values() {
                let connectors = vec.template_fits(&template, &connection.0);
                for connector in connectors.iter() {
                    possibilities.push((connection, template, *connector));   
                }
            }
        }
        let possibilities = rng.shuffle(possibilities);
        for possibility in possibilities.iter() {
            let origin = possibility.0.0 - possibility.2;
            if self.can_place(origin, &possibility.1.size) {
                let mut state_clone = vec.clone();
                state_clone.add(possibility.1, origin);
                state_clone.remove_connection(&possibility.0.0);
                let result = self.recursive_jigsaw(state_clone, depth + 1, max_depth, rng.clone());
                if result.is_some() {
                    return result;
                }
            }
        }
        return None
    }

    pub(crate) fn is_occupied(&self, coord: Coord2) -> bool {
        return self.can_place(coord, &Size2D(1, 1))
    }

    fn can_place(&self, position: Coord2, size: &Size2D) -> bool {
        let rect_a = [position.x, position.y, position.x + size.x() as i32, position.y + size.y() as i32];
        if rect_a[0] < 0 || rect_a[1] < 0 || rect_a[2] >= self.size.x() as i32 || rect_a[3] >= self.size.y() as i32 {
            return false;
        }
        for structure in self.structures.iter() {
            for (position, piece) in structure.vec.iter() {
                let rect_b = [position.x, position.y, position.x + piece.size.x() as i32, position.y + piece.size.y() as i32];
                if rect_a[0] <= rect_b[2] && rect_a[2] >= rect_b[0] && rect_a[1] <= rect_b[3] && rect_a[3] >= rect_b[1] {
                    return false;
                }
            }
        }
        return true;
    }

}


#[cfg(test)]
mod tests_jigsaw_solver {
    use crate::engine::geometry::Size2D;
    use super::*;

    fn parse(size: Size2D, string: &str) -> JigsawPiece {
        let mut tiles = Vec::new();
        for char in string.chars() {
            match char {
                '#' => tiles.push(JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None }),
                'A' => tiles.push(JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: Some(String::from("A")) }),
                'B' => tiles.push(JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: Some(String::from("B")) }),
                'C' => tiles.push(JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: Some(String::from("C")) }),
                '_' => tiles.push(JigsawPieceTile::Air),
                '.' => tiles.push(JigsawPieceTile::Empty),
                _ => ()
            }
        }
        JigsawPiece {
            size,
            tiles
        }
    }

    #[test]
    fn test_single_buildings() {
        let mut solver = JigsawSolver::new(Size2D(64, 64), Rng::rand());
        let mut pool = JigsawPiecePool::new();
        pool.add_piece("a.1", parse(Size2D(3, 3), "###|#_#|###"));
        solver.add_pool("A", pool);

        let mut rng = Rng::seeded(0);

        // Can add the first building
        let result = solver.solve_structure("A", Coord2::xy(10, 10), &mut rng);
        assert_eq!(result.is_some(), true);

        // Can't add another one if overlaps
        let result = solver.solve_structure("A", Coord2::xy(10, 10), &mut rng);
        assert_eq!(result.is_some(), false);

        // Still overlaps
        let result = solver.solve_structure("A", Coord2::xy(12, 12), &mut rng);
        assert_eq!(result.is_some(), false);

        // This is ok
        let result = solver.solve_structure("A", Coord2::xy(14, 14), &mut rng);
        assert_eq!(result.is_some(), true);

    }

    #[test]
    fn test_unique_connections() {
        let mut solver = JigsawSolver::new(Size2D(64, 64), Rng::rand());

        let mut pool = JigsawPiecePool::new();
        pool.add_piece("a.1", parse(Size2D(3, 3), "###|#_B|###"));
        solver.add_pool("A", pool);

        let mut pool = JigsawPiecePool::new();
        pool.add_piece("b.1", parse(Size2D(3, 3), "###|A_#|###"));
        solver.add_pool("B", pool);

        let mut rng = Rng::seeded(0);

        // Can add the 2 piece building
        let result = solver.solve_structure("A", Coord2::xy(10, 10), &mut rng);
        assert_eq!(result.is_some(), true);
        let structure = result.unwrap();
        assert_eq!(structure.vec.len(), 2);

        // The subpiece can't overlap existing structures
        let result = solver.solve_structure("A", Coord2::xy(5, 10), &mut rng);
        assert_eq!(result.is_some(), false);

    }

}

pub(crate) struct JigsawPiecePool {
    pub(crate) pieces: HashMap<String, JigsawPiece>
}

impl JigsawPiecePool {

    pub(crate) fn new() -> Self {
        JigsawPiecePool {
            pieces: HashMap::new()
        }
    }

    pub(crate) fn add_piece(&mut self, name: &str, piece: JigsawPiece) {
        self.pieces.insert(String::from(name), piece);
    }

}

#[derive(Clone)]
pub(crate) struct JigsawPiece {
    pub(crate) size: Size2D,
    pub(crate) tiles: Vec<JigsawPieceTile>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JigsawPieceTile {
    Air,
    Empty,
    Fixed {
        ground: usize,
        object: Option<usize>,
        statue_spot: bool,
        connection: Option<String>
    },
    PathEndpoint,
}

impl JigsawPieceTile {
    fn can_override(&self, another: &JigsawPieceTile) -> bool {
        if *self == JigsawPieceTile::Air || *another == JigsawPieceTile::Air {
            return false
        }
        if *self == JigsawPieceTile::Empty || *another == JigsawPieceTile::Empty {
            return true
        }
        if let JigsawPieceTile::Fixed { ground: _, object: _, statue_spot: _, connection: Some(_) } = *self {
            if let JigsawPieceTile::Fixed { ground: _, object: _, statue_spot: _, connection: Some(_) } = *another {
                return true
            }
        }
        return *another == *self
    }
}


#[derive(Clone)]
pub(crate) struct Structure {
    // TODO: Turn to reference
    pub(crate) vec: Vec<(Coord2, JigsawPiece)>,
    pub(crate) open_connections: Vec<(Coord2, String)>
}

impl Structure {

    pub(crate) fn new() -> Structure {
        Structure { vec: vec!(), open_connections: vec!() }
    }

    pub(crate) fn add(&mut self, template: &JigsawPiece, origin: Coord2) {
        let mut first_c = self.vec.len() > 0;
        self.vec.push((origin, template.clone()));
        for (i, tile) in template.tiles.iter().enumerate() {
            if let JigsawPieceTile::Fixed { ground: _, object: _, statue_spot: _, connection: Some(pool) } = tile {
                // TODO: Actually check if open
                if first_c {
                    first_c = false;
                    continue;
                }
                let x = origin.x as usize + i % template.size.x();
                let y = origin.y as usize + i / template.size.x();
                self.open_connections.push((Coord2::xy(x as i32, y as i32), pool.clone()));
            }
        }
    }

    fn remove_connection(&mut self, connection: &Coord2) {
        let index = self.open_connections.iter().position(|c| c.0 == *connection);
        if let Some(index) = index {
            self.open_connections.remove(index);
        }
    }

    pub(crate) fn template_fits(&self, template: &JigsawPiece, connection: &Coord2) -> Vec<Coord2> {
        let mut vec = Vec::new();
        'candidate: for (i, tile) in template.tiles.iter().enumerate() {
            if let JigsawPieceTile::Fixed { ground: _, object: _, statue_spot: _, connection: Some(_) } = tile {
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