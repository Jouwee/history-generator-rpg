use std::{collections::HashMap, fs};

use toml::{Table, Value};

use crate::{chunk_gen::jigsaw_structure_generator::JigsawPieceTile, engine::geometry::Size2D};

use super::jigsaw_structure_generator::{JigsawPiece, JigsawPiecePool};

pub(crate) struct JigsawParser {
}

// TODO: Refactor
impl JigsawParser {

    pub(crate) fn new() -> Self {
        return Self {
        }
    }

    pub(crate) fn parse_file(&self, path: &str) -> Result<Vec<(String, JigsawPiecePool)>, ()> {
        let contents = fs::read_to_string(path).unwrap();
        self.parse_string(contents)
    }

    pub(crate) fn parse_string(&self, contents: String) -> Result<Vec<(String, JigsawPiecePool)>, ()> {
        // TODO: Review errors
        let value = contents.parse::<Table>().unwrap();

        let mut pools = Vec::new();

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

                            let flip_horizontal = match piece_toml.get("allow_flip_hor") {
                                Some(Value::Boolean(true)) => true,
                                _ => false
                            };

                            let flip_vertical = match piece_toml.get("allow_flip_ver") {
                                Some(Value::Boolean(true)) => true,
                                _ => false
                            };

                            let rotate = match piece_toml.get("allow_rotate") {
                                Some(Value::Boolean(true)) => true,
                                _ => false
                            };

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
                                                        name: pool_name.clone() + piece_name,
                                                        size,
                                                        tiles
                                                    };

                                                    if flip_horizontal {
                                                        pool.add_piece(&(piece_name.clone() + "_flip_hor"), flip_piece_horizontally(&piece));
                                                    }
                                                    if flip_vertical {
                                                        pool.add_piece(&(piece_name.clone() + "_flip_ver"), flip_piece_vertically(&piece));
                                                    }
                                                    if rotate {
                                                        pool.add_piece(&(piece_name.clone() + "_r90"), flip_piece_horizontally(&transpose_piece(&piece)));
                                                        pool.add_piece(&(piece_name.clone() + "_r270"), flip_piece_vertically(&transpose_piece(&piece)));
                                                        pool.add_piece(&(piece_name.clone() + "_r180"), flip_piece_horizontally(&flip_piece_vertically(&piece)));
                                                    }

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

