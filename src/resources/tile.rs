use image::{DynamicImage, ImageReader};

use crate::engine::audio::SoundEffect;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
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
    pub(crate) tileset_image: DynamicImage
}

impl Tile {
    pub(crate) fn new(tile_layer: u16, tileset_path: &str) -> Tile {
        Self { step_sound_effect: None, tileset_image: ImageReader::open(tileset_path).unwrap().decode().unwrap(), tile_layer }
    }
}