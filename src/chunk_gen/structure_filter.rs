use crate::{commons::rng::Rng, resources::{object_tile::ObjectTileId, resources::resources, tile::TileId}, Coord2};

pub(crate) trait StructureFilter {

    fn filter(&mut self, position: Coord2, ground: &TileId, object: Option<ObjectTileId>) -> Option<TileId>;

}


pub(crate) struct NoopFilter {}

impl StructureFilter for NoopFilter {

    fn filter(&mut self, _position: Coord2, _ground: &TileId, _object: Option<ObjectTileId>) -> Option<TileId> {
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

    fn filter(&mut self, _position: Coord2, ground: &TileId, object: Option<ObjectTileId>) -> Option<TileId> {
        let resources = resources();
        let grass = resources.tiles.id_of("tile:grass");

        // TODO: Bring decay chance from resources
                
        if let Some(object) = object {
            if object == resources.object_tiles.id_of("obj:wall") {
                if self.rng.rand_chance((self.age as f32 / 150.).clamp(0.0, 0.9)) {
                    return Some(grass)
                } else {
                    return None;
                }
            }
            if object == resources.object_tiles.id_of("obj:bed") || object == resources.object_tiles.id_of("obj:table") || object == resources.object_tiles.id_of("obj:stool") || object == resources.object_tiles.id_of("obj:barrel") || object == resources.object_tiles.id_of("obj:chair") || object == resources.object_tiles.id_of("obj:tent") {
                return Some(grass)
            }
        }

        let ground = *ground;
        if ground == resources.tiles.id_of("tile:floor") || ground == resources.tiles.id_of("tile:cobblestone") || ground == resources.tiles.id_of("tile:carpet_red") {
            if self.rng.rand_chance((self.age as f32 / 50.).clamp(0.0, 0.9)) {
                return Some(grass)
            }
        }
        None
    }

}