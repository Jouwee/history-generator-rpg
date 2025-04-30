use crate::engine::geometry::Size2D;


pub(crate) struct JigsawSolver {

}

pub(crate) struct JigsawPiecePool {

}

pub(crate) struct JigsawPiece {
    pub(crate) size: Size2D,
    pub(crate) tiles: Vec<JigsawPieceTile>
}


impl JigsawPiece {

    fn parse(size: Size2D, string: &str) -> JigsawPiece {
        let mut tiles = Vec::new();
        for char in string.chars() {
            match char {
                '#' => tiles.push(JigsawPieceTile::Fixed(1)),
                'C' => tiles.push(JigsawPieceTile::Connection),
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

}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum JigsawPieceTile {
    Air,
    Empty,
    Fixed(usize),
    Connection
}

impl JigsawPieceTile {
    fn can_override(&self, another: &JigsawPieceTile) -> bool {
        if *self == JigsawPieceTile::Air || *another == JigsawPieceTile::Air {
            return false
        }
        if *self == JigsawPieceTile::Empty || *another == JigsawPieceTile::Empty {
            return true
        }
        return *another == *self
    }
}