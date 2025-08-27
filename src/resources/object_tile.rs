use serde::{Deserialize, Serialize};

use crate::engine::tilemap::Tile;

// TODO(0xtBbih5): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct ObjectTileId(usize);
impl crate::commons::id_vec::Id for ObjectTileId {
    fn new(id: usize) -> Self {
        Self(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub(crate) struct ObjectTile {
    pub(crate) tile: Tile,
    pub(crate) casts_shadow: bool,
    pub(crate) blocks_movement: bool
}

impl ObjectTile {
    pub(crate) fn new(tile: Tile, blocks_movement: bool) -> Self {
        Self { tile, blocks_movement, casts_shadow: false }
    }

    pub(crate) fn with_shadow(mut self) -> Self {
        self.casts_shadow = true;
        return self
    }
}