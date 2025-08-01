use crate::{commons::rng::Rng, Coord2};

use super::jigsaw_structure_generator::JigsawPieceTile;

pub(crate) trait StructureFilter {

    fn filter(&mut self, position: Coord2, tile: &JigsawPieceTile) -> Option<JigsawPieceTile>;

}


pub(crate) struct NoopFilter {}

impl StructureFilter for NoopFilter {

    fn filter(&mut self, _position: Coord2, _tile: &JigsawPieceTile) -> Option<JigsawPieceTile> {
        return None;
    }

}

pub(crate) struct AbandonedStructureFilter {
    rng: Rng,
    age: u32
}

impl AbandonedStructureFilter {

    pub(crate) fn new(rng: Rng, age: u32) -> Self {
        Self { rng, age }
    }

}

impl StructureFilter for AbandonedStructureFilter {

    fn filter(&mut self, _position: Coord2, _tile: &JigsawPieceTile) -> Option<JigsawPieceTile> {
        if self.rng.rand_chance((self.age as f32 / 500.).clamp(0.0, 0.9)) {
            return Some(JigsawPieceTile::Air);
        }
        return None;
    }

}