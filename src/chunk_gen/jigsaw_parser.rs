use std::{collections::HashMap, fs};

use toml::{Table, Value};

use crate::{chunk_gen::jigsaw_structure_generator::JigsawPieceTile, engine::geometry::Size2D};

use super::jigsaw_structure_generator::{JigsawPiece, JigsawPiecePool, JigsawSolver};

pub(crate) struct JigsawParser {
    path: String,
}

// TODO: Refactor
impl JigsawParser {

    pub(crate) fn new(path: &str) -> Self {
        return Self {
            path: String::from(path)
        }
    }

    pub(crate) fn parse(self, solver: &mut JigsawSolver) -> Result<(), ()> {
        // TODO: Review errors
        let contents = fs::read_to_string(self.path).unwrap();
        let value = contents.parse::<Table>().unwrap();

        let symbols = value.get("symbols");
        if let Some(Value::Table(symbols)) = symbols {
            let symbols = Self::parse_symbols(symbols);
        
            for (pool_name, pool_toml) in value.iter() {
                if pool_name == "symbols" {
                    continue;
                }
                let mut pool = JigsawPiecePool::new();

                if let Value::Table(pool_toml) = pool_toml {

                    for (piece_name, piece_toml) in pool_toml.iter() {
                        if let Value::Table(piece_toml) = piece_toml {

                            let size = piece_toml.get("size");
                            if let Some(size) = size {
                                if let Value::Array(size) = size {
                                    let x = size.get(0);
                                    let y = size.get(1);
                                    if let (Some(x), Some(y)) = (x, y) {
                                        if let (Value::Integer(x), Value::Integer(y)) = (x, y) {
                                            let size = Size2D(*x as usize, *y as usize);

                                            let template = piece_toml.get("template");
                                            if let Some(template) = template {
                                                if let Value::String(template) = template {
                                                    let mut tiles = Vec::new();
                                                    for char in template.chars() {
                                                        match char {
                                                            // TODO:
                                                            // '#' => tiles.push(JigsawPieceTile::Fixed(1)),
                                                            // '_' => tiles.push(JigsawPieceTile::Air),
                                                            '.' => tiles.push(JigsawPieceTile::Empty),
                                                            c => {
                                                                if let Some(tile) = symbols.get(&c) {
                                                                    tiles.push(tile.clone());
                                                                }
                                                            }
                                                        }
                                                    }

                                                    if tiles.len() != size.area() {
                                                        println!("[ERR] Jigsaw piece {pool_name}.{piece_name} expected {} tiles but has {}", size.area(), tiles.len());
                                                        continue;
                                                    }

                                                    let piece = JigsawPiece {
                                                        size,
                                                        tiles
                                                    };

                                                    pool.add_piece(&piece_name, piece);
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            panic!("invalid piece format")
                        } else {
                            panic!("piece is not a table")
                        }
                    }
                } else {
                    panic!("pool is not a table")
                }

                solver.add_pool(&pool_name, pool);
            }
        }


        return Ok(())
    }

    fn parse_symbols(table: &Table) -> HashMap<char, JigsawPieceTile> {
        let mut map = HashMap::new();
        for (symbol, tile) in table.iter() {
            let symbol = symbol.chars().next().unwrap();
            match tile {
                Value::Table(tile_table) => {
                    if let Some(Value::String(connection)) = tile_table.get("connect") {
                        map.insert(symbol, JigsawPieceTile::Connection(connection.clone()));
                        continue;
                    }
                    if let Some(Value::Integer(ground)) = tile_table.get("ground") {
                        let object = match tile_table.get("object") {
                            Some(Value::Integer(object)) => Some(*object as usize),
                            _ => None
                        };
                        let statue_spot = match tile_table.get("statue_spot") {
                            Some(Value::Boolean(true)) => true,
                            _ => false
                        };
                        map.insert(symbol, JigsawPieceTile::Fixed { ground: *ground as usize, object, statue_spot });
                        continue;
                    }
                    if let Some(Value::Boolean(true)) = tile_table.get("path_endpoint") {
                        map.insert(symbol, JigsawPieceTile::PathEndpoint);
                        continue;
                    }
                },
                _ => ()
            };
        }
        return map
    }

}