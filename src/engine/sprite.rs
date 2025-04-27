use std::fmt::Display;

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

pub(crate) struct Sprite {
    pub(crate) texture: Texture
}

impl Sprite {

    pub(crate) fn new(path: impl Display) -> Sprite {
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let path = format!("./assets/sprites/{}", path.to_string());
        let icon = ImageReader::open(&path).unwrap().decode().unwrap();
        Self {
            texture: Texture::from_image(&icon.to_rgba8(), &settings)
        }
    }

}