use serde::{Deserialize, Serialize};

use crate::engine::{assets::ImageSheetAsset, audio::SoundEffect, geometry::Size2D};

// TODO(0xtBbih5): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct TileId(usize);
impl crate::commons::id_vec::Id for TileId {
    fn new(id: usize) -> Self {
        TileId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub(crate) struct Tile {
    pub(crate) step_sound_effect: Option<SoundEffect>,
    pub(crate) tile_layer: u16,
    pub(crate) tileset_image: ImageSheetAsset
}

impl Tile {
    pub(crate) fn new(tile_layer: u16, tileset_path: &str) -> Tile {
        Self { step_sound_effect: None, tileset_image: ImageSheetAsset::new(tileset_path, Size2D(24, 24)), tile_layer }
    }
}