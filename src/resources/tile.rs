use image::{DynamicImage, ImageReader};

use crate::engine::audio::SoundEffect;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct TileId(usize);
impl crate::commons::id_vec::Id for TileId {
    fn new(id: usize) -> Self {
        TileId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub struct Tile {
    pub step_sound_effect: Option<SoundEffect>,
    pub tile_layer: u16,
    pub tileset_image: DynamicImage
}

impl Tile {
    pub fn new(tile_layer: u16, tileset_path: &str) -> Tile {
        Self { step_sound_effect: None, tileset_image: ImageReader::open(tileset_path).unwrap().decode().unwrap(), tile_layer }
    }
}