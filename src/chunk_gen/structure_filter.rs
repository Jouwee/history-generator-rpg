use crate::{commons::{id_vec::Id, rng::Rng}, resources::resources::resources, Coord2};

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
    age: u32,
}

impl AbandonedStructureFilter {

    pub(crate) fn new(rng: Rng, age: u32) -> Self {
        Self { rng, age }
    }

}

impl StructureFilter for AbandonedStructureFilter {

    fn filter(&mut self, _position: Coord2, tile: &JigsawPieceTile) -> Option<JigsawPieceTile> {
        let resources = resources();
        match tile {
            JigsawPieceTile::Fixed { ground, object, statue_spot: _, connection: _ } => {
                // TODO: Bring decay chance from resources
                
                if let Some(object) = object {
                    let object = object - 1;
                    if object == resources.object_tiles.id_of("obj:wall").as_usize() {
                        if self.rng.rand_chance((self.age as f32 / 150.).clamp(0.0, 0.9)) {
                            return Some(JigsawPieceTile::Air)
                        } else {
                            return None;
                        }
                    }
                    if object == resources.object_tiles.id_of("obj:bed").as_usize() || object == resources.object_tiles.id_of("obj:table").as_usize() || object == resources.object_tiles.id_of("obj:stool").as_usize() || object == resources.object_tiles.id_of("obj:barrel").as_usize() || object == resources.object_tiles.id_of("obj:chair").as_usize() || object == resources.object_tiles.id_of("obj:tent").as_usize() {
                        return Some(JigsawPieceTile::Air)
                    }
                }
                let ground = *ground;
                if ground == resources.tiles.id_of("tile:floor").as_usize() || ground == resources.tiles.id_of("tile:cobblestone").as_usize() || ground == resources.tiles.id_of("tile:carpet_red").as_usize() {
                    if self.rng.rand_chance((self.age as f32 / 50.).clamp(0.0, 0.9)) {
                        return Some(JigsawPieceTile::Air)
                    }
                }
            },
            _ => ()
        }
        None
    }

}