                pools.push((pool_name.clone(), pool));
            }
        }


        return Ok(pools)
    }

    fn parse_symbols(table: &Table) -> HashMap<char, JigsawPieceTile> {
        let mut map = HashMap::new();
        for (symbol, tile) in table.iter() {
            let symbol = symbol.chars().next().unwrap();
            match tile {
                Value::Table(tile_table) => {
                    if let Some(Value::Integer(ground)) = tile_table.get("ground") {
                        let object = match tile_table.get("object") {
                            Some(Value::Integer(object)) => Some(*object as usize),
                            _ => None
                        };
                        let statue_spot = match tile_table.get("statue_spot") {
                            Some(Value::Boolean(true)) => true,
                            _ => false
                        };
                        let connection = match tile_table.get("connect") {
                            Some(Value::String(connection)) => Some(connection.clone()),
                            _ => None
                        };
                        map.insert(symbol, JigsawPieceTile::Fixed { ground: *ground as usize, object, statue_spot, connection });
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

fn flip_piece_horizontally(piece: &JigsawPiece) -> JigsawPiece {
    let mut tiles = Vec::new();
    for i in 0..piece.size.area() {
        let x_flipped = piece.size.0 - (i % piece.size.0) - 1;
        let y = i / piece.size.0;
        let j = (y * piece.size.0) + x_flipped;
        tiles.push(piece.tiles.get(j).unwrap().clone());
    }
    return JigsawPiece {
        name: piece.name.clone(),
        size: piece.size.clone(),
        tiles
    };
}

fn flip_piece_vertically(piece: &JigsawPiece) -> JigsawPiece {
    let mut tiles = Vec::new();
    for i in 0..piece.size.area() {
        let x = i % piece.size.0;
        let y_flipped = piece.size.1 - (i / piece.size.0) - 1;
        let j = (y_flipped * piece.size.0) + x;
        tiles.push(piece.tiles.get(j).unwrap().clone());
    }
    return JigsawPiece {
        name: piece.name.clone(),
        size: piece.size.clone(),
        tiles
    };
}

fn transpose_piece(piece: &JigsawPiece) -> JigsawPiece {
    let mut tiles = Vec::new();
    for i in 0..piece.size.area() {
        let y = i % piece.size.1;
        let x = i / piece.size.1;
        let j = (y * piece.size.0) + x;
        tiles.push(piece.tiles.get(j).unwrap().clone());
    }
    return JigsawPiece {
        name: piece.name.clone(),
        size: Size2D(piece.size.1, piece.size.0),
        tiles
    };
}

#[cfg(test)]
mod tests_jigsaw_parses {
    use super::*;

    #[test]
    fn test_simplest_template() {

        let parser = JigsawParser::new();

        let result = parser.parse_string("
[symbols]
'a' = { ground= 1, object= 1 }

[template]
[template.var_a]
size = [1, 1]
template = \"\"\"
a
\"\"\"
".to_string());

        assert_eq!(result.is_ok(), true);
        let pools = result.unwrap();
        assert_eq!(pools.len(), 1);
        let (name, pool) = pools.get(0).unwrap();
        assert_eq!(name, "template");
        assert_eq!(pool.pieces.len(), 1);
        assert_eq!(pool.pieces.contains_key("var_a"), true);
        let piece = pool.pieces.get("var_a").unwrap();
        assert_eq!(piece.size, Size2D(1, 1));
        assert_eq!(piece.tiles.len(), 1);
        let tile = piece.tiles.get(0).unwrap();
        assert_eq!(tile.clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });


    }

    #[test]
    fn test_flip() {

        let parser = JigsawParser::new();

        // Horizontal flip
        let result = parser.parse_string("
[symbols]
'a' = { ground= 1, object= 1 }
'b' = { ground= 2 }

[template]
[template.var_a]
size = [3, 2]
allow_flip_hor = true
template = \"\"\"
a.b
b.a
\"\"\"
".to_string());

        assert_eq!(result.is_ok(), true);
        let pools = result.unwrap();
        let (_, pool) = pools.get(0).unwrap();
        assert_eq!(pool.pieces.len(), 2);
        assert_eq!(pool.pieces.contains_key("var_a"), true);
        assert_eq!(pool.pieces.contains_key("var_a_flip_hor"), true);

        let piece = pool.pieces.get("var_a").unwrap();
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });

        let piece = pool.pieces.get("var_a_flip_hor").unwrap();
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });

        // Vertical flip
        let result = parser.parse_string("
[symbols]
'a' = { ground= 1, object= 1 }
'b' = { ground= 2 }

[template]
[template.var_a]
size = [3, 2]
allow_flip_ver = true
template = \"\"\"
a.b
b.a
\"\"\"
".to_string());

        assert_eq!(result.is_ok(), true);
        let pools = result.unwrap();
        let (_, pool) = pools.get(0).unwrap();
        assert_eq!(pool.pieces.len(), 2);
        assert_eq!(pool.pieces.contains_key("var_a"), true);
        assert_eq!(pool.pieces.contains_key("var_a_flip_ver"), true);

        let piece = pool.pieces.get("var_a").unwrap();
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });

        let piece = pool.pieces.get("var_a_flip_ver").unwrap();
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: Some(1), statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });

    }

    #[test]
    fn test_rotate() {
        let parser = JigsawParser::new();

        // Horizontal flip
        let result = parser.parse_string("
[symbols]
'a' = { ground= 1 }
'b' = { ground= 2 }

[template]
[template.var_a]
size = [3, 2]
allow_rotate = true
template = \"\"\"
a.b
a.b
\"\"\"
".to_string());

        assert_eq!(result.is_ok(), true);
        let pools = result.unwrap();
        let (_, pool) = pools.get(0).unwrap();
        assert_eq!(pool.pieces.len(), 4);
        assert_eq!(pool.pieces.contains_key("var_a"), true);
        assert_eq!(pool.pieces.contains_key("var_a_r90"), true);

        let piece = pool.pieces.get("var_a_r90").unwrap();
        assert_eq!(piece.size, Size2D(2, 3));
        // assert_eq!(piece.debug_string(), "");
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });

        let piece = pool.pieces.get("var_a_r180").unwrap();
        assert_eq!(piece.size, Size2D(3, 2));
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None });

        let piece = pool.pieces.get("var_a_r270").unwrap();
        assert_eq!(piece.size, Size2D(2, 3));
        assert_eq!(piece.tiles.get(0).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(1).unwrap().clone(), JigsawPieceTile::Fixed { ground: 2, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(2).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(3).unwrap().clone(), JigsawPieceTile::Empty);
        assert_eq!(piece.tiles.get(4).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None });
        assert_eq!(piece.tiles.get(5).unwrap().clone(), JigsawPieceTile::Fixed { ground: 1, object: None, statue_spot: false, connection: None });
    }

}