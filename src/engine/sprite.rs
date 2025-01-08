use std::fmt::Display;

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

pub struct Sprite {
    pub path: String,
    pub size: (u32, u32),
    pub texture: Texture
}

impl Sprite {

    pub fn new(path: impl Display) -> Sprite {
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let path = format!("./assets/sprites/{}", path.to_string());
        let icon = ImageReader::open(&path).unwrap().decode().unwrap();
        Self {
            path,
            size: (icon.width(), icon.height()),
            texture: Texture::from_image(&icon.to_rgba8(), &settings)
        }
    }

}