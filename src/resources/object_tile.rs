use crate::engine::tilemap::Tile;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
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
    pub(crate) blocks_movement: bool
}

impl ObjectTile {
    pub(crate) fn new(tile: Tile, blocks_movement: bool) -> Self {
        Self { tile, blocks_movement }
    }
}